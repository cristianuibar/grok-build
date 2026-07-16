---
phase: 01
slug: product-identity-isolated-home
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-07-16
---

# Phase 1 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` (crate-local); `serial_test`; `tempfile`; isolated test binaries for OnceLock |
| **Config file** | none global — per-crate; `rust-toolchain.toml` (1.92.0) |
| **Quick run command** | `cargo test -p xai-grok-config --lib paths -- --nocapture` |
| **Full suite command** | `cargo test -p xai-grok-config -p xai-fast-worktree -p xai-grok-pager-render -p xai-grok-agent -p xai-grok-pager-bin --test home_isolation -- --nocapture` plus `cargo build -p xai-grok-pager-bin --bin bum` |
| **Estimated runtime** | ~60–180 seconds (per-crate; avoid full workspace) |

---

## Sampling Rate

- **After every task commit:** Run the task's `<automated>` verify command
- **After every plan wave:** Run the full suite command for crates touched in that wave
- **Before `/gsd-verify-work`:** Full suite must be green + isolation test green + `cargo build --bin bum`
- **Max feedback latency:** 180 seconds preferred (per-crate)

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 01-01-01 | 01 | 1 | ID-03 | T-01-01 | BUM_HOME only; no GROK_HOME honor | unit | `cargo test -p xai-grok-config --lib paths` | ⚠️ update | ⬜ pending |
| 01-01-02 | 01 | 1 | ID-03 | T-01-01 | Ignore GROK_HOME; BUM_HOME override | unit / isolated bin | `cargo test -p xai-grok-pager --test grok_home_paths` | ⚠️ update | ⬜ pending |
| 01-01-03 | 01 | 1 | ID-03 | T-01-03 | Labels ~/.bum / $BUM_HOME | unit | `cargo test -p xai-grok-pager-render util` | ⚠️ update | ⬜ pending |
| 01-02-01 | 02 | 2 | ID-03 | T-01-04 | Twin BUM_HOME + .bum | unit | `cargo test -p xai-fast-worktree --lib` | ⚠️ update | ⬜ pending |
| 01-02-02 | 02 | 2 | ID-03 | T-01-05 | Fallback .bum; managed bin bum | unit / check | `cargo test -p xai-grok-shell leader` + check workspace/sandbox | ⚠️ update | ⬜ pending |
| 01-03-01 | 03 | 1 | ID-01 | T-01-06 | Binary artifact bum | build | `cargo build -p xai-grok-pager-bin --bin bum` | ❌ after rename | ⬜ pending |
| 01-03-02 | 03 | 1 | ID-01 | T-01-06 | Harness resolves bum | compile | `cargo check -p xai-grok-test-support` | ⚠️ update | ⬜ pending |
| 01-03-03 | 03 | 1 | ID-01 | T-01-06 | PTY CARGO_BIN_EXE_bum | compile | `cargo check -p xai-grok-pager-pty-harness` | ⚠️ update | ⬜ pending |
| 01-04-01 | 04 | 3 | ID-03 | T-01-07 | Sandbox BUM_HOME | compile | `cargo check -p xai-grok-test-support` | ⚠️ update | ⬜ pending |
| 01-04-02 | 04 | 3 | ID-03 | T-01-07 | env_for_pager BUM_HOME | unit | `cargo test -p xai-grok-pager-pty-harness env_for_pager` | ⚠️ update | ⬜ pending |
| 01-04-03 | 04 | 3 | ID-03 | T-01-07 | Fixtures use BUM_HOME | compile | `cargo check -p xai-grok-update -p xai-grok-shell` | ⚠️ update | ⬜ pending |
| 01-05-01 | 05 | 4 | ID-03 | T-01-08 | No stock ~/.grok agent scan | unit | `cargo test -p xai-grok-agent discovery` | ⚠️ update | ⬜ pending |
| 01-05-02 | 05 | 4 | ID-03 | T-01-09 | Zero writes under .grok/.codex | integration | `cargo test -p xai-grok-pager-bin --test home_isolation` | ❌ Wave 0 | ⬜ pending |
| 01-05-03 | 05 | 4 | ID-01, ID-03 | T-01-01/09 | Grep gate + build bum | build + unit + integration | build bum + paths + home_isolation | ⚠️/❌ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Existing infrastructure covers most requirements; create/update during plans (not a separate Wave 0 install):

- [ ] Update `xai-grok-config` path unit tests (`.bum`, `BUM_HOME`) — plan 01-01
- [ ] Update `crates/codegen/xai-grok-pager/tests/grok_home_paths.rs` for `BUM_HOME` + GROK_HOME-ignored — plan 01-01
- [ ] New isolation test: `crates/codegen/xai-grok-pager-bin/tests/home_isolation.rs` — plan 01-05
- [ ] Update twin fixture + display + harness env — plans 01-02, 01-03, 01-04
- [ ] No new test framework install required

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Interactive TUI smoke under BUM_HOME | ID-01, ID-03 | Full TUI PTY optional; automated headless/isolation covers write isolation | Optional: `BUM_HOME=/tmp/bum-test cargo run -p xai-grok-pager-bin --bin bum` then quit; confirm files only under `/tmp/bum-test` |

*Primary phase gate is automated.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references (home_isolation created in plan 05)
- [ ] No watch-mode flags
- [ ] Feedback latency < 180s preferred
- [ ] `nyquist_compliant: true` set after validate-phase

**Approval:** pending
