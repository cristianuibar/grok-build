---
phase: 04-provider-aware-request-routing
plan: 03
subsystem: routing
tags: [provider-routing, default-models, dual-key-credentials, session-oauth, mod-04, mod-05]

requires:
  - phase: 04-provider-aware-request-routing
    provides: pure resolve_provider_route + CODEX_BASE_URL_DEFAULT + EndpointsConfig.codex_base_url (Plan 02)
provides:
  - default_models stamps provider-default base_url via resolve_provider_route
  - ConfigModelOverride provider rebind re-normalizes base + api_base_url
  - public resolve_credentials_for_provider(model, endpoints, xai_key, codex_key)
  - dual-token never_cross_slot GREEN; Codex never gets XAI_API_KEY
  - call-site inventory convert/defer for all production resolve_credentials users
affects:
  - 04-04 (prepare_sampling / ModelsManager / reconstruct dual-key wiring)
  - 04-05 (fail-closed sample when no credential on OAuth-denied host)
  - Phase 7 (full dual-slot subagent isolation)

tech-stack:
  added: []
  patterns:
    - "Catalog stamp + provider rebind must call resolve_provider_route (no parallel base tables)"
    - "resolve_credentials_for_provider takes EndpointsConfig for OAuth trust provenance"
    - "Single-key resolve_credentials maps session_key into model provider slot only"
    - "session_oauth_allowed from resolve_provider_route(provider, endpoints, Some(entry.base_url))"

key-files:
  created: []
  modified:
    - crates/codegen/xai-grok-shell/src/agent/config.rs
    - crates/codegen/xai-grok-shell/src/agent/subagent/mod.rs
    - crates/codegen/xai-grok-shell/tests/provider_routing.rs

key-decisions:
  - "default_models uses resolve_provider_route; Codex api_base_url = None"
  - "Provider-only ConfigModelOverride re-normalizes base + api_base_url via resolver"
  - "Dual-key public API is production dual-slot path; single-key documented as slot-scoped"
  - "OAuth trust from resolve_provider_route + caller EndpointsConfig — never fresh Config::default() string compare"
  - "Subagent Codex: codex_session_key=None (no cross-apply xAI parent token); Phase 7 dual-slot"
  - "MOD-04/MOD-05 remain Pending until Plan 04–05 wire prepare/reconstruct/fail-closed"

patterns-established:
  - "Stamp: route = resolve_provider_route(provider, endpoints, None); Xai keeps api_base_url"
  - "Rebind: if override.provider Some && override.base_url None → re-stamp via resolver"
  - "Credentials priority: own → session(if oauth allowed) → XAI_API_KEY (Xai only) → empty"
  - "provider_routing dual-token tests always pass both keys into dual-key API"

requirements-completed: []  # MOD-04/MOD-05 stay Pending — prepare/reconstruct still Plan 04

coverage:
  - id: D1
    description: default_models stamps Codex/xAI bases via resolve_provider_route; Codex api_base_url None
    requirement: MOD-05
    verification:
      - kind: integration
        ref: cargo test -p xai-grok-shell --test provider_routing codex_model_routes
        status: pass
      - kind: integration
        ref: cargo test -p xai-grok-shell --test provider_routing xai_model_routes
        status: pass
    human_judgment: false
  - id: D2
    description: Provider-only override rebinds base_url; explicit base preserved
    requirement: MOD-04
    verification:
      - kind: integration
        ref: cargo test -p xai-grok-shell --test provider_routing provider_override
        status: pass
    human_judgment: false
  - id: D3
    description: Dual-key never_cross_slot + custom host OAuth skip + configured Codex endpoint OAuth
    requirement: MOD-05
    verification:
      - kind: integration
        ref: cargo test -p xai-grok-shell --test provider_routing never_cross_slot
        status: pass
      - kind: integration
        ref: cargo test -p xai-grok-shell --test provider_routing custom_host_skips_session_oauth
        status: pass
      - kind: integration
        ref: cargo test -p xai-grok-shell --test provider_routing configured_codex_endpoint_allows_session_oauth
        status: pass
    human_judgment: false
  - id: D4
    description: Codex skips XAI_API_KEY env fallback (serial EnvGuard)
    requirement: MOD-05
    verification:
      - kind: integration
        ref: cargo test -p xai-grok-shell --test provider_routing codex_skips_xai_api_key_env_fallback
        status: pass
    human_judgment: false

duration: 9min
completed: 2026-07-16
status: complete
---

# Phase 4 Plan 03: Catalog stamp + dual-key credentials Summary

**Bundled catalog stamps provider-correct base URLs via `resolve_provider_route`, and dual-key `resolve_credentials_for_provider` isolates xAI/Codex session tokens with EndpointsConfig OAuth host provenance.**

## Performance

- **Duration:** 9 min
- **Started:** 2026-07-16T13:33:31Z
- **Completed:** 2026-07-16T13:42:05Z
- **Tasks:** 2/2
- **Files modified:** 3

## Accomplishments

- `default_models()` stamps base_url from `resolve_provider_route`; Codex rows get `api_base_url: None` (D-15)
- `ConfigModelOverride` provider rebind without explicit base re-normalizes base + api_base_url (T-04-12)
- Public `resolve_credentials_for_provider(model, endpoints, xai_session_key, codex_session_key)` with dual-token isolation
- Session OAuth only when `session_oauth_allowed` from route against **caller** EndpointsConfig (T-04-15 / T-04-11)
- Codex never receives `XAI_API_KEY` fallback; empty Codex key still constructs route (D-11)
- Full production call-site inventory recorded below (convert / defer)

## Task Commits

Each task was committed atomically:

1. **Task 1: Provider-aware default_models stamping + override rebind** - `5a3033a`
2. **Task 2: Dual-key resolve_credentials_for_provider + never_cross_slot + inventory** - `00f4d35`

**Plan metadata:** (see final docs commit)

## Files Created/Modified

- `crates/codegen/xai-grok-shell/src/agent/config.rs` — stamp, rebind, dual-key credentials, unit-test updates
- `crates/codegen/xai-grok-shell/src/agent/subagent/mod.rs` — minimum provider-aware resolve (Codex key None)
- `crates/codegen/xai-grok-shell/tests/provider_routing.rs` — catalog/rebind/dual-key/OAuth greens

## resolve_credentials call-site inventory

| Call site | File | Disposition |
|-----------|------|-------------|
| `resolve_credentials` (single-key wrapper) | `agent/config.rs` | **Converted this plan** — maps key into model provider slot; delegates to dual-key with `EndpointsConfig::default()` |
| `resolve_credentials_for_provider` | `agent/config.rs` | **New this plan** — dual-key production dual-slot API |
| `resolve_credentials_enforced` | `agent/config.rs` | **Converted this plan** (via single-key wrapper) — web search / image aux paths inherit provider awareness + no Codex XAI env |
| `try_resolve_model_credentials` | `agent/config.rs` | **Converted this plan** — dual-key with `cfg.endpoints` + provider-slot session key |
| `resolve_model_to_sampling_config` | `agent/config.rs` | **Converted this plan** — uses provider-aware single-key wrapper |
| Subagent `resolve_model_override_to_config` | `agent/subagent/mod.rs` | **Minimum convert this plan** — dual-key; Codex gets `codex_session_key=None` (no xAI cross-apply); **full dual-slot isolation deferred to Phase 7** |
| `ModelsManager::sampling_config` | `agent/models.rs` | **Defer Plan 04** — convert to dual-key / provider-aware |
| `prepare_sampling_config_for_model` | `mvp_agent/agent_ops.rs` | **Defer Plan 04** — primary dual-slot builder |
| web_search e2e / tests | various | Update if break; not phase gate |

## Test inventory (Plan 03 additions / greens)

| Test | Status | Notes |
|------|--------|-------|
| `xai_model_routes_to_proxy_with_xai_token` | green | catalog stamp + session |
| `codex_model_routes_to_codex_backend_with_codex_token` | green | Codex base + api_base_url None |
| `provider_override_rebinds_base_url` | green | rebind HIGH |
| `provider_override_explicit_base_preserved` | green | D-04 |
| `never_cross_slot` | green | dual-token both keys present |
| `model_override_base_url_wins` | green | base preserved; custom host no OAuth |
| `custom_host_skips_session_oauth` | green | T-04-11 |
| `own_credential_on_custom_host_wins` | green | BYOK always allowed |
| `configured_codex_endpoint_allows_session_oauth` | green | EndpointsConfig provenance |
| `codex_skips_xai_api_key_env_fallback` | green | serial + EnvGuard |
| `empty_codex_key_allows_route_construction` | green | D-11 |

**Still intentional RED:** `switch_changes_next_sample_route` (Plan 04).

## Decisions Made

- Catalog stamp and rebind always go through `resolve_provider_route` (review production authority)
- Dual-key API is the only dual-slot-safe path; single-key is slot-scoped and untyped for provenance
- OAuth host policy uses `Some(model.info.base_url)` + caller's `EndpointsConfig` (ban default URL string compare)
- Subagent: do not put parent xAI session token into Codex slot this phase
- Do not mark MOD-04/MOD-05 complete until prepare/reconstruct (04) and fail-closed (05)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Correctness] Unit tests used custom hosts for session OAuth**
- **Found during:** Task 2 (session_oauth_allowed enforcement)
- **Issue:** Colocated unit tests expected SessionToken on `example.com` hosts; policy now denies session OAuth on non–first-party bases
- **Fix:** Session fallthrough unit tests use `CLI_CHAT_PROXY_BASE_URL_DEFAULT` first-party base
- **Files modified:** `agent/config.rs` unit tests
- **Commit:** `00f4d35`

**2. [Rule 2 - Correctness] Subagent maps Codex key to None not parent session**
- **Found during:** Task 2 inventory
- **Issue:** Mapping parent (xAI) session into Codex slot would cross-apply wrong bearer when OAuth allowed
- **Fix:** Codex path uses `codex_session_key=None` + no XAI env; Phase 7 dual-slot
- **Files modified:** `agent/subagent/mod.rs`
- **Commit:** `00f4d35`

**3. [Rule 2 - Correctness] Did not mark MOD-04/MOD-05 complete**
- **Found during:** state updates
- **Issue:** Pure catalog + credentials green, but prepare/reconstruct production path still Plan 04
- **Fix:** Leave REQUIREMENTS Pending (same posture as Plans 01–02)

## Known Stubs

None that block plan goals. `switch_changes_next_sample_route` remains intentional RED scaffold for Plan 04.

## Self-Check: PASSED

- `crates/codegen/xai-grok-shell/src/agent/config.rs` — FOUND
- `crates/codegen/xai-grok-shell/src/agent/subagent/mod.rs` — FOUND
- `crates/codegen/xai-grok-shell/tests/provider_routing.rs` — FOUND
- Commit `5a3033a` — FOUND
- Commit `00f4d35` — FOUND
- `cargo test -p xai-grok-shell --test provider_routing never_cross_slot` — pass
- `cargo test -p xai-grok-shell --test provider_routing codex_model_routes` — pass
- `cargo check -p xai-grok-shell` — pass
)
