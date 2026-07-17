---
phase: 07-cross-provider-multi-agent-orchestration
plan: 04
subsystem: tools
tags: [p7, task-tool, preflight_spawn, SubagentBackend, AGENT-05, cross-provider, async]

requires:
  - phase: 07-cross-provider-multi-agent-orchestration
    provides: Task reasoning_effort wire (02), pre-pending shell spawn gate + pure oauth helpers (03)
  - phase: 06-dual-provider-model-switch
    provides: missing_provider_gate_error, login CLI copy
provides:
  - Async SubagentBackend::preflight_spawn oneshot RPC (ChannelBackend + SubagentEvent::Preflight)
  - Task eager await preflight before background "started" notice (fail closed)
  - Shell coordinator preflight handler using live ChatStateHandle effective-model path
  - Shared resolve_effective_child_model_for_spawn + missing_provider_spawn_gate_message
  - p7_eager (tools) + p7_preflight (shell) green filters
affects:
  - 07-05 dual-token Authorization proofs
  - 07-06 phase gate / AGENT-01 regression

tech-stack:
  added: []
  patterns:
    - "Required async backend preflight (C2-H1) — NOT sync slug-only Arc Fn"
    - "SubagentPreflightInput carries full spawn-resolution subset (type, resume_from, runtime_overrides, fork_context)"
    - "Dual layer: Task preflight_spawn (UX timing) + handle_subagent_request gate (authority)"
    - "Unavailable preflight fail-closed on Task path (no silent bg started)"

key-files:
  created: []
  modified:
    - crates/codegen/xai-grok-tools/src/implementations/grok_build/task/types.rs
    - crates/codegen/xai-grok-tools/src/implementations/grok_build/task/backend.rs
    - crates/codegen/xai-grok-tools/src/implementations/grok_build/task/mod.rs
    - crates/codegen/xai-grok-shell/src/agent/subagent/handle_request.rs
    - crates/codegen/xai-grok-shell/src/agent/subagent/mod.rs
    - crates/codegen/xai-grok-shell/src/agent/mvp_agent/subagent_coordinator.rs
    - crates/codegen/xai-grok-shell/src/agent/subagent/tests/mod.rs

key-decisions:
  - "Production design is async SubagentBackend.preflight_spawn + shell coordinator live resolve (C2-H1)"
  - "Sync slug-only credential Fn is an explicit non-goal — cannot match omit-model inherit / role pins"
  - "Shared resolve_effective_child_model_for_spawn for preflight; spawn path reuses missing_provider_spawn_gate_message"
  - "Preflight Unavailable is fail-closed on Task (unlike describe fail-open)"
  - "Full runtime_overrides + resume_from on preflight request (C3-M1 field parity)"

patterns-established:
  - "Eager Task gate: type validate → model validate → effort parse → preflight_spawn → spawn/started"
  - "Coordinator Preflight arm: try_build_subagent_spawn_context → preflight_subagent_spawn → oneshot"
  - "p7_eager_ (tools mock backend) + p7_preflight_ (shell live resolve) filter convention"

requirements-completed: [AGENT-05, AGENT-02]

coverage:
  - id: D1
    description: "Background Task path fails closed with login prompt before started notice"
    requirement: AGENT-05
    verification:
      - kind: unit
        ref: "cargo test -p xai-grok-tools --lib p7_eager"
        status: pass
    human_judgment: false
  - id: D2
    description: "Async preflight uses live effective model (omit/role/resume/persona)"
    requirement: AGENT-05
    verification:
      - kind: unit
        ref: "cargo test -p xai-grok-shell --lib p7_preflight"
        status: pass
    human_judgment: false
  - id: D3
    description: "Unknown model still fail-closed via existing TaskModelValidator path"
    requirement: AGENT-02
    verification:
      - kind: unit
        ref: "cargo test -p xai-grok-tools --lib p7_ (existing model validator tests)"
        status: pass
    human_judgment: false

duration: 15min
completed: 2026-07-17
status: complete
---

# Phase 7 Plan 04: Async Task preflight_spawn Summary

**Background Task path awaits async `SubagentBackend::preflight_spawn` (live shell effective-model resolve + credential gate) and fails closed with `bum login --provider` before returning the started notice — not a sync slug-only Fn.**

## Performance

- **Duration:** 15 min
- **Started:** 2026-07-17T08:58:50Z
- **Completed:** 2026-07-17T09:13:23Z
- **Tasks:** 2/2
- **Files modified:** 7

## Accomplishments

- Async `preflight_spawn` on `SubagentBackend` + `ChannelBackend` oneshot RPC (`SubagentEvent::Preflight`)
- Task tool always awaits preflight after type/model/effort validation and **before** `tokio::spawn` + started text
- Shell coordinator handles Preflight with shared live resolve (role/persona/resume/omit inherit via `ChatStateHandle`)
- Dual layer retained: `handle_subagent_request` still runs Plan 03 gate for harness-direct spawns
- Green filters: tools `p7_eager` (7) + shell `p7_preflight` (8)

## Task Commits

1. **Task 1: GREEN — SubagentBackend.preflight_spawn + Task eager await** - `2d49481`
2. **Task 2: GREEN — shell coordinator live async preflight** - `c132ca5`

**Plan metadata:** (docs commit after this SUMMARY)

## Files Created/Modified

- `task/types.rs` — `SubagentPreflightInput` / `Outcome` / `Request` + `SubagentEvent::Preflight`
- `task/backend.rs` — trait method + `ChannelBackend` RPC (timeout/unavailable like validate_type)
- `task/mod.rs` — eager await; `make_backend_*` auto-acks Preflight Ok; `p7_eager_*` tests
- `handle_request.rs` — `preflight_subagent_spawn`, `resolve_effective_child_model_for_spawn`, `missing_provider_spawn_gate_message`
- `subagent/mod.rs` — re-exports shared helpers
- `mvp_agent/subagent_coordinator.rs` — Preflight drain arm (no pending/worktree/spawn)
- `subagent/tests/mod.rs` — `p7_preflight_*` live effective-model cases

## Decisions Made

### Production design (C2-H1 locked)

**Required:** async `SubagentBackend::preflight_spawn` via oneshot RPC to shell coordinator with live parent session state.

**Explicit non-goal:** sync `Arc<dyn Fn(slug) -> Option<String>>` (or any slug-only gate installed at agent rebuild) is **not** sufficient for omit-model inherit, role pins, resume pins, or persona/runtime overrides, and is **not** the production design.

### C3-M1 field parity / shared resolve

`SubagentPreflightInput` carries:

| Field | Purpose |
|-------|---------|
| `subagent_type` | Role lookup key |
| `resume_from` | Resume model pin |
| `runtime_overrides` | model, persona, harness, provenance, isolation, capability |
| `parent_session_id` | Spawn context build |
| `fork_context` | Force parent model (harness) |

Shell `resolve_effective_child_model_for_spawn` mirrors spawn precedence (role/persona overrides → resume pin → fork → `resolve_effective_model_config` with live `ChatStateHandle` inherit). Gate message uses Plan 03 pure `oauth_provider_slot_usable` / `missing_provider_gate_error` via `missing_provider_spawn_gate_message` (also used by `handle_subagent_request`).

### Fail-closed semantics

- Missing `SubagentBackendResource` → existing missing_resource error before started
- Preflight `Denied` → `ToolError::invalid_arguments` with shell login-shaped text
- Preflight `Unavailable` → `validation_unavailable` custom error (fail closed, no bg started)

## C2-M3 inventory (SubagentBackend / Task resources)

| Path | Action |
|------|--------|
| `task/backend.rs` `ChannelBackend` | **implement** `preflight_spawn` (production transport) |
| `session/agent_rebuild.rs` ChannelBackend install | **handled** — same ChannelBackend; coordinator Preflight arm |
| `session/acp_session_impl/goal.rs` ChannelBackend | **implement via trait** — same ChannelBackend; spawns via Spawn event still hit shell gate |
| Task tool missing backend | **fail-closed** (`missing_resource`) |
| Task test `make_backend_*` | **allow-stub** Preflight Ok by default; deny stubs for `p7_eager` |
| task_output / kill_task ChannelBackend fixtures | **allow-stub** (ChannelBackend has method; fixtures that only drain Query/Cancel unaffected) |
| Other `impl SubagentBackend` | **none** — sole production impl is `ChannelBackend` |

## Deviations from Plan

None - plan executed as written (async backend preflight + live shell resolve; sync slug Fn rejected).

Minor note: `handle_subagent_request` still computes preflight model inline then calls shared `missing_provider_spawn_gate_message` rather than calling the full `resolve_effective_child_model_for_spawn` (which would re-resolve definition/role mid-spawn). Field-parity is via the shared preflight function for the UX path + shared gate message for both; resume/role/persona logic in `resolve_effective_child_model_for_spawn` matches the spawn preflight block.

## Known Stubs

None — preflight is fully wired for production ChannelBackend sessions.

## Threat Flags

None new beyond plan threat_model (T-07-08..T-07-16 mitigated by awaited async preflight + live resolve).

## Verify Commands (all green)

```bash
cargo test -p xai-grok-tools --lib p7_eager -- --nocapture   # 7 passed
cargo test -p xai-grok-tools --lib p7_ -- --nocapture        # 14 passed
cargo test -p xai-grok-shell --lib p7_preflight -- --nocapture  # 8 passed
cargo check -p xai-grok-tools -p xai-grok-shell
```

## Self-Check: PASSED

- FOUND: `task/types.rs` Preflight types + event variant
- FOUND: `task/backend.rs` `preflight_spawn`
- FOUND: `task/mod.rs` eager await + `p7_eager`
- FOUND: `handle_request.rs` `preflight_subagent_spawn`
- FOUND: `subagent_coordinator.rs` Preflight arm
- FOUND: commit `2d49481`
- FOUND: commit `c132ca5`
- VERIFY: tools p7_eager → 7 passed
- VERIFY: shell p7_preflight → 8 passed
