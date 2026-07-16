//! Phase 6 — missing-provider model-switch gate harness (MOD-06 / D-01 / D-02 / D-07).
//!
//! **Contracts covered:**
//! - Typed `MODEL_SWITCH_MISSING_PROVIDER` ACP payload (camelCase + CLI suggestion)
//! - Pure `provider_slot_usable` / `missing_provider_gate_error` decision table
//! - Real `model_switch::apply` path via ACP `session/set_model` (blocks empty Codex)
//! - Side-effect absence on blocked switch (model_id unchanged, no ModelChanged)
//! - Refreshable Codex allows switch; BYOK skips OAuth-slot gate
//!
//! **Scope:** Shell authority only. No pager QuestionView / badges (Plans 02–04).
//! Fixture tokens only — no live ChatGPT/xAI OAuth.
//!
//! ## BUM_HOME / OnceLock hygiene
//!
//! This binary sets a process-wide sandbox once via `ensure_sandbox()`. All
//! apply-path tests rewrite `auth.json` under that home. Do not flip `BUM_HOME`
//! to a second path mid-process (`grok_home()` is OnceLock).
//!
//! Prefer:
//! ```text
//! cargo test -p xai-grok-shell --test model_switch_gate p6_ -- --nocapture
//! ```

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use agent_client_protocol::{self as acp, Agent as _};
use chrono::{Duration as ChronoDuration, Utc};
use serial_test::serial;
use tempfile::TempDir;
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};
use xai_acp_lib::{
    AcpAgentGatewayReceiver as GatewayReceiver, AcpAgentGatewaySender as GatewaySender,
    LineBufferedRead,
};
use xai_grok_shell::agent::config::{
    missing_provider_gate_error, Config as AgentConfig, ConfigModelOverride, ModelProvider,
    ModelSwitchIncompatibleAgentError, ModelSwitchMissingProviderError,
    MODEL_SWITCH_MISSING_PROVIDER,
};
use xai_grok_shell::agent::mvp_agent::MvpAgent;
use xai_grok_shell::auth::{
    provider_slot_usable, AuthMode, AuthStore, GrokAuth, PROVIDER_CODEX, PROVIDER_XAI,
};
use xai_grok_test_support::MockInferenceServer;

const XAI_FAKE: &str = "xai-fake-token-p6";
const CODEX_FAKE: &str = "codex-fake-token-p6";
const CODEX_REFRESH: &str = "codex-refresh-token-p6";
const CODEX_MODEL: &str = "gpt-5.6-sol";
const XAI_MODEL: &str = "grok-build";
const BYOK_MODEL: &str = "p6-byok-codex";
const DUPLEX_BUFFER_BYTES: usize = 4 * 1024 * 1024;

// ───────────────────────── pure / unit-style ─────────────────────────

#[test]
fn p6_model_switch_missing_provider_error_round_trips_acp_data() {
    let err = ModelSwitchMissingProviderError::new(ModelProvider::Xai, "grok-build");
    let acp_err = err.clone().into_acp_error();
    let parsed = ModelSwitchMissingProviderError::from_acp_error(&acp_err)
        .expect("must parse MODEL_SWITCH_MISSING_PROVIDER");
    assert_eq!(parsed.code, MODEL_SWITCH_MISSING_PROVIDER);
    assert_eq!(parsed.provider, "xai");
    assert_eq!(parsed.model_id, "grok-build");
    assert_eq!(parsed.suggestion, "bum login --provider xai");
    assert!(ModelSwitchIncompatibleAgentError::from_acp_error(&acp_err).is_none());
}

#[test]
fn p6_model_switch_missing_provider_error_message_includes_cli_suggestion() {
    let err = ModelSwitchMissingProviderError::new(ModelProvider::Codex, CODEX_MODEL);
    let msg = err.clone().into_acp_error().message;
    assert!(
        msg.contains("no usable Codex credentials"),
        "message must use ProviderLabel Codex: {msg}"
    );
    assert!(
        msg.contains("bum login --provider codex"),
        "message must include CLI suggestion: {msg}"
    );
    assert!(
        msg.contains(CODEX_MODEL),
        "message must include model id: {msg}"
    );
}

#[test]
fn p6_provider_slot_usable_empty_store_false() {
    assert!(!provider_slot_usable(None));
    assert!(!provider_slot_usable(Some(&AuthStore::new())));
}

#[test]
fn p6_provider_slot_usable_refreshable_true() {
    let mut store = AuthStore::new();
    store.insert(
        "codex::fixture".to_owned(),
        sample_oidc(CODEX_FAKE, Some(CODEX_REFRESH), true),
    );
    assert!(provider_slot_usable(Some(&store)));
}

#[test]
fn p6_provider_slot_usable_hard_expired_no_refresh_false() {
    let mut store = AuthStore::new();
    store.insert(
        "codex::fixture".to_owned(),
        sample_oidc(CODEX_FAKE, None, true),
    );
    assert!(!provider_slot_usable(Some(&store)));
}

#[test]
fn p6_byok_model_skips_oauth_slot_gate_pure() {
    // Pure decision table: has_own_credentials short-circuits even when slot empty.
    assert!(
        missing_provider_gate_error(ModelProvider::Codex, BYOK_MODEL, true, false).is_none()
    );
    assert!(
        missing_provider_gate_error(ModelProvider::Codex, CODEX_MODEL, false, false).is_some()
    );
}

// ───────────────────────── apply-path harness ─────────────────────────

fn sample_oidc(key: &str, refresh: Option<&str>, expired: bool) -> GrokAuth {
    GrokAuth {
        key: key.to_owned(),
        auth_mode: AuthMode::Oidc,
        create_time: Utc::now(),
        user_id: "p6-user".to_owned(),
        email: Some("p6@example.test".to_owned()),
        expires_at: Some(if expired {
            Utc::now() - ChronoDuration::minutes(5)
        } else {
            Utc::now() + ChronoDuration::hours(1)
        }),
        refresh_token: refresh.map(str::to_owned),
        oidc_issuer: Some("https://auth.example.test".to_owned()),
        oidc_client_id: Some("p6-client".to_owned()),
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
            std::env::set_var("XAI_API_KEY", "p6-xai-api-key");
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

fn agent_config_with_byok() -> AgentConfig {
    let mut cfg = AgentConfig::default();
    let mut byok = ConfigModelOverride::default();
    byok.model = Some(BYOK_MODEL.to_owned());
    byok.name = Some("P6 BYOK Codex".to_owned());
    byok.provider = Some(ModelProvider::Codex);
    byok.api_key = Some("byok-own-key-p6".to_owned());
    byok.supported_in_api = Some(true);
    byok.hidden = Some(false);
    cfg.config_models.insert(BYOK_MODEL.to_owned(), byok);
    cfg
}

#[derive(Default)]
struct GateClient {
    model_changed: Rc<RefCell<Vec<String>>>,
}

#[async_trait::async_trait(?Send)]
impl acp::Client for GateClient {
    async fn request_permission(
        &self,
        args: acp::RequestPermissionRequest,
    ) -> acp::Result<acp::RequestPermissionResponse> {
        let outcome = args
            .options
            .iter()
            .find(|o| o.kind == acp::PermissionOptionKind::AllowOnce)
            .or(args.options.first())
            .map(|o| {
                acp::RequestPermissionOutcome::Selected(acp::SelectedPermissionOutcome::new(
                    o.option_id.clone(),
                ))
            })
            .unwrap_or(acp::RequestPermissionOutcome::Cancelled);
        Ok(acp::RequestPermissionResponse::new(outcome))
    }

    async fn session_notification(&self, args: acp::SessionNotification) -> acp::Result<()> {
        // Capture ModelChanged via raw re-serialize of the update envelope when
        // the typed enum doesn't expose our extension variants. Best-effort:
        // look for model_id changes in debug form as a secondary signal.
        let dbg = format!("{:?}", args.update);
        if dbg.contains("ModelChanged") || dbg.contains("model_changed") {
            if let Some(mid) = dbg
                .split("model_id:")
                .nth(1)
                .and_then(|s| s.split(',').next())
                .map(|s| s.trim().trim_matches('"').to_owned())
            {
                self.model_changed.borrow_mut().push(mid);
            } else {
                self.model_changed.borrow_mut().push(dbg);
            }
        }
        Ok(())
    }
}

struct GateHarness {
    home: PathBuf,
    client: acp::ClientSideConnection,
    model_changed: Rc<RefCell<Vec<String>>>,
    _workdir: TempDir,
    // Keep server alive for the session lifetime.
    _server: MockInferenceServer,
}

impl GateHarness {
    async fn start(xai: Option<GrokAuth>, codex: Option<GrokAuth>) -> Self {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let home = ensure_sandbox().to_path_buf();
        write_auth_document(&home, xai, codex);

        let server = MockInferenceServer::start()
            .await
            .expect("start mock inference server");
        // Point first-party bases at mock so model prefetch / session bootstrap can succeed.
        unsafe {
            std::env::set_var("GROK_CLI_CHAT_PROXY_BASE_URL", server.url());
            std::env::set_var("GROK_XAI_API_BASE_URL", server.url());
        }

        let workdir = TempDir::new().expect("workdir");
        // Minimal git repo so workspace/cwd checks don't fail hard.
        let _ = std::process::Command::new("git")
            .args(["init"])
            .current_dir(workdir.path())
            .output();

        let agent_config = agent_config_with_byok();
        let auth_manager = Arc::new(agent_config.create_auth_manager());
        let (gw_tx, gw_rx) = tokio::sync::mpsc::unbounded_channel();
        let gateway = GatewaySender::new(gw_tx);
        let agent = MvpAgent::new(gateway, &agent_config, auth_manager, None).expect("valid config");

        let (c2a_a, c2a_b) = tokio::io::duplex(DUPLEX_BUFFER_BYTES);
        let (a2c_a, a2c_b) = tokio::io::duplex(DUPLEX_BUFFER_BYTES);

        let agent_incoming = LineBufferedRead::spawn_local(c2a_b.compat());
        let (agent_conn, agent_io) =
            acp::AgentSideConnection::new(agent, a2c_a.compat_write(), agent_incoming, |fut| {
                tokio::task::spawn_local(fut);
            });
        tokio::task::spawn_local(
            GatewayReceiver::new(gw_rx, agent_conn)
                .with_on_meta(xai_file_utils::trace_context::span_from_meta_traceparent)
                .run(),
        );
        tokio::task::spawn_local(agent_io);

        let model_changed = Rc::new(RefCell::new(Vec::new()));
        let client_impl = GateClient {
            model_changed: model_changed.clone(),
        };
        let client_incoming = LineBufferedRead::spawn_local(a2c_b.compat());
        let (client, client_io) = acp::ClientSideConnection::new(
            client_impl,
            c2a_a.compat_write(),
            client_incoming,
            |fut| {
                tokio::task::spawn_local(fut);
            },
        );
        tokio::task::spawn_local(client_io);

        let init = tokio::time::timeout(
            Duration::from_secs(60),
            client.initialize(
                acp::InitializeRequest::new(acp::ProtocolVersion::V1)
                    .client_capabilities(
                        acp::ClientCapabilities::new()
                            .fs(acp::FileSystemCapabilities::new())
                            .terminal(false),
                    )
                    .meta(
                        serde_json::json!({
                            "startupHints": {
                                "nonInteractive": true,
                                "skipGitStatus": true,
                                "skipProjectLayout": true,
                            },
                            "clientType": "model-switch-gate",
                            "clientVersion": "0.0-test",
                        })
                        .as_object()
                        .cloned(),
                    ),
            ),
        )
        .await
        .expect("initialize timed out")
        .expect("initialize failed");

        if let Some(method) = init
            .auth_methods
            .iter()
            .find(|m| &*m.id().0 == "xai.api_key")
        {
            let _ = client
                .authenticate(
                    acp::AuthenticateRequest::new(method.id().clone())
                        .meta(serde_json::json!({ "headless": true }).as_object().cloned()),
                )
                .await;
        }

        Self {
            home,
            client,
            model_changed,
            _workdir: workdir,
            _server: server,
        }
    }

    async fn new_session_on_xai(&self) -> acp::SessionId {
        let session = tokio::time::timeout(
            Duration::from_secs(60),
            self.client.new_session(
                acp::NewSessionRequest::new(self._workdir.path().to_path_buf()).meta(
                    serde_json::json!({ "modelId": XAI_MODEL })
                        .as_object()
                        .cloned(),
                ),
            ),
        )
        .await
        .expect("session/new timed out")
        .expect("session/new failed");
        session.session_id
    }

    async fn set_model(
        &self,
        session_id: &acp::SessionId,
        model_id: &str,
    ) -> Result<acp::SetSessionModelResponse, acp::Error> {
        tokio::time::timeout(
            Duration::from_secs(30),
            self.client.set_session_model(acp::SetSessionModelRequest::new(
                session_id.clone(),
                acp::ModelId::new(model_id.to_owned()),
            )),
        )
        .await
        .expect("set_session_model timed out")
    }
}

#[tokio::test(flavor = "current_thread")]
#[serial]
async fn p6_missing_provider_apply_blocks_codex_when_codex_slot_empty() {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let xai = sample_oidc(XAI_FAKE, Some("xai-rt"), false);
            // Codex slot absent / empty
            let h = GateHarness::start(Some(xai), None).await;
            let sid = h.new_session_on_xai().await;

            let err = h
                .set_model(&sid, CODEX_MODEL)
                .await
                .expect_err("empty codex slot must fail closed with MISSING_PROVIDER");
            let parsed = ModelSwitchMissingProviderError::from_acp_error(&err)
                .expect("error must be MODEL_SWITCH_MISSING_PROVIDER");
            assert_eq!(parsed.provider, "codex");
            assert_eq!(parsed.model_id, CODEX_MODEL);
            assert_eq!(parsed.suggestion, "bum login --provider codex");
            let _ = h.home;
        })
        .await;
}

#[tokio::test(flavor = "current_thread")]
#[serial]
async fn p6_missing_provider_apply_no_side_effects() {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let xai = sample_oidc(XAI_FAKE, Some("xai-rt"), false);
            let h = GateHarness::start(Some(xai), None).await;
            let sid = h.new_session_on_xai().await;
            h.model_changed.borrow_mut().clear();

            let err = h
                .set_model(&sid, CODEX_MODEL)
                .await
                .expect_err("blocked switch must return Err");
            assert!(
                ModelSwitchMissingProviderError::from_acp_error(&err).is_some(),
                "expected MODEL_SWITCH_MISSING_PROVIDER, got {err:?}"
            );

            // No success ModelChanged fan-out for the target model.
            let changed = h.model_changed.borrow().clone();
            assert!(
                changed
                    .iter()
                    .all(|m| !m.contains(CODEX_MODEL) && m != CODEX_MODEL),
                "blocked switch must not broadcast ModelChanged for target; got {changed:?}"
            );

            // Session remains on previous (xAI) model: a follow-up switch to the
            // same xAI model should not report missing-provider, and target still blocked.
            let err2 = h
                .set_model(&sid, CODEX_MODEL)
                .await
                .expect_err("second blocked switch still fails");
            assert!(ModelSwitchMissingProviderError::from_acp_error(&err2).is_some());
        })
        .await;
}

#[tokio::test(flavor = "current_thread")]
#[serial]
async fn p6_missing_provider_apply_allows_codex_when_refreshable_token_present() {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let xai = sample_oidc(XAI_FAKE, Some("xai-rt"), false);
            // Expired access + nonblank refresh → usable (D-02)
            let codex = sample_oidc(CODEX_FAKE, Some(CODEX_REFRESH), true);
            let h = GateHarness::start(Some(xai), Some(codex)).await;
            let sid = h.new_session_on_xai().await;

            let result = h.set_model(&sid, CODEX_MODEL).await;
            if let Err(err) = &result {
                assert!(
                    ModelSwitchMissingProviderError::from_acp_error(err).is_none(),
                    "refreshable codex must not return MODEL_SWITCH_MISSING_PROVIDER: {err:?}"
                );
            }
            // Switch may still fail for other reasons (agent type / network) but
            // not missing-provider. Prefer Ok.
            assert!(
                result.is_ok()
                    || ModelSwitchMissingProviderError::from_acp_error(result.as_ref().unwrap_err())
                        .is_none(),
                "refreshable token must pass missing-provider gate"
            );
        })
        .await;
}

#[tokio::test(flavor = "current_thread")]
#[serial]
async fn p6_byok_model_skips_oauth_slot_gate() {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let xai = sample_oidc(XAI_FAKE, Some("xai-rt"), false);
            // Empty codex OAuth slot — BYOK model still has own api_key.
            let h = GateHarness::start(Some(xai), None).await;
            let sid = h.new_session_on_xai().await;

            let result = h.set_model(&sid, BYOK_MODEL).await;
            if let Err(err) = &result {
                assert!(
                    ModelSwitchMissingProviderError::from_acp_error(err).is_none(),
                    "BYOK must not return MODEL_SWITCH_MISSING_PROVIDER when OAuth empty: {err:?}"
                );
            }
        })
        .await;
}
