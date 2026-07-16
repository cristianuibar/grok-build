//! Phase 5 Wave 0 — dual-auth lifecycle harness (AUTH-02..05).
//!
//! **Scope:** Public path-taking storage APIs + behavior-RED contracts for
//! Codex login/logout/status/refresh isolation. No production dual-auth
//! implementation in this binary's plan (05-01). Fake tokens only.
//!
//! ## BUM_HOME / OnceLock hygiene (mandatory)
//!
//! Prefer **explicit `auth_file: &Path` public APIs** (`read_provider_auth_store`,
//! nested fixture paths under `tempfile`). Do **not** mutate `BUM_HOME` /
//! `GROK_HOME` to different values across tests in this process — `grok_home()`
//! is a OnceLock. One process-wide sandbox set once, or a separate subprocess,
//! is the only safe alternative when a future handler requires `grok_home()`.
//!
//! ## Cargo hygiene
//!
//! Prefer:
//! ```text
//! cargo test -p xai-grok-shell --test auth_codex_lifecycle -- --list
//! cargo test -p xai-grok-shell --test auth_codex_lifecycle auth_codex_lifecycle_harness_smoke
//! ```
//! Never use unfiltered `cargo test -p xai-grok-shell --lib` as a Phase 5 gate.
//!
//! ## Option C reconstruct seam (Plan 05 crate-local only)
//!
//! The following names are **reserved** for Plan 05 `--lib` tests under
//! `session/acp_session_tests/codex_reconstruct_refresh_tests.rs` and must
//! **not** appear as integration-binary tests here (cycle 3 name isolation):
//! - `codex_reconstruct_refreshes_mid_session_expiry`
//! - `codex_byok_key_not_overridden`
//! - `codex_oauth_bearer_absent_on_custom_endpoint`
//!
//! Prove them with: `cargo test -p xai-grok-shell --lib <TESTNAME>`.
//!
//! ## Requirements / decisions
//!
//! AUTH-02..05 · D-01..D-13 · D-09 no `~/.codex` import · D-13 mock/fixture only.
//! CI RED sequential until Plans 02–06 GREEN. No live ChatGPT. No Phase 6 gate UX.

use std::collections::BTreeMap;
use std::path::Path;

use chrono::{Duration, Utc};
use indexmap::IndexMap;
use xai_grok_shell::agent::config::{
    inject_url_derived_headers, is_first_party_codex_url, EndpointsConfig, CODEX_BASE_URL_DEFAULT,
    XAI_API_BASE_URL_DEFAULT,
};
use xai_grok_shell::auth::{
    read_provider_auth_store, select_provider_access_token, AuthMode, AuthStore, GrokAuth,
    PROVIDER_CODEX, PROVIDER_XAI,
};

const XAI_FAKE: &str = "xai-fake-token";
const CODEX_FAKE: &str = "codex-fake-token";
const CODEX_REFRESH: &str = "codex-refresh-token";
const CODEX_ACCOUNT: &str = "chatgpt-acct-fixture";

fn xai_scope() -> String {
    // Stable fixture scope; independent of GrokComConfig defaults.
    "xai::fixture".to_owned()
}

fn codex_scope() -> String {
    "codex::fixture".to_owned()
}

fn sample_xai_auth() -> GrokAuth {
    GrokAuth {
        key: XAI_FAKE.to_owned(),
        auth_mode: AuthMode::Oidc,
        create_time: Utc::now(),
        user_id: "xai-user".to_owned(),
        email: Some("xai@example.test".to_owned()),
        expires_at: Some(Utc::now() + Duration::hours(1)),
        refresh_token: Some("xai-refresh-token".to_owned()),
        oidc_issuer: Some("https://auth.x.ai".to_owned()),
        oidc_client_id: Some("xai-client".to_owned()),
        ..Default::default()
    }
}

fn sample_codex_auth() -> GrokAuth {
    GrokAuth {
        key: CODEX_FAKE.to_owned(),
        auth_mode: AuthMode::Oidc,
        create_time: Utc::now(),
        user_id: "codex-user".to_owned(),
        email: Some("codex@example.test".to_owned()),
        organization_id: Some(CODEX_ACCOUNT.to_owned()),
        expires_at: Some(Utc::now() + Duration::hours(1)),
        refresh_token: Some(CODEX_REFRESH.to_owned()),
        oidc_issuer: Some("https://auth.openai.com".to_owned()),
        oidc_client_id: Some("app_EMoamEEZ73f0CkXaXp7hrann".to_owned()),
        ..Default::default()
    }
}

/// Nested multi-slot `auth.json` with both providers (explicit path only).
fn write_dual_auth_fixture(path: &Path) {
    let mut xai: AuthStore = BTreeMap::new();
    xai.insert(xai_scope(), sample_xai_auth());
    let mut codex: AuthStore = BTreeMap::new();
    codex.insert(codex_scope(), sample_codex_auth());
    let document = serde_json::json!({
        "version": 1,
        "providers": {
            "xai": xai,
            "codex": codex,
        }
    });
    std::fs::write(path, serde_json::to_vec_pretty(&document).unwrap()).unwrap();
}

fn pointer_key(doc: &serde_json::Value, provider: &str, scope: &str) -> Option<String> {
    doc.pointer(&format!("/providers/{provider}/{scope}/key"))
        .and_then(|v| v.as_str())
        .map(str::to_owned)
}

// ── Smoke (GREEN on current main) ──────────────────────────────────────────

/// Wave 0 harness smoke: dual-slot tempfile readable via public path APIs.
#[test]
fn auth_codex_lifecycle_harness_smoke() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("auth.json");
    write_dual_auth_fixture(&path);

    let xai = read_provider_auth_store(&path, PROVIDER_XAI)
        .expect("xai slot read")
        .expect("xai slot present");
    let codex = read_provider_auth_store(&path, PROVIDER_CODEX)
        .expect("codex slot read")
        .expect("codex slot present");

    let xai_tok = select_provider_access_token(&xai).expect("xai token");
    let codex_tok = select_provider_access_token(&codex).expect("codex token");
    assert_eq!(xai_tok.key, XAI_FAKE);
    assert_eq!(codex_tok.key, CODEX_FAKE);
    assert_eq!(codex_tok.organization_id.as_deref(), Some(CODEX_ACCOUNT));
    assert_eq!(codex_tok.refresh_token.as_deref(), Some(CODEX_REFRESH));
    assert_eq!(codex_tok.auth_mode, AuthMode::Oidc);

    // Sibling isolation on disk: both keys still present after pure reads.
    let on_disk: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(
        pointer_key(&on_disk, PROVIDER_XAI, &xai_scope()).as_deref(),
        Some(XAI_FAKE)
    );
    assert_eq!(
        pointer_key(&on_disk, PROVIDER_CODEX, &codex_scope()).as_deref(),
        Some(CODEX_FAKE)
    );
}

// ── AUTH-02: Codex login (RED until Plan 03) ───────────────────────────────

/// Login(provider=codex) must persist Oidc under providers.codex and leave xAI.
#[test]
fn codex_login_persists_slot() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("auth.json");
    // Pre-populate only xAI so a future codex login can prove sibling preservation.
    let mut xai: AuthStore = BTreeMap::new();
    xai.insert(xai_scope(), sample_xai_auth());
    let document = serde_json::json!({
        "version": 1,
        "providers": { "xai": xai }
    });
    std::fs::write(&path, serde_json::to_vec_pretty(&document).unwrap()).unwrap();

    // Public surface today has no path-taking Codex login writer. Contract RED
    // until Plan 03 implements in-tree PKCE/device login → providers.codex.
    let codex = read_provider_auth_store(&path, PROVIDER_CODEX).expect("read ok");
    assert!(
        codex.is_some()
            && select_provider_access_token(codex.as_ref().unwrap())
                .is_some_and(|a| a.auth_mode == AuthMode::Oidc && !a.key.is_empty()),
        "Plan 03: codex_login_persists_slot — expect Codex OAuth login to write Oidc \
         providers.codex (access/refresh/expiry/account) while preserving providers.xai \
         (auth_file={})",
        path.display()
    );
}

/// Authorize URL must include PKCE S256 challenge + localhost callback (1455/1457).
#[test]
fn codex_authorize_url_includes_pkce_and_localhost_callback() {
    // No public Codex authorize-URL builder yet (Plan 03: auth/codex browser flow).
    let authorize_url: Option<String> = None;
    let url = authorize_url.as_deref().unwrap_or("");
    assert!(
        url.contains("code_challenge")
            && (url.contains("S256") || url.contains("code_challenge_method=S256"))
            && (url.contains("localhost") || url.contains("127.0.0.1"))
            && (url.contains("1455") || url.contains("1457"))
            && url.contains("auth/callback"),
        "Plan 03: codex_authorize_url_includes_pkce_and_localhost_callback — expect \
         authorize URL with PKCE S256 + http://localhost:{{1455|1457}}/auth/callback \
         (client_id app_EMoamEEZ73f0CkXaXp7hrann)"
    );
}

/// Device endpoints must use OpenAI deviceauth paths under auth.openai.com.
#[test]
fn codex_device_endpoints_use_deviceauth() {
    let usercode: Option<&str> = None;
    let token_poll: Option<&str> = None;
    assert!(
        usercode.is_some_and(|u| u.contains("deviceauth") && u.contains("usercode"))
            && token_poll.is_some_and(|t| t.contains("deviceauth") && t.contains("token")),
        "Plan 03: codex_device_endpoints_use_deviceauth — expect \
         {{issuer}}/api/accounts/deviceauth/usercode and .../token endpoints"
    );
}

/// Device multi-step: pending → slow_down → exchange denied must not write tokens.
#[test]
fn codex_device_pending_slowdown_exchange_denied() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("auth.json");
    write_dual_auth_fixture(&path);
    let before = std::fs::read(&path).unwrap();

    // No public device multi-step runner yet.
    let device_exchange_completed = false;
    assert!(
        device_exchange_completed,
        "Plan 03: codex_device_pending_slowdown_exchange_denied — expect device poll \
         handling authorization_pending / slow_down / access_denied without mutating \
         auth.json (auth_file={})",
        path.display()
    );
    let _ = before;
}

/// OAuth state mismatch must write nothing to either provider slot.
#[test]
fn codex_oauth_state_mismatch_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("auth.json");
    let mut xai: AuthStore = BTreeMap::new();
    xai.insert(xai_scope(), sample_xai_auth());
    let document = serde_json::json!({
        "version": 1,
        "providers": { "xai": xai }
    });
    std::fs::write(&path, serde_json::to_vec_pretty(&document).unwrap()).unwrap();
    let before = std::fs::read(&path).unwrap();

    // No public Codex callback handler yet; contract requires zero writes on state mismatch.
    let state_mismatch_handler_present = false;
    assert!(
        state_mismatch_handler_present,
        "Plan 03: codex_oauth_state_mismatch_writes_nothing — expect callback state \
         mismatch to reject without writing providers.codex or mutating xAI \
         (auth_file={})",
        path.display()
    );
    assert_eq!(
        std::fs::read(&path).unwrap(),
        before,
        "fixture must remain unchanged while seam is missing"
    );
}

// ── AUTH-03: Selective logout (RED until Plan 04) ──────────────────────────

/// Logout --provider codex leaves xAI; logout xAI leaves Codex.
#[test]
fn selective_logout_isolates() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("auth.json");
    write_dual_auth_fixture(&path);

    // No public clear_provider_slot(path, provider) yet.
    let cleared_codex_only = false;
    assert!(
        cleared_codex_only,
        "Plan 04: selective_logout_isolates — expect clear_provider_slot(codex) to remove \
         providers.codex while providers.xai key remains {XAI_FAKE} (auth_file={})",
        path.display()
    );

    // Sanity: dual fixture still intact (no accidental wipe during RED).
    let xai = read_provider_auth_store(&path, PROVIDER_XAI)
        .unwrap()
        .expect("xai present");
    assert_eq!(
        select_provider_access_token(&xai).map(|a| a.key),
        Some(XAI_FAKE.to_owned())
    );
}

/// Logout --all clears both provider slots atomically.
#[test]
fn logout_all_clears_both() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("auth.json");
    write_dual_auth_fixture(&path);

    let cleared_all = false;
    assert!(
        cleared_all,
        "Plan 04: logout_all_clears_both — expect clear_all_provider_slots to empty/remove \
         both providers.xai and providers.codex (auth_file={})",
        path.display()
    );
}

/// Bare logout must fail closed (usage + non-zero) without dual wipe.
#[test]
fn bare_logout_fail_closed() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("auth.json");
    write_dual_auth_fixture(&path);
    let before = std::fs::read(&path).unwrap();

    // Contract: bare logout without --provider/--all must not mutate store.
    // Production dual fail-closed handler lands in Plan 04.
    let bare_logout_rejected_without_mutation = false;
    assert!(
        bare_logout_rejected_without_mutation,
        "Plan 04: bare_logout_fail_closed — expect usage error without clearing either \
         provider slot (auth_file={})",
        path.display()
    );
    assert_eq!(std::fs::read(&path).unwrap(), before);
}

// ── AUTH-04: Status (RED until Plan 02/04) ─────────────────────────────────

/// Status formatter is greppable, lists both providers, never prints secrets.
#[test]
fn auth_status_format_paste_safe() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("auth.json");
    write_dual_auth_fixture(&path);

    // No public format_auth_status yet.
    let formatted: Option<String> = None;
    let text = formatted.unwrap_or_default();
    let has_both = text.contains("xai") && text.contains("codex");
    let no_secrets = !text.contains(XAI_FAKE)
        && !text.contains(CODEX_FAKE)
        && !text.contains(CODEX_REFRESH)
        && !text.contains("xai-refresh-token");
    assert!(
        has_both && no_secrets && !text.is_empty(),
        "Plan 02/04: auth_status_format_paste_safe — expect greppable dual-provider \
         status without access/refresh token substrings (auth_file={})",
        path.display()
    );
}

/// CLI status handler path (run_cli_auth_status) must return dual status text.
#[test]
fn run_cli_auth_status() {
    // Public run_cli_auth_status is Plan 02/04.
    let handler_available = false;
    assert!(
        handler_available,
        "Plan 02/04: run_cli_auth_status — expect CLI handler returning paste-safe \
         dual-provider status string (no secrets)"
    );
}

/// usable=yes when expired but refresh_token present.
#[test]
fn auth_status_usable_expired_refreshable() {
    let mut auth = sample_codex_auth();
    auth.expires_at = Some(Utc::now() - Duration::minutes(5));
    assert!(
        auth.refresh_token.is_some(),
        "fixture must carry refresh_token"
    );

    // No public usable() helper yet for status semantics.
    let usable: Option<bool> = None;
    assert_eq!(
        usable,
        Some(true),
        "Plan 02: auth_status_usable_expired_refreshable — expired access + refresh_token \
         present must report usable=yes / refreshable"
    );
}

/// usable=no when expired and no refresh_token.
#[test]
fn auth_status_usable_expired_no_refresh() {
    let mut auth = sample_codex_auth();
    auth.expires_at = Some(Utc::now() - Duration::minutes(5));
    auth.refresh_token = None;

    let usable: Option<bool> = None;
    assert_eq!(
        usable,
        Some(false),
        "Plan 02: auth_status_usable_expired_no_refresh — expired access without \
         refresh_token must report usable=no"
    );
}

// ── AUTH-05: Independent refresh / headers (RED until Plan 05) ─────────────

/// Outer ensure_fresh must update only codex slot.
#[test]
fn codex_refresh_isolates() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("auth.json");
    write_dual_auth_fixture(&path);

    let ensure_fresh_persisted_codex_only = false;
    assert!(
        ensure_fresh_persisted_codex_only,
        "Plan 05 Task 2: codex_refresh_isolates — expect ensure_fresh_codex_auth to \
         rotate providers.codex access token only; xAI key remains {XAI_FAKE} \
         (auth_file={})",
        path.display()
    );
}

/// invalid_grant on Codex must not wipe xAI.
#[test]
fn codex_invalid_grant_no_xai_wipe() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("auth.json");
    write_dual_auth_fixture(&path);

    let permanent_fail_cleared_codex_only = false;
    assert!(
        permanent_fail_cleared_codex_only,
        "Plan 05 Task 2: codex_invalid_grant_no_xai_wipe — permanent Codex fail must \
         clear/mark only providers.codex; xAI key remains {XAI_FAKE} (auth_file={})",
        path.display()
    );
}

/// Pure data-only: when IdP omits refresh_token, preserve prior RT + identity.
#[test]
fn codex_refresh_preserves_identity_when_response_omits_refresh_token() {
    let prev = sample_codex_auth();
    // Simulated token response without refresh_token rotation.
    let mut refreshed = GrokAuth {
        key: "codex-rotated-access".to_owned(),
        auth_mode: AuthMode::Oidc,
        create_time: Utc::now(),
        user_id: String::new(),
        email: None,
        organization_id: None,
        expires_at: Some(Utc::now() + Duration::hours(1)),
        refresh_token: None, // IdP omitted RT
        oidc_issuer: prev.oidc_issuer.clone(),
        oidc_client_id: prev.oidc_client_id.clone(),
        ..Default::default()
    };

    // Plan 05 Task 1 owns pure CodexRefresher / merge helper. Until exported:
    let pure_merge_applied = false;
    if pure_merge_applied {
        if refreshed.refresh_token.is_none() {
            refreshed.refresh_token = prev.refresh_token.clone();
        }
        refreshed.user_id = prev.user_id.clone();
        refreshed.email = prev.email.clone();
        refreshed.organization_id = prev.organization_id.clone();
    }

    assert_eq!(
        refreshed.refresh_token.as_deref(),
        Some(CODEX_REFRESH),
        "Plan 05 Task 1: codex_refresh_preserves_identity_when_response_omits_refresh_token \
         — keep prior refresh_token when IdP omits it"
    );
    assert_eq!(
        refreshed.organization_id.as_deref(),
        Some(CODEX_ACCOUNT),
        "Plan 05 Task 1: must preserve chatgpt account id / organization_id"
    );
    assert_eq!(
        refreshed.user_id, prev.user_id,
        "Plan 05 Task 1: must preserve user identity across refresh"
    );
}

/// Concurrent ensure_fresh must spend IdP refresh once.
#[test]
fn codex_concurrent_refresh_single_idp_spend() {
    let concurrent_single_spend = false;
    assert!(
        concurrent_single_spend,
        "Plan 05: codex_concurrent_refresh_single_idp_spend — concurrent ensure_fresh \
         must single-flight IdP refresh (one spend of refresh_token)"
    );
}

/// Fresh unexpired token must skip IdP entirely.
#[test]
fn codex_fresh_token_skips_idp() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("auth.json");
    write_dual_auth_fixture(&path);

    let idp_called = true; // no public ensure_fresh yet → cannot prove skip
    assert!(
        !idp_called,
        "Plan 05: codex_fresh_token_skips_idp — hard-unexpired access token must not \
         call IdP refresh (auth_file={})",
        path.display()
    );
}

/// Transient fail with hard-unexpired access keeps old token.
#[test]
fn codex_transient_fail_hard_unexpired_keeps_token() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("auth.json");
    write_dual_auth_fixture(&path);

    let kept_old = false;
    assert!(
        kept_old,
        "Plan 05: codex_transient_fail_hard_unexpired_keeps_token — transient IdP fail \
         while access still unexpired must keep existing codex key {CODEX_FAKE} \
         (auth_file={})",
        path.display()
    );
}

/// Transient fail with hard-expired access yields no usable credential.
#[test]
fn codex_transient_fail_hard_expired_no_credential() {
    let no_usable_credential = false;
    assert!(
        no_usable_credential,
        "Plan 05: codex_transient_fail_hard_expired_no_credential — transient fail with \
         hard-expired access must not surface a usable Codex bearer"
    );
}

/// Trusted first-party Codex endpoint must receive ChatGPT-Account-ID.
#[test]
fn chatgpt_account_id_header_on_codex() {
    let endpoints = EndpointsConfig {
        xai_api_base_url: XAI_API_BASE_URL_DEFAULT.to_owned(),
        codex_base_url: CODEX_BASE_URL_DEFAULT.to_owned(),
        ..EndpointsConfig::default()
    };
    assert!(
        is_first_party_codex_url(CODEX_BASE_URL_DEFAULT, &endpoints),
        "fixture Codex default must be first-party"
    );

    let mut headers = IndexMap::new();
    // Current public inject_url_derived_headers has no account-id arg; Plan 05
    // will inject ChatGPT-Account-ID only for trusted Codex + OAuth material.
    inject_url_derived_headers(&mut headers, None, CODEX_BASE_URL_DEFAULT);

    assert_eq!(
        headers.get("ChatGPT-Account-ID").map(String::as_str),
        Some(CODEX_ACCOUNT),
        "Plan 05: chatgpt_account_id_header_on_codex — trusted Codex endpoint must \
         receive ChatGPT-Account-ID from organization_id / CodexAuthMaterial"
    );
}

/// xAI inference URL must never receive ChatGPT-Account-ID.
#[test]
fn chatgpt_account_id_header_absent_on_xai() {
    let mut headers = IndexMap::new();
    inject_url_derived_headers(&mut headers, None, XAI_API_BASE_URL_DEFAULT);
    assert!(
        !headers.contains_key("ChatGPT-Account-ID"),
        "chatgpt_account_id_header_absent_on_xai — xAI base URL must not get \
         ChatGPT-Account-ID (Plan 05 absence contract)"
    );
}

/// Custom / untrusted endpoint must never receive ChatGPT-Account-ID.
#[test]
fn chatgpt_account_id_header_absent_on_custom_endpoint() {
    let custom = "https://evil.example/v1";
    let mut headers = IndexMap::new();
    inject_url_derived_headers(&mut headers, None, custom);
    assert!(
        !headers.contains_key("ChatGPT-Account-ID"),
        "chatgpt_account_id_header_absent_on_custom_endpoint — custom host must not \
         get ChatGPT-Account-ID via global auth read (Plan 05)"
    );
}
