//! Phase 4 provider-aware request routing — Wave 0 integration harness.
//!
//! **Scope (D-12 / D-13):** public APIs only (`resolve_model_list`, `resolve_credentials`,
//! `sampling_config_for_model`, `inject_url_derived_headers`, catalog types). No shell
//! `--lib` gates. Fake tokens only (`xai-fake-token` / `codex-fake-token`) — no live
//! ChatGPT OAuth, no Phase 5 login UX, no Phase 6 missing-provider gate.
//!
//! **CI RED policy:** this binary intentionally commits behavior-RED contracts for
//! MOD-04/MOD-05. Sequential GSD execution on the phase branch: Plan 01 RED →
//! Plans 02–05 GREEN before mainline CI that runs all integration tests. Do not
//! `#[ignore]` core contracts.
//!
//! **04-REVIEWS (HIGH):**
//! - `switch_changes_next_sample_route` is the production-path contract
//!   (prepare → chat-state / SetSessionModel → reconstruct). Plan 04 wires transforms
//!   A/B. Wave 0 reserves the name as intentional RED — pure dual
//!   `sampling_config_for_model` does **not** fulfill SC-3.
//! - `never_cross_slot` is dual-token isolation (both tokens present simultaneously).
//!   Plan 03 lands `resolve_credentials_for_provider` GREEN.
//!
//! Prefer `cargo test -p xai-grok-shell --test provider_routing` only.

use indexmap::IndexMap;
use serial_test::serial;
use xai_grok_shell::agent::config::{
    inject_url_derived_headers, resolve_credentials, resolve_model_list,
    resolve_provider_route, sampling_config_for_model, Config, ConfigModelOverride,
    EndpointsConfig, ModelEntry, ModelProvider, CLI_CHAT_PROXY_BASE_URL_DEFAULT,
    CODEX_BASE_URL_DEFAULT, XAI_API_BASE_URL_DEFAULT,
};
use xai_grok_shell::sampling::ApiBackend;
use xai_grok_test_support::EnvGuard;

const XAI_FAKE: &str = "xai-fake-token";
const CODEX_FAKE: &str = "codex-fake-token";

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
    let override_url = "https://byok.example/v1";
    let mut entry = catalog_entry("gpt-5.6-sol");
    entry.info.provider = ModelProvider::Codex;
    entry.info.base_url = override_url.to_owned();
    // Clear api_base_url so resolve does not fall through to xAI Platform.
    entry.api_base_url = None;

    let creds = resolve_credentials(&entry, Some(CODEX_FAKE));
    assert_eq!(
        creds.base_url, override_url,
        "D-04: explicit model base_url override must win; got {}",
        creds.base_url
    );
    assert_eq!(creds.api_key.as_deref(), Some(CODEX_FAKE));
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
/// Final GREEN (Plan 03): public dual-key API
/// `resolve_credentials_for_provider(model, endpoints, Some(xai_key), Some(codex_key))`
/// with **both** tokens present returns only the provider-correct key:
/// - Codex model → `codex-fake-token` only (never `xai-fake-token`)
/// - xAI model → `xai-fake-token` only (never `codex-fake-token`)
///
/// Wave 0/Task 2: both tokens always in locals; prove today's single-key API is a
/// cross-slot defect vector (Codex accepts xAI key); fail until dual-key lands.
/// Do **not** reduce this to two independent single-key resolves that each only
/// see one token — that cannot detect cross-slot bugs.
#[test]
fn never_cross_slot() {
    // Both tokens present simultaneously — dual-token fixture (required).
    let xai_key = XAI_FAKE;
    let codex_key = CODEX_FAKE;
    let dual_map = [( "xai", xai_key ), ( "codex", codex_key )];
    assert_eq!(dual_map.len(), 2, "dual-token fixture must hold both slots");
    assert_ne!(xai_key, codex_key);

    let xai_entry = catalog_entry("grok-build");
    let codex_entry = catalog_entry("gpt-5.6-sol");
    assert_eq!(xai_entry.info.provider.as_str(), "xai");
    assert_eq!(codex_entry.info.provider.as_str(), "codex");

    // Documented defect: single-key resolve_credentials is provider-blind.
    // When the wrong slot key is offered for a Codex model, it is accepted today.
    let codex_with_xai_key = resolve_credentials(&codex_entry, Some(xai_key));
    assert_eq!(
        codex_with_xai_key.api_key.as_deref(),
        Some(xai_key),
        "precondition: current single-key API wrongly accepts xAI key for Codex model"
    );
    let xai_with_codex_key = resolve_credentials(&xai_entry, Some(codex_key));
    assert_eq!(
        xai_with_codex_key.api_key.as_deref(),
        Some(codex_key),
        "precondition: current single-key API wrongly accepts Codex key for xAI model"
    );

    // Plan 03 GREEN: dual-key API with both Some(...) never returns the wrong slot.
    // Until that symbol is public, keep compiling via intentional RED scaffold.
    let dual_key_api_ready = false;
    assert!(
        dual_key_api_ready,
        "Plan 03: resolve_credentials_for_provider dual-token isolation — \
         both tokens present (xai={xai_key}, codex={codex_key}): \
         dual_key(codex, both) == {codex_key} only AND dual_key(xai, both) == {xai_key} only \
         (Codex must not receive xai-fake; xAI must not receive codex-fake)"
    );
}

/// SC-3 production-path scaffold (04-REVIEWS HIGH cycles 1+2).
///
/// Final GREEN (Plan 04): compose production transforms A/B —
/// `PreparedSamplingConfig` → chat-state apply → reconstruct attach —
/// so prepare → SetSessionModel/chat-state → reconstruct changes next sample
/// route (base_url + credential slot). Pure dual `sampling_config_for_model`
/// does **not** fulfill this contract.
#[test]
fn switch_changes_next_sample_route() {
    assert!(
        false,
        "Plan 04: wire prepare/reconstruct production transforms A/B — \
         switch_changes_next_sample_route is the production-path contract \
         (prepare → chat-state/SetSessionModel → reconstruct); pure dual \
         sampling_config_for_model alone does not fulfill SC-3"
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
