# Phase 9: Daily-driver end-to-end validation - Context

**Gathered:** 2026-07-17
**Status:** Ready for planning

<domain>
## Phase Boundary

Prove **bum** is usable as the default coding agent for real work across both providers and cross-provider agents. Close OPS-03..OPS-06 with a hybrid bar: automated regression (prior phase gates + thin p9_ smoke, fixture tokens only in CI) **plus** live dual-login human UAT under real xAI + Codex OAuth.

This phase is **validation-first**. Product code lands only when live UAT finds **blockers** that violate OPS-03..06 (routing, credentials, switch gate, cross-provider spawn, daily tools). It does **not** include: public signed install channel, internal crate renames, mass `GROK_*` env rename, custom agentic workflows, full model-catalog matrix, enterprise IdP, or fleet cost dashboards.

</domain>

<decisions>
## Implementation Decisions

### Validation methodology
- **Hybrid proof:** green automated regression (prior `p*_` / phase gates + thin `p9_` smoke with fixtures) **plus** live dual-login human UAT for OPS-03..OPS-06 — matches Phase 7 deferral of live multi-turn dual-login NL E2E to Phase 9; CI stays fixture-only
- **Phase gate requires both** automated green **and** signed human checklist covering all four OPS requirements — not automated-only or human-only
- **Fix blocking bugs in-phase** — small fail-closed / routing / chrome / tool-path fixes that block the daily-driver bar; larger feature work → deferred ideas / later milestone
- **Live UAT environment:** real dual OAuth under `~/.bum` (or isolated temp `BUM_HOME`) on the developer machine; no CI live OAuth secrets

### Session matrix
- **OPS-03 / OPS-04 minimum:** one productive tool turn per provider after login — at least read + one of edit/shell succeeds on a real backend (not fixture-only)
- **OPS-05:** same session — xAI turn → switch to GPT-5.6 → Codex turn (optional reverse) without restarting the CLI
- **OPS-06:** both live directions — Grok parent → Codex child **and** Codex parent → Grok child (NL or Task spawn with explicit model + effort)
- **Models under test:** one current xAI daily model + one GPT-5.6 catalog entry (e.g. Sol) — not a full catalog sweep

### Evidence & pass criteria
- **Live evidence:** structured UAT checklist in phase dir + VERIFICATION.md rows for OPS-03..06 with pass/fail and short notes (who, when, model IDs)
- **Provider capability gaps:** document honestly if Codex/OpenAI cannot support a Grok-only tool feature; remaining supported tools must still clear the daily-driver bar
- **Automated scope:** re-run critical prior-phase gates / residual greps + add thin `p9_` discovery smoke where useful; no intentional-red; no live network required for automated gate
- **Secrets:** never commit tokens, auth.json, or full raw transcripts with secrets; short redacted notes / log excerpts only

### Scope boundary & fix policy
- **Validation-first** — harness, checklists, VALIDATION/PHASE-GATE, evidence docs; product code only for blockers found during UAT
- **Out of scope (unchanged PROJECT outs):** public x.ai install channel, crate rename (`xai-grok-*`), mass `GROK_*` rename, custom workflows, full catalog matrix, enterprise IdP
- **Human UAT:** Cristian runs the live dual-login checklist during execute; agent prepares scripts/checklists and lands automated green first
- **Live OAuth required for green OPS rows** — if network/account fails, block the human path (fix or re-auth); do not mark live OPS rows green on fixtures alone

### Claude's Discretion
- Exact checklist layout, p9_ filter names, and which prior gates are re-invoked in PHASE-GATE
- Whether live UAT is TUI, headless, or both for a given OPS row (prefer TUI daily path when practical)
- How deep automated p9_ smoke goes beyond discovery (fixture dual-token residual only)
- Copy for any new login/switch error strings if UAT finds friction (align Phase 5–7 labels)
- Whether a single UAT session can cover multiple OPS rows when natural (e.g. one dual-login session hits switch + both providers + spawn)

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- Dual-slot auth under `$BUM_HOME/auth.json` (Phases 2, 5) — `providers.xai` / `providers.codex`
- Provider-aware routing (Phase 4) — model → catalog `provider` → base URL + credential slot
- Mid-session switch + missing-provider gate (Phase 6)
- Cross-provider Task spawn + effort + isolation tests `p7_*` (Phase 7 AGENT-01..06 automated)
- Quiet fork gates `p8_*` — no auto-update, telemetry off, bum chrome (Phase 8)
- Prior phase VALIDATION.md / PHASE-GATE.md patterns (green-only, fixture tokens, residual greps)

### Established Patterns
- **Green-only protocol** — no intentional-red filters in phase gates
- **Fixtures for CI** — `xai-fake-token*` / `codex-fake-token*`; live OAuth never required for automated gate
- **Fail closed** — missing provider → login hint; never cross-send bearers
- Per-crate `cargo test -p …` filters; full workspace test is too heavy for gates

### Integration Points
- Live path: `bum login` / `bum login --provider codex` → TUI session → `/model` switch → Task tool cross-provider spawn
- Automated path: shell/pager/tools unit + integration filters already covering routing, auth, spawn
- Evidence: `.planning/phases/09-…/09-VALIDATION.md`, UAT checklist, `09-VERIFICATION.md`

</code_context>

<specifics>
## Specific Ideas

- Phase 7 deferred **live multi-turn dual-login NL E2E** here (AGENT-06 live matrix → OPS-06)
- Success story still applies: main on Grok, “start a Codex Sol medium-effort subagent to research X” returns results under real dual login
- Daily-driver bar from PROJECT: auth, model switch, tools, and sessions work for both providers after v1
- Prefer reusing Phase 7/8 gate structure (VALIDATION map + PHASE-GATE discover+execute) over inventing a new verification product

</specifics>

<deferred>
## Deferred Ideas

- Public signed x.ai install channel for bum
- Internal crate renames (`xai-grok-*`) / mass `GROK_*` env rename
- Custom agentic workflow engine (later milestone)
- Full model-catalog matrix / multi-account cost dashboards
- Enterprise IdP beyond existing xAI OIDC hooks
- Long multi-hour soak sessions (beyond one productive tool turn + switch + spawn matrix)

</deferred>
