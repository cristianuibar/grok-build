---
phase: 6
slug: mid-session-switch-missing-provider-gate
plan: 06
status: green
gate_started: 2026-07-16T23:54:10Z
gate_passed: 2026-07-17T00:17:46Z
requirements: [MOD-03, MOD-06]
fixture_only: true
---

# Phase 6 — Phase Gate Runbook

Automated proof for **MOD-03** (mid-session free switch) and **MOD-06** (missing-provider gate + login UX).

- **Fixtures only** — no live ChatGPT / xAI OAuth tokens or secret env vars required (T-06-17).
- **Both halves required** — shell apply gate **and** pager QuestionView / badge / deferred recovery (T-06-18).
- **Per-subgroup discovery ≥1** before each execute group (T-06-18b / review cycle 2). Aggregate `grep -c p6_` alone is **not** sufficient.
- **Forbidden:** unfiltered `cargo test -p xai-grok-shell --lib` as the sole gate; bare filters `auth_complete`, `needs_login`, `deferred_model_switch` without `p6_`.

Canonical helper is shared with `06-VALIDATION.md`.

---

## Discover + execute helper

```bash
set -euo pipefail

discover() {
  local pkg="$1" filter="$2"; shift 2
  local extra=("$@")
  local n
  n=$(cargo test -p "$pkg" "${extra[@]}" -- --list 2>/dev/null | grep -c "$filter" || true)
  test "$n" -ge 1
  cargo test -p "$pkg" "${extra[@]}" "$filter" -- --nocapture
}
```

### Target selection notes (drift fixes)

| Case | Why |
|------|-----|
| Shell unit `p6_model_switch_missing_provider` uses **`--lib`** | Bare package `-- --list` compiles **all** integration tests; unrelated `signed_managed_config*` fails on missing `test_seam`, yielding empty discovery (false empty). |
| Pager subgroups use **`--lib`** | Bare package list fails on pre-existing `tests/settings_e2e.rs` syntax error (out of Phase 6 scope). Lib tests hold all greened `p6_*` dispatch/unit proofs. |
| Shell integration subgroups use **`--test model_switch_gate`** | Session harness for apply / dual-login / history / mid-turn. |
| Pure routing | `cargo test -p xai-grok-shell --test provider_routing switch_changes_next_sample_route` (no `p6_` required). |

No product renames required this gate; filter args only.

---

## Copy-paste aggregate sequence

```bash
set -euo pipefail

discover() {
  local pkg="$1" filter="$2"; shift 2
  local extra=("$@")
  local n
  n=$(cargo test -p "$pkg" "${extra[@]}" -- --list 2>/dev/null | grep -c "$filter" || true)
  test "$n" -ge 1
  cargo test -p "$pkg" "${extra[@]}" "$filter" -- --nocapture
}

# --- Shell (model_switch_gate + unit + pure routing) ---
discover xai-grok-shell p6_missing_provider --test model_switch_gate
discover xai-grok-shell p6_dual_login --test model_switch_gate
discover xai-grok-shell p6_same_provider --test model_switch_gate
discover xai-grok-shell p6_byok --test model_switch_gate
discover xai-grok-shell p6_history --test model_switch_gate
discover xai-grok-shell p6_mid_turn --test model_switch_gate
discover xai-grok-shell p6_model_switch_missing_provider --lib
cargo test -p xai-grok-shell --test provider_routing switch_changes_next_sample_route -- --nocapture

# --- Pager (lib unit/dispatch) ---
discover xai-grok-pager p6_missing_provider --lib
discover xai-grok-pager p6_transactional_default --lib
discover xai-grok-pager p6_keep_current --lib
discover xai-grok-pager p6_login_now --lib
discover xai-grok-pager p6_deferred --lib
discover xai-grok-pager p6_auth_ --lib
discover xai-grok-pager p6_external_cli --lib
discover xai-grok-pager p6_focus_gained --lib
discover xai-grok-pager p6_refresh_generation --lib
discover xai-grok-pager p6_needs_login --lib
discover xai-grok-pager p6_provider_auth --lib
discover xai-grok-pager p6_refresh --lib
discover xai-grok-pager incompatible_agent --lib
```

---

## Gate results (executed Plan 06 Task 2)

**Status:** GREEN  
**Started:** `2026-07-16T23:54:10Z`  
**Passed:** `2026-07-17T00:17:46Z`  
**Host:** local cargo (debug test profile)

### Shell

| Subgroup | Target | Discovered ≥1 | Result |
|----------|--------|---------------|--------|
| `p6_missing_provider` | `--test model_switch_gate` | 3 | pass (3 ok) |
| `p6_dual_login` | `--test model_switch_gate` | 3 | pass (3 ok) |
| `p6_same_provider` | `--test model_switch_gate` | 1 | pass (1 ok) |
| `p6_byok` | `--test model_switch_gate` | 3 | pass (3 ok) |
| `p6_history` | `--test model_switch_gate` | 1 | pass (1 ok) |
| `p6_mid_turn` | `--test model_switch_gate` | 1 | pass (1 ok) |
| `p6_model_switch_missing_provider` | `--lib` | 2 | pass (2 ok) |
| `switch_changes_next_sample_route` | `--test provider_routing` | n/a (named) | pass (1 ok) |

### Pager

| Subgroup | Target | Discovered ≥1 | Result |
|----------|--------|---------------|--------|
| `p6_missing_provider` | `--lib` | 5 | pass (5 ok) |
| `p6_transactional_default` | `--lib` | 4 | pass (4 ok) |
| `p6_keep_current` | `--lib` | 2 | pass (2 ok) |
| `p6_login_now` | `--lib` | 6 | pass (6 ok) |
| `p6_deferred` | `--lib` | 1 | pass (1 ok) |
| `p6_auth_` | `--lib` | 3 | pass (3 ok) |
| `p6_external_cli` | `--lib` | 1 | pass (1 ok) |
| `p6_focus_gained` | `--lib` | 1 | pass (1 ok) |
| `p6_refresh_generation` | `--lib` | 1 | pass (1 ok) |
| `p6_needs_login` | `--lib` | 6 | pass (6 ok) |
| `p6_provider_auth` | `--lib` | 7 | pass (7 ok) |
| `p6_refresh` | `--lib` | 2 | pass (2 ok) |
| `incompatible_agent` | `--lib` | 4 | pass (4 ok; D-07 non-collapse) |

### ROADMAP criteria coverage

| Criterion | Evidence |
|-----------|----------|
| 1. Mid-session switch; next turn uses new model | shell `p6_dual_login` + next-sample; pure `switch_changes_next_sample_route` |
| 2. Missing creds block + login prompt | shell `p6_missing_provider` + pager QuestionView / login / deferred / external CLI |
| 3. Dual-login free Grok↔GPT one session | shell `p6_dual_login` (+ same_provider / history / mid_turn) |

---

## Explicit exclusions

- Phase 7 subagent orchestration
- Phase 8 rebrand / quiet-fork product work
- Live OAuth browser smoke (optional manual only)
- Unfiltered shell `--lib` as aggregate gate
- Aggregate-only `p6_` crate-wide count without per-subgroup discover

## Deferred (out of gate scope)

| Item | Notes |
|------|-------|
| `xai-grok-shell` integration `signed_managed_config*` compile (`test_seam`) | Pre-existing; forces shell unit discovery via `--lib` |
| `xai-grok-pager` `tests/settings_e2e.rs` syntax error | Pre-existing; forces pager discovery via `--lib` |

---

## Re-run

From repo root: paste the aggregate sequence above. Expect every `discover` line to print cargo pass and exit 0. Update `gate_passed` timestamp on re-verification.
