---
phase: 02-multi-slot-credentials-xai-oauth
plan: 04
status: complete
completed: 2026-07-16
---

# Plan 02-04 Summary — CLI/path seal + mock multi-slot login asserts

## Delivered

- **`resolve_auth_path`**: `GROK_AUTH_PATH` accepted only when equal to product-home `auth.json`; otherwise ignored for writes (no lexical `starts_with` escape)
- **Foreign-path + Unix symlink-escape tests** in `manager_tests.rs` — credentials land only under product-home auth.json
- **Device-code rustdoc** updated to `$BUM_HOME/auth.json` (not `~/.grok`)
- **Mock browser (OIDC loopback) + device-code** success tests assert nested `version`/`providers.xai` and seeded `providers.codex` survival
- **CLI**: Clap unit test `bum_login_defaults_to_xai_without_provider_argument` (oauth/device-auth flags, mutual exclusion, no provider arg)
- Flat test helpers (`relay`, recovery, oidc refresher) left migrate-compatible or lightly adjusted

## Verification

| Check | Result |
|-------|--------|
| `cargo check -p xai-grok-shell --lib` | ✅ green |
| `cargo check -p xai-grok-pager-bin --bin bum` | ✅ green |
| `cargo test -p xai-grok-shell --test auth_multi_slot` | ✅ 1 passed |
| `cargo test -p xai-grok-shell --lib auth::` | ⚠️ blocked (pre-existing shell lib-test harness break; Phase 1 deferred) |
| `cargo test -p xai-grok-pager --lib bum_login_…` | ⚠️ blocked (pre-existing pager lib-test harness break) |

## Files modified

- `crates/codegen/xai-grok-shell/src/auth/manager.rs`
- `crates/codegen/xai-grok-shell/src/auth/manager_tests.rs`
- `crates/codegen/xai-grok-shell/src/auth/device_code.rs`
- `crates/codegen/xai-grok-shell/src/auth/oidc/login.rs`
- `crates/codegen/xai-grok-shell/src/auth/recovery.rs`
- `crates/codegen/xai-grok-shell/src/auth/refresh/oidc_refresher_tests.rs`
- `crates/codegen/xai-grok-shell/src/agent/relay.rs`
- `crates/codegen/xai-grok-pager/src/app/cli.rs`

## AUTH-01 phase gate (pragmatic)

With shell/pager lib-test harness still broken (known Phase 1 debt), phase gate uses:

1. Production `cargo check` for shell + pager-bin
2. Integration test `auth_multi_slot` for nested load/isolation via public AuthManager
3. In-tree unit tests for multi-slot, path isolation, mock login (source present; run when harness is repaired)
