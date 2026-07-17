---
phase: 7
slug: cross-provider-multi-agent-orchestration
plan: 06
status: green
gate_started: 2026-07-17T09:29:19Z
gate_passed: 2026-07-17T09:29:56Z
requirements: [AGENT-01, AGENT-02, AGENT-03, AGENT-04, AGENT-05, AGENT-06]
fixture_only: true
green_only: true
---

# Phase 7 — Phase Gate Runbook

Automated proof for **AGENT-01..06** (same-provider regression, cross-provider spawn,
effort wire, dual-direction Authorization isolation, async fail-closed preflight, NL Task
schema surface).

- **Fixtures only** — no live ChatGPT / xAI OAuth tokens or secret env vars required.
- **Green-only** — all required `p7_` subgroups pass; **no intentional-red** carve-out under
  `p7_` (Plan 01 shipped compile-safe green scaffolds only).
- **Per-subgroup discovery ≥1** before each execute (T-07-11 / Phase 6 lesson). Aggregate
  `grep -c p7_` alone is **not** sufficient.
- **AGENT-05 hard bar (C2-M5):** tools `p7_eager` + shell `p7_preflight` (async
  `SubagentBackend::preflight_spawn` / coordinator live effective-model path — **not**
  sync slug-only Fn). Plan 04 SUMMARY must document async design.
- **D-12 hard bar:** both `p7_isolation_grok_parent_codex` and
  `p7_isolation_codex_parent_grok` are **child-sample Authorization** proofs via Plan 05
  minimal harness (`p7_isolation_spawn_sample_cancel`) — not resolve-only.
- **Forbidden:** unfiltered `cargo test -p xai-grok-shell --lib` as sole gate; live multi-turn
  dual-login NL E2E (deferred Phase 9 / OPS-06); workflow engine / cost dashboard / Phase 8
  rebrand (D-16).

Canonical helper is shared with `07-VALIDATION.md`.

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

### Target selection notes

| Case | Why |
|------|-----|
| Tools / agent / shell unit use **`--lib`** | Narrow unit surface; bare package `-- --list` may compile unrelated integration targets |
| Isolation / missing / parent_model / preflight on **`--lib`** | Plan 05 in-crate seam + Plan 04 coordinator preflight live in `subagent/tests` |
| Spawn missing / smoke / same-provider also on **`--test cross_provider_subagent`** | Integration harness dual-documents; prefer lib when both exist for speed |
| Prefer `--lib` for shell unit when bare list is poisoned | Phase 6 lesson |

No product renames required this gate; filter args only.

---

## Pre-gate design checks (docs — fail AGENT-05 / D-12 if broken)

```bash
# Plan 04 async design (not sync slug-only)
rg -n "async SubagentBackend|preflight_spawn|NOT sync slug-only|not a sync slug" \
  .planning/phases/07-cross-provider-multi-agent-orchestration/07-04-SUMMARY.md

# Plan 05 minimal harness is child-sample Authorization (not resolve-only)
rg -n "p7_isolation_spawn_sample_cancel|Authorization|not resolve-only|MockInferenceServer" \
  .planning/phases/07-cross-provider-multi-agent-orchestration/07-05-SUMMARY.md
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

# ========== Tools (AGENT-03 / AGENT-05 / AGENT-06) ==========
discover xai-grok-tools p7_task_tool_input_schema --lib
discover xai-grok-tools p7_reasoning_effort --lib
discover xai-grok-tools p7_eager --lib   # AGENT-05 C2-M5 hard bar

# ========== Agent (AGENT-06 NL guidance) ==========
discover xai-grok-agent p7_ --lib

# ========== Shell integration (spawn gate + smoke + tool) ==========
discover xai-grok-shell p7_wave0_harness_smoke --test cross_provider_subagent
discover xai-grok-shell p7_spawn_missing_provider --test cross_provider_subagent
discover xai-grok-shell p7_tool --test cross_provider_subagent
discover xai-grok-shell p7_spawn_same_provider --test cross_provider_subagent

# ========== Shell lib — isolation both dirs (D-12 / AGENT-04) ==========
# Direction-name presence (must each discover ≥1)
test "$(cargo test -p xai-grok-shell --lib -- --list 2>/dev/null | grep -c 'p7_isolation_grok_parent_codex' || true)" -ge 1
test "$(cargo test -p xai-grok-shell --lib -- --list 2>/dev/null | grep -c 'p7_isolation_codex_parent_grok' || true)" -ge 1
discover xai-grok-shell p7_isolation --lib
discover xai-grok-shell p7_isolation_grok_parent_codex --lib
discover xai-grok-shell p7_isolation_codex_parent_grok --lib

# ========== Shell lib — missing / parent / tool (C3-M2 execute, not rg-only) ==========
discover xai-grok-shell p7_missing --lib
discover xai-grok-shell p7_parent_model --lib
discover xai-grok-shell p7_tool --lib

# ========== Shell lib — async preflight (AGENT-05 C2-M5) ==========
n_pre=$(cargo test -p xai-grok-shell --lib -- --list 2>/dev/null | grep -c 'p7_preflight' || true)
n_gate=$(cargo test -p xai-grok-shell --lib -- --list 2>/dev/null | grep -c 'p7_credential_gate' || true)
test "$((n_pre + n_gate))" -ge 1
if [ "$n_pre" -ge 1 ]; then discover xai-grok-shell p7_preflight --lib; fi
if [ "$n_gate" -ge 1 ]; then discover xai-grok-shell p7_credential_gate --lib; fi

# ========== Shell lib — effort fail-closed + same-provider spawn ==========
discover xai-grok-shell p7_invalid_effort --lib
discover xai-grok-shell p7_spawn_same_provider --lib
discover xai-grok-shell p7_spawn_missing_provider --lib

# ========== AGENT-01 hard regression (D-14 / D-15) ==========
discover xai-grok-shell reasoning_effort_explicit --lib
discover xai-grok-shell resume_model_pinning --lib
discover xai-grok-shell role_default_used --lib
discover xai-grok-shell persona_resolved --lib
# Cited lifecycle (C2-L2) — spawn→completion metadata + resume pin already above
discover xai-grok-shell upload_lifecycle_spawn_then_completion --lib
```

---

## Checklist (must all pass before gate GREEN)

| # | Check | Status |
|---|-------|--------|
| 1 | Tools `p7_task_tool_input_schema` discover+execute | ✅ |
| 2 | Tools `p7_reasoning_effort` discover+execute | ✅ |
| 3 | Tools `p7_eager` discover+execute (**AGENT-05**) | ✅ |
| 4 | Agent `p7_` effort guidance discover+execute | ✅ |
| 5 | Integration `p7_spawn_missing_provider` + smoke + tool + same_provider | ✅ |
| 6 | Both isolation direction **names** present + execute green | ✅ |
| 7 | `p7_missing` + `p7_parent_model` + `p7_tool` **executed** (C3-M2) | ✅ |
| 8 | Shell `p7_preflight` and/or `p7_credential_gate` executed (**AGENT-05**) | ✅ (`p7_preflight` n=8) |
| 9 | AGENT-01: effort / resume / roles / personas green | ✅ |
| 10 | Lifecycle cited/executed (`upload_lifecycle_spawn_then_completion` + same-provider spawn) | ✅ |
| 11 | Plan 04 async design documented (not slug-only-only) | ✅ |
| 12 | Plan 05 Authorization both dirs are child-sample proofs (not resolve-only) | ✅ |
| 13 | Green-only language — no intentional-red exception | ✅ |
| 14 | Live multi-turn dual-login NL E2E **not** required (Phase 9) | ✅ documented |
| 15 | No unfiltered `cargo test -p xai-grok-shell --lib` as sole gate | ✅ |

---

## Gate results (filled by Plan 06 Task 2)

**Status:** GREEN  
**Started:** `2026-07-17T09:29:19Z`  
**Passed:** `2026-07-17T09:29:56Z`  
**Host:** local cargo (debug test profile)  
**Design checks:** Plan 04 async `preflight_spawn` (NOT sync slug-only) ✅ · Plan 05 minimal harness Authorization both dirs (not resolve-only) ✅

### Tools

| Subgroup | Target | Discovered ≥1 | Result |
|----------|--------|---------------|--------|
| `p7_task_tool_input_schema` | `--lib` | 1 | pass (1 ok) |
| `p7_reasoning_effort` | `--lib` | 2 | pass (2 ok) |
| `p7_eager` | `--lib` | 7 | pass (7 ok) **AGENT-05** |

### Agent

| Subgroup | Target | Discovered ≥1 | Result |
|----------|--------|---------------|--------|
| `p7_` | `--lib` | 1 | pass (1 ok) |

### Shell integration (`cross_provider_subagent`)

| Subgroup | Target | Discovered ≥1 | Result |
|----------|--------|---------------|--------|
| `p7_wave0_harness_smoke` | integration | 1 | pass (1 ok) |
| `p7_spawn_missing_provider` | integration | 4 | pass (4 ok) |
| `p7_tool` | integration | 1 | pass (1 ok) |
| `p7_spawn_same_provider` | integration | 1 | pass (1 ok) |

### Shell lib

| Subgroup | Target | Discovered ≥1 | Result |
|----------|--------|---------------|--------|
| `p7_isolation` | `--lib` | 4 | pass (4 ok) |
| `p7_isolation_grok_parent_codex` | `--lib` | 1 | pass (1 ok) **D-12** |
| `p7_isolation_codex_parent_grok` | `--lib` | 1 | pass (1 ok) **D-12** |
| `p7_missing` | `--lib` | 3 | pass (3 ok) **C3-M2** |
| `p7_parent_model` | `--lib` | 1 | pass (1 ok) **C3-M2** |
| `p7_tool` | `--lib` | 1 | pass (1 ok) **C3-M2** |
| `p7_preflight` | `--lib` | 8 | pass (8 ok) **AGENT-05** (`p7_credential_gate` n=0; preflight covers gate) |
| `p7_invalid_effort` | `--lib` | 4 | pass (4 ok) |
| `p7_spawn_same_provider` | `--lib` | 1 | pass (1 ok) |
| `p7_spawn_missing_provider` | `--lib` | 5 | pass (5 ok) |
| `reasoning_effort_explicit` | `--lib` | 1 | pass (1 ok) **AGENT-01** |
| `resume_model_pinning` | `--lib` | 1 | pass (1 ok) **AGENT-01/D-15** |
| `role_default_used` | `--lib` | 1 | pass (1 ok) **AGENT-01** |
| `persona_resolved` | `--lib` | 1 | pass (1 ok) **AGENT-01** |
| `upload_lifecycle_spawn_then_completion` | `--lib` | 1 | pass (1 ok) **C2-L2 cited lifecycle** |

---

## Explicit exclusions (D-16 / Phase 9)

- Live multi-turn dual-login NL E2E → **Phase 9 (OPS-06)** — not this gate
- Workflow engine, cost dashboards, TUI spawn modal
- Phase 8 rebrand polish
- Live OAuth secrets as CI requirement
- Intentional-red under `p7_` as completion path
- Resolve-only key_prefix as sole D-12 proof
- Sync slug-only credential Fn as AGENT-05 solution
