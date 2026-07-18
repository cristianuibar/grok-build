---
phase: 10-codex-responses-wire-parity
plan: "07"
subsystem: shell-test-fixtures
tags: [rust, sampler-config, fixtures, provider-routing]
requires:
  - phase: 10-06
    provides: "Disabled ResponsesWireProfile compatibility at shell production and support constructors."
provides:
  - "Explicit disabled ResponsesWireProfile values at every planned shell test and fixture SamplerConfig literal."
  - "A compiling shell fixture baseline before cross-provider history and trusted-header work."
affects: [10-03-cross-provider-history-sanitizer, 10-04-trusted-shell-activation, OPS-04]
tech-stack:
  added: []
  patterns:
    - "Shell test fixtures explicitly select ResponsesWireProfile::Disabled until a dedicated trusted OAuth test opts in."
key-files:
  created: []
  modified:
    - crates/codegen/xai-grok-shell/tests/common/mod.rs
    - crates/codegen/xai-grok-shell/tests/provider_routing.rs
    - crates/codegen/xai-grok-shell/src/session/helpers/session_compact.rs
    - crates/codegen/xai-grok-shell/src/session/acp_session_tests/auth_error_no_retry_tests.rs
    - crates/codegen/xai-grok-shell/src/session/acp_session_tests/cancel_running_task_tests.rs
    - crates/codegen/xai-grok-shell/src/agent/subagent/tests/mod.rs
key-decisions:
  - "Keep every compatibility fixture explicitly disabled; no test implicitly selects trusted Codex behavior."
  - "Preserve models, providers, credentials, backends, headers, retries, timing, and assertions unchanged."
patterns-established:
  - "Full test fixture literals name the typed profile next to auth/backend fields, including default-preserving subagent fixtures."
requirements-completed: [OPS-04]
coverage:
  - id: D1
    description: "Shared and provider-routing fixture construction remains field-complete with trusted Responses behavior disabled."
    requirement: OPS-04
    verification:
      - kind: compile
        ref: cargo test -p xai-grok-shell --test provider_routing --no-run
        status: pass
    human_judgment: false
  - id: D2
    description: "Session compaction, authentication, cancellation, and subagent test fixtures compile with the typed profile disabled."
    requirement: OPS-04
    verification:
      - kind: compile
        ref: cargo test -p xai-grok-shell --lib --no-run
        status: pass
    human_judgment: false
duration: ~20min
completed: 2026-07-18
status: complete
---

# Phase 10 Plan 07: Shell Fixture Compatibility Summary

**All planned shell test and fixture sampler configurations now explicitly preserve the disabled Responses wire profile, keeping trusted Codex behavior opt-in.**

## Performance

- **Duration:** ~20 min, including the first shell test-target compilation.
- **Tasks:** 3/3
- **Files modified:** 6

## Accomplishments

- Updated the shared mock configuration and provider-routing helper without changing their route, credential, header, or backend semantics.
- Preserved compaction, BYOK-auth, cancellation, and persistence test setup while making the new typed field explicit.
- Kept all six subagent reasoning-effort fixtures disabled even though their default-preserving struct updates would already inherit the safe default.

## Task Commits

1. **Task 1: Update shared and provider-routing test fixtures with disabled profile compatibility** - `720788d` (test)
2. **Task 2: Update session-compaction, auth-error, and cancellation test literals without weakening their assertions** - `f052a07` (test)
3. **Task 3: Update subagent test support and verify the shell fixture matrix compiles** - `0d4bed9` (test)

## Files Created/Modified

- `crates/codegen/xai-grok-shell/tests/common/mod.rs` - Keeps the shared mock helper disabled.
- `crates/codegen/xai-grok-shell/tests/provider_routing.rs` - Keeps provider-route probes disabled.
- `crates/codegen/xai-grok-shell/src/session/helpers/session_compact.rs` - Keeps compaction sampling generic.
- `crates/codegen/xai-grok-shell/src/session/acp_session_tests/auth_error_no_retry_tests.rs` - Keeps BYOK error-path setup generic.
- `crates/codegen/xai-grok-shell/src/session/acp_session_tests/cancel_running_task_tests.rs` - Keeps persistence and cancellation setups generic.
- `crates/codegen/xai-grok-shell/src/agent/subagent/tests/mod.rs` - Keeps subagent reasoning-fixture sampling generic.

## Decisions Made

- Used `ResponsesWireProfile::Disabled` explicitly at every planned literal, including `..Default::default()` subagent literals, so the trust boundary stays auditable by search.
- Did not add headers, routing logic, request shaping, encrypted-history handling, or trusted-profile activation. Those remain owned by later Phase 10 plans.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- `cargo fmt --all -- --check` remains red on broad pre-existing workspace formatting drift, as already documented by Plans 10-01 and 10-06. It includes unrelated crates and existing formatting differences in the large routing test file; no unrelated formatting was changed. The Plan 07 patch is whitespace-clean under `git diff --check`.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 10-03 can compile and exercise shell-side provider-transition sanitation against field-complete fixtures.
- Plan 10-04 remains the sole owner of trusted first-party Codex OAuth activation and headers.

## Self-Check: PASSED

- Verified all six planned fixture files contain the explicit disabled profile field.
- Verified task commits `720788d`, `f052a07`, and `0d4bed9` are present in git history.
- Verified both required shell compilation commands exited successfully.
- No placeholder, TODO, URL heuristic, header behavior, or trusted-profile activation was introduced.

---
*Phase: 10-codex-responses-wire-parity*
*Completed: 2026-07-18*
