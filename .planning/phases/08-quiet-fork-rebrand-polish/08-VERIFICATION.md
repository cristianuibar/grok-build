---
phase: 08-quiet-fork-rebrand-polish
verified: 2026-07-17T12:03:05Z
status: passed
score: 3/3 must-haves verified
behavior_unverified: 0
overrides_applied: 0
re_verification: false
---

# Phase 8: Quiet fork & rebrand polish — Verification Report

**Phase Goal:** Product presents fully as bum and does not phone home or auto-update as stock Grok Build  
**Verified:** 2026-07-17T12:03:05Z  
**Status:** passed  
**Re-verification:** No — initial verification  
**Mode:** mvp (ROADMAP); verification is goal-backward against ROADMAP success criteria + ID-02 / OPS-01 / OPS-02

## User Flow Coverage (MVP)

| # | User-story step | Expected | Codebase evidence | Status |
|---|-----------------|----------|-------------------|--------|
| 1 | Product presents as **bum** (chrome / help / recovery) | Clap, hero, OAuth, residuals say bum; model brand `Grok Build (xAI)` retained | Clap `name=bum` / `about=bum TUI`; hero `Thanks for using bum.`; OAuth `return to bum`; residual greps clean; `p8_*` ID-02 green | ✓ |
| 2 | Stock auto-update never overwrites bum | No stock channel probe; `bum update` no-op; min-version no-op | `should_check_for_updates` → `false`; `run_update_command` prints disabled copy; `enforce_minimum_version_or_exit` empty; `p8_no_auto_update` / `p8_update_*` / `p8_min_version` green | ✓ |
| 3 | No xAI analytics / phone-home by default | Telemetry Disabled; feedback off; OTLP off; Sentry off | `resolve_telemetry_mode` → Disabled; feedback default false + dispatch short-circuit; instrumentation default Disabled; `quiet_fork_sentry_disabled()==true`; `p8_telemetry` / `p8_feedback` / `p8_internal_otel` / `p8_sentry` green | ✓ |

**Outcome clause verified:** fork identity and privacy hold under fixture-only automated proofs (no live x.ai network required for this phase).

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | Product UI chrome, help text, and user-facing strings present as **bum**, not stock Grok Build (ID-02 / ROADMAP SC1) | ✓ VERIFIED | Clap name/about; welcome badge + hero subtitle; project picker; headless/plugin/auth/mcp recovery copy; OAuth return; minimal welcome; bin banner/crash/serve. Residual C1-H1 greps clean on 7 owned surfaces. Model label `Grok Build (xAI)` still green (D-02). Behavioral: `p8_cli_brand` (3), `p8_welcome` (4), `p8_runtime_cli` (2), `p8_oauth_return` (1), `p8_shell_runtime_cli` (4), `p8_minimal_welcome` (1), `p8_bin_` (3), `dynamic_enum_model_names` (2) — all pass. |
| 2 | Stock xAI auto-update channel is disabled so bum is not overwritten by official Grok Build updates (OPS-01 / ROADMAP SC2) | ✓ VERIFIED | Composition-root hard-off: `should_check_for_updates` always `false`. Explicit `run_update_command` / `finish_update_on_exit` no-op with locked “Stock auto-update is disabled in bum…” copy. `effective_auto_update_enabled(None)==false`; first-run does not persist `true`. `enforce_minimum_version_or_exit` hard no-op. Settings registry default false. Behavioral: `p8_no_auto_update` (1), `p8_update_cmd` (1), `p8_update_no_network` (3), `p8_auto_update` (3), `p8_min_version` (1), `p8_settings_auto_update` (1) — all pass. |
| 3 | Product telemetry / phone-home to xAI analytics is disabled by default (OPS-02 / ROADMAP SC3) — incl. OTLP, feedback, Sentry | ✓ VERIFIED | `resolve_telemetry_mode` defaults `TelemetryMode::Disabled`; remote restrictive-only (remote true cannot enable). Instrumentation mode unset → `Disabled` (`default_mode_exports_otlp()==false`). OTEL exporter `enabled: false` via `build_default_otel_layer_config` + `is_telemetry_disabled_sync`. Sentry init `disabled: quiet_fork_sentry_disabled()` always `true`. Feedback `resolve_feedback` default false; `/feedback` dispatch returns empty effects + “Feedback is disabled in bum (no phone-home).”. Behavioral: `p8_telemetry` (3), `p8_feedback` shell (5) + pager (4), `p8_internal_otel` telemetry/shell/bin (1 each), `p8_sentry` (1) — all pass. |

**Score:** 3/3 truths verified (0 present, behavior-unverified)

### Required Artifacts

| Artifact | Expected | Status | Details |
| -------- | -------- | ------ | ------- |
| `crates/codegen/xai-grok-pager/src/app/cli.rs` | Clap name=bum about=bum TUI | ✓ VERIFIED | `name = "bum"`, `about = "bum TUI"`; p8_cli_brand green |
| `crates/codegen/xai-grok-pager/src/views/welcome/hero_box.rs` | HERO_SUBTITLE Thanks for using bum. | ✓ VERIFIED | constant + p8_welcome_hero tests |
| `crates/codegen/xai-grok-pager/src/views/welcome/mod.rs` | PRODUCT_BADGE_LABEL bum | ✓ VERIFIED | badge/zdr/trust product strings |
| `crates/codegen/xai-grok-pager/src/project_picker/mod.rs` | Run bum in a project directory | ✓ VERIFIED | question string + assert |
| `crates/codegen/xai-grok-pager/src/headless.rs` | bum login recovery | ✓ VERIFIED | residual clean + p8_runtime_cli |
| `crates/codegen/xai-grok-pager/src/plugin_cmd.rs` | bum plugin instructions | ✓ VERIFIED | residual clean + p8_runtime_cli |
| `crates/codegen/xai-grok-pager/src/app/dispatch/notes.rs` | Feedback disabled short-circuit | ✓ VERIFIED | no `SendFeedback`; locked copy |
| `crates/codegen/xai-grok-pager/src/settings/{defs,registry}.rs` | auto_update default false | ✓ VERIFIED | p8_settings_auto_update |
| `crates/codegen/xai-grok-shell/src/auth/oidc/login.rs` | return to bum | ✓ VERIFIED | callback HTML + p8_oauth_return |
| `crates/codegen/xai-grok-shell/src/agent/app.rs` | Open bum | ✓ VERIFIED | browser prompt rebrand |
| `crates/codegen/xai-grok-shell/src/auth/error.rs` (+ device_code, mcp_doctor, plugin) | bum login/mcp/plugin | ✓ VERIFIED | residual greps + p8_shell_runtime_cli |
| `crates/codegen/xai-grok-shell/src/agent/config.rs` | telemetry/feedback quiet defaults | ✓ VERIFIED | Disabled + restrictive remote |
| `crates/codegen/xai-grok-shell/src/auth/credential_provider.rs` | OTEL exporter off by default | ✓ VERIFIED | p8_internal_otel |
| `crates/codegen/xai-grok-pager-minimal/src/welcome.rs` | product span bum | ✓ VERIFIED | MINIMAL_WELCOME_PRODUCT_NAME |
| `crates/codegen/xai-grok-pager-bin/src/main.rs` | banner + update hard-off + Sentry off | ✓ VERIFIED | should_check false; sentry disabled; p8_bin_/update/sentry |
| `crates/codegen/xai-grok-update/src/auto_update.rs` | unwrap_or(false) + no first-run true | ✓ VERIFIED | effective + persist helpers |
| `crates/codegen/xai-grok-update/src/minimum_version.rs` | enforce_minimum_version_or_exit no-op | ✓ VERIFIED | empty public entry + p8_min_version |
| `crates/codegen/xai-grok-telemetry/src/instrumentation.rs` | InstrumentationMode default Disabled | ✓ VERIFIED | resolve None → Disabled |
| `08-PHASE-GATE.md` / `08-VALIDATION.md` | Green gate map | ✓ VERIFIED | Plan 06 artifacts present; gate claims GREEN; this run re-executed subgroups |

### Key Link Verification

| From | To | Via | Status | Details |
| ---- | -- | --- | ------ | ------- |
| `should_check_for_updates` | startup / leader auto-update spawn | always `false` gate | ✓ WIRED | Calls to `run_update_if_available` only inside `if should_check…`; hard-off proven by p8_update_no_network |
| `run_update_command` / `finish_update_on_exit` | stock x.ai channel helpers | early return + disabled message | ✓ WIRED | No `check_update_status` / stock helper calls; counter stays 0 |
| Settings auto_update default false | `effective_auto_update_enabled` | unwrap_or(false) | ✓ WIRED | registry + update crate aligned |
| `dispatch_enter_feedback_mode` / `dispatch_send_feedback` | Effect pipeline | empty vec; no `SendFeedback` | ✓ WIRED | p8_feedback_* green |
| `resolve_telemetry_mode` / `resolve_feedback` | remote settings | restrictive-only | ✓ WIRED | remote true ignored when local unset |
| `quiet_fork_sentry_disabled` | `sentry::init` Config.disabled | always true | ✓ WIRED | `disabled: quiet_fork_sentry_disabled()` |
| instrumentation + `build_default_otel_layer_config` | cli-chat-proxy `/v1/traces` | exporter disabled / mode not Server | ✓ WIRED | p8_internal_otel on three packages |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| -------- | ------------- | ------ | ------------------ | ------ |
| Welcome hero | `HERO_SUBTITLE` / `PRODUCT_BADGE_LABEL` | compile-time constants | Real locked copy (not empty placeholder) | ✓ FLOWING |
| Clap help | command name/about | clap derive attrs | Real “bum” / “bum TUI” | ✓ FLOWING |
| Feedback path | effects from dispatch | short-circuit → empty vec + system message | Intentionally no network payload | ✓ FLOWING (quiet path) |
| Update command | stdout/stderr lines | `stock_auto_update_disabled_lines()` | Real disabled message; no install URL fetch | ✓ FLOWING (no-op) |
| Telemetry client mode | `TelemetryMode` | `resolve_telemetry_mode` default Disabled | Disabled — no Mixpanel client on default path | ✓ FLOWING (off) |
| Sentry | `Config.disabled` | hard `true` | No crash phone-home | ✓ FLOWING (off) |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| -------- | ------- | ------ | ------ |
| Clap brand | `cargo test -p xai-grok-pager p8_cli_brand --lib` | 3 ok | ✓ PASS |
| Welcome chrome | `cargo test -p xai-grok-pager p8_welcome --lib` | 4 ok | ✓ PASS |
| Runtime CLI residuals (pager) | `… p8_runtime_cli --lib` | 2 ok | ✓ PASS |
| Feedback UI short-circuit | `… p8_feedback --lib` (pager) | 4 ok | ✓ PASS |
| Settings auto_update | `… p8_settings_auto_update --lib` | 1 ok | ✓ PASS |
| OAuth return | `cargo test -p xai-grok-shell p8_oauth_return --lib` | 1 ok | ✓ PASS |
| Shell residual CLI | `… p8_shell_runtime_cli --lib` | 4 ok | ✓ PASS |
| Telemetry default + remote | `… p8_telemetry --lib` | 3 ok | ✓ PASS |
| Feedback resolve + force_request | `… p8_feedback --lib` (shell) | 5 ok | ✓ PASS |
| Shell OTEL exporter | `… p8_internal_otel --lib` | 1 ok | ✓ PASS |
| Minimal welcome | `cargo test -p xai-grok-pager-minimal p8_minimal_welcome --lib` | 1 ok | ✓ PASS |
| Bin banner/crash/serve | `cargo test -p xai-grok-pager-bin p8_bin_ --bin bum` | 3 ok | ✓ PASS |
| Update hard-off gate | `… p8_no_auto_update --bin bum` | 1 ok | ✓ PASS |
| Update message | `… p8_update_cmd --bin bum` | 1 ok | ✓ PASS |
| Update no-network | `… p8_update_no_network --bin bum` | 3 ok | ✓ PASS |
| Sentry hard-off | `… p8_sentry --bin bum` | 1 ok | ✓ PASS |
| Bin OTEL default | `… p8_internal_otel --bin bum` | 1 ok | ✓ PASS |
| Auto-update defaults | `cargo test -p xai-grok-update p8_auto_update --lib` | 3 ok | ✓ PASS |
| Min-version no-op | `… p8_min_version --lib` | 1 ok | ✓ PASS |
| Telemetry crate OTLP | `cargo test -p xai-grok-telemetry p8_internal_otel --lib` | 1 ok | ✓ PASS |
| D-02 model brand | `cargo test -p xai-grok-pager --test dynamic_enum_model_names` | 2 ok | ✓ PASS |
| Home isolation baseline | `cargo test -p xai-grok-pager-bin --test home_isolation` | 1 ok | ✓ PASS |
| C1-H1 residual greps | static `rg` stock patterns on 7 surfaces | all clean + bum recovery present | ✓ PASS |

### Probe Execution

| Probe | Command | Result | Status |
| ----- | ------- | ------ | ------ |
| N/A | Phase uses cargo `p8_` filters + residual greps (not `scripts/*/tests/probe-*.sh`) | PHASE-GATE protocol re-run via discover+execute above | SKIP (no conventional probe scripts) |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| ----------- | ---------- | ----------- | ------ | -------- |
| **ID-02** | 08-01…03, 08-06 | Product UI chrome / help / strings as bum | ✓ SATISFIED | Artifacts + residual greps + p8_ ID-02 subgroups |
| **OPS-01** | 08-04, 08-06 | Stock auto-update disabled | ✓ SATISFIED | Hard-off gates + no-op update/min-version + p8_ OPS-01 |
| **OPS-02** | 08-01, 08-05, 08-06 | Telemetry / phone-home off by default | ✓ SATISFIED | Telemetry/feedback/OTLP/Sentry defaults + p8_ OPS-02 |

No orphaned Phase 8 requirements (REQUIREMENTS map only ID-02, OPS-01, OPS-02 to Phase 8).

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| ---- | ---- | ------- | -------- | ------ |
| `xai-grok-shell/.../oidc/login.rs` | 35 | `code=XXX` in doc example | ℹ️ Info | Not a debt marker — URL placeholder in rustdoc |
| `xai-grok-pager-bin/src/main.rs` | ~1335 | Dashboard-disabled error still cites `~/.grok/config.toml` | ℹ️ Info | Path residual outside C1-H1 stock chrome inventory; actual home is `BUM_HOME` (Phase 1). Not stock “Grok Build” product name. Optional cleanup later. |
| `xai-grok-update/.../minimum_version.rs` | error Display strings | Still mention “Grok” / ``grok update`` | ℹ️ Info | Dead private helper only; public `enforce_minimum_version_or_exit` is hard no-op (never surfaces these to users on quiet path). |

No blocker debt markers (`TBD`/`FIXME`/`XXX` unresolved) in phase-touched product paths.

### Human Verification Required

None required for phase gate. Visual TUI chrome is covered by unit asserts on the same constants the renderer uses; live dual-provider daily-driver is **Phase 9** (deferred).

### Deferred Items (not gaps)

| Item | Addressed In | Evidence |
|------|--------------|----------|
| Public signed x.ai install channel for bum | Out of v1 scope | PROJECT / CONTEXT deferred |
| Internal crate rename `xai-grok-*` | Beyond v1 | D-03 leave-internal |
| Mass `GROK_*` → `BUM_*` env rename | Beyond Phase 8 | D-13 leave-internal |
| Live dual-provider E2E | Phase 9 | ROADMAP Phase 9 success criteria |
| Agent system prompt “Grok Build agent” | Beyond chrome gate | PHASE-GATE residual allowlist |

### Gaps Summary

No actionable gaps. All three ROADMAP success criteria and requirements ID-02 / OPS-01 / OPS-02 are implemented in product code and proven green by re-executed fixture-only `p8_` subgroups plus residual inventory greps. PHASE-GATE GREEN claim is independently corroborated by this verification run (not trusted on SUMMARY alone).

---

_Verified: 2026-07-17T12:03:05Z_  
_Verifier: Claude (gsd-verifier)_
