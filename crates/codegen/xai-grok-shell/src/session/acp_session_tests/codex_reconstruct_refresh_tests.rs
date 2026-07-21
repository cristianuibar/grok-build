//! AUTH-05 Option C seam: prove `SessionActor::reconstruct_full_config`
//! invokes ensure_fresh for Codex SessionToken + first-party host, and does
//! **not** override BYOK / custom-endpoint prepared credentials.
//!
//! Run narrow only:
//! ```text
//! cargo test -p xai-grok-shell --lib codex_reconstruct_refreshes_mid_session_expiry
//! cargo test -p xai-grok-shell --lib codex_byok_key_not_overridden
//! cargo test -p xai-grok-shell --lib codex_oauth_bearer_absent_on_custom_endpoint
//! ```

use super::support::*;
use super::*;
use crate::agent::auth_method::ModelByok;
use crate::agent::config::{
    clear_ensure_fresh_codex_test_hooks, set_ensure_fresh_codex_synthetic_success, ModelAuthFacts,
    ModelProvider, CODEX_BASE_URL_DEFAULT,
};
use crate::auth::{AuthMode, AuthProvider, GrokAuth, mutate_provider_store_or_prune};
use chrono::{Duration, Utc};
use indexmap::IndexMap;
use serial_test::serial;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;

const STALE_BEARER: &str = "stale-prepared-session-bearer";
const FRESH_BEARER: &str = "fresh-reconstruct-bearer";
const BYOK_KEY: &str = "model-owned-byok-key";
const CODEX_SCOPE: &str = "codex::fixture";

fn near_expiry_codex_auth() -> GrokAuth {
    GrokAuth {
        key: STALE_BEARER.to_owned(),
        auth_mode: AuthMode::Oidc,
        create_time: Utc::now(),
        user_id: "codex-user".to_owned(),
        email: Some("codex@example.test".to_owned()),
        organization_id: Some("chatgpt-acct-fixture".to_owned()),
        expires_at: Some(Utc::now() + Duration::minutes(2)),
        refresh_token: Some("codex-rt".to_owned()),
        oidc_issuer: Some(crate::auth::codex::CODEX_ISSUER.to_owned()),
        oidc_client_id: Some(crate::auth::codex::CODEX_CLIENT_ID.to_owned()),
        ..Default::default()
    }
}

fn fresh_codex_auth() -> GrokAuth {
    GrokAuth {
        key: FRESH_BEARER.to_owned(),
        auth_mode: AuthMode::Oidc,
        create_time: Utc::now(),
        user_id: "codex-user".to_owned(),
        email: Some("codex@example.test".to_owned()),
        organization_id: Some("chatgpt-acct-fixture".to_owned()),
        expires_at: Some(Utc::now() + Duration::hours(1)),
        refresh_token: Some("codex-rt-rotated".to_owned()),
        oidc_issuer: Some(crate::auth::codex::CODEX_ISSUER.to_owned()),
        oidc_client_id: Some(crate::auth::codex::CODEX_CLIENT_ID.to_owned()),
        ..Default::default()
    }
}

fn write_codex_slot(path: &std::path::Path, auth: GrokAuth) {
    mutate_provider_store_or_prune(path, AuthProvider::Codex, move |store| {
        store.insert(CODEX_SCOPE.to_owned(), auth);
    })
    .expect("write codex slot");
}

async fn make_codex_actor(
    api_key: &str,
    auth_type: xai_chat_state::AuthType,
    base_url: &str,
    provider: ModelProvider,
    byok: ModelByok,
) -> (
    Arc<SessionActor>,
    mpsc::UnboundedReceiver<PersistenceMsg>,
) {
    let (gateway_tx, _) = mpsc::unbounded_channel();
    let (persistence_tx, persistence_rx) = mpsc::unbounded_channel();
    let mut actor = create_test_actor(50_000, 100_000, 85, gateway_tx, persistence_tx).await;
    actor.auth_method_id = test_auth_method_id("cached_token");
    actor.model_auth_facts.replace(Some((
        "gpt-test-codex".to_string(),
        ModelAuthFacts {
            byok,
            auth_scheme: Default::default(),
            provider: Some(provider),
        },
    )));
    actor.chat_state_handle.update_credentials(xai_chat_state::Credentials {
        api_key: Some(api_key.to_owned()),
        auth_type,
        ..Default::default()
    });
    let mut sampling = xai_grok_sampling_types::SamplingConfig {
        base_url: base_url.to_owned(),
        model: "gpt-test-codex".to_owned(),
        max_completion_tokens: None,
        temperature: None,
        top_p: None,
        api_backend: Default::default(),
        extra_headers: Default::default(),
        context_window: std::num::NonZeroU64::new(256_000).unwrap(),
        reasoning_effort: None,
        reasoning_effort_supported: None,
        reasoning_summary_omit: false,
        stream_tool_calls: None,
    };
    // Keep extra_headers empty; reconstruct injects account header.
    let _ = &mut sampling;
    actor.chat_state_handle.update_sampling_config(sampling);
    (Arc::new(actor), persistence_rx)
}

fn mixed_case_reserved_headers() -> IndexMap<String, String> {
    [
        ("chatgpt-account-id", "spoofed-account"),
        ("ORIGINATOR", "codex_cli_rs"),
        ("Session-ID", "spoofed-session"),
        ("THREAD-id", "spoofed-thread"),
        ("X-Client-Request-ID", "spoofed-request"),
    ]
    .into_iter()
    .map(|(name, value)| (name.to_string(), value.to_string()))
    .collect()
}

async fn set_extra_headers(actor: &Arc<SessionActor>, headers: IndexMap<String, String>) {
    let mut sampling = actor
        .chat_state_handle
        .get_sampling_config()
        .await
        .expect("test actor has sampling config");
    sampling.extra_headers = headers;
    actor.chat_state_handle.update_sampling_config(sampling);
}

fn header_value<'a>(headers: &'a IndexMap<String, String>, name: &str) -> Option<&'a str> {
    headers
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case(name))
        .map(|(_, value)| value.as_str())
}

fn assert_reserved_headers_absent(headers: &IndexMap<String, String>) {
    for name in [
        "ChatGPT-Account-ID",
        "originator",
        "session-id",
        "thread-id",
        "x-client-request-id",
    ] {
        assert!(
            header_value(headers, name).is_none(),
            "untrusted route must not retain reserved Codex header {name}: {headers:?}"
        );
    }
}

#[test]
fn trusted_codex_wire_identity_normalizes_loaded_and_subagent_ids() {
    let supplied = uuid::Uuid::new_v4();
    assert_eq!(
        ProviderTransition::for_session_id(&supplied.to_string()).trusted_codex_request_id,
        supplied,
        "valid loaded ACP session IDs stay the trusted wire identity"
    );

    for legacy_id in ["loaded-legacy-session", "subagent-task-id-42"] {
        let identity = ProviderTransition::for_session_id(legacy_id).trusted_codex_request_id;
        uuid::Uuid::try_parse(&identity.to_string())
            .expect("non-UUID session origins must receive a valid wire UUID");
    }
}

/// Mid-session: prepared SessionToken is stale; reconstruct spends RT and
/// surfaces fresh bearer + account header.
///
/// `#[serial]`: process-wide ensure_fresh synthetic hooks.
#[tokio::test(flavor = "current_thread")]
#[serial]
async fn codex_reconstruct_refreshes_mid_session_expiry() {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            clear_ensure_fresh_codex_test_hooks();
            let dir = tempfile::tempdir().expect("tempdir");
            let path = dir.path().join("auth.json");
            write_codex_slot(&path, near_expiry_codex_auth());
            let counter = Arc::new(AtomicUsize::new(0));
            set_ensure_fresh_codex_synthetic_success(
                path.clone(),
                fresh_codex_auth(),
                counter.clone(),
            );

            let (actor, _rx) = make_codex_actor(
                STALE_BEARER,
                xai_chat_state::AuthType::SessionToken,
                CODEX_BASE_URL_DEFAULT,
                ModelProvider::Codex,
                ModelByok::NotByok,
            )
            .await;

            let cfg = actor.reconstruct_full_config().await;
            clear_ensure_fresh_codex_test_hooks();

            assert_eq!(
                cfg.api_key.as_deref(),
                Some(FRESH_BEARER),
                "reconstruct must override stale prepared SessionToken with ensure_fresh material"
            );
            assert_eq!(
                cfg.extra_headers.get("ChatGPT-Account-ID").map(String::as_str),
                Some("chatgpt-acct-fixture"),
                "trusted Codex OAuth must inject ChatGPT-Account-ID"
            );
            assert!(
                cfg.bearer_resolver.is_none(),
                "Codex path never attaches xAI AuthManager bearer resolver"
            );
            assert_eq!(
                counter.load(Ordering::SeqCst),
                1,
                "mid-session reconstruct must spend IdP once for near-expiry RT"
            );
        })
        .await;
}

/// BYOK / ApiKey: reconstruct keeps prepared key; zero IdP.
#[tokio::test(flavor = "current_thread")]
#[serial]
async fn codex_byok_key_not_overridden() {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            clear_ensure_fresh_codex_test_hooks();
            let dir = tempfile::tempdir().expect("tempdir");
            let path = dir.path().join("auth.json");
            write_codex_slot(&path, near_expiry_codex_auth());
            let counter = Arc::new(AtomicUsize::new(0));
            set_ensure_fresh_codex_synthetic_success(
                path.clone(),
                fresh_codex_auth(),
                counter.clone(),
            );

            let (actor, _rx) = make_codex_actor(
                BYOK_KEY,
                xai_chat_state::AuthType::ApiKey,
                CODEX_BASE_URL_DEFAULT,
                ModelProvider::Codex,
                ModelByok::Byok,
            )
            .await;

            let cfg = actor.reconstruct_full_config().await;
            clear_ensure_fresh_codex_test_hooks();

            assert_eq!(
                cfg.api_key.as_deref(),
                Some(BYOK_KEY),
                "BYOK prepared api_key must not be overridden by OAuth ensure_fresh"
            );
            assert_eq!(
                counter.load(Ordering::SeqCst),
                0,
                "BYOK route must not call IdP"
            );
            assert!(
                !cfg.extra_headers.contains_key("ChatGPT-Account-ID"),
                "BYOK must not inject ChatGPT-Account-ID from session material"
            );
        })
        .await;
}

/// Custom host: session OAuth not allowed — no bearer override, zero IdP.
#[tokio::test(flavor = "current_thread")]
#[serial]
async fn codex_oauth_bearer_absent_on_custom_endpoint() {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            clear_ensure_fresh_codex_test_hooks();
            let dir = tempfile::tempdir().expect("tempdir");
            let path = dir.path().join("auth.json");
            write_codex_slot(&path, near_expiry_codex_auth());
            let counter = Arc::new(AtomicUsize::new(0));
            set_ensure_fresh_codex_synthetic_success(
                path.clone(),
                fresh_codex_auth(),
                counter.clone(),
            );

            let custom = "https://evil.example/v1";
            let (actor, _rx) = make_codex_actor(
                "custom-prepared-key",
                xai_chat_state::AuthType::SessionToken,
                custom,
                ModelProvider::Codex,
                ModelByok::NotByok,
            )
            .await;

            let cfg = actor.reconstruct_full_config().await;
            clear_ensure_fresh_codex_test_hooks();

            assert_eq!(
                cfg.api_key.as_deref(),
                Some("custom-prepared-key"),
                "custom endpoint must keep prepared key (no OAuth bearer from ensure_fresh)"
            );
            assert_eq!(
                counter.load(Ordering::SeqCst),
                0,
                "custom non-allowlisted host must not call IdP"
            );
            assert!(
                !cfg.extra_headers.contains_key("ChatGPT-Account-ID"),
                "custom host must not receive ChatGPT-Account-ID"
            );
        })
        .await;
}

/// Trusted reconstruction owns the Codex wire profile and request identity.
/// Every route that does not meet the existing SessionToken + first-party
/// Codex gate must have inherited reserved values removed instead.
#[tokio::test(flavor = "current_thread")]
#[serial]
async fn trusted_codex_reconstruct_enables_profile_and_metadata() {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            clear_ensure_fresh_codex_test_hooks();
            let dir = tempfile::tempdir().expect("tempdir");
            let path = dir.path().join("auth.json");
            write_codex_slot(&path, near_expiry_codex_auth());
            set_ensure_fresh_codex_synthetic_success(
                path,
                fresh_codex_auth(),
                Arc::new(AtomicUsize::new(0)),
            );

            let (trusted, _rx) = make_codex_actor(
                STALE_BEARER,
                xai_chat_state::AuthType::SessionToken,
                CODEX_BASE_URL_DEFAULT,
                ModelProvider::Codex,
                ModelByok::NotByok,
            )
            .await;
            assert_eq!(
                trusted.session_info.id.0.as_ref(),
                "test-actor",
                "fixture intentionally preserves a legacy non-UUID loaded session id"
            );
            set_extra_headers(&trusted, mixed_case_reserved_headers()).await;

            let first = trusted.reconstruct_full_config().await;
            let second = trusted.reconstruct_full_config().await;
            clear_ensure_fresh_codex_test_hooks();

            assert_eq!(
                first.responses_wire_profile,
                xai_grok_sampler::ResponsesWireProfile::TrustedCodex,
                "only the trusted Codex OAuth path enables the Responses profile"
            );
            assert_eq!(header_value(&first.extra_headers, "originator"), Some("bum"));
            assert_eq!(
                header_value(&first.extra_headers, "ChatGPT-Account-ID"),
                Some("chatgpt-acct-fixture"),
                "known account identity remains available only through the trusted gate"
            );
            let session_id = header_value(&first.extra_headers, "session-id")
                .expect("trusted request must include session-id")
                .to_owned();
            uuid::Uuid::parse_str(&session_id).expect("session metadata must be a UUID");
            for name in ["thread-id", "x-client-request-id"] {
                assert_eq!(
                    header_value(&first.extra_headers, name),
                    Some(session_id.as_str()),
                    "{name} must use the stable bum session UUID"
                );
                assert_eq!(
                    header_value(&second.extra_headers, name),
                    Some(session_id.as_str()),
                    "{name} must remain stable across reconstruction"
                );
            }
            assert_eq!(
                header_value(&second.extra_headers, "session-id"),
                Some(session_id.as_str()),
                "session-id must remain stable across reconstruction"
            );
            assert_ne!(
                header_value(&first.extra_headers, "originator"),
                Some("codex_cli_rs"),
                "bum must not identify as the upstream CLI"
            );

            let (byok, _rx) = make_codex_actor(
                BYOK_KEY,
                xai_chat_state::AuthType::ApiKey,
                CODEX_BASE_URL_DEFAULT,
                ModelProvider::Codex,
                ModelByok::Byok,
            )
            .await;
            let (custom, _rx) = make_codex_actor(
                "custom-prepared-key",
                xai_chat_state::AuthType::SessionToken,
                "https://custom.example/v1",
                ModelProvider::Codex,
                ModelByok::NotByok,
            )
            .await;
            let (same_host_different_port, _rx) = make_codex_actor(
                "custom-prepared-key",
                xai_chat_state::AuthType::SessionToken,
                "https://chatgpt.com:8443/backend-api/codex",
                ModelProvider::Codex,
                ModelByok::NotByok,
            )
            .await;
            let (xai, _rx) = make_codex_actor(
                "xai-prepared-key",
                xai_chat_state::AuthType::SessionToken,
                "https://api.x.ai/v1",
                ModelProvider::Xai,
                ModelByok::NotByok,
            )
            .await;

            for actor in [&byok, &custom, &same_host_different_port, &xai] {
                set_extra_headers(actor, mixed_case_reserved_headers()).await;
                let cfg = actor.reconstruct_full_config().await;
                assert_eq!(
                    cfg.responses_wire_profile,
                    xai_grok_sampler::ResponsesWireProfile::Disabled,
                    "untrusted routes must keep the trusted profile disabled"
                );
                assert_reserved_headers_absent(&cfg.extra_headers);
            }
        })
        .await;
}
