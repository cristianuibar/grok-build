//! Codex browser PKCE loopback login (ports 1455→1457, localhost redirect).

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use axum::{
    Router,
    extract::{Query, State},
    http::StatusCode,
    response::Html,
    routing::get,
};
use chrono::Utc;
use serde::Deserialize;
use tokio::net::TcpListener;
use tokio::sync::oneshot;

use super::claims::{decode_codex_claims, resolve_expires_at};
use super::{
    CODEX_AUTH_SCOPE, CODEX_CLIENT_ID, CODEX_FALLBACK_PORT, CODEX_ISSUER, CODEX_ORIGINATOR,
    CODEX_PREFERRED_PORT, CODEX_SCOPES,
};
use crate::auth::oidc::protocol::{Pkce, generate_pkce};
use crate::auth::{
    AuthMode, AuthProvider, GrokAuth, ProviderStoreMutation, mutate_provider_store_or_prune,
};

/// Maximum time to wait for the OAuth callback.
const AUTH_CALLBACK_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(600);

/// Errors from Codex browser / inject login paths.
#[derive(Debug, thiserror::Error)]
pub enum CodexLoginError {
    #[error("OAuth state mismatch — sign-in was not completed")]
    StateMismatch,
    #[error("OAuth error from provider: {0}")]
    OAuthError(String),
    #[error("Missing authorization code")]
    MissingCode,
    #[error("Token exchange failed: {0}")]
    TokenExchange(String),
    #[error(
        "Could not bind OAuth callback on localhost:{preferred} (or {fallback}). \
         Free the port or use --device-auth."
    )]
    BindFailed { preferred: u16, fallback: u16 },
    #[error("Login timed out after 10 minutes. Please try again.")]
    Timeout,
    #[error("Failed to persist Codex credentials: {0}")]
    Persist(String),
    #[error("{0}")]
    Other(String),
}

/// Token endpoint JSON (authorization_code or device exchange).
#[derive(Debug, Clone, Deserialize)]
pub struct CodexTokenResponse {
    pub access_token: String,
    #[serde(default)]
    pub refresh_token: Option<String>,
    #[serde(default)]
    pub id_token: Option<String>,
    #[serde(default)]
    pub expires_in: Option<u64>,
}

/// PKCE + state session used to build the authorize URL (unit-testable).
#[derive(Debug, Clone)]
pub struct CodexAuthorizeSession {
    pub code_verifier: String,
    pub code_challenge: String,
    pub state: String,
    pub port: u16,
    pub authorize_url: String,
    pub redirect_uri: String,
}

impl CodexAuthorizeSession {
    /// Build a new session for the preferred port (or an explicit port for tests).
    pub fn new(port: u16) -> Self {
        let pkce = generate_pkce();
        let state = generate_oauth_state();
        Self::from_parts(port, pkce, state)
    }

    pub(crate) fn from_parts(port: u16, pkce: Pkce, state: String) -> Self {
        let redirect_uri = codex_redirect_uri(port);
        let authorize_url = build_codex_authorize_url(
            CODEX_ISSUER,
            CODEX_CLIENT_ID,
            &redirect_uri,
            &pkce.code_challenge,
            &state,
        );
        Self {
            code_verifier: pkce.code_verifier,
            code_challenge: pkce.code_challenge,
            state,
            port,
            authorize_url,
            redirect_uri,
        }
    }
}

/// Redirect URI host is **`localhost`** (not 127.0.0.1) — required by Codex allow-list.
pub fn codex_redirect_uri(port: u16) -> String {
    format!("http://localhost:{port}/auth/callback")
}

/// Build the ChatGPT authorize URL (PKCE S256 + originator=bum).
pub fn build_codex_authorize_url(
    issuer: &str,
    client_id: &str,
    redirect_uri: &str,
    code_challenge: &str,
    state: &str,
) -> String {
    let issuer = issuer.trim_end_matches('/');
    let mut pairs = vec![
        ("response_type", "code".to_owned()),
        ("client_id", client_id.to_owned()),
        ("redirect_uri", redirect_uri.to_owned()),
        ("scope", CODEX_SCOPES.to_owned()),
        ("code_challenge", code_challenge.to_owned()),
        ("code_challenge_method", "S256".to_owned()),
        ("id_token_add_organizations", "true".to_owned()),
        ("codex_cli_simplified_flow", "true".to_owned()),
        ("state", state.to_owned()),
        ("originator", CODEX_ORIGINATOR.to_owned()),
    ];
    // Keep stable key order for tests / debugging (response_type first).
    let qs = pairs
        .drain(..)
        .map(|(k, v)| format!("{k}={}", urlencoding::encode(&v)))
        .collect::<Vec<_>>()
        .join("&");
    format!("{issuer}/oauth/authorize?{qs}")
}

fn generate_oauth_state() -> String {
    use base64::Engine;
    let bytes: [u8; 32] = rand::random();
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

/// Validate OAuth `state` (cryptographic equality). Fail closed — no side effects.
pub fn validate_codex_oauth_state(expected: &str, received: &str) -> Result<(), CodexLoginError> {
    if expected.is_empty() || received != expected {
        return Err(CodexLoginError::StateMismatch);
    }
    Ok(())
}

/// Map token response + JWT claims into [`GrokAuth`] for the Codex slot.
pub fn grok_auth_from_codex_tokens(tokens: &CodexTokenResponse) -> GrokAuth {
    let now = Utc::now();
    let id_claims = tokens
        .id_token
        .as_deref()
        .map(decode_codex_claims)
        .unwrap_or_default();
    let access_claims = decode_codex_claims(&tokens.access_token);
    let email = id_claims.email.or(access_claims.email);
    let user_id = id_claims
        .user_id
        .or(access_claims.user_id)
        .unwrap_or_default();
    let organization_id = id_claims
        .chatgpt_account_id
        .or(access_claims.chatgpt_account_id);
    let expires_at = resolve_expires_at(
        now,
        tokens.expires_in,
        &tokens.access_token,
        tokens.id_token.as_deref(),
    );
    GrokAuth {
        key: tokens.access_token.clone(),
        auth_mode: AuthMode::Oidc,
        create_time: now,
        user_id,
        email,
        organization_id,
        refresh_token: tokens.refresh_token.clone(),
        expires_at: Some(expires_at),
        oidc_issuer: Some(CODEX_ISSUER.to_owned()),
        oidc_client_id: Some(CODEX_CLIENT_ID.to_owned()),
        ..Default::default()
    }
}

/// Persist Codex tokens under `providers.codex` only (sibling slots preserved).
pub fn persist_codex_tokens(
    auth_file: &Path,
    tokens: &CodexTokenResponse,
) -> Result<GrokAuth, CodexLoginError> {
    let auth = grok_auth_from_codex_tokens(tokens);
    let to_store = auth.clone();
    let outcome = mutate_provider_store_or_prune(auth_file, AuthProvider::Codex, move |store| {
        store.insert(CODEX_AUTH_SCOPE.to_owned(), to_store);
    })
    .map_err(|e| CodexLoginError::Persist(e.to_string()))?;
    match outcome {
        ProviderStoreMutation::DocumentWritten | ProviderStoreMutation::Unchanged => {}
        ProviderStoreMutation::FileDeleted => {
            return Err(CodexLoginError::Persist(
                "unexpected file delete after Codex persist".into(),
            ));
        }
    }
    Ok(auth)
}

/// Apply OAuth callback outcome without network: state / error / optional tokens.
///
/// Used by tests and by the browser handler after exchange. **Never writes** on
/// state mismatch, OAuth error, or missing tokens.
pub fn apply_codex_oauth_callback(
    auth_file: &Path,
    expected_state: &str,
    received_state: Option<&str>,
    oauth_error: Option<&str>,
    tokens: Option<&CodexTokenResponse>,
) -> Result<GrokAuth, CodexLoginError> {
    let received = received_state.unwrap_or("");
    validate_codex_oauth_state(expected_state, received)?;
    if let Some(err) = oauth_error.filter(|e| !e.is_empty()) {
        return Err(CodexLoginError::OAuthError(err.to_owned()));
    }
    let Some(tokens) = tokens else {
        return Err(CodexLoginError::TokenExchange(
            "no tokens available after callback".into(),
        ));
    };
    if tokens.access_token.is_empty() {
        return Err(CodexLoginError::TokenExchange("empty access_token".into()));
    }
    persist_codex_tokens(auth_file, tokens)
}

/// Exchange authorization code at `{issuer}/oauth/token`.
pub async fn exchange_codex_authorization_code(
    issuer: &str,
    client_id: &str,
    redirect_uri: &str,
    code: &str,
    code_verifier: &str,
) -> Result<CodexTokenResponse, CodexLoginError> {
    let token_endpoint = format!("{}/oauth/token", issuer.trim_end_matches('/'));
    let resp = crate::http::shared_client()
        .post(&token_endpoint)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", redirect_uri),
            ("client_id", client_id),
            ("code_verifier", code_verifier),
        ])
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await
        .map_err(|e| CodexLoginError::TokenExchange(e.to_string()))?;

    if !resp.status().is_success() {
        let status = resp.status();
        // Do not surface response body (may contain sensitive detail).
        return Err(CodexLoginError::TokenExchange(format!(
            "token endpoint returned HTTP {status}"
        )));
    }

    resp.json::<CodexTokenResponse>()
        .await
        .map_err(|e| CodexLoginError::TokenExchange(e.to_string()))
}

#[derive(Clone)]
struct CallbackState {
    expected_state: String,
    result_tx: Arc<tokio::sync::Mutex<Option<oneshot::Sender<CallbackOutcome>>>>,
}

enum CallbackOutcome {
    Success { code: String },
    OAuthError(String),
    StateMismatch,
    MissingCode,
}

/// Interactive browser PKCE login → persist `providers.codex`.
pub async fn run_codex_browser_login(auth_file: &Path) -> Result<GrokAuth, CodexLoginError> {
    eprintln!("Signing in with Codex...");
    let (listener, port) = bind_codex_loopback().await?;
    let session = CodexAuthorizeSession::new(port);

    eprintln!("Open this URL to sign in:");
    eprintln!("  {}", session.authorize_url);

    let _ = webbrowser::open(&session.authorize_url);

    let (tx, rx) = oneshot::channel();
    let cb_state = CallbackState {
        expected_state: session.state.clone(),
        result_tx: Arc::new(tokio::sync::Mutex::new(Some(tx))),
    };

    let app = Router::new()
        .route("/auth/callback", get(handle_callback))
        .with_state(cb_state);

    let server = axum::serve(listener, app);
    let server_handle = tokio::spawn(async move {
        // Serve until the first callback completes or timeout cancels us.
        let _ = server.await;
    });

    let outcome = tokio::time::timeout(AUTH_CALLBACK_TIMEOUT, rx)
        .await
        .map_err(|_| CodexLoginError::Timeout)?
        .map_err(|_| CodexLoginError::Other("callback channel closed".into()))?;

    // Drop the server task (listener closes with process).
    server_handle.abort();

    match outcome {
        CallbackOutcome::StateMismatch => Err(CodexLoginError::StateMismatch),
        CallbackOutcome::OAuthError(e) => Err(CodexLoginError::OAuthError(e)),
        CallbackOutcome::MissingCode => Err(CodexLoginError::MissingCode),
        CallbackOutcome::Success { code } => {
            let tokens = exchange_codex_authorization_code(
                CODEX_ISSUER,
                CODEX_CLIENT_ID,
                &session.redirect_uri,
                &code,
                &session.code_verifier,
            )
            .await?;
            // Exchange failure already returned Err — only persist on success.
            persist_codex_tokens(auth_file, &tokens)
        }
    }
}

async fn bind_codex_loopback() -> Result<(TcpListener, u16), CodexLoginError> {
    for port in [CODEX_PREFERRED_PORT, CODEX_FALLBACK_PORT] {
        match TcpListener::bind(("127.0.0.1", port)).await {
            Ok(listener) => return Ok((listener, port)),
            Err(e) => {
                tracing::debug!(port, error = %e, "codex: bind failed, trying next port");
            }
        }
    }
    Err(CodexLoginError::BindFailed {
        preferred: CODEX_PREFERRED_PORT,
        fallback: CODEX_FALLBACK_PORT,
    })
}

async fn handle_callback(
    State(state): State<CallbackState>,
    Query(params): Query<HashMap<String, String>>,
) -> (StatusCode, Html<String>) {
    let received_state = params.get("state").map(String::as_str).unwrap_or("");
    let outcome = if received_state != state.expected_state {
        CallbackOutcome::StateMismatch
    } else if let Some(err) = params.get("error").filter(|e| !e.is_empty()) {
        let desc = params
            .get("error_description")
            .map(|s| s.as_str())
            .filter(|s| !s.is_empty());
        let msg = match desc {
            Some(d) => format!("{err}: {d}"),
            None => err.clone(),
        };
        CallbackOutcome::OAuthError(msg)
    } else if let Some(code) = params.get("code").filter(|c| !c.is_empty()) {
        CallbackOutcome::Success {
            code: code.clone(),
        }
    } else {
        CallbackOutcome::MissingCode
    };

    // Soften success HTML: token exchange + persist happen after this response
    // (WR-03). Never claim "Signed in" before exchange completes.
    let (status, title, message) = match &outcome {
        CallbackOutcome::Success { .. } => (
            StatusCode::OK,
            "Authorization received",
            "Finish sign-in in the terminal. You can close this window after the CLI confirms success.",
        ),
        CallbackOutcome::StateMismatch => (
            StatusCode::BAD_REQUEST,
            "Sign-in failed",
            "State mismatch. Please try again from the terminal.",
        ),
        CallbackOutcome::OAuthError(_) => (
            StatusCode::BAD_REQUEST,
            "Sign-in failed",
            "Authorization was denied or failed. Return to the terminal and try again.",
        ),
        CallbackOutcome::MissingCode => (
            StatusCode::BAD_REQUEST,
            "Sign-in failed",
            "Missing authorization code. Please try again from the terminal.",
        ),
    };

    if let Some(tx) = state.result_tx.lock().await.take() {
        let _ = tx.send(outcome);
    }

    (
        status,
        Html(callback_page_html(title, message)),
    )
}

fn callback_page_html(title: &str, message: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en"><head><meta charset="utf-8"/><title>{title}</title>
<style>
  body{{font-family:system-ui,sans-serif;display:flex;align-items:center;justify-content:center;
    min-height:100vh;background:#0a0a0a;color:#e5e5e5;margin:0}}
  .card{{text-align:center;padding:48px}}
  h1{{font-size:18px}} p{{font-size:14px;color:#a3a3a3}}
</style></head><body><div class="card"><h1>{title}</h1><p>{message}</p></div></body></html>"#
    )
}
