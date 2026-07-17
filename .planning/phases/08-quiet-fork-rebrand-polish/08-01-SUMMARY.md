---
phase: 08-quiet-fork-rebrand-polish
plan: 01
subsystem: testing
tags: [p8, harness, nyquist, telemetry, rebrand, quiet-fork, isolation]

requires:
  - phase: 01-product-identity-isolated-home
    provides: BUM_HOME isolation, home_isolation hermetic pattern, binary name bum
  - phase: 07-cross-provider-multi-agent-orchestration
    provides: green-only pN_ wave-1 scaffold protocol
provides:
  - Wave 1 compile-safe green p8_ filter discovery for shell (telemetry) + pager (smoke)
  - 08-VALIDATION.md requirement → planned filter map for ID-02 / OPS-01 / OPS-02
  - OPS-02 baseline lock: TelemetryMode::Disabled default under p8_telemetry
  - D-02 model brand + home_isolation re-verify anchors
affects:
  - 08-02 product chrome clap/welcome (p8_cli_brand, p8_welcome)
  - 08-03 residual runtime CLI / OAuth / banners
  - 08-04 auto-update hard-off filters
  - 08-05 feedback + remote telemetry hardening + Sentry/OTLP
  - 08-06 phase gate discover+execute

tech-stack:
  added: []
  patterns:
    - "p8_ prefix only green tests (no intentional-red under phase filter)"
    - "Wave 1 locks already-true defaults (telemetry Disabled) without product string flips"
    - "VALIDATION inventory lists Plan 02–05 filter names without landing failing tests"

key-files:
  created: []
  modified:
    - crates/codegen/xai-grok-shell/src/agent/config.rs
    - crates/codegen/xai-grok-pager/src/app/cli.rs
    - .planning/phases/08-quiet-fork-rebrand-polish/08-VALIDATION.md

key-decisions:
  - "No product chrome asserts in Plan 01 — stock clap still name=grok; Plan 02 owns p8_cli_brand"
  - "Telemetry default Disabled is already true (D-08) — wave-1 locks green, no resolve precedence change"
  - "Document explicit features.telemetry enable path; remote restrictive hardening deferred to Plan 05"
  - "home_isolation keeps Phase 1 names in VALIDATION map (no p8_ alias required)"

patterns-established:
  - "Phase 8 green scaffold: p8_ discovery ≥1 in shell + pager before product waves"
  - "Plan 01 greened filters only: p8_telemetry, p8_wave1; product filters inventory-only"

# Scaffold only — ID-02 / OPS-01 / OPS-02 product complete in Plans 02–05 / 06 gate
requirements-completed: []

coverage:
  - id: D1
    description: "Shell p8_telemetry locks resolve_telemetry_mode default Disabled + ConfigSource::Default"
    requirement: OPS-02
    verification:
      - kind: unit
        ref: "cargo test -p xai-grok-shell --lib p8_telemetry"
        status: pass
    human_judgment: false
  - id: D2
    description: "Pager p8_ wave1 harness smoke for filter discovery (no brand string asserts)"
    requirement: ID-02
    verification:
      - kind: unit
        ref: "cargo test -p xai-grok-pager --lib p8_"
        status: pass
    human_judgment: false
  - id: D3
    description: "Model catalog still Grok Build (xAI) for grok-build (D-02) + home_isolation green"
    requirement: ID-02
    verification:
      - kind: unit
        ref: "cargo test -p xai-grok-pager --test dynamic_enum_model_names"
        status: pass
      - kind: integration
        ref: "cargo test -p xai-grok-pager-bin --test home_isolation"
        status: pass
    human_judgment: false

duration: 15min
completed: 2026-07-17
status: complete
---

# Phase 8 Plan 01: Wave 1 p8_ green harness Summary

**Compile-safe Nyquist scaffold: shell `p8_telemetry` (OPS-02 Disabled default) + pager `p8_wave1` discovery smoke; VALIDATION inventory for ID-02/OPS-01/OPS-02 without product rebrand or update hard-off.**

## Performance

- **Duration:** 15 min
- **Started:** 2026-07-17T10:31:00Z
- **Completed:** 2026-07-17T10:46:00Z
- **Tasks:** 2/2
- **Files modified:** 3

## Accomplishments

- Shell lists 2 green `p8_telemetry` tests: default Disabled+Default source; explicit config enable path documented
- Pager lists 1 green `p8_wave1_harness_smoke_compiles` under `--lib p8_` (no clap brand assert)
- `dynamic_enum_model_names` still green for `Grok Build (xAI)` / `grok-build` (D-02)
- `home_isolation` still green (BUM_HOME hermetic baseline)
- `08-VALIDATION.md` maps requirements → planned filters; Plan 01 rows greened; Plans 02–05 inventory only

## Task Commits

1. **Task 1: GREEN scaffold — p8_telemetry + VALIDATION** - `60cde69` (test)
2. **Task 2: GREEN scaffold — pager p8_ wave1 + baselines** - `78eeb98` (test)

**Plan metadata:** `291834f` (docs: complete plan)

## Files Created/Modified

- `crates/codegen/xai-grok-shell/src/agent/config.rs` — `p8_telemetry_resolve_mode_defaults_to_disabled`, `p8_telemetry_disabled_implies_no_easy_enabled_without_explicit_config`
- `crates/codegen/xai-grok-pager/src/app/cli.rs` — `p8_wave1_harness_smoke_compiles`
- `.planning/phases/08-quiet-fork-rebrand-polish/08-VALIDATION.md` — Plan 01 greened map + Plan 02–05 filter inventory

## Decisions Made

- **No product string green asserts in wave 1** — clap still `name=grok` / about Grok Build; asserting bum would be intentional-red under `p8_`
- **Telemetry precedence unchanged** — default Disabled locked; remote re-enable still possible today (Plan 05 hardens)
- **home_isolation names kept** — Phase 1 filter remains the isolation proof; no thin `p8_` wrapper needed

## Telemetry precedence inventory (for Plan 05)

Current `resolve_telemetry_mode` order: **Requirement > Env (`GROK_TELEMETRY_ENABLED`) > Config (`features.telemetry`) > Remote > Default(Disabled)**.

Wave-1 locks only the unset Default path. Remote-true + local-unset still can enable via Remote today — Plan 05 owns restrictive-only remote.

## Plan 02–05 planned green filter names (inventory — not landed)

| Plan | Filters |
|------|---------|
| 02 | `p8_cli_brand`, `p8_welcome` / `p8_hero`, `p8_project_picker`, `p8_runtime_cli` |
| 03 | `p8_oauth_return`, `p8_shell_runtime_cli`, `p8_bin_`, `p8_minimal_welcome` |
| 04 | `p8_no_auto_update` / `p8_should_check`, `p8_auto_update_default`, `p8_min_version`, `p8_update_cmd`, `p8_update_no_network`, `p8_settings_auto_update` |
| 05 | `p8_feedback` (+ default/remote/force), `p8_telemetry` remote restrictive, `p8_internal_otel`, `p8_sentry` |

## Deviations from Plan

None - plan executed exactly as written (green-only scaffolds; no product scope creep).

## Auth Gates

None.

## Known Stubs

None — wave-1 tests are intentional discovery scaffolds, not product stubs.

## Threat Flags

None — no new network endpoints, auth paths, or trust-boundary schema changes.

## Self-Check: PASSED

- FOUND: `crates/codegen/xai-grok-shell/src/agent/config.rs` (p8_telemetry tests)
- FOUND: `crates/codegen/xai-grok-pager/src/app/cli.rs` (p8_wave1 test)
- FOUND: `.planning/phases/08-quiet-fork-rebrand-polish/08-VALIDATION.md`
- FOUND commits: `60cde69`, `78eeb98`
- Verified: `cargo test -p xai-grok-shell --lib p8_telemetry` → 2 passed
- Verified: `cargo test -p xai-grok-pager --lib p8_` → 1 passed
- Verified: `dynamic_enum_model_names` → 2 passed
- Verified: `home_isolation` → 1 passed
