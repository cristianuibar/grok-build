//! Pure dual-provider auth status inspect (AUTH-04).
//!
//! No CLI I/O here — Plan 04 wires `bum auth status` to these builders/formatters.
//! Secrets (access tokens, refresh tokens, raw keys/JWTs) never appear in
//! formatted output (UI-SPEC paste-safe contract).

use std::path::Path;

use chrono::Duration;

use super::model::{AuthMode, AuthProvider, AuthStore, GrokAuth, is_expired_with_buffer};
use super::storage::{AuthStoreReadError, read_auth_document};

// AuthDocument is crate-private; status builds from it only inside this crate.
use super::model::AuthDocument;

/// Per-provider auth status fields (greppable CLI report).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderAuthStatus {
    pub provider: AuthProvider,
    /// Persisted OAuth session/token record exists (nonblank access and/or
    /// refresh_token / session entry).
    pub logged_in: bool,
    /// Hard-unexpired access token **or** refreshable OAuth session (refresh
    /// token present). Not derived from [`super::select_provider_access_token`]
    /// alone — that helper may return expired tokens when no fresh candidate
    /// exists.
    pub usable: bool,
    /// Email or stable account label when available.
    pub account: Option<String>,
    /// Plan/tier label when available (none → em-dash in format).
    pub plan: Option<String>,
}

/// Dual-provider status report (always both slots, xAI then Codex).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthStatusReport {
    pub providers: [ProviderAuthStatus; 2],
}

impl AuthStatusReport {
    /// Build status from an on-disk auth file path.
    ///
    /// Missing file → both providers `logged_in: no` / `usable: no`.
    /// Parse / I/O failures → [`AuthStoreReadError`] (callers surface UI-SPEC
    /// store-unreadable copy; never dump file contents).
    pub fn from_auth_file(path: &Path) -> Result<Self, AuthStoreReadError> {
        match read_auth_document(path) {
            Ok(doc) => Ok(Self::from_document(&doc)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::empty()),
            Err(e) if e.kind() == std::io::ErrorKind::InvalidData => Err(AuthStoreReadError::Parse {
                path: path.to_path_buf(),
            }),
            Err(e) if e.kind() == std::io::ErrorKind::Unsupported => {
                let version = e
                    .to_string()
                    .split_whitespace()
                    .find_map(|tok| tok.parse::<u32>().ok())
                    .unwrap_or(0);
                Err(AuthStoreReadError::UnsupportedVersion {
                    path: path.to_path_buf(),
                    version,
                })
            }
            Err(e) => Err(AuthStoreReadError::Io {
                path: path.to_path_buf(),
                kind: e.kind(),
            }),
        }
    }

    /// Build status from an already-parsed document (pure; no I/O).
    ///
    /// Crate-private: [`AuthDocument`] is not part of the public surface.
    pub(crate) fn from_document(doc: &AuthDocument) -> Self {
        Self {
            providers: [
                status_for_provider(
                    AuthProvider::Xai,
                    doc.providers.get(AuthProvider::Xai.as_str()),
                ),
                status_for_provider(
                    AuthProvider::Codex,
                    doc.providers.get(AuthProvider::Codex.as_str()),
                ),
            ],
        }
    }

    /// Both providers listed as not logged in / not usable.
    pub fn empty() -> Self {
        Self {
            providers: [
                empty_provider_status(AuthProvider::Xai),
                empty_provider_status(AuthProvider::Codex),
            ],
        }
    }

    /// Greppable, paste-safe dual status text (UI-SPEC block format).
    pub fn format(&self) -> String {
        format_auth_status(self)
    }
}

/// Inspect a single provider store for logged_in / usable / account / plan.
///
/// Pure; no I/O. Suitable for unit tests with in-memory fixtures.
pub fn inspect_provider_store(provider: AuthProvider, store: Option<&AuthStore>) -> ProviderAuthStatus {
    status_for_provider(provider, store)
}

/// Usable semantics for one credential record (pure).
///
/// - `true` when access is hard-unexpired (`Duration::zero()` buffer), **or**
/// - `true` when a nonblank refresh_token is present (refreshable OAuth;
///   permanently-invalid is an in-memory AuthManager concern, not on-disk)
/// - `false` when access is hard-expired and no refresh_token
///
/// Intentionally does **not** use [`super::select_provider_access_token`].
pub fn credential_usable(auth: &GrokAuth) -> bool {
    if auth.key.trim().is_empty() && auth.refresh_token.as_ref().is_none_or(|t| t.trim().is_empty()) {
        return false;
    }
    let hard_unexpired =
        !auth.key.trim().is_empty() && !is_expired_with_buffer(auth, Duration::zero());
    if hard_unexpired {
        return true;
    }
    // Refreshable OAuth session: nonblank refresh_token present.
    auth.refresh_token
        .as_ref()
        .is_some_and(|t| !t.trim().is_empty())
}

/// Whether a store contains a persisted session/token record.
pub fn store_logged_in(store: &AuthStore) -> bool {
    store.values().any(record_logged_in)
}

/// Whether any credential in the store is usable under status semantics.
pub fn store_usable(store: &AuthStore) -> bool {
    store
        .values()
        .filter(|a| a.auth_mode != AuthMode::WebLogin)
        .any(credential_usable)
}

/// Whether a provider OAuth slot is usable for switch-time gating (Phase 6 D-02).
///
/// Pure decision over an optional on-disk store for **one** provider slot.
/// Empty / missing store → unusable. Reuses [`store_usable`] / [`credential_usable`]
/// (refreshable OAuth counts as usable). Does **not** consult xAI-only
/// `AuthManager` — Codex must never be inferred from the xAI manager alone.
pub fn provider_slot_usable(store: Option<&AuthStore>) -> bool {
    store.is_some_and(store_usable)
}

fn empty_provider_status(provider: AuthProvider) -> ProviderAuthStatus {
    ProviderAuthStatus {
        provider,
        logged_in: false,
        usable: false,
        account: None,
        plan: None,
    }
}

fn status_for_provider(provider: AuthProvider, store: Option<&AuthStore>) -> ProviderAuthStatus {
    let Some(store) = store.filter(|s| !s.is_empty()) else {
        return empty_provider_status(provider);
    };

    let logged_in = store_logged_in(store);
    let usable = store_usable(store);
    let account = select_account_label(store);
    // No dedicated plan field on GrokAuth yet — always em-dash in format.
    let plan = None;

    ProviderAuthStatus {
        provider,
        logged_in,
        usable,
        account,
        plan,
    }
}

fn record_logged_in(auth: &GrokAuth) -> bool {
    if auth.auth_mode == AuthMode::WebLogin {
        return false;
    }
    let has_access = !auth.key.trim().is_empty();
    let has_refresh = auth
        .refresh_token
        .as_ref()
        .is_some_and(|t| !t.trim().is_empty());
    has_access || has_refresh
}

fn select_account_label(store: &AuthStore) -> Option<String> {
    // Prefer Oidc sessions with email; fall back to any non-WebLogin email.
    let mut entries: Vec<&GrokAuth> = store
        .values()
        .filter(|a| a.auth_mode != AuthMode::WebLogin)
        .collect();
    entries.sort_by_key(|a| match a.auth_mode {
        AuthMode::Oidc => 0u8,
        AuthMode::ApiKey => 1,
        AuthMode::External => 2,
        AuthMode::WebLogin => 3,
    });
    for auth in entries {
        if let Some(email) = auth.email.as_ref().filter(|e| !e.trim().is_empty()) {
            return Some(email.clone());
        }
        if let Some(org) = auth
            .organization_id
            .as_ref()
            .filter(|o| !o.trim().is_empty())
        {
            return Some(org.clone());
        }
        if !auth.user_id.trim().is_empty() {
            return Some(auth.user_id.clone());
        }
    }
    None
}

/// Format UI-SPEC greppable blocks (xAI first, blank line, Codex).
///
/// Never includes access_token, refresh_token, key, or raw JWT material.
pub fn format_auth_status(report: &AuthStatusReport) -> String {
    let mut out = String::new();
    for (i, status) in report.providers.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        out.push_str(status.provider.label());
        out.push_str(":\n");
        out.push_str("  logged_in: ");
        out.push_str(yes_no(status.logged_in));
        out.push('\n');
        out.push_str("  usable: ");
        out.push_str(yes_no(status.usable));
        out.push('\n');
        out.push_str("  account: ");
        out.push_str(status.account.as_deref().unwrap_or("—"));
        out.push('\n');
        out.push_str("  plan: ");
        out.push_str(status.plan.as_deref().unwrap_or("—"));
        out.push('\n');
    }
    out
}

fn yes_no(v: bool) -> &'static str {
    if v { "yes" } else { "no" }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::model::{PROVIDER_CODEX, PROVIDER_XAI};
    use chrono::Utc;
    use std::collections::BTreeMap;

    fn sample_oidc(key: &str, refresh: Option<&str>, expired: bool) -> GrokAuth {
        GrokAuth {
            key: key.to_owned(),
            auth_mode: AuthMode::Oidc,
            create_time: Utc::now(),
            user_id: "u".to_owned(),
            email: Some("user@example.test".to_owned()),
            expires_at: Some(if expired {
                Utc::now() - chrono::Duration::minutes(5)
            } else {
                Utc::now() + chrono::Duration::hours(1)
            }),
            refresh_token: refresh.map(str::to_owned),
            ..Default::default()
        }
    }

    #[test]
    fn usable_expired_with_refresh_is_true() {
        let auth = sample_oidc("access", Some("rt"), true);
        assert!(credential_usable(&auth));
    }

    #[test]
    fn usable_expired_without_refresh_is_false() {
        let auth = sample_oidc("access", None, true);
        assert!(!credential_usable(&auth));
    }

    #[test]
    fn usable_hard_unexpired_without_refresh_is_true() {
        let auth = sample_oidc("access", None, false);
        assert!(credential_usable(&auth));
    }

    #[test]
    fn format_never_emits_token_material() {
        let mut xai = AuthStore::new();
        xai.insert(
            "xai::fixture".to_owned(),
            sample_oidc("xai-secret-token-xyz", Some("xai-rt-secret"), false),
        );
        let mut codex = AuthStore::new();
        codex.insert(
            "codex::fixture".to_owned(),
            sample_oidc("codex-secret-token-xyz", Some("codex-rt-secret"), false),
        );
        let mut providers = BTreeMap::new();
        providers.insert(PROVIDER_XAI.to_owned(), xai);
        providers.insert(PROVIDER_CODEX.to_owned(), codex);
        let doc = AuthDocument {
            version: Some(1),
            providers,
        };
        let text = AuthStatusReport::from_document(&doc).format();
        assert!(text.contains("logged_in: yes"));
        assert!(text.contains("usable: yes"));
        assert!(!text.contains("secret"));
        assert!(!text.contains("xai-secret"));
        assert!(!text.contains("codex-secret"));
    }

    #[test]
    fn p6_provider_slot_usable_empty_store_false() {
        assert!(
            !provider_slot_usable(None),
            "missing store must be unusable"
        );
        assert!(
            !provider_slot_usable(Some(&AuthStore::new())),
            "empty store must be unusable"
        );
    }

    #[test]
    fn p6_provider_slot_usable_refreshable_true() {
        let mut store = AuthStore::new();
        store.insert(
            "codex::fixture".to_owned(),
            sample_oidc("expired-access", Some("refresh-token"), true),
        );
        assert!(
            provider_slot_usable(Some(&store)),
            "expired access + nonblank refresh must be usable (D-02)"
        );
    }

    #[test]
    fn p6_provider_slot_usable_hard_expired_no_refresh_false() {
        let mut store = AuthStore::new();
        store.insert(
            "codex::fixture".to_owned(),
            sample_oidc("expired-access", None, true),
        );
        assert!(
            !provider_slot_usable(Some(&store)),
            "hard-expired access without refresh must be unusable"
        );
    }

    #[test]
    fn empty_document_lists_both_providers() {
        let text = AuthStatusReport::empty().format();
        assert!(text.contains("xAI:"));
        assert!(text.contains("Codex:"));
        assert_eq!(text.matches("logged_in: no").count(), 2);
        assert_eq!(text.matches("usable: no").count(), 2);
    }
}
