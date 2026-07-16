//! Pure-data Codex refresh-token exchange.
//!
//! Talks to the fixed ChatGPT token endpoint (`{issuer}/oauth/token`) and
//! returns classified outcomes **without** mutating `auth.json` / provider
//! stores. Outer `ensure_fresh_codex_auth` owns lock, persist, and permanent
//! clear (TokenRefresher purity contract).

use super::browser::{CodexTokenResponse, grok_auth_from_codex_tokens};
use super::{CODEX_CLIENT_ID, CODEX_ISSUER};
use crate::auth::error::RefreshTokenFailedReason;
use crate::auth::GrokAuth;

/// Outcome of a pure Codex token refresh (no storage mutations).
#[derive(Debug)]
pub enum CodexRefreshResult {
    /// Fresh tokens obtained (identity fields preserved when IdP omits them).
    Success(Box<GrokAuth>),
    /// Terminal IdP failure already classified.
    TerminalError { reason: RefreshTokenFailedReason },
    /// Non-terminal failure (network, 5xx, unknown error body, etc.).
    Failed,
}

/// Classify an OAuth2 `error` code as a terminal refresh failure.
pub fn classify_terminal(error_code: &str) -> Option<RefreshTokenFailedReason> {
    match error_code {
        "invalid_grant" => Some(RefreshTokenFailedReason::RefreshTokenRejected),
        "invalid_client" => Some(RefreshTokenFailedReason::ClientRejected),
        _ => None,
    }
}

/// Default token URL for product Codex OAuth (`https://auth.openai.com/oauth/token`).
pub fn codex_token_url(issuer: &str) -> String {
    format!("{}/oauth/token", issuer.trim_end_matches('/'))
}

/// Merge a token-response into the prior credential, preserving identity when
/// the IdP omits rotation or account claims (mirror `oidc/refresh.rs`).
///
/// Rules:
/// - omitted `refresh_token` → keep prior refresh_token
/// - omitted / empty account claims from JWTs → keep prior organization_id,
///   email, user_id
/// - always preserve prior issuer / client_id (product constants if prior empty)
pub fn merge_codex_refresh_response(prev: &GrokAuth, tokens: &CodexTokenResponse) -> GrokAuth {
    let mut next = grok_auth_from_codex_tokens(tokens);

    // Keep old RT if IdP did not rotate it.
    if next.refresh_token.is_none() {
        next.refresh_token = prev.refresh_token.clone();
    }

    // Preserve identity claims when the new tokens don't carry them.
    if next.organization_id.is_none() {
        next.organization_id = prev.organization_id.clone();
    }
    if next.email.is_none() {
        next.email = prev.email.clone();
    }
    if next.user_id.is_empty() {
        next.user_id = prev.user_id.clone();
    }

    // Issuer / client_id always come from the prior session (or product defaults).
    next.oidc_issuer = prev
        .oidc_issuer
        .clone()
        .or_else(|| Some(CODEX_ISSUER.to_owned()));
    next.oidc_client_id = prev
        .oidc_client_id
        .clone()
        .or_else(|| Some(CODEX_CLIENT_ID.to_owned()));

    // Auth mode stays OIDC session.
    next.auth_mode = prev.auth_mode.clone();

    next
}

/// Exchange `refresh_token` at the Codex token endpoint. Pure data — no disk I/O.
///
/// `token_url_override` lets tests point at a mock HTTP server without rewriting
/// issuer constants on the stored credential.
pub async fn codex_token_exchange(
    auth: &GrokAuth,
    token_url_override: Option<&str>,
) -> CodexRefreshResult {
    let Some(refresh_tok) = auth.refresh_token.as_ref().filter(|t| !t.is_empty()) else {
        tracing::debug!("codex refresh: missing refresh_token");
        return CodexRefreshResult::Failed;
    };
    let client_id = auth
        .oidc_client_id
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or(CODEX_CLIENT_ID);
    let issuer = auth
        .oidc_issuer
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or(CODEX_ISSUER);
    let token_url = token_url_override
        .map(str::to_owned)
        .unwrap_or_else(|| codex_token_url(issuer));

    tracing::debug!(
        token_url_host_len = token_url.len(),
        client_id_len = client_id.len(),
        has_rt = true,
        "codex try_refresh_pure enter"
    );

    let resp = match crate::http::shared_client()
        .post(&token_url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&[
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_tok.as_str()),
            ("client_id", client_id),
        ])
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!(
                error = %e,
                // lengths only — never RT/AT material
                error_len = e.to_string().len(),
                "codex: token refresh request failed"
            );
            return CodexRefreshResult::Failed;
        }
    };

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        if let Some(error_code) = serde_json::from_str::<serde_json::Value>(&body)
            .ok()
            .and_then(|v| v.get("error")?.as_str().map(str::to_owned))
            && let Some(reason) = classify_terminal(&error_code)
        {
            tracing::warn!(
                error_code = %error_code,
                http_status = %status.as_u16(),
                body_len = body.len(),
                "codex: terminal token refresh error"
            );
            return CodexRefreshResult::TerminalError { reason };
        }
        tracing::warn!(
            http_status = %status.as_u16(),
            body_len = body.len(),
            "codex: token refresh failed (transient)"
        );
        return CodexRefreshResult::Failed;
    }

    let tokens = match resp.json::<CodexTokenResponse>().await {
        Ok(t) if !t.access_token.is_empty() => t,
        Ok(_) => {
            tracing::warn!("codex: token response missing access_token");
            return CodexRefreshResult::Failed;
        }
        Err(e) => {
            tracing::warn!(
                error_len = e.to_string().len(),
                "codex: failed to parse token response"
            );
            return CodexRefreshResult::Failed;
        }
    };

    let new_auth = merge_codex_refresh_response(auth, &tokens);
    let idp_rotated = tokens.refresh_token.is_some();
    tracing::debug!(
        idp_rotated,
        key_prefix = crate::auth::token_suffix(&new_auth.key),
        "codex try_refresh_pure token obtained"
    );
    CodexRefreshResult::Success(Box::new(new_auth))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::AuthMode;
    use chrono::{Duration, Utc};

    fn prev_auth() -> GrokAuth {
        GrokAuth {
            key: "old-access".into(),
            auth_mode: AuthMode::Oidc,
            create_time: Utc::now(),
            user_id: "user-prev".into(),
            email: Some("prev@example.test".into()),
            organization_id: Some("acct-prev".into()),
            expires_at: Some(Utc::now() + Duration::hours(1)),
            refresh_token: Some("rt-prev".into()),
            oidc_issuer: Some(CODEX_ISSUER.into()),
            oidc_client_id: Some(CODEX_CLIENT_ID.into()),
            ..Default::default()
        }
    }

    #[test]
    fn merge_preserves_identity_when_response_omits_rt_and_claims() {
        let prev = prev_auth();
        let tokens = CodexTokenResponse {
            access_token: "new-access".into(),
            refresh_token: None,
            id_token: None,
            expires_in: Some(3600),
        };
        let merged = merge_codex_refresh_response(&prev, &tokens);
        assert_eq!(merged.key, "new-access");
        assert_eq!(merged.refresh_token.as_deref(), Some("rt-prev"));
        assert_eq!(merged.organization_id.as_deref(), Some("acct-prev"));
        assert_eq!(merged.email.as_deref(), Some("prev@example.test"));
        assert_eq!(merged.user_id, "user-prev");
        assert_eq!(merged.oidc_issuer.as_deref(), Some(CODEX_ISSUER));
        assert_eq!(merged.oidc_client_id.as_deref(), Some(CODEX_CLIENT_ID));
    }

    #[test]
    fn classify_invalid_grant_is_permanent() {
        assert_eq!(
            classify_terminal("invalid_grant"),
            Some(RefreshTokenFailedReason::RefreshTokenRejected)
        );
        assert!(classify_terminal("server_error").is_none());
    }
}
