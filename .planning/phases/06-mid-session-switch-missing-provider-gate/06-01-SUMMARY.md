---
phase: 06-mid-session-switch-missing-provider-gate
plan: 01
subsystem: auth
tags: [model-switch, missing-provider, acp, oauth, codex, multi-slot]

requires:
  - phase: 05-codex-oauth-dual-auth-lifecycle
    provides: multi-slot auth.json, store_usable/credential_usable, dual login
  - phase: 04-provider-aware-request-routing
    provides: ModelProvider catalog, prepare sampling route
  - phase: 03-model-catalog-gpt-5-6-entries
    provides: GPT-5.6 Codex catalog entries
provides:
  - MODEL_SWITCH_MISSING_PROVIDER typed ACP error
  - provider_slot_usable pure helper (D-02)
  - Authoritative missing-provider gate in model_switch::apply (D-01)
  - model_switch_gate integration harness proving side-effect absence
affects:
  - 06-02 pager QuestionView mapping
  - 06-03 deferred login retry
  - 06-04 auth badges
  - 06-05 dual-login free movement

tech-stack:
  added: []
  patterns:
    - "Typed ACP model-switch error twin of ModelSwitchIncompatibleAgentError"
    - "Fail-closed gate after resolve_model_id before prepare/SetSessionModel"
    - "BYOK has_own_credentials skips OAuth-slot gate"

key-files:
  created:
    - crates/codegen/xai-grok-shell/tests/model_switch_gate.rs
  modified:
    - crates/codegen/xai-grok-shell/src/agent/config.rs
    - crates/codegen/xai-grok-shell/src/agent/handlers/model_switch.rs
    - crates/codegen/xai-grok-shell/src/auth/status.rs
    - crates/codegen/xai-grok-shell/src/auth/mod.rs

key-decisions:
  - "Gate lives in model_switch::apply for all callers (set_session_model, new_session, load_session)"
  - "Usable = store_usable/credential_usable; Codex never uses xAI AuthManager alone"
  - "BYOK models with has_own_credentials skip OAuth-slot gate"
  - "Provider on error always from catalog ModelEntry.info.provider"

patterns-established:
  - "missing_provider_gate_error pure decision table for unit tests + apply"
  - "model_switch_gate MvpAgent+ACP harness with process-wide BUM_HOME sandbox"

requirements-completed: [MOD-06]

coverage:
  - id: D1
    description: Typed MODEL_SWITCH_MISSING_PROVIDER ACP error with camelCase payload and CLI suggestion
    requirement: MOD-06
    verification:
      - kind: unit
        ref: cargo test -p xai-grok-shell --lib p6_model_switch_missing_provider
        status: pass
      - kind: integration
        ref: cargo test -p xai-grok-shell --test model_switch_gate p6_model_switch_missing_provider
        status: pass
    human_judgment: false
  - id: D2
    description: Apply-path gate blocks empty Codex slot and leaves session model unchanged
    requirement: MOD-06
    verification:
      - kind: integration
        ref: cargo test -p xai-grok-shell --test model_switch_gate p6_missing_provider_apply
        status: pass
    human_judgment: false
  - id: D3
    description: Refreshable Codex and BYOK skip missing-provider block
    requirement: MOD-06
    verification:
      - kind: integration
        ref: cargo test -p xai-grok-shell --test model_switch_gate p6_missing_provider_apply_allows_codex_when_refreshable_token_present
        status: pass
      - kind: integration
        ref: cargo test -p xai-grok-shell --test model_switch_gate p6_byok_model_skips_oauth_slot_gate
        status: pass
    human_judgment: false

duration: 16min
completed: 2026-07-16
status: complete
---

# Phase 6 Plan 01: Missing-provider shell gate Summary

**Shell fail-closes mid-session model switch with typed `MODEL_SWITCH_MISSING_PROVIDER` before prepare/SetSessionModel, reusing Phase 5 usable-token semantics (refreshable OK; BYOK skip).**

## Performance

- **Duration:** ~16 min
- **Started:** 2026-07-16T22:14:39Z
- **Completed:** 2026-07-16T22:30:18Z
- **Tasks:** 2/2
- **Files modified:** 5

## Accomplishments

- Added `MODEL_SWITCH_MISSING_PROVIDER` + `ModelSwitchMissingProviderError` (camelCase ACP data, UI-SPEC message, CLI suggestion)
- Pure `provider_slot_usable` / `missing_provider_gate_error` decision table (D-02 + BYOK)
- Authoritative gate in `model_switch::apply` after `resolve_model_id`, before prepare / agent-type / SetSessionModel / ModelChanged
- Integration harness `tests/model_switch_gate.rs` exercises real apply path and proves side-effect absence on block

## Task Commits

1. **Task 1: RED — typed error + pure usable + apply harness contracts** - `d5b11ae`
2. **Task 2: GREEN — gate in model_switch::apply** - `e7a7cab`

## Files Created/Modified

- `crates/codegen/xai-grok-shell/src/agent/config.rs` — error type, pure gate decision helper, unit tests
- `crates/codegen/xai-grok-shell/src/auth/status.rs` — `provider_slot_usable` over store_usable
- `crates/codegen/xai-grok-shell/src/auth/mod.rs` — re-export `provider_slot_usable`
- `crates/codegen/xai-grok-shell/src/agent/handlers/model_switch.rs` — apply-time gate + telemetry
- `crates/codegen/xai-grok-shell/tests/model_switch_gate.rs` — p6_ pure + apply harness

## Decisions Made

- Gate always in `apply` (not only `set_session_model` wrapper) so load/new paths fail closed consistently with MOD-06 spirit
- Disk `read_provider_auth_store` + `provider_slot_usable` is baseline; xAI may also count live AuthManager via `credential_usable` — Codex never uses AuthManager alone
- BYOK via `ModelEntry::has_own_credentials()` short-circuits OAuth-slot check
- Provider field sourced only from catalog `model.info.provider` (T-06-02)

## Deviations from Plan

### Auto-fixed Issues

None - plan executed as written for shell authority.

### Notes

- Plan verify used unfiltered `cargo test -p xai-grok-shell p6_…` which also builds unrelated integration targets; one pre-existing broken target (`signed_managed_config` missing `signed_policy::test_seam`) fails to compile. Gates were run with `--lib` and `--test model_switch_gate` instead (in scope for this plan). Documented for deferred cleanup if needed.

## Auth Gates

None.

## Known Stubs

None.

## Threat Flags

None new beyond plan threat model (T-06-01..05 mitigated by typed CLI-only suggestion, catalog provider, early return).

## Test Results

```text
cargo test -p xai-grok-shell --lib p6_  → 6 passed
cargo test -p xai-grok-shell --test model_switch_gate p6_ → 10 passed
```

## Self-Check: PASSED

- `crates/codegen/xai-grok-shell/tests/model_switch_gate.rs` FOUND
- `MODEL_SWITCH_MISSING_PROVIDER` in config.rs FOUND
- Gate in model_switch.rs FOUND
- Commits `d5b11ae`, `e7a7cab` FOUND
