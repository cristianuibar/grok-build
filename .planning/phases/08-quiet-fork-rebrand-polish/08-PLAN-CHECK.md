# Phase 8 Plan Check — Quiet fork & rebrand polish

**Checked:** 2026-07-17 (initial)  
**Replan from reviews:** 2026-07-17 (Codex cycle-1 → plans updated)  
**Plans verified:** 6 (`08-01` … `08-06`)  
**Status:** **STALE / will re-run after replan** (C1-L2)

**Phase goal:** Product presents fully as **bum** and does not phone home or auto-update as stock Grok Build.

---

## Verdict (pre-replan check — historical)

| Result | Detail |
|--------|--------|
| Prior FAIL | Plan `08-03` YAML frontmatter was broken (closing fence glued to last key_links line). **Fixed before cycle-1 review.** |
| Cycle-1 review | `gsd-tools query verify.plan-structure` reported all six plans valid; verdict **REPLAN** on product gaps (OTLP, remote, residual CLI, hermetic update proofs). |

## Post-replan note (C1-L2)

This PLAN-CHECK artifact is intentionally marked **stale** after the reviews-mode replan that addressed:

| Finding | Plan update |
|---------|-------------|
| C1-H1 residual ID-02 inventory | Plans 02, 03, 06 residual greps |
| C1-H2 internal OTLP | Plan 05 Task 3 + Plan 06 gate |
| C1-H3 remote re-enable | Plan 05 Task 2 restrictive-only |
| C1-M1 force_feedback bypass | Plan 05 Task 1 |
| C1-M2 hermetic update proofs | Plan 04 Task 1 |
| C1-L1 p8_sentry unconditional | Plan 06 PHASE-GATE |
| C1-L3 `&&` verify hygiene | Plans 01/04/05/06 |

**Action for orchestrator:** re-run plan-structure validation + plan-checker after this replan (`will re-run after replan`). Do not treat this file as a fresh PASS until that re-check lands.

---

## Coverage Summary (expected after replan execute)

| Requirement | Plans | Status |
|-------------|-------|--------|
| **ID-02** product chrome + residual CLI as bum | 01, 02, 03, 06 | Planned (expanded residual inventory) |
| **OPS-01** stock auto-update off + hermetic no-network | 01, 04, 06 | Planned (finish_update_on_exit covered) |
| **OPS-02** telemetry/OTLP/feedback/Sentry off | 01, 05, 06 | Planned (OTLP + remote restrictive) |

| ROADMAP success criterion | Delivered by |
|---------------------------|--------------|
| 1 UI chrome / help / strings present as **bum** | 02 + 03 (+ 05 feedback copy) + 06 residual greps |
| 2 Stock xAI auto-update channel disabled | 04 (+ hermetic counters) |
| 3 Product telemetry / phone-home disabled by default | 01 baseline + 05 (OTLP + remote + Sentry + feedback) |

### Locked decisions (CONTEXT D-01..D-15)

| Decision | Primary plan(s) | Notes |
|----------|-----------------|-------|
| D-01 product chrome → bum | 02, 03, 06 | Residual runtime CLI inventory |
| D-02 keep model brands | 01, 02, 06 | No catalog edit |
| D-03 no crate renames | 02, 03, 06 | Exclusions |
| D-04 hard-off auto-update | 04 | `should_check_for_updates` always false |
| D-05 explicit update no-op + message | 04 | UI-SPEC locked copy |
| D-06 min-version ignore | 04 | hard no-op |
| D-07 code defaults + tests | 04 | unwrap_or(false), settings, first-run, hermetic counters |
| D-08 TelemetryMode::Disabled | 01, 05 | + OTLP off (C1-H2) |
| D-09 Mixpanel off default | 05 | via Disabled |
| D-10 Sentry off default | 05, 06 | force disabled + **unconditional** `p8_sentry` |
| D-11 local logs OK | 05 | file logs OK; Server OTLP not |
| D-12 clap bum / bum TUI | 02 | |
| D-13 leave GROK_* internal | 02, 03, 05, 06 | |
| D-14 hero/welcome rebrand | 02, 03 | drop Beta marketing |
| D-15 /feedback quiet | 05 | no SendFeedback + force gate |

### Deferred ideas

| Deferred idea | In plans? |
|---------------|-----------|
| Public signed x.ai install channel | **Excluded** |
| Internal `xai-grok-*` crate rename | **Excluded** |
| Mass `GROK_*` → `BUM_*` env rename | **Excluded** (D-13) |
| Live dual-provider E2E (Phase 9) | **Excluded** |
| Custom agentic workflows | **Excluded** |
| Agent system prompt rebrand | **Excluded** (residual allowlist) |
