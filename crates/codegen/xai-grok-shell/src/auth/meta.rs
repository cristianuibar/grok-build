use serde::{Deserialize, Serialize};

use super::status::AuthStatusReport;

/// Access gate from `grok_build_access_gate`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateInfo {
    pub message: String,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub label: Option<String>,
}

/// Per-provider usable flag for display/status hints (never credentials).
///
/// Booleans only — access tokens, refresh tokens, and raw keys must never
/// appear here. Usable semantics match D-02 / [`super::status::store_usable`]
/// (hard-unexpired access **or** refreshable OAuth).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderSlotUsableMeta {
    #[serde(default)]
    pub usable: bool,
}

/// Dual-slot provider usability snapshot embedded in [`AuthMeta`].
///
/// Display/status hints for the pager badge cache — shell switch gate remains
/// authoritative for apply decisions.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderAuthMetaSlots {
    #[serde(default)]
    pub xai: ProviderSlotUsableMeta,
    #[serde(default)]
    pub codex: ProviderSlotUsableMeta,
}

impl ProviderAuthMetaSlots {
    /// Pure mapping from an [`AuthStatusReport`] (xAI then Codex order).
    pub fn from_report(report: &AuthStatusReport) -> Self {
        let mut slots = Self::default();
        for status in &report.providers {
            match status.provider {
                super::model::AuthProvider::Xai => {
                    slots.xai.usable = status.usable;
                }
                super::model::AuthProvider::Codex => {
                    slots.codex.usable = status.usable;
                }
            }
        }
        slots
    }

    /// Load dual-slot usable flags from an auth file path.
    ///
    /// Missing file → both unusable. Parse/IO errors → both unusable
    /// (fail-closed for display hints; never logs file contents).
    pub fn from_auth_file(path: &std::path::Path) -> Self {
        match AuthStatusReport::from_auth_file(path) {
            Ok(report) => Self::from_report(&report),
            Err(_) => Self::default(),
        }
    }
}

/// Typed auth metadata passed from the shell to the pager via ACP.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuthMeta {
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub auth_mode: Option<String>,
    /// Team principal UUID when the session is a team login (`None` for personal).
    #[serde(default)]
    pub team_id: Option<String>,
    #[serde(default)]
    pub team_name: Option<String>,
    #[serde(default)]
    pub is_zdr: bool,
    #[serde(default)]
    pub team_role: Option<String>,
    #[serde(default)]
    pub coding_data_retention_opt_out: bool,
    #[serde(default)]
    pub show_resolved_model: Option<bool>,
    /// `Some` = user is blocked; `None` = user has access.
    #[serde(default)]
    pub gate: Option<GateInfo>,
    /// User-friendly display name for the current subscription tier
    /// (e.g. "SuperGrok Heavy", "X Premium", "Free"). From CCP `/settings`.
    #[serde(default)]
    pub subscription_tier: Option<String>,
    /// Dual-slot provider usable flags (booleans only). Present when the shell
    /// could inspect dual-auth status; `None` means older clients / unknown.
    ///
    /// Display/status hints only — never tokens. Pager badge cache consumes
    /// this via `apply_auth_meta`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub providers: Option<ProviderAuthMetaSlots>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::model::{
        AuthMode, AuthDocument, AuthProvider, AuthStore, GrokAuth, PROVIDER_CODEX, PROVIDER_XAI,
    };
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
    fn p6_auth_meta_includes_provider_usable_slots() {
        let mut xai = AuthStore::new();
        xai.insert(
            "xai::fixture".to_owned(),
            sample_oidc("xai-access", Some("xai-rt"), false),
        );
        let mut codex = AuthStore::new();
        // Expired access but refreshable → usable under D-02.
        codex.insert(
            "codex::fixture".to_owned(),
            sample_oidc("codex-access", Some("codex-rt"), true),
        );
        let mut providers = BTreeMap::new();
        providers.insert(PROVIDER_XAI.to_owned(), xai);
        providers.insert(PROVIDER_CODEX.to_owned(), codex);
        let doc = AuthDocument {
            version: Some(1),
            providers,
        };
        let report = AuthStatusReport::from_document(&doc);
        let slots = ProviderAuthMetaSlots::from_report(&report);
        assert!(slots.xai.usable);
        assert!(slots.codex.usable);

        let meta = AuthMeta {
            email: Some("user@example.test".into()),
            providers: Some(slots),
            ..Default::default()
        };
        let json = serde_json::to_value(&meta).expect("serialize");
        // camelCase wire shape for nested providers.
        assert_eq!(json["providers"]["xai"]["usable"], true);
        assert_eq!(json["providers"]["codex"]["usable"], true);
        // Never leak tokens through AuthMeta.
        let dumped = json.to_string();
        assert!(!dumped.contains("xai-access"));
        assert!(!dumped.contains("codex-rt"));

        let round: AuthMeta = serde_json::from_value(json).expect("deserialize");
        let providers = round.providers.expect("providers present");
        assert!(providers.xai.usable);
        assert!(providers.codex.usable);
    }

    #[test]
    fn p6_auth_meta_providers_default_absent_for_old_clients() {
        let json = serde_json::json!({
            "email": "old@example.test",
            "is_zdr": false
        });
        let meta: AuthMeta = serde_json::from_value(json).expect("old client shape");
        assert!(meta.providers.is_none());
        assert_eq!(meta.email.as_deref(), Some("old@example.test"));
    }

    #[test]
    fn p6_provider_slots_empty_report_both_unusable() {
        let slots = ProviderAuthMetaSlots::from_report(&AuthStatusReport::empty());
        assert!(!slots.xai.usable);
        assert!(!slots.codex.usable);
        let _ = AuthProvider::Xai; // keep import used if Default path changes
    }
}
