//! Phase 4 provider-aware request routing — integration harness + phase gate.
//!
//! **Contracts covered:**
//! - xAI route (base + credential slot + session OAuth)
//! - Codex route (ChatGPT backend, no XAI env leak)
//! - provider override rebind
//! - dual-token never_cross_slot
//! - prepare/carrier/transform switch (Plan 04)
//! - on-wire Authorization (mock HTTP, fake tokens)
//! - local fail-closed (missing key, empty live resolver, whitespace, blank static)
//! - proxy headers on xAI / off on Codex
//! - Option provider attach policy (None never attaches xAI resolver)
//! - empty Codex key construction (D-11) without panic
//! - custom-host session OAuth denial
//! - endpoints trust provenance
//!
//! **Scope (D-12 / D-13):** public APIs + SamplingClient live boundary. No shell
//! `--lib` gates. Fake tokens only (`xai-fake-token` / `codex-fake-token`) — no live
//! ChatGPT OAuth, no Phase 5 login UX, no Phase 6 missing-provider gate.
//!
//! Prefer `cargo test -p xai-grok-shell --test provider_routing` only.

use chrono::{Duration, Utc};
use futures_util::StreamExt;
use indexmap::IndexMap;
use serial_test::serial;
use std::num::NonZeroU64;
use std::sync::Arc;
use xai_grok_shell::agent::config::{
    apply_prepared_sampling_to_chat_state_fields, inject_url_derived_headers,
    prepare_sampling_credentials, prepared_sampling_config_from_credentials,
    reconstruct_attach_policy_from_facts, resolve_credentials, resolve_credentials_for_provider,
    resolve_model_auth_facts, resolve_model_list, resolve_provider_route,
    sampling_config_for_model, session_key_for_model_provider,
    should_attach_xai_auth_manager_bearer_resolver, Config, ConfigModelOverride, EndpointsConfig,
    ModelAuthFacts, ModelEntry, ModelProvider, CLI_CHAT_PROXY_BASE_URL_DEFAULT,
    CODEX_BASE_URL_DEFAULT, XAI_API_BASE_URL_DEFAULT,
};
use xai_grok_shell::auth::{
    read_provider_auth_store, select_provider_access_token, AuthMode, AuthStore, AuthStoreReadError,
    GrokAuth, PROVIDER_CODEX,
};
use xai_grok_shell::sampling::{
    ApiBackend, BearerResolver, Client, ConversationItem, ConversationRequest, SamplerConfig,
    SamplingError,
};
use xai_grok_test_support::{EnvGuard, MockInferenceServer};

const XAI_FAKE: &str = "xai-fake-token";
const CODEX_FAKE: &str = "codex-fake-token";

/// Test stub for empty / whitespace live bearer resolvers (cycle 2 fail-closed).
#[derive(Debug)]
struct StubBearerResolver(Option<&'static str>);

impl BearerResolver for StubBearerResolver {
    fn current_bearer(&self) -> Option<String> {
        self.0.map(str::to_owned)
    }
}

fn sampler_config_for_mock(
    base_url: &str,
    api_key: Option<&str>,
    model: &str,
    api_backend: ApiBackend,
    extra_headers: IndexMap<String, String>,
    bearer_resolver: Option<Arc<dyn BearerResolver>>,
) -> SamplerConfig {
    SamplerConfig {
        api_key: api_key.map(str::to_owned),
        base_url: base_url.to_owned(),
        model: model.to_owned(),
        max_completion_tokens: Some(64),
        temperature: None,
        top_p: None,
        api_backend,
        auth_scheme: Default::default(),
        extra_headers,
        context_window: 8192,
        force_http1: false,
        max_retries: Some(0),
        stream_tool_calls: false,
        idle_timeout_secs: None,
        reasoning_effort: None,
        origin_client: None,
        client_identifier: None,
        deployment_id: None,
        user_id: None,
        client_version: None,
        attribution_callback: None,
        bearer_resolver,
        supports_backend_search: false,
        compactions_remaining: None,
        compaction_at_tokens: None,
        doom_loop_recovery: None,
        header_injector: None,
    }
}



/// Deterministic endpoints for route assertions (avoid ambient env-only equality).
fn deterministic_endpoints() -> EndpointsConfig {
    EndpointsConfig {
        cli_chat_proxy_base_url: Some(CLI_CHAT_PROXY_BASE_URL_DEFAULT.to_owned()),
        xai_api_base_url: XAI_API_BASE_URL_DEFAULT.to_owned(),
        codex_base_url: CODEX_BASE_URL_DEFAULT.to_owned(),
        ..EndpointsConfig::default()
    }
}

/// D-07: product Codex default is ChatGPT backend, not Platform OpenAI.
#[test]
fn codex_base_url_default_constant() {
    assert_eq!(
        CODEX_BASE_URL_DEFAULT,
        "https://chatgpt.com/backend-api/codex",
        "CODEX_BASE_URL_DEFAULT must be ChatGPT/Codex backend (D-15/D-06)"
    );
}

/// D-07: empty/default codex field resolves to CODEX_BASE_URL_DEFAULT (field override, no env).
#[test]
fn resolve_codex_base_url_default() {
    let endpoints = EndpointsConfig {
        codex_base_url: String::new(),
        ..deterministic_endpoints()
    };
    assert_eq!(
        endpoints.resolve_codex_base_url(),
        CODEX_BASE_URL_DEFAULT,
        "blank codex_base_url must fall back to CODEX_BASE_URL_DEFAULT"
    );

    let whitespace = EndpointsConfig {
        codex_base_url: "   \t  ".to_owned(),
        ..deterministic_endpoints()
    };
    assert_eq!(
        whitespace.resolve_codex_base_url(),
        CODEX_BASE_URL_DEFAULT,
        "whitespace-only codex_base_url must fall back to CODEX_BASE_URL_DEFAULT"
    );

    let stock = deterministic_endpoints();
    assert_eq!(
        stock.resolve_codex_base_url(),
        CODEX_BASE_URL_DEFAULT,
        "deterministic stock codex_base_url must resolve to default"
    );
}

/// D-07: field override wins without env (parallel-CI safe).
#[test]
fn resolve_codex_base_url_field_override() {
    let override_url = "https://codex.enterprise.example/backend-api/codex";
    let endpoints = EndpointsConfig {
        codex_base_url: override_url.to_owned(),
        ..deterministic_endpoints()
    };
    assert_eq!(
        endpoints.resolve_codex_base_url(),
        override_url,
        "non-empty codex_base_url field must win over default"
    );
}

/// D-07: `GROK_CODEX_BASE_URL` env fills `EndpointsConfig::default().codex_base_url`.
#[test]
#[serial]
fn resolve_codex_base_url_env_override() {
    let override_url = "https://codex.env-override.example/backend-api/codex";
    let _guard = EnvGuard::set("GROK_CODEX_BASE_URL", override_url);
    let endpoints = EndpointsConfig::default();
    assert_eq!(
        endpoints.codex_base_url, override_url,
        "Default must read GROK_CODEX_BASE_URL into codex_base_url"
    );
    assert_eq!(
        endpoints.resolve_codex_base_url(),
        override_url,
        "resolve_codex_base_url must return GROK_CODEX_BASE_URL value"
    );
}

/// D-01/D-05/D-09: Xai provider → inference base + slot xai + session OAuth allowed.
#[test]
fn resolve_provider_route_xai_default() {
    let endpoints = deterministic_endpoints();
    let route = resolve_provider_route(ModelProvider::Xai, &endpoints, None);
    assert_eq!(route.base_url, endpoints.resolve_inference_base_url());
    assert_eq!(route.credential_slot, "xai");
    assert_eq!(route.provider, ModelProvider::Xai);
    assert!(
        route.session_oauth_allowed,
        "stock xAI/proxy base must allow session OAuth"
    );
}

/// D-01/D-06/D-09: Codex provider → Codex base + slot codex + session OAuth allowed.
#[test]
fn resolve_provider_route_codex_default() {
    let endpoints = deterministic_endpoints();
    let route = resolve_provider_route(ModelProvider::Codex, &endpoints, None);
    assert_eq!(route.base_url, endpoints.resolve_codex_base_url());
    assert_eq!(route.base_url, CODEX_BASE_URL_DEFAULT);
    assert_eq!(route.credential_slot, "codex");
    assert_eq!(route.provider, ModelProvider::Codex);
    assert!(
        route.session_oauth_allowed,
        "stock Codex/ChatGPT base must allow session OAuth"
    );
}

/// D-04 + D-09: non-empty override wins for base_url; credential_slot still matches provider.
#[test]
fn resolve_provider_route_override_wins() {
    let endpoints = deterministic_endpoints();
    let override_url = "https://byok.example/v1";
    let codex = resolve_provider_route(
        ModelProvider::Codex,
        &endpoints,
        Some(override_url),
    );
    assert_eq!(codex.base_url, override_url);
    assert_eq!(codex.credential_slot, "codex");
    assert_eq!(codex.provider, ModelProvider::Codex);

    let xai = resolve_provider_route(ModelProvider::Xai, &endpoints, Some(override_url));
    assert_eq!(xai.base_url, override_url);
    assert_eq!(xai.credential_slot, "xai");
    assert_eq!(xai.provider, ModelProvider::Xai);
}

/// Review HIGH: custom Codex host must not allow session OAuth bearer.
#[test]
fn resolve_provider_route_custom_host_disallows_session_oauth() {
    let endpoints = deterministic_endpoints();
    let route = resolve_provider_route(
        ModelProvider::Codex,
        &endpoints,
        Some("https://byok.example/v1"),
    );
    assert_eq!(route.base_url, "https://byok.example/v1");
    assert_eq!(route.credential_slot, "codex");
    assert!(
        !route.session_oauth_allowed,
        "non–first-party Codex override must set session_oauth_allowed=false"
    );
}

/// Cycle-3 MEDIUM: provider Xai + custom models_base_url / override → no session OAuth.
#[test]
fn resolve_provider_route_xai_custom_models_base_disallows_session_oauth() {
    let endpoints = deterministic_endpoints();
    let via_override = resolve_provider_route(
        ModelProvider::Xai,
        &endpoints,
        Some("https://byok.example/v1"),
    );
    assert_eq!(via_override.base_url, "https://byok.example/v1");
    assert_eq!(via_override.credential_slot, "xai");
    assert!(
        !via_override.session_oauth_allowed,
        "Xai + non–first-party override must set session_oauth_allowed=false"
    );

    let with_models_base = EndpointsConfig {
        models_base_url: Some("https://byok.example/v1".to_owned()),
        ..deterministic_endpoints()
    };
    let via_models_base =
        resolve_provider_route(ModelProvider::Xai, &with_models_base, None);
    assert_eq!(via_models_base.base_url, "https://byok.example/v1");
    assert!(
        !via_models_base.session_oauth_allowed,
        "Xai + custom models_base_url (inference base) must set session_oauth_allowed=false"
    );
}

/// First-party Codex override (equal to resolve_codex_base_url / chatgpt host) allows OAuth.
#[test]
fn resolve_provider_route_first_party_codex_override_allows_oauth() {
    let endpoints = deterministic_endpoints();
    let stock = endpoints.resolve_codex_base_url();
    let route = resolve_provider_route(ModelProvider::Codex, &endpoints, Some(&stock));
    assert_eq!(route.base_url, stock);
    assert!(
        route.session_oauth_allowed,
        "override equal to resolve_codex_base_url must allow session OAuth"
    );

    let chatgpt = resolve_provider_route(
        ModelProvider::Codex,
        &endpoints,
        Some("https://chatgpt.com/backend-api/codex"),
    );
    assert!(
        chatgpt.session_oauth_allowed,
        "chatgpt.com Codex backend host must allow session OAuth"
    );
}

/// Whitespace-only override falls through to provider default base.
#[test]
fn resolve_provider_route_blank_override_ignored() {
    let endpoints = deterministic_endpoints();
    let xai = resolve_provider_route(ModelProvider::Xai, &endpoints, Some("  \t  "));
    assert_eq!(xai.base_url, endpoints.resolve_inference_base_url());
    assert!(xai.session_oauth_allowed);

    let codex = resolve_provider_route(ModelProvider::Codex, &endpoints, Some("   "));
    assert_eq!(codex.base_url, endpoints.resolve_codex_base_url());
    assert!(codex.session_oauth_allowed);
}

fn catalog_entry(id: &str) -> ModelEntry {
    let list = resolve_model_list(&Config::default(), None);
    list.get(id)
        .cloned()
        .unwrap_or_else(|| panic!("catalog missing {id}; keys={:?}", list.keys().collect::<Vec<_>>()))
}

/// Proves the integration binary links and discovers tests (`--list`).
#[test]
fn provider_routing_harness_smoke() {
    let cfg = Config {
        endpoints: deterministic_endpoints(),
        ..Config::default()
    };
    let list = resolve_model_list(&cfg, None);
    assert!(
        !list.is_empty(),
        "resolve_model_list with deterministic endpoints must return at least one model"
    );
    assert!(
        list.contains_key("grok-build"),
        "catalog must include grok-build"
    );
    assert!(
        list.contains_key("gpt-5.6-sol"),
        "catalog must include gpt-5.6-sol"
    );
}

/// MOD-04 / D-05 / D-09: Grok/xAI model → inference/proxy path + xAI fake token.
#[test]
fn xai_model_routes_to_proxy_with_xai_token() {
    let endpoints = deterministic_endpoints();
    let cfg = Config {
        endpoints: endpoints.clone(),
        ..Config::default()
    };
    let list = resolve_model_list(&cfg, None);
    let entry = list
        .get("grok-build")
        .cloned()
        .expect("catalog must include grok-build");
    assert_eq!(
        entry.info.provider.as_str(),
        "xai",
        "grok-build provider must be wire id \"xai\""
    );
    assert_eq!(
        entry.info.base_url,
        endpoints.resolve_inference_base_url(),
        "catalog xAI base_url must be stamped via resolve_provider_route"
    );
    assert_eq!(
        entry.api_base_url.as_deref(),
        Some(endpoints.xai_api_base_url.as_str()),
        "catalog xAI rows keep api_base_url (D-15 dual endpoint)"
    );

    let creds = resolve_credentials(&entry, Some(XAI_FAKE));
    assert_eq!(creds.api_key.as_deref(), Some(XAI_FAKE));

    let expected = endpoints.resolve_inference_base_url();
    assert_eq!(
        creds.base_url, expected,
        "xAI model base_url must be inference/proxy path (deterministic EndpointsConfig)"
    );
    assert!(
        creds.base_url.contains("cli-chat-proxy") || creds.base_url == endpoints.proxy_url(),
        "xAI route must use cli-chat-proxy / inference path; got {}",
        creds.base_url
    );
}

/// MOD-05 / D-06 / D-09: Codex model → ChatGPT backend base + Codex fake token.
#[test]
fn codex_model_routes_to_codex_backend_with_codex_token() {
    let endpoints = deterministic_endpoints();
    let cfg = Config {
        endpoints: endpoints.clone(),
        ..Config::default()
    };
    let list = resolve_model_list(&cfg, None);
    let entry = list
        .get("gpt-5.6-sol")
        .cloned()
        .expect("catalog must include gpt-5.6-sol");
    assert_eq!(
        entry.info.provider.as_str(),
        "codex",
        "gpt-5.6-sol provider must be wire id \"codex\" (D-02)"
    );

    assert!(
        !entry.info.base_url.contains("cli-chat-proxy"),
        "gpt-5.6-sol base_url must not contain cli-chat-proxy; got {}",
        entry.info.base_url
    );
    assert_eq!(
        entry.info.base_url,
        endpoints.resolve_codex_base_url(),
        "gpt-5.6-sol base_url must equal resolve_codex_base_url; got {}",
        entry.info.base_url
    );
    assert_eq!(
        entry.info.base_url, CODEX_BASE_URL_DEFAULT,
        "deterministic catalog Codex base must be {CODEX_BASE_URL_DEFAULT}"
    );
    assert!(
        entry.api_base_url.is_none(),
        "Codex catalog entries must have api_base_url None (D-15); got {:?}",
        entry.api_base_url
    );

    let creds = resolve_credentials(&entry, Some(CODEX_FAKE));
    assert_eq!(
        creds.api_key.as_deref(),
        Some(CODEX_FAKE),
        "Codex path must use Codex fake token only"
    );
    assert!(
        !creds.base_url.contains("cli-chat-proxy"),
        "resolved Codex base_url must not contain cli-chat-proxy; got {}",
        creds.base_url
    );
    assert_eq!(
        creds.base_url, CODEX_BASE_URL_DEFAULT,
        "resolved Codex base_url must be {CODEX_BASE_URL_DEFAULT}; got {}",
        creds.base_url
    );
}

/// Review HIGH rebind: provider=codex without base_url re-normalizes base via resolver.
#[test]
fn provider_override_rebinds_base_url() {
    let endpoints = deterministic_endpoints();
    let mut config_models = IndexMap::new();
    config_models.insert(
        "grok-build".to_owned(),
        ConfigModelOverride {
            provider: Some(ModelProvider::Codex),
            // no base_url — rebind must re-normalize
            ..ConfigModelOverride::default()
        },
    );
    let cfg = Config {
        endpoints: endpoints.clone(),
        config_models,
        ..Config::default()
    };
    let list = resolve_model_list(&cfg, None);
    let rebound = list
        .get("grok-build")
        .expect("catalog must include grok-build after provider override");
    assert_eq!(rebound.info.provider, ModelProvider::Codex);
    assert_eq!(
        rebound.info.base_url,
        endpoints.resolve_codex_base_url(),
        "provider-only override must re-normalize base via resolve_provider_route"
    );
    assert!(
        rebound.api_base_url.is_none(),
        "provider rebind to Codex must clear xAI api_base_url"
    );
}

/// D-04: provider + explicit base_url keeps the explicit base.
#[test]
fn provider_override_explicit_base_preserved() {
    let endpoints = deterministic_endpoints();
    let override_url = "https://byok.example/v1";
    let mut config_models = IndexMap::new();
    config_models.insert(
        "grok-build".to_owned(),
        ConfigModelOverride {
            provider: Some(ModelProvider::Codex),
            base_url: Some(override_url.to_owned()),
            ..ConfigModelOverride::default()
        },
    );
    let cfg = Config {
        endpoints: endpoints.clone(),
        config_models,
        ..Config::default()
    };
    let list = resolve_model_list(&cfg, None);
    let applied = list
        .get("grok-build")
        .expect("catalog must include grok-build after override");
    assert_eq!(applied.info.provider, ModelProvider::Codex);
    assert_eq!(
        applied.info.base_url, override_url,
        "explicit base_url on override must be preserved (D-04)"
    );
}

/// D-04: explicit per-model `base_url` override wins over provider defaults.
#[test]
fn model_override_base_url_wins() {
    let endpoints = deterministic_endpoints();
    let override_url = "https://byok.example/v1";
    let mut entry = catalog_entry("gpt-5.6-sol");
    entry.info.provider = ModelProvider::Codex;
    entry.info.base_url = override_url.to_owned();
    // Clear api_base_url so resolve does not fall through to xAI Platform.
    entry.api_base_url = None;

    let creds = resolve_credentials_for_provider(
        &entry,
        &endpoints,
        Some(XAI_FAKE),
        Some(CODEX_FAKE),
    );
    assert_eq!(
        creds.base_url, override_url,
        "D-04: explicit model base_url override must win; got {}",
        creds.base_url
    );
    // Custom host denies session OAuth; base still preserved (own_credential would attach key).
    assert!(
        creds.api_key.is_none(),
        "custom override host must not attach session OAuth; got {:?}",
        creds.api_key
    );
}

/// Custom host: session OAuth keys must not attach (session_oauth_allowed false).
#[test]
fn custom_host_skips_session_oauth() {
    let endpoints = deterministic_endpoints();
    let mut entry = catalog_entry("gpt-5.6-sol");
    entry.info.provider = ModelProvider::Codex;
    entry.info.base_url = "https://byok.example/v1".to_owned();
    entry.api_base_url = None;
    entry.api_key = None;
    entry.env_key = None;

    let creds = resolve_credentials_for_provider(
        &entry,
        &endpoints,
        Some(XAI_FAKE),
        Some(CODEX_FAKE),
    );
    assert_eq!(creds.base_url, "https://byok.example/v1");
    assert!(
        creds.api_key.is_none(),
        "custom Codex host must not receive session OAuth keys; got {:?}",
        creds.api_key
    );
}

/// Own credential on custom host always wins (host policy does not block BYOK).
#[test]
fn own_credential_on_custom_host_wins() {
    let endpoints = deterministic_endpoints();
    let mut entry = catalog_entry("gpt-5.6-sol");
    entry.info.provider = ModelProvider::Codex;
    entry.info.base_url = "https://byok.example/v1".to_owned();
    entry.api_base_url = None;
    entry.api_key = Some("byok-own-key".to_owned());

    let creds = resolve_credentials_for_provider(
        &entry,
        &endpoints,
        Some(XAI_FAKE),
        Some(CODEX_FAKE),
    );
    assert_eq!(creds.base_url, "https://byok.example/v1");
    assert_eq!(creds.api_key.as_deref(), Some("byok-own-key"));
}

/// Operator-configured first-party Codex endpoint still allows session OAuth
/// (EndpointsConfig provenance — not default-only string compare).
#[test]
fn configured_codex_endpoint_allows_session_oauth() {
    let configured = "https://codex.enterprise.example/backend-api/codex";
    let endpoints = EndpointsConfig {
        codex_base_url: configured.to_owned(),
        ..deterministic_endpoints()
    };
    let mut entry = catalog_entry("gpt-5.6-sol");
    entry.info.provider = ModelProvider::Codex;
    entry.info.base_url = configured.to_owned();
    entry.api_base_url = None;

    let creds = resolve_credentials_for_provider(
        &entry,
        &endpoints,
        Some(XAI_FAKE),
        Some(CODEX_FAKE),
    );
    assert_eq!(creds.base_url, configured);
    assert_eq!(
        creds.api_key.as_deref(),
        Some(CODEX_FAKE),
        "configured first-party Codex host must allow session OAuth from matching EndpointsConfig"
    );
}

/// D-15: XAI_API_KEY env must never apply to Codex models.
#[test]
#[serial]
fn codex_skips_xai_api_key_env_fallback() {
    let endpoints = deterministic_endpoints();
    let _guard = EnvGuard::set("XAI_API_KEY", "xai-env-should-not-leak");
    let mut entry = catalog_entry("gpt-5.6-sol");
    entry.info.provider = ModelProvider::Codex;
    entry.info.base_url = endpoints.resolve_codex_base_url();
    entry.api_base_url = None;
    entry.api_key = None;
    entry.env_key = None;

    let creds = resolve_credentials_for_provider(&entry, &endpoints, None, None);
    assert_eq!(
        creds.base_url,
        endpoints.resolve_codex_base_url(),
        "Codex base preserved without credentials"
    );
    assert!(
        creds.api_key.is_none(),
        "Codex must not receive XAI_API_KEY fallback; got {:?}",
        creds.api_key
    );
}

/// D-11: empty Codex key still constructs a route (fail-closed is Plan 05).
#[test]
fn empty_codex_key_allows_route_construction() {
    let endpoints = deterministic_endpoints();
    let mut entry = catalog_entry("gpt-5.6-sol");
    entry.info.provider = ModelProvider::Codex;
    entry.info.base_url = endpoints.resolve_codex_base_url();
    entry.api_base_url = None;

    let creds = resolve_credentials_for_provider(&entry, &endpoints, Some(XAI_FAKE), None);
    assert_eq!(creds.base_url, endpoints.resolve_codex_base_url());
    assert!(
        creds.api_key.is_none(),
        "construction with empty Codex slot must leave api_key None"
    );
}

/// Safety / Pitfall 4: Codex base must not receive X-XAI-Token-Auth proxy headers.
#[test]
fn no_proxy_headers_on_codex() {
    let mut headers = IndexMap::new();
    inject_url_derived_headers(&mut headers, None, CODEX_BASE_URL_DEFAULT);
    assert!(
        !headers.contains_key("X-XAI-Token-Auth"),
        "Codex base must not insert X-XAI-Token-Auth; headers={headers:?}"
    );
    assert!(
        !headers.contains_key("x-authenticateresponse"),
        "Codex base must not insert x-authenticateresponse; headers={headers:?}"
    );
}

/// Dual-token isolation (D-09 / T-04-02 / 04-REVIEWS HIGH).
///
/// Both tokens present simultaneously; dual-key API returns only the
/// provider-correct key (never the other slot).
#[test]
fn never_cross_slot() {
    // Both tokens present simultaneously — dual-token fixture (required).
    let xai_key = XAI_FAKE;
    let codex_key = CODEX_FAKE;
    let dual_map = [("xai", xai_key), ("codex", codex_key)];
    assert_eq!(dual_map.len(), 2, "dual-token fixture must hold both slots");
    assert_ne!(xai_key, codex_key);

    let endpoints = deterministic_endpoints();
    let cfg = Config {
        endpoints: endpoints.clone(),
        ..Config::default()
    };
    let list = resolve_model_list(&cfg, None);
    let xai_entry = list
        .get("grok-build")
        .cloned()
        .expect("catalog must include grok-build");
    let codex_entry = list
        .get("gpt-5.6-sol")
        .cloned()
        .expect("catalog must include gpt-5.6-sol");
    assert_eq!(xai_entry.info.provider.as_str(), "xai");
    assert_eq!(codex_entry.info.provider.as_str(), "codex");

    // Dual-key with BOTH slots Some: each model receives only its provider key.
    let codex_creds = resolve_credentials_for_provider(
        &codex_entry,
        &endpoints,
        Some(xai_key),
        Some(codex_key),
    );
    assert_eq!(
        codex_creds.api_key.as_deref(),
        Some(codex_key),
        "Codex model must receive only codex-fake when both tokens present"
    );
    assert_ne!(
        codex_creds.api_key.as_deref(),
        Some(xai_key),
        "Codex model must never receive xai-fake"
    );

    let xai_creds = resolve_credentials_for_provider(
        &xai_entry,
        &endpoints,
        Some(xai_key),
        Some(codex_key),
    );
    assert_eq!(
        xai_creds.api_key.as_deref(),
        Some(xai_key),
        "xAI model must receive only xai-fake when both tokens present"
    );
    assert_ne!(
        xai_creds.api_key.as_deref(),
        Some(codex_key),
        "xAI model must never receive codex-fake"
    );

    // Single-key maps into the model provider slot only (wrong-slot callers get
    // None for the other slot — no longer provider-blind acceptance).
    let codex_with_xai_key = resolve_credentials(&codex_entry, Some(xai_key));
    // session_key is treated as the *Codex* slot key for a Codex model.
    // Offering an xAI token string as the Codex slot would attach it only if
    // session OAuth is allowed — the dual-key API is the safe dual-slot path.
    // Document that single-key cannot type-check provenance:
    assert_eq!(
        codex_with_xai_key.api_key.as_deref(),
        Some(xai_key),
        "single-key still attaches the provided string as this model's slot key \
         (provenance untyped — dual-key required for dual-slot isolation)"
    );
}

fn fixture_grok_auth(key: &str, mode: AuthMode, scope_expires_hours: Option<i64>) -> GrokAuth {
    GrokAuth {
        key: key.to_owned(),
        auth_mode: mode,
        create_time: Utc::now(),
        user_id: "fixture-user".into(),
        email: None,
        first_name: None,
        last_name: None,
        profile_image_asset_id: None,
        principal_type: None,
        principal_id: None,
        team_id: None,
        team_name: None,
        team_role: None,
        organization_id: None,
        organization_name: None,
        organization_role: None,
        user_blocked_reason: None,
        team_blocked_reasons: vec![],
        coding_data_retention_opt_out: false,
        has_grok_code_access: None,
        refresh_token: None,
        expires_at: scope_expires_hours.map(|h| Utc::now() + Duration::hours(h)),
        oidc_issuer: None,
        oidc_client_id: None,
    }
}

/// D-09: only the matching provider slot key is returned.
#[test]
fn session_key_for_model_provider_xai() {
    assert_eq!(
        session_key_for_model_provider(ModelProvider::Xai, Some(XAI_FAKE), Some(CODEX_FAKE)),
        Some(XAI_FAKE)
    );
    assert_eq!(
        session_key_for_model_provider(ModelProvider::Xai, None, Some(CODEX_FAKE)),
        None
    );
}

#[test]
fn session_key_for_model_provider_codex() {
    assert_eq!(
        session_key_for_model_provider(ModelProvider::Codex, Some(XAI_FAKE), Some(CODEX_FAKE)),
        Some(CODEX_FAKE)
    );
    assert_eq!(
        session_key_for_model_provider(ModelProvider::Codex, Some(XAI_FAKE), None),
        None
    );
}

/// Multi-scope: prefer Oidc, skip WebLogin, skip blank keys.
#[test]
fn select_provider_access_token_prefers_oidc_skips_weblogin_skips_blank() {
    let mut store = AuthStore::new();
    store.insert(
        "aaa-weblogin".into(),
        fixture_grok_auth("web-token", AuthMode::WebLogin, Some(24)),
    );
    store.insert(
        "blank-key".into(),
        fixture_grok_auth("   ", AuthMode::Oidc, Some(24)),
    );
    store.insert(
        "zzz-apikey".into(),
        fixture_grok_auth("api-token", AuthMode::ApiKey, Some(24)),
    );
    store.insert(
        "mid-oidc".into(),
        fixture_grok_auth("oidc-token", AuthMode::Oidc, Some(24)),
    );
    let selected = select_provider_access_token(&store).expect("must select Oidc");
    assert_eq!(selected.key, "oidc-token");
    assert_eq!(selected.auth_mode, AuthMode::Oidc);
}

/// Never first BTreeMap entry: WebLogin is first lexicographically but skipped.
#[test]
fn select_provider_access_token_never_first_arbitrary_only() {
    let mut store = AuthStore::new();
    // Lexicographically first scope is WebLogin — must not win.
    store.insert(
        "aaa-first".into(),
        fixture_grok_auth("web-should-skip", AuthMode::WebLogin, Some(24)),
    );
    store.insert(
        "zzz-oidc".into(),
        fixture_grok_auth("oidc-winner", AuthMode::Oidc, Some(24)),
    );
    let selected = select_provider_access_token(&store).expect("must select Oidc");
    assert_eq!(selected.key, "oidc-winner");
}

/// Dual-key prepare credentials: Codex model ignores xAI session key.
#[test]
fn prepare_sampling_credentials_codex_ignores_xai_session() {
    let endpoints = deterministic_endpoints();
    let entry = catalog_entry("gpt-5.6-sol");
    let creds = prepare_sampling_credentials(
        &entry,
        &endpoints,
        Some(XAI_FAKE),
        Some(CODEX_FAKE),
    );
    assert_eq!(creds.api_key.as_deref(), Some(CODEX_FAKE));
    assert_ne!(creds.api_key.as_deref(), Some(XAI_FAKE));
    assert_eq!(
        creds.auth_type,
        xai_chat_state::AuthType::SessionToken,
        "session slot key yields SessionToken provenance"
    );
}

/// Prepared carrier stamps auth_type from credential provenance (not xAI re-resolve).
#[test]
fn prepared_sampling_config_carries_auth_type() {
    let endpoints = deterministic_endpoints();
    let codex = catalog_entry("gpt-5.6-sol");
    let creds = prepare_sampling_credentials(
        &codex,
        &endpoints,
        Some(XAI_FAKE),
        Some(CODEX_FAKE),
    );
    let prepared = prepared_sampling_config_from_credentials(
        &codex,
        creds,
        None,
        None,
        None,
        None,
    );
    assert_eq!(prepared.provider, ModelProvider::Codex);
    assert_eq!(prepared.auth_type, xai_chat_state::AuthType::SessionToken);
    assert_eq!(prepared.sampler_config.api_key.as_deref(), Some(CODEX_FAKE));
    assert_ne!(
        prepared.sampler_config.api_key.as_deref(),
        Some(XAI_FAKE),
        "Codex prepare must not stamp xAI session key"
    );
}

/// Transform A writes prepared auth_type without AuthManager.
#[test]
fn apply_prepared_sampling_to_chat_state_fields_preserves_auth_type() {
    let endpoints = deterministic_endpoints();
    let codex = catalog_entry("gpt-5.6-sol");
    let creds = prepare_sampling_credentials(
        &codex,
        &endpoints,
        Some(XAI_FAKE),
        Some(CODEX_FAKE),
    );
    let prepared = prepared_sampling_config_from_credentials(
        &codex,
        creds,
        None,
        None,
        None,
        None,
    );
    let existing = xai_chat_state::Credentials {
        api_key: Some(XAI_FAKE.to_owned()),
        auth_type: xai_chat_state::AuthType::SessionToken,
        alpha_test_key: None,
        client_version: None,
    };
    let cw = NonZeroU64::new(256_000).unwrap();
    let (chat_sampling, out_creds) =
        apply_prepared_sampling_to_chat_state_fields(&prepared, &existing, cw);
    assert_eq!(out_creds.auth_type, prepared.auth_type);
    assert_eq!(out_creds.api_key.as_deref(), Some(CODEX_FAKE));
    assert_eq!(chat_sampling.base_url, prepared.sampler_config.base_url);
    assert_eq!(chat_sampling.model, prepared.sampler_config.model);
}

/// SC-3: compose production transforms A/B only (no parallel field-copy).
#[test]
fn switch_changes_next_sample_route() {
    let endpoints = deterministic_endpoints();
    let xai_entry = catalog_entry("grok-build");
    let codex_entry = catalog_entry("gpt-5.6-sol");
    let existing = xai_chat_state::Credentials::default();
    let cw = NonZeroU64::new(256_000).unwrap();

    // 1) Prepare for model A (xAI) via production dual-key path
    let xai_creds = prepare_sampling_credentials(
        &xai_entry,
        &endpoints,
        Some(XAI_FAKE),
        Some(CODEX_FAKE),
    );
    let prepared_xai = prepared_sampling_config_from_credentials(
        &xai_entry,
        xai_creds,
        None,
        None,
        None,
        None,
    );
    // 2) Transform A → chat-state
    let (chat_xai, creds_xai) =
        apply_prepared_sampling_to_chat_state_fields(&prepared_xai, &existing, cw);

    // Switch to Codex: prepare + transform A again
    let codex_creds = prepare_sampling_credentials(
        &codex_entry,
        &endpoints,
        Some(XAI_FAKE),
        Some(CODEX_FAKE),
    );
    let prepared_codex = prepared_sampling_config_from_credentials(
        &codex_entry,
        codex_creds,
        None,
        None,
        None,
        None,
    );
    let (chat_codex, creds_codex) =
        apply_prepared_sampling_to_chat_state_fields(&prepared_codex, &creds_xai, cw);

    // 3) Transform B / should_attach from catalog Option provider
    let facts_xai = ModelAuthFacts {
        byok: xai_grok_shell::agent::auth_method::ModelByok::NotByok,
        auth_scheme: Default::default(),
        provider: Some(ModelProvider::Xai),
    };
    let facts_codex = ModelAuthFacts {
        byok: xai_grok_shell::agent::auth_method::ModelByok::NotByok,
        auth_scheme: Default::default(),
        provider: Some(ModelProvider::Codex),
    };
    let attach_xai = reconstruct_attach_policy_from_facts(&facts_xai, true);
    let attach_codex = reconstruct_attach_policy_from_facts(&facts_codex, true);

    assert_ne!(
        chat_xai.base_url, chat_codex.base_url,
        "switch must change base_url"
    );
    assert_eq!(creds_xai.api_key.as_deref(), Some(XAI_FAKE));
    assert_eq!(creds_codex.api_key.as_deref(), Some(CODEX_FAKE));
    assert!(
        attach_xai,
        "xAI model with gate active must allow xAI bearer attach"
    );
    assert!(
        !attach_codex,
        "Codex model must never attach xAI AuthManager bearer"
    );
    assert_eq!(chat_codex.base_url, CODEX_BASE_URL_DEFAULT);
}

/// Missing file vs parse error are distinguishable; both fail closed for keys.
#[test]
fn read_provider_auth_store_missing_vs_parse_error() {
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("no-such-auth.json");
    let missing_result = read_provider_auth_store(&missing, PROVIDER_CODEX);
    assert!(
        matches!(missing_result, Ok(None)),
        "missing file must be Ok(None); got {missing_result:?}"
    );

    let bad = dir.path().join("auth.json");
    std::fs::write(&bad, "{not-valid-json").unwrap();
    let parse_result = read_provider_auth_store(&bad, PROVIDER_CODEX);
    assert!(
        matches!(parse_result, Err(AuthStoreReadError::Parse { .. })),
        "malformed JSON must be Parse error; got {parse_result:?}"
    );
}

#[test]
fn should_attach_xai_auth_manager_bearer_resolver_matrix() {
    assert!(should_attach_xai_auth_manager_bearer_resolver(
        Some(ModelProvider::Xai),
        true
    ));
    assert!(!should_attach_xai_auth_manager_bearer_resolver(
        Some(ModelProvider::Xai),
        false
    ));
    assert!(!should_attach_xai_auth_manager_bearer_resolver(
        Some(ModelProvider::Codex),
        true
    ));
    assert!(!should_attach_xai_auth_manager_bearer_resolver(None, true));
    assert!(!should_attach_xai_auth_manager_bearer_resolver(None, false));
}

#[test]
fn model_auth_facts_provider_some_for_catalog() {
    let grok = resolve_model_auth_facts("grok-build");
    assert_eq!(
        grok.provider,
        Some(ModelProvider::Xai),
        "grok-build must resolve Some(Xai); got {:?}",
        grok.provider
    );
    let sol = resolve_model_auth_facts("gpt-5.6-sol");
    assert_eq!(
        sol.provider,
        Some(ModelProvider::Codex),
        "gpt-5.6-sol must resolve Some(Codex); got {:?}",
        sol.provider
    );
}

#[test]
fn model_auth_facts_provider_none_for_unknown() {
    let empty = resolve_model_auth_facts("");
    assert_eq!(empty.provider, None, "empty model id → provider None");
    let absent = resolve_model_auth_facts("definitely-not-a-real-model-id-zzzz");
    assert_eq!(
        absent.provider, None,
        "absent model id → provider None (never default Xai)"
    );
}

#[test]
fn reconstruct_policy_unknown_model_no_xai_resolver() {
    let facts = ModelAuthFacts {
        byok: xai_grok_shell::agent::auth_method::ModelByok::Unknown,
        auth_scheme: Default::default(),
        provider: None,
    };
    assert!(
        !reconstruct_attach_policy_from_facts(&facts, true),
        "provider None must not attach xAI resolver even when gate active"
    );
}

/// Optional weaker pure-constructor lock (different name from SC-3 contract).
/// Documents that sampling_config_for_model mirrors entry base_url + session key.
#[test]
fn sampling_config_for_model_differs_by_model() {
    let xai = catalog_entry("grok-build");
    let mut codex = catalog_entry("gpt-5.6-sol");
    // Force distinct base for pure constructor comparison until catalog stamps Codex URL.
    codex.info.base_url = CODEX_BASE_URL_DEFAULT.to_owned();

    let xai_cfg = sampling_config_for_model(
        &xai,
        resolve_credentials(&xai, Some(XAI_FAKE)),
        None,
        None,
        None,
        None,
    );
    let codex_cfg = sampling_config_for_model(
        &codex,
        resolve_credentials(&codex, Some(CODEX_FAKE)),
        None,
        None,
        None,
        None,
    );

    assert_ne!(
        xai_cfg.base_url, codex_cfg.base_url,
        "pure helper: xAI and Codex sampling configs must differ in base_url"
    );
    assert_eq!(xai_cfg.api_key.as_deref(), Some(XAI_FAKE));
    assert_eq!(codex_cfg.api_key.as_deref(), Some(CODEX_FAKE));
}

/// MOD-04 regression: cli-chat-proxy bases still get X-XAI-Token-Auth headers.
#[test]
fn xai_proxy_headers_still_apply() {
    let endpoints = deterministic_endpoints();
    let proxy = endpoints.proxy_url();
    assert_eq!(
        proxy, CLI_CHAT_PROXY_BASE_URL_DEFAULT,
        "deterministic endpoints must pin cli-chat-proxy default"
    );

    let mut headers = IndexMap::new();
    inject_url_derived_headers(&mut headers, None, &proxy);
    assert_eq!(
        headers.get("X-XAI-Token-Auth").map(String::as_str),
        Some("xai-grok-cli"),
        "proxy base must insert X-XAI-Token-Auth; headers={headers:?}"
    );
    assert_eq!(
        headers.get("x-authenticateresponse").map(String::as_str),
        Some("authenticate-response"),
        "proxy base must insert x-authenticateresponse; headers={headers:?}"
    );
}

/// D-08: gpt-5.6-sol sampling config preserves model-entry api_backend (responses).
#[test]
fn sampling_config_api_backend_from_model() {
    let entry = catalog_entry("gpt-5.6-sol");
    assert_eq!(
        entry.info.api_backend,
        ApiBackend::Responses,
        "gpt-5.6-sol catalog api_backend must be responses (D-08)"
    );

    let cfg = sampling_config_for_model(
        &entry,
        resolve_credentials(&entry, Some(CODEX_FAKE)),
        None,
        None,
        None,
        None,
    );
    assert_eq!(
        cfg.api_backend,
        ApiBackend::Responses,
        "sampling_config_for_model must preserve entry api_backend Responses"
    );
    assert_eq!(cfg.model, entry.info.model);
}

// =============================================================================
// Plan 05 — local fail-closed + on-wire Authorization (D-11 / D-12 / D-14)
// =============================================================================

/// D-11: missing static key and no resolver → Auth before any HTTP (mock count 0).
#[tokio::test]
async fn missing_credentials_fail_closed_no_http() {
    let server = MockInferenceServer::start().await.expect("start mock");
    server.set_response("should-not-be-reached");
    let cfg = sampler_config_for_mock(
        &server.url(),
        None,
        "test-model",
        ApiBackend::ChatCompletions,
        IndexMap::new(),
        None,
    );
    let client = Client::new(cfg).expect("SamplerConfig without key may construct");
    let result = client
        .conversation_stream(ConversationRequest::from_items(vec![ConversationItem::user(
            "hi",
        )]))
        .await;
    match result {
        Err(SamplingError::Auth(_)) => {}
        Err(e) => panic!("expected local Auth fail-closed, got {e:?}"),
        Ok(_) => panic!("expected local Auth fail-closed, got Ok(stream)"),
    }
    assert_eq!(
        server.request_count(),
        0,
        "fail-closed must send zero HTTP requests"
    );
}

/// Cycle 2 HIGH: resolver present but returns None → no unauthenticated HTTP.
#[tokio::test]
async fn empty_live_resolver_fail_closed_no_http() {
    let server = MockInferenceServer::start().await.expect("start mock");
    server.set_response("should-not-be-reached");
    let cfg = sampler_config_for_mock(
        &server.url(),
        None,
        "test-model",
        ApiBackend::ChatCompletions,
        IndexMap::new(),
        Some(Arc::new(StubBearerResolver(None))),
    );
    let client = Client::new(cfg).expect("construct with empty live resolver");
    let result = client
        .conversation_stream(ConversationRequest::from_items(vec![ConversationItem::user(
            "hi",
        )]))
        .await;
    match result {
        Err(SamplingError::Auth(_)) => {}
        Err(e) => panic!("expected local Auth when resolver yields None, got {e:?}"),
        Ok(_) => panic!("expected local Auth when resolver yields None, got Ok(stream)"),
    }
    assert_eq!(server.request_count(), 0);
}

/// Cycle 2 HIGH: resolver returns whitespace-only → no unauthenticated HTTP.
#[tokio::test]
async fn whitespace_live_resolver_fail_closed_no_http() {
    let server = MockInferenceServer::start().await.expect("start mock");
    server.set_response("should-not-be-reached");
    let cfg = sampler_config_for_mock(
        &server.url(),
        None,
        "test-model",
        ApiBackend::ChatCompletions,
        IndexMap::new(),
        Some(Arc::new(StubBearerResolver(Some("   \t  ")))),
    );
    let client = Client::new(cfg).expect("construct with whitespace live resolver");
    let result = client
        .conversation_stream(ConversationRequest::from_items(vec![ConversationItem::user(
            "hi",
        )]))
        .await;
    match result {
        Err(SamplingError::Auth(_)) => {}
        Err(e) => panic!("expected local Auth when resolver yields whitespace, got {e:?}"),
        Ok(_) => panic!("expected local Auth when resolver yields whitespace, got Ok(stream)"),
    }
    assert_eq!(server.request_count(), 0);
}

/// Cycle 2 HIGH: blank/whitespace static key + no usable resolver → Auth, zero HTTP.
#[tokio::test]
async fn blank_static_key_fail_closed_no_http() {
    let server = MockInferenceServer::start().await.expect("start mock");
    server.set_response("should-not-be-reached");

    for blank in ["", "   ", "\t\n"] {
        let cfg = sampler_config_for_mock(
            &server.url(),
            Some(blank),
            "test-model",
            ApiBackend::ChatCompletions,
            IndexMap::new(),
            None,
        );
        let client = Client::new(cfg).expect("blank static key may construct");
        let result = client
            .conversation_stream(ConversationRequest::from_items(vec![
                ConversationItem::user("hi"),
            ]))
            .await;
        match result {
            Err(SamplingError::Auth(_)) => {}
            Err(e) => panic!("blank static key {blank:?} must fail closed locally, got {e:?}"),
            Ok(_) => panic!("blank static key {blank:?} must fail closed locally, got Ok(stream)"),
        }
    }
    assert_eq!(
        server.request_count(),
        0,
        "blank static keys must not hit mock"
    );
}

/// D-12: xAI-built SamplerConfig sends Authorization Bearer xai-fake-token on the wire.
#[tokio::test]
async fn on_wire_authorization_xai_fake() {
    let server = MockInferenceServer::start().await.expect("start mock");
    server.set_response("OK");

    let endpoints = deterministic_endpoints();
    let entry = catalog_entry("grok-build");
    let creds =
        resolve_credentials_for_provider(&entry, &endpoints, Some(XAI_FAKE), Some(CODEX_FAKE));
    assert_eq!(creds.api_key.as_deref(), Some(XAI_FAKE));

    let mut cfg = sampling_config_for_model(&entry, creds, None, None, None, None);
    // Point live traffic at mock while preserving production-built headers/token.
    cfg.base_url = server.url();
    // Mock is not cli-chat-proxy — drop proxy-only headers so Authorization is the auth under test.
    cfg.extra_headers
        .shift_remove("X-XAI-Token-Auth");
    cfg.extra_headers
        .shift_remove("x-authenticateresponse");

    let client = Client::new(cfg).expect("xAI client with fake token");
    let (mut stream, _) = client
        .conversation_stream(ConversationRequest::from_items(vec![ConversationItem::user(
            "hello xai",
        )]))
        .await
        .expect("sample should succeed with xai-fake-token");
    while stream.next().await.is_some() {}

    let req = server
        .requests()
        .into_iter()
        .find(|r| r.path.contains("chat/completions") || r.path.contains("responses"))
        .expect("mock must record an inference request");
    let expected = format!("Bearer {XAI_FAKE}");
    assert_eq!(
        req.header("authorization"),
        Some(expected.as_str()),
        "xAI path must send Authorization Bearer xai-fake-token"
    );
}

/// D-12: Codex-built SamplerConfig sends codex-fake-token; never X-XAI-Token-Auth on non-proxy mock.
#[tokio::test]
async fn on_wire_authorization_codex_fake() {
    let server = MockInferenceServer::start().await.expect("start mock");
    server.set_response("OK");

    let endpoints = deterministic_endpoints();
    let entry = catalog_entry("gpt-5.6-sol");
    let creds =
        resolve_credentials_for_provider(&entry, &endpoints, Some(XAI_FAKE), Some(CODEX_FAKE));
    assert_eq!(creds.api_key.as_deref(), Some(CODEX_FAKE));
    assert_ne!(creds.api_key.as_deref(), Some(XAI_FAKE));

    let mut cfg = sampling_config_for_model(&entry, creds, None, None, None, None);
    cfg.base_url = server.url();
    // Codex production path must not have injected X-XAI-Token-Auth.
    assert!(
        !cfg.extra_headers.contains_key("X-XAI-Token-Auth"),
        "Codex sampling_config must not carry X-XAI-Token-Auth; headers={:?}",
        cfg.extra_headers
    );

    let client = Client::new(cfg).expect("Codex client with fake token");
    // gpt-5.6-sol is Responses backend.
    let (mut stream, _, _) = client
        .conversation_stream_responses(ConversationRequest::from_items(vec![
            ConversationItem::user("hello codex"),
        ]))
        .await
        .expect("sample should succeed with codex-fake-token");
    while stream.next().await.is_some() {}

    let req = server
        .requests()
        .into_iter()
        .find(|r| r.path.contains("responses") || r.path.contains("chat/completions"))
        .expect("mock must record an inference request");
    let expected = format!("Bearer {CODEX_FAKE}");
    assert_eq!(
        req.header("authorization"),
        Some(expected.as_str()),
        "Codex path must send Authorization Bearer codex-fake-token"
    );
    assert!(
        req.header("x-xai-token-auth").is_none(),
        "Codex non-proxy mock must not receive X-XAI-Token-Auth"
    );
    // Dual-token isolation on the wire: never the xAI fixture.
    let xai_bearer = format!("Bearer {XAI_FAKE}");
    assert_ne!(
        req.header("authorization"),
        Some(xai_bearer.as_str()),
        "Codex request must never carry xai-fake-token"
    );
}

/// D-11 construction: resolve with empty Codex slot still builds SamplerConfig without panic.
#[test]
fn empty_codex_credentials_constructs_route_without_panic() {
    let endpoints = deterministic_endpoints();
    let entry = catalog_entry("gpt-5.6-sol");
    let creds = resolve_credentials_for_provider(&entry, &endpoints, Some(XAI_FAKE), None);
    assert!(creds.api_key.is_none());
    let cfg = sampling_config_for_model(&entry, creds, None, None, None, None);
    assert!(cfg.api_key.is_none());
    // Live client may construct; sample is fail-closed (covered by missing_credentials_*).
    let _client = Client::new(cfg).expect("empty Codex key construction must not panic");
}

/// D-14: invalid-header conversion arms must not log full api_key = %api_key.
#[test]
fn invalid_header_path_does_not_log_full_key() {
    let client_src = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../xai-grok-sampler/src/client.rs"
    ));
    assert!(
        !client_src.contains("api_key = %api_key"),
        "SamplingClient invalid-header paths must not log full api_key (D-14)"
    );
    // Redacted diagnostics should still mention invalid api_key + length-style fields.
    assert!(
        client_src.contains("Invalid api_key"),
        "invalid-header path should still log a redacted diagnostic"
    );
}
