---
gsd_state_version: 1.0
milestone: v1.1
milestone_name: Upstream Grok Build parity
status: ready_to_plan
last_updated: "2026-07-22T00:00:00Z"
last_activity: 2026-07-22
progress:
  total_phases: 9
  completed_phases: 0
  total_plans: 0
  completed_plans: 0
  percent: 0
---

# Project State

## Project Reference

See: `.planning/PROJECT.md` (updated 2026-07-22)

**Core value:** One CLI (`bum`) can log into both xAI and Codex and freely switch between Grok and GPT-5.6 models in a real coding session, including cross-provider subagent orchestration.

**Current focus:** Phase 13 — Immutable Baseline & Provenance for v1.1 upstream parity.

## Current Position

Phase: 13 of 21 (milestone phase 1 of 9) — Immutable Baseline & Provenance
Plan: — (phase not yet planned)
Status: Ready to plan
Last activity: 2026-07-22 — v1.1 roadmap created with 47/47 requirements uniquely mapped

Progress: [░░░░░░░░░░] 0%

## Performance Metrics

**v1.1 velocity:**
- Total plans completed: 0
- Average duration: —
- Total execution time: 0 hours

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| v1.1 | 0 | 0 min | — |

## Accumulated Context

### Decisions

Decisions are logged in `.planning/PROJECT.md` and `.planning/REQUIREMENTS.md`.

- v1.1 is frozen to upstream public SHA `3af4d5d39897855bdcc74f23e690024a5dc05573` (0.2.109), `SOURCE_REV` `0f4d7c91b8b2b408333f6de1e8a76cb8eaa71899`.
- There is no merge base; integrate from the pinned upstream baseline plus a curated bum contract overlay.
- Preserve `bum`, `~/.bum`, dual OAuth, per-model routing, cross-provider children, Codex wire behavior, and quiet/no-phone-home operation.
- A two-parent ancestry bridge is allowed only after validation and must leave the parity tree byte-for-byte unchanged.

### Pending Todos

- Plan Phase 13 with deeper provenance research.
- Run focused phase research before planning Phases 15, 17, 18, 19, and 21.

### Blockers/Concerns

- The exported root `Cargo.toml` is generated, but no public generator entry point is known; resolve in Phase 15 research.
- Public-pin gaps (`ClientToolResult`/`ChatConfig`, workflow-authoring skills, MiniSweAgent traceability) must remain explicit and must not be fabricated.
- The no-merge-base constraint makes premature history joining a hard stop.

## Deferred Items

Items carried from v1.0 remain non-blocking unless absorbed by a mapped v1.1 requirement:

| Category | Item | Status | Deferred At |
|----------|------|--------|-------------|
| tech_debt | Codex parent Task visibility for session-auth-only `grok-build` | pending review | v1.0 |
| tech_debt | Child token longevity regression test | pending review | v1.0 |
| tech_debt | Out-of-band auto-update setting normalization | pending review | v1.0 |
| tech_debt | Residual internal crate rebrand | deferred beyond v1.1 | v1.0 |
| tech_debt | Nyquist validation metadata hygiene | pending review | v1.0 |

## Session Continuity

Last session: 2026-07-22
Stopped at: v1.1 roadmap and unique requirement traceability created; awaiting user approval before Phase 13 planning
Resume file: None
