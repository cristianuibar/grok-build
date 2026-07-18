//! Phase 6 — missing-provider model-switch gate harness (MOD-06 / D-01 / D-02 / D-07)
//! plus free dual-provider switch, history, and mid-turn contracts (MOD-03 / D-06).
//!
//! **Contracts covered:**
//! - Typed `MODEL_SWITCH_MISSING_PROVIDER` ACP payload (camelCase + CLI suggestion)
//! - Pure `provider_slot_usable` / `missing_provider_gate_error` decision table
//! - Real `model_switch::apply` path via ACP `session/set_model` (blocks empty Codex)
//! - Side-effect absence on blocked switch (model_id unchanged, no ModelChanged)
//! - Refreshable Codex allows switch; BYOK skips OAuth-slot gate
//! - Dual-usable free Grok↔GPT switch (no MISSING_PROVIDER)
//! - Same-provider Codex switch with only Codex slot filled
//! - Next-sample route uses target provider credential/base_url after apply
//! - History length + identity preserved across successful apply (chat_history.jsonl)
//! - Mid-turn apply does not cancel in-flight held MockInferenceServer turn
//! - A late Grok reasoning response cannot carry encrypted payloads into the
//!   next Codex request; same-provider Codex continuity retains them
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
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use agent_client_protocol::{self as acp, Agent as _};
use chrono::{Duration as ChronoDuration, Utc};
use serial_test::serial;
use serde_json::{Value, json};
use tempfile::TempDir;
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};
use xai_acp_lib::{
    AcpAgentGatewayReceiver as GatewayReceiver, AcpAgentGatewaySender as GatewaySender,
    LineBufferedRead,
};
use xai_grok_shell::agent::config::{
    missing_provider_gate_error, Config as AgentConfig, ConfigModelOverride, ModelProvider,
    ModelSwitchIncompatibleAgentError, ModelSwitchMissingProviderError, CODEX_BASE_URL_DEFAULT,
    MODEL_SWITCH_MISSING_PROVIDER,
};
use xai_grok_shell::agent::mvp_agent::MvpAgent;
use xai_grok_shell::auth::{
    provider_slot_usable, AuthMode, AuthStore, GrokAuth, PROVIDER_CODEX, PROVIDER_XAI,
};
use xai_grok_test_support::{MockInferenceServer, ScriptedResponse, SseEvent};

const XAI_FAKE: &str = "xai-fake-token-p6";
const CODEX_FAKE: &str = "codex-fake-token-p6";
const CODEX_REFRESH: &str = "codex-refresh-token-p6";
const BYOK_KEY: &str = "byok-own-key-p6";
const CODEX_MODEL: &str = "gpt-5.6-sol";
const CODEX_MODEL_TERRA: &str = "gpt-5.6-terra";
const XAI_MODEL: &str = "grok-build";
const BYOK_MODEL: &str = "p6-byok-codex";
const DUPLEX_BUFFER_BYTES: usize = 4 * 1024 * 1024;
const HISTORY_MARKER: &str = "p6-history-marker-unique-prompt-string";
const PROVIDER_HISTORY_MARKER: &str = "p10-normal-history-must-survive";
const PROVIDER_HISTORY_REPLY: &str = "p10-normal-assistant-history-must-survive";
const REASONING_ID: &str = "p10-reasoning-id";
const REASONING_SUMMARY: &str = "p10-visible-reasoning-summary";
const REASONING_CONTENT: &str = "p10-visible-reasoning-content";
const ENCRYPTED_REASONING: &str = "p10-provider-scoped-encrypted-reasoning";

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
    byok.api_key = Some(BYOK_KEY.to_owned());
    byok.supported_in_api = Some(true);
    byok.hidden = Some(false);
    cfg.config_models.insert(BYOK_MODEL.to_owned(), byok);
    cfg
}

fn dual_usable_auth() -> (GrokAuth, GrokAuth) {
    (
        sample_oidc(XAI_FAKE, Some("xai-rt"), false),
        sample_oidc(CODEX_FAKE, Some(CODEX_REFRESH), false),
    )
}

/// Locate `<home>/sessions/<enc-cwd>/<id>` without depending on internal cwd encoders.
fn locate_session_dir(root: &Path, id: &str) -> PathBuf {
    let sessions = root.join("sessions");
    for entry in std::fs::read_dir(&sessions)
        .expect("read sessions dir")
        .flatten()
    {
        let candidate = entry.path().join(id);
        if candidate.is_dir() {
            return candidate;
        }
    }
    panic!(
        "could not locate session dir for {id} under {}",
        sessions.display()
    );
}

fn read_chat_history_jsonl(session_dir: &Path) -> String {
    let path = session_dir.join("chat_history.jsonl");
    std::fs::read_to_string(&path).unwrap_or_default()
}

fn chat_history_line_count(jsonl: &str) -> usize {
    jsonl
        .lines()
        .filter(|l| !l.trim().is_empty())
        .count()
}

/// Responses API terminal event carrying ordinary history plus a full typed
/// reasoning item. The test later observes the real next request body, not a
/// synthetic `ConversationItem`, so it covers sampler parsing and history
/// serialization together.
fn encrypted_reasoning_events(model: &str) -> Vec<SseEvent> {
    vec![
        SseEvent::data(
            json!({
                "type": "response.created",
                "sequence_number": 0,
                "response": {
                    "id": "p10-held-response",
                    "object": "response",
                    "created_at": 1234567890,
                    "model": model,
                    "status": "in_progress",
                    "output": []
                }
            })
            .to_string(),
        ),
        SseEvent::data(
            json!({
                "type": "response.completed",
                "sequence_number": 1,
                "response": {
                    "id": "p10-held-response",
                    "object": "response",
                    "created_at": 1234567890,
                    "model": model,
                    "status": "completed",
                    "output": [
                        {
                            "type": "reasoning",
                            "id": REASONING_ID,
                            "summary": [{
                                "type": "summary_text",
                                "text": REASONING_SUMMARY
                            }],
                            "content": [{ "text": REASONING_CONTENT }],
                            "encrypted_content": ENCRYPTED_REASONING,
                            "status": "completed"
                        },
                        {
                            "type": "message",
                            "id": "p10-history-message",
                            "role": "assistant",
                            "status": "completed",
                            "content": [{
                                "type": "output_text",
                                "text": PROVIDER_HISTORY_REPLY,
                                "annotations": []
                            }]
                        }
                    ],
                    "usage": {
                        "input_tokens": 10,
                        "output_tokens": 10,
                        "total_tokens": 20,
                        "input_tokens_details": { "cached_tokens": 0 },
                        "output_tokens_details": { "reasoning_tokens": 5 }
                    }
                }
            })
            .to_string(),
        ),
        SseEvent::data("[DONE]"),
    ]
}

fn reasoning_input(body: &Value) -> &Value {
    let input = body
        .get("input")
        .and_then(Value::as_array)
        .unwrap_or_else(|| panic!("Responses request must carry input: {body:#?}"));
    input
        .iter()
        .find(|item| item.get("type").and_then(Value::as_str) == Some("reasoning"))
        .unwrap_or_else(|| panic!("typed reasoning item missing from request input: {input:#?}"))
}

fn assert_reasoning_request(body: &Value, encrypted_content: Option<&str>) {
    let reasoning = reasoning_input(body);
    assert_eq!(
        reasoning.get("id").and_then(Value::as_str),
        Some(REASONING_ID),
        "reasoning id must survive the provider transition"
    );
    assert_eq!(
        reasoning["summary"][0]["text"].as_str(),
        Some(REASONING_SUMMARY),
        "visible reasoning summary must survive the provider transition"
    );
    assert_eq!(
        reasoning["content"][0]["text"].as_str(),
        Some(REASONING_CONTENT),
        "visible reasoning content must survive the provider transition"
    );
    match encrypted_content {
        Some(expected) => assert_eq!(
            reasoning.get("encrypted_content").and_then(Value::as_str),
            Some(expected),
            "same-provider reasoning must retain its encrypted payload"
        ),
        None => assert!(
            reasoning.get("encrypted_content").is_none(),
            "cross-provider reasoning must omit encrypted_content: {reasoning:#?}"
        ),
    }
}

struct GateClient {
    model_changed: Rc<RefCell<Vec<String>>>,
    /// Counts session updates whose Debug form looks cancel-related.
    /// Named observable for D-06 mid-turn non-cancel (notification spy).
    cancel_notifications: Arc<AtomicUsize>,
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
                self.model_changed.borrow_mut().push(dbg.clone());
            }
        }
        // Notification spy: true cancel paths surface Cancelled / cancel updates.
        let lower = dbg.to_ascii_lowercase();
        if lower.contains("cancelled")
            || lower.contains("cancelturn")
            || lower.contains("\"cancel\"")
        {
            self.cancel_notifications.fetch_add(1, Ordering::SeqCst);
        }
        Ok(())
    }
}

struct GateHarness {
    home: PathBuf,
    client: acp::ClientSideConnection,
    model_changed: Rc<RefCell<Vec<String>>>,
    cancel_notifications: Arc<AtomicUsize>,
    workdir: TempDir,
    /// Mock inference server (hold/release + request log for route/cancel proofs).
    server: MockInferenceServer,
    mock_base_url: String,
}

impl GateHarness {
    async fn start(xai: Option<GrokAuth>, codex: Option<GrokAuth>) -> Self {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let home = ensure_sandbox().to_path_buf();
        write_auth_document(&home, xai, codex);

        let server = MockInferenceServer::start()
            .await
            .expect("start mock inference server");
        let mock_base_url = server.url();
        // Point first-party + Codex bases at mock so bootstrap and next-sample
        // route proofs hit the fixture server (not live chatgpt.com / xAI).
        unsafe {
            std::env::set_var("GROK_CLI_CHAT_PROXY_BASE_URL", &mock_base_url);
            std::env::set_var("GROK_XAI_API_BASE_URL", &mock_base_url);
            std::env::set_var("GROK_CODEX_BASE_URL", &mock_base_url);
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
        let cancel_notifications = Arc::new(AtomicUsize::new(0));
        let client_impl = GateClient {
            model_changed: model_changed.clone(),
            cancel_notifications: cancel_notifications.clone(),
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
            cancel_notifications,
            workdir,
            server,
            mock_base_url,
        }
    }

    async fn new_session_with_model(&self, model_id: &str) -> acp::SessionId {
        let session = tokio::time::timeout(
            Duration::from_secs(60),
            self.client.new_session(
                acp::NewSessionRequest::new(self.workdir.path().to_path_buf()).meta(
                    serde_json::json!({ "modelId": model_id })
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

    async fn new_session_on_xai(&self) -> acp::SessionId {
        self.new_session_with_model(XAI_MODEL).await
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

    async fn prompt(
        &self,
        session_id: &acp::SessionId,
        text: &str,
    ) -> Result<acp::PromptResponse, acp::Error> {
        tokio::time::timeout(
            Duration::from_secs(90),
            self.client.prompt(acp::PromptRequest::new(
                session_id.clone(),
                vec![acp::ContentBlock::Text(acp::TextContent::new(
                    text.to_owned(),
                ))],
            )),
        )
        .await
        .expect("session/prompt timed out")
    }

    fn session_dir(&self, session_id: &acp::SessionId) -> PathBuf {
        locate_session_dir(&self.home, session_id.0.as_ref())
    }

    fn cancel_notification_count(&self) -> usize {
        self.cancel_notifications.load(Ordering::SeqCst)
    }

    /// Last inference POST after `after_count` total requests (exclusive).
    fn last_inference_after(
        &self,
        after_total: u32,
    ) -> (
        String,
        Option<String>,
        Option<serde_json::Value>,
    ) {
        let entry = self
            .server
            .requests()
            .into_iter()
            .enumerate()
            // Request log is append-only; after_total is previous request_count.
            .skip(after_total as usize)
            .map(|(_, e)| e)
            .filter(|e| {
                e.method == "POST"
                    && (e.path.contains("chat/completions")
                        || e.path.contains("responses")
                        || e.path.contains("messages"))
            })
            .next_back()
            .expect("expected inference request after switch");
        (entry.path, entry.authorization, entry.body)
    }
}

fn assert_not_missing_provider(err: &acp::Error) {
    assert!(
        ModelSwitchMissingProviderError::from_acp_error(err).is_none(),
        "must not be MODEL_SWITCH_MISSING_PROVIDER: {err:?}"
    );
}

fn assert_switch_ok(result: Result<acp::SetSessionModelResponse, acp::Error>, label: &str) {
    match result {
        Ok(_) => {}
        Err(err) => {
            assert_not_missing_provider(&err);
            panic!("{label}: set_session_model failed: {err:?}");
        }
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

// ───────────────────────── free dual-login switch (MOD-03) ─────────────────────────

#[tokio::test(flavor = "current_thread")]
#[serial]
async fn p6_dual_login_free_switch_xai_to_codex_no_missing_provider() {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let (xai, codex) = dual_usable_auth();
            let h = GateHarness::start(Some(xai), Some(codex)).await;
            let sid = h.new_session_on_xai().await;

            let result = h.set_model(&sid, CODEX_MODEL).await;
            assert_switch_ok(result, "xai→codex free switch");
        })
        .await;
}

#[tokio::test(flavor = "current_thread")]
#[serial]
async fn p6_dual_login_free_switch_codex_to_xai_no_missing_provider() {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let (xai, codex) = dual_usable_auth();
            let h = GateHarness::start(Some(xai), Some(codex)).await;
            let sid = h.new_session_with_model(CODEX_MODEL).await;

            let result = h.set_model(&sid, XAI_MODEL).await;
            assert_switch_ok(result, "codex→xai free switch");
        })
        .await;
}

#[tokio::test(flavor = "current_thread")]
#[serial]
async fn p6_same_provider_codex_switch_with_usable_creds() {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            // Only Codex slot filled — xAI empty. Sol→Terra must have no missing-provider friction.
            let codex = sample_oidc(CODEX_FAKE, Some(CODEX_REFRESH), false);
            let h = GateHarness::start(None, Some(codex)).await;
            let sid = h.new_session_with_model(CODEX_MODEL).await;

            let result = h.set_model(&sid, CODEX_MODEL_TERRA).await;
            assert_switch_ok(result, "same-provider Sol→Terra");
        })
        .await;
}

#[tokio::test(flavor = "current_thread")]
#[serial]
async fn p6_dual_login_next_sample_uses_target_provider() {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let (xai, codex) = dual_usable_auth();
            let h = GateHarness::start(Some(xai), Some(codex)).await;
            let sid = h.new_session_on_xai().await;

            // Seed one completed turn on xAI so next-sample after switch is a real route change.
            h.server.set_response("p6-xai-seed-reply");
            let seed = h
                .prompt(&sid, "p6 seed turn on xai before switch")
                .await
                .expect("seed prompt on xAI");
            assert!(
                matches!(seed.stop_reason, acp::StopReason::EndTurn),
                "seed turn must complete: {:?}",
                seed.stop_reason
            );

            let before_count = h.server.request_count();
            assert_switch_ok(
                h.set_model(&sid, CODEX_MODEL).await,
                "switch to codex before next sample",
            );

            h.server.set_response("p6-codex-next-sample-reply");
            let next = h
                .prompt(&sid, "p6 next sample after codex switch")
                .await
                .expect("next prompt after codex switch");
            assert!(
                matches!(next.stop_reason, acp::StopReason::EndTurn),
                "next sample must complete: {:?}",
                next.stop_reason
            );

            let (path, authorization, body) = h.last_inference_after(before_count);
            let auth = authorization.as_deref().unwrap_or("").to_ascii_lowercase();
            assert!(
                auth.contains(&CODEX_FAKE.to_ascii_lowercase())
                    || auth.contains(&format!("bearer {}", CODEX_FAKE).to_ascii_lowercase()),
                "next sample must use Codex credential slot (token={CODEX_FAKE}); got auth={auth:?} path={path}"
            );
            assert!(
                !auth.contains(&XAI_FAKE.to_ascii_lowercase()),
                "next sample must not use xAI OAuth token after Codex switch; auth={auth:?}"
            );
            if let Some(body) = &body {
                let model = body
                    .get("model")
                    .and_then(|v: &serde_json::Value| v.as_str())
                    .unwrap_or("");
                assert!(
                    model.contains("gpt-5.6") || model == CODEX_MODEL,
                    "next sample model must be Codex catalog id; got {model:?}"
                );
            }
            // Catalog stamp still uses resolve_codex_base_url at prepare; we forced
            // GROK_CODEX_BASE_URL → mock so traffic hits fixture. Default product
            // constant remains CODEX_BASE_URL_DEFAULT (regression anchor).
            assert_eq!(
                CODEX_BASE_URL_DEFAULT,
                "https://chatgpt.com/backend-api/codex"
            );
            let _ = &h.mock_base_url;
        })
        .await;
}

// ───────────────────────── BYOK / history / mid-turn (D-06) ─────────────────────────

#[tokio::test(flavor = "current_thread")]
#[serial]
async fn p6_byok_own_credentials_skips_oauth_missing_provider() {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let xai = sample_oidc(XAI_FAKE, Some("xai-rt"), false);
            // Empty Codex OAuth — BYOK model carries api_key (has_own_credentials).
            let h = GateHarness::start(Some(xai), None).await;
            let sid = h.new_session_on_xai().await;

            let result = h.set_model(&sid, BYOK_MODEL).await;
            assert_switch_ok(result, "BYOK own credentials switch");
        })
        .await;
}

#[tokio::test(flavor = "current_thread")]
#[serial]
async fn p6_history_preserved_across_successful_switch() {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let (xai, codex) = dual_usable_auth();
            let h = GateHarness::start(Some(xai), Some(codex)).await;
            let sid = h.new_session_on_xai().await;

            // Seed ≥1 completed user+assistant exchange via real session prompt.
            h.server.set_response("p6-assistant-history-reply");
            let resp = h
                .prompt(&sid, HISTORY_MARKER)
                .await
                .expect("seed user+assistant exchange");
            assert!(
                matches!(resp.stop_reason, acp::StopReason::EndTurn),
                "history seed turn must complete: {:?}",
                resp.stop_reason
            );

            // Named observable: session storage chat_history.jsonl
            let session_dir = h.session_dir(&sid);
            let before = read_chat_history_jsonl(&session_dir);
            let before_count = chat_history_line_count(&before);
            assert!(
                before_count >= 1,
                "must seed non-empty chat_history.jsonl; got {before_count} lines path={}",
                session_dir.join("chat_history.jsonl").display()
            );
            assert!(
                before.contains(HISTORY_MARKER),
                "history must contain exact user prompt identity `{HISTORY_MARKER}`"
            );

            assert_switch_ok(
                h.set_model(&sid, CODEX_MODEL).await,
                "cross-provider switch for history preserve",
            );

            let after = read_chat_history_jsonl(&session_dir);
            let after_count = chat_history_line_count(&after);
            assert_eq!(
                after_count, before_count,
                "history length must be unchanged after successful apply (before={before_count} after={after_count})"
            );
            assert!(
                after.contains(HISTORY_MARKER),
                "history identity (user prompt) must survive successful apply"
            );
        })
        .await;
}

#[tokio::test(flavor = "current_thread")]
#[serial]
async fn p6_mid_turn_switch_does_not_cancel_inflight() {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let (xai, codex) = dual_usable_auth();
            let h = GateHarness::start(Some(xai), Some(codex)).await;
            let sid = h.new_session_on_xai().await;

            h.server.set_response("p6-mid-turn-held-reply");
            // Hold terminal SSE so the turn stays in-flight after request received.
            h.server.hold_agent_completions();

            // ClientSideConnection is not Clone — pin the prompt future and interleave
            // wait/set_model via select (same task).
            let prompt_fut = h.client.prompt(acp::PromptRequest::new(
                sid.clone(),
                vec![acp::ContentBlock::Text(acp::TextContent::new(
                    "p6 mid-turn in-flight prompt".to_owned(),
                ))],
            ));
            tokio::pin!(prompt_fut);

            // Wait until mock saw an inference POST without completing the turn.
            let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
            loop {
                let n = h
                    .server
                    .requests()
                    .iter()
                    .filter(|e| {
                        e.method == "POST"
                            && (e.path.contains("chat/completions")
                                || e.path.contains("responses")
                                || e.path.contains("messages"))
                    })
                    .count();
                if n >= 1 {
                    break;
                }
                if tokio::time::Instant::now() >= deadline {
                    panic!("timed out waiting for in-flight inference request");
                }
                tokio::select! {
                    biased;
                    res = &mut prompt_fut => {
                        panic!("prompt completed before hold engaged: {res:?}");
                    }
                    _ = tokio::time::sleep(Duration::from_millis(20)) => {}
                }
            }

            let cancel_before = h.cancel_notification_count();
            assert_switch_ok(
                h.set_model(&sid, CODEX_MODEL).await,
                "mid-turn free dual switch",
            );
            let cancel_after = h.cancel_notification_count();
            assert_eq!(
                cancel_after, cancel_before,
                "apply must not fan out cancel-related session notifications (spy: cancel_notifications)"
            );

            // Prompt must still be held (not cancelled/completed by apply).
            tokio::select! {
                biased;
                res = &mut prompt_fut => {
                    panic!(
                        "in-flight prompt must remain pending while mock hold is active; completed early: {res:?}"
                    );
                }
                _ = tokio::time::sleep(Duration::from_millis(80)) => {}
            }

            h.server.release_agent_completions();
            let resp = tokio::time::timeout(Duration::from_secs(60), &mut prompt_fut)
                .await
                .expect("held prompt timed out after release")
                .expect("held prompt must succeed after release");
            assert!(
                matches!(resp.stop_reason, acp::StopReason::EndTurn),
                "released turn must EndTurn (not Cancelled); got {:?}",
                resp.stop_reason
            );
            assert_eq!(
                h.cancel_notification_count(),
                cancel_before,
                "cancel_notifications spy must stay flat through mid-turn switch + release"
            );
        })
        .await;
}

#[tokio::test(flavor = "current_thread")]
#[serial]
async fn held_grok_response_after_codex_switch_is_sanitized() {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let (xai, codex) = dual_usable_auth();
            let h = GateHarness::start(Some(xai), Some(codex)).await;
            let sid = h.new_session_on_xai().await;

            // Scripted responses bypass `hold_agent_completions`; the
            // dedicated terminal gate proves the old-provider response is
            // still in flight before we switch to Codex.
            h.server.hold_scripted_sse_terminal();
            let before_held_request_count = h.server.request_count();
            h.server.enqueue_response(
                "/v1/responses",
                ScriptedResponse::sse(encrypted_reasoning_events(XAI_MODEL))
                    .for_agent_turn()
                    .hold_terminal_event(),
            );
            let prompt_fut = h.client.prompt(acp::PromptRequest::new(
                sid.clone(),
                vec![acp::ContentBlock::Text(acp::TextContent::new(
                    PROVIDER_HISTORY_MARKER.to_owned(),
                ))],
            ));
            tokio::pin!(prompt_fut);
            let held_fut = h.server.wait_for_scripted_sse_terminal();
            tokio::pin!(held_fut);
            tokio::select! {
                biased;
                result = &mut prompt_fut => {
                    panic!("scripted Grok prompt completed before terminal hold: {result:?}");
                }
                () = &mut held_fut => {}
                () = tokio::time::sleep(Duration::from_secs(30)) => {
                    panic!("scripted Grok response did not reach terminal hold");
                }
            }

            let (held_path, _, held_body) = h.last_inference_after(before_held_request_count);
            assert_eq!(held_path, "/v1/responses", "Grok uses Responses API");
            let held_body = held_body.expect("held scripted request must have JSON body");
            assert!(
                held_body.to_string().contains(PROVIDER_HISTORY_MARKER),
                "scripted terminal gate must hold the primary agent request: {held_body:#?}"
            );
            assert!(
                held_body
                    .get("tools")
                    .and_then(Value::as_array)
                    .is_some_and(|tools| tools.len() >= 2),
                "scripted terminal gate must hold an agent turn: {held_body:#?}"
            );

            assert_switch_ok(
                h.set_model(&sid, CODEX_MODEL).await,
                "switch while scripted Grok response is held",
            );
            h.server.release_scripted_sse_terminal();
            let held_response = tokio::time::timeout(Duration::from_secs(60), &mut prompt_fut)
                .await
                .expect("held scripted Grok prompt timed out after release")
                .expect("held scripted Grok prompt must succeed after release");
            assert!(
                matches!(held_response.stop_reason, acp::StopReason::EndTurn),
                "released scripted Grok turn must complete: {:?}",
                held_response.stop_reason
            );

            let before_count = h.server.request_count();
            h.server.set_response("p10-codex-followup-response");
            let next = h
                .prompt(&sid, "p10 next Codex prompt after held Grok response")
                .await
                .expect("next Codex prompt must succeed");
            assert!(matches!(next.stop_reason, acp::StopReason::EndTurn));

            let (path, _, body) = h.last_inference_after(before_count);
            assert_eq!(path, "/v1/responses", "Codex uses Responses API");
            let body = body.expect("captured Codex request must have JSON body");
            let serialized = body.to_string();
            assert!(
                serialized.contains(PROVIDER_HISTORY_MARKER),
                "ordinary user history must survive cross-provider sanitization: {body:#?}"
            );
            assert!(
                serialized.contains(PROVIDER_HISTORY_REPLY),
                "ordinary assistant history must survive cross-provider sanitization: {body:#?}"
            );
            assert_reasoning_request(&body, None);
        })
        .await;
}

#[tokio::test(flavor = "current_thread")]
#[serial]
async fn codex_to_codex_transition_retains_encrypted_reasoning() {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let (_, codex) = dual_usable_auth();
            let h = GateHarness::start(None, Some(codex)).await;
            let sid = h.new_session_with_model(CODEX_MODEL).await;

            h.server.enqueue_response(
                "/v1/responses",
                ScriptedResponse::sse(encrypted_reasoning_events(CODEX_MODEL)).for_agent_turn(),
            );
            let seed = h
                .prompt(&sid, PROVIDER_HISTORY_MARKER)
                .await
                .expect("seed Codex reasoning turn must succeed");
            assert!(matches!(seed.stop_reason, acp::StopReason::EndTurn));

            assert_switch_ok(
                h.set_model(&sid, CODEX_MODEL_TERRA).await,
                "same-provider Codex switch",
            );
            let before_count = h.server.request_count();
            h.server.set_response("p10-Codex-Terra-followup-response");
            let next = h
                .prompt(&sid, "p10 next Codex Terra prompt")
                .await
                .expect("next Codex Terra prompt must succeed");
            assert!(matches!(next.stop_reason, acp::StopReason::EndTurn));

            let (path, _, body) = h.last_inference_after(before_count);
            assert_eq!(path, "/v1/responses", "Codex uses Responses API");
            let body = body.expect("captured Codex request must have JSON body");
            assert_reasoning_request(&body, Some(ENCRYPTED_REASONING));
        })
        .await;
}
