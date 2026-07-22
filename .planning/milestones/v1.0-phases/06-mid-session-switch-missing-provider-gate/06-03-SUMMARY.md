---
phase: 06-mid-session-switch-missing-provider-gate
plan: 03
subsystem: ui
tags: [model-switch, missing-provider, deferred-login, Login-now, external-cli-poll]

requires:
  - phase: 06-mid-session-switch-missing-provider-gate
    provides: Gate-open DeferredModelSwitch + MissingProvider QuestionView (Plan 02)
  - phase: 06-mid-session-switch-missing-provider-gate
    provides: Dual-slot provider_auth cache + RefreshProviderAuthStatus (Plan 04)
provides:
  - Login now consumes gate-open DeferredModelSwitch preserving persist_default
  - Provider-scoped recovery (xAI Authenticate vs Codex CLI-primary)
  - try_apply_deferred_model_switch_if_ready on AuthComplete + status refresh
  - Bounded external CLI poll with generation cancel
  - Live-session settings Login now → retry with persist_default true E2E
affects:
  - 06-06 verification of full MOD-06 recovery loop

tech-stack:
  added: []
  patterns:
    - "Login now reuses Plan 02 gate-open DeferredModelSwitch (never force persist_default false)"
    - "Codex Login now is CLI-primary — no xAI OAuth substitute (T-06-09)"
    - "try_apply only when required_provider slot usable on AppView dual-slot cache"
    - "Bounded poll + generation stamp on RefreshProviderAuthStatus for stale cancel"

key-files:
  created: []
  modified:
    - crates/codegen/xai-grok-pager/src/app/actions.rs
    - crates/codegen/xai-grok-pager/src/app/app_view.rs
    - crates/codegen/xai-grok-pager/src/app/dispatch/session/lifecycle.rs
    - crates/codegen/xai-grok-pager/src/app/dispatch/auth.rs
    - crates/codegen/xai-grok-pager/src/app/dispatch/task_result.rs
    - crates/codegen/xai-grok-pager/src/app/effects/mod.rs
    - crates/codegen/xai-grok-pager/src/app/event_loop.rs
    - crates/codegen/xai-grok-pager/src/app/dispatch/tests/task_result.rs
    - crates/codegen/xai-grok-pager/src/app/dispatch/tests/mod.rs

key-decisions:
  - "ProviderLoginPoll lives on AppView (generation + remaining) rather than AgentSession to avoid mass struct churn"
  - "RefreshProviderAuthStatus carries Option<generation> so badge paths stay untagged and polls are cancelable"
  - "Codex Login now never calls dispatch_login; xAI reuses existing mid-session Authenticate"

patterns-established:
  - "Gate-open stash → Login now consume → arm poll → AuthComplete/refresh try_apply → SwitchModel(persist_default)"
  - "Stale generation TaskResult updates cache only; no SwitchModel"
  - "Keep current / apply success clear poll and bump generation"

requirements-completed: [MOD-06, MOD-03]

coverage:
  - id: D1
    description: Login now consumes DeferredModelSwitch with provider + persist_default; Codex no xAI OAuth; emits RefreshProviderAuthStatus
    requirement: MOD-06
    verification:
      - kind: unit
        ref: cargo test -p xai-grok-pager --lib p6_login_now
        status: pass
    human_judgment: false
  - id: D2
    description: AuthComplete + external status refresh apply deferred only when required slot usable; stale generation ignored
    requirement: MOD-06
    verification:
      - kind: unit
        ref: cargo test -p xai-grok-pager --lib p6_auth_
        status: pass
      - kind: unit
        ref: cargo test -p xai-grok-pager --lib p6_external_cli
        status: pass
      - kind: unit
        ref: cargo test -p xai-grok-pager --lib p6_refresh_generation
        status: pass
    human_judgment: false
  - id: D3
    description: Live-session settings Login now retry persists default; Keep current zero-persist; FocusGained emission
    requirement: MOD-03
    verification:
      - kind: unit
        ref: cargo test -p xai-grok-pager --lib p6_live_session
        status: pass
      - kind: unit
        ref: cargo test -p xai-grok-pager --lib p6_focus_gained
        status: pass
    human_judgment: false

duration: 13min
completed: 2026-07-16
status: complete
---

# Phase 6 Plan 03: Login now recovery & external CLI deferred apply Summary

**Login now reuses the Plan 02 gate-open DeferredModelSwitch (including persist_default), arms provider-scoped recovery (xAI OAuth vs Codex CLI-primary), and auto-retries SwitchModel when AuthComplete or external status refresh marks the required provider usable — with bounded poll generation cancel for stale refreshes.**

## Performance

- **Duration:** ~13 min
- **Started:** 2026-07-16T23:38:50Z
- **Completed:** 2026-07-16T23:51:55Z
- **Tasks:** 2/2
- **Files modified:** 9

## Accomplishments

- Login now **consumes** gate-open `DeferredModelSwitch` without forcing `persist_default: false`
- Codex Login now is CLI-primary (scrollback + poll); never starts xAI `Authenticate`
- `try_apply_deferred_model_switch_if_ready` on AuthComplete and `ProviderAuthStatusRefreshed`
- Bounded poll (`ScheduleProviderLoginPoll`, max 30 × 4s) with generation cancel
- FocusGained tags refresh with active poll generation while awaiting
- Live-session settings → MissingProvider → Login now → usable → Ok: one `default_model` PersistSetting
- Keep current zero-persist + generation bump (Plan 02 regression retained)

## Task Commits

1. **Tasks 1–2: Login now recovery + AuthComplete/external apply** - `5963a03` (feat)

Tasks were implemented together (shared Effect generation field, poll state, try_apply, and test suite) in a single atomic feat commit.

## Files Created/Modified

- `actions.rs` — `RefreshProviderAuthStatus { generation }`, `ScheduleProviderLoginPoll`, tagged TaskResults
- `app_view.rs` — `ProviderLoginPoll`, arm/clear/generation helpers, FocusGained tags poll gen
- `lifecycle.rs` — Login now body + `try_apply_deferred_model_switch_if_ready`
- `auth.rs` — mid-session/welcome AuthComplete calls try_apply; dispatch_login visibility for lifecycle
- `task_result.rs` — refresh apply + poll tick + AuthFailed optional toast
- `effects/mod.rs` — generation passthrough + schedule poll sleep
- `event_loop.rs` — startup refresh uses `generation: None`
- `dispatch/tests/task_result.rs` — full `p6_login_now_*` / `p6_auth_*` / external / focus / generation suite
- `dispatch/tests/mod.rs` — test AppView poll fields

## Decisions Made

- **AppView poll state** rather than AgentSession fields (fewer construction sites; poll is app-global I/O)
- **Optional generation on refresh Effect** — badge/auth paths pass `None` (always may try_apply); polls pass `Some`
- **Codex never uses dispatch_login** — RESEARCH A1 / T-06-09
- **`gen` is a reserved keyword in edition 2024** — tests use `poll_generation`

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Critical] Edition 2024 reserves `gen` identifier**
- **Found during:** Task 1 compile
- **Issue:** `let gen = ...` failed to compile under edition 2024
- **Fix:** Renamed to `poll_generation` / `poll_gen` patterns
- **Files modified:** `task_result.rs` tests + handler
- **Commit:** `5963a03`

**2. [Rule 1 - Bug] Live-session E2E asserted app.models without catalog entry**
- **Found during:** Task 2 test
- **Issue:** `set_default_model_inner` only mirrors `app.models` when target is in `available`
- **Fix:** Seed `app.models.available` with target in the E2E test
- **Files modified:** `dispatch/tests/task_result.rs`
- **Commit:** `5963a03`

## Known Stubs

None — Login now recovery body is fully wired (Plan 02 stub closed).

## Threat Flags

None new beyond plan threat model. Mitigations applied:
- T-06-09: Codex path never starts xAI OAuth
- T-06-11b: wrong-provider usable does not apply
- T-06-11c: max poll attempts + generation cancel

## Verification

```
cargo test -p xai-grok-pager --lib -- p6_login_now p6_deferred p6_keep_current p6_auth_ \
  p6_external_cli p6_focus_gained p6_refresh_generation p6_login_failed \
  p6_session_created_applies p6_live_session p6_missing_provider p6_provider_auth
# 31 passed
cargo check -p xai-grok-pager  # green
```

## Self-Check: PASSED

- SUMMARY path exists at `.planning/phases/06-mid-session-switch-missing-provider-gate/06-03-SUMMARY.md`
- Commit `5963a03` found on main
- must_haves covered: Login now consume, AuthComplete/external apply, FocusGained emission, generation cancel, live-session persist_default true, Keep current zero-persist
