//! Phase 7 — Wave 1 green harness for cross-provider multi-agent orchestration.
//!
//! **Contracts covered (compile-safe green scaffold only):**
//! - Dual-fixture BUM_HOME + auth.json smoke (`xai-fake-token` / `codex-fake-token`)
//! - Pure `missing_provider_gate_error` login-hint decision (reuse Phase 6 public API)
//! - Same-provider usable path → gate None (D-08 pure helper lock)
//! - Public Phase 4 route isolation (base_url host + credential_slot differ both ways)
//!
//! **Not in this file (later plans):**
//! - Plan 03: spawn-time missing-provider gate on handle_subagent_request
//! - Plan 04: eager Task preflight for child credentials
//! - Plan 05: dual-token mock HTTP Authorization proofs
//! - Plan 06: full same-provider spawn/resume lifecycle gate
//!
//! Prefer:
//! ```text
//! cargo test -p xai-grok-shell --test cross_provider_subagent p7_ -- --nocapture
//! ```
//!
//! ## BUM_HOME / OnceLock hygiene
//!
//! This binary sets a process-wide sandbox once via `ensure_sandbox()`. Do not
//! flip `BUM_HOME` to a second path mid-process (`grok_home()` is OnceLock).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use chrono::{Duration as ChronoDuration, Utc};
use serial_test::serial;
use tempfile::TempDir;
use xai_grok_shell::agent::config::{
    missing_provider_gate_error, EndpointsConfig, ModelProvider, CODEX_BASE_URL_DEFAULT,
    CLI_CHAT_PROXY_BASE_URL_DEFAULT, XAI_API_BASE_URL_DEFAULT, resolve_provider_route,
};
use xai_grok_shell::auth::{
    provider_slot_usable, read_provider_auth_store, select_provider_access_token, AuthMode,
    AuthStore, GrokAuth, PROVIDER_CODEX, PROVIDER_XAI,
};

const XAI_FAKE: &str = "xai-fake-token-p7";
const CODEX_FAKE: &str = "codex-fake-token-p7";
const CODEX_REFRESH: &str = "codex-refresh-token-p7";
const CODEX_MODEL: &str = "gpt-5.6-sol";
const XAI_MODEL: &str = "grok-build";

// ───────────────────────── fixtures ─────────────────────────

fn sample_oidc(key: &str, refresh: Option<&str>, expired: bool) -> GrokAuth {
    GrokAuth {
        key: key.to_owned(),
        auth_mode: AuthMode::Oidc,
        create_time: Utc::now(),
        user_id: "p7-user".to_owned(),
        email: Some("p7@example.test".to_owned()),
        expires_at: Some(if expired {
            Utc::now() - ChronoDuration::minutes(5)
        } else {
            Utc::now() + ChronoDuration::hours(1)
        }),
        refresh_token: refresh.map(str::to_owned),
        oidc_issuer: Some("https://auth.example.test".to_owned()),
        oidc_client_id: Some("p7-client".to_owned()),
        ..Default::default()
    }
}

/// Process-wide product home for this integration binary (OnceLock-safe).
fn ensure_sandbox() -> &'static Path {
    static SANDBOX: OnceLock<TempDir> = OnceLock::new();
    let dir = SANDBOX.get_or_init(|| {
        let d = TempDir::new().expect("tempdir");
        // SAFETY: this test binary owns BUM_HOME; set once before any grok_home().
        unsafe {
            std::env::set_var("BUM_HOME", d.path());
            std::env::set_var("GROK_TELEMETRY_ENABLED", "false");
            std::env::set_var("GROK_FEEDBACK_ENABLED", "false");
            std::env::set_var("GROK_TRACE_UPLOAD", "false");
            std::env::set_var("XAI_API_KEY", "p7-xai-api-key");
        }
        d
    });
    dir.path()
}

fn write_auth_document(home: &Path, xai: Option<GrokAuth>, codex: Option<GrokAuth>) {
    let mut providers = BTreeMap::new();
    if let Some(auth) = xai {
        let mut store = AuthStore::new();
        store.insert("xai::fixture".to_owned(), auth);
        providers.insert(PROVIDER_XAI.to_owned(), store);
    }
    if let Some(auth) = codex {
        let mut store = AuthStore::new();
        store.insert("codex::fixture".to_owned(), auth);
        providers.insert(PROVIDER_CODEX.to_owned(), store);
    }
    let document = serde_json::json!({
        "version": 1,
        "providers": providers,
    });
    std::fs::write(
        home.join("auth.json"),
        serde_json::to_vec_pretty(&document).expect("serialize auth"),
    )
    .expect("write auth.json");
}

fn auth_path(home: &Path) -> PathBuf {
    home.join("auth.json")
}

fn deterministic_endpoints() -> EndpointsConfig {
    EndpointsConfig {
        cli_chat_proxy_base_url: Some(CLI_CHAT_PROXY_BASE_URL_DEFAULT.to_owned()),
        xai_api_base_url: XAI_API_BASE_URL_DEFAULT.to_owned(),
        codex_base_url: CODEX_BASE_URL_DEFAULT.to_owned(),
        ..EndpointsConfig::default()
    }
}

// ───────────────────────── pure / unit-style ─────────────────────────

/// AGENT-05 pure decision: empty Codex slot → login-shaped error (D-05/D-07).
#[test]
fn p7_missing_provider_gate_error_suggests_bum_login_for_empty_codex() {
    let err = missing_provider_gate_error(ModelProvider::Codex, CODEX_MODEL, false, false)
        .expect("empty Codex slot must fail closed");
    assert_eq!(err.provider, "codex");
    assert_eq!(err.model_id, CODEX_MODEL);
    assert_eq!(err.suggestion, "bum login --provider codex");
    let msg = err.into_acp_error().message;
    assert!(
        msg.contains("bum login --provider codex"),
        "message must include CLI suggestion: {msg}"
    );
}

/// AGENT-05 pure decision: usable Codex slot → gate None.
#[test]
fn p7_missing_provider_gate_error_none_when_slot_usable() {
    assert!(
        missing_provider_gate_error(ModelProvider::Codex, CODEX_MODEL, false, true).is_none(),
        "usable slot must not block"
    );
    assert!(
        missing_provider_gate_error(ModelProvider::Xai, XAI_MODEL, false, true).is_none(),
        "usable xAI slot must not block"
    );
}

/// AGENT-01 / D-08: same-provider usable path is friction-free (pure helper).
#[test]
fn p7_spawn_same_provider_no_extra_friction_when_parent_usable() {
    // Parent and child share Codex; slot usable → no missing-provider gate.
    assert!(
        missing_provider_gate_error(ModelProvider::Codex, CODEX_MODEL, false, true).is_none()
    );
    // Parent and child share xAI; slot usable → no missing-provider gate.
    assert!(
        missing_provider_gate_error(ModelProvider::Xai, XAI_MODEL, false, true).is_none()
    );
    // BYOK has_own_credentials short-circuits even when slot empty (same as Phase 6).
    assert!(
        missing_provider_gate_error(ModelProvider::Codex, CODEX_MODEL, true, false).is_none()
    );
}

/// D-09 route isolation via public Phase 4 resolvers (not private override→config).
///
/// Full dual-token Authorization proofs are Plan 05; this locks base_url host and
/// credential_slot differ for Grok vs gpt-5.6-sol catalog providers.
#[test]
fn p7_resolve_route_isolates_base_url_key_prefix_both_directions() {
    let endpoints = deterministic_endpoints();

    let xai_route = resolve_provider_route(ModelProvider::Xai, &endpoints, None);
    let codex_route = resolve_provider_route(ModelProvider::Codex, &endpoints, None);

    assert_eq!(xai_route.credential_slot, "xai");
    assert_eq!(codex_route.credential_slot, "codex");
    assert_ne!(
        xai_route.credential_slot, codex_route.credential_slot,
        "credential slots must differ across providers"
    );

    let xai_host = reqwest::Url::parse(&xai_route.base_url)
        .expect("xai base_url parse")
        .host_str()
        .unwrap_or("")
        .to_owned();
    let codex_host = reqwest::Url::parse(&codex_route.base_url)
        .expect("codex base_url parse")
        .host_str()
        .unwrap_or("")
        .to_owned();
    assert!(
        !xai_host.is_empty() && !codex_host.is_empty(),
        "both routes must have hosts; xai={xai_host:?} codex={codex_host:?}"
    );
    assert_ne!(
        xai_host, codex_host,
        "Grok and Codex stock bases must not share host (isolation stub)"
    );
    assert_eq!(codex_route.base_url, CODEX_BASE_URL_DEFAULT);
}

// ───────────────────────── harness smoke ─────────────────────────

/// Infrastructure smoke: dual-slot fixture tokens readable under BUM_HOME sandbox.
/// Plan 05 owns Authorization/routing proofs; this only locks fixture plumbing.
#[test]
#[serial]
fn p7_wave0_harness_smoke_compiles_and_runs() {
    let home = ensure_sandbox();
    write_auth_document(
        home,
        Some(sample_oidc(XAI_FAKE, Some("xai-rt"), false)),
        Some(sample_oidc(CODEX_FAKE, Some(CODEX_REFRESH), false)),
    );

    let path = auth_path(home);
    let xai_store = read_provider_auth_store(&path, PROVIDER_XAI)
        .expect("xai store read")
        .expect("xai slot present");
    let codex_store = read_provider_auth_store(&path, PROVIDER_CODEX)
        .expect("codex store read")
        .expect("codex slot present");

    assert!(provider_slot_usable(Some(&xai_store)));
    assert!(provider_slot_usable(Some(&codex_store)));

    let xai_tok = select_provider_access_token(&xai_store).expect("xai token");
    let codex_tok = select_provider_access_token(&codex_store).expect("codex token");
    assert_eq!(xai_tok.key, XAI_FAKE);
    assert_eq!(codex_tok.key, CODEX_FAKE);
    // Never assert full Authorization header bodies (T-07-01).
}

/// Empty Codex slot readable as None from dual-document write (xAI only).
#[test]
#[serial]
fn p7_empty_codex_slot_reads_none_with_xai_present() {
    let home = ensure_sandbox();
    write_auth_document(
        home,
        Some(sample_oidc(XAI_FAKE, Some("xai-rt"), false)),
        None,
    );
    let path = auth_path(home);
    assert!(
        read_provider_auth_store(&path, PROVIDER_XAI)
            .expect("xai read")
            .is_some()
    );
    assert!(
        read_provider_auth_store(&path, PROVIDER_CODEX)
            .expect("codex read")
            .is_none(),
        "missing Codex provider key must read as None"
    );
}
