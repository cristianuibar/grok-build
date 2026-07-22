---
phase: 04-provider-aware-request-routing
plan: 01
subsystem: testing
tags: [provider-routing, integration-tests, wave0, mod-04, mod-05, fake-tokens]

requires:
  - phase: 03-model-catalog-gpt56
    provides: ModelProvider catalog entries (grok-build xai, gpt-5.6-* codex) and public resolve_model_list
provides:
  - Wave 0 integration harness tests/provider_routing.rs
  - Behavior-RED dual-route contracts for Codex base_url + dual-token never_cross_slot + production switch scaffold
  - Green locks for smoke, xAI route, override base_url, proxy/Codex header policy, api_backend responses
affects:
  - 04-02 (CODEX_BASE_URL_DEFAULT / resolve_provider_route GREEN)
  - 04-03 (dual-key credentials + catalog stamp GREEN)
  - 04-04 (prepare/reconstruct switch GREEN)
  - 04-05 (fail-closed + on-wire Authorization)

tech-stack:
  added: []
  patterns:
    - "Wave 0 integration binary on public shell APIs only (no shell --lib)"
    - "Behavior-RED intentional until sequential Plans 02–05; no #[ignore] on core contracts"
    - "Deterministic EndpointsConfig construction for route assertions"
    - "Fake tokens xai-fake-token / codex-fake-token only"

key-files:
  created:
    - crates/codegen/xai-grok-shell/tests/provider_routing.rs
  modified: []

key-decisions:
  - "switch_changes_next_sample_route is intentional assert!(false) Plan 04 scaffold — not pure dual sampling_config_for_model SC-3"
  - "never_cross_slot keeps both tokens in scope and fails until dual-key API; documents single-key cross-slot defect"
  - "Codex default URL asserted as string literal https://chatgpt.com/backend-api/codex until Plan 02 exports CODEX_BASE_URL_DEFAULT"

patterns-established:
  - "provider_routing harness mirrors model_catalog.rs plain #[test] style"
  - "Optional pure lock named sampling_config_for_model_differs_by_model distinct from SC-3 production contract"

requirements-completed: []  # MOD-04/MOD-05 stay Pending until Plans 02–05 turn routing GREEN

coverage:
  - id: D1
    description: Wave 0 harness compiles, --list discovers contracts, smoke green
    requirement: MOD-04
    verification:
      - kind: integration
        ref: cargo test -p xai-grok-shell --test provider_routing -- --list
        status: pass
      - kind: integration
        ref: cargo test -p xai-grok-shell --test provider_routing provider_routing_harness_smoke
        status: pass
    human_judgment: false
  - id: D2
    description: Codex dual-route base_url contract present and behavior-RED
    requirement: MOD-05
    verification:
      - kind: integration
        ref: cargo test -p xai-grok-shell --test provider_routing codex_model_routes_to_codex_backend_with_codex_token
        status: fail
    human_judgment: false
  - id: D3
    description: Dual-token never_cross_slot scaffold + production switch scaffold named honestly
    requirement: MOD-04
    verification:
      - kind: integration
        ref: cargo test -p xai-grok-shell --test provider_routing never_cross_slot
        status: fail
      - kind: integration
        ref: cargo test -p xai-grok-shell --test provider_routing switch_changes_next_sample_route
        status: fail
    human_judgment: false
  - id: D4
    description: Header locks and api_backend responses lock green
    requirement: MOD-04
    verification:
      - kind: integration
        ref: cargo test -p xai-grok-shell --test provider_routing xai_proxy_headers_still_apply
        status: pass
      - kind: integration
        ref: cargo test -p xai-grok-shell --test provider_routing no_proxy_headers_on_codex
        status: pass
      - kind: integration
        ref: cargo test -p xai-grok-shell --test provider_routing sampling_config_api_backend_from_model
        status: pass
    human_judgment: false

duration: 12min
completed: 2026-07-16
status: complete
---

# Phase 4 Plan 01: Wave 0 provider_routing harness Summary

**Integration binary encodes MOD-04/MOD-05 dual-route + dual-token + switch contracts with smoke green and intentional behavior-RED until Plans 02–05.**

## Performance

- **Duration:** 12 min
- **Started:** 2026-07-16T13:20:06Z
- **Completed:** 2026-07-16T13:32:00Z
- **Tasks:** 2/2
- **Files modified:** 1 created

## Accomplishments

- Created `crates/codegen/xai-grok-shell/tests/provider_routing.rs` Wave 0 harness on public APIs only
- Smoke + xAI route + override + header locks + api_backend Responses green today
- Codex catalog base_url, never_cross_slot dual-token, and production switch path reserved as honest behavior-RED (no `#[ignore]`)

## Task Commits

Each task was committed atomically:

1. **Task 1: Wave 0 compiling smoke harness + behavior-RED dual-route contracts** - `8c943cf`
2. **Task 2: Dual-token never_cross_slot scaffold + header locks + api_backend** - `719611b`

**Plan metadata:** (see final docs commit)

## Files Created/Modified

- `crates/codegen/xai-grok-shell/tests/provider_routing.rs` — Wave 0 MOD-04/MOD-05 integration harness (10 tests)

## Test inventory

| Test | Expected Plan 01 | Notes |
|------|------------------|-------|
| `provider_routing_harness_smoke` | green | binary + catalog discoverability |
| `xai_model_routes_to_proxy_with_xai_token` | green | MOD-04 path already correct |
| `codex_model_routes_to_codex_backend_with_codex_token` | RED | catalog still stamps cli-chat-proxy |
| `model_override_base_url_wins` | green | D-04 explicit override |
| `no_proxy_headers_on_codex` | green | Pitfall 4 lock |
| `never_cross_slot` | RED | dual-token Plan 03 target |
| `switch_changes_next_sample_route` | RED | production path Plan 04 scaffold |
| `sampling_config_for_model_differs_by_model` | green | pure helper (not SC-3) |
| `xai_proxy_headers_still_apply` | green | MOD-04 regression |
| `sampling_config_api_backend_from_model` | green | D-08 responses |

## Decisions Made

- `switch_changes_next_sample_route` uses `assert!(false, "Plan 04: wire prepare/reconstruct…")` rather than claiming SC-3 via pure dual `sampling_config_for_model`
- `never_cross_slot` proves both tokens in scope + today's single-key cross-slot defect, then fails until dual-key API is public
- Expected Codex URL is the string constant `https://chatgpt.com/backend-api/codex` (Plan 02 will export `CODEX_BASE_URL_DEFAULT`)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Correctness] Did not leave MOD-04/MOD-05 marked complete after Wave 0**
- **Found during:** state updates after Task 2
- **Issue:** Plan frontmatter lists MOD-04/MOD-05, but Plan 01 only scaffolds RED tests — product routing is still wrong for Codex
- **Fix:** Reverted REQUIREMENTS.md checkboxes/traceability to Pending; Wave 0 does not claim req complete
- **Files modified:** `.planning/REQUIREMENTS.md`, `04-01-SUMMARY.md`
- **Commit:** docs commit for plan completion

## Known Stubs

None in production code (test-only RED scaffolds are intentional Wave 0 contracts, not product stubs).

## Threat Flags

None beyond plan threat model (fake tokens only; dual-token isolation encoded for Plan 03 mitigate).

## Self-Check: PASSED

- FOUND: `crates/codegen/xai-grok-shell/tests/provider_routing.rs`
- FOUND: commit `8c943cf`
- FOUND: commit `719611b`
- FOUND: `cargo test -p xai-grok-shell --test provider_routing -- --list` discovers 10 tests including `never_cross_slot` and `switch_changes_next_sample_route`
- FOUND: smoke green
