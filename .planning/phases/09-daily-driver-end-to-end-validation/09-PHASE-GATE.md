---
phase: 9
slug: daily-driver-end-to-end-validation
plan: 02
status: green
gate_started: 2026-07-17T15:14:46Z
gate_passed_automated: 2026-07-17T15:21:09Z
requirements: [OPS-03, OPS-04, OPS-05, OPS-06]
fixture_only: true
green_only: true
human_uat_required: true
human_uat_passed: true
gate_passed_hybrid: 2026-07-20
gate_reverified: 2026-07-22
nyquist_compliant: true
---

# Phase 9 — Phase Gate Runbook

Hybrid proof for **OPS-03..OPS-06** (productive dual-provider daily-driver sessions):
automated residual re-run of critical `p6_`/`p7_`/`p8_` subgroups + thin `p9_` discovery
smoke, **plus** signed live dual-login human UAT (Plans 03–05).

- **Fixtures only (automated half)** — no live ChatGPT / xAI OAuth tokens or secret env vars.
- **Green-only** — all required inventory filters pass; **no intentional-red** under gate
  filters (D-11).
- **Per-subgroup discovery ≥1** before each execute (T-09-03 / Phase 6–8 lesson). Aggregate
  `grep -c p9_` alone is **not** sufficient.
- **C1-M2:** Task 2 verify + this document claim the **same full P0/P1/p9_ inventory** —
  no subset SoT.
- **C1-L3 isolation:** list both direction names (`p7_isolation_grok_parent_codex`,
  `p7_isolation_codex_parent_grok`) then execute aggregate `p7_isolation` **once** — do
  **not** re-execute the two direction-specific filters after the aggregate.
- **C1-L2 home_isolation:** discover-before-execute (`hermetic` or `home` names ≥1) then
  run the integration target — never bare execute without discovery.
- **Chains use `&&` only** — never `;` / `|| true` masking failures on execute paths.
- **One positional TESTNAME** per `cargo test` invocation.
- **D-02 / D-16:** automated half GREEN does **not** claim live OPS-03..06 PASS. Full gate
  GREEN requires signed human UAT (`human_uat_required: true`).
- **Forbidden:** unfiltered `cargo test -p xai-grok-shell --lib` as sole gate; live multi-turn
  as automated; CI OAuth secrets; marking live OPS from fixtures; inventing intentional-red
  wrappers when discover n=0 (restore prior greened names instead).

Canonical map: `09-VALIDATION.md`. Prior product: `07-PHASE-GATE.md` / `08-PHASE-GATE.md`.

---

## Discover + execute helpers

```bash
set -euo pipefail

# discover+execute: list count ≥1 then run filter (canonical 07/08/VALIDATION)
discover() {
  local pkg="$1" filter="$2"; shift 2
  local extra=("$@")
  local n
  n=$(cargo test -p "$pkg" "${extra[@]}" -- --list 2>/dev/null | grep -c "$filter" || true)
  echo "discover $pkg $filter (${extra[*]:-}): n=$n"
  test "$n" -ge 1
  cargo test -p "$pkg" "${extra[@]}" "$filter" -- --nocapture
}

# list_only: direction-name presence without re-executing (C1-L3)
list_only() {
  local pkg="$1" filter="$2"; shift 2
  local extra=("$@")
  local n
  n=$(cargo test -p "$pkg" "${extra[@]}" -- --list 2>/dev/null | grep -c "$filter" || true)
  echo "list_only $pkg $filter (${extra[*]:-}): n=$n"
  test "$n" -ge 1
}
```

### Target selection notes

| Case | Why |
|------|-----|
| Isolation / spawn / parent / telemetry on **`--lib`** | Phase 7–8 in-crate seams; prefer lib when both exist |
| OPS-05 switch residual on **`--test model_switch_gate`** / **`provider_routing`** | Phase 6 integration targets |
| Thin `p9_` / optional same-provider on **`--test cross_provider_subagent`** | Plan 01 residual + Phase 7 harness |
| Quiet-fork bin knobs on **`--bin bum`** | Composition-root unit tests |
| Identity on **`--test home_isolation`** | Integration; discover hermetic/home first (C1-L2) |

---

## Full P0 / P1 / P2 automated inventory (SoT — C1-M2)

| Group | Package / target | Filter | Mode |
|-------|------------------|--------|------|
| P2 | `xai-grok-shell --test cross_provider_subagent` | `p9_` | discover+execute |
| P0 OPS-05 | `xai-grok-shell --test model_switch_gate` | `p6_dual_login` | discover+execute |
| P0 OPS-05 | `xai-grok-shell --test model_switch_gate` | `p6_missing_provider` | discover+execute |
| P0 OPS-05 | `xai-grok-shell --test provider_routing` | `switch_changes_next_sample_route` | discover+execute |
| P0 OPS-06 | `xai-grok-shell --lib` | `p7_isolation_grok_parent_codex` | list_only (direction name) |
| P0 OPS-06 | `xai-grok-shell --lib` | `p7_isolation_codex_parent_grok` | list_only (direction name) |
| P0 OPS-06 | `xai-grok-shell --lib` | `p7_isolation` | discover+execute **once** (aggregate both dirs — C1-L3) |
| P0 OPS-06 | `xai-grok-tools --lib` | `p7_eager` | discover+execute |
| P0 OPS-06 | `xai-grok-shell --lib` | `p7_spawn_missing_provider` | discover+execute |
| P0 OPS-06 | `xai-grok-shell --lib` | `p7_parent_model` | discover+execute |
| P1 quiet | `xai-grok-shell --lib` | `p8_telemetry` | discover+execute |
| P1 quiet | `xai-grok-pager-bin --bin bum` | `p8_no_auto_update` | discover+execute |
| P1 quiet | `xai-grok-pager-bin --bin bum` | `p8_sentry` | discover+execute |
| P1 quiet | `xai-grok-pager-bin --test home_isolation` | `hermetic` / `home` | discover then execute (C1-L2) |
| P1 optional | `xai-grok-shell --test cross_provider_subagent` | `p7_spawn_same_provider` | discover+execute if present |

---

## Copy-paste aggregate sequence (automated half — sole runtime SoT)

Run from repo root. This block is the sole automated source of truth (C1-M2).

```bash
set -euo pipefail

discover() {
  local pkg="$1" filter="$2"; shift 2
  local extra=("$@")
  local n
  n=$(cargo test -p "$pkg" "${extra[@]}" -- --list 2>/dev/null | grep -c "$filter" || true)
  echo "discover $pkg $filter (${extra[*]:-}): n=$n"
  test "$n" -ge 1
  cargo test -p "$pkg" "${extra[@]}" "$filter" -- --nocapture
}

list_only() {
  local pkg="$1" filter="$2"; shift 2
  local extra=("$@")
  local n
  n=$(cargo test -p "$pkg" "${extra[@]}" -- --list 2>/dev/null | grep -c "$filter" || true)
  echo "list_only $pkg $filter (${extra[*]:-}): n=$n"
  test "$n" -ge 1
}

# ========== P2 — thin phase discovery smoke (D-11) ==========
discover xai-grok-shell p9_ --test cross_provider_subagent

# ========== P0 OPS-05 residual (full — C1-M2) ==========
discover xai-grok-shell p6_dual_login --test model_switch_gate
discover xai-grok-shell p6_missing_provider --test model_switch_gate
discover xai-grok-shell switch_changes_next_sample_route --test provider_routing

# ========== P0 OPS-06 residual (isolation hygiene C1-L3) ==========
# Direction-name presence only — do NOT re-execute these filters after aggregate
list_only xai-grok-shell p7_isolation_grok_parent_codex --lib
list_only xai-grok-shell p7_isolation_codex_parent_grok --lib
# Aggregate once runs both dirs
discover xai-grok-shell p7_isolation --lib

discover xai-grok-tools p7_eager --lib
discover xai-grok-shell p7_spawn_missing_provider --lib
discover xai-grok-shell p7_parent_model --lib

# Optional P1 same-provider residual (if discoverable)
if cargo test -p xai-grok-shell --test cross_provider_subagent -- --list 2>/dev/null \
  | grep -q 'p7_spawn_same_provider'; then
  discover xai-grok-shell p7_spawn_same_provider --test cross_provider_subagent
fi

# ========== P1 quiet residual (full — C1-M2) ==========
discover xai-grok-shell p8_telemetry --lib
discover xai-grok-pager-bin p8_no_auto_update --bin bum
discover xai-grok-pager-bin p8_sentry --bin bum

# home_isolation discover-before-execute (C1-L2)
echo "discover home_isolation (hermetic|home)"
n_home=$(cargo test -p xai-grok-pager-bin --test home_isolation -- --list 2>/dev/null \
  | grep -cE 'hermetic|home' || true)
echo "discover xai-grok-pager-bin home_isolation: n=$n_home"
test "$n_home" -ge 1
cargo test -p xai-grok-pager-bin --test home_isolation -- --nocapture

echo "PHASE 9 AUTOMATED HALF: ALL SUBGROUPS GREEN"
echo "NOTE: automated half GREEN still requires human_uat for full gate GREEN (D-02, D-16)"
```

---

## Checklist — automated half (must all pass before automated GREEN)

| # | Check | Status |
|---|-------|--------|
| 1 | `p9_` discover+execute (`cross_provider_subagent`) | ✅ n=1 |
| 2 | `p6_dual_login` discover+execute | ✅ n=3 |
| 3 | `p6_missing_provider` discover+execute | ✅ n=3 |
| 4 | `switch_changes_next_sample_route` discover+execute | ✅ n=1 |
| 5 | `p7_isolation_grok_parent_codex` list_only n≥1 | ✅ n=1 |
| 6 | `p7_isolation_codex_parent_grok` list_only n≥1 | ✅ n=1 |
| 7 | `p7_isolation` aggregate discover+execute **once** | ✅ n=4 |
| 8 | `p7_eager` (tools `--lib`) discover+execute | ✅ n=7 |
| 9 | `p7_spawn_missing_provider` (`--lib`) discover+execute | ✅ n=5 |
| 10 | `p7_parent_model` discover+execute | ✅ n=1 |
| 11 | `p8_telemetry` discover+execute | ✅ n=3 |
| 12 | `p8_no_auto_update` discover+execute | ✅ n=1 |
| 13 | `p8_sentry` discover+execute | ✅ n=1 |
| 14 | `home_isolation` discover (hermetic\|home) then execute | ✅ n=1 |
| 15 | Green-only — no intentional-red exception | ✅ |
| 16 | No unfiltered shell `--lib` sole gate; fixture tokens only | ✅ |

---

## Human UAT section (required for full gate GREEN — D-02, D-16)

**human_uat_required: true**

Automated residual green is **not** sufficient. Live dual-login UAT must be signed for
each OPS row before phase gate GREEN. Evidence lives in `09-UAT.md` (Plan 03) and
`09-VERIFICATION.md` (Plan 05). **Do not** mark live OPS from fixtures alone.

| Req | Live proof | Signed PASS |
|-----|------------|-------------|
| **OPS-03** | Productive xAI tool turn after login (`09-UAT.md` §OPS-03) | ✅ PASS |
| **OPS-04** | Productive GPT-5.6 tool turn after Codex login (`09-UAT.md` §OPS-04) | ✅ PASS |
| **OPS-05** | Same-session switch without restart (`09-UAT.md` §OPS-05) | ✅ PASS |
| **OPS-06** | Cross-provider spawn both dirs live (`09-UAT.md` §OPS-06) | ✅ PASS |

### Human gate fail conditions

- Any OPS-03..06 live row unsigned or fixture-only claimed as PASS
- Environment skip treated as phase pass
- Secrets / tokens / `auth.json` committed or pasted into docs (D-12)
- Network/account failure left unresolved without re-auth or fix (D-16)

### Secrets policy (D-12)

- Automated: fixture tokens only (`xai-fake-token*` / `codex-fake-token*`)
- Live UAT: operator-local `~/.bum` credentials — never commit
- Gate logs must not dump Authorization headers or refresh tokens

---

## Gate results (filled by Plan 02 Task 2)

**Automated half status:** GREEN  
**Full gate status:** GREEN (automated residual + signed live OPS-03..06)
**Started:** `2026-07-17T15:14:46Z`  
**Passed (automated):** `2026-07-17T15:21:09Z`  
**Passed (hybrid):** `2026-07-20`
**Fresh automated re-verification:** `2026-07-22` — all documented subgroups GREEN
**Host:** local cargo (debug test profile)  
**Protocol:** per-subgroup discover ≥1 then execute; list_only both isolation dirs then aggregate once; home_isolation discover-before-execute; `&&` chains only; no intentional-red; no live OAuth

### Per-subgroup results (automated)

| Subgroup | Package | Target | Mode | Discovered ≥1 | Result |
|----------|---------|--------|------|---------------|--------|
| `p9_` | shell | `--test cross_provider_subagent` | discover+execute | 1 | pass (1 ok) |
| `p6_dual_login` | shell | `--test model_switch_gate` | discover+execute | 3 | pass (3 ok) |
| `p6_missing_provider` | shell | `--test model_switch_gate` | discover+execute | 3 | pass (3 ok) |
| `switch_changes_next_sample_route` | shell | `--test provider_routing` | discover+execute | 1 | pass (1 ok) |
| `p7_isolation_grok_parent_codex` | shell | `--lib` | list_only | 1 | present (C1-L3) |
| `p7_isolation_codex_parent_grok` | shell | `--lib` | list_only | 1 | present (C1-L3) |
| `p7_isolation` | shell | `--lib` | discover+execute once | 4 | pass (4 ok; both dirs) |
| `p7_eager` | tools | `--lib` | discover+execute | 7 | pass (7 ok) |
| `p7_spawn_missing_provider` | shell | `--lib` | discover+execute | 5 | pass (5 ok) |
| `p7_parent_model` | shell | `--lib` | discover+execute | 1 | pass (1 ok) |
| `p8_telemetry` | shell | `--lib` | discover+execute | 3 | pass (3 ok) |
| `p8_no_auto_update` | pager-bin | `--bin bum` | discover+execute | 1 | pass (1 ok) |
| `p8_sentry` | pager-bin | `--bin bum` | discover+execute | 1 | pass (1 ok) |
| `home_isolation` | pager-bin | `--test home_isolation` | discover then execute | 1 (`hermetic`) | pass (1 ok) |

---

## Explicit exclusions

| Exclusion | Authority |
|-----------|-----------|
| Live multi-turn dual-login as **automated** gate | D-04, D-11 |
| CI live OAuth secrets | D-04, D-12 |
| Marking live OPS-03..06 green from fixtures alone | D-16 |
| Intentional-red under gate filters | D-11 |
| Full re-run of entire 06/07/08 gates beyond this inventory | Plan 02 scope / C1-M2 minimum |
| Public install channel, crate rename, mass `GROK_*`, custom workflows | D-14 |
| Claiming `nyquist_compliant: true` before human sign-off | Plan 05 |

---

## Footer (D-02)

**Full hybrid gate GREEN.**
`human_uat_required: true` · `human_uat_passed: true` · OPS-03..06 live PASS ·
`nyquist_compliant: true`.
