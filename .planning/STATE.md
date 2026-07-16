---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: milestone
current_phase: 4
current_phase_name: Provider-aware request routing
status: planning
stopped_at: Completed 03-03-PLAN.md
last_updated: "2026-07-16T11:20:53.960Z"
last_activity: 2026-07-16
last_activity_desc: Phase 3 complete, transitioned to Phase 4
progress:
  total_phases: 3
  completed_phases: 3
  total_plans: 12
  completed_plans: 12
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-07-16)

**Core value:** One CLI (`bum`) can log into both xAI and Codex and freely switch between Grok and GPT-5.6 models in a real coding session — including cross-provider subagent orchestration.
**Current focus:** Phase 3 — Model catalog & GPT-5.6 entries (executing)

## Current Position

Phase: 4 of 9 (Provider-aware request routing)
Plan: Not started
Status: Ready to plan
Last activity: 2026-07-16 — Phase 3 complete, transitioned to Phase 4

Progress: [██████████] 100%

## Performance Metrics

**Velocity:**

- Total plans completed: 3
- Average duration: 20.7min
- Total execution time: 1.0 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 01 | 3/5 | 62min | 20.7min |
| 3 | 3 | - | - |

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
| Phase 03 P02 | 3min | 3 tasks | 2 files |
| Phase 03 P03 | 36min | 4 tasks | 6 files |

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
- [Phase ?]: Prefetch collision uses remove-then-append of ModelProvider::Codex rows (Sol→Terra→Luna after remote), not replace-in-place
- [Phase ?]: Empty Some(prefetched) still injects bundled Codex when !has_custom_endpoint (Q1)
- [Phase ?]: GPT catalog visibility independent of Codex login; available_models only uses visible_for_auth
- [Phase ?]: ACP meta.provider always inserted from trusted ModelInfo.provider (xai|codex)
- [Phase ?]: CLI format_cli_model_row: star/dash id (name); no (default) suffix
- [Phase ?]: Interactive /model UAT optional advisory; automated ACP+CLI+settings is MOD-01/02 gate

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

Last session: 2026-07-16T11:03:59.740Z
Stopped at: Completed 03-03-PLAN.md
Resume file: None
Next: continue Phase 1 remaining plans (01-04, 01-05)
