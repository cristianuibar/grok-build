# Phase 4: Provider-aware request routing - Research

**Researched:** 2026-07-16
**Domain:** Brownfield multi-provider sampling route resolution (base URL + credential slot + api_backend) on existing Rust sampler/shell
**Confidence:** HIGH

## Summary

Phase 4 is a **routing seam** change, not a new client or OAuth stack. Catalog already tags models with `ModelProvider` (`xai` | `codex`) from Phase 3, and multi-slot `auth.json` already has `providers.xai` / `providers.codex` from Phase 2. What is missing: every path that builds `SamplerConfig` still treats inference like a single xAI/cli-chat-proxy world — GPT rows inherit `endpoints.resolve_inference_base_url()` (cli-chat-proxy) and credentials from `AuthManager::current_or_expired()` (xAI in-memory only). [VERIFIED: codebase]

The locked contract is a **pure resolver**: selected model’s catalog `provider` → `(base_url, credential_slot, api_backend)`, with per-model `base_url` / `api_key` overrides still winning. Grok stays on cli-chat-proxy/xAI; GPT goes to the ChatGPT/Codex backend base (`https://chatgpt.com/backend-api/codex` by default + env override), **never** the xAI proxy, using the Codex credential slot / injected fixture bearer. Wire shape reuses the existing OpenAI-compatible Responses sampler (`api_backend: responses` already on GPT catalog rows). [VERIFIED: codebase + STACK research]

**Critical implementation risks:** (1) `default_models()` stamps **all** bundled entries with proxy `base_url` and `api_base_url = xai_api_base_url`, including `provider=codex`. (2) `prepare_sampling_config_for_model` and `ModelsManager::sampling_config` call `resolve_credentials` with the xAI session key only. (3) `reconstruct_full_config` may re-apply an **xAI** `AuthManagerBearerResolver` on the next sample and overwrite a Codex bearer if the session-token gate is active. (4) `inject_url_derived_headers` must **not** add `X-XAI-Token-Auth` for Codex bases. [VERIFIED: codebase]

**Primary recommendation:** Add Codex base URL to `EndpointsConfig` (default + `GROK_CODEX_BASE_URL` env, mirror existing endpoint pattern); implement a pure `resolve_provider_route` / provider-aware `resolve_credentials` used by `prepare_sampling_config_for_model`, `ModelsManager::sampling_config`, and reconstruct/bearer seams; fix default catalog URL stamping by provider; prove with unit tests + fake tokens (no live ChatGPT, no Phase 5 OAuth, no Phase 6 login gate UX).

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
#### Route resolution contract
- Prefer a **pure resolver**: selected model → catalog `provider` → `(base_url, credential_slot, api_backend)` used when building / rebuilding sampling config for a turn
- **Catalog `provider` field is authority** for routing (`xai` | `codex`); missing/unknown follows Phase 3 default to **`xai`**
- On mid-session model switch, **rebuild SamplingConfig for the next sample** (base URL + credential slot + backend) — do not require process restart; full switch UX / gate is Phase 6
- **Explicit per-model `base_url` / `api_key` overrides still win** over provider defaults when set (existing override chain preserved)

#### Backend endpoints & API shape
- **Grok/xAI requests** continue on the existing **cli-chat-proxy / xAI path** via current endpoints — preserve stock Grok behavior
- **GPT/Codex requests** go to the **Codex/ChatGPT backend base URL** (OpenAI Responses / Codex-compatible), **not** cli-chat-proxy
- Codex base URL: **built-in default + env override** (mirror xAI endpoint pattern in `xai-grok-env`)
- Wire/API shape: **reuse existing OpenAI-compatible sampler path** (`api_backend` / Responses-style already in tree); set provider-correct base_url + bearer — do not invent a parallel client this phase

#### Credential slot & auth for sampling
- Credential slot from **model’s provider binding**: `xai` → xAI slot, `codex` → Codex slot — **never** send xAI bearer to Codex or Codex bearer to xAI
- Prefer **live bearer resolve per request** (existing AuthManager / SharedApiKeyProvider-style seam) so refresh stays correct; construction-time static key only where already required
- **Missing Codex credentials this phase:** allow route construction for tests with **injected fake tokens**; do **not** implement polished missing-provider login gate UX (Phase 6). Fail closed if neither real nor test-injected credential exists for a live sample
- **API-key fallback:** keep xAI API-key fallback under `providers.xai` only; Codex path is **ChatGPT OAuth primary** (Platform API key not the product path). Optional Codex API-key only if needed for CI fixtures — not user-facing primary

#### Verification & phase boundary
- Prove MOD-04/MOD-05 with **automated tests using fake tokens** — assert resolved base URL + Authorization credential slot/provider differ for xAI vs Codex models; assert model switch changes next sample’s resolved route
- **Do not** implement full Codex OAuth login/logout/status/refresh (Phase 5) or missing-provider switch gate UX (Phase 6) here — only the routing seam those phases will call
- Codex credentials in unit/integration tests: **inject fake / fixture bearer** (and optional mock base URL); no live ChatGPT login required to pass Phase 4
- Observability: **reuse structured sampling fields** already used (`base_url`, etc.); ensure provider/slot is visible in debug where useful; **never log raw secrets**

### Claude's Discretion
- Exact module placement for the pure resolver (shell models vs small shared helper)
- Concrete Codex/ChatGPT default base URL string and env var name (mirror existing `xai-grok-env` style)
- How `ShellAuthCredentialProvider` (or equivalent) selects slot from active model provider without breaking xAI-only sessions
- Whether `api_backend` defaults need per-provider values or stay model-entry driven
- Test layout (shell vs sampler) and mock HTTP vs config-only assertions

### Deferred Ideas (OUT OF SCOPE)
- Full ChatGPT/Codex OAuth login/logout/status/refresh lifecycle → Phase 5
- Mid-session switch UX + missing-provider block + login prompt → Phase 6
- Cross-provider subagent credential/backend isolation → Phase 7
- Quiet fork / telemetry/auto-update → Phase 8
- Live dual-provider daily-driver e2e → Phase 9
- Platform API key as primary Codex path
- Additional providers beyond xAI + Codex (PROV-V2-01)
- Richer per-provider capability matrix UI (MOD-V2-01)
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| MOD-04 | Requests for Grok/xAI models use the xAI / cli-chat-proxy path with xAI credentials | Keep `provider=xai` → `proxy_url()` / existing dual endpoint; credentials from xAI slot / `AuthManager::current_or_expired`; `inject_url_derived_headers` only for cli-chat-proxy hosts |
| MOD-05 | Requests for GPT/Codex models use the OpenAI/Codex (ChatGPT backend) path with ChatGPT OAuth credentials — not Platform API key semantics and not the xAI proxy | `provider=codex` → Codex base URL (not proxy); credentials from `providers.codex` / injected fixture bearer; no xAI bearer cross-slot; Responses `api_backend` already on catalog rows |
</phase_requirements>

## Project Constraints (from AGENTS.md / PROJECT.md)

| Directive | Implication for Phase 4 |
|-----------|-------------------------|
| Stay on Rust workspace (edition 2024, Tokio, existing harness) | Branch `SamplerConfig` only — no new sampler crate / agent rewrite [VERIFIED: AGENTS.md] |
| Routing: per-model provider selection, not global mode | Resolver keys off catalog `provider` of the **selected** model [VERIFIED: PROJECT.md] |
| Identity: ChatGPT OAuth primary for Codex (not Platform API key) | Default Codex base is ChatGPT backend, not `api.openai.com/v1` [VERIFIED: PROJECT.md + STACK] |
| Grok → xAI/proxy; GPT → OpenAI/Codex with Codex credentials | Explicit dual path in route table [VERIFIED: PROJECT.md] |
| Root `Cargo.toml` is generated — edit per-crate manifests | No new workspace dep expected [VERIFIED: AGENTS.md] |
| Prefer per-crate tests (`cargo test -p xai-grok-shell`) | Follow Phase 3 `tests/model_catalog.rs` style; add routing-focused tests [VERIFIED: codebase] |
| Never log secrets | Log `base_url`, provider, slot id, key **prefix** only [VERIFIED: conventions] |
| clippy: use `dunce::canonicalize` not std canonicalize | N/A for pure routing; keep when touching FS tests [VERIFIED: clippy.toml] |

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Catalog `provider` authority | Shell agent config (`ModelInfo` / `ModelEntry`) | `default_models.json` | Phase 3 SoT; missing → `xai` |
| Pure route resolve `(base_url, slot, api_backend)` | Shell agent config (pure fn) | EndpointsConfig | Keep pure for unit tests; endpoints supply defaults |
| Codex / xAI endpoint defaults + env | Shell `EndpointsConfig` (+ optional `xai-grok-env` constant) | — | Mirror `proxy_url` / `GROK_XAI_API_BASE_URL` pattern |
| Credential slot selection | Shell auth + prepare_sampling path | Auth storage `providers.*` | Never cross-slot; Phase 5 fills real Codex OAuth |
| Live bearer per request | Sampler `BearerResolver` + shell reconstruct | AuthManager | Existing seam; must become provider-aware |
| Build `SamplerConfig` | `prepare_sampling_config_for_model` / `sampling_config_for_model` | `ModelsManager::sampling_config` | Model switch + startup both call these |
| Persist route for next sample | Session `handle_set_session_model` → chat-state `SamplingConfig` + `Credentials` | `reconstruct_full_config` | Switch already rebuilds; must carry correct base_url/key |
| HTTP wire (Responses stream) | `xai-grok-sampler` client | — | Unchanged; only base_url + Authorization change |
| Missing-provider UX gate | **Out of scope** (Phase 6) | — | Fail closed for live sample without creds; no polished login prompt |
| Full Codex OAuth lifecycle | **Out of scope** (Phase 5) | — | Routing must accept fixture / future slot |

## Standard Stack

### Core

| Library / component | Version / location | Purpose | Why Standard |
|---------------------|-------------------|---------|--------------|
| Rust workspace | edition 2024, toolchain 1.92.0 | Entire product | PROJECT non-negotiable |
| `xai-grok-shell` agent config/models | `agent/config.rs`, `agent/models.rs`, `mvp_agent/agent_ops.rs` | Route resolve + SamplingConfig build | Existing sampling construction |
| `xai-grok-sampler` | `SamplerConfig`, `BearerResolver`, `client.rs` | HTTP streaming client | Reuse OpenAI-compatible path |
| `xai-grok-sampling-types` | `SamplingConfig`, `ApiBackend` | Chat-state lightweight config | Model switch updates this |
| `xai-grok-shell` auth multi-slot | `auth/model.rs` `PROVIDER_XAI`/`PROVIDER_CODEX`, `auth/storage.rs` | Credential slots | Phase 2 store |
| `AuthManager` | `auth/manager.rs` | Live xAI bearer (+ future multi-principal) | Existing session token source |
| `EndpointsConfig` | `agent/config.rs` | Proxy + xAI API bases; extend for Codex | Existing env/default pattern |
| `default_models.json` | `xai-grok-models` | Catalog with `provider` + `api_backend` | Phase 3 complete |

### Supporting

| Library / component | Version | Purpose | When to Use |
|---------------------|---------|---------|-------------|
| `serial_test` + `xai-grok-test-support` | workspace | Env-isolated tests, mock HTTP | Routing + optional mock server |
| `reqwest` / rustls | workspace | Already used by sampler | No new HTTP stack |
| `async-openai` 0.33 | workspace | Responses wire types | Already in sampler |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| ChatGPT backend `…/backend-api/codex` | `api.openai.com/v1` + ChatGPT OAuth | **Rejected** — wrong product surface; OAuth tokens ≠ Platform keys [CITED: STACK.md / openai codex] |
| Parallel Codex client crate | Reuse sampler | **Rejected by CONTEXT** |
| Infer provider from model id / `agent_type` | Catalog `provider` | **Rejected** — Phase 3 explicit binding |
| Global “provider mode” | Per-model route | **Rejected by PROJECT** |
| Full dual AuthManager now | Minimal slot read + fixture inject | OAuth lifecycle is Phase 5; routing only needs slot selection + tests |

**Installation:** none — no new crates.

```bash
# Focused verification
cargo test -p xai-grok-shell --test provider_routing   # planned new test target (or config unit tests)
cargo test -p xai-grok-shell --lib resolve_  # if unit tests live in config.rs
cargo check -p xai-grok-shell -p xai-grok-sampler
```

**Version verification:** host `rustc 1.92.0`, `cargo 1.92.0` — matches `rust-toolchain.toml`. No new packages. [VERIFIED: environment]

## Package Legitimacy Audit

> Phase installs **no** external packages.

| Package | Registry | Age | Downloads | Source Repo | Verdict | Disposition |
|---------|----------|-----|-----------|-------------|---------|-------------|
| — | — | — | — | — | — | N/A — no installs |

**Packages removed due to [SLOP] verdict:** none  
**Packages flagged as suspicious [SUS]:** none

## Architecture Patterns

### System Architecture Diagram

```text
  model picker / set_session_model(model_id)
              │
              ▼
     catalog ModelEntry
        provider: xai | codex   ◄── authority (Phase 3)
        optional base_url / api_key overrides
              │
              ▼
  ┌─────────────────────────────────────────┐
  │  pure resolve_provider_route(provider,  │
  │    endpoints, model overrides)          │
  │  → base_url, credential_slot, backend   │
  └─────────────────────────────────────────┘
              │
      ┌───────┴────────┐
      ▼                ▼
 provider=xai      provider=codex
 base=cli-chat-    base=chatgpt.com/
   proxy (or         backend-api/codex
   models_base /     (+ env override)
   override)
 slot=providers.xai  slot=providers.codex
 inject X-XAI-*      NO xAI proxy headers
   if proxy URL      optional ChatGPT-Account-ID
      │                │
      └───────┬────────┘
              ▼
   ResolvedCredentials + SamplerConfig
   (api_key snapshot + optional BearerResolver)
              │
              ▼
   SessionActor handle_set_session_model
   → chat-state SamplingConfig.base_url/model/api_backend
   → chat-state Credentials.api_key
              │
              ▼  next sample / turn
   reconstruct_full_config()
   → SamplingClient::new / post()
   → POST {base_url}/responses
      Authorization: Bearer <slot-correct token>
```

### Recommended Project Structure (touch points only)

```text
crates/codegen/
├── xai-grok-shell/src/
│   ├── agent/
│   │   ├── config.rs          # EndpointsConfig codex field; pure route; resolve_credentials
│   │   ├── models.rs          # ModelsManager::sampling_config uses provider-aware resolve
│   │   ├── handlers/model_switch.rs  # already calls prepare_sampling_config_for_model
│   │   └── mvp_agent/agent_ops.rs    # prepare_sampling_config_for_model (PRIMARY)
│   ├── session/acp_session_impl/
│   │   ├── model_switch.rs    # writes SamplingConfig + Credentials
│   │   └── sampler_turn.rs    # reconstruct_full_config + BearerResolver (MUST be slot-aware)
│   └── auth/
│       ├── model.rs           # PROVIDER_XAI / PROVIDER_CODEX
│       ├── manager.rs         # optional: read fixture codex key / future multi-principal
│       └── storage.rs         # read providers.codex for fixture / Phase 5 prep
├── xai-grok-sampler/src/
│   ├── config.rs              # SamplerConfig / BearerResolver (reuse)
│   └── client.rs              # endpoint(base_url, "responses") — no change required
└── xai-grok-models/default_models.json  # already provider-tagged; URL stamping is code-side
```

### Pattern 1: Pure provider route resolver

**What:** Pure function mapping `ModelProvider` + endpoints + optional model overrides → route triple.  
**When to use:** Every place that currently assumes `info.base_url` is already correct for the provider.  
**Example (prescriptive shape):**

```rust
// Placement recommendation: agent/config.rs next to resolve_credentials
// [ASSUMED] exact names discretionary

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
    if let Some(url) = model_base_url_override.filter(|s| !s.trim().is_empty()) {
        return ProviderRoute {
            base_url: url.to_owned(),
            credential_slot: provider.as_str(), // still slot by provider
            provider,
        };
    }
    match provider {
        ModelProvider::Xai => ProviderRoute {
            base_url: endpoints.resolve_inference_base_url(),
            credential_slot: crate::auth::model::PROVIDER_XAI,
            provider,
        },
        ModelProvider::Codex => ProviderRoute {
            base_url: endpoints.resolve_codex_base_url(),
            credential_slot: crate::auth::model::PROVIDER_CODEX,
            provider,
        },
    }
}
```

**Override rule (locked):** model-entry explicit `base_url` / `api_key` / `env_key` (BYOK) wins over provider defaults; provider still selects credential **slot** when using session/OAuth credentials (never use xAI session for Codex BYOK-absent path).

### Pattern 2: Provider-aware credential resolve

**What:** Extend `resolve_credentials` (or a sibling) so session key is **slot-selected**, not always xAI.  
**When to use:** `prepare_sampling_config_for_model`, `ModelsManager::sampling_config`, aux model helpers that share the chat credential path.

Priority (preserve existing, add provider awareness):

1. Model own credential (`api_key` / `env_key`) → keep model `base_url` / BYOK semantics  
2. Else provider session token for **that** provider’s slot  
3. Else xAI-only: `XAI_API_KEY` / `api_base_url` fallback (**not** for Codex product path)  
4. Else empty key + provider-default base_url (tests may inject; live sample fail-closed)

### Pattern 3: Model switch already rebuilds next sample — fix the builder

**What:** `handlers/model_switch.rs` → `prepare_sampling_config_for_model` → `SessionCommand::SetSessionModel` → `handle_set_session_model` updates chat-state `base_url` + `api_key`.  
**When to use:** Do **not** invent a parallel switch path. Fix prepare/resolve so switch automatically routes correctly. [VERIFIED: codebase `model_switch.rs` L115–199, `model_switch.rs` session L48–76]

### Pattern 4: BearerResolver must not cross-slot

**What:** Today `AuthManagerBearerResolver` always reads `AuthManager::current_or_expired()` (xAI). If left on for Codex models, the next sample can **replace** Codex Authorization with an xAI bearer.  
**When to use:** Wire resolver only for the active model’s provider; or implement `ProviderScopedBearerResolver { provider, auth_manager }` that returns Codex fixture/slot key for `codex` and xAI for `xai`. Prefer live resolve when slot has a token; static `api_key` when fixture-injected for tests. [VERIFIED: `sampler_turn.rs` L245–347]

### Anti-Patterns to Avoid

- **Sending GPT traffic to cli-chat-proxy:** current default after Phase 3 — broken product path  
- **Using `api.openai.com/v1` with ChatGPT OAuth tokens as primary:** Platform ≠ subscription [CITED: STACK.md]  
- **Conflating `agent_type` with provider:** harness vs auth slot (Phase 3 locked)  
- **Cross-slot bearer:** xAI token on Codex base or reverse  
- **Adding `X-XAI-Token-Auth` to Codex hosts:** `inject_url_derived_headers` is URL-based — keep it; ensure Codex base is not classified as cli-chat-proxy  
- **Implementing Phase 5 OAuth or Phase 6 login modal here**  
- **Logging full bearer tokens** — use prefixes / has_* flags only (sampler already logs prefixes carefully)

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| HTTP streaming / Responses | New client crate | `xai-grok-sampler` + `api_backend: Responses` | Full retry/SSE/doom-loop already present |
| Auth storage format | New auth.json schema | Phase 2 `providers.xai` / `providers.codex` | Isolation already proven |
| Model switch plumbing | New ACP command | Existing `set_session_model` / `prepare_sampling_config_for_model` | Already rebuilds SamplingConfig |
| URL-derived proxy headers | Sampler URL sniffing | Shell `inject_url_derived_headers` | Sampler is intentionally URL-agnostic |
| Codex OAuth PKCE | Login UI / token exchange | Phase 5 | Out of scope; inject fakes for tests |
| Endpoint defaults | Hardcode only in catalog JSON | `EndpointsConfig` + env | Custom/enterprise overrides + testability |

**Key insight:** The product already has the **right architecture** (per-model `SamplerConfig` + bearer resolver). Phase 4 only makes those fields **provider-correct**.

## Concrete Integration Points (file:line)

| # | Location | Role in Phase 4 |
|---|----------|-----------------|
| 1 | `agent/config.rs` ~46–48 | `CLI_CHAT_PROXY_BASE_URL_DEFAULT`, `XAI_API_BASE_URL_DEFAULT` — add Codex default constant |
| 2 | `agent/config.rs` ~140–249, 273–314, 540–574 | `EndpointsConfig` + `proxy_url` / `resolve_inference_base_url` / `Default` — add `codex_base_url` + `resolve_codex_base_url()` |
| 3 | `agent/config.rs` ~3432–3470 | `default_models()` — **stop** stamping Codex rows with proxy + `xai_api_base_url` |
| 4 | `agent/config.rs` ~3368–3393 | `ModelProvider` — authority; use `as_str()` for slots |
| 5 | `agent/config.rs` ~4377–4421 | `resolve_credentials` — provider-aware session key + base_url |
| 6 | `agent/config.rs` ~4689–4737 | `sampling_config_for_model` — consumes credentials.base_url |
| 7 | `agent/config.rs` ~4753–4769 | `inject_url_derived_headers` — proxy-only; verify Codex URL not matched |
| 8 | `agent/config.rs` ~5568–5593 | `default_models_dual_endpoint_routing` test — **will break** if still asserts Codex → proxy; update by provider |
| 9 | `agent/models.rs` ~929–962 | `ModelsManager::sampling_config` — uses resolve_credentials + session xAI only today |
| 10 | `mvp_agent/agent_ops.rs` ~1084–1165 | **`prepare_sampling_config_for_model`** — primary model-switch builder |
| 11 | `handlers/model_switch.rs` ~115–199 | Calls prepare → `SetSessionModel` |
| 12 | `session/.../model_switch.rs` ~48–76 | Writes chat-state SamplingConfig + Credentials from SamplerConfig |
| 13 | `session/.../sampler_turn.rs` ~231–353 | `reconstruct_full_config` + **xAI-only** BearerResolver |
| 14 | `sampler/config.rs` ~49–102, 165–170 | `SamplerConfig`, `BearerResolver` trait |
| 15 | `sampler/client.rs` ~506–520, 549–567, ~773–777 | Logs base_url; live Authorization override; `endpoint(base, "responses")` |
| 16 | `auth/model.rs` ~16–20 | `PROVIDER_XAI`, `PROVIDER_CODEX` |
| 17 | `auth/manager.rs` ~752–754, 2298–2329 | `current_or_expired` (xAI), `shared_api_key_provider` |
| 18 | `auth/storage.rs` | Multi-slot read/write; fixture helpers for codex seed |
| 19 | `xai-grok-models/default_models.json` | GPT rows: `provider: codex`, `api_backend: responses` — no base_url field |
| 20 | `xai-grok-env/src/lib.rs` | Production proxy defaults pattern to mirror for Codex constant (optional dual home) |

## Discretion Recommendations (planner defaults)

| Discretion item | Recommendation | Confidence |
|-----------------|----------------|------------|
| Module placement | Pure resolver + endpoint helper in `agent/config.rs` next to `resolve_credentials` (same crate as `ModelProvider` / endpoints). Avoid new crate. | HIGH |
| Codex default base URL | `https://chatgpt.com/backend-api/codex` | HIGH [CITED: `.planning/research/STACK.md`; openai/codex ChatGPT path] |
| Env var name | `GROK_CODEX_BASE_URL` (mirror `GROK_XAI_API_BASE_URL` / `GROK_CLI_CHAT_PROXY_BASE_URL`; keep `GROK_` operational prefix until Phase 8). Optional compiled constant in `xai-grok-env` only if shell already re-exports env constants the same way. | MEDIUM (name discretionary; pattern HIGH) |
| `api_backend` | Stay **model-entry driven** (catalog already `responses` for Grok + GPT). Resolver may pass through; do not invent per-provider backend table this phase. | HIGH |
| ShellAuthCredentialProvider | Leave proxy/upload storage clients on **xAI** AuthManager (they talk to xAI services). Sampling path is the one that must select Codex slot — do **not** break xAI-only sessions by making storage provider-global. | HIGH |
| Live bearer for Codex this phase | Prefer: snapshot fixture key into `SamplerConfig.api_key` + optional resolver that reads **Codex slot from disk/memory** when present. Full multi-principal AuthManager refresh = Phase 5. For unit tests, inject fake token without AuthManager. | HIGH |
| ChatGPT-Account-ID | Optional: if fixture `GrokAuth.organization_id` (or dedicated field later) is set, inject `ChatGPT-Account-ID` via `extra_headers`. Not required to pass Phase 4 success criteria (base_url + slot). Full claims mapping = Phase 5. | MEDIUM |
| Test layout | Prefer `crates/codegen/xai-grok-shell/tests/provider_routing.rs` (integration style like `model_catalog.rs`) **plus** pure unit tests in `config.rs` for `resolve_provider_route`. Avoid full e2e binary tests this phase. | HIGH |

## Common Pitfalls

### Pitfall 1: Codex models still point at cli-chat-proxy
**What goes wrong:** GPT samples hit xAI proxy with wrong/missing auth.  
**Why:** `default_models()` sets every entry’s `base_url = resolve_inference_base_url()` and `api_base_url = xai_api_base_url`.  
**How to avoid:** Branch default stamping on `provider`; Codex → `resolve_codex_base_url()`, `api_base_url: None` (or Codex-only if ever needed).  
**Warning signs:** `default_models_dual_endpoint_routing` still green for GPT→proxy.

### Pitfall 2: Cross-slot bearer via reconstruct_full_config
**What goes wrong:** After switch to GPT, next sample Authorization is still xAI.  
**Why:** BearerResolver ignores model provider.  
**How to avoid:** Provider-scoped resolver; or disable resolver for Codex and rely on credentials set at switch until Phase 5 multi-principal.  
**Warning signs:** Tests that only check `handle_set_session_model` api_key but not reconstruct path.

### Pitfall 3: Per-model override suppressed incorrectly
**What goes wrong:** BYOK users lose custom base_url when provider defaults applied last.  
**Why:** Wrong merge order.  
**How to avoid:** Explicit model base_url/api_key **before** provider defaults (locked).  
**Warning signs:** Existing BYOK / env_key tests fail.

### Pitfall 4: Proxy headers on Codex host
**What goes wrong:** Spurious `X-XAI-Token-Auth` / client-mode headers confuse ChatGPT backend.  
**Why:** Misclassifying Codex URL as cli-chat-proxy.  
**How to avoid:** Keep `is_cli_chat_proxy_url` strict; unit-test inject headers for both bases.  
**Warning signs:** Headers map contains `X-XAI-Token-Auth` for `chatgpt.com` base.

### Pitfall 5: Using Platform API base as “Codex”
**What goes wrong:** 401 / wrong billing with ChatGPT OAuth tokens.  
**Why:** Conflating OpenAI Platform with ChatGPT backend.  
**How to avoid:** Default only `https://chatgpt.com/backend-api/codex`; document Platform as non-primary (deferred).  

### Pitfall 6: Scope creep into OAuth / gate UX
**What goes wrong:** Phase 4 blocks on browser login work.  
**Why:** Tempting to “finish” dual auth.  
**How to avoid:** Fixture tokens only; Phase 5/6 deferred explicitly.  

### Pitfall 7: Aux paths (web_search, image_description) silently stay on xAI
**What goes wrong:** Not MOD-04/05 blockers, but inconsistency if aux uses main model’s credentials without provider.  
**How to avoid:** This phase: document that **chat/sample path** is in scope; leave aux on existing xAI/default model helpers unless they call the same prepare path for a Codex model id. Do not block phase on full aux matrix. [ASSUMED: aux out of MOD-04/05 primary success criteria]

## Code Examples

### Existing: sampling config construction (today — provider-blind)

```rust
// Source: crates/codegen/xai-grok-shell/src/agent/mvp_agent/agent_ops.rs ~1084–1163
// session key is always AuthManager::current_or_expired() (xAI)
let mut credentials = resolve_credentials(
    model,
    session.as_ref().map(|a| a.key.as_str()),
);
let mut config = crate::agent::config::sampling_config_for_model(
    model,
    credentials,
    alpha_test_key,
    client_version,
    deployment_id,
    user_id,
);
```

### Existing: model switch rebuilds SamplingConfig fields

```rust
// Source: crates/codegen/xai-grok-shell/src/session/acp_session_impl/model_switch.rs ~48–76
self.chat_state_handle
    .update_sampling_config(xai_grok_sampling_types::SamplingConfig {
        base_url: sampling_config.base_url.clone(),
        model: sampling_config.model.clone(),
        // ...
        api_backend: sampling_config.api_backend.clone(),
        // ...
    });
self.chat_state_handle
    .update_credentials(xai_chat_state::Credentials {
        api_key: sampling_config.api_key.clone(),
        // ...
    });
```

### Existing: sampler client uses base_url + live bearer

```rust
// Source: crates/codegen/xai-grok-sampler/src/client.rs ~549–567, ~773–777
fn post(&self, url: impl reqwest::IntoUrl) -> reqwest::RequestBuilder {
    // if bearer_resolver present → overrides Authorization
}
fn endpoint(&self, path: &str) -> String {
    format!("{base}/{path}") // base_url + "responses"
}
```

### Prescriptive test sketch (fake tokens)

```rust
// tests/provider_routing.rs (recommended)
#[test]
fn xai_model_routes_to_proxy_with_xai_token() {
    let endpoints = EndpointsConfig::default();
    let entry = /* ModelEntry provider=Xai, no overrides */;
    let creds = resolve_credentials_for_provider(&entry, Some("xai-fake"), None, &endpoints);
    assert!(creds.base_url.contains("cli-chat-proxy") || creds.base_url == endpoints.proxy_url());
    assert_eq!(creds.api_key.as_deref(), Some("xai-fake"));
}

#[test]
fn codex_model_routes_to_codex_backend_with_codex_token() {
    let endpoints = EndpointsConfig::default();
    let entry = /* ModelEntry provider=Codex, no overrides */;
    let creds = resolve_credentials_for_provider(&entry, None, Some("codex-fake"), &endpoints);
    assert_eq!(creds.base_url, endpoints.resolve_codex_base_url());
    assert!(!creds.base_url.contains("cli-chat-proxy"));
    assert_eq!(creds.api_key.as_deref(), Some("codex-fake"));
}

#[test]
fn model_override_base_url_wins() {
    // entry with explicit base_url "https://byok.example/v1"
    // still never uses the other provider's token when session tokens provided
}

#[test]
fn switch_changes_next_sample_route() {
    // prepare_sampling_config_for_model(grok) vs prepare(... gpt-5.6-sol)
    // assert base_url + api_key differ
}
```

## Recommended Task Decomposition (for planner)

| Wave | Plan focus | Outcomes |
|------|------------|----------|
| **0** | Test scaffold | `tests/provider_routing.rs` (or config unit module) with failing expectations for Codex route + dual credentials |
| **1** | Endpoints + pure resolver | `CODEX_BASE_URL_DEFAULT`, `EndpointsConfig::resolve_codex_base_url`, `resolve_provider_route`, unit tests + env override |
| **2** | Credential resolve + default_models stamping | Provider-aware `resolve_credentials` / prepare path; fix `default_models()` Codex URLs; update dual-endpoint tests |
| **3** | Wire prepare + ModelsManager + reconstruct | `prepare_sampling_config_for_model`, `sampling_config()`, BearerResolver slot-aware; switch tests |
| **4** | Observability + fail-closed | Debug fields `provider`/`credential_slot`; live sample without key fails closed (no polished UX); no secret logs |

Granularity note: project `granularity: fine` — planner may split Wave 2–3 into smaller PLAN.md files but should keep pure resolver before session wiring.

## State of the Art

| Old Approach (pre-Phase 4) | Current Approach (this phase) | When | Impact |
|----------------------------|-------------------------------|------|--------|
| Single inference base (cli-chat-proxy / models_base) | Dual bases by catalog provider | Phase 4 | GPT leaves xAI proxy |
| Session key = xAI only | Slot selected by model provider | Phase 4 | Cross-provider switch safe |
| Catalog provider labels only | Provider drives HTTP route | Phase 3 → 4 | Completes MOD-04/05 |
| ChatGPT OAuth | Deferred Phase 5 | later | Routing accepts fixtures first |

**Deprecated/outdated for this product path:**
- Treating ChatGPT OAuth tokens as Platform API keys on `api.openai.com/v1` as the **primary** GPT path [CITED: STACK.md]

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Aux tool sampling (web_search defaults, etc.) need not fully multi-provider for MOD-04/05 | Pitfalls | Planner may under-specify if product expects aux on Codex mid-session |
| A2 | Env name `GROK_CODEX_BASE_URL` is preferred over `BUM_CODEX_*` this phase | Discretion | User may want BUM_ prefix already — rename is cheap |
| A3 | `ChatGPT-Account-ID` not required for automated Phase 4 proofs | Discretion | Live Phase 9 may need header before OAuth polish if tests go live early |
| A4 | AuthManager remains single-principal in-memory for xAI; Codex may be disk-slot / fixture until Phase 5 | Architecture | If planner assumes full dual AuthManager now, scope explodes |

**Verified claims (not assumptions):** integration file:lines, catalog provider field, current resolve_credentials behavior, SamplerConfig fields, STACK Codex base URL as research SoT for this fork.

## Open Questions

1. **Should `xai-grok-env` grow a Codex constant, or only shell `EndpointsConfig`?**  
   - What we know: proxy defaults live in both env crate and shell constants.  
   - Recommendation: shell `EndpointsConfig` is sufficient for Phase 4; optional thin constant in env crate if other crates need it later.

2. **In-memory multi-principal AuthManager now or Phase 5?**  
   - What we know: CONTEXT wants live bearer resolve preferred; Phase 5 owns OAuth lifecycle.  
   - Recommendation: minimal slot read API + fixture inject for Codex; keep xAI AuthManager as today; design trait so Phase 5 drops in refresh without rewriting routing.

3. **Fail-closed UX surface when Codex sample has no key?**  
   - What we know: no polished gate (Phase 6); fail closed required for live sample.  
   - Recommendation: return existing auth/sampling error path (401/missing key) without new TUI modal.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| rustc / cargo | build/test | ✓ | 1.92.0 | — |
| Node (gsd-tools only) | planning tooling | ✓ | v22.22.1 | — |
| Live ChatGPT / Codex network | Phase 4 proofs | ✗ not required | — | Fake tokens + unit assertions |
| Docker | — | n/a | — | — |

**Missing dependencies with no fallback:** none for Phase 4.  
**Step 2.6:** external runtime deps limited to Rust toolchain (available).

## Validation Architecture

> `workflow.nyquist_validation` is enabled (true) in `.planning/config.json`.

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Built-in `cargo test` (Rust) |
| Config file | per-crate; shell uses `tests/*.rs` + `#[cfg(test)]` in modules |
| Quick run command | `cargo test -p xai-grok-shell --test provider_routing -q` (or lib filter) |
| Full suite command | `cargo test -p xai-grok-shell --test provider_routing --test model_catalog` + targeted lib tests |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| MOD-04 | xAI model → proxy/xAI base + xAI credential slot | unit | `cargo test -p xai-grok-shell --test provider_routing xai_route -q` | ❌ Wave 0 |
| MOD-05 | Codex model → Codex base + Codex credential (fake) | unit | `cargo test -p xai-grok-shell --test provider_routing codex_route -q` | ❌ Wave 0 |
| MOD-04/05 | Explicit model base_url override wins | unit | `… override_wins` | ❌ Wave 0 |
| Switch SC-3 | prepare_sampling for model A vs B changes base_url + key | unit | `… switch_changes_route` | ❌ Wave 0 |
| Safety | No X-XAI headers on Codex base | unit | `… no_proxy_headers_on_codex` | ❌ Wave 0 |
| Safety | Never cross-slot tokens | unit | `… never_cross_slot` | ❌ Wave 0 |
| Regression | Existing dual-endpoint xAI rows still correct | unit | existing `default_models_dual_endpoint_routing` (updated) | ✅ needs edit |

### Sampling Rate

- **Per task commit:** targeted `cargo test -p xai-grok-shell` filter for touched tests  
- **Per wave merge:** full `provider_routing` + `model_catalog` + config dual-endpoint  
- **Phase gate:** green before `/gsd:verify-work`; no live ChatGPT required  

### Wave 0 Gaps

- [ ] `crates/codegen/xai-grok-shell/tests/provider_routing.rs` — MOD-04/MOD-05 route + switch proofs  
- [ ] Update `default_models_dual_endpoint_routing` expectations for Codex entries  
- [ ] Optional: unit tests colocated in `config.rs` for pure `resolve_provider_route`  
- [ ] Framework install: none  

## Security Domain

> `security_enforcement` enabled; ASVS level 1.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|------------------|
| V2 Authentication | partial | Multi-slot credentials; no new OAuth this phase; fixture tokens in tests only |
| V3 Session Management | partial | Live bearer refresh remains xAI-first until Phase 5; do not leak tokens across slots |
| V4 Access Control | yes | Credential **slot isolation** — never send wrong provider’s bearer |
| V5 Input Validation | yes | Provider enum serde; URL from config/env; model id catalog resolve |
| V6 Cryptography | no new | Reuse existing TLS (rustls); no hand-rolled crypto |

### Known Threat Patterns for multi-provider routing

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Cross-provider credential use | Elevation / Info disclosure | Slot selected strictly from `ModelProvider`; tests assert non-cross |
| Secret logging | Info disclosure | Structured logs: `base_url`, `has_api_key`, auth **prefix** only |
| SSRF via custom base_url | Tampering | Existing BYOK allows custom URLs; no new open-proxy — keep current policy |
| Token injection via test fixtures | Spoofing (test-only) | Fixtures only under test temp `BUM_HOME`; never ship fake tokens |
| Proxy header leakage to third party | Tampering | `inject_url_derived_headers` only on cli-chat-proxy URL class |

## Sources

### Primary (HIGH confidence)

- Codebase: `agent/config.rs`, `models.rs`, `mvp_agent/agent_ops.rs`, `handlers/model_switch.rs`, `session/.../model_switch.rs`, `sampler_turn.rs`, `sampler/{config,client}.rs`, `auth/{model,manager,storage}.rs`, `default_models.json` — verified this session  
- Phase 2/3 CONTEXT + RESEARCH — multi-slot store + catalog provider field  
- `.planning/research/STACK.md` — ChatGPT Codex base URL + header contract (prior research vs openai/codex)

### Secondary (MEDIUM confidence)

- `.planning/research/SUMMARY.md` — dual-path sampling architecture recommendation  
- Local `~/.codex` presence — confirms product uses ChatGPT backend ecosystem (not used as runtime import)

### Tertiary (LOW confidence)

- Exact production requirement for `ChatGPT-Account-ID` on every request without multi-workspace — defer full enforcement to Phase 5 if fixtures omit it  

## Metadata

**Confidence breakdown:**
- Standard stack: **HIGH** — brownfield; no new packages  
- Architecture: **HIGH** — integration points line-verified  
- Pitfalls: **HIGH** — cross-slot + default_models stamping confirmed  
- Codex default URL string: **HIGH** for fork research SoT / **MEDIUM** until Phase 5 live traffic  

**Research date:** 2026-07-16  
**Valid until:** ~30 days (stable brownfield; Codex backend URL rarely changes)

## Runtime State Inventory

> Not a rename/refactor/migration phase — **omitted**. No product string rename in scope.
