# Phase 4: Provider-aware request routing - Pattern Map

**Mapped:** 2026-07-16
**Files analyzed:** 13
**Analogs found:** 13 / 13

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `xai-grok-shell/src/agent/config.rs` — `CODEX_BASE_URL_DEFAULT` + `EndpointsConfig` | config / utility | transform (env → URL) | same file: `CLI_CHAT_PROXY_BASE_URL_DEFAULT`, `XAI_API_BASE_URL_DEFAULT`, `proxy_url` / `resolve_inference_base_url` | exact |
| `xai-grok-shell/src/agent/config.rs` — `resolve_provider_route` (new pure fn) | utility / service | transform | same file: `resolve_credentials` + `EndpointsConfig` resolvers | exact |
| `xai-grok-shell/src/agent/config.rs` — `default_models()` URL stamping | service | batch / transform | same fn (~3432–3484): stamps all rows with proxy + `xai_api_base_url` today | exact |
| `xai-grok-shell/src/agent/config.rs` — `resolve_credentials` (+ provider-aware sibling) | service | transform | same fn (~4377–4421) priority chain | exact |
| `xai-grok-shell/src/agent/config.rs` — `sampling_config_for_model` / `inject_url_derived_headers` | service | transform → request-response | same fns (~4689–4770) | exact |
| `xai-grok-shell/src/agent/models.rs` — `ModelsManager::sampling_config` | service | request-response | same method (~929–962) | exact |
| `xai-grok-shell/src/agent/mvp_agent/agent_ops.rs` — `prepare_sampling_config_for_model` | service | request-response | same method (~1084–1165) **PRIMARY builder** | exact |
| `xai-grok-shell/src/session/acp_session_impl/sampler_turn.rs` — `reconstruct_full_config` + bearer | service / middleware | request-response / streaming | same method (~231–353) + local `AuthManagerBearerResolver` | exact |
| `xai-grok-shell/src/session/acp_session_impl/model_switch.rs` — `handle_set_session_model` | service | request-response | same method (~48–76) writes chat-state route | exact (verify fields already flow) |
| `xai-grok-shell/src/agent/handlers/model_switch.rs` | controller | request-response | same (~115–199): already calls `prepare_sampling_config_for_model` | inherit |
| `xai-grok-sampler/src/config.rs` + `client.rs` | model / client | streaming request-response | `SamplerConfig` / `BearerResolver` / `post()` | inherit (reuse; no parallel client) |
| `xai-grok-shell/src/auth/{model,storage,manager,credential_provider}.rs` | model / service | file-I/O + request-response | Phase 2 multi-slot + `write_fixture_auth_document` | role-match (slot ids + fixtures; leave ShellAuth on xAI) |
| `xai-grok-shell/tests/provider_routing.rs` (**new**) | test | transform + request-response | `tests/model_catalog.rs` + `tests/auth_multi_slot.rs` + config unit tests | exact |

**Reference-only (already exist — do not reinvent):**
- `ModelProvider` enum + `as_str()` — `agent/config.rs` ~3369–3393
- `PROVIDER_XAI` / `PROVIDER_CODEX` — `auth/model.rs` ~16–20
- GPT catalog rows with `provider: codex`, `api_backend: responses` — `default_models.json` (Phase 3)
- Chat-state `SamplingConfig` / `Credentials` update on switch — session `model_switch.rs`

**Likely no product code change:** `xai-grok-sampler` client wire path; `handlers/model_switch.rs` orchestration; `ShellAuthCredentialProvider` (xAI proxy/upload only).

---

## Pattern Assignments

### `EndpointsConfig` + Codex default URL (config, transform)

**Analog:** `crates/codegen/xai-grok-shell/src/agent/config.rs`

**Constants pattern** (lines 46–48):
```rust
/// Default base URL for the cli chat proxy.
pub const CLI_CHAT_PROXY_BASE_URL_DEFAULT: &str = "https://cli-chat-proxy.grok.com/v1";
/// Default base URL for the public xAI API.
pub const XAI_API_BASE_URL_DEFAULT: &str = "https://api.x.ai/v1";
```

**Copy for Codex** (placement: immediately after `XAI_API_BASE_URL_DEFAULT`):
```rust
/// Default base URL for the ChatGPT / Codex backend (not Platform API).
pub const CODEX_BASE_URL_DEFAULT: &str = "https://chatgpt.com/backend-api/codex";
```

**Field + resolver pattern** — mirror `xai_api_base_url` / `proxy_url` / `resolve_inference_base_url` (lines 143–150, 253–258, 306–314, 540–545):
```rust
// On EndpointsConfig:
/// Env: `GROK_CODEX_BASE_URL`. ChatGPT Codex backend base for provider=codex models.
#[serde(default = "default_codex_base_url")] // or plain String like xai_api_base_url
pub codex_base_url: String,

// Resolver (impl EndpointsConfig):
pub fn resolve_codex_base_url(&self) -> String {
    blank_as_unset(&Some(self.codex_base_url.clone()))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| CODEX_BASE_URL_DEFAULT.to_owned())
    // Prefer same style as xai_api_base_url: non-Option String filled in Default
}

// Default (line 540+):
codex_base_url: std::env::var("GROK_CODEX_BASE_URL")
    .unwrap_or_else(|_| CODEX_BASE_URL_DEFAULT.to_owned()),
```

**Env naming analog:** `GROK_CLI_CHAT_PROXY_BASE_URL` / `GROK_XAI_API_BASE_URL` (Default impl lines 543–545). Keep `GROK_` prefix this phase (Phase 8 rebrand).

**Do not require** `xai-grok-env` changes for Phase 4 — shell `EndpointsConfig` is the SoT (env crate only has proxy/asset/ws production constants today; lines 22–33 of `xai-grok-env/src/lib.rs`).

---

### `resolve_provider_route` pure resolver (utility, transform)

**Analog:** pure resolvers on `EndpointsConfig` + credential resolve in same file; authority enum already present.

**Provider authority** (lines 3365–3393):
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ModelProvider {
    #[default]
    Xai,
    Codex,
}

impl ModelProvider {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Xai => "xai",
            Self::Codex => "codex",
        }
    }
}
```

**Credential slot constants** (`auth/model.rs` 16–20):
```rust
pub const PROVIDER_XAI: &str = "xai";
pub const PROVIDER_CODEX: &str = "codex";
```

**Prescriptive pure shape** (place next to `resolve_credentials` ~4377; unit-test without AuthManager):
```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderRoute {
    pub base_url: String,
    pub credential_slot: &'static str, // PROVIDER_XAI | PROVIDER_CODEX
    pub provider: ModelProvider,
}

pub fn resolve_provider_route(
    provider: ModelProvider,
    endpoints: &EndpointsConfig,
    model_base_url_override: Option<&str>,
) -> ProviderRoute {
    let credential_slot = match provider {
        ModelProvider::Xai => crate::auth::model::PROVIDER_XAI,
        ModelProvider::Codex => crate::auth::model::PROVIDER_CODEX,
    };
    if let Some(url) = model_base_url_override.filter(|s| !s.trim().is_empty()) {
        return ProviderRoute {
            base_url: url.to_owned(),
            credential_slot,
            provider,
        };
    }
    match provider {
        ModelProvider::Xai => ProviderRoute {
            base_url: endpoints.resolve_inference_base_url(),
            credential_slot,
            provider,
        },
        ModelProvider::Codex => ProviderRoute {
            base_url: endpoints.resolve_codex_base_url(),
            credential_slot,
            provider,
        },
    }
}
```

**Override rule (locked):** explicit model `base_url` / BYOK `api_key`/`env_key` win for URL/key; provider still selects **credential slot** when falling through to session/OAuth credentials. Do **not** use `agent_type` for routing.

---

### `default_models()` provider-aware URL stamping (service, batch)

**Analog / broken baseline:** same function lines 3432–3484

```rust
let config = ModelEntryConfig {
    id: m.id,
    model: m.model,
    base_url: endpoints.resolve_inference_base_url(),           // ALL rows today
    api_base_url: Some(endpoints.xai_api_base_url.clone()),     // ALL rows today
    // ...
    provider: m.provider,
    api_backend: m.api_backend,
    // ...
};
```

**Copy structure; branch on `m.provider`:**
```rust
let (base_url, api_base_url) = match m.provider {
    ModelProvider::Xai => (
        endpoints.resolve_inference_base_url(),
        Some(endpoints.xai_api_base_url.clone()),
    ),
    ModelProvider::Codex => (
        endpoints.resolve_codex_base_url(),
        None, // no xAI Platform fallback for Codex product path
    ),
};
```

**Regression test to update:** `default_models_dual_endpoint_routing` (lines 5568–5593) currently asserts **every** entry with `api_base_url` routes session → proxy. After stamping fix, Codex rows have `api_base_url: None` and session base = Codex URL — rewrite assertions by `entry.info.provider` (or `entry.provider` on config entry).

---

### `resolve_credentials` provider-aware (service, transform)

**Analog:** existing priority chain lines 4373–4421

```rust
/// Priority: model api_key/env_key > session token > XAI_API_KEY.
pub fn resolve_credentials(model: &ModelEntry, session_key: Option<&str>) -> ResolvedCredentials {
    let info = model.info();
    let (api_key, base_url, auth_type) = if let Some(key) = model.own_credential() {
        (Some(key), info.base_url.clone(), xai_chat_state::AuthType::ApiKey)
    } else if let Some(key) = session_key {
        (Some(key.to_owned()), info.base_url.clone(), xai_chat_state::AuthType::SessionToken)
    } else if let Ok(key) = crate::agent::auth_method::read_xai_api_key_env() {
        let url = model.api_base_url.clone().unwrap_or_else(|| info.base_url.clone());
        (Some(key), url, xai_chat_state::AuthType::ApiKey)
    } else {
        (None, info.base_url.clone(), xai_chat_state::AuthType::ApiKey)
    };
    // ...
}
```

**`ResolvedCredentials` shape** (lines 4354–4359) — keep; base_url remains the route field consumers trust:
```rust
pub struct ResolvedCredentials {
    pub api_key: Option<String>,
    pub base_url: String,
    pub auth_type: xai_chat_state::AuthType,
    pub auth_scheme: AuthScheme,
}
```

**Extend priority (preserve order, add provider awareness):**
1. Model own credential (`own_credential`) → keep model `info.base_url` / BYOK
2. Else **provider-selected** session/fixture token for that slot (caller passes correct key; or add `session_key_for_provider` arg / dual keys)
3. Else **xAI only:** `XAI_API_KEY` + `api_base_url` fallback — **never** for `ModelProvider::Codex`
4. Else empty key + provider-default base_url from `resolve_provider_route` when catalog stamp missing

**Suggested signature evolution** (planner discretion — either is fine):
- Keep `resolve_credentials(model, session_key)` but ensure callers pass **slot-correct** `session_key` and set `info.base_url` correctly via catalog stamping / route apply; **or**
- Add `resolve_credentials_for_provider(model, xai_key, codex_key, endpoints)` used by prepare path.

**Debug logging pattern** (lines 4413–4415) — extend fields, never log full secrets:
```rust
tracing::debug!(
    model = %info.model,
    provider = %info.provider.as_str(),
    auth_type = ?auth_type,
    base_url = %base_url,
    "resolved credentials"
);
```

**Kill-switch / aux:** `enforce_disable_api_key_auth` + `resolve_credentials_enforced` (4426–4457) stay xAI-first-party URL gated via `is_first_party_xai_url` — Codex ChatGPT base must **not** match first-party xAI URL helper (verify).

---

### `sampling_config_for_model` + `inject_url_derived_headers` (service, transform)

**Analog:** lines 4689–4770

```rust
pub fn sampling_config_for_model(
    model: &ModelEntry,
    credentials: ResolvedCredentials,
    alpha_test_key: Option<String>,
    client_version: Option<String>,
    deployment_id: Option<String>,
    user_id: Option<String>,
) -> SamplerConfig {
    let info = model.info();
    let mut extra_headers = info.extra_headers.clone();
    inject_url_derived_headers(
        &mut extra_headers,
        alpha_test_key.as_deref(),
        &credentials.base_url,
    );
    SamplerConfig {
        api_key: credentials.api_key,
        model: info.model.clone(),
        base_url: credentials.base_url,
        api_backend: info.api_backend.clone(), // model-entry driven — do not invent per-provider table
        // ...
        bearer_resolver: None,
        // ...
    }
}

pub fn inject_url_derived_headers(
    headers: &mut IndexMap<String, String>,
    alpha_test_key: Option<&str>,
    base_url: &str,
) {
    if crate::util::is_cli_chat_proxy_url(base_url) {
        headers.entry("X-XAI-Token-Auth".to_string())
            .or_insert_with(|| "xai-grok-cli".to_string());
        // ... x-authenticateresponse, CLIENT_MODE_HEADER
    }
    let _ = (alpha_test_key, base_url);
}
```

**Copy:** leave URL classification strict (`is_cli_chat_proxy_url`). Codex default host must **not** receive `X-XAI-Token-Auth`.

**Existing unit tests to mirror** (5156–5198):
- `inject_url_derived_headers_adds_proxy_headers_for_cli_chat_proxy_url`
- `inject_url_derived_headers_skips_proxy_headers_for_external_url`

Add sibling: Codex base URL → no proxy headers (same style as `api.x.ai` skip test).

---

### `prepare_sampling_config_for_model` (service, request-response) — PRIMARY

**Analog:** `crates/codegen/xai-grok-shell/src/agent/mvp_agent/agent_ops.rs` lines 1084–1165

```rust
pub(crate) fn prepare_sampling_config_for_model(
    &self,
    model: &ModelEntry,
    origin_client: Option<crate::http::OriginClientInfo>,
) -> SamplingConfig {
    let preferred = self.cfg.borrow().grok_com_config.preferred_method;
    let session = match preferred {
        Some(crate::auth::PreferredAuthMethod::ApiKey) => None,
        _ if self.is_session_based_auth() => self.auth_manager.current_or_expired(),
        _ => None,
    };
    let mut credentials = resolve_credentials(
        model,
        session.as_ref().map(|a| a.key.as_str()), // TODAY: always xAI in-memory
    );
    // OIDC preferred / disable_api_key_auth / SessionToken override...
    let mut config = crate::agent::config::sampling_config_for_model(
        model, credentials, alpha_test_key, client_version, deployment_id, user_id,
    );
    config.origin_client = origin_client;
    config
}
```

**Pattern to preserve:** preferred method gates, `enforce_disable_api_key_auth`, SessionToken overrides for session-based auth, unified_log warnings without secrets.

**Change:** when `model.info.provider == Codex`, resolve bearer from **Codex slot** (disk fixture / future multi-principal), **not** `AuthManager::current_or_expired()` xAI key. When Xai, keep current AuthManager path. Never pass xAI key into Codex `resolve_credentials` session arm (and reverse).

**Fixture inject for tests:** allow construction with fake Codex token without Phase 5 OAuth — either seed multi-slot auth.json and read slot, or inject via test-only hook / explicit model `api_key` for pure unit tests.

---

### `ModelsManager::sampling_config` (service, request-response)

**Analog:** `agent/models.rs` lines 929–962

```rust
pub fn sampling_config(&self) -> SamplingConfig {
    // resolve current_model from catalog...
    let session_auth = auth_manager.current_or_expired();
    let credentials =
        resolve_credentials(current_model, session_auth.as_ref().map(|a| a.key.as_str()));
    sampling_config_for_model(
        current_model,
        credentials,
        config.endpoints.alpha_test_key.clone(),
        // ...
    )
}
```

**Same fix as prepare path:** slot-select by `current_model.info.provider`. Keep fallback `ModelEntry::fallback` behavior for empty catalog (xAI defaults OK).

---

### `reconstruct_full_config` + BearerResolver (service, request-response)

**Analog:** `session/acp_session_impl/sampler_turn.rs` lines 228–353

```rust
struct AuthManagerBearerResolver(std::sync::Arc<crate::auth::AuthManager>);
impl xai_grok_sampler::BearerResolver for AuthManagerBearerResolver {
    fn current_bearer(&self) -> Option<String> {
        self.0.current_or_expired().map(|a| a.key) // xAI only today
    }
}
// ...
bearer_resolver: if use_bearer_resolver {
    self.auth_manager.as_ref().map(|am| -> SharedBearerResolver {
        Arc::new(AuthManagerBearerResolver(am.clone()))
    })
} else {
    None
},
```

**Sampler trait to implement** (`xai-grok-sampler/src/config.rs` 165–170):
```rust
pub trait BearerResolver: Send + Sync + std::fmt::Debug {
    fn current_bearer(&self) -> Option<String>;
}
pub type SharedBearerResolver = std::sync::Arc<dyn BearerResolver>;
```

**Live Authorization override** (`client.rs` 549–567) — unchanged consumer:
```rust
fn post(&self, url: impl reqwest::IntoUrl) -> reqwest::RequestBuilder {
    // if bearer_resolver present → overrides Authorization Bearer
}
```

**Critical anti-cross-slot pattern:**
- Wire `AuthManagerBearerResolver` **only** when active model provider is xAI **and** session-token gate is active.
- For Codex: either (a) provider-scoped resolver reading Codex slot, or (b) disable resolver and rely on `Credentials.api_key` snapshot set at switch until Phase 5 multi-principal refresh.
- Prefer (a) if a cheap disk/memory read exists; (b) is acceptable this phase if tests prove `api_key` + `base_url` survive reconstruct without overwrite.

**Observability analog** (`client.rs` 507–519, 571–588): log `base_url`, `has_api_key`, `has_bearer_resolver`, **auth header prefix only** — never full bearer.

**Also re-applies** `inject_url_derived_headers` on reconstruct (281–285) — Codex base must stay non-proxy.

---

### Session model switch path (controller → service, request-response)

**Handler already rebuilds next sample** — `agent/handlers/model_switch.rs` 115–199:
```rust
let mut model_sampling =
    agent.prepare_sampling_config_for_model(&model, handle.origin_client.clone());
// ...
handle.cmd_tx.send(SessionCommand::SetSessionModel {
    sampling_config: model_sampling,
    // ...
});
```

**Session apply** — `session/.../model_switch.rs` 48–76:
```rust
self.chat_state_handle.update_sampling_config(xai_grok_sampling_types::SamplingConfig {
    base_url: sampling_config.base_url.clone(),
    model: sampling_config.model.clone(),
    api_backend: sampling_config.api_backend.clone(),
    // ...
});
self.chat_state_handle.update_credentials(xai_chat_state::Credentials {
    api_key: sampling_config.api_key.clone(),
    auth_type: crate::agent::config::resolve_chat_state_auth_type(
        sampling_config.model.as_str(),
        session_key.as_deref(), // still xAI auth_manager for auth_type hint
        existing.auth_type,
    ),
    // ...
});
```

**Pattern:** do **not** invent a parallel switch path. Fix prepare/resolve/reconstruct so switch automatically carries provider-correct `base_url` + `api_key`. Handler remains inherit-only unless `resolve_chat_state_auth_type` needs provider context (verify; out of MOD-04/05 if auth_type already correct from SamplerConfig).

---

### Auth slots & fixtures (model / file-I/O) — reference + test inject

**Slot constants** — already shipped Phase 2 (`auth/model.rs` 16–23).

**Multi-slot document read** — `storage.rs` 141–147 returns **xAI slot only** for production AuthManager:
```rust
pub fn read_auth_json(auth_file: &Path) -> std::io::Result<AuthStore> {
    let doc = read_auth_document(auth_file)?;
    Ok(doc.providers.get(PROVIDER_XAI).cloned().unwrap_or_default())
}
```

**Test fixture helper** — `storage.rs` 660–678:
```rust
#[cfg(test)]
pub(crate) fn write_fixture_auth_document(
    path: &Path,
    xai: AuthStore,
    codex: Option<AuthStore>,
) -> std::io::Result<()> { /* providers.xai + optional providers.codex */ }
```

**Integration fixture style** — `tests/auth_multi_slot.rs` 8–48: write nested JSON with both slots; assert xAI load does not clobber Codex.

**`ShellAuthCredentialProvider`** (`credential_provider.rs` 16–51): leave on `AuthManager::current_or_expired()` — used for **proxy/upload/storage** HTTP, not chat sampling. Do **not** make it provider-global.

**`shared_api_key_provider`** (`manager.rs` 2298–2329): same xAI bridge for tools — out of MOD-04/05 sampling path; do not break.

**Optional Phase 4 minimal API:** pure/read helper `read_provider_store(path, PROVIDER_CODEX)` or load Codex fixture key for prepare path tests — prefer reusing `read_auth_document` + `providers.get(PROVIDER_CODEX)` rather than expanding AuthManager multi-principal (Phase 5).

---

### `tests/provider_routing.rs` (test, transform + request-response) — NEW

**Primary analogs:**
1. `tests/model_catalog.rs` — public crate surface, `Config::default()`, `resolve_model_list`, provider asserts
2. `tests/auth_multi_slot.rs` — tempfile multi-slot document + fake tokens
3. `config.rs` unit module — `default_models_dual_endpoint_routing`, `inject_url_derived_headers_*`, `resolve_credentials_*`

**Harness header pattern** (`model_catalog.rs` 1–11):
```rust
//! Phase 4 provider routing integration tests.
//!
//! Assert base_url + credential slot differ by catalog provider; no live ChatGPT.

use xai_grok_shell::agent::config::{
    resolve_credentials, /* resolve_provider_route, */ Config, EndpointsConfig,
    ModelEntry, ModelProvider, /* ... */
};
```

**Prescriptive cases (from RESEARCH):**
| Test | Assert |
|------|--------|
| `xai_model_routes_to_proxy_with_xai_token` | base is proxy / inference URL; key = xAI fake |
| `codex_model_routes_to_codex_backend_with_codex_token` | base = `resolve_codex_base_url()`; **not** cli-chat-proxy; key = Codex fake |
| `model_override_base_url_wins` | explicit entry base_url preferred; still no cross-slot token |
| `switch_changes_next_sample_route` | prepare_sampling for Grok vs GPT differs base_url + api_key |
| `no_proxy_headers_on_codex` | `inject_url_derived_headers` on Codex base omits `X-XAI-Token-Auth` |
| `never_cross_slot` | xAI session token not applied as Codex session arm and reverse |

**Colocated pure units:** `resolve_provider_route` + env override for `GROK_CODEX_BASE_URL` inside `config.rs` `#[cfg(test)]` with `serial_test` / env guards if mutating env (see existing env tests in config.rs ~7756+).

**Commands:**
```bash
cargo test -p xai-grok-shell --test provider_routing -q
cargo test -p xai-grok-shell --lib default_models_dual_endpoint -q
cargo check -p xai-grok-shell -p xai-grok-sampler
```

---

### Sampler crate reuse (inherit)

**`SamplerConfig`** (`config.rs` 49–127): construction-time `api_key` + optional `bearer_resolver`; `base_url` + `api_backend` + `extra_headers`. Shell builds; sampler stays URL-agnostic.

**`endpoint` join** (`client.rs` 772–776):
```rust
fn endpoint(&self, path: &str) -> String {
    format!("{base}/{path}") // base_url + "responses"
}
```

**No new client.** GPT rows already `api_backend: responses`. Only base_url + Authorization + headers change.

---

## Shared Patterns

### Catalog `provider` is routing authority
**Source:** Phase 3 `ModelProvider` on `ModelInfo` / entries; wire ids match `PROVIDER_*`  
**Apply to:** `resolve_provider_route`, `default_models` stamping, prepare/models sampling, tests  
**Not authority:** `agent_type` (harness), model id string inference

### Credential slot isolation (never cross-slot)
**Source:** Phase 2 multi-slot `auth.json` + sampling resolve  
**Apply to:** `resolve_credentials` callers, BearerResolver wiring, fixture seeds  
```rust
// xai → PROVIDER_XAI / AuthManager xAI key
// codex → PROVIDER_CODEX / fixture or future OAuth — never xAI bearer
```

### Live bearer resolve preferred; static snapshot for fixtures
**Source:** `BearerResolver` + `shared_api_key_provider` comment (manager.rs 2316–2325)  
**Apply to:** reconstruct_full_config for xAI; Codex may snapshot fixture into `api_key` until Phase 5 refresh  
**Analog live resolve:**
```rust
// sampler client post() overrides Authorization when bearer_resolver returns Some
```

### Explicit model overrides win over provider defaults
**Source:** existing `resolve_credentials` own_credential arm + model `base_url`  
**Apply to:** pure route (override URL first) + credential priority (BYOK before session)

### Proxy-only derived headers
**Source:** `inject_url_derived_headers` + `is_cli_chat_proxy_url`  
**Apply to:** any SamplingConfig build path; unit-test Codex host negative case

### Observability without secrets
**Source:** sampler `client_new` / `client_post` logs (`has_api_key`, auth **prefix**)  
**Apply to:** new debug fields `provider`, `credential_slot`, `base_url` — never log raw tokens

### Model switch rebuilds next sample (do not re-plumb ACP)
**Source:** handlers + session `SetSessionModel`  
**Apply to:** fix builders only; Phase 6 owns UX gate for missing provider

### ShellAuthCredentialProvider stays xAI
**Source:** `credential_provider.rs` + RESEARCH discretion  
**Apply to:** do not redirect storage/upload auth when selecting Codex models for chat

---

## No Analog Found

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| — | — | — | **None.** All Phase 4 touch points are brownfield extensions of existing resolve/build/reconstruct seams. New pure types (`ProviderRoute`) and `tests/provider_routing.rs` copy adjacent patterns. |

**Closest “new shape” guidance:** RESEARCH Pattern 1–4; pure function style matches `blank_as_unset` / endpoint resolvers rather than a new crate.

---

## Metadata

**Analog search scope:**  
`crates/codegen/xai-grok-shell/src/agent/{config,models,mvp_agent,handlers}`,  
`session/acp_session_impl/{model_switch,sampler_turn}`,  
`auth/{model,storage,manager,credential_provider}`,  
`crates/codegen/xai-grok-sampler/src/{config,client}`,  
`crates/codegen/xai-grok-env/src/lib.rs`,  
`crates/codegen/xai-grok-shell/tests/{model_catalog,auth_multi_slot}.rs`

**Files scanned:** ~25 primary paths (targeted reads on multi-kLOC files via line ranges)  
**Pattern extraction date:** 2026-07-16  
**Planner note:** Wave order from RESEARCH — pure resolver + endpoints → credential/default_models → prepare/ModelsManager/reconstruct → tests/observability. Do not implement Phase 5 OAuth or Phase 6 login gate.
