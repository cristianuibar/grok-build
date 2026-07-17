---
phase: 07-cross-provider-multi-agent-orchestration
plan: 03
subsystem: shell
tags: [subagent, spawn-gate, missing-provider, reasoning-effort, cross-provider, oauth]

requires:
  - phase: 07-cross-provider-multi-agent-orchestration
    provides: p7_ green harness, dual-token fixtures, task_model_override_error regression
  - phase: 06-dual-provider-model-switch
    provides: missing_provider_gate_error, provider_slot_usable store helper, model_switch gate shape
  - phase: 04-provider-aware-routing
    provides: catalog provider field, resolve_model_override_to_config dual-key
provides:
  - Authoritative pre-pending/pre-worktree missing-provider spawn gate in handle_subagent_request
  - Pure oauth_provider_slot_usable(auth_path, provider, live_xai_auth) shared with model_switch
  - Tool reasoning_effort fail-closed (parse + unsupported model) with harness soft-skip
  - p7_spawn_missing_provider_* and p7_invalid_effort_* green filters
affects:
  - 07-04 eager Task preflight (reuses pure helpers / gate semantics)
  - 07-05 dual-token Authorization proofs
  - 07-06 phase gate / AGENT-01 regression

tech-stack:
  added: []
  patterns:
    - "resolve definition/role/resume → effective model preflight → credential gate → insert_pending → worktree"
    - "oauth_provider_slot_usable pure inputs; Codex never inferred from xAI AuthManager alone"
    - "send_pre_spawn_failure before insert_pending; send_failure after"
    - "Tool effort: parse first; hard-fail on Err or unsupported; Harness soft-skip"

key-files:
  created: []
  modified:
    - crates/codegen/xai-grok-shell/src/agent/config.rs
    - crates/codegen/xai-grok-shell/src/agent/handlers/model_switch.rs
    - crates/codegen/xai-grok-shell/src/agent/subagent/handle_request.rs
    - crates/codegen/xai-grok-shell/src/agent/subagent/mod.rs
    - crates/codegen/xai-grok-shell/src/agent/mvp_agent/subagent_coordinator.rs
    - crates/codegen/xai-grok-shell/src/test_support/lsp_runtime.rs
    - crates/codegen/xai-grok-shell/src/agent/subagent/tests/mod.rs
    - crates/codegen/xai-grok-shell/tests/cross_provider_subagent.rs

key-decisions:
  - "Authoritative gate in handle_subagent_request (Task eager preflight is Plan 04)"
  - "Pure oauth_provider_slot_usable in config.rs with explicit path + optional live xAI auth"
  - "auth_json_path_override on SubagentSpawnContext for deterministic gate tests without BUM_HOME OnceLock"
  - "Tool unsupported-effort hard-fails; Harness/role soft-skips"
  - "Tool unknown model remains existing task_model_override_error (no parent-fallback rework)"

patterns-established:
  - "Spawn gate ordering: preflight model → missing_provider_gate_error → insert_pending"
  - "Spawn error copy: Cannot spawn subagent with model … Run: bum login --provider {id}"

requirements-completed: [AGENT-02, AGENT-03, AGENT-04, AGENT-05]

coverage:
  - id: D1
    description: "Pre-pending missing-provider spawn gate blocks Codex child when Codex slot empty"
    requirement: AGENT-05
    verification:
      - kind: unit
        ref: "cargo test -p xai-grok-shell --lib p7_spawn_missing_provider"
        status: pass
      - kind: integration
        ref: "cargo test -p xai-grok-shell --test cross_provider_subagent p7_spawn_missing_provider"
        status: pass
    human_judgment: false
  - id: D2
    description: "No pending/active child and no worktree on gate failure (C2-M2)"
    requirement: AGENT-05
    verification:
      - kind: unit
        ref: "p7_spawn_missing_provider_leaves_no_pending_or_active_child / creates_neither_worktree"
        status: pass
    human_judgment: false
  - id: D3
    description: "Pure oauth_provider_slot_usable shared helper unit-tested"
    requirement: AGENT-04
    verification:
      - kind: unit
        ref: "cargo test -p xai-grok-shell --lib p7_provider_slot"
        status: pass
    human_judgment: false
  - id: D4
    description: "Tool invalid/unsupported effort fail-closed; supported Sol applies medium"
    requirement: AGENT-03
    verification:
      - kind: unit
        ref: "cargo test -p xai-grok-shell --lib p7_invalid_effort"
        status: pass
    human_judgment: false
  - id: D5
    description: "Tool unknown model existing reject regression"
    requirement: AGENT-02
    verification:
      - kind: unit
        ref: "p7_tool_unknown_model_*"
        status: pass
      - kind: integration
        ref: "cargo test -p xai-grok-shell --test cross_provider_subagent p7_tool"
        status: pass
    human_judgment: false

duration: 32min
completed: 2026-07-17
status: complete
---

# Phase 7 Plan 03: Pre-pending spawn gate + Tool effort fail-closed Summary

**Authoritative shell spawn path: missing-provider credential gate runs after effective child model preflight and before insert_pending/worktree; pure oauth_provider_slot_usable shared with model_switch; Tool effort fail-closed with supported/unsupported fixtures.**

## Performance

- **Duration:** 32 min
- **Started:** 2026-07-17T08:24:55Z
- **Completed:** 2026-07-17T08:56:46Z
- **Tasks:** 2/2
- **Files modified:** 8

## Accomplishments

- Moved spawn credential gate before `insert_pending` and worktree create/rehydrate (C2-M2 / D-05)
- Extracted pure `oauth_provider_slot_usable(auth_path, provider, live_xai_auth)` used by model_switch and subagent
- Tool `reasoning_effort` parse/unsupported hard-fail via `apply_subagent_reasoning_effort`; harness soft-skip retained
- Green `p7_spawn_missing_provider*`, `p7_provider_slot*`, `p7_invalid_effort*`, `p7_tool*` filters

## Task Commits

1. **Task 1: GREEN — pure provider_slot_usable + pre-pending/pre-worktree missing-provider spawn gate** - `604e46a`
2. **Task 2: GREEN — Tool effort fail-closed + unknown-model existing-reject regression** - `c275bd9` (tests; production effort path landed with Task 1 in `handle_request.rs`)

**Plan metadata:** (docs commit after this SUMMARY)

## Files Created/Modified

- `crates/codegen/xai-grok-shell/src/agent/config.rs` — `oauth_provider_slot_usable`, `missing_provider_spawn_error_message`, unit tests
- `crates/codegen/xai-grok-shell/src/agent/handlers/model_switch.rs` — thin wrapper over pure helper
- `crates/codegen/xai-grok-shell/src/agent/subagent/handle_request.rs` — preflight + gate before pending; effort fail-closed
- `crates/codegen/xai-grok-shell/src/agent/subagent/mod.rs` — `auth_json_path_override` for deterministic tests
- `crates/codegen/xai-grok-shell/src/agent/mvp_agent/subagent_coordinator.rs` — field default `None`
- `crates/codegen/xai-grok-shell/src/test_support/lsp_runtime.rs` — field default
- `crates/codegen/xai-grok-shell/src/agent/subagent/tests/mod.rs` — production-path gate + effort tests
- `crates/codegen/xai-grok-shell/tests/cross_provider_subagent.rs` — composition harness for Plan 03 filters

## Decisions Made

- Gate authority stays in shell `handle_subagent_request` (Plan 04 adds eager Task preflight for UX timing only)
- Pure helper lives in `config.rs` next to `missing_provider_gate_error` (not MvpAgent-private)
- Test-only `auth_json_path_override` avoids BUM_HOME OnceLock contamination for full-path unit tests
- Explicit Tool effort on models without support hard-fails; role/harness soft-skip unchanged
- Did not rework Tool unknown-model parent-fallback (already rejected by `task_model_override_error`)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing critical functionality] auth_json_path_override for gate testability**
- **Found during:** Task 1
- **Issue:** Full `handle_subagent_request` gate tests would read `$BUM_HOME/auth.json` (OnceLock), flaky against developer credentials
- **Fix:** Optional `SubagentSpawnContext.auth_json_path_override`; production leaves `None` → grok_home path
- **Files modified:** `subagent/mod.rs`, `handle_request.rs`, `subagent_coordinator.rs`, `lsp_runtime.rs`
- **Commit:** `604e46a`

**2. [Rule 1 - Bug] usable-slot allow test matched unrelated "no usable api_key" sampling error**
- **Found during:** Task 1 verify
- **Issue:** Assertion used `err.contains("no usable")` which matched post-gate sampling credential errors
- **Fix:** Assert absence of spawn-gate phrases only (`bum login --provider`, `Cannot spawn subagent with model`)
- **Files modified:** `subagent/tests/mod.rs`
- **Commit:** `c275bd9`

## Known Stubs

None — gate and effort paths are fully wired for Tool provenance.

## Threat Flags

None new beyond plan threat_model (T-07-04..T-07-12 mitigated by pre-pending gate + pure catalog provider).

## Verify Commands (all green)

```bash
cargo test -p xai-grok-shell --test cross_provider_subagent p7_spawn_missing_provider
cargo test -p xai-grok-shell --test cross_provider_subagent p7_spawn_same_provider
cargo test -p xai-grok-shell --lib p7_provider_slot
cargo test -p xai-grok-shell --lib p7_spawn_missing_provider
cargo test -p xai-grok-shell --lib p7_invalid_effort
cargo test -p xai-grok-shell --test cross_provider_subagent p7_tool
cargo test -p xai-grok-shell --lib reasoning_effort_explicit
```

## Self-Check: PASSED

- SUMMARY.md present
- Commits `604e46a`, `c275bd9` present
- Key symbols `oauth_provider_slot_usable`, `missing_provider_spawn_error_message`, `apply_subagent_reasoning_effort` present
