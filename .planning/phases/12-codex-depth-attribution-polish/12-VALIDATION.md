---
phase: 12
slug: codex-depth-attribution-polish
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-07-21
---

# Phase 12 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `cargo test` 1.92.0 plus scoped `rg` assertions |
| **Config file** | `rust-toolchain.toml` |
| **Quick run command** | `cargo test -p xai-grok-pager --lib p12_ -- --nocapture` |
| **Full suite command** | Run all discovered `p12_` filters, existing apply-patch and trusted-originator filters, scoped README/guide/notice gates, and `cargo fmt --all -- --check` |
| **Estimated runtime** | ~180 seconds |

## Sampling Rate

- **After every task commit:** Run the changed crate's single `p12_` filter plus `cargo fmt --all -- --check`
- **After every plan wave:** Run all discovered `p12_` filters and existing apply-patch/originator filters
- **Before `$gsd-verify-work`:** Full focused suite and non-vacuous static gates must be green
- **Max feedback latency:** 180 seconds

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 12-01-01 | 01 | 1 | ID-02 / D-10 | T-12-01 | Shipped docs identify bum and allow provider/legal names only in classified contexts | unit + static | `cargo test -p xai-grok-pager --lib p12_embedded_docs_use_bum_product_identity -- --nocapture` | ❌ W0 | ⬜ pending |
| 12-01-02 | 01 | 1 | D-10 | T-12-01 | Embedded capability disclosure states non-stock identity and supported/deferred boundaries | unit | `cargo test -p xai-grok-pager --lib p12_capability_disclosure_is_embedded_and_complete -- --nocapture` | ❌ W0 | ⬜ pending |
| 12-02-01 | 02 | 1 | OPS-04 / D-10 | T-12-02 | Existing Codex patch tool remains attributed and its registered identity is unchanged | unit | `cargo test -p xai-grok-tools --lib apply_patch -- --nocapture` and `cargo test -p xai-grok-agent --lib p12_codex_toolset_identity -- --nocapture` | partial / ❌ W0 | ⬜ pending |
| 12-02-02 | 02 | 1 | Product honesty | T-12-01 | Trusted routes retain bum originator without leaking reserved identity to other routes | unit + integration | `cargo test -p xai-grok-shell --lib trusted_codex_reconstruct_enables_profile_and_metadata -- --nocapture` and `cargo test -p xai-grok-shell --test model_switch_gate trusted_codex_wire_headers_are_sent_and_stable -- --nocapture` | ✅ | ⬜ pending |
| 12-03-01 | 03 | 2 | Notices / deferred scope | T-12-02 | Notices remain non-vacuous and the phase adds no WS transport or stock identity header | static | scoped `rg` checks and phase-diff allowlist in `12-PHASE-GATE.md` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

## Wave 0 Requirements

- [ ] `xai-grok-pager/src/docs.rs::p12_embedded_docs_use_bum_product_identity` — scan every embedded guide/reference using a documented allowlist
- [ ] `xai-grok-pager/src/docs.rs::p12_capability_disclosure_is_embedded_and_complete` — assert stable matrix/disclaimer markers
- [ ] `xai-grok-agent/src/config.rs::p12_codex_toolset_identity` — lock `Codex:apply_patch` and default bum edit-tool separation
- [ ] `12-PHASE-GATE.md` — define non-vacuous discovery, static README/notice checks, diff allowlist, and no-live-UAT rule
- [ ] No framework installation is required

## Manual-Only Verifications

All phase behaviors have automated verification. No live OAuth credentials or provider network access are required because this phase does not change runtime or wire behavior.

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all missing references
- [ ] No watch-mode flags
- [ ] Feedback latency < 180s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
