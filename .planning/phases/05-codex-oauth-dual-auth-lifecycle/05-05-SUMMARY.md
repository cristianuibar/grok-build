---
phase: 05-codex-oauth-dual-auth-lifecycle
plan: 05
subsystem: auth
tags: [codex, oauth, refresh, ensure_fresh, reconstruct, session-token, chatgpt-account-id]

requires:
  - phase: 05-03
    provides: Codex OAuth login/persist into providers.codex
  - phase: 05-04
    provides: dual logout + status; clear_provider_slot_with_lock
provides:
  - Pure CodexRefresher (TokenRefresher data-only)
  - ensure_fresh_codex_auth with lock-held RT rotation + sibling adopt
  - reconstruct_full_config production invoker (SessionToken + session_oauth_allowed)
  - CodexAuthMaterial typed return into request construction
  - Trusted-host ChatGPT-Account-ID inject
  - Option C reconstruct seam tests + isolation/concurrency suite
affects:
  - 05-06 phase gate
  - Phase 6 missing-provider UX (not started)

tech-stack:
  added: []
  patterns:
    - "TokenRefresher purity: data only; outer ensure owns persist/clear"
    - "auth.json.lock held across IdP + guard-held mutate/clear_with_lock"
    - "reconstruct OAuth gate: Codex + SessionToken + first-party URL"

key-files:
  created:
    - crates/codegen/xai-grok-shell/src/auth/codex/refresh.rs
    - crates/codegen/xai-grok-shell/src/auth/codex/ensure_fresh.rs
    - crates/codegen/xai-grok-shell/src/auth/refresh/codex_refresher.rs
    - crates/codegen/xai-grok-shell/src/session/acp_session_tests/codex_reconstruct_refresh_tests.rs
  modified:
    - crates/codegen/xai-grok-shell/src/session/acp_session_impl/sampler_turn.rs
    - crates/codegen/xai-grok-shell/src/agent/config.rs
    - crates/codegen/xai-grok-shell/src/auth/codex/mod.rs
    - crates/codegen/xai-grok-shell/src/auth/refresh/mod.rs
    - crates/codegen/xai-grok-shell/src/session/acp_session.rs
    - crates/codegen/xai-grok-shell/tests/auth_codex_lifecycle.rs
    - crates/codegen/xai-grok-env/src/lib.rs
    - crates/codegen/xai-grok-workspace/src/workspace_ops.rs
    - crates/codegen/xai-grok-workspace/src/handle.rs
    - crates/codegen/xai-grok-memory/src/storage.rs
    - crates/codegen/xai-grok-memory/src/embedding.rs
    - crates/codegen/xai-grok-shell-base/src/env.rs
    - crates/codegen/xai-grok-shell-base/src/cpu_profile.rs

key-decisions:
  - "Production AUTH-05 hook is SessionActor::reconstruct_full_config, not ModelsManager prepare alone"
  - "Bearer override only when Codex + AuthType::SessionToken + first-party Codex URL"
  - "Permanent fail uses clear_provider_slot_with_lock while holding lock (never reacquire)"
  - "Synthetic IdP hooks for isolation tests without BUM_HOME OnceLock mutation"

patterns-established:
  - "Option C: crate-local --lib reconstruct seam tests prove wiring"
  - "Dependency test helpers under cfg(any(test, debug_assertions)) for --lib dependents"

requirements-completed: [AUTH-05]

coverage:
  - id: D1
    description: Pure Codex refresh identity preserve when RT omitted
    requirement: AUTH-05
    verification:
      - kind: integration
        ref: cargo test -p xai-grok-shell --test auth_codex_lifecycle codex_refresh_preserves_identity
        status: pass
    human_judgment: false
  - id: D2
    description: Option C reconstruct mid-session refresh seam
    requirement: AUTH-05
    verification:
      - kind: unit
        ref: cargo test -p xai-grok-shell --lib codex_reconstruct_refreshes_mid_session_expiry
        status: pass
    human_judgment: false
  - id: D3
    description: BYOK and custom endpoint never get OAuth bearer override
    requirement: AUTH-05
    verification:
      - kind: unit
        ref: cargo test -p xai-grok-shell --lib codex_byok_key_not_overridden
        status: pass
      - kind: unit
        ref: cargo test -p xai-grok-shell --lib codex_oauth_bearer_absent_on_custom_endpoint
        status: pass
    human_judgment: false
  - id: D4
    description: Lock-held isolation, concurrent single IdP spend, transient policies
    requirement: AUTH-05
    verification:
      - kind: integration
        ref: cargo test -p xai-grok-shell --test auth_codex_lifecycle codex_refresh_isolates
        status: pass
      - kind: integration
        ref: cargo test -p xai-grok-shell --test auth_codex_lifecycle codex_invalid_grant_no_xai_wipe
        status: pass
      - kind: integration
        ref: cargo test -p xai-grok-shell --test auth_codex_lifecycle codex_concurrent_refresh_single_idp_spend
        status: pass
      - kind: integration
        ref: cargo test -p xai-grok-shell --test auth_codex_lifecycle codex_fresh_token_skips_idp
        status: pass
      - kind: integration
        ref: cargo test -p xai-grok-shell --test auth_codex_lifecycle codex_transient_fail
        status: pass
    human_judgment: false
  - id: D5
    description: Trusted-host ChatGPT-Account-ID inject + absence matrix
    requirement: AUTH-05
    verification:
      - kind: integration
        ref: cargo test -p xai-grok-shell --test auth_codex_lifecycle chatgpt_account_id_header
        status: pass
      - kind: integration
        ref: cargo test -p xai-grok-shell --test provider_routing no_proxy_headers_on_codex
        status: pass
    human_judgment: false

duration: 29min
completed: 2026-07-16
status: complete
---

# Phase 5 Plan 05: Independent Codex refresh + reconstruct invoker Summary

**Per-request Codex ensure_fresh on reconstruct_full_config with lock-held RT rotation, pure CodexRefresher, BYOK/custom gates, and typed CodexAuthMaterial (AUTH-05)**

## Performance

- **Duration:** 29 min
- **Started:** 2026-07-16T17:50:02Z
- **Completed:** 2026-07-16T18:19:25Z
- **Tasks:** 3/3
- **Files modified:** 16+

## Accomplishments

- Pure `codex/refresh.rs` + data-only `CodexRefresher` (TokenRefresher purity; identity preserved when IdP omits RT/claims)
- `ensure_fresh_codex_auth` holds `auth.json.lock` across re-read → IdP → guard-held persist; permanent fail via `clear_provider_slot_with_lock` only
- Production wire: `SessionActor::reconstruct_full_config` for Codex + SessionToken + first-party host only
- Option C seam tests GREEN (`--lib` narrow filters); isolation/concurrency/transient integration GREEN
- Trusted-host `ChatGPT-Account-ID` inject + absence matrix; `no_proxy_headers_on_codex` regression green

## Task Commits

1. **Task 1: Pure Codex refresh + CodexRefresher + identity** - `8df472a` (feat)
2. **Task 2: ensure_fresh + reconstruct wire + Option C seam** - `e656fc6` (feat)
3. **Task 3: Trusted-host ChatGPT-Account-ID inject** - delivered in `e656fc6` (header helper + matrix tests co-landed with Task 2)

## Files Created/Modified

- `auth/codex/refresh.rs` — pure refresh_token grant exchange + merge identity
- `auth/codex/ensure_fresh.rs` — lock-held ensure_fresh + synthetic IdP test hooks
- `auth/refresh/codex_refresher.rs` — TokenRefresher data-only
- `session/acp_session_impl/sampler_turn.rs` — reconstruct OAuth gate + material apply
- `agent/config.rs` — re-exports, snapshot invalidate, `inject_chatgpt_account_id_header`
- `session/acp_session_tests/codex_reconstruct_refresh_tests.rs` — Option C trio
- `tests/auth_codex_lifecycle.rs` — isolation/concurrency/transient/header GREEN
- Dependency test helpers (`debug_assertions`) so shell `--lib` tests compile

## Decisions Made

- Followed locked cycle 1–3 redesign: reconstruct production hook, SessionToken + session_oauth_allowed gate, clear_with_lock, Option C seam
- Synthetic IdP hooks (no BUM_HOME mutation) for concurrent/isolation proofs
- Permanent fail returns no material and clears prepared api_key on eligible reconstruct path

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Shell `--lib` tests failed to compile due to cfg(test)-only helpers in dependencies**
- **Found during:** Task 2 (Option C seam)
- **Issue:** `WorkspaceOps::for_test`, `EnvVarGuard`, `MemoryStorage::with_paths`, `MockEmbeddingProvider`, cpu-profile test methods only existed under `cfg(test)` on their defining crates — invisible when shell runs `--lib` against dependency builds
- **Fix:** Expand visibility to `cfg(any(test, debug_assertions))` for those helpers; fix missing `PathBuf` import in `extensions/bundle.rs` tests
- **Files modified:** xai-grok-workspace, xai-grok-memory, xai-grok-env, xai-grok-shell-base, shell extensions/bundle.rs
- **Verification:** `cargo test -p xai-grok-shell --lib codex_reconstruct_*` compiles and passes
- **Committed in:** `e656fc6`

**2. [Rule 2 - Critical] Task 3 header inject co-landed with Task 2**
- **Found during:** Task 2 implementation (same reconstruct path needs account header)
- **Issue:** Account header is part of reconstruct material application; splitting commits would leave Task 2 mid-state without headers
- **Fix:** Implemented `inject_chatgpt_account_id_header` and positive/absence tests with Task 2 commit
- **Verification:** chatgpt_account_id_header_* + no_proxy_headers_on_codex green

## AUTH-05 Test Results (all green)

| Test | Filter | Result |
|------|--------|--------|
| identity preserve | `--test auth_codex_lifecycle codex_refresh_preserves_identity` | PASS |
| reconstruct mid-session | `--lib codex_reconstruct_refreshes_mid_session_expiry` | PASS |
| BYOK not overridden | `--lib codex_byok_key_not_overridden` | PASS |
| custom endpoint no OAuth | `--lib codex_oauth_bearer_absent_on_custom_endpoint` | PASS |
| refresh isolates | `--test auth_codex_lifecycle codex_refresh_isolates` | PASS |
| invalid_grant no xAI wipe | `--test auth_codex_lifecycle codex_invalid_grant_no_xai_wipe` | PASS |
| concurrent single spend | `--test auth_codex_lifecycle codex_concurrent_refresh_single_idp_spend` | PASS |
| fresh skips IdP | `--test auth_codex_lifecycle codex_fresh_token_skips_idp` | PASS |
| transient pair | `--test auth_codex_lifecycle codex_transient_fail` | PASS |
| account header matrix | `--test auth_codex_lifecycle chatgpt_account_id_header` | PASS |
| no_proxy regression | `--test provider_routing no_proxy_headers_on_codex` | PASS |

## Known Stubs

None — ensure_fresh, reconstruct wire, and header inject are fully implemented (synthetic IdP is intentional test inject, not a production stub).

## Threat Flags

None new beyond plan `<threat_model>` (T-05-15..26 mitigated as designed).

## Self-Check: PASSED
