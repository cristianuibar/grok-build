---
phase: 04-provider-aware-request-routing
plan: 04
subsystem: routing
tags: [provider-routing, prepare-sampling, model-switch, bearer-resolver, dual-key, mod-04, mod-05]

requires:
  - phase: 04-provider-aware-request-routing
    provides: dual-key resolve_credentials_for_provider + resolve_provider_route + catalog stamp (Plan 03)
provides:
  - PreparedSamplingConfig carrier with auth_type + provider
  - Production transform A apply_prepared_sampling_to_chat_state_fields used by model_switch
  - Production transform B reconstruct_attach_policy_from_facts / should_attach
  - ModelAuthFacts.provider Option<ModelProvider> (None for unknown)
  - select_provider_access_token (lookup_auth quality; never BTreeMap first)
  - read_provider_auth_store Result with missing vs parse diagnostics
  - Codex credential snapshot at prepare (Phase 5 owns live multi-principal)
  - switch_changes_next_sample_route GREEN on exact production transforms A/B
affects:
  - 04-05 (fail-closed sample / observability)
  - Phase 5 (live Codex AuthManager multi-principal refresh)
  - Phase 6 (missing-provider UX)

tech-stack:
  added: []
  patterns:
    - "Option A SetSessionModel carrier: sampling_config + auth_type + provider from prepare"
    - "Transform A/B are public pure functions; production and tests share them"
    - "ModelAuthFacts.provider Option — never Default/Xai for unknown"
    - "Codex tokens snapshotted at prepare/switch only (not reconstruct hot path)"

key-files:
  created: []
  modified:
    - crates/codegen/xai-grok-shell/src/agent/config.rs
    - crates/codegen/xai-grok-shell/src/agent/mvp_agent/agent_ops.rs
    - crates/codegen/xai-grok-shell/src/agent/models.rs
    - crates/codegen/xai-grok-shell/src/agent/handlers/model_switch.rs
    - crates/codegen/xai-grok-shell/src/session/commands.rs
    - crates/codegen/xai-grok-shell/src/session/acp_session_impl/model_switch.rs
    - crates/codegen/xai-grok-shell/src/session/acp_session_impl/sampler_turn.rs
    - crates/codegen/xai-grok-shell/src/session/acp_session_impl/run_loop.rs
    - crates/codegen/xai-grok-shell/src/auth/model.rs
    - crates/codegen/xai-grok-shell/src/auth/storage.rs
    - crates/codegen/xai-grok-shell/src/auth/mod.rs
    - crates/codegen/xai-grok-shell/tests/provider_routing.rs

key-decisions:
  - "Option A: SetSessionModel carries auth_type + provider fields (not full PreparedSamplingConfig payload type)"
  - "prepare_prepared_sampling_config_for_model is the production prepare; prepare_sampling_config_for_model unwraps .sampler_config for legacy call sites"
  - "Codex session key via snapshot_codex_session_key_from_auth_store at prepare (disk once; Phase 5 live refresh)"
  - "select_provider_access_token ranks Oidc>ApiKey>External; skips WebLogin/blank; prefers non-expired; scope-stable tiebreak"
  - "should_attach true only Some(Xai) && gate; None and Some(Codex) never attach"

patterns-established:
  - "Carrier: prepare stamps auth_type from ResolvedCredentials; model_switch writes via transform A"
  - "Attach: reconstruct_full_config uses reconstruct_attach_policy_from_facts(facts, gate.active())"
  - "Tests compose prepare_sampling_credentials → prepared_sampling_config_from_credentials → A → B"

requirements-completed: [MOD-04, MOD-05]

coverage:
  - id: D1
    description: prepare dual-key + Codex ignore xAI session; prepared auth_type carrier
    requirement: MOD-04
    verification:
      - kind: integration
        ref: cargo test -p xai-grok-shell --test provider_routing prepare_sampling_credentials_codex_ignores_xai_session
        status: pass
      - kind: integration
        ref: cargo test -p xai-grok-shell --test provider_routing prepared_sampling_config_carries_auth_type
        status: pass
    human_judgment: false
  - id: D2
    description: Transform A preserves auth_type; model_switch no xAI AuthManager re-resolve
    requirement: MOD-04
    verification:
      - kind: integration
        ref: cargo test -p xai-grok-shell --test provider_routing apply_prepared_sampling_to_chat_state_fields_preserves_auth_type
        status: pass
    human_judgment: false
  - id: D3
    description: switch_changes_next_sample_route GREEN on production transforms A/B
    requirement: MOD-04
    verification:
      - kind: integration
        ref: cargo test -p xai-grok-shell --test provider_routing switch_changes_next_sample_route
        status: pass
    human_judgment: false
  - id: D4
    description: Option provider attach policy; unknown never attaches xAI resolver
    requirement: MOD-05
    verification:
      - kind: integration
        ref: cargo test -p xai-grok-shell --test provider_routing should_attach_xai_auth_manager_bearer_resolver
        status: pass
      - kind: integration
        ref: cargo test -p xai-grok-shell --test provider_routing model_auth_facts
        status: pass
      - kind: integration
        ref: cargo test -p xai-grok-shell --test provider_routing never_cross_slot
        status: pass
    human_judgment: false
  - id: D5
    description: select_provider_access_token multi-scope + store missing-vs-parse
    requirement: MOD-05
    verification:
      - kind: integration
        ref: cargo test -p xai-grok-shell --test provider_routing select_provider_access_token
        status: pass
      - kind: integration
        ref: cargo test -p xai-grok-shell --test provider_routing read_provider_auth_store_missing_vs_parse_error
        status: pass
    human_judgment: false

duration: 8min
completed: 2026-07-16
status: complete
---

# Phase 4 Plan 04: Provider-aware prepare + reconstruct Summary

**Next-sample routing is provider-correct: prepare dual-keys into a PreparedSamplingConfig carrier, model_switch applies transform A without xAI AuthManager re-resolve, and reconstruct attaches live xAI bearer only when ModelAuthFacts.provider is Some(Xai).**

## Performance

- **Duration:** 8 min
- **Started:** 2026-07-16T13:43:50Z
- **Completed:** 2026-07-16T13:51:13Z
- **Tasks:** 2/2
- **Files modified:** 13

## Accomplishments

- `prepare_prepared_sampling_config_for_model` dual-key: xAI AuthManager slot + Codex disk snapshot; never cross-slot
- `SetSessionModel` Option A carrier (`auth_type` + `provider`); `model_switch` calls transform A, removes xAI-only `resolve_chat_state_auth_type`
- `ModelAuthFacts.provider: Option<ModelProvider>`; reconstruct transform B / `should_attach` only for `Some(Xai)`
- `select_provider_access_token` lookup_auth-quality selection; `read_provider_auth_store` Result distinguishes missing vs parse
- `switch_changes_next_sample_route` GREEN composing exact production transforms A+B (40/40 provider_routing tests)

## Task Commits

1. **Task 1+2: prepare carrier + transform A/B + reconstruct Option provider** - `1188888`
   (Tasks co-committed: transform B / ModelAuthFacts.provider required for switch test and reconstruct wiring in the same wave)

**Plan metadata:** (pending docs commit)

## Files Created/Modified

- `crates/codegen/xai-grok-shell/src/agent/config.rs` — public helpers: session_key, prepare_sampling_credentials, PreparedSamplingConfig, transform A/B, should_attach, snapshot_codex, ModelAuthFacts.provider
- `crates/codegen/xai-grok-shell/src/agent/mvp_agent/agent_ops.rs` — dual-key prepare_prepared_sampling_config_for_model
- `crates/codegen/xai-grok-shell/src/agent/models.rs` — ModelsManager::sampling_config dual-key
- `crates/codegen/xai-grok-shell/src/agent/handlers/model_switch.rs` — send carrier fields
- `crates/codegen/xai-grok-shell/src/session/commands.rs` — SetSessionModel auth_type + provider
- `crates/codegen/xai-grok-shell/src/session/acp_session_impl/model_switch.rs` — transform A; no AuthManager re-resolve
- `crates/codegen/xai-grok-shell/src/session/acp_session_impl/sampler_turn.rs` — transform B attach policy
- `crates/codegen/xai-grok-shell/src/auth/model.rs` — select_provider_access_token; AuthStore public
- `crates/codegen/xai-grok-shell/src/auth/storage.rs` — read_provider_auth_store Result
- `crates/codegen/xai-grok-shell/tests/provider_routing.rs` — full Plan 04 contract suite GREEN

## Decisions Made

- **Option A carrier** on `SetSessionModel` (`auth_type` + `provider` fields) rather than replacing the payload with a single `PreparedSamplingConfig` type — keeps ACP command shape explicit and matches preferred plan option.
- **Legacy** `prepare_sampling_config_for_model` remains and unwraps `.sampler_config` so acp_agent / profile override call sites stay compile-compatible.
- **Codex Phase 4 snapshot:** `snapshot_codex_session_key_from_auth_store` at prepare only; reconstruct keeps chat-state Credentials.api_key and never re-reads auth.json for Codex. Phase 5 owns live multi-principal Codex AuthManager.
- **ShellAuthCredentialProvider** unchanged (xAI-only for storage/upload).

## Deviations from Plan

None - plan executed as written. Tasks 1 and 2 committed together because transform B / Option provider are required for the switch_changes production-path test in Task 1.

## Known Stubs

None — no placeholder empty collections or TODO UI paths introduced.

## Threat Flags

None new beyond plan `<threat_model>` mitigations (T-04-02, T-04-08, T-04-09, T-04-13, T-04-16 applied).

## Self-Check: PASSED

- [x] `crates/codegen/xai-grok-shell/src/agent/config.rs` — FOUND
- [x] `crates/codegen/xai-grok-shell/src/session/acp_session_impl/model_switch.rs` — FOUND
- [x] `crates/codegen/xai-grok-shell/src/session/acp_session_impl/sampler_turn.rs` — FOUND
- [x] `crates/codegen/xai-grok-shell/tests/provider_routing.rs` — FOUND
- [x] Commit `1188888` — FOUND
- [x] `cargo test -p xai-grok-shell --test provider_routing` — 40 passed
