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
