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
use xai_grok_shell::auth::codex::{
    self, CODEX_AUTH_SCOPE, CODEX_CLIENT_ID, CODEX_ISSUER, CODEX_PREFERRED_PORT, CodexAuthorizeSession,
    CodexTokenResponse, apply_codex_oauth_callback, build_codex_authorize_url, codex_device_token_url,
    codex_device_usercode_url, codex_redirect_uri, persist_codex_tokens,
};
use xai_grok_shell::auth::{
    bare_logout_usage, clear_all_provider_slots, clear_provider_slot, credential_usable,
    format_auth_status, format_dual_logout_message, inspect_provider_store, logout_all_provider_slots,
    logout_provider_slot, mutate_provider_store_or_prune, read_provider_auth_store,
    select_provider_access_token, write_cli_auth_status, AuthMode, AuthProvider, AuthStatusReport,
    AuthStore, GrokAuth, ProviderStoreMutation, PROVIDER_CODEX, PROVIDER_XAI,
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

// ── AUTH-02: Codex login (Plan 03 GREEN) ───────────────────────────────────

fn fixture_jwt_with_account(account_id: &str, exp: i64) -> String {
    use base64::Engine;
    let enc = base64::engine::general_purpose::URL_SAFE_NO_PAD;
    let header = enc.encode(r#"{"alg":"RS256","typ":"JWT"}"#);
    let payload = enc.encode(format!(
        r#"{{"email":"codex-login@example.test","exp":{exp},"https://api.openai.com/auth":{{"chatgpt_account_id":"{account_id}","chatgpt_user_id":"u-codex"}}}}"#
    ));
    format!("{header}.{payload}.fake-signature")
}

/// Login(provider=codex) must persist Oidc under providers.codex and leave xAI.
#[test]
fn codex_login_persists_slot() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("auth.json");
    // Pre-populate only xAI so codex login proves sibling preservation.
    let mut xai: AuthStore = BTreeMap::new();
    xai.insert(xai_scope(), sample_xai_auth());
    let document = serde_json::json!({
        "version": 1,
        "providers": { "xai": xai }
    });
    std::fs::write(&path, serde_json::to_vec_pretty(&document).unwrap()).unwrap();

    let exp = chrono::Utc::now().timestamp() + 3600;
    let id_token = fixture_jwt_with_account(CODEX_ACCOUNT, exp);
    let access = fixture_jwt_with_account(CODEX_ACCOUNT, exp);
    let tokens = CodexTokenResponse {
        access_token: access,
        refresh_token: Some("codex-rt-injected".to_owned()),
        id_token: Some(id_token),
        expires_in: None, // force JWT exp path
    };
    let auth = persist_codex_tokens(&path, &tokens).expect("persist codex");
    assert_eq!(auth.auth_mode, AuthMode::Oidc);
    assert!(!auth.key.is_empty());
    assert_eq!(auth.organization_id.as_deref(), Some(CODEX_ACCOUNT));
    assert_eq!(auth.email.as_deref(), Some("codex-login@example.test"));
    assert!(auth.expires_at.is_some(), "expires_at from JWT exp");
    assert_eq!(auth.oidc_issuer.as_deref(), Some(CODEX_ISSUER));
    assert_eq!(auth.oidc_client_id.as_deref(), Some(CODEX_CLIENT_ID));

    let codex = read_provider_auth_store(&path, PROVIDER_CODEX)
        .expect("read ok")
        .expect("codex slot present");
    let tok = select_provider_access_token(&codex).expect("codex token");
    assert_eq!(tok.auth_mode, AuthMode::Oidc);
    assert!(!tok.key.is_empty());
    assert_eq!(tok.organization_id.as_deref(), Some(CODEX_ACCOUNT));
    assert!(codex.contains_key(CODEX_AUTH_SCOPE));

    // Sibling xAI preserved.
    let on_disk: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(
        pointer_key(&on_disk, PROVIDER_XAI, &xai_scope()).as_deref(),
        Some(XAI_FAKE)
    );
}

/// Authorize URL must include PKCE S256 challenge + localhost callback (1455/1457).
#[test]
fn codex_authorize_url_includes_pkce_and_localhost_callback() {
    let session = CodexAuthorizeSession::new(CODEX_PREFERRED_PORT);
    let url = &session.authorize_url;
    // redirect_uri is query-encoded (`auth%2Fcallback`); scopes encode spaces as %20.
    assert!(
        url.contains("code_challenge")
            && url.contains("code_challenge_method=S256")
            && url.contains("localhost")
            && url.contains("1455")
            && (url.contains("auth/callback") || url.contains("auth%2Fcallback"))
            && url.contains(CODEX_CLIENT_ID)
            && url.contains("originator=bum")
            && url.contains("offline_access"),
        "authorize URL missing PKCE/localhost/callback: {url}"
    );
    assert_eq!(
        session.redirect_uri,
        codex_redirect_uri(CODEX_PREFERRED_PORT)
    );
    assert!(session.redirect_uri.starts_with("http://localhost:"));
    // Bind host is 127.0.0.1; redirect host string must remain localhost.
    assert!(!session.redirect_uri.contains("127.0.0.1"));

    let rebuilt = build_codex_authorize_url(
        CODEX_ISSUER,
        CODEX_CLIENT_ID,
        &session.redirect_uri,
        &session.code_challenge,
        &session.state,
    );
    assert_eq!(rebuilt, session.authorize_url);
}

/// Device endpoints must use OpenAI deviceauth paths under auth.openai.com.
#[test]
fn codex_device_endpoints_use_deviceauth() {
    let usercode = codex_device_usercode_url(CODEX_ISSUER);
    let token_poll = codex_device_token_url(CODEX_ISSUER);
    assert!(
        usercode.contains("deviceauth") && usercode.contains("usercode"),
        "usercode endpoint: {usercode}"
    );
    assert!(
        token_poll.contains("deviceauth") && token_poll.contains("token"),
        "token poll endpoint: {token_poll}"
    );
    assert!(usercode.starts_with("https://auth.openai.com/api/accounts/deviceauth/usercode"));
    assert!(token_poll.starts_with("https://auth.openai.com/api/accounts/deviceauth/token"));
}

/// Device multi-step: pending → slow_down → access_denied must not write tokens.
#[tokio::test]
async fn codex_device_pending_slowdown_exchange_denied() {
    use axum::{
        Json, Router,
        extract::State,
        http::StatusCode,
        routing::post,
    };
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("auth.json");
    write_dual_auth_fixture(&path);
    let before = std::fs::read(&path).unwrap();

    #[derive(Clone)]
    struct MockState {
        polls: Arc<AtomicUsize>,
    }

    async fn usercode() -> Json<serde_json::Value> {
        Json(serde_json::json!({
            "device_auth_id": "dev-auth-1",
            "user_code": "ABCD-EFGH",
            "interval": 1
        }))
    }

    async fn token_poll(State(st): State<MockState>) -> (StatusCode, Json<serde_json::Value>) {
        let n = st.polls.fetch_add(1, Ordering::SeqCst);
        match n {
            0 => (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "authorization_pending"})),
            ),
            1 => (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "slow_down"})),
            ),
            _ => (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "access_denied"})),
            ),
        }
    }

    let st = MockState {
        polls: Arc::new(AtomicUsize::new(0)),
    };
    let app = Router::new()
        .route("/api/accounts/deviceauth/usercode", post(usercode))
        .route("/api/accounts/deviceauth/token", post(token_poll))
        .with_state(st.clone());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        axum::serve(listener, app).await.ok();
    });

    let issuer = format!("http://{addr}");
    let result = codex::run_codex_device_poll_only_with_base(&path, &issuer, CODEX_CLIENT_ID).await;
    server.abort();

    assert!(
        result.is_err(),
        "access_denied must fail without writing tokens"
    );
    assert_eq!(
        std::fs::read(&path).unwrap(),
        before,
        "device denied/pending path must not mutate auth.json"
    );
    assert!(
        st.polls.load(Ordering::SeqCst) >= 3,
        "expected pending + slow_down + denied polls, got {}",
        st.polls.load(Ordering::SeqCst)
    );
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

    let tokens = CodexTokenResponse {
        access_token: "should-not-write".to_owned(),
        refresh_token: Some("rt".to_owned()),
        id_token: None,
        expires_in: Some(3600),
    };
    let err = apply_codex_oauth_callback(
        &path,
        "expected-state",
        Some("wrong-state"),
        None,
        Some(&tokens),
    )
    .expect_err("state mismatch must fail");
    assert!(
        matches!(err, codex::CodexLoginError::StateMismatch),
        "got {err:?}"
    );

    // OAuth error path also no-write.
    let err = apply_codex_oauth_callback(
        &path,
        "expected-state",
        Some("expected-state"),
        Some("access_denied"),
        Some(&tokens),
    )
    .expect_err("oauth error must fail");
    assert!(matches!(err, codex::CodexLoginError::OAuthError(_)));

    // Exchange failure (no tokens) no-write.
    let err = apply_codex_oauth_callback(
        &path,
        "expected-state",
        Some("expected-state"),
        None,
        None,
    )
    .expect_err("missing tokens must fail");
    assert!(matches!(err, codex::CodexLoginError::TokenExchange(_)));

    assert_eq!(
        std::fs::read(&path).unwrap(),
        before,
        "fixture must remain unchanged on state/error/exchange failure"
    );
    assert!(
        read_provider_auth_store(&path, PROVIDER_CODEX)
            .unwrap()
            .is_none()
            || read_provider_auth_store(&path, PROVIDER_CODEX)
                .unwrap()
                .is_some_and(|s| s.is_empty())
    );
}

// ── AUTH-03 foundation: public provider-slot mutate/clear (Plan 02 GREEN) ──

/// Mutate Codex only; raw JSON xAI key unchanged.
#[test]
fn mutate_provider_codex_preserves_xai() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("auth.json");
    write_dual_auth_fixture(&path);

    let outcome = mutate_provider_store_or_prune(&path, AuthProvider::Codex, |codex| {
        if let Some(auth) = codex.get_mut(&codex_scope()) {
            auth.key = "codex-mutated-token".to_owned();
        }
    })
    .expect("mutate codex");
    assert_eq!(outcome, ProviderStoreMutation::DocumentWritten);

    let on_disk: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(
        pointer_key(&on_disk, PROVIDER_XAI, &xai_scope()).as_deref(),
        Some(XAI_FAKE),
        "xAI sibling must be preserved byte-stable on disk"
    );
    assert_eq!(
        pointer_key(&on_disk, PROVIDER_CODEX, &codex_scope()).as_deref(),
        Some("codex-mutated-token")
    );
}

/// Mutate xAI only; Codex key unchanged.
#[test]
fn mutate_provider_xai_preserves_codex() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("auth.json");
    write_dual_auth_fixture(&path);

    mutate_provider_store_or_prune(&path, AuthProvider::Xai, |xai| {
        if let Some(auth) = xai.get_mut(&xai_scope()) {
            auth.key = "xai-mutated-token".to_owned();
        }
    })
    .expect("mutate xai");

    let on_disk: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(
        pointer_key(&on_disk, PROVIDER_CODEX, &codex_scope()).as_deref(),
        Some(CODEX_FAKE),
        "Codex sibling must be preserved"
    );
    assert_eq!(
        pointer_key(&on_disk, PROVIDER_XAI, &xai_scope()).as_deref(),
        Some("xai-mutated-token")
    );
}

/// clear_provider_slot(codex) leaves xAI; does not delete auth.json.
#[test]
fn clear_provider_codex_leaves_xai() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("auth.json");
    write_dual_auth_fixture(&path);

    let outcome = clear_provider_slot(&path, AuthProvider::Codex).expect("clear codex");
    assert_eq!(outcome, ProviderStoreMutation::DocumentWritten);
    assert!(path.exists(), "file must remain while xAI slot is non-empty");

    assert!(
        read_provider_auth_store(&path, PROVIDER_CODEX)
            .unwrap()
            .is_none()
            || read_provider_auth_store(&path, PROVIDER_CODEX)
                .unwrap()
                .is_some_and(|s| s.is_empty()),
        "codex slot cleared"
    );
    let xai = read_provider_auth_store(&path, PROVIDER_XAI)
        .unwrap()
        .expect("xai present");
    assert_eq!(
        select_provider_access_token(&xai).map(|a| a.key),
        Some(XAI_FAKE.to_owned())
    );
}

/// Clearing the last remaining provider deletes auth.json.
#[test]
fn clear_last_provider_deletes_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("auth.json");
    // Only xAI present.
    let mut xai: AuthStore = BTreeMap::new();
    xai.insert(xai_scope(), sample_xai_auth());
    let document = serde_json::json!({
        "version": 1,
        "providers": { "xai": xai }
    });
    std::fs::write(&path, serde_json::to_vec_pretty(&document).unwrap()).unwrap();

    let outcome = clear_provider_slot(&path, AuthProvider::Xai).expect("clear last");
    assert_eq!(outcome, ProviderStoreMutation::FileDeleted);
    assert!(!path.exists(), "auth.json pruned when all slots empty");
}

/// clear_all_provider_slots empties both under one lock (atomic dual clear).
#[test]
fn clear_all_provider_slots_empties_both() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("auth.json");
    write_dual_auth_fixture(&path);

    let outcome = clear_all_provider_slots(&path).expect("clear all");
    assert_eq!(outcome, ProviderStoreMutation::FileDeleted);
    assert!(!path.exists(), "dual clear prunes empty document");
}

/// Unknown provider strings fail closed at parse (typed API has no string path).
#[test]
fn auth_provider_parse_rejects_unknown() {
    assert_eq!(AuthProvider::parse("xai"), Some(AuthProvider::Xai));
    assert_eq!(AuthProvider::parse("codex"), Some(AuthProvider::Codex));
    assert_eq!(AuthProvider::parse("openai"), None);
    assert_eq!(AuthProvider::parse("anthropic"), None);
    assert_eq!(AuthProvider::parse(""), None);
    assert_eq!(AuthProvider::parse("XAI"), None, "case-sensitive allow-list");
}

// ── AUTH-03: Selective logout CLI (Plan 04 — disk SoT clear_provider_slot) ─

/// Logout --provider codex leaves xAI; logout xAI leaves Codex.
/// CLI path uses blocking logout_provider_slot (not nonblocking remove_scope).
#[test]
fn selective_logout_isolates() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("auth.json");
    write_dual_auth_fixture(&path);

    // --provider codex
    let codex_out =
        logout_provider_slot(&path, AuthProvider::Codex).expect("logout codex");
    assert!(codex_out.was_logged_in);
    assert_eq!(codex_out.provider, Some(AuthProvider::Codex));
    assert!(!codex_out.cleared_all);
    assert!(
        format_dual_logout_message(&codex_out).contains("Codex"),
        "selective success copy must name Codex"
    );

    let on_disk: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(
        pointer_key(&on_disk, PROVIDER_XAI, &xai_scope()).as_deref(),
        Some(XAI_FAKE),
        "xAI must remain after Codex logout"
    );
    assert!(
        pointer_key(&on_disk, PROVIDER_CODEX, &codex_scope()).is_none(),
        "Codex slot must be cleared"
    );

    // reverse: restore dual, clear xAI, Codex remains
    write_dual_auth_fixture(&path);
    let xai_out = logout_provider_slot(&path, AuthProvider::Xai).expect("logout xai");
    assert!(xai_out.was_logged_in);
    assert_eq!(xai_out.provider, Some(AuthProvider::Xai));
    let on_disk: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(
        pointer_key(&on_disk, PROVIDER_CODEX, &codex_scope()).as_deref(),
        Some(CODEX_FAKE),
        "Codex must remain after xAI logout"
    );
    assert!(
        pointer_key(&on_disk, PROVIDER_XAI, &xai_scope()).is_none(),
        "xAI slot must be cleared"
    );
}

/// Logout --all clears both provider slots atomically (one-lock clear_all).
#[test]
fn logout_all_clears_both() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("auth.json");
    write_dual_auth_fixture(&path);

    let out = logout_all_provider_slots(&path).expect("logout --all");
    assert!(out.was_logged_in);
    assert!(out.cleared_all);
    assert_eq!(out.provider, None);
    assert_eq!(
        format_dual_logout_message(&out),
        "Logged out of all providers"
    );
    assert!(
        !path.exists(),
        "atomic dual clear must prune empty auth.json"
    );
}

/// Bare logout must fail closed (usage + non-zero) without dual wipe.
#[test]
fn bare_logout_fail_closed() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("auth.json");
    write_dual_auth_fixture(&path);
    let before = std::fs::read(&path).unwrap();

    // Path-taking CLI handler: bare (no provider, no all) → Err + zero mutation.
    // Use a minimal config; grok_com_config is only needed for AuthManager
    // secondary invalidate which bare path never reaches.
    let config = xai_grok_shell::agent::config::Config::default();
    let err = xai_grok_shell::auth::run_cli_logout_at_path(&path, &config, None, false)
        .expect_err("bare logout must fail closed");
    let msg = format!("{err:#}");
    assert!(
        msg.contains("Specify a provider or clear all")
            || msg.contains("bum logout --provider"),
        "bare logout must surface UI-SPEC usage, got: {msg}"
    );
    assert_eq!(
        std::fs::read(&path).unwrap(),
        before,
        "bare logout must not mutate either provider slot"
    );
    // Usage helper matches UI-SPEC body.
    assert!(bare_logout_usage().contains("bum logout --all"));
    assert!(bare_logout_usage().contains("bum logout --provider xai"));
    assert!(bare_logout_usage().contains("bum logout --provider codex"));
}

// ── AUTH-04: Status pure model (Plan 02 GREEN; CLI handler Plan 04) ────────

/// Status formatter is greppable, lists both providers, never prints secrets.
#[test]
fn auth_status_format_paste_safe() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("auth.json");
    write_dual_auth_fixture(&path);

    let report = AuthStatusReport::from_auth_file(&path).expect("status from file");
    let text = format_auth_status(&report);
    // UI-SPEC labels are `xAI` / `Codex` (not bare wire keys alone).
    let has_both = text.contains("xAI:") && text.contains("Codex:");
    let has_keys = text.contains("logged_in:")
        && text.contains("usable:")
        && text.contains("account:")
        && text.contains("plan:");
    let no_secrets = !text.contains(XAI_FAKE)
        && !text.contains(CODEX_FAKE)
        && !text.contains(CODEX_REFRESH)
        && !text.contains("xai-refresh-token");
    assert!(
        has_both && has_keys && no_secrets && !text.is_empty(),
        "auth_status_format_paste_safe — greppable dual-provider status without \
         access/refresh token substrings (auth_file={}, text={text:?})",
        path.display()
    );
    // Account emails are OK to show; tokens are not.
    assert!(text.contains("xai@example.test") || text.contains("codex@example.test"));
}

/// Empty store still lists both providers with logged_in: no.
#[test]
fn auth_status_both_providers_always_listed() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("auth.json");
    // Missing file → empty report.
    let report = AuthStatusReport::from_auth_file(&path).expect("missing file is empty");
    let text = report.format();
    assert!(text.contains("xAI:"));
    assert!(text.contains("Codex:"));
    assert_eq!(text.matches("logged_in: no").count(), 2);
    assert_eq!(text.matches("usable: no").count(), 2);
}

/// Asymmetric login: one yes, one no.
#[test]
fn auth_status_asymmetric_login() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("auth.json");
    let mut xai: AuthStore = BTreeMap::new();
    xai.insert(xai_scope(), sample_xai_auth());
    let document = serde_json::json!({
        "version": 1,
        "providers": { "xai": xai }
    });
    std::fs::write(&path, serde_json::to_vec_pretty(&document).unwrap()).unwrap();

    let report = AuthStatusReport::from_auth_file(&path).expect("status");
    let xai_st = &report.providers[0];
    let codex_st = &report.providers[1];
    assert_eq!(xai_st.provider, AuthProvider::Xai);
    assert_eq!(codex_st.provider, AuthProvider::Codex);
    assert!(xai_st.logged_in && xai_st.usable);
    assert!(!codex_st.logged_in && !codex_st.usable);
    let text = report.format();
    assert!(text.contains("xAI:\n  logged_in: yes"));
    assert!(text.contains("Codex:\n  logged_in: no"));
}

/// CLI status handler path (run_cli_auth_status) must return dual status text.
///
/// Test name kept for Plan 04 gate filter: `run_cli_auth_status`.
#[test]
fn run_cli_auth_status() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("auth.json");
    write_dual_auth_fixture(&path);

    let text = xai_grok_shell::auth::run_cli_auth_status(&path).expect("status handler");
    assert!(text.contains("xAI:"), "must list xAI block");
    assert!(text.contains("Codex:"), "must list Codex block");
    assert!(text.contains("logged_in: yes"));
    assert!(text.contains("usable: yes"));
    assert!(text.contains("account:"));
    assert!(text.contains("plan:"));
    // Paste-safe: never emit access/refresh token substrings.
    assert!(!text.contains(XAI_FAKE));
    assert!(!text.contains(CODEX_FAKE));
    assert!(!text.contains(CODEX_REFRESH));
    assert!(!text.contains("xai-refresh-token"));

    // Injectable Write path (same production handler).
    let mut buf = Vec::new();
    write_cli_auth_status(&path, &mut buf).expect("write status");
    let written = String::from_utf8(buf).unwrap();
    assert!(written.contains("xAI:") && written.contains("Codex:"));
    assert!(!written.contains(XAI_FAKE));
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

    assert!(
        credential_usable(&auth),
        "expired access + refresh_token present must report usable=yes"
    );

    let mut store: AuthStore = BTreeMap::new();
    store.insert(codex_scope(), auth);
    let st = inspect_provider_store(AuthProvider::Codex, Some(&store));
    assert!(st.logged_in, "session record present → logged_in");
    assert!(st.usable, "refreshable OAuth → usable");
}

/// usable=no when expired and no refresh_token.
#[test]
fn auth_status_usable_expired_no_refresh() {
    let mut auth = sample_codex_auth();
    auth.expires_at = Some(Utc::now() - Duration::minutes(5));
    auth.refresh_token = None;

    assert!(
        !credential_usable(&auth),
        "expired access without refresh_token must report usable=no"
    );

    let mut store: AuthStore = BTreeMap::new();
    store.insert(codex_scope(), auth);
    let st = inspect_provider_store(AuthProvider::Codex, Some(&store));
    assert!(st.logged_in, "nonblank access key still means logged_in");
    assert!(!st.usable, "hard-expired without refresh → not usable");
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
