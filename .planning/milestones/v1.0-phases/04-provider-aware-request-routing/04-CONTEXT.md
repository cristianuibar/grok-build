# Phase 4: Provider-aware request routing - Context

**Gathered:** 2026-07-16
**Status:** Ready for planning

<domain>
## Phase Boundary

Selecting a model must route **each sample/turn** to the correct backend with that provider’s credentials: Grok/xAI models → xAI / cli-chat-proxy + xAI credentials; GPT/Codex models → OpenAI/Codex (ChatGPT backend) + ChatGPT OAuth credentials — **not** the xAI proxy and **not** Platform API key semantics as the primary path (MOD-04, MOD-05).

**In scope:**
- Pure (or pure-core) model → provider → (base_url, credential_slot, api_backend) resolver
- Catalog `provider` field as authority (`xai` | `codex`; missing defaults to `xai` per Phase 3)
- Rebuild `SamplingConfig` (base URL + credential slot + backend) on **next sample** after model switch
- Explicit per-model `base_url` / `api_key` overrides still win over provider defaults
- Codex/ChatGPT base URL via built-in default + env override (mirror xAI endpoint pattern)
- Reuse existing OpenAI-compatible sampler path; set provider-correct base_url + bearer
- Provider-aware credential resolution: xAI slot for `xai`, Codex slot for `codex` — never cross-slot
- Prefer live bearer resolve per request (existing AuthManager / SharedApiKeyProvider-style seam)
- Missing Codex credentials: allow route construction for tests with injected fake tokens; no polished login gate UX (Phase 6)
- Automated tests with fake tokens proving base URL + Authorization differ by provider; switch changes next sample only
- Observability: structured fields already used (base_url etc.); no secrets in logs

**Out of scope (later phases):**
- Full ChatGPT/Codex OAuth login/logout/status/refresh UX → Phase 5
- Mid-session switch UX polish & missing-provider block + login prompt → Phase 6
- Cross-provider subagent spawn → Phase 7
- Quiet fork / rebrand polish → Phase 8
- Live dual-provider daily-driver e2e → Phase 9
- Platform API key as primary Codex product path
- Additional providers beyond xAI + Codex/OpenAI (PROV-V2-01)

</domain>

<decisions>
## Implementation Decisions

### Route resolution contract
- Prefer a **pure resolver**: selected model → catalog `provider` → `(base_url, credential_slot, api_backend)` used when building / rebuilding sampling config for a turn
- **Catalog `provider` field is authority** for routing (`xai` | `codex`); missing/unknown follows Phase 3 default to **`xai`**
- On mid-session model switch, **rebuild SamplingConfig for the next sample** (base URL + credential slot + backend) — do not require process restart; full switch UX / gate is Phase 6
- **Explicit per-model `base_url` / `api_key` overrides still win** over provider defaults when set (existing override chain preserved)

### Backend endpoints & API shape
- **Grok/xAI requests** continue on the existing **cli-chat-proxy / xAI path** via current endpoints — preserve stock Grok behavior
- **GPT/Codex requests** go to the **Codex/ChatGPT backend base URL** (OpenAI Responses / Codex-compatible), **not** cli-chat-proxy
- Codex base URL: **built-in default + env override** (mirror xAI endpoint pattern in `xai-grok-env`)
- Wire/API shape: **reuse existing OpenAI-compatible sampler path** (`api_backend` / Responses-style already in tree); set provider-correct base_url + bearer — do not invent a parallel client this phase

### Credential slot & auth for sampling
- Credential slot from **model’s provider binding**: `xai` → xAI slot, `codex` → Codex slot — **never** send xAI bearer to Codex or Codex bearer to xAI
- Prefer **live bearer resolve per request** (existing AuthManager / SharedApiKeyProvider-style seam) so refresh stays correct; construction-time static key only where already required
- **Missing Codex credentials this phase:** allow route construction for tests with **injected fake tokens**; do **not** implement polished missing-provider login gate UX (Phase 6). Fail closed if neither real nor test-injected credential exists for a live sample
- **API-key fallback:** keep xAI API-key fallback under `providers.xai` only; Codex path is **ChatGPT OAuth primary** (Platform API key not the product path). Optional Codex API-key only if needed for CI fixtures — not user-facing primary

### Verification & phase boundary
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

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- Phase 3: `ModelProvider` enum (`Xai` | `Codex`) on `ModelInfo` / catalog entries; missing deserializes to `Xai`
- `ModelEntry` / catalog resolve in `xai-grok-shell` `agent/config.rs` + `agent/models.rs` — id, name, base_url, api_backend, agent_type, provider, …
- `SamplingConfig` in `xai-grok-sampler` — `api_key`, `base_url`, `api_backend`, optional live bearer resolve per request
- Phase 2 multi-slot auth: `providers.xai` / `providers.codex` in `auth.json`; `ShellAuthCredentialProvider` / AuthManager bearer seams
- Endpoints: `xai-grok-env` production defaults + env overrides (cli-chat-proxy, xAI API base)

### Established Patterns
- Sampling config built from current model + auth state (`Models` / session path → `sampling_config()`)
- Bearer injection via credential provider rather than only construction-time static key
- Catalog `provider` is auth/routing binding; `agent_type` is harness type — do not conflate
- Per-model `api_base_url` / overrides already exist on entries; provider defaults should fill when unset

### Integration Points
- Turn sampling: shell session / MvpAgent builds `SamplingConfig` → sampler client/actor HTTP
- Model switch: next sample must pick up new model’s provider route (Phase 6 owns UX gate)
- Phase 5 will populate real Codex OAuth into `providers.codex`; routing must already select that slot
- Tests: fake tokens + assert base_url + Authorization differ by provider; no live ChatGPT required

</code_context>

<specifics>
## Specific Ideas

- Align with PROJECT: Grok → xAI/cli-chat-proxy + xAI tokens; GPT → OpenAI/Codex ChatGPT backend + ChatGPT OAuth — never the xAI proxy for GPT
- Success criteria demand switch changes **next sample’s** resolved base URL / credential slot (fake tokens OK)
- Keep single mixed picker semantics — routing is per selected model, not a session-global provider mode
- Phase 5/6 depend on this seam: resolve correctly even when Codex login UX is not finished

</specifics>

<deferred>
## Deferred Ideas

- Full ChatGPT/Codex OAuth login/logout/status/refresh lifecycle → Phase 5
- Mid-session switch UX + missing-provider block + login prompt → Phase 6
- Cross-provider subagent credential/backend isolation → Phase 7
- Quiet fork / telemetry/auto-update → Phase 8
- Live dual-provider daily-driver e2e → Phase 9
- Platform API key as primary Codex path
- Additional providers beyond xAI + Codex (PROV-V2-01)
- Richer per-provider capability matrix UI (MOD-V2-01)

</deferred>
