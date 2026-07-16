//! Data-only Codex [`TokenRefresher`].
//!
//! Returns [`RefreshOutcome`] only — never mutates provider stores, never
//! clears slots, never touches xAI `AuthManager`. Outer `ensure_fresh_codex_auth`
//! owns lock-held re-read, sibling adopt, persist, and permanent clear.

use crate::auth::codex::{codex_token_exchange, CodexRefreshResult};
use crate::auth::manager::RefreshReason;
use crate::auth::GrokAuth;

use super::{RefreshOutcome, TokenRefresher};

/// Pure Codex refresh handle: exchanges the bound credential's refresh_token
/// and returns a classified outcome.
pub(crate) struct CodexRefresher {
    /// Credential whose refresh_token will be spent (snapshot under lock).
    auth: GrokAuth,
    /// Optional full token URL override for mock IdP tests.
    token_url_override: Option<String>,
}

impl CodexRefresher {
    pub(crate) fn new(auth: GrokAuth) -> Self {
        Self {
            auth,
            token_url_override: None,
        }
    }

    pub(crate) fn with_token_url(mut self, url: impl Into<String>) -> Self {
        self.token_url_override = Some(url.into());
        self
    }

    /// Credential this refresher will send (for tried_key on permanent fail).
    pub(crate) fn tried_key(&self) -> &str {
        &self.auth.key
    }
}

#[async_trait::async_trait]
impl TokenRefresher for CodexRefresher {
    async fn refresh(&self, reason: RefreshReason) -> RefreshOutcome {
        tracing::debug!(
            reason = ?reason,
            has_rt = self.auth.refresh_token.is_some(),
            key_prefix = crate::auth::token_suffix(&self.auth.key),
            "codex refresher enter"
        );

        match codex_token_exchange(&self.auth, self.token_url_override.as_deref()).await {
            CodexRefreshResult::Success(new_auth) => RefreshOutcome::Success(new_auth),
            CodexRefreshResult::TerminalError { reason } => {
                RefreshOutcome::permanent(reason, Some(self.auth.key.clone()))
            }
            CodexRefreshResult::Failed => RefreshOutcome::transient("Codex token refresh failed"),
        }
    }
}

/// Map a pure [`CodexRefreshResult`] into [`RefreshOutcome`] (shared helper).
#[allow(dead_code)]
pub(crate) fn outcome_from_codex_result(
    result: CodexRefreshResult,
    tried_key: Option<String>,
) -> RefreshOutcome {
    match result {
        CodexRefreshResult::Success(auth) => RefreshOutcome::Success(auth),
        CodexRefreshResult::TerminalError { reason } => {
            RefreshOutcome::permanent(reason, tried_key)
        }
        CodexRefreshResult::Failed => RefreshOutcome::transient("Codex token refresh failed"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::AuthMode;
    use chrono::{Duration, Utc};

    fn sample() -> GrokAuth {
        GrokAuth {
            key: "k".into(),
            auth_mode: AuthMode::Oidc,
            create_time: Utc::now(),
            refresh_token: Some("rt".into()),
            expires_at: Some(Utc::now() + Duration::hours(1)),
            oidc_issuer: Some(crate::auth::codex::CODEX_ISSUER.into()),
            oidc_client_id: Some(crate::auth::codex::CODEX_CLIENT_ID.into()),
            ..Default::default()
        }
    }

    #[test]
    fn refresher_is_constructible_without_store_side_effects() {
        let r = CodexRefresher::new(sample()).with_token_url("http://127.0.0.1:9/oauth/token");
        assert_eq!(r.tried_key(), "k");
        // No disk writes possible from construction.
    }
}
