---
phase: 01-product-identity-isolated-home
plan: 02
subsystem: infra
tags: [BUM_HOME, twin-resolver, managed-bin, bum, auto-update, worktree, leader]

requires:
  - phase: 01-01
    provides: pure resolve_product_home (BUM_HOME only); managed application leaf bin/bum
provides:
  - twin resolve_grok_home lockstep with SoT (var_os BUM_HOME, empty reject, default .bum)
  - GrokHomeFixture on BUM_HOME / bum-home
  - workspace worktree fallback .bum
  - leader managed_grok_bin_name bum/bum.exe
  - updater swap_managed_bin_links installs bin/bum (no grok alias under product home)
  - install + concurrent + downgrade-matrix tests on managed bin/bum
affects:
  - 01-04-remaining-home-env-cutover
  - worktree DB paths
  - leader spawn / managed install relaunch
  - auto-update activation path

tech-stack:
  added: []
  patterns:
    - leaf twin mirrors pure SoT without crate dep (var_os + empty reject + home_dir)
    - managed product command leaf bum under product home; download stems may stay historical

key-files:
  created:
    - .planning/phases/01-product-identity-isolated-home/deferred-items.md
  modified:
    - crates/codegen/xai-fast-worktree/src/db/mod.rs
    - crates/codegen/xai-fast-worktree/src/db/tests.rs
    - crates/codegen/xai-grok-workspace/src/worktree/mod.rs
    - crates/codegen/xai-grok-shell/src/leader/mod.rs
    - crates/codegen/xai-grok-sandbox/src/paths.rs
    - crates/codegen/xai-grok-update/src/auto_update.rs
    - crates/codegen/xai-grok-update/src/version.rs
    - crates/codegen/xai-grok-update/tests/common/mod.rs
    - crates/codegen/xai-grok-update/tests/test_install_internal.rs
    - crates/codegen/xai-grok-update/tests/test_concurrent_convergence.rs
    - crates/codegen/xai-grok-update/tests/test_downgrade_matrix.rs

key-decisions:
  - "Twin uses std::env::home_dir (SoT parity) with HOME fallback only for error messaging; no crate dep on xai-grok-config this phase"
  - "Managed install leaf under product home is bum only — no bin/grok compatibility alias (D-BIN)"
  - "Download artifact stems may retain grok-<version>-platform; installed managed command is bum"

patterns-established:
  - "Leaf twin resolvers: var_os + empty-as-absent + default segment lockstep with pure resolve_product_home"
  - "Update integration tests isolate product home via BUM_HOME OnceLock in common::test_home"

requirements-completed: [ID-01, ID-03]

coverage:
  - id: D1
    description: Twin resolve_grok_home uses BUM_HOME (var_os, empty reject) and defaults to .bum
    requirement: ID-03
    verification:
      - kind: unit
        ref: cargo test -p xai-fast-worktree --lib
        status: pass
    human_judgment: false
  - id: D2
    description: Workspace worktree fallback joins .bum; managed leader bin is bum
    requirement: ID-01
    verification:
      - kind: other
        ref: cargo check -p xai-grok-shell --lib && cargo check -p xai-grok-workspace -p xai-grok-sandbox
        status: pass
    human_judgment: false
  - id: D3
    description: Updater swap_managed_bin_links provisions product-home bin/bum; install/concurrent/downgrade tests green
    requirement: ID-01
    verification:
      - kind: integration
        ref: cargo test -p xai-grok-update --test test_install_internal
        status: pass
      - kind: integration
        ref: cargo test -p xai-grok-update --test test_concurrent_convergence
        status: pass
      - kind: integration
        ref: cargo test -p xai-grok-update --test test_downgrade_matrix
        status: pass
    human_judgment: false

duration: 19min
completed: 2026-07-16
status: complete
---

# Phase 01 Plan 02: Twin / Managed Bin / Updater Leaf Summary

**Worktree twin, leader managed bin, and auto-update activation all target `BUM_HOME` / `~/.bum` / `bin/bum` in lockstep with config SoT — no managed `bin/grok` alias under product home.**

## Performance

- **Duration:** 19 min
- **Started:** 2026-07-16T02:47:09Z
- **Completed:** 2026-07-16T03:06:27Z
- **Tasks:** 3/3
- **Files modified:** 12

## Accomplishments

- Twin `resolve_grok_home` is semantic lockstep with pure `resolve_product_home`: `var_os("BUM_HOME")`, empty rejection, non-Unicode honored, default `.bum` via `home_dir`
- Workspace fallback and docs no longer invent a stock `.grok` product tree on twin failure
- Leader managed leaf and hard-coded managed-install tests use `bum` / `bum.exe`
- Updater `swap_managed_bin_links` installs `bin/bum`; install, concurrent convergence, and downgrade-matrix assert managed `bin/bum`

## Task Commits

1. **Task 1: Lockstep twin resolve_grok_home + GrokHomeFixture + platform note** - `fcd836e` (feat)
2. **Task 2: Workspace fallback + managed bin leaf + leader hard-coded tests + sandbox** - `478219c` (feat)
3. **Task 3: Updater managed bin leaf bin/bum + install + downgrade-matrix tests** - `612dba9` (feat)

**Plan metadata:** `48e29b4` (docs: complete twin managed-bin updater plan)

## Files Created/Modified

- `crates/codegen/xai-fast-worktree/src/db/mod.rs` — twin resolver + fixture on BUM_HOME
- `crates/codegen/xai-fast-worktree/src/db/tests.rs` — empty/override/non-Unicode/fixture unit tests
- `crates/codegen/xai-grok-workspace/src/worktree/mod.rs` — `.bum` fallback + BUM_HOME fixture
- `crates/codegen/xai-grok-shell/src/leader/mod.rs` — `managed_grok_bin_name` → bum; tests updated
- `crates/codegen/xai-grok-sandbox/src/paths.rs` — docs for writable `$BUM_HOME` / `~/.bum`
- `crates/codegen/xai-grok-update/src/auto_update.rs` — managed leaf bum in `swap_managed_bin_links`
- `crates/codegen/xai-grok-update/src/version.rs` — on-disk version docs for `~/.bum/bin/bum`
- `crates/codegen/xai-grok-update/tests/common/mod.rs` — `test_home` sets `BUM_HOME`
- `crates/codegen/xai-grok-update/tests/test_install_internal.rs` — managed bin/bum assertions
- `crates/codegen/xai-grok-update/tests/test_concurrent_convergence.rs` — managed bin/bum
- `crates/codegen/xai-grok-update/tests/test_downgrade_matrix.rs` — managed bin/bum
- `.planning/phases/01-product-identity-isolated-home/deferred-items.md` — shell lib-test compile debt

## Decisions Made

- Align twin user-home with SoT `std::env::home_dir()`; document leaf-crate boundary (no config dep this phase)
- No managed `bin/grok` compatibility alias under product home (D-BIN)
- Historical download stems (`grok-<version>-platform`) retained; installed managed command is `bum`

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing critical] Workspace worktree test fixture still set GROK_HOME**
- **Found during:** Task 2
- **Issue:** After twin cutover, `worktree_db_fixture` still set `GROK_HOME`, so isolation would miss the twin resolver
- **Fix:** Fixture sets/restores `BUM_HOME` and uses `bum-home` dir name
- **Files modified:** `crates/codegen/xai-grok-workspace/src/worktree/mod.rs`
- **Committed in:** `478219c`

**2. [Rule 2 - Missing critical] Update test_home still set GROK_HOME**
- **Found during:** Task 3
- **Issue:** Config SoT only honors `BUM_HOME`; tests setting `GROK_HOME` would not isolate product home
- **Fix:** `common::test_home` sets `BUM_HOME`
- **Files modified:** `crates/codegen/xai-grok-update/tests/common/mod.rs`
- **Committed in:** `612dba9`

**3. [Rule 2 - Docs] version.rs managed-path docs still said ~/.grok/bin/grok**
- **Found during:** Task 3
- **Issue:** On-disk version probe already uses `grok_application()` → bin/bum; docs lagged
- **Fix:** Updated rustdoc to product-home bin/bum; noted historical download stems
- **Files modified:** `crates/codegen/xai-grok-update/src/version.rs`
- **Committed in:** `612dba9`

---

**Total deviations:** 3 auto-fixed (Rule 2)
**Impact on plan:** Correctness-only; no scope creep. Pre-existing shell lib-test compile failures deferred (not caused by this plan).

## Issues Encountered

- `cargo test -p xai-grok-shell leader` cannot compile the full lib-test crate due to pre-existing missing test APIs (`WorkspaceOps::for_test`, `EnvVarGuard`, etc.). Documented in `deferred-items.md`. Production `cargo check -p xai-grok-shell --lib` is green; managed leaf verified by source + helper-using unit tests in tree.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Twin, managed bin, and updater leaf no longer reintroduce `~/.grok` / `bin/grok` product paths
- Remaining home-env cutover (01-04) and product identity polish (01-05) can proceed without re-opening managed leaf naming
- Shell lib-test harness repair remains out of phase scope

## Known Stubs

None.

## Threat Flags

None beyond plan register (T-01-04 twin, T-01-05 fallback, T-01-12 updater leaf mitigated as specified).

## Self-Check: PASSED

- All key files present on disk
- Commits `fcd836e`, `478219c`, `612dba9` present in git log
- Artifacts contain `BUM_HOME` / `"bum"` / managed `bin/bum` as required by plan must_haves
