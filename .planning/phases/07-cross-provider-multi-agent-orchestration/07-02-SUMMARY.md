---
phase: 07-cross-provider-multi-agent-orchestration
plan: 02
subsystem: tools
tags: [p7, task-tool, reasoning_effort, subagent, AGENT-03, AGENT-06, schema]

requires:
  - phase: 07-cross-provider-multi-agent-orchestration
    provides: p7_ green harness, Task model wire tests, effort-None gap lock
  - phase: 04-provider-aware-routing
    provides: catalog provider routing (child model validation unchanged)
provides:
  - TaskToolInput.reasoning_effort Option<String> + schemars description
  - Canonical effort wire into SubagentRuntimeOverrides (max→xhigh)
  - Fail-closed invalid effort at Task boundary before spawn
  - Full TaskToolInput literal migration (~26 sites + tool-types test)
  - Parent-facing NL effort guidance via task_effort_guidance / build_task_description
affects:
  - 07-03 spawn-time missing-provider gate (effort already on overrides)
  - 07-04 eager Task preflight
  - 07-05 dual-token Authorization proofs
  - 07-06 phase gate / AGENT-01

tech-stack:
  added: []
  patterns:
    - "Task effort parse: pure string allowlist mirroring ReasoningEffort::from_str (tools↔sampling-types cycle)"
    - "Canonical store via as_str form (max→xhigh); never store raw alias"
    - "sanitize_reasoning_effort_arg keeps none (valid effort) unlike model sanitize_optional_arg"

key-files:
  created: []
  modified:
    - crates/common/xai-tool-types/src/task.rs
    - crates/codegen/xai-grok-tools/src/implementations/grok_build/task/mod.rs
    - crates/codegen/xai-grok-agent/src/builder.rs

key-decisions:
  - "Parse effort with local allowlist equivalent to ReasoningEffort::from_str (cannot depend on sampling-types from tools — cyclic)"
  - "Preserve none/minimal as accepted tokens; do not strip none via sanitize_optional_arg"
  - "Store canonical Display/as_str tokens on overrides (max→xhigh)"
  - "Effort allowed on resume_from (model still soft-ignored/pinned)"
  - "NL guidance is description appendix only — no workflow engine"

patterns-established:
  - "Task boundary fail-closed effort: invalid_arguments + accepted token list before bg spawn"
  - "p7_ effort tests cover schema, wire (incl max alias), invalid reject, omit, blank sentinel"

requirements-completed: [AGENT-03, AGENT-06]

coverage:
  - id: D1
    description: "TaskToolInput.reasoning_effort schema + serde field for model-facing Task tool"
    requirement: AGENT-03
    verification:
      - kind: unit
        ref: "cargo test -p xai-grok-tools --lib p7_task_tool_input_schema_includes_reasoning_effort"
        status: pass
    human_judgment: false
  - id: D2
    description: "Valid effort threads as canonical tokens; max→xhigh; omit stays None; invalid reject before spawn"
    requirement: AGENT-03
    verification:
      - kind: unit
        ref: "cargo test -p xai-grok-tools --lib p7_"
        status: pass
    human_judgment: false
  - id: D3
    description: "Parent Task description documents model + reasoning_effort for NL orchestration"
    requirement: AGENT-06
    verification:
      - kind: unit
        ref: "cargo test -p xai-grok-agent --lib p7_build_task_description_includes_effort_guidance"
        status: pass
    human_judgment: false

duration: 5min
completed: 2026-07-17
status: complete
---

# Phase 7 Plan 02: TaskToolInput.reasoning_effort schema + wire Summary

**Task tool exposes optional reasoning_effort, wires canonical tokens into SubagentRuntimeOverrides (max→xhigh), rejects invalid effort fail-closed, and documents NL model+effort guidance for parent models.**

## Performance

- **Duration:** 5 min
- **Started:** 2026-07-17T08:24:55Z
- **Completed:** 2026-07-17T08:29:50Z
- **Tasks:** 2/2
- **Files modified:** 3

## Accomplishments

- `TaskToolInput.reasoning_effort: Option<String>` after `model` with schemars product-token description (D-03, D-15 resume note)
- Execute path: sanitize → allowlist parse → canonical store on `SubagentRuntimeOverrides.reasoning_effort`; invalid → `invalid_arguments` before spawn (D-04)
- Full workspace `TaskToolInput { ... }` literal migration (tools tests + tool-types test)
- Tools `p7_` suite: 7 green tests (schema, wire multi-token, max→xhigh, invalid, omit, blank sentinel, model regression)
- Agent `task_effort_guidance` + `p7_build_task_description_includes_effort_guidance` for AGENT-06 / D-13

## Task Commits

1. **Task 1: GREEN — TaskToolInput.reasoning_effort schema + full literal migration + canonical wire + invalid reject** - `ce42267`
2. **Task 2: GREEN — NL Task description effort guidance (AGENT-06 surface)** - `0057ae6`

**Plan metadata:** (docs commit after this SUMMARY)

## Files Created/Modified

- `crates/common/xai-tool-types/src/task.rs` — `reasoning_effort` field + schema text + test literal
- `crates/codegen/xai-grok-tools/src/implementations/grok_build/task/mod.rs` — parse/sanitize helpers, wire overrides, full literal migration, p7_ tests
- `crates/codegen/xai-grok-agent/src/builder.rs` — `task_effort_guidance` appended after model guidance + p7_ unit test

## Decisions Made

- Local allowlist parser in tools (mirrors `ReasoningEffort::from_str`/`as_str`) because `xai-grok-tools` → `xai-grok-sampling-types` is a **cargo cycle** (sampling-types already depends on tools)
- Custom `sanitize_reasoning_effort_arg` keeps `"none"` (valid effort) instead of `sanitize_optional_arg` which drops it as a model sentinel
- Canonical storage only (`max` → `"xhigh"`) so shell re-parse sees stable tokens
- Effort pass-through on resume; model soft-ignore/pin unchanged (D-15)
- Description appendix only for NL orchestration — no multi-step workflow instructions

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Avoid cyclic crate dependency for ReasoningEffort::from_str**
- **Found during:** Task 1
- **Issue:** Adding `xai-grok-sampling-types` to `xai-grok-tools` creates a package cycle (sampling-types depends on tools).
- **Fix:** Implemented pure-string allowlist `canonicalize_reasoning_effort_token` with identical vocabulary/aliases to `ReasoningEffort::from_str` and canonical `as_str` outputs.
- **Files modified:** `crates/codegen/xai-grok-tools/src/implementations/grok_build/task/mod.rs`, briefly `Cargo.toml` (reverted)
- **Commit:** `ce42267`

**2. [Rule 2 - Correctness] Preserve `none` effort token under sanitization**
- **Found during:** Task 1
- **Issue:** Plan text suggested `sanitize_optional_arg`, which treats `"none"` as a sentinel and would drop a valid `ReasoningEffort` token (C2-L3).
- **Fix:** `sanitize_reasoning_effort_arg` only drops empty/`null`/`undefined`; keeps `none`/`minimal`.
- **Files modified:** `task/mod.rs`
- **Commit:** `ce42267`

Otherwise plan executed as written (full literal migration, canonical wire, NL guidance, green p7_ tests).

## Known Stubs

None — product field and wire are live; no placeholder effort path.

## Threat Flags

None new beyond plan threat model (T-07-02 mitigated by allowlist + fail-closed; T-07-03 model validator still gates catalog).

## Self-Check: PASSED

- FOUND: `crates/common/xai-tool-types/src/task.rs`
- FOUND: `crates/codegen/xai-grok-tools/src/implementations/grok_build/task/mod.rs`
- FOUND: `crates/codegen/xai-grok-agent/src/builder.rs`
- FOUND: `ce42267`
- FOUND: `0057ae6`
- Verify: `cargo test -p xai-grok-tools --lib p7_` → 7 passed
- Verify: `cargo test -p xai-grok-agent --lib p7_` → 1 passed
- Verify: `cargo check -p xai-tool-types -p xai-grok-tools` → ok
