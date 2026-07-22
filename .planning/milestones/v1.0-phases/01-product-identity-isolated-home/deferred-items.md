# Deferred items (phase 01)

## Pre-existing: xai-grok-shell lib tests do not compile

**Found during:** 01-02 Task 2 verification (`cargo test -p xai-grok-shell leader`)

**Issue:** Compiling `xai-grok-shell` with `cfg(test)` fails with ~31 errors unrelated to managed-bin / product-home changes, including:

- `crate::env::EnvVarGuard` unresolved
- `WorkspaceOps::for_test()` missing
- `MemoryStorage::with_paths` missing
- `CpuProfileManager::{start_with_engine_for_test,force_unsupported_for_test}` missing
- `MockEmbeddingProvider` unresolved

**Impact:** Cannot run in-crate leader unit tests (including `resolve_binary_*`) via `cargo test -p xai-grok-shell --lib` until the broader test harness is repaired.

**Mitigation used in 01-02:** `cargo check -p xai-grok-shell --lib` (production) green; source inspection confirms `managed_grok_bin_name` returns `bum`/`bum.exe` and hard-coded test fixtures use that helper.

**Not caused by:** 01-02 managed leaf rename or twin home work.

**Reconfirmed during:** 01-04 Task 3 (`cargo test -p xai-grok-shell leader`) — same pre-existing lib-test break; product-home fixture cutover verified via shell **integration** tests (`test_config_update_isolation`, `test_mcp_permission_persistence`, `test_debug_logging` compile) and workspace lib filters instead.

## Pre-existing: permission::resolution tests pick up host `~/.claude`

**Found during:** 01-04 Task 3 (`cargo test -p xai-grok-workspace --lib permission::resolution`)

**Issue:** Several resolution tests that do **not** isolate `HOME` can merge the developer’s real `~/.claude/settings*.json` (e.g. `discovery_with_no_settings_files`, `claude_only_returns_claude_settings_source` — expected 1 rule, got host merge). Product-home–isolated cases (`load_claude_env_*` with `BUM_HOME` + optional `HOME` guards) pass.

**Not caused by:** BUM_HOME product-home setter cutover (those failing tests never set the product-home env key).

## Reconfirmed during 01-05

**Found during:** Task 1 (`cargo test -p xai-grok-shell bundle|config`)

Same pre-existing `xai-grok-shell` lib-test compile break (~32 errors). Mitigated via:
- `cargo check -p xai-grok-shell --lib`
- `cargo test -p xai-grok-shell --test test_bundled_root_product_home`
- `cargo test -p xai-grok-agent discovery`
