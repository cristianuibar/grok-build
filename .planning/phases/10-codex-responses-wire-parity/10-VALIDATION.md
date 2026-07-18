---
phase: 10
slug: codex-responses-wire-parity
status: draft
nyquist_compliant: false
wave_0_complete: true
created: 2026-07-18
---

# Phase 10 — Validation Strategy

> Per-phase validation contract for Codex Responses wire parity and the two Phase 9 reliability blockers it closes.

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` crate-local unit and integration suites |
| **Config file** | Workspace `Cargo.toml`; no new framework setup |
| **Quick run command** | `cargo test -p xai-grok-sampling-types encrypted_content --lib` |
| **Full suite command** | Focused sampling-types, sampler, and shell commands specified in each PLAN.md |
| **Estimated runtime** | Under 60 seconds per focused command, subject to normal Rust incremental build state |

## Sampling Rate

- **After every task commit:** Run the task's focused `<automated>` command.
- **After every plan wave:** Run all focused suites affected by that wave.
- **Before `$gsd-verify-work`:** Run `cargo fmt --all -- --check` and the consolidated Phase 10 suite.
- **Max feedback latency:** one focused Rust test command; do not use watch mode.

## Per-Task Verification Map

The executor updates this table after the approved plans establish final task IDs. The required behavior layers are fixed now:

| Behavior | Requirements | Threat Ref | Automated Proof |
|----------|--------------|------------|-----------------|
| Profile disabled by default and cannot leak through sampler or shell literals | OPS-04 | T-10-01 | sampler-core plus shell production/test-fixture compile checks |
| Trusted request body preserves generic conversion while applying profile body fields and tools-only controls | OPS-04 | T-10-02, T-10-03 | sampling-types/sampler serialization tests |
| Delta-only completed response is non-empty; terminal text is not duplicated; multi-output order and incomplete behavior remain intact; genuine empty retry remains bounded | OPS-04 | T-10-04, T-10-05 | sampler stream unit tests plus `test_actor` request-counter controls |
| Cross-provider switch retains ordinary context but removes foreign encrypted payloads, including a late old-provider response | OPS-05 | T-10-06, T-10-07 | actor-owned provider/epoch unit tests plus deterministically held scripted-response capture |
| Trusted headers do not leak and session metadata remains stable | OPS-04 | T-10-08, T-10-09 | reconstruction matrix plus actual outbound header capture, collision stripping, and trusted-to-untrusted switch regression |
| Encrypted-content variants are classified only as HTTP 400 recovery cases | OPS-05 | T-10-10 | sampling-types predicate, shell classification, and end-to-end one-request/no-compaction recovery test |
| Live evidence is not fabricated, secret-bearing, or falsely marked PASS | OPS-04, OPS-05 | T-10-11, T-10-12, T-10-13 | redacted human checkpoint and validation-artifact inspection |

## Wave 0 Requirements

- [x] Existing Rust test infrastructure covers all Phase 10 requirements.
- [x] Each plan names concrete regression tests and a non-vacuous focused `cargo test` command.

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| GPT-5.6 daily-driver turn completes once after Codex login | OPS-04 | Requires real dual-login session and actual service response | Rebuild `bum`; select `gpt-5.6-sol`; send `hi`; then read/edit a file; retain only redacted evidence and confirm no retry. |
| Grok → GPT-5.6 session switch remains productive | OPS-05 | Requires real provider credentials and switched live history | In one real session use Grok, switch to `gpt-5.6-sol`, send `hi`, and confirm no encrypted-content 400 while ordinary context remains usable. |

## Validation Sign-Off

- [ ] All approved plan tasks have `<automated>` verification.
- [ ] Sampling continuity has no three consecutive tasks without automated proof.
- [x] Existing infrastructure covers all required test layers.
- [x] No watch-mode flags are permitted.
- [ ] `nyquist_compliant: true` is set only after execution evidence is recorded.

**Approval:** pending execution and redacted OPS-04/OPS-05 evidence
