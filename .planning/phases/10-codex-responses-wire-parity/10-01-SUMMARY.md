---
phase: 10-codex-responses-wire-parity
plan: "01"
subsystem: sampler
tags: [rust, responses-api, codex, request-serialization]
requires:
  - phase: 09-daily-driver-end-to-end-validation
    provides: "Generic Responses system-to-instructions conversion and the Codex wire failure evidence."
provides:
  - "A disabled-by-default typed Responses wire profile carried from SamplerConfig into client defaults."
  - "Trusted Codex request shaping applied after provider-agnostic conversation conversion."
affects: [10-02-sse-fallback, 10-04-trusted-shell-activation, OPS-04]
tech-stack:
  added: []
  patterns:
    - "Shell-owned trust is represented as an explicit sampler capability, never inferred from a URL."
    - "Trusted Responses policy is applied after generic conversation conversion."
key-files:
  created: []
  modified:
    - crates/codegen/xai-grok-sampler/src/config.rs
    - crates/codegen/xai-grok-sampler/src/client.rs
    - crates/codegen/xai-grok-sampler/src/actor/state.rs
    - crates/codegen/xai-grok-sampler/src/lib.rs
    - crates/codegen/xai-grok-sampler/tests/test_actor.rs
key-decisions:
  - "Keep generic store/include/stream defaults unchanged; only the trusted profile removes concise reasoning summary and injects tool controls."
  - "Profile activation remains shell-owned for Plan 10-04; sampler code receives no endpoint or credential heuristic."
patterns-established:
  - "ResponsesWireProfile::Disabled is explicit in full sampler configuration literals and inherited through partial literals."
  - "Profile-on/profile-off JSON tests prove trusted behavior without changing generic callers."
requirements-completed: [OPS-04]
coverage:
  - id: D1
    description: "Typed Responses wire profile defaults safely, survives serde omission, and reaches private client defaults."
    requirement: OPS-04
    verification:
      - kind: unit
        ref: crates/codegen/xai-grok-sampler/src/client.rs#responses_wire_profile_defaults_and_propagates_to_client_defaults
        status: pass
      - kind: unit
        ref: crates/codegen/xai-grok-sampler/src/config.rs#config_without_responses_wire_profile_deserializes_to_disabled
        status: pass
    human_judgment: false
  - id: D2
    description: "Trusted Codex streaming requests serialize explicit privacy, continuity, reasoning, and tools-only controls while generic callers retain existing defaults."
    requirement: OPS-04
    verification:
      - kind: unit
        ref: crates/codegen/xai-grok-sampler/src/client.rs#trusted_codex_responses_profile_on_off_serializes_exactly
        status: pass
    human_judgment: false
  - id: D3
    description: "Responses conversion continues to lift system history into instructions and never emits a system role in input."
    requirement: OPS-04
    verification:
      - kind: unit
        ref: crates/codegen/xai-grok-sampling-types/src/conversation.rs#responses_api_never_emits_system_role_in_input
        status: pass
    human_judgment: false
duration: 11min
completed: 2026-07-18
status: complete
---

# Phase 10 Plan 01: Typed Responses Wire Profile Summary

**A disabled typed Codex Responses profile now reaches the sampler and produces trust-gated, tool-aware wire JSON without altering generic conversion behavior.**

## Performance

- **Duration:** 11 min
- **Started:** 2026-07-18T08:00:00Z
- **Completed:** 2026-07-18T08:11:00Z
- **Tasks:** 3/3
- **Files modified:** 5

## Accomplishments

- Added `ResponsesWireProfile::{Disabled, TrustedCodex}` with a serde-safe disabled default and propagated it into `ClientDefaults`.
- Preserved sampler actor and integration fixture behavior until a later shell trust gate explicitly selects the trusted profile.
- Applied trusted request policy after generic conversion: no stale response ID or concise summary, plus auto/parallel tool controls only when tools exist.
- Locked profile-on/profile-off serialized JSON and the existing no-system-role conversion invariant with focused Rust tests.

## Task Commits

1. **Task 1: Add a disabled-by-default semantic Responses wire profile and propagate it to client defaults** - `b20115e` (feat)
2. **Task 2: Update every sampler-crate configuration literal with the disabled compatibility value** - `c35fc66` (feat)
3. **Task 3: Apply the trusted profile after generic conversion and lock the serialized request contract** - `fce4072` (feat)

## Files Created/Modified

- `crates/codegen/xai-grok-sampler/src/config.rs` - Typed, serde-defaulted caller capability.
- `crates/codegen/xai-grok-sampler/src/client.rs` - Client-default propagation, trusted request shaping, and serialized wire tests.
- `crates/codegen/xai-grok-sampler/src/actor/state.rs` - Explicit disabled profile for the full actor unit fixture.
- `crates/codegen/xai-grok-sampler/src/lib.rs` - Public re-export alongside existing sampler configuration types.
- `crates/codegen/xai-grok-sampler/tests/test_actor.rs` - Explicit disabled profile for the actor integration fixture.

## Decisions Made

- Retained the current generic `store:false`, encrypted include, and stream behavior to avoid changing xAI, BYOK, or custom callers.
- Scoped only the Codex-specific differences to `TrustedCodex`: clear `previous_response_id`, omit `reasoning.summary`, and set auto/parallel controls only with converted tools.
- Kept profile activation out of the sampler; Plan 10-04 owns the trusted first-party Codex OAuth gate and headers.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Updated the actor unit fixture with the new public config field during Task 1**
- **Found during:** Task 1
- **Issue:** The exact library test compiles the actor unit module, whose full `SamplerConfig` literal could not compile until it explicitly selected the disabled profile.
- **Fix:** Added the actor fixture's disabled value with the profile implementation; Task 2 retained ownership of the integration fixture compatibility update.
- **Files modified:** `crates/codegen/xai-grok-sampler/src/actor/state.rs`
- **Verification:** `cargo test -p xai-grok-sampler --lib client::tests::responses_wire_profile_defaults_and_propagates_to_client_defaults -- --exact`
- **Committed in:** `b20115e`

**Total deviations:** 1 auto-fixed (1 blocking compile compatibility adjustment).
**Impact on plan:** No behavior or scope change; the profile remains disabled in every fixture.

## Issues Encountered

- `cargo fmt --all -- --check` remains red on pre-existing formatting drift across unrelated workspace files (for example `xai-fast-worktree`, `xai-grok-config`, pager, tools, and update crates). It was already failing before this plan's edits and was not reformatted here. The changed test code was individually formatted; `git diff --check`, focused tests, sampler test-target compilation, and `cargo clippy -p xai-grok-sampler --lib` passed.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 10-04 can select `TrustedCodex` only after its existing first-party OAuth trust gate succeeds and can inject trusted-only headers at the shell boundary.
- Plan 10-02 can independently repair terminal SSE text reconstruction; this plan preserves the generic request path it will use.

## Self-Check: PASSED

- Verified all five modified source/test files exist and the three task commits are present in git history.
- No placeholder or TODO-style stub was introduced by this plan.

---
*Phase: 10-codex-responses-wire-parity*
*Completed: 2026-07-18*
