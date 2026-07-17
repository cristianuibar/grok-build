---
phase: 8
slug: quiet-fork-rebrand-polish
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-07-17
updated: 2026-07-17
plans_verified: []
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
>   must **not** regress.
> - OPS-01 — auto-update chokepoints (`should_check_for_updates`, effective default false,
>   no first-run true persist, `run_update` / CLI no-op message, min-version enforcer no-op,
>   hermetic no-network including `finish_update_on_exit` / Ctrl+U).
> - OPS-02 — `TelemetryMode::Disabled` default, feedback default false + dispatch short-circuit
>   (no `Effect::SendFeedback`), Sentry off by default, **internal OTLP exporter off by
>   default**, **remote settings restrictive-only** for telemetry/feedback, debug
>   `force_feedback_request` gated when feedback disabled.
>
> **Green-only protocol:** Plan 01 lands compile-safe **green** `p8_` scaffolds only — no
> intentional-red under `p8_`. Plans 02–05 add green product proofs. Phase gate claims
> **all required subgroups green** — never “green except expected-red.”
>
> **Plan-time scaffold:** This file is authored at plan time (Nyquist 8e). Plan 01 updates
> filter inventory as scaffolds land; Plan 06 finalizes greened filters + PHASE-GATE.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Cargo built-in `cargo test` (pager lib, update, shell config, pager-bin tests) |
| **Config file** | none global — per-crate; `rust-toolchain.toml` (1.92.0) |
| **Quick run command** | `cargo test -p xai-grok-pager --lib p8_ -- --nocapture` |
| **Full suite command** | See Plan 06 / `08-PHASE-GATE.md` (discover+execute all `p8_` subgroups) |
| **Estimated runtime** | ~60–180 seconds after first compile (targeted crates only) |

### Cargo verify hygiene (locked)

| Rule | Detail |
|------|--------|
| One TESTNAME filter | Cargo accepts **one** positional TESTNAME filter per invocation |
| Multi-test coverage | Chain single-filter invocations with `&&` |
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

## Requirements → Test Map

| Req ID | Behavior | Test Type | Planned filter / command | Exists at plan-time? | Owning plan |
|--------|----------|-----------|--------------------------|----------------------|-------------|
| ID-02 | clap `name`/`about` present as bum | unit | `p8_cli_brand` / `p8_clap` | ❌ scaffold Plan 01 | 01, 02 |
| ID-02 | hero badge / subtitle “bum” | unit | `p8_welcome` / `p8_hero` | ❌ scaffold Plan 01 | 01, 02 |
| ID-02 | project picker product copy | unit | `p8_project_picker` | ❌ | 02 |
| ID-02 | headless + plugin_cmd residual CLI instruct bum | unit | `p8_runtime_cli` | ❌ | 02 |
| ID-02 | OAuth return “return to bum” | unit | `p8_oauth_return` | ❌ | 03 |
| ID-02 | shell auth/device/mcp/plugin residual CLI instruct bum | unit | `p8_shell_runtime_cli` | ❌ | 03 |
| ID-02 | pager-bin / minimal banner “bum” + crash/server residual | unit/e2e | `p8_bin_` / `p8_minimal_welcome` | ❌ | 03 |
| ID-02 | residual inventory greps closed (or documented deferrals) | static | PHASE-GATE residual greps (C1-H1) | ❌ | 06 |
| ID-02 | model catalog still `Grok Build (xAI)` / `grok-build` | regression | existing model catalog + `p8_model_label` | ✅ partial | 01, 06 |
| OPS-01 | `should_check_for_updates` always false | unit | `p8_no_auto_update` / `p8_should_check` | ❌ | 01, 04 |
| OPS-01 | auto_update effective default false; no first-run true | unit | `p8_auto_update_default` | ❌ | 01, 04 |
| OPS-01 | min-version enforcer does not install | unit | `p8_min_version` | ❌ | 04 |
| OPS-01 | update CLI/status no-op + locked message | unit | `p8_update_cmd` | ❌ | 04 |
| OPS-01 | hermetic zero stock-helper calls (incl. finish_update_on_exit) | unit | `p8_update_no_network` | ❌ | 04 |
| OPS-01 | settings registry default auto_update false | unit | settings e2e / `p8_settings_auto_update` | ✅ exists (must flip) | 04 |
| OPS-02 | telemetry mode default Disabled | unit | existing + `p8_telemetry` | ✅ partial | 01, 05 |
| OPS-02 | remote telemetry true + local unset stays Disabled | unit | `p8_telemetry` remote restrictive (C1-H3) | ❌ | 05 |
| OPS-02 | resolve_feedback default false | unit | flip `resolve_feedback_defaults_*` + `p8_feedback_default` | ✅ exists (must flip) | 05 |
| OPS-02 | remote feedback true + local unset stays false | unit | `p8_feedback` remote restrictive (C1-H3) | ❌ | 05 |
| OPS-02 | feedback dispatch no `SendFeedback` + locked message | unit | `p8_feedback` | ❌ | 05 |
| OPS-02 | force_feedback / debug trigger no network when disabled | unit | `p8_feedback` force gate (C1-M1) | ❌ | 05 |
| OPS-02 | internal OTLP exporter off by default (TUI + non-TUI) | unit | `p8_internal_otel` / `p8_internal_otel_off_by_default` | ❌ | 05, 06 |
| OPS-02 | Sentry off by default | unit | `p8_sentry` (unconditional gate) | ❌ | 05, 06 |
| ALL | hermetic isolation / no stock home writes | integration | `home_isolation` (Phase 1) | ✅ | 01, 06 |

---

## Locked decisions coverage (from CONTEXT)

| Decision | Verified by |
|----------|-------------|
| All product chrome → bum | `p8_cli_*`, `p8_welcome*`, picker/OAuth/banner + `p8_runtime_cli` / `p8_shell_runtime_cli` + residual greps |
| Keep model brands | `p8_model_label` + existing catalog tests |
| Hard-off stock auto-update | `p8_should_check` / `p8_no_auto_update` |
| Update command no-op + clear message | `p8_update_cmd` + UI-SPEC string assert |
| Hermetic no stock update helpers | `p8_update_no_network` (incl. finish_update_on_exit) |
| Min-version ignore | `p8_min_version` |
| Telemetry Disabled default | `p8_telemetry` + existing |
| Remote cannot re-enable telemetry/feedback | `p8_telemetry` / `p8_feedback` remote restrictive |
| Internal OTLP off default | `p8_internal_otel` |
| Feedback quiet / no phone-home | `p8_feedback` + default false + force gate |
| Sentry off default | `p8_sentry` (unconditional discover) |
| Leave internal crates / GROK_* env | no rename tasks; residual allowlist |

---

## Explicit out-of-scope (do not gate)

| Item | Rationale |
|------|-----------|
| Agent/system prompt “Grok Build agent” strings | RESEARCH Q2 RESOLVED: defer beyond chrome gate unless Phase 9 needs |
| Full user-guide markdown archaeology | RESEARCH Q3 RESOLVED: rebrand extracted runtime docs if touched; not full CHANGELOG |
| Public x.ai signed install channel | PROJECT out of scope |
| Crate renames `xai-grok-*` | Phase 1 lock |
| Live dual-provider E2E | Phase 9 |

### Residual chrome note

`Open Grok Build` (shell `agent/app.rs` or similar) is **in-scope for ID-02** if still user-visible after Plans 02–03. Plan 03/06 inventory greps must either rebrand it or list it as fixed residual with owning task — not silent exclusion.

### Residual runtime CLI inventory (C1-H1) — owned surfaces

| Surface | Owning plan | Gate |
|---------|-------------|------|
| `pager/src/headless.rs` auth recovery | 02 | `p8_runtime_cli` + residual grep |
| `pager/src/plugin_cmd.rs` help/errors | 02 | `p8_runtime_cli` + residual grep |
| `shell/src/auth/error.rs` | 03 | `p8_shell_runtime_cli` + residual grep |
| `shell/src/auth/device_code.rs` user bail | 03 | `p8_shell_runtime_cli` + residual grep |
| `shell/src/mcp_doctor.rs` user println | 03 | `p8_shell_runtime_cli` + residual grep |
| `shell/src/plugin.rs` user Display | 03 | `p8_shell_runtime_cli` + residual grep |
| `pager-bin/src/main.rs` agent server / crash / banner | 03 | `p8_bin_` + residual grep |

### Residual allowlist (not product chrome — do not fail greps)

| Pattern | Rationale |
|---------|-----------|
| Model label `Grok Build (xAI)` / id `grok-build` | D-02 model brand |
| SuperGrok commercial SKU/name | Billing product, not CLI identity |
| Crate/package `xai-grok-*` | D-03 internal names |
| Types `GrokAuth`, headers `x-grok-client-*` | Internal API surface |
| Agent system prompt “Grok Build agent” | Deferred beyond chrome gate |
| Managed source label `grok.com` | Host identity, not CLI binary name |

### OPS-02 OTLP + remote filters (C1-H2, C1-H3)

| Filter | Proves |
|--------|--------|
| `p8_internal_otel` / `p8_internal_otel_off_by_default` | Default instrumentation not Server export; exporter.enabled false |
| `p8_telemetry` remote-true local-unset | Stays Disabled |
| `p8_feedback` remote-true local-unset | Stays false |
| `p8_feedback` force/debug disabled | No feedback API record when disabled |
| `p8_sentry` | Unconditional phase-gate subgroup |

---

## Sampling Rate

- **Per task commit:** filtered `p8_` tests in touched crate
- **Per wave merge:** all discoverable `p8_` + flipped default asserts for that wave
- **Phase gate (Plan 06):** discover+execute every required subgroup (incl. unconditional `p8_sentry` + `p8_internal_otel`); residual greps; model-label regression green; home_isolation green

---

## Wave gaps (close during execute)

- [ ] Plan 01: green `p8_` scaffolds (cli, welcome, telemetry baseline, model label, isolation smoke)
- [ ] Plan 02–03: product chrome + residual runtime CLI greened filters
- [ ] Plan 04: auto-update / min-version / settings / `p8_update_no_network` greened filters
- [ ] Plan 05: feedback + remote restrictive + OTLP + sentry greened filters
- [ ] Plan 06: `nyquist_compliant: true`, `wave_0_complete: true`, PHASE-GATE doc + residual greps

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
