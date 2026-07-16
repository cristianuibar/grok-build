//! Codex device-code login (proprietary deviceauth API — not RFC 8628).
//!
//! Flow (openai/codex `device_code_auth.rs`):
//! 1. POST `{issuer}/api/accounts/deviceauth/usercode` → `device_auth_id` + `user_code`
//! 2. Poll POST `{issuer}/api/accounts/deviceauth/token` until authorization_code
//! 3. Exchange code at `{issuer}/oauth/token` with redirect `{issuer}/deviceauth/callback`
//! 4. Persist only `providers.codex`

use std::path::Path;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use super::browser::{
    CodexLoginError, CodexTokenResponse, exchange_codex_authorization_code, persist_codex_tokens,
};
use super::{CODEX_CLIENT_ID, CODEX_ISSUER};

const DEFAULT_POLL_INTERVAL_SECS: u64 = 5;
const SLOW_DOWN_INCREMENT_SECS: u64 = 5;
const MAX_WAIT: Duration = Duration::from_secs(15 * 60);

/// Device usercode endpoint under the accounts API.
pub fn codex_device_usercode_url(issuer: &str) -> String {
    format!(
        "{}/api/accounts/deviceauth/usercode",
        issuer.trim_end_matches('/')
    )
}

/// Device token poll endpoint under the accounts API.
pub fn codex_device_token_url(issuer: &str) -> String {
    format!(
        "{}/api/accounts/deviceauth/token",
        issuer.trim_end_matches('/')
    )
}

/// Browser verification URL shown to the user.
pub fn codex_device_verify_url(issuer: &str) -> String {
    format!("{}/codex/device", issuer.trim_end_matches('/'))
}

/// Redirect used for device authorization-code exchange.
fn device_exchange_redirect_uri(issuer: &str) -> String {
    format!("{}/deviceauth/callback", issuer.trim_end_matches('/'))
}

#[derive(Debug, Clone)]
pub struct CodexDeviceCode {
    pub verification_url: String,
    pub user_code: String,
    device_auth_id: String,
    interval: u64,
}

#[derive(Serialize)]
struct UserCodeReq {
    client_id: String,
}

#[derive(Deserialize)]
struct UserCodeResp {
    device_auth_id: String,
    #[serde(alias = "user_code", alias = "usercode")]
    user_code: String,
    #[serde(default)]
    interval: Option<serde_json::Value>,
}

#[derive(Serialize)]
struct TokenPollReq {
    device_auth_id: String,
    user_code: String,
}

#[derive(Deserialize)]
struct CodeSuccessResp {
    authorization_code: String,
    #[serde(default)]
    code_challenge: Option<String>,
    code_verifier: String,
}

#[derive(Deserialize)]
struct PollErrorBody {
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    error_description: Option<String>,
}

fn parse_interval(value: Option<serde_json::Value>) -> u64 {
    match value {
        Some(serde_json::Value::Number(n)) => n.as_u64().unwrap_or(DEFAULT_POLL_INTERVAL_SECS),
        Some(serde_json::Value::String(s)) => s
            .trim()
            .parse()
            .unwrap_or(DEFAULT_POLL_INTERVAL_SECS),
        _ => DEFAULT_POLL_INTERVAL_SECS,
    }
    .max(1)
}

/// Request a device code from the Codex deviceauth API.
pub async fn request_codex_device_code(
    issuer: &str,
    client_id: &str,
) -> Result<CodexDeviceCode, CodexLoginError> {
    let url = codex_device_usercode_url(issuer);
    let body = UserCodeReq {
        client_id: client_id.to_owned(),
    };
    let resp = crate::http::shared_client()
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&body)
        .timeout(Duration::from_secs(30))
        .send()
        .await
        .map_err(|e| CodexLoginError::Other(format!("device usercode request failed: {e}")))?;

    if resp.status().as_u16() == 404 {
        return Err(CodexLoginError::Other(
            "device code login is not enabled for this Codex server. Use browser login or --oauth."
                .into(),
        ));
    }
    if !resp.status().is_success() {
        let status = resp.status();
        return Err(CodexLoginError::Other(format!(
            "device code request failed with HTTP {status}"
        )));
    }

    let uc: UserCodeResp = resp
        .json()
        .await
        .map_err(|e| CodexLoginError::Other(format!("invalid usercode response: {e}")))?;

    // Defend against control characters from a malicious issuer.
    if !uc
        .user_code
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-')
    {
        return Err(CodexLoginError::Other(
            "server returned invalid user_code format".into(),
        ));
    }

    Ok(CodexDeviceCode {
        verification_url: codex_device_verify_url(issuer),
        user_code: uc.user_code,
        device_auth_id: uc.device_auth_id,
        interval: parse_interval(uc.interval),
    })
}

/// Poll until authorization code, then exchange and persist to `auth_file`.
///
/// Handles `authorization_pending` / 403/404 as pending, `slow_down`,
/// `access_denied` / `expired_token` as terminal (no write).
pub async fn complete_codex_device_login(
    issuer: &str,
    client_id: &str,
    device_code: CodexDeviceCode,
    auth_file: &Path,
) -> Result<crate::auth::GrokAuth, CodexLoginError> {
    let code_resp = poll_for_authorization_code(issuer, &device_code).await?;
    let redirect_uri = device_exchange_redirect_uri(issuer);
    let tokens = exchange_codex_authorization_code(
        issuer,
        client_id,
        &redirect_uri,
        &code_resp.authorization_code,
        &code_resp.code_verifier,
    )
    .await?;
    // Only persist after successful exchange.
    let _ = code_resp.code_challenge; // IdP-supplied; exchange uses code_verifier
    persist_codex_tokens(auth_file, &tokens)
}

async fn poll_for_authorization_code(
    issuer: &str,
    device_code: &CodexDeviceCode,
) -> Result<CodeSuccessResp, CodexLoginError> {
    let url = codex_device_token_url(issuer);
    let mut poll_interval = Duration::from_secs(device_code.interval.max(1));
    let start = Instant::now();

    loop {
        if start.elapsed() >= MAX_WAIT {
            return Err(CodexLoginError::Other(
                "device auth timed out after 15 minutes".into(),
            ));
        }

        tokio::time::sleep(poll_interval).await;

        let body = TokenPollReq {
            device_auth_id: device_code.device_auth_id.clone(),
            user_code: device_code.user_code.clone(),
        };
        let resp = crate::http::shared_client()
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&body)
            .timeout(Duration::from_secs(30))
            .send()
            .await
            .map_err(|e| CodexLoginError::Other(format!("device token poll failed: {e}")))?;

        let status = resp.status();
        if status.is_success() {
            return resp.json::<CodeSuccessResp>().await.map_err(|e| {
                CodexLoginError::TokenExchange(format!("invalid device token success body: {e}"))
            });
        }

        // Parse optional OAuth-style error body (when present).
        let text = resp.text().await.unwrap_or_default();
        let err_body: Option<PollErrorBody> = serde_json::from_str(&text).ok();
        let error_code = err_body
            .as_ref()
            .and_then(|b| b.error.as_deref())
            .unwrap_or("");

        match error_code {
            "authorization_pending" => continue,
            "slow_down" => {
                poll_interval += Duration::from_secs(SLOW_DOWN_INCREMENT_SECS);
                continue;
            }
            "access_denied" => {
                return Err(CodexLoginError::OAuthError(
                    "Authorization denied. The user rejected the request.".into(),
                ));
            }
            "expired_token" => {
                return Err(CodexLoginError::Other(
                    "Device code expired. Run `bum login --provider codex --device-auth` again."
                        .into(),
                ));
            }
            _ => {
                // openai/codex treats 403/404 as pending until timeout.
                if status.as_u16() == 403 || status.as_u16() == 404 {
                    continue;
                }
                // Do not include body (may leak tokens).
                return Err(CodexLoginError::Other(format!(
                    "device auth failed with HTTP {status}"
                )));
            }
        }
    }
}

/// CLI device login against production issuer.
pub async fn run_codex_device_login(
    auth_file: &Path,
) -> Result<crate::auth::GrokAuth, CodexLoginError> {
    run_codex_device_login_with_base(auth_file, CODEX_ISSUER, CODEX_CLIENT_ID).await
}

/// Device login with injectable issuer base (mock IdP tests).
pub async fn run_codex_device_login_with_base(
    auth_file: &Path,
    issuer: &str,
    client_id: &str,
) -> Result<crate::auth::GrokAuth, CodexLoginError> {
    eprintln!("Signing in with Codex...");
    let device_code = request_codex_device_code(issuer, client_id).await?;

    eprintln!("To sign in, open this URL in your browser:");
    eprintln!();
    eprintln!("  {}", device_code.verification_url);
    eprintln!();
    eprintln!("Then enter this code:");
    eprintln!();
    eprintln!("  {}", device_code.user_code);
    eprintln!();
    eprintln!(
        "\x1b[90mOnly continue with a code you requested. \
         Don't share it with anyone.\x1b[0m"
    );
    eprintln!();
    eprintln!("Waiting for authorization...");

    let _ = webbrowser::open(&device_code.verification_url);
    complete_codex_device_login(issuer, client_id, device_code, auth_file).await
}

/// Multi-step device poll harness for tests: drives pending → slow_down → terminal.
///
/// Returns the final result without printing CLI copy. Does not write on denied/expired.
pub async fn run_codex_device_poll_only_with_base(
    auth_file: &Path,
    issuer: &str,
    client_id: &str,
) -> Result<crate::auth::GrokAuth, CodexLoginError> {
    let device_code = request_codex_device_code(issuer, client_id).await?;
    complete_codex_device_login(issuer, client_id, device_code, auth_file).await
}
