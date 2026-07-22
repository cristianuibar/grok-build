---
phase: 9
slug: daily-driver-end-to-end-validation
status: green
nyquist_compliant: true
wave_0_complete: true
wave_0_inventory_ready: true
created: 2026-07-17
updated: 2026-07-22
plan_01_complete: true
plan_02_complete: true
plan_03_complete: true
plan_04_complete: true
plan_05_complete: true
---

# Phase 9 — Validation Strategy

> Hybrid daily-driver bar for **OPS-03..OPS-06**: automated green (prior `p6_`/`p7_`/`p8_`
> subset + thin fixture `p9_`) **and** signed live dual-login human UAT.
> GREEN ⇔ both halves. Never fixture-only for live OPS rows. No CI live OAuth secrets.

**Plan 01 (inventory):** dual auto/human map finalized + greened `p9_` residual.  
**Plan 02 (PHASE-GATE):** `09-PHASE-GATE.md` full P0/P1/p9_ automated half GREEN.
**Plan 03 (UAT runbook):** `09-UAT.md` + `scripts/uat-preflight.sh` prepared the live matrix.
**Plan 04 (live execute):** OPS-03..06 signed PASS, including both OPS-06 directions.
**Plan 05 (hybrid close):** full automated inventory re-run GREEN on 2026-07-22; canonical verification passed.
**nyquist_compliant:** `true` — both automated and live halves are GREEN.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Cargo built-in `cargo test` (+ bash `discover()` for gates) |
| **Config file** | none global — per-crate; `rust-toolchain.toml` 1.92.0 |
| **Quick run command** | `cargo test -p xai-grok-shell --test model_switch_gate p6_dual_login -- --nocapture` |
| **p9_ residual command** | `cargo test -p xai-grok-shell --test cross_provider_subagent p9_ -- --nocapture` |
| **Full suite command** | See `09-PHASE-GATE.md` (Plan 02 — prior subset + thin `p9_` + residual + human sign-off) |
| **Estimated runtime** | ~2–8 min automated after compile; live UAT ~30–90 min human |

### Cargo verify hygiene (locked)

| Rule | Detail |
|------|--------|
| One TESTNAME filter | One positional filter per `cargo test` |
| Multi-test coverage | Chain with `&&` only — never `;` or `\|\| true` masking failures |
| Discovery assert | Every required subgroup: `discover` → `n >= 1` then execute |
| Unique prefixes | New thin proofs `p9_`; re-runs keep `p6_`/`p7_`/`p8_` names |
| Green-only | No intentional-red / expected-fail under gate filters |
| Forbidden | Unfiltered full workspace; bare `cargo test -p … --lib` as sole gate; aggregate-only `grep -c p9_` |
| Automated network | Fixture tokens only (`xai-fake-token*` / `codex-fake-token*`) |
| Secrets | Never commit tokens / auth.json / secret transcripts (D-12) |

### Discovery assert helper (canonical)

```bash
set -euo pipefail
discover() {
  local pkg="$1" filter="$2"; shift 2
  local extra=("$@")
  local n
  n=$(cargo test -p "$pkg" "${extra[@]}" -- --list | grep -c "$filter" || true)
  test "$n" -ge 1
  cargo test -p "$pkg" "${extra[@]}" "$filter" -- --nocapture
}
```

---

## Sampling Rate

- **After every task commit:** focused filter for touched crate; docs-only → static path check
- **After every plan wave:** all automated subgroups planned for that wave green
- **Phase gate:** full automated aggregate **and** signed UAT for OPS-03..06
- **Max feedback latency:** targeted crates preferred; avoid full workspace

---

## Phase Requirements → Dual Auto / Human Map

Every OPS row has **both** automated residual command(s) **and** required live UAT proof.
Live UAT is **Required** (not advisory). Fixture-green alone never marks live OPS PASS (D-16).

| Req ID | Behavior | Automated residual command(s) | Live UAT proof | D-NN | Status |
|--------|----------|-------------------------------|----------------|------|--------|
| **OPS-03** | Productive xAI tool turn after login | Routing/auth residual + `home_isolation` + `p9_`; full residual re-run GREEN 2026-07-22 | `09-UAT.md` §OPS-03; operator live PASS 2026-07-18 | D-01, D-02, D-05, D-08, D-09, D-16 | ✅ PASS |
| **OPS-04** | Productive GPT-5.6 tool turn after Codex login | `p9_`, dual-login, missing-provider and Codex regression coverage GREEN | `09-UAT.md` §OPS-04; operator live PASS 2026-07-18 | D-01, D-02, D-05, D-08, D-09, D-16 | ✅ PASS |
| **OPS-05** | Same-session switch productive | `p6_dual_login`, `p6_missing_provider`, and `switch_changes_next_sample_route` GREEN 2026-07-22 | `09-UAT.md` §OPS-05; same-process live PASS 2026-07-18 | D-01, D-02, D-06, D-08, D-09, D-16 | ✅ PASS |
| **OPS-06** | Cross-provider spawn both dirs live | Both direction names present; `p7_isolation`, eager, missing-provider, parent-model and same-provider residuals GREEN 2026-07-22 | `09-UAT.md` §OPS-06; both directions live PASS 2026-07-20 | D-01, D-02, D-07, D-08, D-09, D-16 | ✅ PASS |
| ALL | Quiet fork still holds | `p8_telemetry` (shell `--lib`); `p8_no_auto_update` + `p8_sentry` (pager-bin); `home_isolation` | n/a (auto residual; chrome checked in UAT preflight) | D-11, D-14 | ✅ greened Plan 02 re-run |
| ALL | Thin phase discovery smoke | **`p9_daily_driver_dual_slot_readable_and_empty_codex_login_hint`** via `cargo test -p xai-grok-shell --test cross_provider_subagent p9_ -- --nocapture` | n/a (fixture-only; never substitutes live OPS) | D-11, D-12 | ✅ greened Plan 01 (+ re-run Plan 02) |

### Live UAT column (hard rule)

| Behavior | Requirement | Why Manual | Required? | Instructions |
|----------|-------------|------------|-----------|--------------|
| Real xAI coding turn (read + edit/shell) | OPS-03 | Live backend + OAuth | **Required** | `09-UAT.md` §OPS-03 — after preflight (`scripts/uat-preflight.sh`): disposable workspace (C1-M3) + CLI chrome (C1-M4) + dual login; live read + edit/shell on disposable path |
| Real Codex/GPT-5.6 coding turn | OPS-04 | Live backend + OAuth | **Required** | `09-UAT.md` §OPS-04 — same preflight; GPT-5.6 catalog entry (default `gpt-5.6-sol`); document capability gaps |
| Same-session provider switch | OPS-05 | Real multi-turn session | **Required** | `09-UAT.md` §OPS-05 — same CLI process; xAI → `/model` GPT without restart (optional reverse) |
| Cross-provider Task both dirs | OPS-06 | Live dual-login spawn | **Required** | `09-UAT.md` §OPS-06 — **both** dirs Grok→Codex and Codex→Grok (NL or Task + model + effort) |

**Preflight helper (non-secret):** `.planning/phases/09-daily-driver-end-to-end-validation/scripts/uat-preflight.sh`  
Prints numbered steps, optional binary build, CLI chrome sample (`bum --help`), fail-closed scoped credential path guards + phase-diff secret scan. Never stores tokens or auto-marks UAT PASS.

**Do not** mark live OPS rows green on fixture-only evidence.  
**Do not** treat environment skip as phase pass.  
If network/account fails → block human path (fix or re-auth) (D-16).

Default models: current xAI daily (e.g. `grok-build`) + one GPT-5.6 catalog entry (e.g. `gpt-5.6-sol`). Prefer TUI path. Disposable workspace + chrome preflight required before matrix (see `09-UAT.md`).

---

## Greened `p9_` residual (Plan 01)

| Property | Value |
|----------|-------|
| **Test name** | `p9_daily_driver_dual_slot_readable_and_empty_codex_login_hint` |
| **File** | `crates/codegen/xai-grok-shell/tests/cross_provider_subagent.rs` |
| **Discover + execute** | `cargo test -p xai-grok-shell --test cross_provider_subagent p9_ -- --nocapture` |
| **Count** | Exactly **1** required residual (≤3 allowed; optional route residual not landed) |
| **Differentiation (C1-M1)** | Single **composition** residual: dual-slot write + both providers readable via `read_provider_auth_store` **and** empty-Codex fail-closed login-hint (`bum login --provider codex`) in one test — **not** dual clones of `p7_wave0_harness_smoke_compiles_and_runs` / `p7_missing_provider_gate_error_suggests_bum_login_for_empty_codex` |
| **Fixtures** | `xai-fake-token-p9` / `codex-fake-token-p9` under temp `BUM_HOME` via `ensure_sandbox` (OnceLock hygiene; no ambient `~/.bum`) |
| **Network** | None — fixture-only (D-04, D-11) |
| **Bearer / Authorization isolation** | **Not** claimed here — remains **`p7_isolation_*`** (C1-L1) |
| **Route metadata isolation** | Optional `p9_route_metadata_*` **not** added — host/slot already covered by `p7_resolve_route_isolates_base_url_key_prefix_both_directions` (re-run residual, not re-implemented under `p9_`) |
| **p7 re-run residual (not re-copied under p9_)** | `p7_wave0_harness_smoke_*` + `p7_missing_provider_gate_error_*` stay Plan 02 re-run inventory (C1-M1) |

```bash
# Plan 01 verify (must stay green)
set -euo pipefail
n=$(cargo test -p xai-grok-shell --test cross_provider_subagent p9_ -- --list 2>/dev/null | grep -c 'p9_' || true)
test "$n" -ge 1 && test "$n" -le 3
cargo test -p xai-grok-shell --test cross_provider_subagent p9_ -- --nocapture
```

---

## Recommended automated re-run inventory (P0 / P1 / P2)

Source of truth for Plan 02 PHASE-GATE automated half (C1-M2). Plan 01 maps filters; Plan 02 discovers+executes.
**Plan 02 status:** full inventory GREEN on `2026-07-17T15:21:09Z` — see `09-PHASE-GATE.md` gate results.
Cross-link: run from repo root the copy-paste block in **`09-PHASE-GATE.md`** (sole automated SoT).

| Priority | Package | Target | Filter | Why | Notes |
|----------|---------|--------|--------|-----|-------|
| P0 | `xai-grok-shell` | `--test model_switch_gate` | `p6_dual_login` | OPS-05 residual | Dual-login fixture |
| P0 | `xai-grok-shell` | `--test model_switch_gate` | `p6_missing_provider` | Fail-closed residual | Login-hint path |
| P0 | `xai-grok-shell` | `--test provider_routing` | `switch_changes_next_sample_route` | Route after switch | OPS-05 auto half |
| P0 | `xai-grok-shell` | `--lib` | `p7_isolation` | OPS-06 Authorization residual | Aggregate execute once |
| P0 | `xai-grok-shell` | `--lib` | `p7_isolation_grok_parent_codex` | Dir 1 discover | list_only then aggregate OK |
| P0 | `xai-grok-shell` | `--lib` | `p7_isolation_codex_parent_grok` | Dir 2 discover | list_only then aggregate OK |
| P0 | `xai-grok-tools` | `--lib` | `p7_eager` | Spawn preflight residual | |
| P0 | `xai-grok-shell` | `--lib` | `p7_spawn_missing_provider` | Fail-closed spawn | |
| P0 | `xai-grok-shell` | `--lib` | `p7_parent_model` | Parent model stability | |
| P1 | `xai-grok-shell` | `--test cross_provider_subagent` | `p7_spawn_same_provider` | AGENT-01 residual | p7 re-run, not p9_ |
| P1 | `xai-grok-shell` | `--lib` | `p8_telemetry` | Quiet fork residual | |
| P1 | `xai-grok-pager-bin` | `--bin bum` | `p8_no_auto_update` | No stock overwrite | |
| P1 | `xai-grok-pager-bin` | `--bin bum` | `p8_sentry` | No phone-home | |
| P1 | `xai-grok-pager-bin` | `--test home_isolation` | `hermetic` / full | Identity residual | discover-before-execute |
| P2 | `xai-grok-shell` | `--test cross_provider_subagent` | `p9_` | Thin discovery residual | **Plan 01 greened** |

**C1-M1 note:** `p7_wave0_harness_smoke` and `p7_missing_provider_gate_error_suggests_bum_login_for_empty_codex` behavioral coverage is **re-run residual** under Plan 02 (or covered compositionally by `p9_`), not re-implemented as dual `p9_` clones.

---

## D-NN coverage for OPS rows

| Decision | Covers | How this inventory honors it |
|----------|--------|------------------------------|
| D-01 | Hybrid methodology | Dual auto + human columns on every OPS row |
| D-02 | Gate needs both halves | Live rows Required; auto residual listed; GREEN deferred to Plan 05 |
| D-03 | Fix blockers in-phase only | Product code out of Plan 01 scope |
| D-04 | Live under `~/.bum` / temp `BUM_HOME`; no CI live secrets | `p9_` fixture-only temp `BUM_HOME`; live path Plan 03–04 |
| D-05 | OPS-03/04 productive tool turn | Live UAT required; auto residual only |
| D-06 | OPS-05 same process switch | Live UAT + `p6_dual_login` / switch route residual |
| D-07 | OPS-06 both spawn dirs | Live UAT + `p7_isolation` both dirs |
| D-08 | Models under test | Defaults: `grok-build` + `gpt-5.6-sol` |
| D-09 | Structured evidence | UAT + VERIFICATION (Plans 03/05) |
| D-10 | Honest capability gaps | Document in UAT/VERIFICATION if found |
| D-11 | Automated: prior gates + thin p9_; green-only; no live network | Inventory + greened `p9_` |
| D-12 | Never commit secrets | Fake tokens only; no auth.json in git |
| D-13 | Validation-first | Plan 01 docs + fixture residual only |
| D-14 | Deferred PROJECT outs | See Exclusions |
| D-15 | Cristian runs live UAT | Human path later plans |
| D-16 | Live OAuth required for green OPS | All four signed live rows PASS; no fixture substitution |

---

## Wave 0 Requirements

- [x] `09-VALIDATION.md` — this file (dual auto/human map; nyquist frontmatter) — **Plan 01**
- [x] `09-PHASE-GATE.md` — discover+execute aggregate + human sign-off section — **Plan 02** (automated half GREEN; human UAT unsigned)
- [x] `09-UAT.md` — required live dual-login checklist (not advisory) — **Plan 03**
- [x] Thin `p9_*` green smoke (0–3 tests) — fixture dual residual / discovery — **Plan 01** (`p9_daily_driver_dual_slot_readable_and_empty_codex_login_hint`)
- [x] `scripts/uat-preflight.sh` — non-secret helper (path + phase-diff secret gates) — **Plan 03**
- [x] After execute: `09-VERIFICATION.md` with OPS rows + model IDs + operator sign-off — **Plan 05**

**Wave 0 inventory readiness (Plan 01):** VALIDATION + `p9_` complete.  
**PHASE-GATE automated half:** full P0/P1/p9_ inventory GREEN, freshly re-run 2026-07-22.
**UAT:** OPS-03..06 signed live PASS.
**Full Wave 0 close:** complete through Plan 05.
Frontmatter: `wave_0_inventory_ready: true`; `wave_0_complete: true`; `nyquist_compliant: true`.

---

## Explicit exclusions

| Exclusion | Authority |
|-----------|-----------|
| Public signed install channel, crate rename (`xai-grok-*`), mass `GROK_*` env rename, custom agentic workflows, full catalog matrix, enterprise IdP, fleet cost dashboards | D-14 |
| CI live OAuth secrets / browser automation as required gate | D-04 |
| Intentional-red / expected-fail under `p9_` or phase gate filters | D-11 |
| Marking live OPS-03..06 green from fixtures alone | D-16 |
| Dual `p9_` clones of `p7_wave0_harness_smoke_*` and `p7_missing_provider_gate_error_suggests_bum_login_for_empty_codex` | C1-M1 |
| Claiming HTTP Authorization / bearer isolation from route metadata or `p9_` residual | C1-L1 (`p7_isolation_*` owns bearer) |
| Product feature work in Plan 01 (validation-first) | D-13 |

---

## Validation Sign-Off

- [x] Plan 01 tasks have `<automated>` verify
- [x] Sampling continuity: Plan 01 Task 1 cargo + Task 2 static rg
- [x] Wave 0 inventory for VALIDATION + `p9_` closed
- [x] PHASE-GATE automated half discover+execute full P0/P1/p9_ inventory — **Plan 02**
- [x] UAT runbook + non-secret preflight (Plan 03)
- [x] Full Wave 0 (signed live UAT + VERIFICATION) — Plans 04/05
- [x] No watch-mode flags
- [x] Hybrid gate: automated green **and** signed UAT — Plan 05
- [x] `nyquist_compliant: true` set in frontmatter when gate closes

**Approval:** GREEN — automated residual and signed live UAT halves both pass; canonical verification passed.
)
