---
phase: 07-cross-provider-multi-agent-orchestration
plan: 01
subsystem: testing
tags: [p7, subagent, cross-provider, harness, nyquist, task-tool]

requires:
  - phase: 06-dual-provider-model-switch
    provides: missing_provider_gate_error, dual-slot auth fixtures, BUM_HOME sandbox patterns
  - phase: 04-provider-aware-routing
    provides: resolve_provider_route, credential_slot isolation
provides:
  - Wave 1 compile-safe green p7_ filter discovery for tools + shell
  - Dual-token BUM_HOME integration harness scaffold (cross_provider_subagent)
  - Gap-lock that Task.reasoning_effort remains None until Plan 02
  - Pure gate / route isolation anchors reused by Plans 03–06
affects:
  - 07-02 Task schema reasoning_effort + wire
  - 07-03 spawn-time missing-provider gate
  - 07-04 eager Task preflight
  - 07-05 dual-token Authorization proofs
  - 07-06 phase gate / same-provider regression

tech-stack:
  added: []
  patterns:
    - "p7_ prefix only green tests (no intentional-red under phase filter)"
    - "Fixture tokens xai-fake-token* / codex-fake-token* only"
    - "Gap-lock tests document missing product behavior without failing"

key-files:
  created:
    - crates/codegen/xai-grok-shell/tests/cross_provider_subagent.rs
  modified:
    - crates/codegen/xai-grok-tools/src/implementations/grok_build/task/mod.rs
    - crates/codegen/xai-grok-shell/src/agent/subagent/tests/mod.rs

key-decisions:
  - "No TaskToolInput.reasoning_effort references in Plan 01 — field lands in Plan 02 with tests"
  - "Tool unknown-model framed as existing task_model_override_error reject (not future parent-fallback)"
  - "Route isolation uses public resolve_provider_route; Authorization wire is Plan 05"
  - "Historical smoke name p7_wave0_harness_smoke_* kept for filter stability; plan wave: 1"

patterns-established:
  - "Phase 7 green scaffold: p7_ discovery ≥1 per crate subgroup before product waves"
  - "cross_provider_subagent binary mirrors model_switch_gate BUM_HOME OnceLock sandbox"

requirements-completed: [AGENT-01, AGENT-02, AGENT-03, AGENT-04, AGENT-05, AGENT-06]

coverage:
  - id: D1
    description: "Tools p7_ Task contracts lock model wire + effort-None gap + model schema property"
    requirement: AGENT-03
    verification:
      - kind: unit
        ref: "cargo test -p xai-grok-tools --lib p7_"
        status: pass
    human_judgment: false
  - id: D2
    description: "Shell cross_provider_subagent dual-fixture smoke + pure gate/route isolation green harness"
    requirement: AGENT-05
    verification:
      - kind: integration
        ref: "cargo test -p xai-grok-shell --test cross_provider_subagent p7_"
        status: pass
    human_judgment: false
  - id: D3
    description: "Tool unknown-model rejection regression via existing task_model_override_error"
    requirement: AGENT-02
    verification:
      - kind: unit
        ref: "cargo test -p xai-grok-shell --lib p7_tool_unknown_model"
        status: pass
    human_judgment: false

duration: 8min
completed: 2026-07-17
status: complete
---

# Phase 7 Plan 01: Wave 1 p7_ green harness Summary

**Compile-safe Nyquist scaffold: tools + shell `p7_` green filters with dual-token fixtures and Task effort-None gap lock (no product schema/gate yet).**

## Performance

- **Duration:** 8 min
- **Started:** 2026-07-17T08:15:22Z
- **Completed:** 2026-07-17T08:23:13Z
- **Tasks:** 2/2
- **Files modified:** 3

## Accomplishments

- Tools crate lists 3 green `p7_` tests: model wire, effort-None gap lock, schema includes `model` (not `reasoning_effort`)
- Shell integration harness `cross_provider_subagent` with 6 green `p7_` tests (smoke, pure gate, route isolation, empty-slot read)
- Lib unit anchor: Tool unknown-model fails closed via existing `task_model_override_error`
- Zero intentional-red under `p7_`; no `TaskToolInput.reasoning_effort` field references

## Task Commits

1. **Task 1: GREEN scaffold — Task tool p7_ contracts** - `422602f`
2. **Task 2: GREEN scaffold — shell cross_provider_subagent harness** - `3912175`

**Plan metadata:** (docs commit after this SUMMARY)

## Files Created/Modified

- `crates/codegen/xai-grok-tools/src/implementations/grok_build/task/mod.rs` — three `p7_` unit tests (capture-backend + schemars)
- `crates/codegen/xai-grok-shell/tests/cross_provider_subagent.rs` — dual-token sandbox integration harness
- `crates/codegen/xai-grok-shell/src/agent/subagent/tests/mod.rs` — `p7_tool_unknown_model_rejected_by_existing_task_model_override_error`

## Decisions Made

- Followed expected-green protocol strictly: all `p7_*` tests compile and pass on current APIs
- Did not implement Task schema field, shell spawn gate, eager preflight, or dual Authorization product work
- Tool unknown-model framed as regression of **existing** reject path (addresses review MEDIUM stale parent-fallback premise)
- Isolation stubs use public `resolve_provider_route` only; private `resolve_model_override_to_config` not called from external tests

## Product inventory for later plans (names only — not failing tests)

### Plan 02 (schema + wire)

- `p7_task_tool_input_schema_includes_reasoning_effort_property`
- `p7_reasoning_effort_medium_wires_to_runtime_overrides`
- `p7_reasoning_effort_invalid_rejects_fail_closed`
- `p7_reasoning_effort_omit_stays_none`

### Plan 03 (spawn-time gate)

- `p7_spawn_blocks_when_child_provider_slot_empty`
- `p7_spawn_allows_when_child_slot_usable`
- Pre-worktree gate ordering proof

### Plan 04 (eager Task preflight)

- `p7_task_eager_preflight_missing_provider_before_bg_spawn`

### Plan 05 (dual Authorization)

- `p7_child_request_uses_child_provider_bearer_and_base_url`
- `p7_parent_route_unchanged_after_cross_provider_spawn`

### Plan 06 (phase gate / AGENT-01)

- Same-provider spawn/resume/roles lifecycle green under `p7_`

## Deviations from Plan

None - plan executed exactly as written.

Minor note: added one extra pure fixture test `p7_empty_codex_slot_reads_none_with_xai_present` (still green scaffold; dual-document read hygiene) and the optional lib unit for Tool unknown-model as allowed by the plan.

## Known Stubs

None — harness intentionally gap-locks product behavior without stubbing UI or mock empty data paths.

## Threat Flags

None beyond plan T-07-01 mitigation (fake tokens only; no full Authorization body asserts).

## Self-Check: PASSED

- FOUND: `crates/codegen/xai-grok-tools/src/implementations/grok_build/task/mod.rs` (p7_ tests)
- FOUND: `crates/codegen/xai-grok-shell/tests/cross_provider_subagent.rs`
- FOUND: `crates/codegen/xai-grok-shell/src/agent/subagent/tests/mod.rs` (p7_ unit)
- FOUND: commit `422602f`
- FOUND: commit `3912175`
- VERIFY: tools `p7_` → 3 passed
- VERIFY: shell `--test cross_provider_subagent p7_` → 6 passed
- VERIFY: smoke `p7_wave0_harness_smoke` → 1 passed
- VERIFY: lib `p7_tool_unknown_model` → 1 passed
