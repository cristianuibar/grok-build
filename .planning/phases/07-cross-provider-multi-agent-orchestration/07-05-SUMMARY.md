---
phase: 07-cross-provider-multi-agent-orchestration
plan: 05
subsystem: shell
tags: [p7, isolation, dual-token, Authorization, subagent, D-12, AGENT-04, mock-http]

requires:
  - phase: 07-cross-provider-multi-agent-orchestration
    provides: pre-pending spawn gate + oauth helpers (03), async Task preflight (04)
  - phase: 04-provider-aware-routing
    provides: resolve_credentials_for_provider, sampling_config_for_model, MockInferenceServer wire patterns
provides:
  - In-crate minimal harness p7_isolation_spawn_sample_cancel (C2-M4)
  - Dual-direction mock HTTP Authorization proofs (Grok→Codex and Codex→Grok)
  - Parent model stability after real handle_subagent_request spawn
  - Missing-slot no-outbound mock proofs both directions
  - auth_json_path_override dual-slot credential resolve for deterministic fixtures
affects:
  - 07-06 phase gate / AGENT-01 regression
  - Phase 9 live multi-turn E2E

tech-stack:
  added: []
  patterns:
    - "Minimal harness: production resolve_effective_model_config → one MockInferenceServer sample → capture Authorization → timeout cancel/join"
    - "auth_json_path_override dual-slot read for child resolve (Codex disk + xAI AuthManager/disk)"
    - "Resolved base_url from production endpoints; sample retarget only (session OAuth host policy preserved)"
    - "Resolve-only key_prefix is complement — D-12 requires wire Authorization both dirs"

key-files:
  created: []
  modified:
    - crates/codegen/xai-grok-shell/src/agent/subagent/mod.rs
    - crates/codegen/xai-grok-shell/src/agent/subagent/tests/mod.rs
    - crates/codegen/xai-grok-shell/tests/cross_provider_subagent.rs

key-decisions:
  - "In-crate subagent::tests seam preferred over public export (pub(crate) resolve paths)"
  - "Harness function: p7_isolation_spawn_sample_cancel — 15s sample timeout, stream drain as cancel/join"
  - "auth_json_path_override drives dual-slot keys in resolve_model_override_to_config (Rule 2 test seam + determinism)"
  - "Both Authorization directions mandatory — no one-direction waiver"

patterns-established:
  - "p7_isolation_* / p7_parent_model_* / p7_missing_child_* filter convention for Plan 05"
  - "Fixture constants xai-fake-token-p7-lib / codex-fake-token-p7-lib equality asserts only (T-07-01)"

requirements-completed: [AGENT-02, AGENT-04, AGENT-05, AGENT-06]

coverage:
  - id: D1
    description: "Grok parent → Codex child mock HTTP Authorization = codex fixture"
    requirement: AGENT-04
    verification:
      - kind: unit
        ref: "cargo test -p xai-grok-shell --lib p7_isolation_grok_parent_codex"
        status: pass
    human_judgment: false
  - id: D2
    description: "Codex parent → Grok child mock HTTP Authorization = xai fixture"
    requirement: AGENT-04
    verification:
      - kind: unit
        ref: "cargo test -p xai-grok-shell --lib p7_isolation_codex_parent_grok"
        status: pass
    human_judgment: false
  - id: D3
    description: "Never cross-slot on child SamplingConfig seed (complement)"
    requirement: AGENT-02
    verification:
      - kind: unit
        ref: "cargo test -p xai-grok-shell --lib p7_isolation_never_cross_slot"
        status: pass
    human_judgment: false
  - id: D4
    description: "Parent model unchanged after real handle_subagent_request cross-provider spawn"
    requirement: AGENT-04
    verification:
      - kind: unit
        ref: "cargo test -p xai-grok-shell --lib p7_parent_model"
        status: pass
    human_judgment: false
  - id: D5
    description: "Missing child slot fails closed with login hint and zero mock outbound both dirs"
    requirement: AGENT-05
    verification:
      - kind: unit
        ref: "cargo test -p xai-grok-shell --lib p7_missing_child"
        status: pass
    human_judgment: false
  - id: D6
    description: "Tool reasoning_effort medium applied on child config (AGENT-03/06 automated)"
    requirement: AGENT-06
    verification:
      - kind: unit
        ref: "cargo test -p xai-grok-shell --lib p7_isolation_reasoning_effort_medium"
        status: pass
    human_judgment: false

duration: 10min
completed: 2026-07-17
status: complete
---

# Phase 7 Plan 05: Dual-token isolation harness + Authorization both dirs Summary

**In-crate minimal harness drives production child resolve → one mock HTTP sample and proves Grok↔Codex Authorization isolation both ways, parent model stability after real spawn, and missing-slot fail-closed with zero outbound traffic.**

## Performance

- **Duration:** 10 min
- **Started:** 2026-07-17T09:14:43Z
- **Completed:** 2026-07-17T09:24:56Z
- **Tasks:** 2/2
- **Files modified:** 3

## Accomplishments

- **C2-M4 minimal harness** `p7_isolation_spawn_sample_cancel` in `subagent/tests`:
  1. Dual-slot auth.json fixtures (`xai-fake-token-p7-lib` / `codex-fake-token-p7-lib`)
  2. Production `resolve_effective_model_config` (same path as `handle_subagent_request`)
  3. One outbound sample via `MockInferenceServer` + `crate::sampling::Client`
  4. Capture: Authorization, resolved base_url, mock host, child model, parent model before/after, effort
  5. Cancel/join: drain stream within **15s** timeout (timeout fails the test); drop client
- **D-12 both directions mandatory** (no waiver):
  - `p7_isolation_grok_parent_codex_child_route` — Bearer codex fixture; base_url matches Codex route
  - `p7_isolation_codex_parent_grok_child_route` — Bearer xai fixture; base_url matches xAI route
- **D-11 parent stability** after real `handle_subagent_request` with live `ChatStateHandle` re-read
- **D-05..D-07** `p7_missing_child_*` login-shaped error + `request_count == 0` both dirs
- **AGENT-06** effort medium on child SamplingConfig with wire still Codex token

## Test seam (document for Plan 06 / verifier)

| Item | Detail |
|------|--------|
| Placement | `crates/codegen/xai-grok-shell/src/agent/subagent/tests/mod.rs` (in-crate; reaches private resolve) |
| Helper | `async fn p7_isolation_spawn_sample_cancel(parent_model, child_model, xai_present, codex_present, reasoning_effort) -> P7IsolationCapture` |
| Production path | `resolve_effective_model_config` + optional `apply_subagent_reasoning_effort` + real `handle_subagent_request` for parent stability |
| Credential seam | `resolve_model_override_to_config` reads dual slots from `auth_json_path_override` when set; else process Codex snapshot + `ctx.auth` |
| Timeout | 15s on mock sample; fails test on hang |
| Filters | `p7_isolation`, `p7_parent_model`, `p7_missing_child` / `p7_missing` |
| Not acceptance | Resolve-only key_prefix without mock HTTP Authorization |

## Task Commits

1. **Task 1: GREEN — test seam + dual-direction isolation + parent stability** - `63eb6f4`
2. **Task 2: GREEN — missing child slot no wrong-backend (docs + lib tests in Task 1)** - `cccb839`

**Plan metadata:** (docs commit after this SUMMARY)

## Files Created/Modified

- `subagent/mod.rs` — `provider_slot_access_key_from_path` + override-aware dual-slot resolve
- `subagent/tests/mod.rs` — harness + `p7_isolation_*` + `p7_parent_model_*` + `p7_missing_child_*`
- `tests/cross_provider_subagent.rs` — documents Plan 05 lib seam filters

## Decisions Made

- Preferred **in-crate** seam over public harness export (review HIGH: private resolve not reachable from external `tests/`)
- Sample retargets only `base_url` to mock after production route construction (preserves session OAuth policy on first-party bases) — same proven Phase 4 wire pattern, but driven through subagent resolve
- Task 2 no-outbound proofs co-located with Task 1 harness in the same module so gate + mock share fixture builders; external binary documents filter names only

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing critical functionality] Dual-slot resolve via auth_json_path_override**
- **Found during:** Task 1
- **Issue:** `resolve_model_override_to_config` read Codex only from process-global `snapshot_codex_session_key_from_auth_store()` (`grok_home()/auth.json`), so dual-fixture isolation tests could not deterministically seed Codex without mutating process home / OnceLock
- **Fix:** When `auth_json_path_override` is set, read both provider slots from that path (xAI prefers live `ctx.auth`, falls back to disk); production path unchanged when override is None
- **Files modified:** `crates/codegen/xai-grok-shell/src/agent/subagent/mod.rs`
- **Commit:** `63eb6f4`

Otherwise plan executed as written (both Authorization directions, parent real-spawn stability, missing no-outbound).

## Threat Flags

None — fixtures only; Authorization compared to known constants; no new network endpoints outside MockInferenceServer in tests.

## Known Stubs

None — harness performs real mock HTTP samples; not resolve-only.

## Self-Check: PASSED

- FOUND: `crates/codegen/xai-grok-shell/src/agent/subagent/mod.rs` (dual-slot resolve)
- FOUND: `p7_isolation_spawn_sample_cancel` + both dir tests in `subagent/tests/mod.rs`
- FOUND: commits `63eb6f4`, `cccb839`
- VERIFY: `cargo test -p xai-grok-shell --lib p7_isolation` (4 ok)
- VERIFY: `cargo test -p xai-grok-shell --lib p7_parent_model` (1 ok)
- VERIFY: `cargo test -p xai-grok-shell --lib p7_missing_child` (2 ok)
