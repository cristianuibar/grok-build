---
phase: 10-codex-responses-wire-parity
plan: "06"
subsystem: shell-configuration
tags: [rust, sampler-config, codex, provider-routing]
requires:
  - phase: 10-01
    provides: "The disabled-by-default ResponsesWireProfile field on SamplerConfig."
provides:
  - "Explicit disabled ResponsesWireProfile values at every listed shell production/config-support constructor."
  - "A field-complete shell build baseline before trusted Codex OAuth activation."
affects: [10-04-trusted-shell-activation, 10-07-shell-fixture-compatibility, OPS-04]
tech-stack:
  added: []
  patterns:
    - "Full shell SamplerConfig literals explicitly select ResponsesWireProfile::Disabled until the trusted OAuth boundary opts in."
key-files:
  created: []
  modified:
    - crates/codegen/xai-grok-shell/src/tools/config.rs
    - crates/codegen/xai-grok-shell/src/agent/config.rs
    - crates/codegen/xai-grok-shell/src/session/acp_session_impl/sampler_turn.rs
    - crates/codegen/xai-grok-shell/src/auth/credential_provider.rs
    - crates/codegen/xai-grok-shell/src/agent/subagent/mod.rs
    - crates/codegen/xai-grok-shell/src/trace_classifier/mod.rs
    - crates/codegen/xai-grok-shell/src/test_support/lsp_runtime.rs
key-decisions:
  - "Keep the Responses wire profile disabled at every compatibility constructor; Plan 10-04 remains the only trusted first-party Codex OAuth activation owner."
  - "Use explicit enum values even on default-preserving literals so future readers can audit the trust boundary by search."
patterns-established:
  - "Shell production, support, and fixture constructors use the typed profile instead of URL or header sentinels."
requirements-completed: [OPS-04]
coverage:
  - id: D1
    description: "All seven planned shell production/config-support constructor paths compile with an explicit disabled Responses wire profile."
    requirement: OPS-04
    verification:
      - kind: other
        ref: cargo check -p xai-grok-shell --lib
        status: pass
    human_judgment: false
duration: 10min
completed: 2026-07-18
status: complete
---

# Phase 10 Plan 06: Shell Literal Compatibility Summary

**Every planned shell sampler configuration path now keeps the typed Responses wire profile explicitly disabled, preserving routing and header behavior until the trusted OAuth boundary activates it.**

## Performance

- **Duration:** 10 min
- **Started:** 2026-07-18T08:16:00Z
- **Completed:** 2026-07-18T08:26:34Z
- **Tasks:** 3/3
- **Files modified:** 7

## Accomplishments

- Added an explicit disabled profile to tool and catalog-derived configuration constructors without changing model, backend, credential, header, retry, or reasoning selection.
- Kept session reconstruction and credential-provider compatibility paths disabled, leaving trusted Codex headers and profile activation to Plan 10-04.
- Applied the same disabled baseline to subagent inheritance, trace classification, and the LSP runtime support fixture.

## Task Commits

1. **Task 1: Update tools and agent configuration constructors without changing routing semantics** - `4a8df63` (feat)
2. **Task 2: Update session reconstruction and credential-provider literals as compatibility-only work** - `085fe8f` (feat)
3. **Task 3: Update subagent, trace, and LSP runtime support constructors with default-preserving values** - `195c932` (feat)

## Files Created/Modified

- `crates/codegen/xai-grok-shell/src/tools/config.rs` - Keeps the fallback web-search sampler profile disabled.
- `crates/codegen/xai-grok-shell/src/agent/config.rs` - Keeps catalog-derived sampling configuration disabled.
- `crates/codegen/xai-grok-shell/src/session/acp_session_impl/sampler_turn.rs` - Keeps reconstructed session configuration disabled pending the trusted OAuth gate.
- `crates/codegen/xai-grok-shell/src/auth/credential_provider.rs` - Makes the credential-provider sampler test fixture explicit.
- `crates/codegen/xai-grok-shell/src/agent/subagent/mod.rs` - Keeps inherited child sampling configuration disabled.
- `crates/codegen/xai-grok-shell/src/trace_classifier/mod.rs` - Keeps trace-classifier side requests disabled.
- `crates/codegen/xai-grok-shell/src/test_support/lsp_runtime.rs` - Keeps the LSP runtime support fixture disabled.

## Decisions Made

- Used `ResponsesWireProfile::Disabled` explicitly at every planned constructor, including literals that already use `..SamplerConfig::default()`, so no route is silently opted into trusted Codex behavior.
- Did not add headers, provider detection, URL heuristics, encrypted-error recovery, or profile activation; those are owned by later Phase 10 plans.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- `cargo fmt --all -- --check` remains red on pre-existing workspace formatting drift outside this plan (as documented by Plan 10-01). The touched patch is whitespace-clean under `git diff --check`; no unrelated formatting was changed.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 10-04 can now enable `TrustedCodex` only at its existing first-party SessionToken OAuth boundary without field-completeness fallout.
- Plan 10-07 remains responsible for its separate shell test-fixture compatibility scope.

## Self-Check: PASSED

- Verified all seven modified source/support files and this summary exist.
- Verified task commits `4a8df63`, `085fe8f`, and `195c932` are present in git history.
- No placeholder, TODO, or behavior stub was introduced by the compatibility-only changes.

---
*Phase: 10-codex-responses-wire-parity*
*Completed: 2026-07-18*
