---
phase: 8
slug: quiet-fork-rebrand-polish
plan: 06
status: green
gate_started: 2026-07-17T11:41:06Z
gate_passed: 2026-07-17T11:52:03Z
requirements: [ID-02, OPS-01, OPS-02]
fixture_only: true
green_only: true
---

# Phase 8 — Phase Gate Runbook

Automated proof for **ID-02**, **OPS-01**, **OPS-02** (product chrome as bum; stock
auto-update hard-off; product telemetry / feedback / OTLP / Sentry phone-home off by
default).

- **Fixtures only** — no live x.ai update channel, Mixpanel, Sentry, or OTLP network.
- **Green-only** — all required `p8_` subgroups pass; **no intentional-red** carve-out under
  `p8_` (Plan 01 shipped compile-safe green scaffolds only; Plans 02–05 greened product).
- **Per-subgroup discovery ≥1** before each execute (T-08-12 / Phase 6–7 lesson). Aggregate
  `grep -c p8_` alone is **not** sufficient.
- **Unconditional `p8_sentry`** (C1-L1) and **`p8_internal_otel`** (C1-H2) — discover ≥1 required.
- **Non-vacuous residual greps** (C1-H1) on owned runtime CLI surfaces.
- **D-02 model brand lock** — `dynamic_enum_model_names` still green for `Grok Build (xAI)`.
- **Chains use `&&` only** (C1-L3) — never `;` / `|| true` masking failures on execute paths.
- **Forbidden:** unfiltered full-workspace test; live Phase 9 dual-provider E2E; public x.ai
  install channel; crate rename; mass `GROK_*` rename.

Canonical map: `08-VALIDATION.md`.

---

## Discover + execute helper

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
```

### Target selection notes

| Case | Why |
|------|-----|
| Pager / shell / update / telemetry / minimal unit use **`--lib`** | Narrow unit surface |
| Pager-bin unit tests live on **`--bin bum`** | Composition-root tests module |
| Isolation uses **`--test home_isolation`** | Integration target |
| Model brand uses **`--test dynamic_enum_model_names`** | Integration regression (D-02) |

---

## Pre-gate design checks (docs)

```bash
# VALIDATION claims green for all three requirements + OTLP/sentry/residual
rg -n "p8_internal_otel|p8_sentry|runtime_cli|remote|C1-H1|nyquist_compliant: true" \
  .planning/phases/08-quiet-fork-rebrand-polish/08-VALIDATION.md

# Plan 05 landed remote restrictive + OTLP + sentry
rg -n "p8_sentry|p8_internal_otel|restrictive-only|remote_true" \
  .planning/phases/08-quiet-fork-rebrand-polish/08-05-SUMMARY.md
```

---

## Copy-paste aggregate sequence

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

# ========== ID-02 — pager product chrome ==========
discover xai-grok-pager p8_cli_brand --lib
discover xai-grok-pager p8_welcome --lib
discover xai-grok-pager p8_runtime_cli --lib
discover xai-grok-pager p8_feedback --lib
discover xai-grok-pager p8_settings_auto_update --lib
# Aggregate after per-subgroup (optional breadth; still green-only)
discover xai-grok-pager p8_ --lib

# ========== ID-02 — shell OAuth + residual runtime CLI ==========
discover xai-grok-shell p8_oauth_return --lib
discover xai-grok-shell p8_shell_runtime_cli --lib

# ========== ID-02 — minimal + bin banners ==========
discover xai-grok-pager-minimal p8_minimal_welcome --lib
discover xai-grok-pager-bin p8_bin_ --bin bum

# ========== OPS-01 — auto-update hard-off ==========
discover xai-grok-pager-bin p8_no_auto_update --bin bum
discover xai-grok-pager-bin p8_update_cmd --bin bum
discover xai-grok-pager-bin p8_update_no_network --bin bum
discover xai-grok-update p8_auto_update --lib
discover xai-grok-update p8_min_version --lib

# ========== OPS-02 — telemetry / feedback / OTLP / Sentry ==========
discover xai-grok-shell p8_telemetry --lib
discover xai-grok-shell p8_feedback --lib
discover xai-grok-telemetry p8_internal_otel --lib   # C1-H2 hard bar
discover xai-grok-shell p8_internal_otel --lib
discover xai-grok-pager-bin p8_internal_otel --bin bum
discover xai-grok-pager-bin p8_sentry --bin bum      # C1-L1 unconditional

# ========== D-02 model brand lock + home isolation ==========
echo "discover dynamic_enum_model_names"
n=$(cargo test -p xai-grok-pager --test dynamic_enum_model_names -- --list 2>/dev/null | grep -c 'dynamic_enum' || true)
echo "discover xai-grok-pager dynamic_enum_model_names: n=$n"
test "$n" -ge 1
cargo test -p xai-grok-pager --test dynamic_enum_model_names -- --nocapture

echo "discover home_isolation"
n=$(cargo test -p xai-grok-pager-bin --test home_isolation -- --list 2>/dev/null | grep -c 'hermetic\|home' || true)
echo "discover xai-grok-pager-bin home_isolation: n=$n"
test "$n" -ge 1
cargo test -p xai-grok-pager-bin --test home_isolation -- --nocapture

# ========== C1-H1 residual inventory greps (non-vacuous; exit 1 if stock chrome remains) ==========
# Patterns are exact stock recovery / product-chrome strings that rebrand must remove.
# Allowlist (do not fail): model Grok Build (xAI), SuperGrok, xai-grok-*, GrokAuth, grok.com, GROK_*, agent prompts.
STOCK_PAT='Run `grok login`|Run `grok mcp|Run `grok plugin|Grok agent server starting|Grok crashed during|grok \(pager\) -|Open Grok Build|Grok Build TUI|Thanks for using Grok'
RESIDUAL_FILES=(
  crates/codegen/xai-grok-shell/src/auth/error.rs
  crates/codegen/xai-grok-shell/src/auth/device_code.rs
  crates/codegen/xai-grok-shell/src/mcp_doctor.rs
  crates/codegen/xai-grok-shell/src/plugin.rs
  crates/codegen/xai-grok-pager/src/plugin_cmd.rs
  crates/codegen/xai-grok-pager/src/headless.rs
  crates/codegen/xai-grok-pager-bin/src/main.rs
)
for f in "${RESIDUAL_FILES[@]}"; do
  test -f "$f"
  if rg -n "$STOCK_PAT" "$f"; then
    echo "RESIDUAL FAIL: stock product CLI chrome still present in $f"
    exit 1
  fi
  echo "residual clean: $f"
done

# Positive presence: recovery copy uses bum (non-vacuous — greps are not empty filters)
rg -n 'bum login|bum plugin|bum mcp|return to bum|bum \(pager\)|bum agent server|bum crashed' \
  crates/codegen/xai-grok-shell/src/auth/error.rs \
  crates/codegen/xai-grok-shell/src/auth/device_code.rs \
  crates/codegen/xai-grok-shell/src/mcp_doctor.rs \
  crates/codegen/xai-grok-shell/src/plugin.rs \
  crates/codegen/xai-grok-pager/src/plugin_cmd.rs \
  crates/codegen/xai-grok-pager/src/headless.rs \
  crates/codegen/xai-grok-pager-bin/src/main.rs \
  | head -40

# Static non-change proofs (D-03 / D-13) — document only; no product rename required
test "$(rg -l 'name = "xai-grok-' crates/codegen/*/Cargo.toml 2>/dev/null | wc -l)" -ge 1
echo "D-03: xai-grok-* crate names retained (leave-internal)"
echo "D-13: GROK_* env family left internal (no mass rename in Phase 8)"
echo "Deferred: public x.ai install channel; Phase 9 live E2E; agent system prompts"

echo "PHASE 8 GATE: ALL SUBGROUPS GREEN"
```

---

## Checklist (must all pass before gate GREEN)

| # | Check | Status |
|---|-------|--------|
| 1 | Pager `p8_cli_brand` discover+execute | ✅ n=3 |
| 2 | Pager `p8_welcome` discover+execute | ✅ n=4 |
| 3 | Pager `p8_runtime_cli` discover+execute | ✅ n=2 |
| 4 | Pager `p8_feedback` discover+execute | ✅ n=4 |
| 5 | Pager `p8_settings_auto_update` discover+execute | ✅ n=1 |
| 6 | Shell `p8_oauth_return` + `p8_shell_runtime_cli` | ✅ n=1 / n=4 |
| 7 | Minimal `p8_minimal_welcome` + bin `p8_bin_` | ✅ n=1 / n=3 |
| 8 | OPS-01: `p8_no_auto_update` + `p8_update_cmd` + `p8_update_no_network` | ✅ n=1 / 1 / 3 |
| 9 | OPS-01: update `p8_auto_update` + `p8_min_version` | ✅ n=3 / n=1 |
| 10 | OPS-02: shell `p8_telemetry` + `p8_feedback` | ✅ n=3 / n=5 |
| 11 | OPS-02: `p8_internal_otel` (telemetry + shell + bin) | ✅ n=1 each |
| 12 | OPS-02: `p8_sentry` unconditional discover ≥1 + execute | ✅ n=1 |
| 13 | D-02 `dynamic_enum_model_names` green | ✅ n=2 |
| 14 | `home_isolation` green | ✅ n=1 |
| 15 | C1-H1 residual greps clean on all 7 owned surfaces | ✅ |
| 16 | Green-only — no intentional-red exception | ✅ |
| 17 | Deferred scope excluded (public channel, crate rename, Phase 9 E2E) | ✅ documented |

---

## Gate results (filled by Plan 06 Task 2)

**Status:** GREEN  
**Started:** `2026-07-17T11:41:06Z`  
**Passed:** `2026-07-17T11:52:03Z`  
**Host:** local cargo (debug test profile)  
**Protocol:** per-subgroup discover ≥1 then execute; `&&` chains only; no intentional-red; no live x.ai network

### Per-subgroup results

| Subgroup | Package | Target | Discovered ≥1 | Result |
|----------|---------|--------|---------------|--------|
| `p8_cli_brand` | pager | `--lib` | 3 | pass (3 ok) |
| `p8_welcome` | pager | `--lib` | 4 | pass (4 ok) |
| `p8_runtime_cli` | pager | `--lib` | 2 | pass (2 ok) |
| `p8_feedback` | pager | `--lib` | 4 | pass (4 ok) |
| `p8_settings_auto_update` | pager | `--lib` | 1 | pass (1 ok) |
| `p8_` (aggregate) | pager | `--lib` | 17 | pass (17 ok) |
| `p8_oauth_return` | shell | `--lib` | 1 | pass (1 ok) |
| `p8_shell_runtime_cli` | shell | `--lib` | 4 | pass (4 ok) |
| `p8_minimal_welcome` | pager-minimal | `--lib` | 1 | pass (1 ok) |
| `p8_bin_` | pager-bin | `--bin bum` | 3 | pass (3 ok) |
| `p8_no_auto_update` | pager-bin | `--bin bum` | 1 | pass (1 ok) |
| `p8_update_cmd` | pager-bin | `--bin bum` | 1 | pass (1 ok) |
| `p8_update_no_network` | pager-bin | `--bin bum` | 3 | pass (3 ok) |
| `p8_auto_update` | update | `--lib` | 3 | pass (3 ok) |
| `p8_min_version` | update | `--lib` | 1 | pass (1 ok) |
| `p8_telemetry` | shell | `--lib` | 3 | pass (3 ok) |
| `p8_feedback` | shell | `--lib` | 5 | pass (5 ok) |
| `p8_internal_otel` | telemetry | `--lib` | 1 | pass (1 ok) **C1-H2** |
| `p8_internal_otel` | shell | `--lib` | 1 | pass (1 ok) |
| `p8_internal_otel` | pager-bin | `--bin bum` | 1 | pass (1 ok) |
| `p8_sentry` | pager-bin | `--bin bum` | 1 | pass (1 ok) **C1-L1 unconditional** |
| `dynamic_enum_model_names` | pager | integration | 2 | pass (2 ok) **D-02** |
| `home_isolation` | pager-bin | integration | 1 | pass (1 ok) |
| residual greps (C1-H1) | 7 surfaces | static | n/a | pass (all clean) |

### Residual inventory results (C1-H1)

| Surface | Stock chrome zero | bum recovery present |
|---------|-------------------|----------------------|
| `shell/src/auth/error.rs` | ✅ | ✅ `bum login` |
| `shell/src/auth/device_code.rs` | ✅ | ✅ `bum login` |
| `shell/src/mcp_doctor.rs` | ✅ | ✅ `bum login` / `bum mcp` |
| `shell/src/plugin.rs` | ✅ | ✅ `bum plugin` |
| `pager/src/plugin_cmd.rs` | ✅ | ✅ `bum plugin` |
| `pager/src/headless.rs` | ✅ | ✅ `bum login` |
| `pager-bin/src/main.rs` | ✅ | ✅ `bum (pager)` / agent server / crashed |

---

## Residual allowlist (must not fail greps)

| Pattern | Rationale |
|---------|-----------|
| Model label `Grok Build (xAI)` / id `grok-build` | D-02 model brand |
| SuperGrok commercial SKU/name | Billing product, not CLI identity |
| Crate/package `xai-grok-*` | D-03 internal names |
| Types `GrokAuth`, headers `x-grok-client-*` | Internal API surface |
| Agent system prompt “Grok Build agent” | Deferred beyond chrome gate |
| Managed host `grok.com` | Host identity, not CLI binary name |
| Dev env knobs `GROK_*` | D-13 leave-internal |

## Explicit deferred exclusions

- Public signed x.ai install channel for bum
- Internal monorepo crate rename (`xai-grok-*` → bum crates)
- Mass `GROK_*` → `BUM_*` env rename
- Live dual-provider daily-driver E2E (Phase 9)
