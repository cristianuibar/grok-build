---
phase: 08-quiet-fork-rebrand-polish
plan: 04
subsystem: privacy
tags: [ops-01, auto-update, min-version, settings, hermetic, p8, quiet-fork]

requires:
  - phase: 08-quiet-fork-rebrand-polish
    provides: Wave 1 p8_ harness + pager-bin bum banner (Plans 01–03)
  - phase: 01-product-identity-isolated-home
    provides: Binary name bum, BUM_HOME isolation
provides:
  - should_check_for_updates hard-off always false (D-04/D-07)
  - bum update / finish_update_on_exit no-op with UI-SPEC locked message (D-05, C1-M2)
  - auto_update effective None→false; no first-run true persist (D-07)
  - enforce_minimum_version_or_exit hard no-op (D-06)
  - Settings auto_update default false + Stock update channel (disabled in bum) description
  - Green p8_no_auto_update, p8_update_cmd, p8_update_no_network, p8_auto_update, p8_min_version, p8_settings_auto_update
affects:
  - 08-05 telemetry/feedback quiet path (update crate retained; no phone-home via update)
  - 08-06 residual greps for stock update messaging

tech-stack:
  added: []
  patterns:
    - "Composition-root hard-off gate + thin stock-helper seams with cfg(test) AtomicUsize counters (C1-M2)"
    - "Pure effective_auto_update_enabled / first_run_auto_update_persist_value policy helpers in update crate"
    - "FinishUpdateOnExit::ChannelDisabled so quit-for-update never claims install success/retry"

key-files:
  created: []
  modified:
    - crates/codegen/xai-grok-pager-bin/src/main.rs
    - crates/codegen/xai-grok-update/src/auto_update.rs
    - crates/codegen/xai-grok-update/src/minimum_version.rs
    - crates/codegen/xai-grok-pager/src/settings/defs.rs
    - crates/codegen/xai-grok-pager/src/settings/registry.rs
    - crates/codegen/xai-grok-pager/tests/settings_e2e.rs
    - crates/codegen/xai-grok-pager/src/app/dispatch/settings/setters.rs
    - crates/codegen/xai-grok-pager/src/app/app_view.rs
    - .planning/phases/08-quiet-fork-rebrand-polish/08-VALIDATION.md

key-decisions:
  - "should_check_for_updates always false — ignore debug flag, CLI flag, and GROK_DISABLE_AUTOUPDATER"
  - "run_update_command exits Ok with two UI-SPEC lines; never check_update_status/run_update"
  - "finish_update_on_exit returns ChannelDisabled and never awaits stock install path (C1-M2)"
  - "auto_update None→false; first_run_auto_update_persist_value always None (no true write)"
  - "enforce_minimum_version_or_exit entry-level hard no-op; keep private helper for floor unit tests"
  - "Settings description: Stock update channel (disabled in bum)."

patterns-established:
  - "stock_run_update_if_available / stock_check_update_status / stock_run_update seams + STOCK_UPDATE_HELPER_CALLS counter"
  - "effective_auto_update_enabled(Option<bool>) pure policy shared by check_update_background and run_update_if_available"
  - "pr13_effective_default(auto_update)=false aligned with registry SettingKind default"

requirements-completed: [OPS-01]

coverage:
  - id: D1
    description: "should_check_for_updates always false for false and true flags"
    requirement: OPS-01
    verification:
      - kind: unit
        ref: "cargo test -p xai-grok-pager-bin --bin bum p8_no_auto_update"
        status: pass
    human_judgment: false
  - id: D2
    description: "bum update prints UI-SPEC disabled copy and exits Ok without stock helpers"
    requirement: OPS-01
    verification:
      - kind: unit
        ref: "cargo test -p xai-grok-pager-bin --bin bum p8_update_cmd"
        status: pass
      - kind: unit
        ref: "cargo test -p xai-grok-pager-bin --bin bum p8_update_no_network"
        status: pass
    human_judgment: false
  - id: D3
    description: "finish_update_on_exit never calls stock helpers (None / Some fail / Some ok)"
    requirement: OPS-01
    verification:
      - kind: unit
        ref: "cargo test -p xai-grok-pager-bin --bin bum p8_update_no_network_finish_update_on_exit"
        status: pass
    human_judgment: false
  - id: D4
    description: "auto_update None is off; first-run does not persist true; status product prefix bum"
    requirement: OPS-01
    verification:
      - kind: unit
        ref: "cargo test -p xai-grok-update --lib p8_auto_update"
        status: pass
    human_judgment: false
  - id: D5
    description: "enforce_minimum_version_or_exit is hard no-op (no network/exit)"
    requirement: OPS-01
    verification:
      - kind: unit
        ref: "cargo test -p xai-grok-update --lib p8_min_version"
        status: pass
    human_judgment: false
  - id: D6
    description: "Settings auto_update default false + disabled-channel description; e2e toggles from off"
    requirement: OPS-01
    verification:
      - kind: unit
        ref: "cargo test -p xai-grok-pager --lib p8_settings_auto_update"
        status: pass
      - kind: unit
        ref: "cargo test -p xai-grok-pager --test settings_e2e auto_update"
        status: pass
    human_judgment: false

duration: 11min
completed: 2026-07-17
status: complete
---

# Phase 8 Plan 04: Stock auto-update hard-off Summary

**OPS-01 hard-off: composition-root gate always false, bum update / Ctrl+U finish no-op with locked UI-SPEC message and hermetic zero stock-helper calls, auto_update None→false without first-run true persist, min-version entry no-op, settings default off.**

## Performance

- **Duration:** 11 min
- **Started:** 2026-07-17T11:05:12Z
- **Completed:** 2026-07-17T11:16:00Z
- **Tasks:** 3/3
- **Files modified:** 9

## Accomplishments

- `should_check_for_updates` hard-off (release + debug); startup and leader branches cannot reach stock helpers
- Explicit `bum update` and quit-for-update (`finish_update_on_exit`) print locked disabled copy and never invoke stock channel helpers (C1-M2 counters stay 0)
- Update crate: `effective_auto_update_enabled(None) == false`; first-run never writes `Some(true)`; status printers use `bum - v…`
- `enforce_minimum_version_or_exit` immediate no-op — never force-installs stock Grok Build
- Settings registry/default/e2e flipped to off; description `Stock update channel (disabled in bum).`

## Task Commits

1. **Task 1: Composition-root hard-off + CLI no-op + hermetic proofs** - `e016157` (feat)
2. **Task 2: Update crate defaults false + min-version no-op + product printers** - `25fa072` (feat)
3. **Task 3: Settings registry default false + description + e2e flip** - `e46f5f5` (feat)

**Plan metadata:** `8bdfeca` (docs: complete plan)

## Files Created/Modified

- `crates/codegen/xai-grok-pager-bin/src/main.rs` — gate, run_update_command, finish_update_on_exit, stock seams + p8_* tests
- `crates/codegen/xai-grok-update/src/auto_update.rs` — effective default false, no first-run true, bum status strings + p8_auto_update_*
- `crates/codegen/xai-grok-update/src/minimum_version.rs` — public entry hard no-op + p8_min_version_or_exit_is_noop
- `crates/codegen/xai-grok-pager/src/settings/defs.rs` — Bool default false + UI-SPEC description
- `crates/codegen/xai-grok-pager/src/settings/registry.rs` — unwrap_or(false), assert false, p8_settings_auto_update_default_false
- `crates/codegen/xai-grok-pager/tests/settings_e2e.rs` — expected defaults/toggles; fix pre-existing broken doc comment
- `crates/codegen/xai-grok-pager/src/app/dispatch/settings/setters.rs` — pr13_effective_default + set_auto_update unwrap_or(false)
- `crates/codegen/xai-grok-pager/src/app/app_view.rs` — auto_update mirror doc default false
- `08-VALIDATION.md` — Plan 04 OPS-01 rows greened

## Decisions Made

- Central hard-off at `should_check_for_updates` rather than scattering call-site rewrites
- Hermetic proofs via cfg(test) AtomicUsize on thin stock seams + structural startup/leader gate helper
- `FinishUpdateOnExit::ChannelDisabled` avoids false “Update installed” / “retry” UX after Ctrl+U
- Min-version: entry no-op preferred over deleting floor evaluation helpers used by unit tests
- Settings description prefers UI-SPEC “disabled in bum” wording over promising stock installs

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Fixed pre-existing settings_e2e syntax error**
- **Found during:** Task 3 (settings_e2e compile)
- **Issue:** Corrupted doc comment around `pr10_picker_seeds_choices_idx_from_pager_snapshot_plan_mode_active` contained a bare `}` and broke the test crate compile (blocked plan verification)
- **Fix:** Restored a valid one-line doc comment for the snapshot-seeding test
- **Files modified:** `crates/codegen/xai-grok-pager/tests/settings_e2e.rs`
- **Committed in:** `e46f5f5`

**2. [Rule 2 - Missing critical functionality] Aligned setter effective default with registry**
- **Found during:** Task 3
- **Issue:** Plan file list omitted `setters.rs` / `app_view.rs`, but `pr13_effective_default("auto_update")` and `set_auto_update` still used `unwrap_or(true)` / `Some(true)`, which would mis-handle idempotence and rollback vs the new default
- **Fix:** Flip effective default and comments to false to match registry + update crate
- **Files modified:** `setters.rs`, `app_view.rs`
- **Committed in:** `e46f5f5`

**3. [Rule 2] Combined TDD RED/GREEN into single feat commits per task**
- **Found during:** Task 1
- **Issue:** `should_check_for_updates` already returned false under debug_assertions, so pure RED for the gate alone would not fail in test profile
- **Fix:** Ship tests + hard-off implementation together with hermetic counter proofs that fail if stock helpers are invoked
- **Verification:** All plan `p8_*` filters green

## Known Stubs

None that block OPS-01. Stock update crate install paths remain in-tree but are unreachable from composition-root hard-off and min-version no-op; not user-facing stubs.

## Threat Flags

None new beyond plan threat model (T-08-05, T-08-06 mitigated by this plan).

## Self-Check: PASSED

- `crates/codegen/xai-grok-pager-bin/src/main.rs` FOUND
- `crates/codegen/xai-grok-update/src/auto_update.rs` FOUND
- `crates/codegen/xai-grok-update/src/minimum_version.rs` FOUND
- `crates/codegen/xai-grok-pager/src/settings/defs.rs` FOUND
- `crates/codegen/xai-grok-pager/src/settings/registry.rs` FOUND
- Commits `e016157`, `25fa072`, `e46f5f5` FOUND
- Verification: p8_no_auto_update, p8_update_cmd, p8_update_no_network, p8_auto_update, p8_min_version, p8_settings_auto_update, settings_e2e auto_update all pass
