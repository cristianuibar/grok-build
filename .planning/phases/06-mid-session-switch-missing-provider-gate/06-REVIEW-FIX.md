---
phase: 06-mid-session-switch-missing-provider-gate
fixed_at: 2026-07-17T04:30:00Z
review_path: .planning/phases/06-mid-session-switch-missing-provider-gate/06-REVIEW.md
iteration: 1
findings_in_scope: 8
fixed: 8
skipped: 0
status: all_fixed
---

# Phase 6: Code Review Fix Report

**Fixed at:** 2026-07-17T04:30:00Z  
**Source review:** `.planning/phases/06-mid-session-switch-missing-provider-gate/06-REVIEW.md`  
**Iteration:** 1  

**Summary:**
- Findings in scope: 8 (1 critical + 6 warning + 1 easy info)
- Fixed: 8
- Skipped: 0

## Fixed Issues

### CR-01: Deferred recovery cleared before SwitchModel succeeds

**Files modified:** `crates/codegen/xai-grok-pager/src/app/dispatch/session/lifecycle.rs`, `crates/codegen/xai-grok-pager/src/app/dispatch/tests/task_result.rs`  
**Commit:** `4faf38a`  
**Applied fix:** `try_apply_deferred_model_switch_if_ready` no longer clears `deferred_model_switch` when emitting `Effect::SwitchModel`; sets `model_switch_pending` and skips re-entry while pending. Deferred is cleared only on `SwitchModelComplete(Ok)` when the completed model matches the stash. `Other` failures leave deferred intact and re-arm provider login poll + refresh. Added `p6_other_switch_failure_keeps_deferred_for_retry`.

### WR-01: MissingProvider QuestionView opens on `active_view`, not `agent_id`

**Files modified:** `crates/codegen/xai-grok-pager/src/app/dispatch/session/lifecycle.rs`  
**Commit:** `4faf38a`  
**Applied fix:** `open_missing_provider_login_question` and `open_agent_type_mismatch_question` take `agent_id`, open on that agent, and `switch_to_agent` when focus differs so Login now / Keep current attach to the gated session.

### WR-02: Auto-apply does not dismiss an open MissingProviderLogin modal

**Files modified:** `crates/codegen/xai-grok-pager/src/app/dispatch/session/lifecycle.rs`  
**Commit:** `4faf38a`  
**Applied fix:** Before emitting SwitchModel from `try_apply`, dismiss `LocalQuestionKind::MissingProviderLogin` and restore the stashed prompt so Keep current cannot orphan-apply after credentials land under the gate UI.

### WR-03: Busy QuestionView still stashes deferred without missing-provider UI

**Files modified:** `crates/codegen/xai-grok-pager/src/app/dispatch/session/lifecycle.rs`  
**Commit:** `4faf38a`  
**Applied fix:** Gate-open path stashes `deferred_model_switch` only after MissingProviderLogin modal opens successfully. On busy modal, surface `error.user_message()` in scrollback and do not stash (no silent auto-apply later).

### WR-04: `load_session` / `new_session` silently ignore missing-provider apply failures

**Files modified:** `crates/codegen/xai-grok-shell/src/agent/mvp_agent/acp_agent.rs`  
**Commit:** `f095030`  
**Applied fix:** Handle `model_switch::apply` `Err` on new_session and load_session. For `ModelSwitchMissingProviderError`, log at error and send `ModelAutoSwitched` with the typed user message so the TUI surfaces a ModelUnavailable scrollback event instead of silently staying on the construction-time model. Other apply errors log at warn.

### WR-05: MissingProvider restore drops reasoning effort

**Files modified:** `crates/codegen/xai-grok-pager/src/app/dispatch/session/lifecycle.rs`  
**Commit:** `4faf38a`  
**Applied fix:** IncompatibleAgent and MissingProvider restore only call `set_current` when current is the blocked target; preserve current reasoning effort instead of forcing `None`. Transactional path (already on prev) no longer wipes effort via `set_current(prev, None)`.

**Status note:** `fixed: requires human verification` for effort-only races if an optimistic path ever reintroduces `set_current(target, None)` before complete — full `prev_effort` threading on Effect/TaskResult was not required after transactional set_default_model.

### WR-06: Pager usable cache can stay stale-true across unreadable auth

**Files modified:** `crates/codegen/xai-grok-pager/src/app/effects/mod.rs`, `crates/codegen/xai-grok-pager/src/app/actions.rs`, `crates/codegen/xai-grok-pager/src/app/dispatch/task_result.rs`  
**Commit:** `c5ff1c7` (+ dispatch comment in `b6e1a79`)  
**Applied fix:** Hard `AuthStatusReport::from_auth_file` errors emit `Some(false)/Some(false)` (fail-closed both slots). Soft join errors still emit `None/None` (keep last cache). Docs on TaskResult updated. Test asserts hard fail-closed path.

### IN-01: Poll exhaustion clears poll without generation bump

**Files modified:** `crates/codegen/xai-grok-pager/src/app/dispatch/task_result.rs`  
**Commit:** `b6e1a79`  
**Applied fix:** Exhaust path calls `app.clear_provider_login_poll()` so generation bumps match arm/clear contract; deferred left for FocusGained.

## Skipped Issues

None — all in-scope findings fixed.

## Info not in scope

- **IN-02:** CLI logout badge freshness still depends on FocusGained (documented; not changed).
- **IN-03:** Shell gate telemetry event name left as-is for v1.

## Verification

- `cargo check -p xai-grok-pager --lib` — pass
- `cargo check -p xai-grok-shell --lib` — pass
- `cargo test -p xai-grok-pager --lib p6_` — **44 passed** (includes new CR-01 regression)

---

_Fixed: 2026-07-17T04:30:00Z_  
_Fixer: Claude (gsd-code-fixer)_  
_Iteration: 1_  
