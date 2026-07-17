# Phase 7: Cross-provider multi-agent orchestration - Context

**Gathered:** 2026-07-17
**Status:** Ready for planning

<domain>
## Phase Boundary

Parent session on one provider can spawn a child subagent on another with correct **model**, **reasoning effort**, **credentials**, and **backend routing**. Existing same-provider subagent spawn/resume/roles/personas must not regress. Missing child-provider login fails closed with a clear login prompt. Natural-language orchestration works when the parent uses the Task tool with explicit model + effort (e.g. Grok parent → Codex Sol medium-effort research child).

This phase does **not** include: custom agentic workflow engine (later milestone), fleet cost dashboards (AGENT-V2), richer first-class TUI effort chrome beyond spawn args (MOD-V2-01), or full daily-driver E2E polish (Phase 9). Quiet fork/rebrand strings are Phase 8 unless wrong paths/CLI names block spawn.

</domain>

<decisions>
## Implementation Decisions

### Spawn contract (model + effort)
- **Omit `Task.model` → inherit parent model/provider** (existing same-provider default). Explicit catalog slug required only when user/parent wants a different model (including cross-provider).
- **Validate explicit model via existing `TaskModelValidator` / live catalog** — unknown slug rejects before spawn; valid GPT entries (e.g. `gpt-5.6-sol`) are allowed even when parent is xAI (and vice versa). Do not restrict child model to parent’s provider.
- **Expose `reasoning_effort` (or equivalent) on the model-facing Task tool schema** and wire it through `SubagentRuntimeOverrides.reasoning_effort` (field already exists; Task currently hardcodes `None`). Omit effort → inherit child’s model default / parent effort behavior already used by the harness.
- **Effort vocabulary:** reuse existing product tokens (`low` / `medium` / `high` / `xhigh` and any catalog-supported aliases). Invalid effort rejects with a clear tool error listing accepted values — no silent clamp.

### Cross-provider credential gate
- **Check usable credentials for the child’s resolved provider at spawn time** (before background launch), analogous to Phase 6 switch-time gate: token present and not permanently unusable; expired-but-refreshable is OK (refresh on first child sample).
- **Fail closed — never fall back to parent bearer or parent backend** when child provider is missing/unusable. Do not rewrite parent credentials into the child slot.
- **Error surface:** typed tool/ACP error naming the missing provider (xAI / Codex) plus explicit CLI hint `bum login --provider {xai|codex}`. No silent 401 as primary UX for spawn.
- **Same-provider spawns** (child model same provider as parent, parent already usable): **no extra gate friction**. Gate only when target child model’s provider lacks usable creds (including cross-provider and same-provider-if-somehow-missing).

### Child routing isolation
- **Child session builds SamplingConfig from the child’s model → catalog `provider` → base_url + credential slot + api_backend** using Phase 4 pure resolver — not parent’s fixed route.
- **Child turns resolve bearer from the child’s provider slot only** (live AuthManager / SharedApiKeyProvider-style per request). Parent and child may run concurrent turns with different providers without clobbering each other.
- **Parent session model/provider is unchanged by spawn** — spawning a Codex child does not switch the parent’s current model.
- **Prove with automated tests + dual fake tokens:** assert child request Authorization + base URL match child provider; parent continues on its own slot; missing child slot fails spawn with login-shaped error (no wrong-backend request).

### NL orchestration & phase boundary
- **Enable NL orchestration via Task tool contract + schema/docs** so the parent model can call `task` with `model` + `reasoning_effort` when the user asks (e.g. “start a Codex Sol medium-effort subagent to research X”). Prefer light schema/description improvements over a new workflow engine.
- **Same-provider regression is a hard gate (AGENT-01):** spawn/resume/roles/personas/parent↔child routing keep working when parent and child share a provider; cross-provider is additive.
- **`resume_from` keeps existing pin:** resume inherits source child’s model (ignore model override on resume — already soft-ignored). Document that cross-provider resume stays on the prior child model.
- **Out of scope this phase:** custom agentic workflows, multi-account cost dashboards, Phase 8 rebrand polish, Phase 9 full daily-driver matrix (except automated proofs needed for AGENT-01..06).

### Claude's Discretion
- Exact module placement for spawn-time provider gate (Task tool pre-check vs shell `handle_subagent_request` — prefer single authoritative check so tool and harness paths cannot diverge)
- How deep to thread effort into child SamplingConfig rebuild vs session-level override
- Test layout (tools unit vs shell integration) and fixture patterns for dual-slot spawn
- Whether to extend goal-orchestrator / harness-internal spawns with effort in this phase or only model-facing Task path
- Copy strings for missing-provider spawn errors (align with Phase 6 provider labels)

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `SubagentRequest` + `SubagentRuntimeOverrides` already carry `model` and `reasoning_effort` (`crates/codegen/xai-grok-tools/src/implementations/grok_build/task/types.rs`)
- `Task` tool accepts optional `model`; validates via `TaskModelValidator`; currently sets `reasoning_effort: None` on spawn (`task/mod.rs`)
- Phase 4 provider-aware routing: model → catalog `provider` → base_url / credential slot / backend
- Phase 5 dual-slot AuthManager (`providers.xai` / `providers.codex`) with independent refresh
- Phase 6 missing-provider gate at model switch (`model_switch::apply`, usable-token checks, deferred switch + login hint)
- Subagent backend channel: `SubagentBackend` / shell `handle_subagent_request` / session actor spawn

### Established Patterns
- Catalog `provider` field is authority for routing (`xai` | `codex`); missing defaults to `xai`
- Fail closed on missing credentials — never cross-send bearers
- Tool errors prefer actionable messages (valid slug lists, login CLI hints)
- Resume soft-ignores model override; source model pinned

### Integration Points
- `Task` tool → `SubagentBackend::spawn` → shell subagent coordinator → child session SamplingConfig
- Auth: multi-slot store under `$BUM_HOME/auth.json`; live bearer resolve per request
- Model catalog: mixed Grok + GPT-5.6 entries with provider labels
- ACP/headless: typed errors for machine clients; TUI toast/modal patterns from Phase 6 for human-facing gates where applicable

</code_context>

<specifics>
## Specific Ideas

- Success story: main on Grok, user says “start a Codex Sol medium-effort subagent to research X” → child on `gpt-5.6-sol` at medium effort returns results to parent.
- Cross-provider directions both ways: Grok parent → Codex child and Codex parent → Grok child (OPS-06 related; prove routing/auth isolation for both).
- Prefer extending existing Task + subagent plumbing over inventing a parallel multi-agent API.

</specifics>

<deferred>
## Deferred Ideas

- Custom agentic workflow engine / workflow product surface — later milestone (WF-V2-01)
- Cross-provider multi-account / cost attribution dashboards — AGENT-V2-01
- Richer per-provider capability matrix UI and first-class TUI effort settings beyond spawn args — MOD-V2-01
- Full quiet rebrand of strings/chrome — Phase 8
- Daily-driver end-to-end validation matrix — Phase 9

</deferred>
