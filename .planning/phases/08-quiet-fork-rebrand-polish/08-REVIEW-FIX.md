---
phase: 08-quiet-fork-rebrand-polish
fixed_at: 2026-07-17T12:23:01Z
review_path: .planning/phases/08-quiet-fork-rebrand-polish/08-REVIEW.md
iteration: 1
findings_in_scope: 6
fixed: 4
skipped: 2
status: partial
---

# Phase 8: Code Review Fix Report

**Fixed at:** 2026-07-17T12:23:01Z  
**Source review:** `.planning/phases/08-quiet-fork-rebrand-polish/08-REVIEW.md`  
**Iteration:** 1

**Summary:**
- Findings in scope: 6 (WR-01 … WR-06; Info skipped per scope)
- Fixed: 4
- Skipped: 2

## Fixed Issues

### WR-01: Leader auto-update check still treats `None` as enabled

**Files modified:** `crates/codegen/xai-grok-pager-bin/src/main.rs`  
**Commit:** `06b23af`  
**Applied fix:** Leader `check_fn` now uses `auto_update::effective_auto_update_enabled` so `None` and `Some(false)` both refuse the stock path (D-07), instead of only skipping on `Some(false)`.

### WR-02: `ensure_latest_on_disk` has no auto-update / quiet-fork gate

**Files modified:**
- `crates/codegen/xai-grok-update/src/auto_update.rs`
- `crates/codegen/xai-grok-update/tests/common/mod.rs`
- `crates/codegen/xai-grok-update/tests/test_concurrent_convergence.rs`
- `crates/codegen/xai-grok-update/tests/test_downgrade_matrix.rs`

**Commit:** `4550944`  
**Applied fix:** Library entry loads config and returns a no-op `EnsureLatestOutcome` when `effective_auto_update_enabled` is false (before network/install). Integration tests that intentionally exercise install call `enable_stock_auto_update()` after setup.

### WR-03: Auto-update setting remains user-toggleable and claims “restart to apply”

**Files modified:**
- `crates/codegen/xai-grok-pager/src/app/dispatch/settings/setters.rs`
- `crates/codegen/xai-grok-pager/src/settings/defs.rs`
- `crates/codegen/xai-grok-pager/src/app/dispatch/tests/router.rs`

**Commit:** `e602cf3`  
**Applied fix:** `set_auto_update(true)` shows toast “Stock auto-update is permanently disabled in bum.” and returns no effects (no persist). Setting description updated to “permanently disabled … cannot re-enable.” Added `p8_set_auto_update_true_is_refused`.

### WR-04: Stock-update test seams never wrap production call sites

**Files modified:** `crates/codegen/xai-grok-pager-bin/src/main.rs`  
**Commit:** `b15678e`  
**Applied fix:** Production paths now go through seams:
- startup / stdio → `stock_run_update_if_available`
- leader hourly → `stock_ensure_latest_on_disk`
- TUI background → `stock_check_update_background`

Seams still increment `STOCK_UPDATE_HELPER_CALLS` under `cfg(test)`.

## Skipped Issues

### WR-05: TUI feedback hard-off is unconditional; opt-in config is dead for slash path

**File:** `crates/codegen/xai-grok-pager/src/app/dispatch/notes.rs`  
**Reason:** Not low-risk — requires choosing a product model (permanent TUI hard-off vs config-gated re-enable) and potentially changing ACP/`FeedbackManager` network paths. Intentional hard-off remains; deferred to a deliberate product decision.  
**Original issue:** Dual model: TUI always local-only while ACP/debug can still network when feedback is enabled.

### WR-06: `p8_feedback_force_request_*` tests use sleep-based network assertions

**File:** `crates/codegen/xai-grok-shell/src/session/feedback_manager.rs`  
**Reason:** Not low-risk flake rewrite — needs mock client / joinable task design rather than a quick sleep tweak. Deferred.  
**Original issue:** Fixed `50ms` / poll-to-`500ms` timing can false-pass or false-fail under load.

## Info findings

Not in fix scope (user: skip info-level nitpicks). Deferred: IN-01 … IN-05.

## Test results

| Suite | Result |
|-------|--------|
| `cargo test -p xai-grok-update --lib p8_` | **4 passed** |
| `cargo test -p xai-grok-pager-bin --bin bum p8_` | **10 passed** |
| `cargo test -p xai-grok-pager --lib p8_` | **18 passed** (includes new `p8_set_auto_update_true_is_refused`) |

No model-brand / `grok-build` catalog surfaces were touched. Green `p8_` gates held.

---

_Fixed: 2026-07-17T12:23:01Z_  
_Fixer: Claude (gsd-code-fixer)_  
_Iteration: 1_
