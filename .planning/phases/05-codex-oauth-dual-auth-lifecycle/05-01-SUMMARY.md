---
phase: 05-codex-oauth-dual-auth-lifecycle
plan: 01
subsystem: auth
tags: [oauth, codex, dual-auth, clap, wave0, tdd, integration-tests]

requires:
  - phase: 02-multi-slot-credentials
    provides: multi-slot auth.json providers.xai/providers.codex + path-taking readers
  - phase: 04-provider-aware-routing
    provides: PROVIDER_* constants, inject_url_derived_headers, first-party Codex URL helpers
provides:
  - Wave 0 auth_codex_lifecycle integration harness (smoke green + AUTH-02..05 RED)
  - Clap parse surface for --provider login/logout and bum auth status
  - pager-bin match-arm compile glue (xAI-only handlers until Plans 03–04)
affects:
  - 05-02 status formatter / usable semantics
  - 05-03 Codex OAuth login
  - 05-04 selective logout + status CLI
  - 05-05 independent refresh + headers
  - 05-06 GREEN consolidation

tech-stack:
  added: []
  patterns:
    - Wave 0 RED harness with intentional Plan-NN assertion messages
    - Explicit auth_file tempfile paths (no multi-BUM_HOME OnceLock races)
    - Clap ValueEnum provider wire ids + Auth { Status } subcommand early lock
    - Integration twin for pager clap tests when lib --tests pre-broken

key-files:
  created:
    - crates/codegen/xai-grok-shell/tests/auth_codex_lifecycle.rs
    - crates/codegen/xai-grok-pager/tests/auth_cli_parse.rs
    - .planning/phases/05-codex-oauth-dual-auth-lifecycle/deferred-items.md
  modified:
    - crates/codegen/xai-grok-pager/src/app/cli.rs
    - crates/codegen/xai-grok-pager/src/app/mod.rs
    - crates/codegen/xai-grok-pager-bin/src/main.rs

key-decisions:
  - "Wave 0 only: no production Codex OAuth, dual logout, status formatter, or refresher"
  - "AuthProviderArg ValueEnum (xai|codex) on Login/Logout; bare login provider=None = xAI default (D-01)"
  - "Logout fields provider Option + all bool with conflicts_with; bare parse allowed, handler fail-closed later (D-05)"
  - "Command::Auth { Status } for bum auth status (D-06); pager-bin stub bail until Plan 02/04"
  - "Option C reconstruct trio names intentionally absent from integration binary (Plan 05 --lib only)"
  - "Runnable clap verify via --test auth_cli_parse because pager lib-test target is pre-broken"

patterns-established:
  - "auth_codex_lifecycle: dual fixture + read_provider_auth_store smoke; RED contracts name Plan-NN seams"
  - "auth_cli_parse integration binary mirrors unit tests in cli.rs for CI-runnable filters"
  - "pager-bin ignores new clap fields with _ until Plans 03–04 implement bodies"

# Wave 0 scaffolds AUTH-02..05 RED contracts only — leave Pending until Plans 02–06 GREEN
requirements-completed: []

coverage:
  - id: D1
    description: Wave 0 dual-auth lifecycle integration harness with smoke green and AUTH-02..05 named RED contracts
    requirement: AUTH-02
    verification:
      - kind: integration
        ref: crates/codegen/xai-grok-shell/tests/auth_codex_lifecycle.rs#auth_codex_lifecycle_harness_smoke
        status: pass
      - kind: integration
        ref: crates/codegen/xai-grok-shell/tests/auth_codex_lifecycle.rs#--list (23 contracts; no Option C trio)
        status: pass
    human_judgment: false
  - id: D2
    description: Behavior-RED login/logout/status/refresh/header contracts (intentionally fail until Plans 02–06)
    requirement: AUTH-03
    verification:
      - kind: integration
        ref: crates/codegen/xai-grok-shell/tests/auth_codex_lifecycle.rs#codex_login_persists_slot
        status: fail
    human_judgment: false
    rationale: "Wave 0 RED is the expected gate; GREEN lands in later plans"
  - id: D3
    description: Clap parse locks for --provider login, logout --all, bum auth status; pager-bin compiles
    requirement: AUTH-04
    verification:
      - kind: integration
        ref: crates/codegen/xai-grok-pager/tests/auth_cli_parse.rs#bum_login_provider_codex_parses
        status: pass
      - kind: integration
        ref: crates/codegen/xai-grok-pager/tests/auth_cli_parse.rs#bum_auth_status_parses
        status: pass
      - kind: other
        ref: cargo check -p xai-grok-pager-bin
        status: pass
    human_judgment: false

duration: 15min
completed: 2026-07-16
status: complete
---

# Phase 5 Plan 01: Wave 0 dual-auth harness + clap scaffolds Summary

**Wave 0 RED harness `auth_codex_lifecycle` plus clap parse locks for dual-provider login/logout/status — no production Codex OAuth yet**

## Performance

- **Duration:** 15 min
- **Started:** 2026-07-16T16:57:17Z
- **Completed:** 2026-07-16T17:11:56Z
- **Tasks:** 2/2
- **Files modified:** 6 (created 3, modified 3)

## Accomplishments

- Created `tests/auth_codex_lifecycle.rs` with OnceLock/explicit-path hygiene, dual-slot smoke (green), and AUTH-02..05 behavior-RED contracts naming Plan 02–06 seams
- Reserved Option C reconstruct trio for Plan 05 `--lib` only (absent from integration `--list`)
- Locked clap surface: `Login.provider`, `Logout { provider, all }`, `Auth { Status }`; pager-bin match arms compile with xAI-only stubs
- Runnable clap filters via `tests/auth_cli_parse.rs` (mirrors unit tests in `cli.rs`)

## Task Commits

1. **Task 1: Wave 0 auth_codex_lifecycle smoke + AUTH-02..05 RED** - `d5e0a44` (test)
2. **Task 2: Clap parse scaffolds + pager-bin compile** - `9e5c33b` (feat)

**Plan metadata:** (pending final docs commit)

## Files Created/Modified

- `crates/codegen/xai-grok-shell/tests/auth_codex_lifecycle.rs` — Wave 0 dual-auth lifecycle harness
- `crates/codegen/xai-grok-pager/src/app/cli.rs` — AuthProviderArg, AuthCommand, Login/Logout/Auth clap fields + unit tests
- `crates/codegen/xai-grok-pager/src/app/mod.rs` — re-export AuthProviderArg / AuthCommand
- `crates/codegen/xai-grok-pager/tests/auth_cli_parse.rs` — runnable D-01/D-05/D-06 parse tests
- `crates/codegen/xai-grok-pager-bin/src/main.rs` — match arms for new fields; Auth Status stub
- `.planning/phases/05-codex-oauth-dual-auth-lifecycle/deferred-items.md` — pre-existing pager lib-test breakage

## Decisions Made

- Early clap fields (not deferred to Plans 03–04) so parse tests are real locks; handlers remain stubs
- Integration twin for clap tests because `cargo test -p xai-grok-pager --lib` is pre-broken (pager-render `cfg(test)` helpers)
- RED contracts use compile-safe fixtures + `assert!(false / Plan-NN)` when public seams do not exist yet
- Account-header absence tests on xAI/custom already pass (header never injected); positive Codex header stays RED until Plan 05

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Runnable clap verify path via integration binary**
- **Found during:** Task 2
- **Issue:** Plan verify `cargo test -p xai-grok-pager <name>` fails compiling the full lib-test target (~169 pre-existing errors: `xai-grok-pager-render` `#[cfg(test)]` helpers not visible to dependent crate tests)
- **Fix:** Added `tests/auth_cli_parse.rs` with the same named tests; unit tests still co-located in `cli.rs` for plan co-location; documented in `deferred-items.md`
- **Files modified:** `crates/codegen/xai-grok-pager/tests/auth_cli_parse.rs`, `src/app/mod.rs` (exports), `deferred-items.md`
- **Verification:** `cargo test -p xai-grok-pager --test auth_cli_parse <each name>` all pass; `cargo check -p xai-grok-pager-bin` green
- **Committed in:** `9e5c33b`

**Total deviations:** 1 auto-fixed (Rule 3)
**Impact on plan:** Required to make Wave 0 clap contracts executable without fixing unrelated lib-test infra. No production dual-auth scope creep.

## Issues Encountered

- Pre-existing `xai-grok-pager` lib-test compile breakage blocked plan-literal verify command; mitigated with `--test auth_cli_parse` and deferred-items log

## Known Stubs

| Location | Stub | Reason |
|----------|------|--------|
| pager-bin `Command::Login` | `provider: _` ignored | Plan 03 implements Codex/xAI dispatch |
| pager-bin `Command::Logout` | `provider`/`all` ignored; still xAI-only `run_cli_logout` | Plan 04 dual logout |
| pager-bin `Command::Auth::Status` | `anyhow::bail!(...not implemented...)` | Plan 02/04 status handler |
| auth_codex_lifecycle RED tests | intentional assert fails | Production seams Plans 02–06 |

## Threat Flags

None new beyond plan register (fake tokens only; no new network endpoints).

## Auth Gates

None.

## Next Phase Readiness

- Plans 02–06 can GREEN against named contracts in `auth_codex_lifecycle` and clap locks
- Option C reconstruct still owned by Plan 05 `--lib` narrow TESTNAME only
- Do not use unfiltered `cargo test -p xai-grok-shell --lib` as a phase gate

## Verification run (this plan)

```text
cargo test -p xai-grok-shell --test auth_codex_lifecycle -- --list   # 23 tests; no Option C names
cargo test -p xai-grok-shell --test auth_codex_lifecycle auth_codex_lifecycle_harness_smoke  # pass
cargo test -p xai-grok-shell --test auth_codex_lifecycle codex_login_persists_slot           # fail (RED)
cargo test -p xai-grok-pager --test auth_cli_parse bum_login_defaults_to_xai_without_provider_argument  # pass
cargo test -p xai-grok-pager --test auth_cli_parse bum_login_provider_codex_parses  # pass
cargo test -p xai-grok-pager --test auth_cli_parse bum_logout_all_parses  # pass
cargo test -p xai-grok-pager --test auth_cli_parse bum_auth_status_parses  # pass
cargo check -p xai-grok-pager-bin  # ok
```

## Self-Check: PASSED

- FOUND: `crates/codegen/xai-grok-shell/tests/auth_codex_lifecycle.rs`
- FOUND: `crates/codegen/xai-grok-pager/tests/auth_cli_parse.rs`
- FOUND: `crates/codegen/xai-grok-pager/src/app/cli.rs` (AuthProviderArg / Auth / Logout fields)
- FOUND: commits `d5e0a44`, `9e5c33b`

---
*Phase: 05-codex-oauth-dual-auth-lifecycle*
*Completed: 2026-07-16*
