---
phase: 06-mid-session-switch-missing-provider-gate
plan: 05
subsystem: testing
tags: [model-switch, dual-login, history, mid-turn, byok, session-harness, mod-03]

requires:
  - phase: 06-01
    provides: model_switch_gate harness, missing_provider gate, BYOK pure/apply
  - phase: 04-provider-aware-request-routing
    provides: Phase 4 route rebuild, switch_changes_next_sample_route pure regression
  - phase: 05-codex-oauth-dual-auth-lifecycle
    provides: dual-slot auth.json fixtures, store_usable semantics
provides:
  - Session-level dual free Grok↔GPT switch proofs (MOD-03)
  - Same-provider Codex switch without missing-provider friction
  - Next-sample route uses target provider credential after apply
  - chat_history.jsonl history preserve across successful apply (D-06)
  - Mid-turn hold_agent_completions non-cancel (D-06)
  - BYOK has_own_credentials apply success with empty OAuth
affects:
  - 06-06 phase verification / roadmap SC-3
  - verifier dual-login free movement

tech-stack:
  added: []
  patterns:
    - "GateHarness session apply + MockInferenceServer hold for mid-turn observables"
    - "chat_history.jsonl length + exact prompt identity snapshot (no empty-history tautology)"
    - "cancel_notifications AtomicUsize spy on GateClient session_notification"

key-files:
  created: []
  modified:
    - crates/codegen/xai-grok-shell/tests/model_switch_gate.rs

key-decisions:
  - "History observable = session storage chat_history.jsonl via locate_session_dir (not provider_routing)"
  - "Mid-turn non-cancel = hold_agent_completions + pinned prompt pending + cancel_notifications spy + EndTurn after release"
  - "Next-sample route = post-switch prompt Authorization must carry CODEX_FAKE not XAI_FAKE"
  - "GROK_CODEX_BASE_URL pointed at mock so Codex next-sample hits fixture (product default constant still asserted)"

patterns-established:
  - "p6_dual_login_* / p6_history_* / p6_mid_turn_* session apply contracts"
  - "dual_usable_auth() + GateHarness::start dual-slot fixture pattern"

requirements-completed: [MOD-03, MOD-06]

coverage:
  - id: D1
    description: Dual-usable free mid-session switch Grok→GPT without MODEL_SWITCH_MISSING_PROVIDER
    requirement: MOD-03
    verification:
      - kind: integration
        ref: cargo test -p xai-grok-shell --test model_switch_gate p6_dual_login_free_switch_xai_to_codex
        status: pass
    human_judgment: false
  - id: D2
    description: Dual-usable free reverse switch GPT→Grok without missing-provider
    requirement: MOD-03
    verification:
      - kind: integration
        ref: cargo test -p xai-grok-shell --test model_switch_gate p6_dual_login_free_switch_codex_to_xai
        status: pass
    human_judgment: false
  - id: D3
    description: Same-provider Codex Sol→Terra with only Codex slot succeeds
    requirement: MOD-03
    verification:
      - kind: integration
        ref: cargo test -p xai-grok-shell --test model_switch_gate p6_same_provider_codex_switch
        status: pass
    human_judgment: false
  - id: D4
    description: Next sample after apply uses target provider credential/model
    requirement: MOD-03
    verification:
      - kind: integration
        ref: cargo test -p xai-grok-shell --test model_switch_gate p6_dual_login_next_sample_uses_target_provider
        status: pass
    human_judgment: false
  - id: D5
    description: chat_history.jsonl length + user prompt identity preserved across successful switch
    requirement: MOD-03
    verification:
      - kind: integration
        ref: cargo test -p xai-grok-shell --test model_switch_gate p6_history_preserved_across_successful_switch
        status: pass
    human_judgment: false
  - id: D6
    description: Mid-turn apply with held MockInferenceServer does not cancel in-flight turn
    requirement: MOD-03
    verification:
      - kind: integration
        ref: cargo test -p xai-grok-shell --test model_switch_gate p6_mid_turn_switch_does_not_cancel_inflight
        status: pass
    human_judgment: false
  - id: D7
    description: BYOK has_own_credentials skips OAuth missing-provider at apply
    requirement: MOD-06
    verification:
      - kind: integration
        ref: cargo test -p xai-grok-shell --test model_switch_gate p6_byok_own_credentials_skips_oauth_missing_provider
        status: pass
    human_judgment: false
  - id: D8
    description: Pure routing regression switch_changes_next_sample_route remains green
    requirement: MOD-03
    verification:
      - kind: unit
        ref: cargo test -p xai-grok-shell --test provider_routing switch_changes_next_sample_route
        status: pass
    human_judgment: false

duration: 12min
completed: 2026-07-17
status: complete
---

# Phase 6 Plan 05: Dual free switch & session continuity Summary

**Session-level fixture tests prove MOD-03 free Grok↔GPT movement with dual usable tokens, D-06 history preserve + mid-turn non-cancel, BYOK skip, and next-sample target-provider routing — not pure routing substitutes.**

## Performance

- **Duration:** ~12 min
- **Started:** 2026-07-16T22:59:33Z
- **Completed:** 2026-07-16T23:11:00Z
- **Tasks:** 2/2
- **Files modified:** 1

## Accomplishments

- Extended Plan 01 `model_switch_gate` MvpAgent+ACP harness for dual-login free apply, same-provider Codex, next-sample Authorization route, history, and mid-turn hold
- Named history API: seed via `session/prompt` → snapshot `chat_history.jsonl` (line count + exact `HISTORY_MARKER` prompt string) before/after successful cross-provider `set_session_model`
- Named mid-turn spy: `GateClient.cancel_notifications` (`AtomicUsize`) + `MockInferenceServer::hold_agent_completions` / `release_agent_completions`; assert prompt stays pending during apply and ends `StopReason::EndTurn` after release
- Next-sample proof: after apply to Codex, prompt hits mock with `Authorization` carrying `CODEX_FAKE` (not `XAI_FAKE`) and Codex model id
- BYOK: `p6_byok_own_credentials_skips_oauth_missing_provider` requires apply `Ok` with empty Codex OAuth slot

## Task Commits

1. **Task 1+2: Dual free switch, history, mid-turn, BYOK session contracts** - `61686e4`

_Note: Both TDD tasks are pure session integration tests over already-shipped Plan 01 gate / Phase 4 routing; single atomic test commit (no production code changes)._

## Files Created/Modified

- `crates/codegen/xai-grok-shell/tests/model_switch_gate.rs` — harness extensions + `p6_dual_login_*`, `p6_same_provider_*`, `p6_dual_login_next_sample_*`, `p6_byok_own_*`, `p6_history_*`, `p6_mid_turn_*`

## Decisions Made

- **History surface:** `locate_session_dir` + `chat_history.jsonl` read (session storage), not chat-state handle (not public from test binary)
- **Mid-turn hold:** pin `session/prompt` future and interleave `set_session_model` on same `ClientSideConnection` (not Clone) via `tokio::select!`
- **Codex mock base:** set `GROK_CODEX_BASE_URL` to mock URL for traffic capture; assert product `CODEX_BASE_URL_DEFAULT` constant unchanged
- **provider_routing:** left `switch_changes_next_sample_route` as pure routing regression only (no session history claims)

## Named observables (review cycle 2)

| Contract | Observable | API / symbol |
|----------|------------|--------------|
| History preserve | line count + `HISTORY_MARKER` substring | `read_chat_history_jsonl` / `session_dir.join("chat_history.jsonl")` |
| Mid-turn non-cancel | cancel spy flat + prompt pending + `EndTurn` | `GateClient.cancel_notifications`, `hold_agent_completions` |
| Next-sample route | Auth + model on inference POST after switch | `MockInferenceServer::requests` Authorization / body.model |
| Free dual switch | apply `Ok`, not `MODEL_SWITCH_MISSING_PROVIDER` | `set_session_model` via `GateHarness::set_model` |

## Deviations from Plan

### Auto-fixed Issues

None - plan executed as written for session harness proofs.

### Notes

- Task 1 and Task 2 landed in one commit because both only extend the same integration binary and share harness helpers; no production `model_switch.rs` changes required.
- Serial `#[serial]` + full MvpAgent bootstrap makes each apply-path test ~30s; filter with `p6_dual_login` / `p6_history` / `p6_mid_turn` as in plan verify.

## Auth Gates

None.

## Known Stubs

None.

## Threat Flags

None new. T-06-15 mitigated by next-sample Authorization asserting Codex token not xAI after switch.

## Test Results

```text
cargo test -p xai-grok-shell --test model_switch_gate p6_dual_login → 3 passed
cargo test -p xai-grok-shell --test model_switch_gate p6_same_provider → 1 passed
cargo test -p xai-grok-shell --test model_switch_gate p6_byok → 3 passed
cargo test -p xai-grok-shell --test model_switch_gate p6_history → 1 passed
cargo test -p xai-grok-shell --test model_switch_gate p6_mid_turn → 1 passed
cargo test -p xai-grok-shell --test provider_routing switch_changes_next_sample_route → 1 passed
```

## Self-Check: PASSED

- `crates/codegen/xai-grok-shell/tests/model_switch_gate.rs` FOUND
- Commit `61686e4` FOUND
- p6_dual_login / p6_history / p6_mid_turn / p6_byok / p6_same_provider tests FOUND
