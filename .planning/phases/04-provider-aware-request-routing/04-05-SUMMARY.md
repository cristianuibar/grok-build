---
phase: 04-provider-aware-request-routing
plan: 05
subsystem: routing
tags: [provider-routing, fail-closed, authorization, observability, mod-04, mod-05, d-11, d-12, d-14]

requires:
  - phase: 04-provider-aware-request-routing
    provides: dual-key prepare/reconstruct transforms + catalog stamp (Plans 01–04)
provides:
  - SamplingClient local fail-closed preflight (nonblank static key or nonblank live bearer)
  - Empty/whitespace live resolver fail-closed (cycle 2 HIGH)
  - Mock on-wire Authorization proofs for xAI vs Codex fake tokens
  - Invalid-header logs redacted to api_key_len only (D-14)
  - Routing debug fields provider + credential_slot + base_url + has_api_key without secrets
  - Full provider_routing phase gate green (48 tests)
affects:
  - Phase 5 (live Codex AuthManager multi-principal refresh / OAuth UX)
  - Phase 6 (missing-provider switch gate UX)
  - Phase 9 (live dual-driver e2e)

tech-stack:
  added: []
  patterns:
    - "Live-sample fail-closed at SamplingClient::post (Result) before any network I/O"
    - "Whitespace-only static keys and resolved bearers treated as unusable material"
    - "SamplerConfig construction without key remains allowed; sample path rejects"
    - "MockInferenceServer Authorization capture as phase-gate proof (not config-field equality alone)"

key-files:
  created: []
  modified:
    - crates/codegen/xai-grok-sampler/src/client.rs
    - crates/codegen/xai-grok-shell/src/agent/config.rs
    - crates/codegen/xai-grok-shell/src/sampling/mod.rs
    - crates/codegen/xai-grok-shell/tests/provider_routing.rs

key-decisions:
  - "Fail-closed lives in SamplingClient::post (returns Result) so every HTTP path is covered; construct still allows empty keys"
  - "Blank/whitespace static api_key skipped at header seed; resolver None/whitespace does not leave unauthenticated headers"
  - "Invalid-header arms log api_key_len only — no full api_key = %api_key (D-14)"
  - "Accepted pre-existing short Authorization prefix diagnostics on client_post (MEDIUM scope note)"
  - "No Phase 5 OAuth browser flow and no Phase 6 missing-provider modal (D-13)"

patterns-established:
  - "Usable-material rule: nonblank static OR nonblank current_bearer() before post"
  - "provider_routing mock suite is the MOD-04/05 automated phase gate including on-wire Authorization"
  - "Shell re-exports BearerResolver for integration tests without new crates"

requirements-completed: [MOD-04, MOD-05]

coverage:
  - id: D1
    description: Local fail-closed when no usable credentials (None+None, blank static)
    requirement: MOD-04
    verification:
      - kind: integration
        ref: cargo test -p xai-grok-shell --test provider_routing missing_credentials_fail_closed
        status: pass
      - kind: integration
        ref: cargo test -p xai-grok-shell --test provider_routing blank_static_key_fail_closed
        status: pass
    human_judgment: false
  - id: D2
    description: Empty/whitespace live resolver fails closed with zero HTTP (cycle 2 HIGH)
    requirement: MOD-04
    verification:
      - kind: integration
        ref: cargo test -p xai-grok-shell --test provider_routing empty_live_resolver_fail_closed
        status: pass
      - kind: integration
        ref: cargo test -p xai-grok-shell --test provider_routing whitespace_live_resolver_fail_closed
        status: pass
    human_judgment: false
  - id: D3
    description: On-wire Authorization Bearer for xAI and Codex fake tokens; Codex no X-XAI-Token-Auth
    requirement: MOD-05
    verification:
      - kind: integration
        ref: cargo test -p xai-grok-shell --test provider_routing on_wire_authorization_xai_fake
        status: pass
      - kind: integration
        ref: cargo test -p xai-grok-shell --test provider_routing on_wire_authorization_codex_fake
        status: pass
    human_judgment: false
  - id: D4
    description: Invalid-header path does not log full api_key (D-14)
    requirement: MOD-04
    verification:
      - kind: unit
        ref: cargo test -p xai-grok-shell --test provider_routing invalid_header_path_does_not_log_full_key
        status: pass
    human_judgment: false
  - id: D5
    description: Full phase gate suite green (provider_routing + model_catalog + cargo check)
    requirement: MOD-05
    verification:
      - kind: integration
        ref: cargo test -p xai-grok-shell --test provider_routing
        status: pass
      - kind: integration
        ref: cargo test -p xai-grok-shell --test model_catalog
        status: pass
      - kind: other
        ref: cargo check -p xai-grok-shell -p xai-grok-sampler
        status: pass
    human_judgment: false
  - id: D6
    description: Empty Codex credentials construct route without panic (D-11 construction)
    requirement: MOD-04
    verification:
      - kind: integration
        ref: cargo test -p xai-grok-shell --test provider_routing empty_codex_credentials_constructs_route_without_panic
        status: pass
    human_judgment: false

duration: 9min
completed: 2026-07-16
status: complete
---

# Phase 4 Plan 5: Fail-closed sample + on-wire Authorization Summary

**SamplingClient local fail-closed (incl. empty live resolver) with redacted invalid-key logs and mock on-wire Authorization proofs for xAI/Codex fake tokens — full Phase 4 automated gate green.**

## Performance

- **Duration:** ~9 min
- **Started:** 2026-07-16T13:52:45Z
- **Completed:** 2026-07-16T14:01:45Z (approx)
- **Tasks:** 2/2
- **Files modified:** 4

## Accomplishments

- Live sample path fails closed before HTTP when static key and live bearer are both unusable (None, blank, whitespace, resolver→None/whitespace)
- MockInferenceServer asserts exact `Authorization: Bearer xai-fake-token` / `codex-fake-token` and zero requests on fail-closed rows
- Invalid-header conversion arms log `api_key_len` only (no full key); resolve debug adds `credential_slot`
- Full `provider_routing` (48) + `model_catalog` (24) + cargo check shell/sampler green; no Phase 5/6 UX modules

## Task Commits

Each task was committed atomically:

1. **Task 1: Local fail-closed + secret log fix + mock on-wire Authorization** - `fdd2663`
2. **Task 2: Phase gate — full provider_routing + model_catalog + cargo check** - verification-only (suite green after Task 1; module contract docs included in Task 1 commit)

**Plan metadata:** (pending docs commit)

## Files Created/Modified

- `crates/codegen/xai-grok-sampler/src/client.rs` — `ensure_usable_auth_material`, `post` → `Result`, blank-key skip, redacted invalid-header logs
- `crates/codegen/xai-grok-shell/src/agent/config.rs` — resolve debug: `credential_slot` + nonblank `has_api_key`
- `crates/codegen/xai-grok-shell/src/sampling/mod.rs` — re-export `BearerResolver` / shared types for tests
- `crates/codegen/xai-grok-shell/tests/provider_routing.rs` — fail-closed + on-wire Authorization + D-14 source lock + contract module docs

## Decisions Made

- Fail-closed at `SamplingClient::post` (returns `Result`) so all six HTTP entry points share one gate; `SamplerConfig` / `Client::new` still allow empty keys for pure resolve and header unit tests
- Whitespace-only static keys never seed Authorization; whitespace-only resolvers never overwrite and never count as usable
- Short Authorization prefix logging on `client_post` left as accepted pre-existing diagnostic (plan MEDIUM scope note)
- D-13: files_modified review — no login/OAuth/modal/gate UX paths

## Deviations from Plan

### Auto-fixed Issues

None - plan executed as written. Minor implementation notes:

1. **`post` signature → `Result`** — Required so preflight cannot be skipped; updated production call sites (`?`) and unit tests that build requests via `post`.
2. **agent_ops.rs / sampler_turn.rs** listed in plan `files_modified` but needed no code changes — Plan 04 already wired prepare/carrier; fail-closed is sampler-local.
3. **TDD RED/GREEN as single commit** — Implementation and tests landed together after verifying RED would fail on pre-change behavior; atomic commit `fdd2663` covers Task 1.

## Threat Mitigations Applied

| Threat | Disposition | Evidence |
|--------|-------------|---------|
| T-04-06 Information Disclosure (full api_key logs) | mitigate | `api_key_len` only; `invalid_header_path_does_not_log_full_key` |
| T-04-07 empty credentials sample | mitigate | local Auth before HTTP |
| T-04-10 accidental Phase 5/6 UX | mitigate | D-13 files/diff review; no new UX modules |
| T-04-14 unauthenticated sample send | mitigate | usable-material preflight incl. empty live resolver |
| T-04-SC package installs | accept | no new packages |

## Known Stubs

None.

## Scope notes (accepted)

- Pre-existing short Authorization / x-api-key prefix fields on `client_post` info logs remain (not expanded; not mandatory to eliminate this phase)
- Phase 5 multi-principal live Codex AuthManager refresh deferred
- Phase 6 missing-provider switch gate UX deferred
- Live ChatGPT dual-driver e2e deferred to Phase 9

## Verification

```text
cargo test -p xai-grok-shell --test provider_routing -- --nocapture
# 48 passed

cargo test -p xai-grok-shell --test model_catalog -- --nocapture
# 24 passed

cargo check -p xai-grok-shell -p xai-grok-sampler
# ok

cargo test -p xai-grok-sampler --lib client::tests
# 35 passed
```

## Self-Check: PASSED

- `crates/codegen/xai-grok-sampler/src/client.rs` — FOUND
- `crates/codegen/xai-grok-shell/tests/provider_routing.rs` — FOUND
- Commit `fdd2663` — FOUND
