---
phase: 01
slug: product-identity-isolated-home
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-07-16
updated: 2026-07-16
reviews_cycle: 1
---

# Phase 1 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Updated after Codex review cycle 1 replan (hermetic isolation, updater, bundle, full fixtures).

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` (crate-local); `serial_test`; `tempfile`; **one env scenario per isolated test binary** for OnceLock |
| **Config file** | none global — per-crate; `rust-toolchain.toml` (1.92.0) |
| **Quick run command** | `cargo test -p xai-grok-config --lib paths -- --nocapture` |
| **Full suite command** | `cargo test -p xai-grok-config -p xai-fast-worktree -p xai-grok-pager-render -p xai-grok-agent -p xai-grok-update --test test_install_internal -p xai-grok-pager-bin --test home_isolation -- --nocapture` plus `cargo build -p xai-grok-pager-bin --bin bum` and shell-inclusive `rg` gate from plan 01-05 |
| **Estimated runtime** | ~90–240 seconds (per-crate; avoid full workspace) |

---

## Sampling Rate

- **After every task commit:** Run the task's `<automated>` verify command (prefer `&&` chains — never `;` that masks failures)
- **After every plan wave:** Run the full suite command for crates touched in that wave
- **Before `/gsd-verify-work`:** Full suite must be green + hermetic isolation test green + `cargo build --bin bum` + shell-inclusive GROK_HOME gate
- **Max feedback latency:** 180–240 seconds preferred (per-crate)

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 01-01-01 | 01 | 1 | ID-03 | T-01-01 | Pure resolve + BUM_HOME only | unit | `cargo test -p xai-grok-config --lib paths` | ⚠️ update | ⬜ pending |
| 01-01-02 | 01 | 1 | ID-03 | T-01-01 | One env scenario per process | isolated bins | `cargo test -p xai-grok-pager --test grok_home_paths && --test grok_home_ignore_grok_home` | ❌ new ignore bin | ⬜ pending |
| 01-01-03 | 01 | 1 | ID-03 | T-01-03 | Labels ~/.bum / $BUM_HOME | unit | `cargo test -p xai-grok-pager-render util` | ⚠️ update | ⬜ pending |
| 01-02-01 | 02 | 2 | ID-03 | T-01-04 | Twin BUM_HOME + .bum | unit | `cargo test -p xai-fast-worktree --lib` | ⚠️ update | ⬜ pending |
| 01-02-02 | 02 | 2 | ID-03 | T-01-05 | Fallback .bum; leader bin/bum tests | unit / check | `cargo test -p xai-grok-shell leader && cargo check workspace/sandbox` | ⚠️ update | ⬜ pending |
| 01-02-03 | 02 | 2 | ID-01, ID-03 | T-01-12 | Updater managed bin/bum | integration | `cargo test -p xai-grok-update --test test_install_internal && --test test_concurrent_convergence` | ⚠️ update | ⬜ pending |
| 01-03-01 | 03 | 1 | ID-01 | T-01-06 | Binary artifact bum | build | `cargo build -p xai-grok-pager-bin --bin bum` | ❌ after rename | ⬜ pending |
| 01-03-02 | 03 | 1 | ID-01 | T-01-06 | Harness resolves bum | compile | `cargo check -p xai-grok-test-support` | ⚠️ update | ⬜ pending |
| 01-03-03 | 03 | 1 | ID-01 | T-01-06 | PTY CARGO_BIN_EXE_bum | compile | `cargo check -p xai-grok-pager-pty-harness` | ⚠️ update | ⬜ pending |
| 01-04-01 | 04 | 3 | ID-03 | T-01-07 | Sandbox + lock/log .bum | compile + rg | `cargo check -p xai-grok-test-support` + no join(".grok") in env/leader | ⚠️ update | ⬜ pending |
| 01-04-02 | 04 | 3 | ID-03 | T-01-07 | Full PTY product home | unit + rg | `cargo test -p xai-grok-pager-pty-harness env_for_pager` | ⚠️ update | ⬜ pending |
| 01-04-03 | 04 | 3 | ID-03 | T-01-07 | Shell inventory BUM_HOME | behavioral | `cargo test -p xai-grok-shell --test test_debug_logging` + leader | ⚠️ update | ⬜ pending |
| 01-05-01 | 05 | 4 | ID-03 | T-01-08/13/14 | No stock agents/roles; bundle SoT | unit | agent discovery + shell bundle + config | ⚠️ update | ⬜ pending |
| 01-05-02 | 05 | 4 | ID-03 | T-01-09 | Hermetic recursive isolation | integration | `cargo test -p xai-grok-pager-bin --test home_isolation` | ❌ Wave 0 | ⬜ pending |
| 01-05-03 | 05 | 4 | ID-01, ID-03 | T-01-11 | Shell-inclusive gate + build bum | build + rg | build bum + paths + home_isolation + shell rg | ⚠️/❌ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Existing infrastructure covers most requirements; create/update during plans (not a separate Wave 0 install):

- [ ] Pure `resolve_product_home` unit tests + config path flip — plan 01-01
- [ ] Second isolated binary `grok_home_ignore_grok_home` — plan 01-01
- [ ] Updater managed bin/bum tests — plan 01-02
- [ ] Full PTY + shell fixture inventory — plan 01-04
- [ ] New hermetic isolation test: `home_isolation.rs` — plan 01-05
- [ ] Bundle + roles/personas cutover tests — plan 01-05
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
- [ ] Wave 0 covers all MISSING references (home_isolation + ignore_grok binary created in plans)
- [ ] Verify commands use `&&` not `;`
- [ ] No watch-mode flags
- [ ] Feedback latency < 240s preferred
- [ ] `nyquist_compliant: true` set after validate-phase

**Approval:** pending
