---
phase: 05-codex-oauth-dual-auth-lifecycle
plan: 06
subsystem: auth
tags: [oauth, codex, dual-auth, ensure-fresh, reconstruct, phase-gate, AUTH-02, AUTH-03, AUTH-04, AUTH-05]

requires:
  - phase: 05-codex-oauth-dual-auth-lifecycle
    provides: Plans 01–05 dual OAuth lifecycle, multi-slot store, CLI logout/status, ensure_fresh + Option C reconstruct seam
provides:
  - Phase 5 gate fully green for AUTH-02..05 including Option C reconstruct
  - Stable concurrent ensure_fresh single-IdP-spend under full suite
  - Regression green: auth_multi_slot, provider_routing, pager auth clap
  - Deferred-scope audit clean (no Phase 6 gate UX, no stock import, no Platform key primary)
affects:
  - phase-06-missing-provider-switch-gate
  - verify-work / UAT for dual auth

tech-stack:
  added: []
  patterns:
    - "#[serial] for process-wide ensure_fresh synthetic IdP hooks"
    - "Path-scoped synthetic IdP (hooks.auth_file must match call path)"
    - "Option C AUTH-05 proof via cargo test -p xai-grok-shell --lib <TESTNAME> only"
    - "Pager clap via --test auth_cli_parse (lib-test target has pre-existing 169 compile errors)"

key-files:
  created: []
  modified:
    - crates/codegen/xai-grok-shell/tests/auth_codex_lifecycle.rs
    - crates/codegen/xai-grok-shell/src/auth/codex/ensure_fresh.rs
    - crates/codegen/xai-grok-shell/src/session/acp_session_tests/codex_reconstruct_refresh_tests.rs

key-decisions:
  - "Gate residual: serialize hook-using tests rather than rewrite ensure_fresh architecture"
  - "cargo fmt --all --check and clippy -D warnings treated non-fatal for pre-existing noise outside residual fix policy"
  - "Pager clap gate uses integration binary auth_cli_parse (same test names as cli.rs unit tests)"

patterns-established:
  - "Phase gate never uses unfiltered shell --lib; Option C always --lib + narrow TESTNAME"
  - "Concurrent IdP single-flight proven with multi_thread + serial + path-scoped synthetic"

requirements-completed: [AUTH-02, AUTH-03, AUTH-04, AUTH-05]

coverage:
  - id: D1
    description: Full auth_codex_lifecycle suite green (31 tests) including concurrent refresh, identity preserve, usable semantics, device multi-step, state-mismatch, selective logout, status
    requirement: AUTH-02
    verification:
      - kind: integration
        ref: cargo test -p xai-grok-shell --test auth_codex_lifecycle
        status: pass
    human_judgment: false
  - id: D2
    description: Option C SessionActor::reconstruct_full_config mid-session refresh + BYOK/custom endpoint contracts
    requirement: AUTH-05
    verification:
      - kind: unit
        ref: cargo test -p xai-grok-shell --lib codex_reconstruct_refreshes_mid_session_expiry
        status: pass
      - kind: unit
        ref: cargo test -p xai-grok-shell --lib codex_byok_key_not_overridden
        status: pass
      - kind: unit
        ref: cargo test -p xai-grok-shell --lib codex_oauth_bearer_absent_on_custom_endpoint
        status: pass
    human_judgment: false
  - id: D3
    description: Dual-slot storage isolation + logout/status (AUTH-03/04) covered by lifecycle + multi_slot regression
    requirement: AUTH-03
    verification:
      - kind: integration
        ref: cargo test -p xai-grok-shell --test auth_codex_lifecycle selective_logout_isolates
        status: pass
      - kind: integration
        ref: cargo test -p xai-grok-shell --test auth_multi_slot
        status: pass
    human_judgment: false
  - id: D4
    description: Auth status paste-safe dual provider listing (AUTH-04)
    requirement: AUTH-04
    verification:
      - kind: integration
        ref: cargo test -p xai-grok-shell --test auth_codex_lifecycle run_cli_auth_status
        status: pass
      - kind: integration
        ref: cargo test -p xai-grok-pager --test auth_cli_parse bum_auth_status_parses
        status: pass
    human_judgment: false
  - id: D5
    description: Provider routing MOD-04/05 + never_cross_slot regression green
    verification:
      - kind: integration
        ref: cargo test -p xai-grok-shell --test provider_routing
        status: pass
    human_judgment: false
  - id: D6
    description: Optional live smoke (browser Codex login under BUM_HOME temp)
    verification: []
    human_judgment: true
    rationale: Live IdP + browser loopback not exercisable in CI mocks (D-13 automated primary)

duration: 31min
completed: 2026-07-16
status: complete
---

# Phase 5 Plan 06: Phase gate AUTH-02..05 Summary

**Full dual OAuth lifecycle + Option C reconstruct seam + multi-slot/routing/clap regressions green; concurrent ensure_fresh IdP single-flight stabilized under full suite**

## Performance

- **Duration:** 31 min
- **Started:** 2026-07-16T18:20:30Z
- **Completed:** 2026-07-16T18:51:43Z
- **Tasks:** 2/2
- **Files modified:** 3

## Accomplishments

- Phase 5 automated map green: `auth_codex_lifecycle` 31/31 (incl. concurrent single IdP, identity preserve, device, usable, logout/status)
- AUTH-05 Option C seam green via narrow `--lib` filters (not ensure_fresh-only)
- Regressions green: `auth_multi_slot`, `provider_routing` (50), pager `auth_cli_parse` (6)
- `cargo check -p xai-grok-shell` and `-p xai-grok-pager-bin` green
- Deferred-scope audit clean; COVERAGE.md still accurate (revoke/import/Platform OPT-OUT)

## Task Commits

1. **Task 1: Full AUTH-02..05 lifecycle + Option C reconstruct seam green** - `db2e69a` (fix)
2. **Task 2: Regression matrix + clap + build/fmt/clippy + deferred-scope audit** - no code delta (audit-only; gates already green)

**Plan metadata:** (final docs commit)

## Files Created/Modified

- `crates/codegen/xai-grok-shell/tests/auth_codex_lifecycle.rs` — `#[serial]` + clear hooks at start of ensure_fresh tests
- `crates/codegen/xai-grok-shell/src/auth/codex/ensure_fresh.rs` — path-scope synthetic IdP to call path
- `crates/codegen/xai-grok-shell/src/session/acp_session_tests/codex_reconstruct_refresh_tests.rs` — `#[serial]` + clear hooks on Option C trio

## Decisions Made

- Stabilize concurrent IdP gate with `serial_test` + path-scoped synthetic rather than architectural rewrite
- Use `auth_cli_parse` integration binary for pager clap (pre-existing lib-test compile failures out of scope)
- Document pre-existing `cargo fmt --all --check` and `clippy -D warnings` noise as non-fatal (plan residual policy)

## Gate Results

| Gate | Command | Result |
|------|---------|--------|
| Lifecycle suite | `cargo test -p xai-grok-shell --test auth_codex_lifecycle` | **PASS** 31/31 (×3 stable after fix) |
| Option C reconstruct | `--lib codex_reconstruct_refreshes_mid_session_expiry` | **PASS** |
| Option C BYOK | `--lib codex_byok_key_not_overridden` | **PASS** |
| Option C custom host | `--lib codex_oauth_bearer_absent_on_custom_endpoint` | **PASS** |
| Multi-slot regression | `--test auth_multi_slot` | **PASS** 1/1 |
| Provider routing | `--test provider_routing` | **PASS** 50/50 |
| Clap login/logout/status | `--test auth_cli_parse` (full binary) | **PASS** 6/6 |
| Check shell | `cargo check -p xai-grok-shell` | **PASS** |
| Check pager-bin | `cargo check -p xai-grok-pager-bin` | **PASS** |
| fmt all | `cargo fmt --all --check` | **FAIL** pre-existing (~36 dirty files; not residual gate bugs) |
| clippy -D | `cargo clippy -p xai-grok-shell … -- -D warnings` | **FAIL** pre-existing / dep noise; not residual to AUTH gate |
| `#[ignore]` core contracts | scan lifecycle + reconstruct | **none** |
| COVERAGE.md | authorize/token/refresh/device/claims INTEGRATE; revoke/import/Platform OPT-OUT | **accurate** |
| Deferred scope | no stock `~/.codex` import path; no Platform API-key primary; no Phase 6 missing-provider UX strings | **clean** |

### Known non-gate items

- `cargo test -p xai-grok-pager --lib` still fails with ~169 pre-existing compile errors (`xai-grok-pager-render` cfg(test) helpers) — clap proven via `auth_cli_parse` integration binary
- Mass `cargo fmt --all` intentionally **not** applied (would be drive-by across non-Phase-5 crates)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Flaky concurrent ensure_fresh IdP counter under full suite**
- **Found during:** Task 1 (`codex_concurrent_refresh_single_idp_spend` counter=0 when suite ran in parallel; passed alone)
- **Issue:** Process-wide synthetic IdP hooks raced with sibling ensure_fresh tests that clear/overwrite hooks mid-flight; concurrent join saw no synthetic and returned hard-unexpired material after real IdP miss
- **Fix:** `#[serial]` on all hook-using lifecycle + reconstruct tests; clear hooks at start; path-scope synthetic to matching `auth_file`
- **Files modified:** `auth_codex_lifecycle.rs`, `ensure_fresh.rs`, `codex_reconstruct_refresh_tests.rs`
- **Verification:** full lifecycle suite ×3 green; concurrent test stable
- **Committed in:** `db2e69a`

---

**Total deviations:** 1 auto-fixed (Rule 1)
**Impact on plan:** Required for trustworthy phase gate; no architecture or Phase 6 scope creep

## Issues Encountered

- Full-suite race on global test hooks (fixed)
- Pager `--lib` test target unusable for clap (pre-existing; used integration binary)
- `fmt --all --check` / `clippy -D warnings` blocked by pre-existing noise (documented non-fatal per plan)

## Auth Gates

None (no external service login required for automated gate)

## Known Stubs

None that block Phase 5 goals

## Human verification notes (optional live smoke)

1. `export BUM_HOME=$(mktemp -d)` (or equivalent temp product home)
2. `bum login --provider codex` (browser ChatGPT OAuth)
3. `bum auth status` — Codex logged_in, no raw tokens in paste-safe output
4. Optionally also log into xAI; then `bum logout --provider codex` — xAI untouched
5. Why human: live IdP + browser loopback not mockable in CI

## Next Phase Readiness

- Phase 5 sealed: dual OAuth lifecycle automated green including mid-session reconstruct (AUTH-05 Option C)
- Ready for Phase 6 missing-provider switch gate UX
- No blockers from this plan

## Self-Check: PASSED

- [x] SUMMARY path exists: `.planning/phases/05-codex-oauth-dual-auth-lifecycle/05-06-SUMMARY.md`
- [x] Commit `db2e69a` present on branch
- [x] Gate table results match last local run
- [x] requirements AUTH-02..05 listed for mark-complete

---
*Phase: 05-codex-oauth-dual-auth-lifecycle*
*Completed: 2026-07-16*
