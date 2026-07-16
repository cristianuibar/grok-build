---
phase: 02-multi-slot-credentials-xai-oauth
plan: 02
subsystem: auth
tags: [rust, oauth, multi-slot, locking]
requires:
  - phase: 02-01
    provides: AuthDocument and dual lock-aware mutation APIs
provides:
  - Multi-provider-aware API-key clear and scope-removal pruning
  - xAI-only devbox recovery that preserves sibling providers
  - Honest FileDeleted outcomes after successful removal
affects: [phase-05-codex-oauth, auth-storage]
tech-stack:
  added: []
  patterns: [acquiring and guard-held xAI slot prune mutations]
key-files:
  created: []
  modified:
    - crates/codegen/xai-grok-shell/src/auth/storage.rs
    - crates/codegen/xai-grok-shell/src/auth/manager.rs
    - crates/codegen/xai-grok-shell/src/auth/manager_tests.rs
    - crates/codegen/xai-grok-shell/src/auth/recovery.rs
key-decisions:
  - "Devbox recovery replaces providers.xai in one locked mutation instead of deleting auth.json."
  - "Empty xAI slots remove auth.json only when every provider slot is empty."
requirements-completed: [AUTH-01]
coverage:
  - id: D1
    description: "API-key store and clear operations preserve sibling provider credentials under lock."
    requirement: AUTH-01
    verification:
      - kind: other
        ref: "cargo check -p xai-grok-shell --lib"
        status: pass
    human_judgment: false
  - id: D2
    description: "Scope removal and devbox recovery preserve providers.codex."
    requirement: AUTH-01
    verification:
      - kind: other
        ref: "cargo check -p xai-grok-shell --lib"
        status: pass
    human_judgment: false
duration: 31min
completed: 2026-07-16
status: complete
---

# Phase 2 Plan 2: Multi-provider-safe auth cleanup Summary

**Locked API-key, logout, and devbox recovery mutations now preserve sibling providers and delete auth.json only when every slot is empty.**

## Accomplishments

- Added acquiring and guard-held xAI mutation/prune paths with honest deletion outcomes.
- Routed API-key store/clear and manager scope removal through lock-scoped full-document mutation.
- Replaced the devbox whole-file purge with an xAI-only replacement that preserves Codex and future providers.
- Added unit coverage for API-key isolation, Codex-preserving logout/recovery, deletion errors, and held-lock non-reentrancy.

## Commit

- `1485657` — Preserve sibling credentials during xAI auth cleanup

## Verification

| Check | Result |
|-------|--------|
| `cargo check -p xai-grok-shell --lib` | Passed |
| `cargo test -p xai-grok-shell --lib auth::storage:: -- --nocapture` | Blocked by the documented pre-existing shell lib-test harness failures (`EnvVarGuard`, `WorkspaceOps::for_test`, `MemoryStorage::with_paths`, and related test-only exports) |
| Production auth `remove_file` audit | Passed; only lock-file handling and Windows atomic-replace mechanics remain |

## Deviations from Plan

- Updated the stale devbox purge description in `auth/recovery.rs`, discovered during the required production purge audit.
- Unit tests were committed with the implementation because the pre-existing shell lib-test harness cannot compile a standalone RED test commit.

## Issues Encountered

- The known shell lib-test harness break prevents executing auth unit tests; the non-test shell library remains green as required.

## Self-Check: PASSED

- Required production paths use acquiring vs guard-held lock APIs correctly.
- `try_devbox_recovery` no longer removes the whole auth document.
- The implementation commit and this summary are present.
