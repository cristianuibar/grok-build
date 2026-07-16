# Phase 3: Model catalog & GPT-5.6 entries - Context

**Gathered:** 2026-07-16
**Status:** Ready for planning

<domain>
## Phase Boundary

Deliver a **mixed, provider-labeled model catalog** so the model selector lists Grok/xAI models alongside GPT-5.6 family options in one list, with every entry carrying an explicit provider binding (`xai` | `codex`) usable by Phase 4 routing.

**In scope:**
- Ship GPT-5.6 family catalog entries (Sol / Terra / Luna) labeled as Codex/OpenAI
- Keep Grok/xAI models in the same mixed picker list
- Explicit `provider` binding on every catalog entry (schema + parse + surface)
- Provider-visible labels in selector / list output
- Automated tests proving mixed list + binding + no auth-based filtering of GPT rows

**Out of scope (later phases):**
- Provider-aware request routing / base URL + credential selection → Phase 4
- Codex/ChatGPT OAuth login/logout/status → Phase 5
- Mid-session switch behavior & missing-provider gate UX → Phase 6
- Cross-provider subagent orchestration → Phase 7
- Quiet fork / rebrand polish → Phase 8
- Live dual-provider daily-driver e2e → Phase 9
- Custom BYOK GPT entries as a product requirement this phase

</domain>

<decisions>
## Implementation Decisions

### GPT-5.6 catalog entries
- Ship **Sol, Terra, and Luna** as the GPT-5.6 family set (matches daily-driver / codex-cli usage)
- Prefer stable IDs **`gpt-5.6-sol`**, **`gpt-5.6-terra`**, **`gpt-5.6-luna`**; research may refine wire IDs if Codex/OpenAI differs — keep display names stable
- **Extend** embedded `default_models.json` (and any remote/default merge path already used) so GPT entries ship in the same catalog source of truth as Grok — not a separate hardcoded picker list
- GPT entries need: id/slug, display `name`, short `description`, **`provider: codex`**, reasonable `context_window`, and enough fields to appear in the selector; full Codex backend routing deferred to Phase 4

### Provider binding schema
- Add an **explicit `provider` field** on each catalog entry (`"xai"` | `"codex"`) — do **not** treat `agent_type` alone as the provider binding for routing later
- Canonical ids match Phase 2 auth slots: **`xai`** and **`codex`**
- All current Grok entries get **`provider: "xai"`** explicitly in the catalog
- Missing `provider` defaults to **`xai`** for backward compatibility with existing configs/tests, with a clear parse path; new ship defaults always set the field explicitly

### Mixed-list presentation
- **Single flat mixed list** — no session-global provider mode that filters the whole session (PROJECT non-negotiable)
- Show provider as a **label/badge or name suffix** (e.g. `GPT-5.6 Sol (Codex)` / provider meta) so users see who serves the model
- Default ordering: **Grok/xAI entries first**, then GPT-5.6 family (Sol → Terra → Luna unless research says otherwise)
- **Show GPT entries even if Codex is not logged in** this phase — missing-provider gate is Phase 6; catalog completeness is the Phase 3 goal

### Default model & phase boundary
- Keep **`grok-build` (xAI) as the default model** for the milestone default path
- Phase 3 is **catalog + provider binding + selector visibility only** — do not implement turn routing to Codex backends here
- Prove with **automated tests**: mixed list contains Grok + GPT entries; each entry has correct provider; no auth-based filtering of GPT rows; default remains Grok
- Do not require custom BYOK GPT entries for v1 success criteria this phase

### Claude's Discretion
- Exact serde field placement (`ModelInfo` vs `ModelEntry` vs config TOML mirror), ACP meta wire shape for provider label, and whether display is suffix vs badge column
- Whether GPT entries use placeholder/stub `base_url` values safe for list-only vs empty with defaults
- How deep to thread provider into remote model merge without breaking existing remote catalog overrides
- Snapshot/test harness style for picker vs shell list APIs
- Wire-id refinements during research if Codex catalog differs from `gpt-5.6-*` (preserve Sol/Terra/Luna family + display names)

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `crates/codegen/xai-grok-models/default_models.json` + `DEFAULT_MODELS_JSON` — embedded default catalog (currently Grok-centric, single `grok-build` entry)
- `ModelEntry` / `ModelEntryConfig` / `ModelInfo` in `xai-grok-shell` `agent/config.rs` — rich per-model config (id, name, description, base_url, api_backend, agent_type, context_window, hidden, supported_in_api, reasoning_efforts, …)
- `agent/models.rs` — catalog resolve, prefetched remote merge, visibility (`user_selectable`, `hidden`, `supported_in_api`)
- Pager: `models.rs` CLI list, `acp/model_state.rs`, settings modal picker, slash `/model`
- Phase 2 multi-slot auth: provider ids **`xai`** / **`codex`** already established in `auth.json` schema

### Established Patterns
- Catalog merge: CLI/env/config.toml/remote/defaults; `agent_type` is agent harness type (e.g. `grok-build`, `codex`), **not** auth provider
- Picker displays `name` when set; ACP `ModelInfo` carries id + name (+ meta for efforts)
- Defaults embedded at compile time via `include_str!` on `default_models.json`
- Auth slots are multi-provider; model catalog still assumes xAI-centric defaults

### Integration Points
- Default model resolution and list building in shell `agent/models.rs` / `config::resolve_model_*`
- ACP model list exposed to pager TUI and `bum models` / slash model command
- Phase 4 will consume `provider` binding to select credential slot + backend
- Phase 6 will gate selection when provider credentials are missing — catalog must still list GPT now

</code_context>

<specifics>
## Specific Ideas

- Align GPT family with existing codex-cli skill IDs: `gpt-5.6-sol`, `gpt-5.6-terra`, `gpt-5.6-luna`
- Provider label should be obvious in the mixed list without requiring a separate “provider mode”
- Default stays on Grok so day-to-day xAI users are not surprised after catalog expansion
- “Reserved” Codex auth slot (Phase 2) pairs with catalog entries that bind to `codex` before OAuth ships

</specifics>

<deferred>
## Deferred Ideas

- Provider-aware request routing (base URL + credentials per provider) → Phase 4
- Codex/ChatGPT OAuth lifecycle → Phase 5
- Missing-provider block + login prompt on model switch → Phase 6
- Cross-provider subagent spawn using catalog models → Phase 7
- Custom BYOK / user-defined GPT models as first-class v1 requirement
- Richer capability matrix UI and reasoning-effort first-class TUI settings beyond catalog fields (MOD-V2-01)
- Additional providers beyond xAI + Codex/OpenAI (PROV-V2-01)

</deferred>
