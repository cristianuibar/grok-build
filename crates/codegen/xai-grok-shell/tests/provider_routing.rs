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
use xai_grok_shell::agent::config::{
    inject_url_derived_headers, resolve_credentials, resolve_model_list,
    sampling_config_for_model, Config, EndpointsConfig, ModelEntry, ModelProvider,
    CLI_CHAT_PROXY_BASE_URL_DEFAULT, XAI_API_BASE_URL_DEFAULT,
};

/// Locked Codex/ChatGPT backend default (Plan 02 exports `CODEX_BASE_URL_DEFAULT`).
const CODEX_BASE_URL_DEFAULT: &str = "https://chatgpt.com/backend-api/codex";

const XAI_FAKE: &str = "xai-fake-token";
const CODEX_FAKE: &str = "codex-fake-token";

/// Deterministic endpoints for route assertions (avoid ambient env-only equality).
fn deterministic_endpoints() -> EndpointsConfig {
    EndpointsConfig {
        cli_chat_proxy_base_url: Some(CLI_CHAT_PROXY_BASE_URL_DEFAULT.to_owned()),
        xai_api_base_url: XAI_API_BASE_URL_DEFAULT.to_owned(),
        ..EndpointsConfig::default()
    }
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
    let entry = catalog_entry("grok-build");
    assert_eq!(
        entry.info.provider.as_str(),
        "xai",
        "grok-build provider must be wire id \"xai\""
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
/// Behavior-RED until Plans 02–03 stamp Codex base_url and dual-key credentials.
#[test]
fn codex_model_routes_to_codex_backend_with_codex_token() {
    let entry = catalog_entry("gpt-5.6-sol");
    assert_eq!(
        entry.info.provider.as_str(),
        "codex",
        "gpt-5.6-sol provider must be wire id \"codex\" (D-02)"
    );

    // Catalog-level RED: stamped base_url must be Codex default, not cli-chat-proxy.
    assert!(
        !entry.info.base_url.contains("cli-chat-proxy"),
        "MOD-05 RED: gpt-5.6-sol base_url must not contain cli-chat-proxy; got {} \
         (Plan 02/03: stamp Codex rows with {CODEX_BASE_URL_DEFAULT})",
        entry.info.base_url
    );
    assert_eq!(
        entry.info.base_url, CODEX_BASE_URL_DEFAULT,
        "MOD-05 RED: gpt-5.6-sol base_url must equal Codex default {CODEX_BASE_URL_DEFAULT}; got {}",
        entry.info.base_url
    );

    let creds = resolve_credentials(&entry, Some(CODEX_FAKE));
    assert_eq!(
        creds.api_key.as_deref(),
        Some(CODEX_FAKE),
        "Codex path must use Codex fake token only"
    );
    assert!(
        !creds.base_url.contains("cli-chat-proxy"),
        "MOD-05: resolved Codex base_url must not contain cli-chat-proxy; got {}",
        creds.base_url
    );
    assert_eq!(
        creds.base_url, CODEX_BASE_URL_DEFAULT,
        "MOD-05 RED: resolved Codex base_url must be {CODEX_BASE_URL_DEFAULT}; got {}",
        creds.base_url
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

/// Dual-token isolation (D-09 / T-04-02).
///
/// Final GREEN (Plan 03): `resolve_credentials_for_provider(model, endpoints,
/// Some(xai-fake), Some(codex-fake))` returns only the provider-correct key —
/// Codex never receives xai-fake; xAI never receives codex-fake.
///
/// Wave 0: both tokens always in scope; assert intentional RED that dual-key
/// API is not ready yet (do not leave as two independent single-key resolves
/// that each only see one token — that cannot detect cross-slot bugs).
#[test]
fn never_cross_slot() {
    let xai_key = XAI_FAKE;
    let codex_key = CODEX_FAKE;
    // Both tokens present simultaneously — required dual-token fixture.
    let _both_present = (xai_key, codex_key);

    let xai_entry = catalog_entry("grok-build");
    let codex_entry = catalog_entry("gpt-5.6-sol");
    assert_eq!(xai_entry.info.provider.as_str(), "xai");
    assert_eq!(codex_entry.info.provider.as_str(), "codex");

    // Today's single-key API is a defect vector: feeding the wrong slot key
    // is accepted because resolve_credentials is provider-blind.
    let wrongly_fed = resolve_credentials(&codex_entry, Some(xai_key));
    assert_eq!(
        wrongly_fed.api_key.as_deref(),
        Some(xai_key),
        "precondition: current single-key API accepts xAI key for Codex model \
         (cross-slot defect Plan 03 must close)"
    );

    // Plan 03 GREEN target: dual-key selection with both tokens present.
    let dual_key_api_ready = false;
    assert!(
        dual_key_api_ready,
        "Plan 03: resolve_credentials_for_provider dual-token isolation — \
         with both tokens present (xai={xai_key}, codex={codex_key}), \
         Codex model must yield only {codex_key} and xAI model only {xai_key}"
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
