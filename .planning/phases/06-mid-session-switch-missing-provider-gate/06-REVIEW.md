---
phase: 06-mid-session-switch-missing-provider-gate
reviewed: 2026-07-17T00:41:36Z
depth: standard
files_reviewed: 22
files_reviewed_list:
  - crates/codegen/xai-grok-shell/src/agent/handlers/model_switch.rs
  - crates/codegen/xai-grok-shell/src/agent/config.rs
  - crates/codegen/xai-grok-shell/src/agent/mvp_agent/mod.rs
  - crates/codegen/xai-grok-shell/src/agent/mvp_agent/acp_agent.rs
  - crates/codegen/xai-grok-shell/src/auth/status.rs
  - crates/codegen/xai-grok-shell/src/auth/meta.rs
  - crates/codegen/xai-grok-shell/src/auth/mod.rs
  - crates/codegen/xai-grok-shell/tests/model_switch_gate.rs
  - crates/codegen/xai-grok-pager/src/app/actions.rs
  - crates/codegen/xai-grok-pager/src/app/agent.rs
  - crates/codegen/xai-grok-pager/src/app/app_view.rs
  - crates/codegen/xai-grok-pager/src/app/dispatch/session/lifecycle.rs
  - crates/codegen/xai-grok-pager/src/app/dispatch/task_result.rs
  - crates/codegen/xai-grok-pager/src/app/dispatch/auth.rs
  - crates/codegen/xai-grok-pager/src/app/effects/mod.rs
  - crates/codegen/xai-grok-pager/src/app/event_loop.rs
  - crates/codegen/xai-grok-pager/src/app/agent_view/mod.rs
  - crates/codegen/xai-grok-pager/src/views/question_view.rs
  - crates/codegen/xai-grok-pager/src/slash/commands/model.rs
  - crates/codegen/xai-grok-pager/src/settings/registry.rs
  - crates/codegen/xai-grok-pager/src/app/dispatch/settings/setters.rs
  - crates/codegen/xai-grok-pager/src/app/dispatch/settings/ui.rs
findings:
  critical: 1
  warning: 6
  info: 3
  total: 10
status: findings
---

# Phase 6: Code Review Report

**Reviewed:** 2026-07-17T00:41:36Z  
**Depth:** standard  
**Files Reviewed:** 22  
**Status:** findings  

## Summary

Phase 6’s shell gate is structurally sound: `MODEL_SWITCH_MISSING_PROVIDER` is typed and camelCase, provider comes from the catalog, Codex is never inferred from the xAI `AuthManager`, BYOK skips the OAuth-slot check, and the pager maps MissingProvider before IncompatibleAgent/Other. Dual-slot badge cache is booleans-only.

The adversarial pass still found **real recovery and multi-agent correctness bugs** in the pager deferred path, plus silent load/new-session apply failures. Shell enforcement is fail-closed; the main risk is **lost or misrouted deferred recovery** and **silent wrong-model resume**.

## Narrative Findings (AI reviewer)

## Critical Issues

### CR-01: Deferred recovery cleared before SwitchModel succeeds

**File:** `crates/codegen/xai-grok-pager/src/app/dispatch/session/lifecycle.rs:1407-1424`  
**Severity:** critical  
**Issue:** `try_apply_deferred_model_switch_if_ready` sets `deferred_model_switch = None` and clears the login poll **before** `Effect::SwitchModel` completes. If the subsequent ACP switch fails as `Other` (transient network, internal error, actor closed) — not MissingProvider — `handle_switch_model_complete` only surfaces a scrollback error. The Login-now recovery intent (`required_provider`, `persist_default`, target model) is gone; poll will not re-arm.

This is the primary post-login auto-apply path (AuthComplete + ProviderAuthStatusRefreshed). A single flaky switch after successful Codex CLI login drops the deferred target permanently.

**Fix:** Keep the deferred stash until `SwitchModelComplete(Ok)`, or re-stash on non-MissingProvider failure:

```rust
// On try_apply: do not clear deferred yet; set a "pending_apply" flag
// OR on SwitchModelComplete Err(Other | IncompatibleAgent):
agent.session.deferred_model_switch = Some(deferred_snapshot);
// re-arm poll if Login-now recovery was active
```

Prefer: clear deferred only in the `Ok(())` arm of `handle_switch_model_complete` when the completed model matches the deferred target (carry deferred identity on the Effect/TaskResult).

## Warnings

### WR-01: MissingProvider QuestionView opens on `active_view`, not `agent_id`

**File:** `crates/codegen/xai-grok-pager/src/app/dispatch/session/lifecycle.rs:1220-1259` + `265-327`  
**Severity:** warning  
**Issue:** `handle_switch_model_complete` correctly stashes deferred on the **completing** `agent_id`, but `open_missing_provider_login_question` always uses `app.active_view`. Multi-agent / dashboard: SwitchModelComplete for agent A while viewing agent B opens the modal on B (or no-ops if not Agent), while deferred remains on A. Answer handler also mutates the **active** agent (`dispatch_missing_provider_login_answered` lines 1313–1350), so Login now can attach recovery to the wrong session.

`try_apply` intentionally walks all agents, so this is reachable without the user switching the focused session after Login now.

**Fix:** Pass `agent_id` into `open_missing_provider_login_question` / answer dispatch; open and answer against that agent (optionally `switch_to_agent` first). Mirror the fix for any copy-pasted IncompatibleAgent path if multi-session is in scope.

### WR-02: Auto-apply does not dismiss an open MissingProviderLogin modal

**File:** `crates/codegen/xai-grok-pager/src/app/dispatch/session/lifecycle.rs:1383-1426`  
**Severity:** warning  
**Issue:** Deferred is stashed at gate-open (before Login now). Untagged refreshes (`generation: None` from FocusGained / AuthComplete) always pass `provider_login_poll_generation_current` and call `try_apply`. If credentials appear while the Login now / Keep current modal is still open, SwitchModel runs under the modal. Success leaves an orphan QuestionView whose Keep current no longer rolls back the applied model; Login now re-starts recovery for an already-switched target.

**Fix:** In `try_apply` (or Ok complete), if the agent has `LocalQuestionKind::MissingProviderLogin`, take/dismiss the question and restore the stashed prompt. Optionally only auto-apply after Login now (`required_provider` + poll armed), and treat gate-open stash as “intent only” until the user chooses Login now.

### WR-03: Busy QuestionView still stashes deferred without missing-provider UI

**File:** `crates/codegen/xai-grok-pager/src/app/dispatch/session/lifecycle.rs:1244-1249` + `283-286`  
**Severity:** warning  
**Issue:** Deferred is written **before** `open_missing_provider_login_question`. If another question is already open, the user only sees “Finish answering the current question first” — no missing-provider copy — while `deferred_model_switch` remains set. Later FocusGained/auth refresh can auto-apply without the user ever seeing Login/Keep options.

**Fix:** Only stash deferred after the modal opens successfully; on busy, push a system/toast with `error.user_message()` and do not stash (user re-invokes switch).

### WR-04: `load_session` / `new_session` silently ignore missing-provider apply failures

**File:** `crates/codegen/xai-grok-shell/src/agent/mvp_agent/acp_agent.rs:1112-1117`, `1850-1855`  
**Severity:** warning  
**Issue:** Internal callers use `let _ = model_switch::apply(...).await`. Phase 6 now fails closed for missing provider on these paths too. Restoring a Codex-bound session without usable Codex credentials leaves the session on the construction-time default model with **no ACP error, toast, or MODEL_SWITCH_MISSING_PROVIDER surface**. User continues under the wrong provider/model.

**Fix:** On `Err` with `ModelSwitchMissingProviderError`, attach session meta / pager notification (or fall back to a clearly labeled degraded state). At minimum log at error and expose a session flag the TUI can toast on SessionCreated.

### WR-05: MissingProvider restore drops reasoning effort

**File:** `crates/codegen/xai-grok-pager/src/app/dispatch/session/lifecycle.rs:1226-1230`  
**Severity:** warning  
**Issue:** Rollback uses `set_current(prev.clone(), None)`, clearing effort even when the failed switch did not optimistically change current (defensive path). Same pattern on IncompatibleAgent. If current ever equals the target (re-apply / effort-only race), prior effort is wiped.

**Fix:** Preserve previous effort: thread `prev_effort` on `Effect::SwitchModel` / `SwitchModelError` and restore `set_current(prev, prev_effort)`.

### WR-06: Pager usable cache can stay stale-true across unreadable auth

**File:** `crates/codegen/xai-grok-pager/src/app/effects/mod.rs:1601-1631` + `task_result.rs:477-484`  
**Severity:** warning  
**Issue:** Refresh IO/parse failure emits `(None, None)` and dispatch **keeps** the last cache. Shell gate fail-closes (`provider_oauth_slot_usable` → false). After a previously usable slot, a corrupt/unreadable `auth.json` still shows badges as logged-in and can trigger try_apply → shell MissingProvider churn until a successful refresh or in-process logout clears flags.

**Fix:** On hard read errors, fail-closed both slots to `false` (or a tri-state Unknown that never reports usable for try_apply). Keep stale-only for soft/transient None if desired, but document and separate Unknown vs known-usable.

## Info

### IN-01: Poll exhaustion clears poll without generation bump

**File:** `crates/codegen/xai-grok-pager/src/app/dispatch/task_result.rs:514-517`  
**Issue:** Exhaust path sets `provider_login_poll = None` without `clear_provider_login_poll()`’s generation bump. In-flight ticks are still ignored (`poll` is None), but this diverges from the arm/clear contract and is easy to regress.

**Fix:** Call `app.clear_provider_login_poll()` (or bump generation) on exhaust while leaving deferred intact.

### IN-02: CLI logout badge freshness depends on FocusGained

**File:** `crates/codegen/xai-grok-pager/src/app/dispatch/task_result.rs:1052-1054` (comment)  
**Issue:** TUI bare `/logout` is CLI-pointer-only; dual-slot cache is not refreshed until FocusGained/startup refresh. Documented; badges can lie until focus regain after `bum logout --provider …`.

**Fix:** Optional short poll after showing the logout toast, or document in UX copy.

### IN-03: Shell gate telemetry logs rejected switches as ModelSwitched success=false

**File:** `crates/codegen/xai-grok-shell/src/agent/handlers/model_switch.rs:88-95`  
**Issue:** Fine for analytics; ensure dashboards don’t count these as user-initiated successful switches. No credential fields in the event — good.

**Fix:** None required for v1; optional distinct event name.

## Shell gate notes (no separate defect)

- `missing_provider_gate_error` decision table and ACP round-trip look correct.
- `provider_oauth_slot_usable`: disk first, xAI AuthManager fallback only for `ModelProvider::Xai`, fail-closed on store errors — correct for MOD-06 / D-02.
- `AuthMeta.providers` booleans-only; tests assert no token leakage.
- Effects map MissingProvider before IncompatibleAgent; malformed provider → Other.

---

_Reviewed: 2026-07-17T00:41:36Z_  
_Reviewer: Claude (gsd-code-reviewer)_  
_Depth: standard_  
