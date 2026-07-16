# Architecture Research

**Domain:** Multi-provider AI coding-agent CLI (bum on Grok Build fork)
**Researched:** 2026-07-16
**Confidence:** HIGH

## Standard Architecture

### System Overview

v1 does **not** rewrite the harness. It inserts a thin **provider plane** between the existing session/sampler path and external IdPs/APIs. Agent tools, workspace, ACP, and the Action→Effect TUI loop stay as today.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│  Composition root: `xai-grok-pager-bin` → binary `bum`                        │
│  Home: `~/.bum` (BUM_HOME / rebranded GROK_HOME default)                      │
└───────────────────────────────┬─────────────────────────────────────────────┘
                                │
          ┌─────────────────────┼─────────────────────┐
          ▼                     ▼                     ▼
┌──────────────────┐  ┌────────────────────┐  ┌────────────────────┐
│ TUI (Pager)      │  │ Agent runtimes     │  │ Leader process     │
│ model picker +   │  │ stdio/headless/ACP │  │ Unix-socket IPC    │
│ login UX         │  │                    │  │                    │
└────────┬─────────┘  └─────────┬──────────┘  └─────────┬──────────┘
         │                      │                        │
         └──────────────────────┼────────────────────────┘
                                ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  Agent runtime (`xai-grok-shell`)                                            │
│  MvpAgent · session · tools · MCP · hooks                                    │
│                                                                              │
│  ┌──────────────────── PROVIDER PLANE (v1 addition) ─────────────────────┐  │
│  │ ProviderRegistry │ CredentialHub │ ModelCatalog │ RequestRouter        │  │
│  │  xai | openai    │ multi-slot    │ model→prov.  │ model→SamplerConfig  │  │
│  │  (+ future)      │ auth managers │ + static GPT │ + bearer per provid. │  │
│  └────────┬─────────────────┬───────────────┬───────────────┬────────────┘  │
│           │                 │               │               │               │
│  ┌────────▼──────┐  ┌───────▼──────┐  ┌─────▼─────┐  ┌──────▼──────────┐  │
│  │ ModelsManager │  │ Auth (shell) │  │ chat-state│  │ sampler turn    │  │
│  │ + provider    │  │ multi-scope  │  │ sampling  │  │ reconstruct_    │  │
│  │ field         │  │ store        │  │ config    │  │ full_config     │  │
│  └───────────────┘  └──────┬───────┘  └───────────┘  └──────┬──────────┘  │
└────────────────────────────┼────────────────────────────────┼─────────────┘
                             │                                │
              ┌──────────────┴──────────────┐    ┌────────────▼────────────┐
              ▼                             ▼    ▼                         │
     ~/.bum/auth.json              xai-grok-sampler                        │
     (scoped credentials)          SamplingClient + BearerResolver         │
              │                             │                              │
     ┌────────┴────────┐           ┌────────┴────────┐                     │
     ▼                 ▼           ▼                 ▼                     │
 auth.x.ai      ChatGPT/Codex   cli-chat-proxy    OpenAI/Codex API         │
 (xAI OAuth)    OAuth (PKCE)    + xAI tokens      + Codex tokens           │
```

### Component Responsibilities

| Component | Responsibility | Typical Implementation | Monorepo touch points |
|-----------|----------------|------------------------|------------------------|
| **ProviderRegistry** | Known providers (v1: `xai`, `openai_codex`); static metadata (issuer, default base URL, login capability) | Small enum + const table; no plugin loading in v1 | New module under `xai-grok-shell/src/auth/provider.rs` (or leaf `xai-grok-providers` only if shared) |
| **CredentialHub** | Multi-provider credential ownership: load/store, proactive refresh, 401 recovery **per provider** | Façade over N `AuthManager`-like slots sharing one `auth.json` file lock | Extend `shell/src/auth/` (`manager.rs`, `storage.rs`, `model.rs`); keep `xai-grok-auth` traits |
| **Provider login flows** | Interactive + device-code OAuth per provider | xAI: existing `oidc/` + `device_code.rs`; Codex: new ChatGPT OAuth (browser/PKCE, optional device-code) | `shell/src/auth/flow.rs`, `oidc/`, new `auth/codex/` or `auth/openai/`; CLI `login` in `pager-bin` / `pager/cli.rs` |
| **ModelCatalog** | Unified picker list: Grok (proxy/remote) + GPT-5.6 family (bundled/static); each entry carries `provider` | Extend `ModelInfo` / `ModelEntry` + merge in `ModelsManager` | `shell/src/agent/models.rs`, `agent/config.rs` (`ModelInfo`), `xai-grok-models/default_models.json` |
| **RequestRouter** | `model_id` → `ModelEntry` → base URL, API backend, credential slot, URL-derived headers | Pure function path on existing `sampling_config_for_model` / `resolve_credentials` | `agent/config.rs` (`resolve_credentials`, `sampling_config_for_model`, `inject_url_derived_headers`); `session/acp_session_impl/sampler_turn.rs` |
| **Provider-aware BearerResolver** | Per-request bearer from the **correct** provider manager (not a single global token) | `Arc<dyn BearerResolver>` that keys off current model’s provider | `xai-grok-sampler` trait unchanged; new resolver in shell (`AuthManagerBearerResolver` → multi) |
| **Missing-provider gate** | Block model select / first sample if provider has no usable credential; prompt that provider’s login | Gate in `set_model` / session set_model path + TUI effect | `ModelsManager::set_current_model_id`, ACP `session/set_model`, pager dispatch/effects for login modal |
| **Product home / branding** | Isolate identity under `~/.bum`; binary `bum`; no xAI update/telemetry phone-home | Path default + env rename; feature flags / no-ops on update & Mixpanel | `xai-grok-config` paths, `shell/util/grok_home`, `xai-grok-update`, `xai-grok-telemetry`, pager-bin |

## Recommended Project Structure

v1 prefers **in-place extension** of existing crates over a parallel rewrite tree.

```
crates/codegen/
├── xai-grok-pager-bin/          # Composition root; ship binary name `bum`
├── xai-grok-pager/              # TUI: multi-provider picker labels, login effects
│   └── src/app/{dispatch,effects,cli}.rs
├── xai-grok-shell/              # Primary integration surface for provider plane
│   └── src/
│       ├── auth/
│       │   ├── provider.rs      # NEW: ProviderId, ProviderMeta, CredentialHub
│       │   ├── model.rs         # AuthStore scopes; extend for openai_codex scopes
│       │   ├── manager.rs       # Keep; hub owns one manager (or logical slot) per provider
│       │   ├── storage.rs       # Shared auth.json lock; multi-key map (already BTreeMap)
│       │   ├── oidc/ + device_code.rs   # xAI (existing)
│       │   ├── openai/          # NEW: ChatGPT/Codex OAuth + refresher
│       │   ├── flow.rs          # login CLI: `bum login xai` / `bum login openai`
│       │   └── credential_provider.rs   # ShellAuthCredentialProvider → hub-aware
│       ├── agent/
│       │   ├── models.rs        # Catalog merge + auth gate on set_current_model_id
│       │   └── config.rs        # ModelInfo.provider; resolve_credentials by provider
│       └── session/acp_session_impl/
│           └── sampler_turn.rs  # reconstruct_full_config: provider-scoped bearer
├── xai-grok-models/
│   └── default_models.json      # Grok + GPT-5.6 entries with provider + base_url
├── xai-grok-sampler/            # Prefer zero/low change: already URL-agnostic
├── xai-grok-auth/               # Keep AuthCredentialProvider / HttpAuth traits
├── xai-grok-config/             # Default home ~/.bum; BUM_HOME / GROK_HOME alias
└── xai-grok-update|telemetry/   # Disable phone-home for bum product builds
```

### Structure Rationale

- **Provider plane lives in shell auth + agent config:** That is where OAuth, catalog, and sampling config already meet. A new top-level crate is optional later; v1 should not pay for monorepo cycle thrash.
- **Sampler stays URL-agnostic:** `SamplerConfig` already carries `base_url`, `api_backend`, `auth_scheme`, `extra_headers`, and `bearer_resolver`. Routing belongs **above** the sampler (shell), not inside HTTP stream parsers.
- **Auth store remains one file with multi-key map:** `AuthStore = BTreeMap<String, GrokAuth>` already scopes by issuer/API-key key. Add stable keys for Codex (e.g. `openai::chatgpt` / issuer URL) rather than inventing a second file format.
- **Models carry provider:** `ModelInfo` already has per-model `base_url` and `api_backend`. Adding `provider: ProviderId` makes routing explicit and enables the missing-provider gate without URL sniffing.

## Architectural Patterns

### Pattern 1: Provider as credential + endpoint boundary

**What:** A provider is the unit of (1) OAuth/API identity, (2) default inference base URL, (3) catalog source. Models never “float” without a provider; credentials never cross providers.

**When to use:** Always for multi-provider agent CLIs. Matches industry “connection / connected-account” OAuth modeling and Claude Code’s compile-time provider set.

**Trade-offs:** Clear isolation and fail-closed gates; slightly more bookkeeping than a single global session token.

**Example (conceptual):**

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ProviderId {
    Xai,
    OpenaiCodex,
}

pub struct ProviderMeta {
    pub id: ProviderId,
    pub display_name: &'static str,
    /// Stable auth.json scope key
    pub auth_scope: &'static str,
    pub default_base_url: &'static str,
}

// ModelInfo gains:
// provider: ProviderId  // required for routing + login gate
```

### Pattern 2: Per-model routing, not session-global “mode”

**What:** The active model id alone decides which provider credentials and base URL are used. Session history is shared; transport is not.

**When to use:** Product requirement: mixed picker, switch anytime (PROJECT.md). Avoids “Codex mode” / “Grok mode” session forks.

**Trade-offs:** Conversation may mix models with different tool/API quirks; document provider gaps instead of forking session storage in v1.

**Existing seam:** `ModelsManager::set_current_model_id` + session rebuild of sampling config (`reconstruct_full_config` in `sampler_turn.rs`).

### Pattern 3: Multi-slot credential hub over single AuthManager API

**What:** Keep the battle-tested `AuthManager` (file lock, refresh chain, sleep gate, proactive refresh) but own **one slot per provider**. A thin `CredentialHub` resolves `ProviderId → Arc<AuthManager>` (or one manager with multi-scope active map — prefer **one manager per provider** if scope/refresher coupling is tight).

**When to use:** Dual OAuth with independent refresh tokens (xAI vs ChatGPT). Critical: refresh-token reuse bugs are provider-local; do not serialize all providers behind one in-memory “current” bearer only.

**Trade-offs:** Two proactive refresh loops; still one on-disk `auth.json` with shared flock (already designed for multi-scope map).

**Codex/community warning:** Access/refresh tokens must persist after refresh; failed disk write → token-family revocation loops. Reuse existing `write_auth_json` atomic/fallback path.

### Pattern 4: Catalog merge (remote xAI + static OpenAI)

**What:** xAI catalog continues via cli-chat-proxy `/v1/models` + cache. OpenAI/GPT models ship from `default_models.json` (and/or config.toml) with `provider = openai_codex` and OpenAI base URL. Unified picker = merge + sort/group.

**When to use:** OpenAI does not expose the same entitlement-gated proxy catalog; static GPT-5.6 family is the v1 source of truth (IDs confirmed at implement time against current OpenAI/Codex availability).

**Trade-offs:** GPT list is not live-entitlement-filtered; missing-provider gate + runtime 401/403 UX cover entitlement failures.

### Pattern 5: Missing-provider gate (fail closed)

**What:** Before accepting a model switch or starting a turn, require `CredentialHub::has_usable(provider)`. On failure, return a typed error the TUI maps to “Login to OpenAI / xAI” rather than a mid-stream 401.

**When to use:** Always for OAuth-backed models. Aligns with PROJECT.md “no silent failure mid-turn.”

**Trade-offs:** Extra check on hot path (cheap disk/memory snapshot). Do not auto-open browser without user action in headless mode — surface CLI `bum login openai` instead.

## Data Flow

### Login flow (provider-scoped)

```
User: `bum login openai` | TUI login action
    ↓
CLI/TUI → shell auth::flow (provider argument)
    ↓
Provider login strategy
  · xai  → existing OIDC/device-code against auth.x.ai
  · openai_codex → ChatGPT OAuth (browser loopback / device-code if needed)
    ↓
Tokens written to ~/.bum/auth.json under provider scope key
    ↓
CredentialHub hot-swaps that provider’s AuthManager
    ↓
Optional: refresh model catalog (xAI only); GPT static list already present
    ↓
TUI/CLI shows provider as authenticated
```

**Codex reference shape (external, do not share path):** stock CLI caches ChatGPT OAuth under `~/.codex/auth.json` (`tokens.access_token`, `refresh_token`, `account_id`) with auto-refresh. bum **copies the pattern**, stores under `~/.bum`, never reads/writes `~/.codex` in v1.

### Model switch flow

```
User selects model in picker / ACP session/set_model
    ↓
ModelsManager resolves ModelEntry (id → provider, base_url, api_backend, …)
    ↓
Missing-provider gate:
  if !CredentialHub.has_usable(entry.provider) → typed error → login prompt
    ↓
set_current_model_id + model_switch_watch bump
    ↓
Session rebuilds SamplingConfig via sampling_config_for_model + provider bearer
    ↓
Next turn uses new transport; conversation history unchanged
```

### Chat turn / sampling flow

```
ACP session/prompt
    ↓
ChatStateActor records user message
    ↓
sampler_turn::reconstruct_full_config
    · read chat-state SamplingConfig (model, base_url, api_backend, …)
    · resolve provider from ModelCatalog
    · attach BearerResolver bound to that provider’s AuthManager
    · inject_url_derived_headers only for cli-chat-proxy URLs (xAI path)
    ↓
SamplerActor / SamplingClient POST stream
  · Grok  → cli-chat-proxy (or configured proxy) + xAI bearer + X-XAI-Token-Auth
  · GPT   → OpenAI/Codex API base + ChatGPT OAuth bearer (no xAI proxy headers)
    ↓
Stream events → ACP notifications → TUI
    · 401 → provider-scoped refresh_after_unauthorized → single retry
    · permanent auth fail → surface re-login for that provider only
```

### State management

| State | Owner | Multi-provider note |
|-------|--------|---------------------|
| TUI AppView / AgentView | pager dispatch | Show multi-provider auth badges; login modals |
| Current model id | `ModelsManager` | Must map to provider for gates |
| Conversation | `ChatStateActor` | Shared across providers |
| Credentials | `CredentialHub` + `~/.bum/auth.json` | Per-provider scopes; shared file lock |
| Sampler concurrent requests | `SamplerActor` | Config is per-request; mid-flight switch should not mutate in-flight auth mid-stream (existing cancel/switch patterns) |

### Key Data Flows (summary)

1. **Login:** User → flow → provider OAuth → scoped `auth.json` → hub.
2. **Catalog:** defaults JSON (+ remote xAI fetch) → `ModelsManager` → picker ACP models.
3. **Route:** model id → entry.provider + base_url → credentials + headers → `SamplerConfig`.
4. **Turn:** session → reconstruct_full_config → sampler stream → tools → continue.

## Scaling Considerations

This is a **local single-user CLI**, not a multi-tenant gateway. Scale concerns differ:

| Scale | Architecture adjustments |
|-------|--------------------------|
| 1 developer, 2 providers (v1) | CredentialHub + static GPT catalog; no plugin providers |
| More providers later | Add `ProviderId` variants + login strategy trait; keep sampler URL-agnostic |
| Multi-account per provider | Deferred (PROJECT out of scope); would need account picker + multi-scope keys |
| Shared team policy | Existing managed config / signed policy remain xAI-centric; do not block OpenAI path on xAI team policy |

### Scaling Priorities

1. **First bottleneck:** Wrong-token routing (xAI bearer on OpenAI URL or vice versa) — fix with explicit provider on model + hub-scoped resolver.
2. **Second bottleneck:** Refresh-token races across processes — keep existing `auth.json.lock` and per-scope refresh semantics.
3. **Third bottleneck:** Provider-specific API/tool gaps (web search via proxy, etc.) — document and fall back; do not dual-implement every tool in v1.

## Anti-Patterns

### Anti-Pattern 1: Global “current provider” mode

**What people do:** One session flag `provider = openai` that filters the whole picker and auth.

**Why it's wrong:** Breaks mixed Grok/GPT workflow; forces re-login mental model; fights existing per-model `base_url` design.

**Do this instead:** Per-model `provider` field; route every sample from the active model entry.

### Anti-Pattern 2: Single AuthManager “current bearer” for all backends

**What people do:** Keep one in-memory token; swap on model switch.

**Why it's wrong:** Loses the other provider’s proactive refresh; racey under concurrent subagent/aux sampling; 401 recovery refreshes the wrong IdP.

**Do this instead:** Multi-slot hub; `BearerResolver` closes over the provider required by the request’s model.

### Anti-Pattern 3: Sniffing provider from base_url alone

**What people do:** `if url.contains("openai") { … }`.

**Why it's wrong:** Custom proxies, enterprise gateways, and test doubles break; headers like `X-XAI-Token-Auth` must only attach for true cli-chat-proxy (already special-cased in `inject_url_derived_headers`).

**Do this instead:** Explicit `ProviderId` on catalog entries; URL helpers remain for header injection only where already correct.

### Anti-Pattern 4: Reading stock `~/.codex` or `~/.grok`

**What people do:** Import credentials for convenience.

**Why it's wrong:** Violates isolated `~/.bum` product identity; couples refresh/logout to foreign CLIs; PROJECT.md explicitly out of scopes sharing in v1.

**Do this instead:** Full re-login into `~/.bum/auth.json`.

### Anti-Pattern 5: Implementing multi-provider inside the sampler crate

**What people do:** Hard-code OpenAI vs xAI branches in `SamplingClient`.

**Why it's wrong:** Sampler is deliberately URL-agnostic; branches belong in shell config assembly. Sampler already supports Responses/ChatCompletions/Messages backends.

**Do this instead:** Build correct `SamplerConfig` in shell; leave stream parsers generic.

### Anti-Pattern 6: Auto-login browser on every missing credential in headless/CI

**What people do:** Open OAuth whenever a sample fails.

**Why it's wrong:** Breaks non-interactive agents; surprising UX.

**Do this instead:** Typed `ProviderAuthRequired { provider }` error; interactive TUI offers login; headless prints `bum login <provider>`.

## Integration Points

### External Services

| Service | Integration Pattern | Notes |
|---------|---------------------|-------|
| xAI OAuth (`auth.x.ai`) | Existing device/browser OIDC in `shell/src/auth` | Preserve; store under bum home scopes |
| cli-chat-proxy (`cli-chat-proxy.grok.com/v1`) | Bearer + `X-XAI-Token-Auth` via `inject_url_derived_headers` | Grok models only |
| ChatGPT / Codex OAuth | Browser PKCE (+ device-code option) | Mirror Codex CLI behavior; isolate credentials in `~/.bum` |
| OpenAI/Codex inference API | OpenAI-compatible HTTP via existing `async-openai` / sampler streams | Prefer Responses/ChatCompletions already in sampler; confirm GPT-5.6 model ids at implement time |
| xAI auto-update / Mixpanel / Sentry | Disable or no-op for bum product identity | Quiet local fork requirement |

### Internal Boundaries

| Boundary | Communication | Notes |
|----------|---------------|-------|
| Pager ↔ Shell | ACP JSON-RPC | Model list, set_model, auth status notifications |
| Shell auth ↔ Sampler | `SamplerConfig` + `BearerResolver` | No auth crate dependency from sampler |
| ModelsManager ↔ CredentialHub | Provider gate on set_model | Do not fetch OpenAI catalog via xAI proxy |
| Session ↔ Tools | Unchanged registry | Provider gaps documented; tools stay workspace-local |
| Config paths ↔ All | `grok_home()` / `BUM_HOME` | Default `~/.bum`; env override for tests |

### Monorepo touch map (implementation checklist)

| Concern | Primary files/crates |
|---------|----------------------|
| Home rename | `xai-grok-config/src/paths.rs`, shell `util/grok_home`, env docs |
| Auth store multi-provider | `shell/src/auth/{model,storage,manager,flow}.rs` |
| Codex OAuth | new `shell/src/auth/openai/*` + refresher in `auth/refresh/` |
| Model provider field + GPT list | `agent/config.rs` `ModelInfo`, `xai-grok-models/default_models.json` |
| Catalog merge | `agent/models.rs` |
| Credential resolve | `agent/config.rs` `resolve_credentials`, `sampling_config_for_model` |
| Turn-time bearer | `session/acp_session_impl/sampler_turn.rs` |
| Login CLI | `pager/src/app/cli.rs`, `pager-bin` Command routing |
| TUI gate UX | `pager` dispatch/effects + auth views |
| Quiet fork | `xai-grok-update`, `xai-grok-telemetry`, Mixpanel init sites |
| Binary name | `xai-grok-pager-bin` `[[bin]]` / install scripts → `bum` |

## Suggested Build Order

Dependencies flow **identity → credentials → catalog → routing → UX → quiet/rebrand polish**. Do not start with TUI chrome before routing is testable headless.

### Phase A — Product identity & home (foundation)

1. Default home `~/.bum` (`BUM_HOME` / dual-read `GROK_HOME` only if needed for tests — prefer clean cut).
2. Binary rename path to `bum` (composition root wiring).
3. Smoke: config, sessions, auth path resolve under new home.

**Unblocks:** All credential and session persistence.

### Phase B — Provider types & multi-slot credential store

1. Introduce `ProviderId` + auth scope keys.
2. CredentialHub / multi-scope load: xAI continues to work unchanged as `ProviderId::Xai`.
3. Keep single-provider behavior green (regression).

**Unblocks:** Second OAuth without rewriting sampler.

### Phase C — Model catalog provider tagging + static GPT models

1. Add `provider` (and OpenAI `base_url`) to `ModelInfo` / defaults JSON.
2. Merge static GPT-5.6 family into `ModelsManager` available list.
3. Picker can list both; selection may still fail sample until D/E.

**Unblocks:** Explicit routing keys.

### Phase D — Request routing & provider-scoped bearer

1. `resolve_credentials` / `sampling_config_for_model` branch on `provider`.
2. `reconstruct_full_config` attaches hub resolver for that provider.
3. Ensure `inject_url_derived_headers` never tags OpenAI URLs with xAI proxy headers.
4. Unit tests: Grok config vs GPT config golden headers/base_url/auth.

**Unblocks:** Correct traffic once Codex tokens exist.

### Phase E — Codex / ChatGPT OAuth login

1. Implement login + refresh + logout for `openai_codex`.
2. CLI: `bum login openai` / `bum logout openai` (and keep xAI).
3. Persist tokens in `~/.bum/auth.json` with existing lock writers.
4. Headless-friendly status command.

**Unblocks:** Real GPT-5.6 turns.

### Phase F — Missing-provider gate + TUI/CLI UX

1. Gate on `session/set_model` and pre-turn.
2. TUI: prompt provider login; show dual auth status.
3. Headless: clear error + exit code / ACP error object.

**Unblocks:** Daily-driver safety (fail closed).

### Phase G — Quiet fork + rebrand polish

1. Disable auto-update channel and product telemetry phone-home.
2. Remaining UI strings / docs chrome as `bum`.
3. End-to-end daily-driver validation: dual login, switch mid-session, tools both paths.

**Unblocks:** v1 ship criteria.

### Dependency graph

```
A home/binary
 └── B provider + multi-slot auth
      └── C catalog + provider on models
           └── D routing + bearer
                ├── E Codex OAuth  (can overlap late C/D once scopes exist)
                └── F gate + UX (needs D + E for full path)
                     └── G quiet + polish
```

**Parallelism:** A is serial first. B then unlocks C and E in parallel once scope keys exist; D needs C; F needs D+E; G can start mid-F for update/telemetry kill-switches.

## Sources

- Local codebase maps: `.planning/codebase/ARCHITECTURE.md`, `STRUCTURE.md`, `INTEGRATIONS.md` (2026-07-16)
- In-tree auth/sampler seams: `xai-grok-shell/src/auth/*`, `agent/models.rs`, `agent/config.rs` (`ModelInfo`, `resolve_credentials`, `sampling_config_for_model`), `session/acp_session_impl/sampler_turn.rs`, `xai-grok-sampler` (`SamplerConfig`, `BearerResolver`)
- Project requirements: `.planning/PROJECT.md` (multi-provider, `~/.bum`, fail-closed gate)
- Codex auth product docs: ChatGPT sign-in, `~/.codex/auth.json` cache, device-code/headless patterns (OpenAI Codex auth documentation)
- Multi-provider agent patterns: provider as compile-time set + model routing layer (Claude Code / Hermes-style analyses); OAuth connection/account separation for long-lived agent tokens
- Community Codex OAuth pitfalls: refresh must persist to disk; do not share/copy tokens across machines carelessly (token-family revocation)

---
*Architecture research for: multi-provider OAuth + model routing inside Grok Build → bum*
*Researched: 2026-07-16*
