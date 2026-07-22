---
phase: 08-quiet-fork-rebrand-polish
reviewed: 2026-07-17T12:00:00Z
depth: standard
files_reviewed: 39
files_reviewed_list:
  - crates/codegen/xai-grok-pager-bin/src/main.rs
  - crates/codegen/xai-grok-pager-minimal/Cargo.toml
  - crates/codegen/xai-grok-pager-minimal/src/welcome.rs
  - crates/codegen/xai-grok-pager/Cargo.toml
  - crates/codegen/xai-grok-pager/src/app/agent.rs
  - crates/codegen/xai-grok-pager/src/app/agent_view/mod.rs
  - crates/codegen/xai-grok-pager/src/app/app_view.rs
  - crates/codegen/xai-grok-pager/src/app/cli.rs
  - crates/codegen/xai-grok-pager/src/app/dispatch/billing.rs
  - crates/codegen/xai-grok-pager/src/app/dispatch/notes.rs
  - crates/codegen/xai-grok-pager/src/app/dispatch/settings/setters.rs
  - crates/codegen/xai-grok-pager/src/app/dispatch/tests/billing.rs
  - crates/codegen/xai-grok-pager/src/app/dispatch/tests/notes.rs
  - crates/codegen/xai-grok-pager/src/app/mod.rs
  - crates/codegen/xai-grok-pager/src/headless.rs
  - crates/codegen/xai-grok-pager/src/minimal/api.rs
  - crates/codegen/xai-grok-pager/src/plugin_cmd.rs
  - crates/codegen/xai-grok-pager/src/project_picker/mod.rs
  - crates/codegen/xai-grok-pager/src/settings/defs.rs
  - crates/codegen/xai-grok-pager/src/settings/registry.rs
  - crates/codegen/xai-grok-pager/src/views/welcome/hero_box.rs
  - crates/codegen/xai-grok-pager/src/views/welcome/mod.rs
  - crates/codegen/xai-grok-pager/tests/pty_e2e/minimal/minimal_new_session_keeps_history_and_resets.rs
  - crates/codegen/xai-grok-pager/tests/pty_e2e/minimal/minimal_slash_switches_from_fullscreen.rs
  - crates/codegen/xai-grok-pager/tests/settings_e2e.rs
  - crates/codegen/xai-grok-shell/src/agent/app.rs
  - crates/codegen/xai-grok-shell/src/agent/config.rs
  - crates/codegen/xai-grok-shell/src/auth/credential_provider.rs
  - crates/codegen/xai-grok-shell/src/auth/device_code.rs
  - crates/codegen/xai-grok-shell/src/auth/error.rs
  - crates/codegen/xai-grok-shell/src/auth/oidc/login.rs
  - crates/codegen/xai-grok-shell/src/extensions/debug.rs
  - crates/codegen/xai-grok-shell/src/extensions/feedback.rs
  - crates/codegen/xai-grok-shell/src/mcp_doctor.rs
  - crates/codegen/xai-grok-shell/src/plugin.rs
  - crates/codegen/xai-grok-shell/src/session/feedback_manager.rs
  - crates/codegen/xai-grok-telemetry/src/instrumentation.rs
  - crates/codegen/xai-grok-update/src/auto_update.rs
  - crates/codegen/xai-grok-update/src/minimum_version.rs
findings:
  critical: 0
  warning: 6
  info: 5
  total: 11
status: issues_found
---

# Phase 8: Code Review Report

**Reviewed:** 2026-07-17T12:00:00Z
**Depth:** standard
**Files Reviewed:** 39
**Status:** issues_found

## Summary

Adversarial review of Phase 8 quiet-fork work (ID-02 rebrand, OPS-01 auto-update hard-off, OPS-02 telemetry/feedback/OTLP/Sentry). Scope: source diffs from `60cde69^` (phase product start) through HEAD, excluding planning artifacts.

**High-level assessment:** Default quiet-fork paths are largely sound — `should_check_for_updates` always false, `run_update_command` / `finish_update_on_exit` no-op, min-version entry hard no-op, telemetry/feedback defaults Disabled/false with remote restrictive-only, OTLP dual-gated (instrumentation mode + exporter), Sentry forced off at pager-bin, TUI `/feedback` never emits `Effect::SendFeedback`. Model catalog labels (`Grok Build (xAI)` / `grok-build`) were not touched.

**Key concerns:** incomplete auto-update policy application below the composition-root gate (leader check_fn + `ensure_latest_on_disk`), settings UX that still lets users “enable” a hard-disabled channel, test seams that do not wrap production call sites, and a few test-quality issues. No ship-blocking phone-home on the current hard-off path was proven, but defense-in-depth and misleading user/settings surfaces need fixes.

## Narrative Findings (AI reviewer)

## Warnings

### WR-01: Leader auto-update check still treats `None` as enabled (pre-D-07 semantics)

**File:** `crates/codegen/xai-grok-pager-bin/src/main.rs:1217-1221`
**Issue:** The leader `check_fn` still uses stock “skip only if explicitly false” logic:

```rust
if current_config.cli.auto_update == Some(false) {
    return false;
}
match auto_update::ensure_latest_on_disk(&uc).await {
```

Phase 8 introduced `effective_auto_update_enabled` (`None` → `false`) and applied it in `check_update_background` / `run_update_if_available`, but **not** here. Today this is dead because `should_check_for_updates` is always `false` and the whole `LeaderAutoUpdateConfig` is `None`. If that composition-root gate is ever relaxed (debug flag, partial rollback, alternate binary entry), leader hourly checks would treat unset `auto_update` as **on** and call `ensure_latest_on_disk` (network + possible stock install).
**Fix:** Align with the shared policy and refuse network when off:

```rust
use xai_grok_update::auto_update::effective_auto_update_enabled;
// ...
if !effective_auto_update_enabled(current_config.cli.auto_update) {
    return false;
}
```

Prefer also routing through a hard composition-root no-op check_fn rather than re-arming stock install when the channel is disabled.

### WR-02: `ensure_latest_on_disk` has no auto-update / quiet-fork gate

**File:** `crates/codegen/xai-grok-update/src/auto_update.rs:257-279`
**Issue:** `ensure_latest_on_disk` always `fetch_latest_version` and may `run_install_script` with **no** `effective_auto_update_enabled` check. Unlike `run_update_if_available` / `check_update_background`, this chokepoint relies entirely on callers. Phase 8’s only production caller is the leader path (WR-01), currently gated at composition root — but the library API remains a silent stock phone-home/install entry for any future or external caller.
**Fix:** Gate at the library boundary:

```rust
pub async fn ensure_latest_on_disk(update_config: &UpdateConfig) -> Result<EnsureLatestOutcome> {
    let current_config = config::load_config().await;
    if !effective_auto_update_enabled(current_config.cli.auto_update) {
        return Ok(EnsureLatestOutcome {
            installed: None,
            relaunch_needed: false,
        });
    }
    // ... existing logic
}
```

Optionally add a composition-root hard-off param if config can still be `Some(true)` while bum must never touch the stock channel.

### WR-03: Auto-update setting remains user-toggleable and claims “restart to apply”

**File:** `crates/codegen/xai-grok-pager/src/app/dispatch/settings/setters.rs:1851-1873`
**Also:** `crates/codegen/xai-grok-pager/src/settings/defs.rs:1221-1232`
**Issue:** Registry default/description correctly say stock channel is disabled, but `set_auto_update` still persists `Some(true)`, toasts success with “restart to apply”, and writes `[cli].auto_update = true`. Composition-root hard-off means the toggle **never** re-enables probes (`should_check_for_updates` always false; `bum update` always no-ops). Users can believe auto-update is on while remaining permanently offline from the stock channel — false sense of safety/currency, and config drift that interacts with WR-01/WR-02 if gates loosen.
**Fix:** Either (a) hide/lock the setting (read-only false), or (b) keep the setting but refuse `true` with an explicit toast:

```rust
if new {
    app.show_toast("Stock auto-update is permanently disabled in bum.");
    return vec![];
}
```

Do not emit `Effect::PersistSetting` for a no-op hard-off path.

### WR-04: Stock-update test seams never wrap production call sites

**File:** `crates/codegen/xai-grok-pager-bin/src/main.rs:2031-2066` (seams), `909-915`, `987-994`, `1221` (production still calls `auto_update::*` directly)
**Issue:** `stock_run_update_if_available` / `stock_check_update_status` / `stock_run_update` + `STOCK_UPDATE_HELPER_CALLS` exist for hermetic proofs, but production still calls `auto_update::run_update_if_available` / `check_update_background` / `ensure_latest_on_disk` directly. Tests for `run_update_command` / `finish_update_on_exit` correctly prove those paths never touch seams; `p8_update_no_network_startup_and_leader_gated` only asserts the pure `should_check` boolean. A future change that re-enables `should_check` without using seams would not fail the counter-based network proofs.
**Fix:** Route every remaining production stock-channel call through the seams (or delete seams and assert at the update-crate policy helpers + gate pure functions only). Prefer:

```rust
if should_check_for_updates(no_auto_update) {
    stock_run_update_if_available(...).await.ok();
}
```

### WR-05: TUI feedback hard-off is unconditional; opt-in config is dead for slash path

**File:** `crates/codegen/xai-grok-pager/src/app/dispatch/notes.rs:31-77`
**Issue:** `dispatch_enter_feedback_mode` / `dispatch_send_feedback` always short-circuit with no config check. Meanwhile `resolve_feedback` still honors `GROK_FEEDBACK_ENABLED` / `[features] feedback` / requirements, and ACP `x.ai/feedback` + `FeedbackManager` respect `feedback_enabled`. Result: env/config “enable feedback” does **not** re-enable TUI `/feedback`, while ACP/debug can still network when enabled. Intentional hard-off is fine, but the dual model is easy to misread and leaves `Effect::SendFeedback` fully wired in effects (`effects/mod.rs:3183+`) for any future accidental emitter.
**Fix:** Pick one model and lock it in tests:
1. **Hard-off (current product intent):** also refuse ACP/feedback-manager network unless a single explicit quiet-fork opt-in; document that TUI is permanently local-only.
2. **Config-gated:** consult `is_feedback_enabled()` in dispatch so opt-in re-enables TUI consistently.

Also consider removing or `debug_assert`ing dead `Effect::SendFeedback` production handling if TUI is permanently hard-off.

### WR-06: `p8_feedback_force_request_*` tests use sleep-based network assertions (flake risk)

**File:** `crates/codegen/xai-grok-shell/src/session/feedback_manager.rs:1490-1548`
**Issue:** Disabled path waits fixed `50ms` then asserts zero hits; enabled path polls up to ~500ms. Racey timing can false-pass (request still in flight) or false-fail under load. This weakens the C1-M1 force-feedback gate proof.
**Fix:** Prefer deterministic completion:

```rust
// e.g. join record task, or use a oneshot/notify in a test double FeedbackClient
// instead of a real HTTP server + sleep.
```

Or use a mock client trait that increments a counter synchronously when `record` is invoked.

## Info

### IN-01: Unreachable `FinishUpdateOnExit::{Installed,Failed}` arms

**File:** `crates/codegen/xai-grok-pager-bin/src/main.rs:1937-1946`
**Issue:** `finish_update_on_exit` always returns `ChannelDisabled`, so Installed/Failed message arms are dead. Fine as future-proofing, but confuses readers about whether install can still complete.
**Fix:** Collapse to a single path or mark arms with `#[allow(unreachable_patterns)]` + comment that only ChannelDisabled is live for quiet fork.

### IN-02: Misleading legacy test name for feedback default

**File:** `crates/codegen/xai-grok-shell/src/agent/config.rs:8848-8853`
**Issue:** `resolve_feedback_defaults_to_true_when_unset` now asserts `!value`. Name contradicts behavior and confuses grep/blame.
**Fix:** Rename to `resolve_feedback_defaults_to_false_when_unset` or delete in favor of `p8_feedback_resolve_defaults_to_false`.

### IN-03: Stale doc comment on `is_telemetry_disabled_sync`

**File:** `crates/codegen/xai-grok-shell/src/agent/config.rs:2930-2936`
**Issue:** Doc says “`true` only when explicitly off,” but `SyncBoolFlag` default is `false` for the enabled flag, so **unset → disabled = true**. That matches quiet-fork OTEL intent and the Plan 05 switch away from `is_telemetry_explicitly_disabled_sync`, but the comment is wrong.
**Fix:** Document actual semantics: “true when unset or explicitly disabled; false only when config/env/requirements enable telemetry.”

### IN-04: Residual stock product wording in non-user clap docs

**File:** `crates/codegen/xai-grok-pager/src/app/cli.rs:188`
**Issue:** Doc comment still says “used by `grok workspace`” after product rebrand of user-facing copy. Not user-visible, but residual identity debt on a Phase 8-touched surface.
**Fix:** Update to `` `bum workspace` ``.

### IN-05: Screen-mode relaunch tests still seed argv as `grok`

**File:** `crates/codegen/xai-grok-pager/src/app/screen_mode_relaunch.rs:445` (pre-existing; adjacent to phase identity work)
**Issue:** Unit fixtures still use `["grok", ...]` argv. Not a runtime product-chrome bug (argv0 comes from real process), but weakens confidence that relaunch preserves `bum` binary name if logic ever keys off args[0].
**Fix:** Prefer `["bum", ...]` in new/edited fixtures when touching this area.

## Focus-area checklist

| Focus | Result |
|-------|--------|
| Accidental break of model brands / routing | **No regression found** in phase diffs; model catalog / `grok-build` ids untouched; billing kept SuperGrok + `referrer=grok-build` |
| Residual phone-home paths | Defaults off for telemetry, feedback, OTLP (dual gate), Sentry; **latent** stock update install still reachable if composition-root gate weakens (WR-01/02) |
| Update no-op correctness | `should_check`, `run_update_command`, `finish_update_on_exit`, min-version entry are hard no-ops; settings/toggle inconsistency (WR-03) |
| Remote settings restrictive-only | Telemetry + feedback remote-true ignored when local unset; remote-false can force off — correct for C1-H3 |
| Test quality | Solid pure-policy unit tests; network counter tests don’t wrap production sites (WR-04); sleep-based force-feedback tests (WR-06); misleading legacy test name (IN-02) |

---

_Reviewed: 2026-07-17T12:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
