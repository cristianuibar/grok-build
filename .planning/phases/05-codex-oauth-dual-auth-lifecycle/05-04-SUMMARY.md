---
phase: 05-codex-oauth-dual-auth-lifecycle
plan: 04
subsystem: auth
tags: [oauth, codex, dual-auth, logout, auth-status, acp, cli, fail-closed]

requires:
  - phase: 05-codex-oauth-dual-auth-lifecycle
    provides: Public clear_provider_slot / clear_all_provider_slots + pure AuthStatusReport
  - phase: 05-codex-oauth-dual-auth-lifecycle
    provides: Wave 0 lifecycle harness RED contracts for AUTH-03/04
provides:
  - Dual-safe CLI logout (--provider / --all / bare fail-closed) with blocking disk authority
  - Path-taking run_cli_auth_status / write_cli_auth_status paste-safe dual status
  - TUI /logout fail-closed + ACP handle_logout requiring provider|all
affects:
  - 05-05 independent Codex refresh / mid-session health
  - Daily dual-session credential management UX

tech-stack:
  added: []
  patterns:
    - Blocking clear_provider_slot / clear_all_provider_slots as disk SoT for logout
    - AuthManager::remove_scope / clear only secondary in-memory invalidate after disk clear
    - Bare destructive CLI/ACP fail-closed; --all explicit atomic dual clear
    - Status returns String or injectable Write; never dumps auth.json body

key-files:
  created: []
  modified:
    - crates/codegen/xai-grok-shell/src/auth/flow.rs
    - crates/codegen/xai-grok-shell/src/auth/mod.rs
    - crates/codegen/xai-grok-shell/src/extensions/auth.rs
    - crates/codegen/xai-grok-shell/tests/auth_codex_lifecycle.rs
    - crates/codegen/xai-grok-pager-bin/src/main.rs
    - crates/codegen/xai-grok-pager/src/app/dispatch/auth.rs
    - crates/codegen/xai-grok-pager/src/slash/commands/logout.rs

key-decisions:
  - "Disk logout authority is clear_provider_slot / clear_all_provider_slots (blocking), not remove_scope"
  - "TUI /logout fail-closed (D-03 minimal) — toast points to bum logout --provider|--all; no Effect::Logout"
  - "ACP bare logout fail-closed: require provider or all=true; selective uses blocking clear APIs"
  - "run_cli_auth_status returns String; write_cli_auth_status injects Write for tests"
  - "XAI_API_KEY warning only after xAI or --all clear; never invent Codex Platform key warning"

patterns-established:
  - "logout_provider_slot / logout_all_provider_slots path-taking for OnceLock-safe tests"
  - "format_dual_logout_message UI-SPEC copy shared by CLI"
  - "Secondary AuthManager clear only after successful disk mutation"

requirements-completed: [AUTH-03, AUTH-04]

coverage:
  - id: D1
    description: Selective and atomic dual CLI logout with bare fail-closed and blocking disk SoT
    requirement: AUTH-03
    verification:
      - kind: integration
        ref: crates/codegen/xai-grok-shell/tests/auth_codex_lifecycle.rs#selective_logout_isolates
        status: pass
      - kind: integration
        ref: crates/codegen/xai-grok-shell/tests/auth_codex_lifecycle.rs#logout_all_clears_both
        status: pass
      - kind: integration
        ref: crates/codegen/xai-grok-shell/tests/auth_codex_lifecycle.rs#bare_logout_fail_closed
        status: pass
    human_judgment: false
  - id: D2
    description: Dual-provider auth status CLI with paste-safe greppable output
    requirement: AUTH-04
    verification:
      - kind: integration
        ref: crates/codegen/xai-grok-shell/tests/auth_codex_lifecycle.rs#run_cli_auth_status
        status: pass
      - kind: integration
        ref: crates/codegen/xai-grok-shell/tests/auth_codex_lifecycle.rs#auth_status_format_paste_safe
        status: pass
      - kind: integration
        ref: crates/codegen/xai-grok-pager/tests/auth_cli_parse.rs#bum_auth_status_parses
        status: pass
    human_judgment: false
  - id: D3
    description: TUI and ACP logout dual-safe (fail-closed without provider/all intent)
    requirement: AUTH-03
    verification:
      - kind: other
        ref: crates/codegen/xai-grok-pager/src/app/dispatch/auth.rs#dispatch_logout
        status: pass
      - kind: other
        ref: crates/codegen/xai-grok-shell/src/extensions/auth.rs#handle_logout
        status: pass
    human_judgment: false

duration: 10min
completed: 2026-07-16
status: complete
---

# Phase 5 Plan 04: Dual logout + auth status CLI Summary

**Blocking selective/`--all` logout and paste-safe dual `bum auth status`, with TUI/ACP fail-closed against silent dual wipe (AUTH-03, AUTH-04)**

## Performance

- **Duration:** 10 min
- **Started:** 2026-07-16T17:38:52Z
- **Completed:** 2026-07-16T17:48:52Z
- **Tasks:** 2/2
- **Files modified:** 7

## Accomplishments

- CLI logout uses blocking `clear_provider_slot` / `clear_all_provider_slots` as disk source of truth; `AuthManager::remove_scope` only secondary in-memory invalidate after xAI/`--all`
- Bare `bum logout` prints UI-SPEC usage and fails closed with zero mutation; selective isolates both directions; `--all` atomic one-lock clear
- `run_cli_auth_status` / `write_cli_auth_status` return greppable xAI-then-Codex status without secrets; wired through pager-bin
- TUI `/logout` fail-closed toast + ACP `handle_logout` requires `provider` or `all=true` (no silent dual wipe)

## Task Commits

Each task was committed atomically:

1. **Task 1: Dual logout CLI — blocking disk authority, selective, atomic --all, bare fail-closed** - `cd0ec62` (feat)
2. **Task 2: bum auth status testable I/O + TUI/ACP dual-safe logout** - `d75c801` (feat)

**Plan metadata:** `600154a` (docs: complete plan)

## Files Created/Modified

- `crates/codegen/xai-grok-shell/src/auth/flow.rs` — `logout_provider_slot`, `logout_all_provider_slots`, `run_cli_logout`/`_at_path`, `run_cli_auth_status`, `write_cli_auth_status`, UI-SPEC copy helpers
- `crates/codegen/xai-grok-shell/src/auth/mod.rs` — re-exports dual logout/status surface
- `crates/codegen/xai-grok-shell/src/extensions/auth.rs` — dual-safe ACP logout (provider|all required; blocking clear)
- `crates/codegen/xai-grok-shell/tests/auth_codex_lifecycle.rs` — AUTH-03/04 contracts GREEN
- `crates/codegen/xai-grok-pager-bin/src/main.rs` — wire Logout provider/all + Auth Status
- `crates/codegen/xai-grok-pager/src/app/dispatch/auth.rs` — fail-closed `/logout` dispatch
- `crates/codegen/xai-grok-pager/src/slash/commands/logout.rs` — dual-safe description copy

## Decisions Made

- **Disk SoT:** Blocking `clear_provider_slot` / `clear_all_provider_slots` for CLI and ACP dual logout; never treat nonblocking `remove_scope` as authoritative clear (T-05-23)
- **TUI contract (D-03 preferred minimal):** Fail-closed before ACP — `dispatch_logout` shows toast pointing at `bum logout --provider|--all` and returns no `Effect::Logout`
- **ACP contract:** Bare `x.ai/auth/logout` without `provider`/`all` returns invalid_params; selective/all use path-taking blocking clear under product `auth.json`
- **Status I/O:** Handler returns `String`; `write_cli_auth_status` for injectable `Write`; unreadable store → UI-SPEC path-label error without dumping file body
- **XAI_API_KEY note:** Only after xAI selective or `--all` paths when env set; Codex never invents Platform key warning

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] pager-bin auth status used wrong paths crate**
- **Found during:** Task 2 wiring
- **Issue:** `xai_grok_paths::grok_home` is not linked from pager-bin
- **Fix:** Use `xai_grok_shell::util::grok_home::grok_home()` (existing binary pattern)
- **Files modified:** `crates/codegen/xai-grok-pager-bin/src/main.rs`
- **Verification:** `cargo check -p xai-grok-pager-bin`
- **Committed in:** `cd0ec62`

**2. [Rule 1 - Bug] Test name collided with imported `run_cli_auth_status`**
- **Found during:** Task 2 RED→GREEN
- **Issue:** Integration test fn `run_cli_auth_status` conflicted with the public import
- **Fix:** Call fully-qualified `xai_grok_shell::auth::run_cli_auth_status` inside the test (keep filter name for plan gate)
- **Files modified:** `crates/codegen/xai-grok-shell/tests/auth_codex_lifecycle.rs`
- **Verification:** `cargo test -p xai-grok-shell --test auth_codex_lifecycle run_cli_auth_status`
- **Committed in:** `cd0ec62`

---

**Total deviations:** 2 auto-fixed (1 Rule 3, 1 Rule 1)
**Impact on plan:** Local correctness fixes only; no architectural scope change.

## Issues Encountered

- `cargo test -p xai-grok-pager --lib` has pre-existing unrelated failures (169 errors); used integration binary `auth_cli_parse` for `bum_auth_status_parses` (GREEN). Out of scope — not fixed.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- AUTH-03/04 CLI lifecycle surfaces complete; Plan 05 can rely on blocking clear APIs and dual status inspect for refresh isolation / permanent-fail cleanup
- TUI rich dual-logout chrome deferred by D-03; CLI is the selective logout path
- No remote OAuth revoke (COVERAGE OPT-OUT) remains intentional

## Known Stubs

None that block AUTH-03/04 goals. Legacy `perform_logout` retained for xAI-scope call sites but ACP bare path no longer invokes it without provider/all.

## Self-Check: PASSED

- FOUND: `crates/codegen/xai-grok-shell/src/auth/flow.rs` (logout_provider_slot, run_cli_auth_status)
- FOUND: `crates/codegen/xai-grok-shell/src/extensions/auth.rs` (handle_logout dual-safe)
- FOUND: commits `cd0ec62`, `d75c801`
- GREEN: selective_logout_isolates, logout_all_clears_both, bare_logout_fail_closed, run_cli_auth_status, auth_status_format_paste_safe, bum_auth_status_parses

---
*Phase: 05-codex-oauth-dual-auth-lifecycle*
*Completed: 2026-07-16*
