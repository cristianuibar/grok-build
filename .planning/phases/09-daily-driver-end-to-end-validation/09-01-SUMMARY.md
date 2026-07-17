---
phase: 09-daily-driver-end-to-end-validation
plan: 01
subsystem: testing
tags: [p9, validation, dual-login, fixture, nyquist, OPS-03, OPS-04, OPS-05, OPS-06]

requires:
  - phase: 07-cross-provider-multi-agent-orchestration
    provides: cross_provider_subagent harness, p7_ isolation / missing-provider residual
  - phase: 08-quiet-fork-rebrand-polish
    provides: green-only p8_ residual pattern
provides:
  - Differentiated green-only p9_ composition residual under cross_provider_subagent
  - Finalized 09-VALIDATION.md dual auto/human map for OPS-03..06 with D-NN citations
  - Wave 0 inventory readiness for VALIDATION + p9_ (PHASE-GATE/UAT still pending)
affects:
  - 09-02 PHASE-GATE discover of p9_
  - 09-03 UAT runbook
  - 09-05 hybrid verification close

tech-stack:
  added: []
  patterns:
    - "Single p9_ composition residual (dual-slot readable + empty-slot login-hint) instead of dual p7 clones"
    - "VALIDATION dual columns: automated residual + Required live UAT per OPS row"

key-files:
  created:
    - .planning/phases/09-daily-driver-end-to-end-validation/09-01-SUMMARY.md
  modified:
    - crates/codegen/xai-grok-shell/tests/cross_provider_subagent.rs
    - .planning/phases/09-daily-driver-end-to-end-validation/09-VALIDATION.md

key-decisions:
  - "Landed one composition residual p9_daily_driver_dual_slot_readable_and_empty_codex_login_hint (C1-M1)"
  - "Skipped optional p9_route_metadata_* — host/slot already covered by p7_resolve_route; bearer remains p7_isolation_* (C1-L1)"
  - "wave_0_inventory_ready true; wave_0_complete and nyquist_compliant stay false until later plans"

patterns-established:
  - "p9_ prefix discoverable via cargo test --test cross_provider_subagent p9_"
  - "Fixture tokens xai-fake-token-p9 / codex-fake-token-p9 under ensure_sandbox OnceLock BUM_HOME"

# Plan association (frontmatter of 09-01-PLAN). Live OPS product green remains Plans 03–05 (D-16);
# REQUIREMENTS.md checkboxes intentionally NOT marked complete by this plan.
requirements-completed: [OPS-03, OPS-04, OPS-05, OPS-06]

coverage:
  - id: D1
    description: "Differentiated green-only p9_ composition residual (dual-slot + empty-Codex login-hint) discoverable under cross_provider_subagent"
    requirement: OPS-03
    verification:
      - kind: integration
        ref: "cargo test -p xai-grok-shell --test cross_provider_subagent p9_ -- --nocapture # p9_daily_driver_dual_slot_readable_and_empty_codex_login_hint"
        status: pass
    human_judgment: false
  - id: D2
    description: "09-VALIDATION.md dual-maps OPS-03..06 to automated residual commands and Required live UAT with D-NN citations"
    requirement: OPS-03
    verification:
      - kind: other
        ref: "rg OPS-03|OPS-04|OPS-05|OPS-06|p9_|live UAT required|D-0 .planning/.../09-VALIDATION.md"
        status: pass
    human_judgment: false
  - id: D3
    description: "Live OPS rows remain pending; fixture residual never substitutes live UAT (D-16)"
    requirement: OPS-05
    verification: []
    human_judgment: true
    rationale: "Live dual-login UAT is Plans 03–05; Plan 01 only inventories the requirement"

# Metrics
duration: 2min
completed: 2026-07-17
status: complete
---

# Phase 9 Plan 01: Validation map and p9_ residual Summary

**Single differentiated green `p9_` composition residual plus finalized dual auto/human VALIDATION inventory for OPS-03..06**

## Performance

- **Duration:** ~2 min (warm compile)
- **Started:** 2026-07-17T15:10:31Z
- **Completed:** 2026-07-17T15:12:04Z
- **Tasks:** 2/2
- **Files modified:** 2 (+ SUMMARY)

## Accomplishments

- Landed exactly one green fixture-only residual: `p9_daily_driver_dual_slot_readable_and_empty_codex_login_hint` (dual-slot readable + empty Codex `bum login --provider codex` fail-closed in one test)
- Finalized `09-VALIDATION.md` with dual auto/human columns, P0/P1/P2 re-run inventory, D-NN coverage table, exclusions (D-14/D-04/D-11/D-16/C1-M1/C1-L1), and Wave 0 inventory checkboxes for VALIDATION + p9_
- Confirmed discover `n=1` (≤3), green execute, and no dual p7 clone names under `p9_`

## Task Commits

Each task was committed atomically:

1. **Task 1: GREEN single differentiated p9_ residual smoke** - `83fe627` (test)
2. **Task 2: Finalize 09-VALIDATION.md dual auto/human inventory** - `06e7d7f` (docs)

## Files Created/Modified

- `crates/codegen/xai-grok-shell/tests/cross_provider_subagent.rs` — Phase 9 section + composition residual
- `.planning/phases/09-daily-driver-end-to-end-validation/09-VALIDATION.md` — dual map, greened p9_ record, exclusions, Wave 0 inventory
- `.planning/phases/09-daily-driver-end-to-end-validation/09-01-SUMMARY.md` — this file

## Decisions Made

- **Composition residual over dual clones:** one test composes dual-slot store + empty-slot login-hint (addresses C1-M1)
- **No optional route metadata p9_:** `p7_resolve_route_isolates_base_url_key_prefix_both_directions` already covers host/slot; bearer isolation stays `p7_isolation_*` (C1-L1)
- **Frontmatter:** `wave_0_inventory_ready: true`, `wave_0_complete: false`, `nyquist_compliant: false` until Plan 05

## Deviations from Plan

None - plan executed exactly as written.

Optional route residual was explicitly skippable when already covered; skipped intentionally per plan action step 4.

## Issues Encountered

None

## User Setup Required

None - no external service configuration required. Live UAT credentials are operator-local in later plans.

## Next Phase Readiness

- Plan 02 can discover `p9_` non-vacuously and compose prior-gate re-runs from the P0/P1 inventory table
- Live OPS rows remain pending (Plans 03–05); do not claim hybrid GREEN yet
- Unrelated dirty file left untouched: `09-04-PLAN.md` (pre-existing workspace change)

## Verify Results

```text
discover n=1
test p9_daily_driver_dual_slot_readable_and_empty_codex_login_hint ... ok
Task 2 rg dual-map anchors: PASS
```

## Self-Check: PASSED

- FOUND: `crates/codegen/xai-grok-shell/tests/cross_provider_subagent.rs` (`p9_daily_driver_*`)
- FOUND: `09-VALIDATION.md` with `p9_` + OPS dual map
- FOUND commits: `83fe627`, `06e7d7f`

---
*Phase: 09-daily-driver-end-to-end-validation*
*Completed: 2026-07-17*
)
