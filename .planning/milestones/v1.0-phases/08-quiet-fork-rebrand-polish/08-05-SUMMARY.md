---
phase: 08-quiet-fork-rebrand-polish
plan: 05
subsystem: privacy
tags: [ops-02, telemetry, feedback, otlp, sentry, remote-settings, quiet-fork, p8]

requires:
  - phase: 08-quiet-fork-rebrand-polish
    provides: Wave 1 p8_ harness + identity rebrand + auto-update hard-off (Plans 01–04)
  - phase: 01-product-identity-isolated-home
    provides: Binary name bum, BUM_HOME / ~/.bum isolation
provides:
  - /feedback enter+send short-circuit; no Effect::SendFeedback (D-15)
  - force_feedback_request / debug trigger gated when feedback disabled (C1-M1)
  - resolve_feedback default false; remote restrictive-only for feedback + telemetry (C1-H3)
  - InstrumentationMode default Disabled (not Server); exporter.enabled false unless telemetry explicit (C1-H2)
  - Sentry hard-off at pager-bin init (D-10)
  - Green p8_feedback, p8_telemetry, p8_internal_otel_off_by_default, p8_sentry tests
affects:
  - 08-06 residual greps for feedback/telemetry/Sentry/OTLP phone-home
  - Future developer opt-in via GROK_FEEDBACK_ENABLED / GROK_TELEMETRY_ENABLED / GROK_INSTRUMENTATION=server

tech-stack:
  added: []
  patterns:
    - "Restrictive-only remote: remote true ignored when local unset; remote false may force off"
    - "Feedback manager force path returns local synthetic without API write when disabled"
    - "Dual OTLP gate: InstrumentationMode not Server + OtelExporterConfig.enabled false by default"
    - "Composition-root quiet_fork_sentry_disabled() pure policy helper"

key-files:
  created: []
  modified:
    - crates/codegen/xai-grok-pager/src/app/dispatch/notes.rs
    - crates/codegen/xai-grok-pager/src/app/dispatch/tests/notes.rs
    - crates/codegen/xai-grok-shell/src/session/feedback_manager.rs
    - crates/codegen/xai-grok-shell/src/extensions/debug.rs
    - crates/codegen/xai-grok-shell/src/extensions/feedback.rs
    - crates/codegen/xai-grok-shell/src/agent/config.rs
    - crates/codegen/xai-grok-telemetry/src/instrumentation.rs
    - crates/codegen/xai-grok-shell/src/auth/credential_provider.rs
    - crates/codegen/xai-grok-pager-bin/src/main.rs

key-decisions:
  - "Remote telemetry/feedback is restrictive-only — remote true + local-unset stays Disabled/false (C1-H3 mandatory)"
  - "resolve_feedback .default(false); remote feature_flag only applied when remote is Some(false)"
  - "force_feedback_request skips record_feedback_request when feedback_enabled is false; returns local synthetic"
  - "InstrumentationMode defaults to Disabled when GROK_INSTRUMENTATION unset (was Server)"
  - "build_default_otel_layer_config uses !is_telemetry_disabled_sync() so absence is not enable"
  - "Sentry Config.disabled forced true at pager-bin (quiet_fork_sentry_disabled); crate stays linked"
  - "Local logs under ~/.bum remain OK (D-11); Log/Chrome instrumentation still env-opt-in"

patterns-established:
  - "FEEDBACK_DISABLED_MESSAGE locked copy constant in notes.rs for UI-SPEC alignment"
  - "resolve_instrumentation_mode(Option<&str>) pure helper for OTLP default unit tests"
  - "p8_feedback / p8_telemetry / p8_internal_otel / p8_sentry unconditional discovery names for Plan 06"

requirements-completed: [OPS-02]

coverage:
  - id: D1
    description: "/feedback enter shows disabled message, empty effects, stays Normal mode"
    requirement: OPS-02
    verification:
      - kind: unit
        ref: "cargo test -p xai-grok-pager --lib p8_feedback"
        status: pass
    human_judgment: false
  - id: D2
    description: "/feedback send never emits Effect::SendFeedback; no stock thank-you"
    requirement: OPS-02
    verification:
      - kind: unit
        ref: "cargo test -p xai-grok-pager --lib p8_feedback_send"
        status: pass
    human_judgment: false
  - id: D3
    description: "force_feedback_request does not POST when feedback_enabled=false"
    requirement: OPS-02
    verification:
      - kind: unit
        ref: "cargo test -p xai-grok-shell --lib p8_feedback_force_request_noop_when_disabled"
        status: pass
    human_judgment: false
  - id: D4
    description: "resolve_feedback defaults false; remote true local unset stays false"
    requirement: OPS-02
    verification:
      - kind: unit
        ref: "cargo test -p xai-grok-shell --lib p8_feedback"
        status: pass
    human_judgment: false
  - id: D5
    description: "resolve_telemetry_mode remote true local unset stays Disabled"
    requirement: OPS-02
    verification:
      - kind: unit
        ref: "cargo test -p xai-grok-shell --lib p8_telemetry"
        status: pass
    human_judgment: false
  - id: D6
    description: "Internal OTLP off by default (instrumentation + exporter config)"
    requirement: OPS-02
    verification:
      - kind: unit
        ref: "cargo test -p xai-grok-telemetry --lib p8_internal_otel"
        status: pass
      - kind: unit
        ref: "cargo test -p xai-grok-shell --lib p8_internal_otel"
        status: pass
      - kind: unit
        ref: "cargo test -p xai-grok-pager-bin --bin bum p8_internal_otel"
        status: pass
    human_judgment: false
  - id: D7
    description: "Sentry hard-disabled by quiet fork policy at composition root"
    requirement: OPS-02
    verification:
      - kind: unit
        ref: "cargo test -p xai-grok-pager-bin --bin bum p8_sentry"
        status: pass
    human_judgment: false

duration: 11min
completed: 2026-07-17
status: complete
---

# Phase 8 Plan 05: Quiet telemetry/feedback (OPS-02) Summary

**Default path never phones home: /feedback short-circuits, remote cannot re-enable telemetry/feedback, internal OTLP exporter off, Sentry hard-off.**

## Performance

- **Duration:** 11 min
- **Started:** 2026-07-17T11:20:06Z
- **Completed:** 2026-07-17T11:31:33Z
- **Tasks:** 3/3
- **Files modified:** 9

## Accomplishments

- Pager `/feedback` enter/send show `Feedback is disabled in bum (no phone-home).`, stay Normal, never `Effect::SendFeedback` or stock team thank-you (D-15 / UI-SPEC).
- `force_feedback_request` / debug trigger no longer bypass enabled checks for network writes (C1-M1).
- `resolve_feedback` defaults false; remote settings for feedback and telemetry are **restrictive-only** (C1-H3).
- `InstrumentationMode` default is **Disabled** (not Server); `build_default_otel_layer_config` sets `exporter.enabled=false` unless telemetry is explicitly on (C1-H2).
- Pager-bin forces Sentry `disabled: true` (D-10); Mixpanel remains inactive under `TelemetryMode::Disabled` (D-08/D-09).
- Local logs under `~/.bum` unchanged (D-11).

## Task Commits

Each task was committed atomically:

1. **Task 1: Pager /feedback short-circuit + force_feedback gate** - `0205b1c` (feat)
2. **Task 2: resolve_feedback default false + remote restrictive-only** - `3cfbcaf` (feat)
3. **Task 3: Internal OTLP off by default + Sentry hard-off** - `d7ed4ed` (feat)

**Plan metadata:** `9bd081a` (docs: complete quiet telemetry/feedback plan)

## Files Created/Modified

- `crates/codegen/xai-grok-pager/src/app/dispatch/notes.rs` — enter/send short-circuit + locked copy constants
- `crates/codegen/xai-grok-pager/src/app/dispatch/tests/notes.rs` — p8_feedback_* unit tests
- `crates/codegen/xai-grok-shell/src/session/feedback_manager.rs` — force path gated; axum counter tests
- `crates/codegen/xai-grok-shell/src/extensions/debug.rs` — docs: no longer claims enabled-check bypass for feedback API
- `crates/codegen/xai-grok-shell/src/extensions/feedback.rs` — quiet disabled message (no env CTA)
- `crates/codegen/xai-grok-shell/src/agent/config.rs` — resolve_feedback false + remote restrictive for feedback/telemetry
- `crates/codegen/xai-grok-telemetry/src/instrumentation.rs` — default Disabled; pure resolve helper + p8_internal_otel
- `crates/codegen/xai-grok-shell/src/auth/credential_provider.rs` — exporter.enabled uses is_telemetry_disabled_sync
- `crates/codegen/xai-grok-pager-bin/src/main.rs` — quiet_fork_sentry_disabled hard-off + p8_sentry / p8_internal_otel tests

## Decisions Made

1. **Remote restrictive-only (mandatory C1-H3):** For both `feedback_enabled` and `telemetry_mode`/`telemetry_enabled`, remote may only further disable or stay off. Remote-true + local-unset resolves false/`Disabled` with `ConfigSource::Default` (ignored remote). Remote false still surfaces as `ConfigSource::Remote`. Explicit env (`GROK_FEEDBACK_ENABLED`, `GROK_TELEMETRY_ENABLED`), features/config, and requirements pins still win for developers — not advertised in TUI.
2. **OTLP dual gate (C1-H2):** (a) `resolve_instrumentation_mode(None) → Disabled` so `build_tracer_provider` never takes the Server export path by default; (b) `OtelExporterConfig.enabled` false unless `!is_telemetry_disabled_sync()` (explicit enable). Both TUI and `init_tracing_simple` inherit via `build_default_otel_layer_config` + `build_otel_layer`.
3. **Sentry hard-off:** Unconditional `disabled: true` at composition root rather than only `is_error_reporting_disabled_sync()` — privacy hard guarantee even if Sentry-specific env were set. No opt-in UX.
4. **Feedback force path:** Keep returning a local synthetic `FeedbackRequest` when disabled (UI exercise) but skip API record; enabled config still records for developer e2e.

## Deviations from Plan

None material — plan executed as written.

### Auto-fixed Issues

None.

## Auth Gates

None.

## Known Stubs

None. Feedback command remains registered (discoverable) but fully short-circuited; disabled message is intentional product behavior, not a data stub.

## Threat Flags

None new beyond plan threat model mitigations (T-08-08…T-08-16).

## Residual Risks

- **Dev opt-in env knobs remain** (`GROK_FEEDBACK_ENABLED`, `GROK_TELEMETRY_ENABLED`, `GROK_INSTRUMENTATION=server`) — intentional for developers (D-13 / accepted residual T-08-11); not advertised in UX.
- **`is_telemetry_explicitly_disabled_sync` still defaults “not disabled”** for any other callers; only `build_default_otel_layer_config` was switched to `is_telemetry_disabled_sync`. Other call sites of the explicit-disabled helper (if any outside this path) should be audited in Plan 06 residual greps.
- **Sentry crate still linked**; hard-off is runtime `disabled: true` only — DSN presence cannot phone home when disabled.
- **Slash `/feedback` still listed** in help/registry by design (discoverable + disabled).
- Plan 06 should residual-grep for stock thank-you strings, `InstrumentationMode::Server` default assumptions, and remote re-enable of telemetry/feedback.

## Self-Check: PASSED

- `notes.rs` FEEDBACK_DISABLED_MESSAGE present
- Commits `0205b1c`, `3cfbcaf`, `d7ed4ed` on main
- p8_feedback (pager+shell), p8_telemetry, p8_internal_otel (telemetry+shell+bin), p8_sentry (bin) green
