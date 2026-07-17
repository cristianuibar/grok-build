---
phase: 9
slug: daily-driver-end-to-end-validation
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-07-17
---

# Phase 9 — Validation Strategy

> Hybrid daily-driver bar for **OPS-03..OPS-06**: automated green (prior `p6_`/`p7_`/`p8_`
> subset + thin fixture `p9_`) **and** signed live dual-login human UAT.
> GREEN ⇔ both halves. Never fixture-only for live OPS rows. No CI live OAuth secrets.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Cargo built-in `cargo test` (+ bash `discover()` for gates) |
| **Config file** | none global — per-crate; `rust-toolchain.toml` 1.92.0 |
| **Quick run command** | `cargo test -p xai-grok-shell --test model_switch_gate p6_dual_login -- --nocapture` |
| **Full suite command** | See `09-PHASE-GATE.md` (prior subset + thin `p9_` + residual + human sign-off) |
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
| Secrets | Never commit tokens / auth.json / secret transcripts |

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

## Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? | Status |
|--------|----------|-----------|-------------------|--------------|--------|
| OPS-03 | Productive xAI tool turn after login | **live UAT required** + auto residual | UAT §OPS-03; residual: routing/auth/`home_isolation` | UAT ❌ W0; auto ✅ prior | ⬜ pending |
| OPS-04 | Productive GPT-5.6 tool turn after Codex login | **live UAT required** + auto residual | UAT §OPS-04; residual: dual-slot + codex route | UAT ❌ W0; auto ✅ prior | ⬜ pending |
| OPS-05 | Same-session switch productive | **live UAT** + auto | `discover xai-grok-shell p6_dual_login --test model_switch_gate`; `switch_changes_next_sample_route` | auto ✅; live ❌ W0 | ⬜ pending |
| OPS-06 | Cross-provider spawn both dirs live | **live UAT** + auto | `p7_isolation` both dirs + `p7_eager` + `p7_spawn_missing_provider` | auto ✅; live ❌ W0 | ⬜ pending |
| ALL | Quiet fork still holds | auto residual | `p8_telemetry`, `p8_no_auto_update`, `p8_sentry`, `home_isolation` | ✅ prior | ⬜ pending |
| ALL | Thin phase discovery smoke | unit/integration | `p9_*` (green-only, fixture) | ❌ Wave 0 | ⬜ pending |

---

## Recommended automated re-run inventory

| Priority | Package | Target | Filter | Why |
|----------|---------|--------|--------|-----|
| P0 | `xai-grok-shell` | `--test model_switch_gate` | `p6_dual_login` | OPS-05 residual |
| P0 | `xai-grok-shell` | `--test model_switch_gate` | `p6_missing_provider` | Fail-closed residual |
| P0 | `xai-grok-shell` | `--test provider_routing` | `switch_changes_next_sample_route` | Route after switch |
| P0 | `xai-grok-shell` | `--lib` | `p7_isolation` | OPS-06 Authorization residual |
| P0 | `xai-grok-shell` | `--lib` | `p7_isolation_grok_parent_codex` | Dir 1 |
| P0 | `xai-grok-shell` | `--lib` | `p7_isolation_codex_parent_grok` | Dir 2 |
| P0 | `xai-grok-tools` | `--lib` | `p7_eager` | Spawn preflight residual |
| P0 | `xai-grok-shell` | `--lib` | `p7_spawn_missing_provider` | Fail-closed spawn |
| P0 | `xai-grok-shell` | `--lib` | `p7_parent_model` | Parent model stability |
| P1 | `xai-grok-shell` | `--test cross_provider_subagent` | `p7_spawn_same_provider` | AGENT-01 residual |
| P1 | `xai-grok-shell` | `--lib` | `p8_telemetry` | Quiet fork residual |
| P1 | `xai-grok-pager-bin` | `--bin bum` | `p8_no_auto_update` | No stock overwrite |
| P1 | `xai-grok-pager-bin` | `--bin bum` | `p8_sentry` | No phone-home |
| P1 | `xai-grok-pager-bin` | `--test home_isolation` | `hermetic` | Identity residual |
| P2 | shell/tools | `--lib` or targeted | `p9_*` | Thin discovery residual |

---

## Manual / Human Verifications (REQUIRED for gate GREEN)

| Behavior | Requirement | Why Manual | Required? | Instructions |
|----------|-------------|------------|-----------|--------------|
| Real xAI coding turn (read + edit/shell) | OPS-03 | Live backend + OAuth | **Required** | `09-UAT.md` §OPS-03 |
| Real Codex/GPT-5.6 coding turn | OPS-04 | Live backend + OAuth | **Required** | `09-UAT.md` §OPS-04 |
| Same-session provider switch | OPS-05 | Real multi-turn session | **Required** | `09-UAT.md` §OPS-05 |
| Cross-provider Task both dirs | OPS-06 | Live dual-login spawn | **Required** | `09-UAT.md` §OPS-06 |

**Do not** mark live OPS rows green on fixture-only evidence.  
**Do not** treat environment skip as phase pass.  
If network/account fails → block human path (fix or re-auth).

Default models: current xAI daily (e.g. `grok-build`) + one GPT-5.6 catalog entry (e.g. `gpt-5.6-sol`). Prefer TUI path.

---

## Wave 0 Requirements

- [ ] `09-VALIDATION.md` — this file (dual auto/human map; nyquist frontmatter)
- [ ] `09-PHASE-GATE.md` — discover+execute aggregate + human sign-off section
- [ ] `09-UAT.md` — required live dual-login checklist (not advisory)
- [ ] Thin `p9_*` green smoke (0–3 tests) — fixture dual residual / discovery
- [ ] Optional `scripts/uat-preflight.sh` — non-secret helper
- [ ] After execute: `09-VERIFICATION.md` with OPS rows + model IDs + operator sign-off

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Hybrid gate: automated green **and** signed UAT
- [ ] `nyquist_compliant: true` set in frontmatter when gate closes

**Approval:** pending
