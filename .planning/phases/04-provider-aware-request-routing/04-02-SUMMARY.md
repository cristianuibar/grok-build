---
phase: 04-provider-aware-request-routing
plan: 02
subsystem: routing
tags: [provider-routing, resolve_provider_route, codex-base-url, session-oauth, mod-04, mod-05]

requires:
  - phase: 04-provider-aware-request-routing
    provides: Wave 0 provider_routing harness (Plan 01)
provides:
  - CODEX_BASE_URL_DEFAULT + EndpointsConfig.codex_base_url + resolve_codex_base_url
  - GROK_CODEX_BASE_URL env fill (mirror GROK_XAI_API_BASE_URL)
  - Public pure resolve_provider_route / ProviderRoute with session_oauth_allowed
  - is_first_party_codex_url allowlist (chatgpt.com Codex path + configured codex host)
  - Pure-route + custom-host OAuth denial tests green under --test provider_routing
affects:
  - 04-03 (default_models stamp + dual-key credentials must call resolve_provider_route)
  - 04-04 (prepare/reconstruct slot + oauth-allowed decisions)
  - 04-05 (fail-closed sample when session OAuth denied and no own credential)

tech-stack:
  added: []
  patterns:
    - "resolve_provider_route is production authority for provider-default base_url + session_oauth_allowed"
    - "codex_base_url String (not Option) matching xai_api_base_url field style"
    - "session_oauth_allowed from final base_url first-party classification only"
    - "Xai custom models_base_url / override denies session OAuth (cycle-3 MEDIUM)"

key-files:
  created: []
  modified:
    - crates/codegen/xai-grok-shell/src/agent/config.rs
    - crates/codegen/xai-grok-shell/src/auth/mod.rs
    - crates/codegen/xai-grok-shell/tests/provider_routing.rs

key-decisions:
  - "CODEX_BASE_URL_DEFAULT = https://chatgpt.com/backend-api/codex (not Platform api.openai.com)"
  - "codex_base_url is String like xai_api_base_url; blank → resolve_codex_base_url default"
  - "session_oauth_allowed false for non–first-party final URL including Xai + custom models_base_url"
  - "Re-export PROVIDER_XAI/PROVIDER_CODEX from auth root (model module stays private)"
  - "MOD-04/MOD-05 remain Pending until Plans 03–05 wire stamp + dual-key + prepare"

patterns-established:
  - "First-party Codex: chatgpt.com|/backend-api/codex OR host of resolve_codex_base_url()"
  - "credential_slot always follows ModelProvider even when override wins for URL"
  - "provider_routing pure greens use field overrides for parallel CI; one serial EnvGuard env test"

requirements-completed: []  # MOD-04/MOD-05 stay Pending — pure route only; stamp/credentials/prepare later

coverage:
  - id: D1
    description: CODEX_BASE_URL_DEFAULT + EndpointsConfig.codex_base_url + resolve_codex_base_url + env override
    requirement: MOD-05
    verification:
      - kind: integration
        ref: cargo test -p xai-grok-shell --test provider_routing resolve_codex_base_url
        status: pass
      - kind: integration
        ref: cargo test -p xai-grok-shell --test provider_routing codex_base_url_default
        status: pass
    human_judgment: false
  - id: D2
    description: Pure resolve_provider_route Xai/Codex defaults + override wins + blank ignored
    requirement: MOD-04
    verification:
      - kind: integration
        ref: cargo test -p xai-grok-shell --test provider_routing resolve_provider_route
        status: pass
    human_judgment: false
  - id: D3
    description: Custom host and Xai custom models_base_url deny session OAuth; first-party Codex allows
    requirement: MOD-05
    verification:
      - kind: integration
        ref: cargo test -p xai-grok-shell --test provider_routing resolve_provider_route_custom_host
        status: pass
      - kind: integration
        ref: cargo test -p xai-grok-shell --test provider_routing resolve_provider_route_xai_custom
        status: pass
      - kind: integration
        ref: cargo test -p xai-grok-shell --test provider_routing resolve_provider_route_first_party_codex
        status: pass
    human_judgment: false

duration: 8min
completed: 2026-07-16
status: complete
---

# Phase 4 Plan 02: Pure provider route + Codex endpoint Summary

**Public `resolve_provider_route` is the production authority for base_url + credential_slot + session_oauth_allowed, with ChatGPT Codex default endpoint and first-party OAuth host policy.**

## Performance

- **Duration:** 8 min
- **Started:** 2026-07-16T13:24:27Z
- **Completed:** 2026-07-16T13:32:22Z
- **Tasks:** 2/2
- **Files modified:** 3

## Accomplishments

- Shipped `CODEX_BASE_URL_DEFAULT`, `EndpointsConfig.codex_base_url`, `resolve_codex_base_url()`, and `GROK_CODEX_BASE_URL` env fill
- Shipped public pure `ProviderRoute` + `resolve_provider_route` + `is_first_party_codex_url`
- Session OAuth denied for custom/BYOK hosts including Xai + custom `models_base_url` (cycle-3 MEDIUM)
- 11 new pure-route/endpoint tests green under `--test provider_routing`; catalog stamp still deferred to Plan 03

## Task Commits

Each task was committed atomically:

1. **Task 1: Codex endpoint default + env override on EndpointsConfig** - `e893e23`
2. **Task 2: Public pure resolve_provider_route + ProviderRoute + OAuth host policy** - `9216bb0`

**Plan metadata:** (see final docs commit)

## Files Created/Modified

- `crates/codegen/xai-grok-shell/src/agent/config.rs` — Codex endpoint surface + pure route resolver
- `crates/codegen/xai-grok-shell/src/auth/mod.rs` — re-export `PROVIDER_XAI` / `PROVIDER_CODEX`
- `crates/codegen/xai-grok-shell/tests/provider_routing.rs` — pure-route + endpoint greens

## Test inventory (Plan 02 additions)

| Test | Status | Notes |
|------|--------|-------|
| `codex_base_url_default_constant` | green | D-07 / D-15 |
| `resolve_codex_base_url_default` | green | blank/whitespace → default |
| `resolve_codex_base_url_field_override` | green | parallel-CI safe |
| `resolve_codex_base_url_env_override` | green | serial + EnvGuard |
| `resolve_provider_route_xai_default` | green | D-05/D-09 |
| `resolve_provider_route_codex_default` | green | D-06/D-09 |
| `resolve_provider_route_override_wins` | green | D-04 + slot |
| `resolve_provider_route_custom_host_disallows_session_oauth` | green | review HIGH |
| `resolve_provider_route_xai_custom_models_base_disallows_session_oauth` | green | cycle-3 MEDIUM |
| `resolve_provider_route_first_party_codex_override_allows_oauth` | green | first-party allow |
| `resolve_provider_route_blank_override_ignored` | green | whitespace fallthrough |

**Still intentional RED (later plans):** `codex_model_routes_to_codex_backend_with_codex_token` (03), `never_cross_slot` (03), `switch_changes_next_sample_route` (04).

## Decisions Made

- Default Codex URL is ChatGPT backend, not Platform OpenAI
- `codex_base_url: String` matches `xai_api_base_url` (intentional vs proxy `Option`)
- `session_oauth_allowed` classified from **final** base_url only; stock defaults → true
- Credential slot constants re-exported at `crate::auth::{PROVIDER_XAI, PROVIDER_CODEX}` because `auth::model` is private
- Did not stamp `default_models()` or dual-key credentials (Plan 03)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Correctness] Re-export PROVIDER_* at auth root**
- **Found during:** Task 2 compile
- **Issue:** Plan referenced `crate::auth::model::PROVIDER_*` but `model` is a private module
- **Fix:** `pub use model::{…, PROVIDER_CODEX, PROVIDER_XAI, …}` and call sites use `crate::auth::PROVIDER_*`
- **Files modified:** `src/auth/mod.rs`, `src/agent/config.rs`
- **Commit:** `9216bb0`

**2. [Rule 2 - Correctness] Did not mark MOD-04/MOD-05 complete**
- **Found during:** state updates after Task 2
- **Issue:** Plan frontmatter lists MOD-04/MOD-05, but pure route only — catalog still stamps cli-chat-proxy; dual-key and prepare not wired
- **Fix:** Leave REQUIREMENTS Pending (same posture as Plan 01); full req completion is Plans 03–05
- **Files modified:** none (skipped `requirements.mark-complete`)

## Known Stubs

None in production path for this plan. Catalog Codex base_url stamping intentionally still wrong until Plan 03.

## Threat Flags

None beyond plan threat model (T-04-03/T-04-11 mitigated by `session_oauth_allowed`; Plan 03 enforces in credentials).

## Self-Check: PASSED

- FOUND: `CODEX_BASE_URL_DEFAULT`, `resolve_codex_base_url`, `ProviderRoute`, `resolve_provider_route`, `is_first_party_codex_url`
- FOUND: commit `e893e23`
- FOUND: commit `9216bb0`
- FOUND: `cargo test -p xai-grok-shell --test provider_routing resolve_provider_route` 7/7 pass
- FOUND: `cargo test -p xai-grok-shell --test provider_routing resolve_codex_base_url` 3/3 pass
- FOUND: full binary 18 pass / 3 intentional RED
