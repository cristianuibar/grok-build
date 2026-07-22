---
phase: 11
slug: codex-effort-catalog-fidelity
# status lifecycle: draft (seeded by plan-phase) → validated (set by validate-phase §6)
# audit-milestone §5.5 distinguishes NOT-VALIDATED (draft) from PARTIAL (validated + nyquist_compliant: false) (#2117)
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-07-21
---

# Phase 11 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust workspace, edition 2024) |
| **Config file** | Cargo.toml (workspace) — no Wave 0 install needed |
| **Quick run command** | `cargo test -p xai-grok-sampling-types --lib` |
| **Full suite command** | `cargo test -p xai-grok-sampling-types --lib && cargo test -p xai-grok-sampler --lib && cargo test -p xai-grok-shell --test model_switch_gate && cargo test -p xai-grok-pager --lib` |
| **Estimated runtime** | quick ~30s; full ~5–10 min |

---

## Sampling Rate

- **After every task commit:** Run the quick command plus the crate suite the task touched
- **After every plan wave:** Run the full suite command
- **Before `/gsd-verify-work`:** Full suite must be green (pager `--lib` baseline 7142 pass / 0 fail must not regress)
- **Max feedback latency:** ~600 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| (filled by planner) | | | MOD-01/MOD-02/OPS-04 | — | catalog-driven clamp never widens levels beyond advertised list | unit/wire/behavior | see Test Infrastructure | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements — no Wave 0 stubs needed (seams: `xai-grok-sampling-types` unit tests, `xai-grok-sampler` wire serialization tests, `xai-grok-shell` `model_switch_gate`, `xai-grok-pager` dispatch tests).

---

## Manual-Only Verifications

- Optional live effort-menu spot-check under real dual login (NOT a phase gate — CONTEXT decision; automated wire fixtures are the required evidence)
