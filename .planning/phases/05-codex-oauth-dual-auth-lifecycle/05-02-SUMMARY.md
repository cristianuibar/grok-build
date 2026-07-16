---
phase: 05-codex-oauth-dual-auth-lifecycle
plan: 02
subsystem: auth
tags: [oauth, codex, dual-auth, multi-slot, status, auth-provider, storage]

requires:
  - phase: 05-codex-oauth-dual-auth-lifecycle
    provides: Wave 0 auth_codex_lifecycle harness + multi-slot auth.json readers
  - phase: 02-multi-slot-credentials
    provides: providers map, mutate_xai_store_or_prune, AuthFileLock still_live pattern
provides:
  - Public typed AuthProvider (Xai|Codex) with allow-list parse
  - Public mutate_provider_store_or_prune / clear_provider_slot / clear_all_provider_slots
  - Guard-held clear_provider_slot_with_lock / mutate_provider_store_or_prune_with_lock (no reacquire)
  - Pure AuthStatusReport + format_auth_status with correct logged_in/usable semantics
affects:
  - 05-03 Codex OAuth login (persist via mutate_provider_store_or_prune Codex)
  - 05-04 selective logout CLI + auth status CLI handler
  - 05-05 ensure_fresh permanent-fail via clear_provider_slot_with_lock

tech-stack:
  added: []
  patterns:
    - Typed AuthProvider enum for mutation (fail closed; no arbitrary map keys)
    - Provider-parameterized RMW with still_live ×2 + prune-when-all-empty
    - Status usable ≠ select_provider_access_token Some (hard-unexpired or refreshable)
    - Public path-taking APIs for integration tests (OnceLock-safe explicit auth_file)

key-files:
  created:
    - crates/codegen/xai-grok-shell/src/auth/status.rs
  modified:
    - crates/codegen/xai-grok-shell/src/auth/model.rs
    - crates/codegen/xai-grok-shell/src/auth/storage.rs
    - crates/codegen/xai-grok-shell/src/auth/mod.rs
    - crates/codegen/xai-grok-shell/tests/auth_codex_lifecycle.rs

key-decisions:
  - "AuthProvider::{Xai,Codex} with as_str → xai|codex; parse allow-list only"
  - "XaiStoreMutation generalized to public ProviderStoreMutation; xAI helpers delegate"
  - "with_lock mutate/clear are pub(crate) (AuthFileLock crate-private); acquiring forms are public"
  - "usable: hard-unexpired access (zero buffer) OR nonblank refresh_token; not select_provider_access_token"
  - "logged_in: nonblank access and/or refresh_token (skip WebLogin)"
  - "Status format UI-SPEC: xAI then Codex, greppable keys, plan em-dash, never emit tokens"
  - "CLI logout/status handlers remain RED (Plan 04); storage primitives GREEN"

patterns-established:
  - "mutate_provider_store_or_prune(path, AuthProvider, f) for all dual-slot writes"
  - "clear_provider_slot_with_lock while holding ensure_fresh lock (never reacquire)"
  - "AuthStatusReport::from_auth_file + format_auth_status for paste-safe inspect"

requirements-completed: [AUTH-03, AUTH-04]

coverage:
  - id: D1
    description: Public provider-parameterized multi-slot mutate/clear with sibling isolation and last-slot file prune
    requirement: AUTH-03
    verification:
      - kind: integration
        ref: crates/codegen/xai-grok-shell/tests/auth_codex_lifecycle.rs#mutate_provider_codex_preserves_xai
        status: pass
      - kind: integration
        ref: crates/codegen/xai-grok-shell/tests/auth_codex_lifecycle.rs#mutate_provider_xai_preserves_codex
        status: pass
      - kind: integration
        ref: crates/codegen/xai-grok-shell/tests/auth_codex_lifecycle.rs#clear_provider_codex_leaves_xai
        status: pass
      - kind: integration
        ref: crates/codegen/xai-grok-shell/tests/auth_codex_lifecycle.rs#clear_last_provider_deletes_file
        status: pass
      - kind: integration
        ref: crates/codegen/xai-grok-shell/tests/auth_codex_lifecycle.rs#clear_all_provider_slots_empties_both
        status: pass
      - kind: integration
        ref: crates/codegen/xai-grok-shell/tests/auth_multi_slot.rs#public_auth_manager_loads_xai_from_nested_multi_slot_document
        status: pass
    human_judgment: false
  - id: D2
    description: Pure dual-provider status with paste-safe format and correct usable semantics
    requirement: AUTH-04
    verification:
      - kind: integration
        ref: crates/codegen/xai-grok-shell/tests/auth_codex_lifecycle.rs#auth_status_format_paste_safe
        status: pass
      - kind: integration
        ref: crates/codegen/xai-grok-shell/tests/auth_codex_lifecycle.rs#auth_status_usable_expired_refreshable
        status: pass
      - kind: integration
        ref: crates/codegen/xai-grok-shell/tests/auth_codex_lifecycle.rs#auth_status_usable_expired_no_refresh
        status: pass
      - kind: integration
        ref: crates/codegen/xai-grok-shell/tests/auth_codex_lifecycle.rs#auth_status_both_providers_always_listed
        status: pass
      - kind: integration
        ref: crates/codegen/xai-grok-shell/tests/auth_codex_lifecycle.rs#auth_status_asymmetric_login
        status: pass
    human_judgment: false

duration: 7min
completed: 2026-07-16
status: complete
---

# Phase 5 Plan 02: Public multi-slot storage + paste-safe status Summary

**Public AuthProvider-parameterized RMW/clear plus pure dual-provider status with correct logged_in/usable semantics (AUTH-03/04 core GREEN)**

## Performance

- **Duration:** 7 min
- **Started:** 2026-07-16T17:13:42Z
- **Completed:** 2026-07-16T17:21:09Z
- **Tasks:** 2/2
- **Files modified:** 5 (created 1, modified 4)

## Accomplishments

- Exported typed `AuthProvider` and public path-taking `mutate_provider_store_or_prune`, `clear_provider_slot`, `clear_all_provider_slots` callable from integration tests
- Guard-held `clear_provider_slot_with_lock` / `mutate_provider_store_or_prune_with_lock` never reacquire (Plan 05 permanent-fail path)
- Rewired `mutate_xai_store_or_prune*` to generic helper with `AuthProvider::Xai`; `ProviderStoreMutation` replaces xAI-only enum
- Pure `auth/status.rs`: `logged_in` vs `usable` (hard-unexpired **or** refreshable); greppable UI-SPEC format never emits tokens
- Isolation + paste-safe + usable lifecycle tests GREEN; `auth_multi_slot` regression green; login/refresh/CLI handlers stay RED

## Task Commits

Each task was committed atomically (coupled surface landed in one feat):

1. **Task 1: Public AuthProvider-slot mutate/clear RMW** - `96773c2` (feat)
2. **Task 2: Pure dual-provider status + usable semantics** - `96773c2` (feat)

**Plan metadata:** (pending docs commit)

## Files Created/Modified

- `crates/codegen/xai-grok-shell/src/auth/model.rs` — `AuthProvider` enum + parse/label/all
- `crates/codegen/xai-grok-shell/src/auth/storage.rs` — public provider RMW/clear/clear_all; with_lock variants
- `crates/codegen/xai-grok-shell/src/auth/status.rs` — pure status report + formatter
- `crates/codegen/xai-grok-shell/src/auth/mod.rs` — public re-exports
- `crates/codegen/xai-grok-shell/tests/auth_codex_lifecycle.rs` — isolation/status GREEN; CLI logout/status still RED for Plan 04

## Decisions Made

- Typed provider identity only — unknown strings fail at `AuthProvider::parse`, never reach the map writer
- Acquiring public APIs for integration; with_lock stays `pub(crate)` with crate-private `AuthFileLock`
- Status usable must not use `select_provider_access_token` alone (expired fallback would false-positive usable)
- Permanently-invalid is in-memory AuthManager concern; disk status treats nonblank refresh as refreshable

## Deviations from Plan

### Auto-fixed Issues

None - plan executed as written.

### Notes

- Tasks 1–2 shared one feat commit because status exports and storage APIs are required together for a compiling `auth/mod.rs` public surface; both task acceptance criteria verified.
- `cargo test -p xai-grok-shell --lib` remains pre-broken (workspace `WorkspaceOps::for_test` / unrelated test_support) — out of scope; deferred from Phase 5 gates per Wave 0 hygiene.

## Green test names (Plan 02)

**Storage isolation:**
- `mutate_provider_codex_preserves_xai`
- `mutate_provider_xai_preserves_codex`
- `clear_provider_codex_leaves_xai`
- `clear_last_provider_deletes_file`
- `clear_all_provider_slots_empties_both`
- `auth_provider_parse_rejects_unknown`
- `public_auth_manager_loads_xai_from_nested_multi_slot_document` (auth_multi_slot)
- `auth_codex_lifecycle_harness_smoke`

**Status:**
- `auth_status_format_paste_safe`
- `auth_status_usable_expired_refreshable`
- `auth_status_usable_expired_no_refresh`
- `auth_status_both_providers_always_listed`
- `auth_status_asymmetric_login`

## Still RED (later plans)

- Plan 03: login / PKCE / device
- Plan 04: CLI `selective_logout_isolates`, `logout_all_clears_both`, `bare_logout_fail_closed`, `run_cli_auth_status`
- Plan 05: refresh isolation / headers

## Threat Flags

None beyond plan register — no new network endpoints; status formatter mitigates T-05-03 (paste-safe tests); typed mutate mitigates T-05-04; with_lock clear mitigates T-05-26.

## Known Stubs

None that block Plan 02 goals. CLI handlers intentionally RED until Plan 04.

## Self-Check: PASSED

- FOUND: `crates/codegen/xai-grok-shell/src/auth/status.rs`
- FOUND: `crates/codegen/xai-grok-shell/src/auth/storage.rs` (`mutate_provider_store_or_prune`)
- FOUND: `AuthProvider` in `model.rs`
- FOUND commit: `96773c2`
- Verified filters: `mutate_provider`, `clear_provider`, `auth_status_format_paste_safe`, `auth_status_usable`, `auth_multi_slot`
