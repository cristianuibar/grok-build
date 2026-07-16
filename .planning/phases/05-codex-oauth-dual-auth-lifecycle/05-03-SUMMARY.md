---
phase: 05-codex-oauth-dual-auth-lifecycle
plan: 03
subsystem: auth
tags: [oauth, codex, pkce, deviceauth, dual-auth, chatgpt, login]

requires:
  - phase: 05-codex-oauth-dual-auth-lifecycle/01
    provides: Wave 0 RED contracts + clap --provider scaffold
  - phase: 05-codex-oauth-dual-auth-lifecycle/02
    provides: Public multi-slot mutate/clear/status + AuthProvider type
provides:
  - In-tree Codex browser PKCE + deviceauth OAuth under auth/codex/
  - providers.codex persist with claims/expiry mapping
  - bum login --provider codex CLI path without xAI post_login_sync
  - GREEN AUTH-02 lifecycle contracts (authorize, persist, state, device)
affects:
  - 05-04 selective logout / auth status CLI
  - 05-05 Codex refresh + ChatGPT-Account-ID headers
  - 05-06 dual-auth polish

tech-stack:
  added: []
  patterns:
    - "Fixed-path ChatGPT OAuth (no OIDC discovery) with injectable issuer base for mock IdP"
    - "Codex login early-return before managed_config::post_login_sync"
    - "expires_at: expires_in > JWT exp > 5m conservative fallback"

key-files:
  created:
    - crates/codegen/xai-grok-shell/src/auth/codex/mod.rs
    - crates/codegen/xai-grok-shell/src/auth/codex/browser.rs
    - crates/codegen/xai-grok-shell/src/auth/codex/claims.rs
    - crates/codegen/xai-grok-shell/src/auth/codex/device.rs
  modified:
    - crates/codegen/xai-grok-shell/src/auth/mod.rs
    - crates/codegen/xai-grok-shell/src/auth/flow.rs
    - crates/codegen/xai-grok-shell/src/auth/oidc/protocol.rs
    - crates/codegen/xai-grok-pager-bin/src/main.rs
    - crates/codegen/xai-grok-shell/tests/auth_codex_lifecycle.rs

key-decisions:
  - "Scope key CODEX_AUTH_SCOPE = https://auth.openai.com::app_EMoamEEZ73f0CkXaXp7hrann"
  - "Conservative expires_at fallback = 5 minutes when expires_in and JWT exp both missing"
  - "Device flow mirrors openai/codex proprietary deviceauth (not RFC 8628); inject issuer for tests"
  - "Codex CLI path returns before post_login_sync (T-05-22)"

patterns-established:
  - "apply_codex_oauth_callback for fail-closed state/error/exchange with no disk write"
  - "persist_codex_tokens → mutate_provider_store_or_prune(AuthProvider::Codex) only"
  - "run_cli_login_for_provider(provider) dispatches Codex vs xAI"

requirements-completed: [AUTH-02]

coverage:
  - id: D1
    description: "Authorize URL includes PKCE S256, localhost callback ports, client_id, originator=bum"
    requirement: AUTH-02
    verification:
      - kind: integration
        ref: cargo test -p xai-grok-shell --test auth_codex_lifecycle codex_authorize_url_includes_pkce_and_localhost_callback
        status: pass
    human_judgment: false
  - id: D2
    description: "Inject Codex login persists providers.codex Oidc with claims; preserves xAI sibling"
    requirement: AUTH-02
    verification:
      - kind: integration
        ref: cargo test -p xai-grok-shell --test auth_codex_lifecycle codex_login_persists_slot
        status: pass
    human_judgment: false
  - id: D3
    description: "OAuth state mismatch / error / missing tokens write nothing"
    requirement: AUTH-02
    verification:
      - kind: integration
        ref: cargo test -p xai-grok-shell --test auth_codex_lifecycle codex_oauth_state_mismatch_writes_nothing
        status: pass
    human_judgment: false
  - id: D4
    description: "Device endpoints use deviceauth usercode/token paths"
    requirement: AUTH-02
    verification:
      - kind: integration
        ref: cargo test -p xai-grok-shell --test auth_codex_lifecycle codex_device_endpoints_use_deviceauth
        status: pass
    human_judgment: false
  - id: D5
    description: "Device multi-step pending → slow_down → access_denied leaves auth.json unchanged"
    requirement: AUTH-02
    verification:
      - kind: integration
        ref: cargo test -p xai-grok-shell --test auth_codex_lifecycle codex_device_pending_slowdown_exchange_denied
        status: pass
    human_judgment: false
  - id: D6
    description: "CLI bum login --provider codex parses (clap scaffold + pager-bin dispatch wired)"
    requirement: AUTH-02
    verification:
      - kind: unit
        ref: cargo test -p xai-grok-pager bum_login_provider_codex_parses
        status: unknown
    human_judgment: false
    # note: pager --lib test binary currently fails unrelated pre-existing compile errors in theme/prompt tests; clap unit test source and pager-bin wiring verified via cargo check -p xai-grok-pager-bin

duration: 15min
completed: 2026-07-16
status: complete
---

# Phase 5 Plan 03: Codex OAuth dual-auth login Summary

**In-tree ChatGPT/Codex PKCE + deviceauth OAuth persists only `providers.codex`; `bum login --provider codex` returns before xAI managed-config sync.**

## Performance

- **Duration:** 15 min
- **Started:** 2026-07-16T17:22:16Z
- **Completed:** 2026-07-16T17:37:00Z
- **Tasks:** 2/2
- **Files modified:** 9 (+ 4 new under auth/codex/)

## Accomplishments

- Implemented `auth/codex/` mirror of openai/codex wire contract (issuer, public client_id, 1455→1457, localhost `/auth/callback`, deviceauth paths, originator=bum)
- Fail-closed OAuth: state mismatch / IdP error / exchange failure never write `providers.codex`; sibling xAI preserved on success
- `expires_at` from `expires_in` or JWT `exp`, with 5-minute conservative fallback (never multi-day silent TTL)
- CLI: `run_cli_login_for_provider` + pager-bin maps `--provider codex` → Codex path **without** `managed_config::post_login_sync`
- AUTH-02 integration contracts GREEN (authorize, persist, state, device endpoints, device multi-step denied)

## Task Commits

1. **Task 1: Codex module constants, PKCE URL, claims, mock exchange persist + failure isolation** — `f2e8d1c` (feat)
2. **Task 2: Codex device-code multi-step + CLI login provider dispatch** — `aa90a03` (feat)

**Plan metadata:** (docs commit after SUMMARY/STATE)

## Files Created/Modified

- `crates/codegen/xai-grok-shell/src/auth/codex/mod.rs` — constants + `run_codex_login`
- `crates/codegen/xai-grok-shell/src/auth/codex/browser.rs` — authorize URL, loopback, exchange, persist, callback apply
- `crates/codegen/xai-grok-shell/src/auth/codex/claims.rs` — JWT email / chatgpt_account_id / exp
- `crates/codegen/xai-grok-shell/src/auth/codex/device.rs` — deviceauth usercode/poll + inject base
- `crates/codegen/xai-grok-shell/src/auth/oidc/protocol.rs` — `generate_pkce` / `Pkce` pub(crate)
- `crates/codegen/xai-grok-shell/src/auth/mod.rs` — `pub mod codex` + `run_cli_login_for_provider` export
- `crates/codegen/xai-grok-shell/src/auth/flow.rs` — provider dispatch; Codex early return
- `crates/codegen/xai-grok-pager-bin/src/main.rs` — Login arm provider mapping
- `crates/codegen/xai-grok-shell/tests/auth_codex_lifecycle.rs` — AUTH-02 GREEN contracts

## Decisions Made

- **CODEX_AUTH_SCOPE** = `https://auth.openai.com::app_EMoamEEZ73f0CkXaXp7hrann` (issuer::client_id parity)
- **Conservative TTL** = 5 minutes when neither `expires_in` nor JWT `exp` is parseable
- **Device flow** = proprietary Codex deviceauth JSON (usercode → poll auth code → oauth/token exchange), not xAI RFC 8628
- **No Platform obtain_api_key** (COVERAGE OPT-OUT)
- **Mock IdP** via axum listener + `run_codex_device_poll_only_with_base` (no wiremock package added)

## Deviations from Plan

### Auto-fixed Issues

None - plan executed as written.

### Notes

- **Pre-existing:** `cargo test -p xai-grok-pager --lib` fails with unrelated theme/prompt/test_support compile errors (out of scope). Clap test `bum_login_provider_codex_parses` remains in source from Plan 01; `cargo check -p xai-grok-pager-bin` proves Login provider wiring compiles.
- Claims unit tests live under `codex/claims.rs` `#[cfg(test)]` but full `--lib` is blocked by the same pre-existing workspace test_support breakage; claims covered via `codex_login_persists_slot` inject path.

## AUTH-02 verification (GREEN)

```text
cargo test -p xai-grok-shell --test auth_codex_lifecycle codex_authorize_url_includes_pkce_and_localhost_callback  # ok
cargo test -p xai-grok-shell --test auth_codex_lifecycle codex_login_persists_slot  # ok
cargo test -p xai-grok-shell --test auth_codex_lifecycle codex_oauth_state_mismatch_writes_nothing  # ok
cargo test -p xai-grok-shell --test auth_codex_lifecycle codex_device_endpoints_use_deviceauth  # ok
cargo test -p xai-grok-shell --test auth_codex_lifecycle codex_device_pending_slowdown_exchange_denied  # ok
```

## Threat mitigations applied

| ID | Mitigation |
|----|------------|
| T-05-06 | Cryptographic state compare; no-write tests |
| T-05-07 | Errors never print access/refresh tokens |
| T-05-08 | Fixed localhost redirect path/ports only |
| T-05-09 | `mutate_provider_store_or_prune(AuthProvider::Codex)` only; sibling preserve test |
| T-05-22 | Codex path returns before `post_login_sync` |

## Known Stubs

None that block AUTH-02. Live ChatGPT browser smoke remains optional (D-13).

## Self-Check: PASSED

- `crates/codegen/xai-grok-shell/src/auth/codex/mod.rs` FOUND
- `crates/codegen/xai-grok-shell/src/auth/codex/browser.rs` FOUND
- `crates/codegen/xai-grok-shell/src/auth/codex/claims.rs` FOUND
- `crates/codegen/xai-grok-shell/src/auth/codex/device.rs` FOUND
- Commit `f2e8d1c` FOUND
- Commit `aa90a03` FOUND
