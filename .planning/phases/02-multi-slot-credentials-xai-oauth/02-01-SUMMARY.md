---
phase: 02-multi-slot-credentials-xai-oauth
plan: 01
status: complete
completed: 2026-07-16
---

# Plan 02-01 Summary — Multi-slot schema + dual mutation APIs

## Delivered

- **`AuthDocument`** with optional `version` and `providers: BTreeMap<String, AuthStore>` (`pub(crate)`)
- Constants: `PROVIDER_XAI` (`"xai"`), `PROVIDER_CODEX` (`"codex"`), `AUTH_DOCUMENT_VERSION` (`1`)
- **Read path:** nested document → xAI slot; legacy flat → treat as xAI; unsupported `version > 1` → fail-closed (`Unsupported` error, not corrupt-empty recovery)
- **Dual mutation APIs:**
  - Acquiring: `mutate_auth_document` / `write_auth_json`
  - Guard-held: `mutate_auth_document_with_lock` / `write_auth_json_with_lock` (never re-acquires lock; `still_live` checks)
- Eager nested rewrite on write; preserves sibling providers (e.g. seeded `providers.codex`)
- Manager rewire: startup cleanup, scope-removal persist, `update`/`save` with optional held lock, enrichment with-lock vs acquiring on timeout
- Unit tests in `storage.rs` for migrate, nested write, codex survival, concurrency, reentrancy, unsupported version, fixture helper

## Verification

| Check | Result |
|-------|--------|
| `cargo check -p xai-grok-shell --lib` | ✅ green |
| `cargo test -p xai-grok-shell --lib auth::storage::` | ⚠️ **blocked** by pre-existing shell lib-test harness break (same as Phase 1 deferred-items: `WorkspaceOps::for_test`, `MemoryStorage::with_paths`, `EnvVarGuard`, …) — not introduced by this plan |

## Files modified

- `crates/codegen/xai-grok-shell/src/auth/model.rs`
- `crates/codegen/xai-grok-shell/src/auth/storage.rs`
- `crates/codegen/xai-grok-shell/src/auth/manager.rs`
- `crates/codegen/xai-grok-shell/src/auth/manager/enrichment.rs`
- `crates/codegen/xai-grok-shell/src/auth/manager/lock.rs`

## Follow-ups for later plans

- Plan 02: prune/delete multi-provider-aware + API keys + `try_devbox_recovery`
- Plan 03: AuthManager/Bearer agent-turn tests
- Plan 04: GROK_AUTH_PATH isolation + mock login multi-slot asserts
- When shell lib-test harness is repaired, run full `auth::storage::` suite
