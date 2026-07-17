---
phase: 8
slug: quiet-fork-rebrand-polish
status: green
nyquist_compliant: true
wave_0_complete: true
created: 2026-07-17
updated: 2026-07-17
plans_verified: [01, 02, 03, 04, 05, 06]
gate: 08-PHASE-GATE.md
---

# Phase 8 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution and final gate.
> Prove **ID-02**, **OPS-01**, **OPS-02** (product chrome as bum; stock auto-update hard-off;
> product telemetry / feedback phone-home off by default) with **hermetic unit/integration
> tests only** — no live x.ai update channel or analytics required for CI gates.
>
> **Authority:**
> - ID-02 — user-facing product chrome (clap, welcome/hero, project picker, OAuth return,
>   billing product strings, pager-bin banner, pager-minimal) **plus residual runtime CLI
>   copy** (auth/error, device_code, mcp_doctor, plugin_cmd, headless, shell plugin,
>   pager-bin crash/server). Model catalog labels like `Grok Build (xAI)` / id `grok-build`
>   must **not** regress (D-02).
> - OPS-01 — auto-update chokepoints (`should_check_for_updates`, effective default false,
>   no first-run true persist, `run_update` / CLI no-op message, min-version enforcer no-op,
>   hermetic no-network including `finish_update_on_exit` / Ctrl+U).
> - OPS-02 — `TelemetryMode::Disabled` default, feedback default false + dispatch short-circuit
>   (no `Effect::SendFeedback`), Sentry off by default, **internal OTLP exporter off by
>   default**, **remote settings restrictive-only** for telemetry/feedback, debug
>   `force_feedback_request` gated when feedback disabled.
>
> **Green-only protocol:** Plan 01 landed compile-safe **green** `p8_` scaffolds only — no
> intentional-red under `p8_`. Plans 02–05 added green product proofs. Phase gate (Plan 06)
> claims **all required subgroups green** — never “green except expected-red.”
>
> **Plan 06 finalize:** This file maps every requirement to concrete greened filters + residual
> greps. Runnable sequence: `08-PHASE-GATE.md`.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Cargo built-in `cargo test` (pager lib, update, shell, telemetry, pager-bin, minimal) |
| **Config file** | none global — per-crate; `rust-toolchain.toml` (1.92.0) |
| **Quick run command** | `cargo test -p xai-grok-pager --lib p8_ -- --nocapture` |
| **Full suite command** | See `08-PHASE-GATE.md` (discover+execute all required subgroups) |
| **Estimated runtime** | ~60–180 seconds after first compile (targeted crates only) |

### Cargo verify hygiene (locked)

| Rule | Detail |
|------|--------|
| One TESTNAME filter | Cargo accepts **one** positional TESTNAME filter per invocation |
| Multi-test coverage | Chain single-filter invocations with `&&` only (C1-L3) |
| Exit status | Prefer **no pipe** on cargo execute path that masks failures |
| Chains | Use `&&` only — never `;` or trailing `\|\| true` that masks failures |
| Discovery assert | **Every** required subgroup: `discover()` → `test "$n" -ge 1` then execute |
| Unique prefixes | Phase 8 proofs use `p8_` |
| Green-only | No intentional-red / `#[ignore]` expected-red under filters the gate discovers as `p8_` |
| Forbidden gates | Unfiltered full-workspace test; aggregate-only `grep -c p8_` as sole gate |
| Network | Tests must not require live x.ai update/analytics endpoints |

### Discovery assert helper (canonical)

```bash
set -euo pipefail
discover() {
  local pkg="$1" filter="$2"; shift 2
  local extra=("$@")
  local n
  # list failures must surface: do not hide cargo errors with 2>/dev/null alone
  n=$(cargo test -p "$pkg" "${extra[@]}" -- --list | grep -c "$filter" || true)
  test "$n" -ge 1
  cargo test -p "$pkg" "${extra[@]}" "$filter" -- --nocapture
}
```

> **Per-subgroup rule:** every required filter below must pass its own `discover`.
> Aggregate `grep -c p8_` per crate is **not** a substitute for a missing subgroup.

### Harness policy

| Allowed | Forbidden |
|---------|-----------|
| `cargo test -p xai-grok-pager --lib p8_<filter>` | Live x.ai update download as required gate |
| `cargo test -p xai-grok-update --lib p8_<filter>` | Re-enabling stock auto-update default |
| `cargo test -p xai-grok-pager-bin --test home_isolation` / `p8_` gates | Treating model label “Grok Build (xAI)” as product chrome to rename |
| Pure unit tests for update gates / feedback dispatch | Intentional-red under `p8_` |
| Fixture/env traps (`BUM_HOME`, no network) | Real secrets / live Mixpanel/Sentry in CI |

---

## Requirements → Test Map (final greened)

| Req ID | Behavior | Test Type | Greened filter / command | Exists? | Owning plan | D-NN |
|--------|----------|-----------|--------------------------|---------|-------------|------|
| ID-02 | clap `name`/`about` present as bum | unit | `p8_cli_brand` (`--lib`) | ✅ green | 02 | D-01, D-12 |
| ID-02 | hero badge / subtitle “bum” | unit | `p8_welcome` (`--lib`) | ✅ green | 02 | D-01, D-14 |
| ID-02 | project picker product copy | unit | `p8_project_picker` (covered by `p8_`) | ✅ green | 02 | D-01 |
| ID-02 | headless + plugin_cmd residual CLI instruct bum | unit | `p8_runtime_cli` (`--lib`) | ✅ green | 02 | D-01, C1-H1 |
| ID-02 | billing product strings bum (SuperGrok kept) | unit | `p8_billing` / `billing` (`--lib`) | ✅ green | 02 | D-01 |
| ID-02 | OAuth return “return to bum” | unit | `p8_oauth_return` (shell `--lib`) | ✅ green | 03 | D-01 |
| ID-02 | shell auth/device/mcp/plugin residual CLI instruct bum | unit | `p8_shell_runtime_cli` (shell `--lib`) | ✅ green | 03 | D-01, C1-H1 |
| ID-02 | pager-bin banner + crash/server residual | unit | `p8_bin_` (`--bin bum`) | ✅ green | 03 | D-01 |
| ID-02 | pager-minimal welcome product bum | unit | `p8_minimal_welcome` (`--lib`) | ✅ green | 03 | D-01 |
| ID-02 | residual inventory greps closed (non-vacuous) | static | PHASE-GATE residual greps (C1-H1) | ✅ green | 06 | C1-H1 |
| ID-02 | model catalog still `Grok Build (xAI)` / `grok-build` | regression | `dynamic_enum_model_names` | ✅ green | 01, 06 | D-02 |
| ID-02 | wave-1 pager `p8_` discovery smoke | unit | `p8_wave1` / `p8_` on pager `--lib` | ✅ green | 01 | harness |
| OPS-01 | `should_check_for_updates` always false | unit | `p8_no_auto_update` (`--bin bum`) | ✅ green | 04 | D-04, D-07 |
| OPS-01 | auto_update effective default false; no first-run true | unit | `p8_auto_update` (update `--lib`) | ✅ green | 04 | D-07 |
| OPS-01 | min-version enforcer does not install | unit | `p8_min_version` (update `--lib`) | ✅ green | 04 | D-06 |
| OPS-01 | update CLI/status no-op + locked message | unit | `p8_update_cmd` (`--bin bum`) | ✅ green | 04 | D-05 |
| OPS-01 | hermetic zero stock-helper calls (incl. finish_update_on_exit) | unit | `p8_update_no_network` (`--bin bum`) | ✅ green | 04 | D-05, C1-M2 |
| OPS-01 | settings registry default auto_update false | unit | `p8_settings_auto_update` (pager `--lib`) | ✅ green | 04 | D-07 |
| OPS-02 | telemetry mode default Disabled | unit | `p8_telemetry` (shell `--lib`) | ✅ green | 01, 05 | D-08 |
| OPS-02 | explicit config still can enable (dev path) | unit | `p8_telemetry_disabled_implies_no_easy_enabled_without_explicit_config` | ✅ green | 01 | D-08 |
| OPS-02 | remote telemetry true + local unset stays Disabled | unit | `p8_telemetry_remote_true_local_unset_stays_disabled` | ✅ green | 05 | D-08, C1-H3 |
| OPS-02 | resolve_feedback default false | unit | `p8_feedback_resolve_defaults_to_false` | ✅ green | 05 | D-15 |
| OPS-02 | remote feedback true + local unset stays false | unit | `p8_feedback_remote_true_local_unset_stays_false` | ✅ green | 05 | D-15, C1-H3 |
| OPS-02 | feedback dispatch no `SendFeedback` + locked message | unit | `p8_feedback` (pager `--lib`) | ✅ green | 05 | D-15 |
| OPS-02 | force_feedback / debug trigger no network when disabled | unit | `p8_feedback_force_request_noop_when_disabled` | ✅ green | 05 | D-15, C1-M1 |
| OPS-02 | internal OTLP exporter off by default (TUI + non-TUI) | unit | `p8_internal_otel` (telemetry + shell + bin) | ✅ green | 05 | D-09, C1-H2 |
| OPS-02 | Sentry off by default (unconditional gate) | unit | `p8_sentry` (`--bin bum`) | ✅ green | 05 | D-10, C1-L1 |
| ALL | hermetic isolation / no stock home writes | integration | `home_isolation` (pager-bin) | ✅ green | 01, 06 | Phase 1 |

---

## Locked decisions coverage (from CONTEXT D-01..D-15)

| Decision | Verified by |
|----------|-------------|
| D-01 All product chrome → bum | `p8_cli_brand`, `p8_welcome`, picker/OAuth/banner + `p8_runtime_cli` / `p8_shell_runtime_cli` + residual greps |
| D-02 Keep model brands | `dynamic_enum_model_names` — catalog still `Grok Build (xAI)` / `grok-build` |
| D-03 Leave internal crates `xai-grok-*` | Static inventory — no rename tasks; residual allowlist |
| D-04 Hard-off stock auto-update | `p8_no_auto_update` / `should_check_for_updates` always false |
| D-05 Update command no-op + clear message | `p8_update_cmd` + UI-SPEC string assert |
| D-06 Min-version ignore | `p8_min_version` |
| D-07 Hermetic no stock update helpers + default false | `p8_update_no_network`, `p8_auto_update`, `p8_settings_auto_update` |
| D-08 Telemetry Disabled default | `p8_telemetry` |
| D-09 Mixpanel / product analytics off under Disabled | `p8_telemetry` + TelemetryMode::Disabled path |
| D-10 Sentry off default | `p8_sentry` (unconditional discover ≥1) |
| D-11 Local logs under ~/.bum OK | home_isolation + no gate requiring log deletion |
| D-12 Clap name/about bum | `p8_cli_brand` |
| D-13 Leave internal `GROK_*` env family | Static exclusion — residual allowlist; no mass rename |
| D-14 Hero / welcome badge bum | `p8_welcome`, `p8_minimal_welcome` |
| D-15 Quiet `/feedback` no phone-home | `p8_feedback` + default false + remote restrictive + force gate |

---

## Explicit out-of-scope (do not gate)

| Item | Rationale |
|------|-----------|
| Agent/system prompt “Grok Build agent” strings | RESEARCH Q2 RESOLVED: defer beyond chrome gate unless Phase 9 needs |
| Full user-guide markdown archaeology | RESEARCH Q3 RESOLVED: rebrand extracted runtime docs if touched; not full CHANGELOG |
| Public x.ai signed install channel | PROJECT out of scope (deferred) |
| Crate renames `xai-grok-*` | Phase 1 / D-03 lock |
| Mass `GROK_*` → `BUM_*` env rename | D-13 leave-internal |
| Live dual-provider E2E | Phase 9 |

---

## Residual runtime CLI inventory (C1-H1) — owned surfaces

| Surface | Owning plan | Gate proof |
|---------|-------------|------------|
| `pager/src/headless.rs` auth recovery | 02 | `p8_runtime_cli` + residual grep clean |
| `pager/src/plugin_cmd.rs` help/errors | 02 | `p8_runtime_cli` + residual grep clean |
| `shell/src/auth/error.rs` | 03 | `p8_shell_runtime_cli` + residual grep clean |
| `shell/src/auth/device_code.rs` user bail | 03 | `p8_shell_runtime_cli` + residual grep clean |
| `shell/src/mcp_doctor.rs` user println | 03 | `p8_shell_runtime_cli` + residual grep clean |
| `shell/src/plugin.rs` user Display | 03 | `p8_shell_runtime_cli` + residual grep clean |
| `pager-bin/src/main.rs` agent server / crash / banner | 03 | `p8_bin_` + residual grep clean |

### Residual grep patterns (PHASE-GATE; must match **zero**)

Stock product recovery / chrome instructions that must not remain on owned surfaces:

```text
Run `grok login` | Run `grok mcp | Run `grok plugin
Grok agent server starting | Grok crashed during | grok (pager) -
Open Grok Build | Grok Build TUI | Thanks for using Grok
```

### Residual allowlist (not product chrome — do not fail greps)

| Pattern | Rationale |
|---------|-----------|
| Model label `Grok Build (xAI)` / id `grok-build` | D-02 model brand |
| SuperGrok commercial SKU/name | Billing product, not CLI identity |
| Crate/package `xai-grok-*` | D-03 internal names |
| Types `GrokAuth`, headers `x-grok-client-*` | Internal API surface |
| Agent system prompt “Grok Build agent” | Deferred beyond chrome gate |
| Managed source label `grok.com` | Host identity, not CLI binary name |
| Test negative-asserts (`!msg.contains("grok login")`) | Prove absence; not user-facing chrome |
| Dev env knobs `GROK_*` (D-13) | Internal; not advertised TUI path |

### Static non-change proofs (D-03 / D-13)

| Check | How |
|-------|-----|
| D-03 crates still `xai-grok-*` | `rg -l 'name = "xai-grok-' crates/codegen/*/Cargo.toml \| head` — packages retain prefix |
| D-13 `GROK_*` env family left internal | No mass rename in Plans 01–06; residual allowlist documents leave-internal |

---

## OPS-02 OTLP + remote filters (C1-H2, C1-H3)

| Filter | Crate | Proves |
|--------|-------|--------|
| `p8_internal_otel` / `p8_internal_otel_off_by_default` | telemetry `--lib`, shell `--lib`, bin `bum` | Default instrumentation not Server export; exporter.enabled false |
| `p8_telemetry` remote-true local-unset | shell `--lib` | Stays Disabled |
| `p8_feedback` remote-true local-unset | shell `--lib` | Stays false |
| `p8_feedback` force/debug disabled | shell `--lib` | No feedback API record when disabled |
| `p8_sentry` | pager-bin `--bin bum` | Unconditional phase-gate subgroup (C1-L1) |

---

## Sampling Rate

- **Per task commit:** filtered `p8_` tests in touched crate
- **Per wave merge:** all discoverable `p8_` + flipped default asserts for that wave
- **Phase gate (Plan 06):** discover+execute every required subgroup (incl. unconditional `p8_sentry` + `p8_internal_otel`); residual greps; model-label regression green; home_isolation green

---

## Wave gaps (closed)

- [x] Plan 01: green `p8_` scaffolds — shell `p8_telemetry`, pager `p8_wave1`, model catalog + `home_isolation`
- [x] Plan 02: pager TUI product chrome + residual CLI greened
- [x] Plan 03: OAuth return + shell residual + pager-bin/minimal greened
- [x] Plan 04: auto-update / min-version / settings / `p8_update_no_network` greened
- [x] Plan 05: feedback + remote restrictive + OTLP + sentry greened
- [x] Plan 06: `nyquist_compliant: true`, `wave_0_complete: true`, PHASE-GATE + residual greps

### Plan 01 greened filters

| Crate | Filter | Command |
|-------|--------|---------|
| `xai-grok-shell` | `p8_telemetry` | `cargo test -p xai-grok-shell --lib p8_telemetry -- --nocapture` |
| `xai-grok-pager` | `p8_` / `p8_wave1` | `cargo test -p xai-grok-pager --lib p8_ -- --nocapture` |
| `xai-grok-pager` | model brand regression | `cargo test -p xai-grok-pager --test dynamic_enum_model_names -- --nocapture` |
| `xai-grok-pager-bin` | `home_isolation` | `cargo test -p xai-grok-pager-bin --test home_isolation -- --nocapture` |

### Plan 02 greened filters

| Crate | Filter | Command |
|-------|--------|---------|
| `xai-grok-pager` | `p8_cli_brand` | `cargo test -p xai-grok-pager --lib p8_cli_brand -- --nocapture` |
| `xai-grok-pager` | `p8_welcome` | `cargo test -p xai-grok-pager --lib p8_welcome -- --nocapture` |
| `xai-grok-pager` | `p8_runtime_cli` | `cargo test -p xai-grok-pager --lib p8_runtime_cli -- --nocapture` |
| `xai-grok-pager` | `p8_` (aggregate incl. project_picker / billing) | `cargo test -p xai-grok-pager --lib p8_ -- --nocapture` |
| `xai-grok-pager` | model brand regression (D-02) | `cargo test -p xai-grok-pager --test dynamic_enum_model_names -- --nocapture` |

### Plan 03 greened filters

| Crate | Filter | Command |
|-------|--------|---------|
| `xai-grok-shell` | `p8_oauth_return` | `cargo test -p xai-grok-shell --lib p8_oauth_return -- --nocapture` |
| `xai-grok-shell` | `p8_shell_runtime_cli` | `cargo test -p xai-grok-shell --lib p8_shell_runtime_cli -- --nocapture` |
| `xai-grok-pager-minimal` | `p8_minimal_welcome` | `cargo test -p xai-grok-pager-minimal --lib p8_minimal_welcome -- --nocapture` |
| `xai-grok-pager-bin` | `p8_bin_` | `cargo test -p xai-grok-pager-bin --bin bum p8_bin_ -- --nocapture` |

### Plan 04 greened filters

| Crate | Filter | Command |
|-------|--------|---------|
| `xai-grok-pager-bin` | `p8_no_auto_update` | `cargo test -p xai-grok-pager-bin --bin bum p8_no_auto_update -- --nocapture` |
| `xai-grok-pager-bin` | `p8_update_cmd` | `cargo test -p xai-grok-pager-bin --bin bum p8_update_cmd -- --nocapture` |
| `xai-grok-pager-bin` | `p8_update_no_network` | `cargo test -p xai-grok-pager-bin --bin bum p8_update_no_network -- --nocapture` |
| `xai-grok-update` | `p8_auto_update` | `cargo test -p xai-grok-update --lib p8_auto_update -- --nocapture` |
| `xai-grok-update` | `p8_min_version` | `cargo test -p xai-grok-update --lib p8_min_version -- --nocapture` |
| `xai-grok-pager` | `p8_settings_auto_update` | `cargo test -p xai-grok-pager --lib p8_settings_auto_update -- --nocapture` |

### Plan 05 greened filters

| Crate | Filter | Command |
|-------|--------|---------|
| `xai-grok-pager` | `p8_feedback` | `cargo test -p xai-grok-pager --lib p8_feedback -- --nocapture` |
| `xai-grok-shell` | `p8_feedback` | `cargo test -p xai-grok-shell --lib p8_feedback -- --nocapture` |
| `xai-grok-shell` | `p8_telemetry` | `cargo test -p xai-grok-shell --lib p8_telemetry -- --nocapture` |
| `xai-grok-telemetry` | `p8_internal_otel` | `cargo test -p xai-grok-telemetry --lib p8_internal_otel -- --nocapture` |
| `xai-grok-shell` | `p8_internal_otel` | `cargo test -p xai-grok-shell --lib p8_internal_otel -- --nocapture` |
| `xai-grok-pager-bin` | `p8_internal_otel` | `cargo test -p xai-grok-pager-bin --bin bum p8_internal_otel -- --nocapture` |
| `xai-grok-pager-bin` | `p8_sentry` | `cargo test -p xai-grok-pager-bin --bin bum p8_sentry -- --nocapture` |

### Plan 06 greened residual + regressions

| Check | Command | Result |
|-------|---------|--------|
| Residual inventory (C1-H1) | See `08-PHASE-GATE.md` residual section | ✅ GREEN `2026-07-17T11:52:03Z` — 7/7 surfaces clean |
| Model brand (D-02) | `cargo test -p xai-grok-pager --test dynamic_enum_model_names -- --nocapture` | ✅ 2 passed (catalog still `Grok Build (xAI)` / provider suffixes) |
| home_isolation | `cargo test -p xai-grok-pager-bin --test home_isolation -- --nocapture` | ✅ `hermetic_temp_home_writes_only_under_bum_home` |
| Full gate | `bash` sequence in `08-PHASE-GATE.md` | ✅ GREEN — all subgroups discover≥1 + execute; unconditional `p8_sentry` + `p8_internal_otel` |

---

## UI-SPEC copy locks (assert in tests where practical)

| Surface | Locked copy |
|---------|-------------|
| Clap | `name=bum`, about includes `bum TUI` (or equivalent) |
| Hero subtitle | `Thanks for using bum.` |
| Project picker | product name `bum` (not Grok Build) |
| OAuth return | `return to bum` |
| Feedback disabled | `Feedback is disabled in bum (no phone-home).` |
| Update disabled | `Stock auto-update is disabled in bum. Install or build locally to upgrade.` |
