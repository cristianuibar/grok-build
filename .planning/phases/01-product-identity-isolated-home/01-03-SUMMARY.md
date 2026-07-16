---
phase: 01-product-identity-isolated-home
plan: 03
subsystem: infra
tags: [binary, cargo, bum, test-harness, pty-harness, D-BIN, ID-01]

requires: []
provides:
  - "Shipped [[bin]] name bum on xai-grok-pager-bin (package name unchanged)"
  - "CARGO_BIN_EXE_bum / target/debug/bum resolution in test-support and pty-harness"
affects:
  - 01-product-identity-isolated-home
  - test harnesses
  - local cargo run / CI binary artifact name

tech-stack:
  added: []
  patterns:
    - "Single user-facing cargo bin target bum; crate/package names stay xai-grok-*"
    - "Harness resolution: GROK_BINARY|PAGER_BINARY → CARGO_BIN_EXE_bum → cargo build -p xai-grok-pager-bin --bin bum"

key-files:
  created: []
  modified:
    - crates/codegen/xai-grok-pager-bin/Cargo.toml
    - crates/codegen/xai-grok-test-support/src/env.rs
    - crates/codegen/xai-grok-pager-pty-harness/src/env.rs
    - crates/codegen/xai-grok-pager-pty-harness/src/bin/pty_scenario.rs
    - crates/codegen/xai-grok-pager-pty-harness/src/bin/scroll_matrix.rs

key-decisions:
  - "D-BIN: sole [[bin]] is bum — no dual grok/xai-grok-pager alias in v1"
  - "Keep GROK_BINARY env override name this phase (D-SCOPE non-home knobs)"
  - "Function name grok_binary() retained; returns bum path"

patterns-established:
  - "cargo run -p xai-grok-pager-bin --bin bum is the local dev invocation"
  - "CARGO_BIN_EXE_bum is the cargo-test auto env for the composition-root binary"

requirements-completed: [ID-01]

coverage:
  - id: D1
    description: "cargo build -p xai-grok-pager-bin --bin bum produces invocable bum; package default-run is bum"
    requirement: ID-01
    verification:
      - kind: other
        ref: "cargo build -p xai-grok-pager-bin --bin bum"
        status: pass
      - kind: other
        ref: "cargo metadata --no-deps | rg '\"name\":\"bum\"'"
        status: pass
      - kind: other
        ref: "cargo build -p xai-grok-pager-bin --bin xai-grok-pager (must fail)"
        status: pass
    human_judgment: false
  - id: D2
    description: "test-support resolves bum via GROK_BINARY, CARGO_BIN_EXE_bum, or local pager-bin build"
    requirement: ID-01
    verification:
      - kind: other
        ref: "cargo check -p xai-grok-test-support"
        status: pass
    human_judgment: false
  - id: D3
    description: "PTY harness resolves bum; no CARGO_BIN_EXE_xai-grok-pager / --bin xai-grok-pager in test-support, pty-harness, or pager-bin"
    requirement: ID-01
    verification:
      - kind: other
        ref: "cargo check -p xai-grok-pager-pty-harness"
        status: pass
      - kind: other
        ref: "! rg CARGO_BIN_EXE_xai-grok-pager|--bin xai-grok-pager (scoped harness packages)"
        status: pass
    human_judgment: false

duration: 21min
completed: 2026-07-16
status: complete
---

# Phase 1 Plan 03: Ship binary as bum Summary

**Composition-root `[[bin]]` renamed to `bum` with test-support and PTY harness resolution on `CARGO_BIN_EXE_bum` / `target/debug/bum` (crate names unchanged)**

## Performance

- **Duration:** 21 min
- **Started:** 2026-07-16T02:24:08Z
- **Completed:** 2026-07-16T02:44:52Z
- **Tasks:** 3
- **Files modified:** 5

## Accomplishments

- Shipped sole user-facing cargo bin target `bum` on package `xai-grok-pager-bin` (`default-run = "bum"`)
- Fixed test-support local build package (`-p xai-grok-pager-bin` not `-p xai-grok-pager`) and artifact name
- PTY harness + scenario CLIs resolve `CARGO_BIN_EXE_bum` with no dual-bin alias

## Task Commits

Each task was committed atomically:

1. **Task 1: Ship [[bin]] as bum** - `cc774a5` (chore/bin rename)
2. **Task 2: Test-support binary resolution for bum** - `51253b3`
3. **Task 3: PTY harness binary resolution for bum** - `4269784`

**Plan metadata:** (pending docs commit)

## Files Created/Modified

- `crates/codegen/xai-grok-pager-bin/Cargo.toml` — `default-run` / `[[bin]] name = "bum"`; comment docs for local `cargo run`
- `crates/codegen/xai-grok-test-support/src/env.rs` — `bum` path, `CARGO_BIN_EXE_bum`, build via pager-bin
- `crates/codegen/xai-grok-pager-pty-harness/src/env.rs` — same resolution order for pager_binary()
- `crates/codegen/xai-grok-pager-pty-harness/src/bin/pty_scenario.rs` — clap help `CARGO_BIN_EXE_bum`
- `crates/codegen/xai-grok-pager-pty-harness/src/bin/scroll_matrix.rs` — clap help `CARGO_BIN_EXE_bum`

## Decisions Made

- Followed D-BIN: single ship name `bum`, no `grok` / `xai-grok-pager` binary alias
- Retained `GROK_BINARY` env override and `grok_binary()` function name (D-SCOPE; behavior returns bum)
- Left pure branding/docs outside scoped packages (e.g. bash command-splitting examples, Phase 8 chrome) deferred

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- Initial `cargo build` failed because `dotslash` (for repo `bin/protoc`) was not on PATH; resolved by exporting `$HOME/.cargo/bin` so `dotslash` + toolchain are available. Not a plan deviation — environment setup for verification.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- ID-01 ship name path for this plan is complete
- Phase-level ID-01 completeness still couples to plan 01-02 (managed updater leaf)
- Downstream plans (01-04 home env cutover, 01-05 isolation proof) can assume harnesses look for `bum`
- Remaining in-repo docs/`--bin xai-grok-pager` strings outside test-support/pty-harness/pager-bin are deferred branding or test fixtures (e.g. bash_command_splitting examples)

## Self-Check: PASSED

- FOUND: crates/codegen/xai-grok-pager-bin/Cargo.toml
- FOUND: crates/codegen/xai-grok-test-support/src/env.rs
- FOUND: crates/codegen/xai-grok-pager-pty-harness/src/env.rs
- FOUND: crates/codegen/xai-grok-pager-pty-harness/src/bin/pty_scenario.rs
- FOUND: crates/codegen/xai-grok-pager-pty-harness/src/bin/scroll_matrix.rs
- FOUND: cc774a5, 51253b3, 4269784

---
*Phase: 01-product-identity-isolated-home*
*Completed: 2026-07-16*
