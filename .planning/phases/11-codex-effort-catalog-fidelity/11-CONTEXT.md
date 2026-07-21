# Phase 11: Codex effort & catalog fidelity - Context

**Gathered:** 2026-07-21
**Status:** Ready for planning

<domain>
## Phase Boundary

GPT-5.6 effort levels work like official Codex: mid-session Grok→Codex switch soft-clamps effort to a supported level instead of hard-failing, the wire sends `reasoning.effort` only for supported levels, and `reasoning.summary` aligns with catalog defaults (`none` → omit). Catalog effort menus (criterion 1) already landed 2026-07-18 in `default_models.json` (low/medium/high/xhigh for Sol/Terra/Luna). This phase closes the remaining soft-clamp UX, wire emission rules, summary alignment, and UAT-notes closure. No ultra ladder UI. No Grok/xAI path changes.

</domain>

<decisions>
## Implementation Decisions

### Soft-clamp semantics (mid-session switch)
- Clamp rule: official Codex semantics — keep the current effort if it is in the model's supported list; otherwise clamp to the **middle entry** of the supported list, falling back to the model default (verified in codex-rs `core/src/session/turn_context.rs:243-263`). *Correction 2026-07-21: discuss originally proposed "nearest-supported downgrade" believing it matched official Codex; Grok research showed official uses middle-of-list. Phase goal is "work like official Codex", so official semantics govern.* `max`→`xhigh` parse alias unchanged
- User feedback: one-line non-blocking TUI notice when clamp occurs ("effort clamped to X")
- Preference stickiness: user's chosen effort stays sticky; clamp applies per-request at build time — switching back to Grok restores the original preference
- Location: single choke point in the request-build/route path (shared function all callers route through), not switch-time UI handlers

### Wire mapping & catalog
- `reasoning.effort` emission: send only when the model advertises supported levels AND the (post-clamp) effort is in that list; omit otherwise — per `.planning/research/CODEX-RESPONSES-WIRE.md` §5
- Ultra→max ladder UI: skipped (YAGNI) — low/medium/high/xhigh already in catalog; `max` maps to `xhigh` at parse
- `reasoning.summary` alignment (optional criterion 5): in scope — omit summary when catalog default is `none`, matching official Codex
- Scope isolation: clamp/emission logic on the Codex provider path only; Grok/xAI effort behavior untouched

### Verification & UAT closure
- Criterion 4 evidence: update UAT runbook/notes to remove effort as a product blocker, referencing the passing tests
- Live verification: automated tests + wire-shape fixtures suffice — phase criteria are UX polish, not OPS live gates (D-16 untriggered); live effort spot-check optional for operator
- Regression coverage: soft-clamp unit tests + wire emission tests added to the consolidated focused suite

### Claude's Discretion
- Exact TUI notice wording/placement
- Test file organization within existing suite conventions
- Whether clamp helper lives in sampling-types or the Codex request builder (pick the single-choke-point location the code actually routes through)

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `ReasoningEffort` enum (`crates/codegen/xai-grok-sampling-types/src/types.rs` ~line 788+): parses none/minimal/low/medium/high/xhigh/max; `max`→`xhigh` mapping exists at parse
- `supports_reasoning_effort_meta` / `parse_reasoning_effort_meta` helpers + `REASONING_EFFORT_META_KEY` in same file
- Catalog: `crates/codegen/xai-grok-models/default_models.json` — GPT-5.6 entries already advertise `reasoning_efforts` low/medium/high/xhigh with Codex-aligned defaults (sol: low; terra/luna: medium)
- Phase 10 typed sampler profile (Plan 10-01, disabled everywhere) — trust-gated Codex profile machinery
- Ground truth reference: `/home/cristian/bum/codex/codex-rs/protocol/src/openai_models.rs` + `models-manager/models.json` (official effort enum/catalog patterns)

### Established Patterns
- Conversation state mutations go through actor commands (Phase 10 sanitizer decision) — no caller-side whole-history read/replace
- Wire assertions via consolidated focused suite (26/26 pattern from 10-05)
- Check-only formatting; no mutating workspace formatter runs

### Integration Points
- Codex request builder (Responses API path) — where `reasoning.effort`/`reasoning.summary` are serialized
- Model switch flow (Phase 6 mid-session switch gate) — where active effort meets new model's supported list
- Effort picker TUI (enabled post catalog fix) — reads per-model `reasoning_efforts`

</code_context>

<specifics>
## Specific Ideas

- Official Codex CLI behavior is the contract: soft-clamp like codex-rs does, never hard-fail a turn over effort
- Research doc `.planning/research/CODEX-RESPONSES-WIRE.md` §5 + "Effort catalog" section is the wire spec for this phase

</specifics>

<deferred>
## Deferred Ideas

- Ultra→max full ladder UI alias tier (explicitly skipped this phase; revisit only if product wants full Codex ladder)
- WS incremental / tool naming / attribution depth — Phase 12

</deferred>
