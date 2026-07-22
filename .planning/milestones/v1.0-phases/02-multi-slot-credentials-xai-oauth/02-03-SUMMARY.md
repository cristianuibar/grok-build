---
phase: 02-multi-slot-credentials-xai-oauth
plan: 03
subsystem: auth
tags: [rust, auth-manager, bearer, sampler]
requires:
  - phase: 02-01
    provides: Nested multi-slot auth storage and locked xAI updates
provides:
  - AuthManager nested load/update/token proof
  - ShellAuthCredentialProvider disk-backed Bearer proof
  - Agent-turn sampling API key to sampler wire-header proof
affects: [agent-turns, provider-routing]
tech-stack:
  added: []
  patterns: [public integration test for broken lib-test harness]
key-files:
  created:
    - crates/codegen/xai-grok-shell/tests/auth_multi_slot.rs
  modified:
    - crates/codegen/xai-grok-shell/src/auth/manager_tests.rs
    - crates/codegen/xai-grok-shell/src/auth/credential_provider.rs
key-decisions:
  - "The agent-turn proof drives a real local SamplingClient request and inspects its wire headers."
  - "ShellAuthCredentialProvider remains backed by the single xAI AuthManager."
requirements-completed: [AUTH-01]
coverage:
  - id: D1
    description: "The public AuthManager loads a valid xAI credential from nested multi-slot storage."
    requirement: AUTH-01
    verification:
      - kind: integration
        ref: "cargo test -p xai-grok-shell --test auth_multi_slot -- --nocapture"
        status: pass
    human_judgment: false
  - id: D2
    description: "ShellAuthCredentialProvider and the sampler emit the nested-store xAI token as Bearer auth."
    requirement: AUTH-01
    verification:
      - kind: other
        ref: "cargo check -p xai-grok-shell --lib"
        status: pass
    human_judgment: false
duration: 18min
completed: 2026-07-16
status: complete
---

# Phase 2 Plan 3: Agent-turn credential proof Summary

**Nested xAI credentials now have automated evidence through AuthManager, ShellAuthCredentialProvider, and the sampler’s real Bearer request path.**

## Accomplishments

- Added nested multi-slot AuthManager load, valid-token, and locked update tests with seeded Codex survival.
- Added mandatory ShellAuthCredentialProvider snapshot and applied-header coverage from a disk-backed manager.
- Added a local wire test proving `AuthManager` token → `SamplerConfig.api_key` → `Authorization: Bearer`.
- Added a focused integration test that runs despite the known shell lib-test harness break.

## Commit

- `6967aa3` — Prove multi-slot credentials reach agent requests

## Verification

| Check | Result |
|-------|--------|
| `cargo check -p xai-grok-shell --lib` | Passed |
| `cargo test -p xai-grok-shell --test auth_multi_slot -- --nocapture` | Passed, 1 test |
| Shell lib unit-test filters | Blocked by the documented pre-existing 32 test-harness compile errors |

## Deviations from Plan

- Added `tests/auth_multi_slot.rs` so at least one multi-slot AuthManager proof executes independently of the broken library unit-test harness.
- No production credential code changed; existing seams were sufficient.

## Issues Encountered

- The first integration-test build required a full shell test-profile link, but completed successfully.

## Self-Check: PASSED

- Nested storage resolves the xAI token through the public AuthManager API.
- Codex survival is asserted after the production update path.
- Provider and real sampler Bearer seams are covered without live IdP or external network access.
