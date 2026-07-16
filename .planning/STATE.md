---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: milestone
current_phase: 3
current_phase_name: Model catalog & GPT-5.6 entries
status: in_progress
stopped_at: Completed 03-01-PLAN.md
last_updated: "2026-07-16T10:21:04.327Z"
last_activity: 2026-07-16
last_activity_desc: Phase 3 plan 01 complete — ModelProvider + mixed catalog
progress:
  total_phases: 9
  completed_phases: 2
  total_plans: 12
  completed_plans: 10
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-07-16)

**Core value:** One CLI (`bum`) can log into both xAI and Codex and freely switch between Grok and GPT-5.6 models in a real coding session — including cross-provider subagent orchestration.
**Current focus:** Phase 3 — Model catalog & GPT-5.6 entries (executing)

## Current Position

Phase: 3 of 9 (Model catalog & GPT-5.6 entries)
Plan: 2 of 3 (next: 03-02-PLAN.md)
Status: In progress — 03-01 complete
Last activity: 2026-07-16 — Phase 3 plan 01 ModelProvider + mixed GPT-5.6 catalog

Progress: [████████░░] 83%

## Performance Metrics

**Velocity:**

- Total plans completed: 3
- Average duration: 20.7min
- Total execution time: 1.0 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 01 | 3/5 | 62min | 20.7min |

**Recent Trend:**

- Last 5 plans: 01-02 (19min), 01-03 (21min), 01-01 (22min)
- Trend: —

*Updated after each plan completion*
**Per-Plan Metrics:**

| Plan | Duration | Tasks | Files |
|------|----------|-------|-------|
| Phase 01 P03 | 21min | 3 tasks | 5 files |
| Phase 01 P01 | 22min | 3 tasks | 5 files |
| Phase 01 P02 | 19min | 3 tasks | 12 files |
| Phase 01 P04 | 16min | 3 tasks | 61 files |
| Phase 01 P05 | 20min | 3 tasks | 16 files |
| Phase 03 P01 | 11min | 3 tasks | 9 files |

## Accumulated Context

### Decisions

Decisions are logged in PROJECT.md Key Decisions table.
Recent decisions affecting current work:

- Product/CLI name `bum` with full rebrand and isolated `~/.bum`
- Codex auth = ChatGPT OAuth (not Platform API key primary); GPT → Codex backend
- Mixed model picker; per-model routing (not global provider mode)
- Cross-provider subagents in v1; custom agentic workflows deferred
- Quiet fork: disable xAI auto-update + product telemetry
- [Phase ?]: D-BIN: sole [[bin]] is bum — no dual grok/xai-grok-pager alias in v1
- [Phase ?]: Keep GROK_BINARY env override name this phase (D-SCOPE non-home knobs)
- [Phase ?]: Function name grok_binary() retained; returns bum path
- [Phase ?]: Product home override is BUM_HOME only; GROK_HOME is never read as product home
- [Phase ?]: Pure resolve_product_home takes optional OsString + PathBuf — no process env in unit tests
- [Phase ?]: Kept public symbol grok_home() and OnceLock static name GROK_HOME this phase
- [Phase ?]: Twin uses home_dir for SoT parity; no config crate dep this phase
- [Phase ?]: Managed product bin leaf under home is bum only (no grok alias)
- [Phase ?]: Download stems may stay grok-*; installed managed command is bum
- [Phase ?]: Product-home test sandboxes use BUM_HOME only (no dual-read of GROK_HOME)
- [Phase ?]: PTY product_home() helper; project-local .grok layout left for workspace skills/config
- [Phase ?]: Drop legacy HOME/.grok agent scan when product home differs (D-MIGRATE: no dual-read)
- [Phase ?]: User roles/personas under product home; project cwd/.grok preserved (D-PLUGIN)
- [Phase ?]: bundled_root via grok_home(); extension tests inject explicit roots (OnceLock-safe)
- [Phase ?]: Production product-root readers BUM_HOME only; operational labels teach BUM_HOME/~/.bum (D-SCOPE)
- [Phase ?]: ModelProvider (xai|codex) explicit on catalog chain; missing defaults to xai
- [Phase ?]: GPT-5.6 Sol/Terra/Luna ship with stock agent_type; routing deferred to Phase 4
- [Phase ?]: Phase 3 catalog proofs use cargo test --test model_catalog (not shell --lib)

### Pending Todos

None yet.

### Blockers/Concerns

- ChatGPT Responses transport (HTTP SSE vs WS) needs live validation during Phase 4–5 planning
- GPT-5.6 model IDs may be plan/region gated — reconfirm at implement time
- Codex public OAuth `client_id` reuse is common practice but not a formal partner API

## Deferred Items

| Category | Item | Status | Deferred At |
|----------|------|--------|-------------|
| v2 | Custom agentic workflows (WF-V2-01) | Deferred | init |
| v2 | Import stock grok/codex credential stores | Deferred | init |
| v2 | N-provider marketplace / multi-account aliases | Deferred | init |
| v2 | API-key fallback primary path (AUTH-V2-01 secondary only) | Deferred | init |

## Session Continuity

Last session: 2026-07-16T10:21:04.319Z
Stopped at: Completed 03-01-PLAN.md
Resume file: None
Next: continue Phase 1 remaining plans (01-04, 01-05)
