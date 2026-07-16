---
phase: 01-product-identity-isolated-home
plan: 01
subsystem: infra
tags: [bum-home, BUM_HOME, paths, OnceLock, isolation, display-labels]

requires: []
provides:
  - pure resolve_product_home (BUM_HOME only)
  - default product home ~/.bum via grok_home() OnceLock
  - managed application leaf bin/bum
  - display labels ~/.bum / $BUM_HOME
  - process-isolated integration tests for override and GROK_HOME ignore
affects:
  - 01-02-twin-managed-bin
  - product-state writers (auth, sessions, marketplace, logs)
  - pager path helpers

tech-stack:
  added: []
  patterns:
    - pure env-free resolver for multi-scenario unit tests
    - one OnceLock/env scenario per integration-test process

key-files:
  created:
    - crates/codegen/xai-grok-pager/tests/grok_home_ignore_grok_home.rs
  modified:
    - crates/codegen/xai-grok-config/src/paths.rs
    - crates/codegen/xai-grok-config/src/lib.rs
    - crates/codegen/xai-grok-pager-render/src/util.rs
    - crates/codegen/xai-grok-pager/tests/grok_home_paths.rs

key-decisions:
  - "Product home override is BUM_HOME only; GROK_HOME is never read as product home"
  - "Pure resolve_product_home takes optional OsString + PathBuf — no process env in unit tests"
  - "Kept public symbol grok_home() and OnceLock static name GROK_HOME this phase"

patterns-established:
  - "Pure helper for multi-branch path logic; OnceLock public path tested once per process"
  - "Explicit remove_var(BUM_HOME) before default-home scenarios so ambient env cannot invalidate tests"

requirements-completed: [ID-03]

coverage:
  - id: D1
    description: Pure resolve_product_home with BUM_HOME override and default .bum join
    requirement: ID-03
    verification:
      - kind: unit
        ref: crates/codegen/xai-grok-config/src/paths.rs#resolve_product_home_*
        status: pass
    human_judgment: false
  - id: D2
    description: Production SoT uses ~/.bum + BUM_HOME only; managed bin/bum leaf
    requirement: ID-03
    verification:
      - kind: unit
        ref: cargo test -p xai-grok-config --lib paths
        status: pass
    human_judgment: false
  - id: D3
    description: Process-isolated BUM_HOME override and GROK_HOME-ignore integration tests
    requirement: ID-03
    verification:
      - kind: integration
        ref: crates/codegen/xai-grok-pager/tests/grok_home_paths.rs#bum_home_override_path_helpers
        status: pass
      - kind: integration
        ref: crates/codegen/xai-grok-pager/tests/grok_home_ignore_grok_home.rs#grok_home_env_is_ignored_for_product_home
        status: pass
    human_judgment: false
  - id: D4
    description: Display path labels ~/.bum and $BUM_HOME
    requirement: ID-03
    verification:
      - kind: unit
        ref: cargo test -p xai-grok-pager-render util
        status: pass
    human_judgment: false

duration: 22min
completed: 2026-07-16
status: complete
---

# Phase 1 Plan 01: Product Home SoT Summary

**Default product home cut over to `~/.bum` with pure `resolve_product_home`, `BUM_HOME`-only override, `bin/bum` managed leaf, and process-isolated path tests**

## Performance

- **Duration:** 22 min
- **Started:** 2026-07-16T02:23:19Z
- **Completed:** 2026-07-16T02:45:01Z
- **Tasks:** 3
- **Files modified:** 5

## Accomplishments

- Extracted pure `resolve_product_home` (no process env / OnceLock) and wired `default_grok_home` / `grok_home` / `user_grok_home` to `BUM_HOME` only
- Default home ends with `.bum`; managed application path is `bin/bum` (`bum.exe` on Windows)
- Display helpers label default as `~/.bum` and override as `$BUM_HOME`
- Two process-isolated pager integration binaries: BUM_HOME override, and GROK_HOME trap + explicit `remove_var("BUM_HOME")`
- Mandatory crate docs in `xai-grok-config` describe `$BUM_HOME` / `~/.bum` product root

## Task Commits

Each task was committed atomically:

1. **Task 1: Pure product-home resolver + SoT cutover + unit tests** - `66aad62` (feat)
2. **Task 2: Process-isolated binary tests** - `d66787e` (test)
3. **Task 3: Display home path labels for bum** - `e9597fd` (feat)

**Plan metadata:** `693d06c` (docs: complete plan)

## Files Created/Modified

- `crates/codegen/xai-grok-config/src/paths.rs` — pure resolver, SoT cutover, unit tests
- `crates/codegen/xai-grok-config/src/lib.rs` — crate docs for `$BUM_HOME` / `~/.bum`
- `crates/codegen/xai-grok-pager-render/src/util.rs` — display labels + unit tests
- `crates/codegen/xai-grok-pager/tests/grok_home_paths.rs` — BUM_HOME override binary
- `crates/codegen/xai-grok-pager/tests/grok_home_ignore_grok_home.rs` — GROK_HOME ignore binary

## Decisions Made

- Followed locked D-HOME: `BUM_HOME` only; never dual-honor `GROK_HOME`
- Pure helper for multi-scenario tests; no multi-env OnceLock scenarios in one process
- Left `system_config_dir()` as `/etc/grok` and OnceLock static name `GROK_HOME` unchanged (D-SCOPE / plan)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] None-user-home fallback test expected `./.bum` but canonicalize yields absolute cwd**
- **Found during:** Task 1 (pure helper unit tests)
- **Issue:** `dunce::canonicalize(".")` resolves to absolute path before joining `.bum`, matching production shape of historical `default_grok_home`
- **Fix:** Assert against canonicalize-then-join expected path instead of literal `./.bum`
- **Files modified:** `crates/codegen/xai-grok-config/src/paths.rs`
- **Verification:** `cargo test -p xai-grok-config --lib paths` green
- **Committed in:** `66aad62` (Task 1)

---

**Total deviations:** 1 auto-fixed (1 bug assertion alignment)
**Impact on plan:** No scope change; production precedence unchanged.

## Issues Encountered

- Host lacked Rust toolchain and `protoc`/dotslash; installed rustup 1.92.0, `dotslash`, and `protobuf-compiler` to run verification
- Full `xai-grok-pager` dependency graph is heavy (first compile multi-minute); subsequent test runs reuse artifacts

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Product home SoT ready for twin `xai_fast_worktree` lockstep (plan 01-02) and remaining writers
- Public `grok_home()` symbol retained; callers pick up `~/.bum` automatically
- Binary rename / harness env is out of scope for this plan (other Phase 1 plans)

## Known Stubs

None.

## Threat Flags

None beyond plan threat model (T-01-01..03 mitigated by BUM_HOME-only + labels).

## Self-Check: PASSED

- FOUND: `crates/codegen/xai-grok-config/src/paths.rs` (contains `BUM_HOME`, `resolve_product_home`, `.bum`)
- FOUND: `crates/codegen/xai-grok-config/src/lib.rs` (contains `BUM_HOME`)
- FOUND: `crates/codegen/xai-grok-pager-render/src/util.rs` (contains `~/.bum`)
- FOUND: `crates/codegen/xai-grok-pager/tests/grok_home_paths.rs`
- FOUND: `crates/codegen/xai-grok-pager/tests/grok_home_ignore_grok_home.rs`
- FOUND: commit `66aad62`
- FOUND: commit `d66787e`
- FOUND: commit `e9597fd`
- Tests: config paths 19/19; pager-render util 61/61; grok_home_paths 1/1; grok_home_ignore_grok_home 1/1

---
*Phase: 01-product-identity-isolated-home*
*Completed: 2026-07-16*
