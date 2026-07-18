---
phase: 10-codex-responses-wire-parity
plan: "04"
subsystem: shell-sampler-routing
tags: [rust, codex, responses-api, oauth, headers, error-recovery, regression-tests]
requires:
  - phase: 10-codex-responses-wire-parity
    provides: "Responses wire-profile types from Plan 01, provider-transition safety from Plan 03, and trusted-route configuration from Plans 06 and 07."
provides:
  - "First-party Codex SessionToken OAuth activates the trusted Responses profile and bum-owned stable request identity headers."
  - "Reserved Codex identity headers cannot leak from inherited configuration to xAI, BYOK, or custom routes."
  - "One shared HTTP-400 encrypted-content predicate drives sampler retry classification and shell terminal recovery before compaction."
affects: [10-05-validation, OPS-04, OPS-05, trusted-codex-routing]
tech-stack:
  added: []
  patterns:
    - "Trusted outbound metadata is constructed at the shell reconstruction boundary, after case-insensitive inherited-header removal."
    - "Error-info recovery calls a sampling-types predicate instead of duplicating text matching."
    - "Strict recovery integration tests count every inference POST after auxiliary title traffic is drained."
key-files:
  created: []
  modified:
    - crates/codegen/xai-grok-shell/src/agent/config.rs
    - crates/codegen/xai-grok-shell/src/session/acp_session_impl/sampler_turn.rs
    - crates/codegen/xai-grok-shell/src/session/acp_session_tests/codex_reconstruct_refresh_tests.rs
    - crates/codegen/xai-grok-shell/src/session/acp_session_tests/auth_error_no_retry_tests.rs
    - crates/codegen/xai-grok-shell/tests/model_switch_gate.rs
    - crates/codegen/xai-grok-shell/tests/provider_routing.rs
    - crates/codegen/xai-grok-sampling-types/src/error.rs
key-decisions:
  - "Codex identity/profile activation requires the existing first-party Codex SessionToken OAuth gate; sampler code does not guess URLs."
  - "The effective port is part of first-party Codex authority, preventing same-host/different-port custom endpoints from gaining OAuth metadata."
  - "Recognized encrypted-content HTTP 400s are terminal before compaction so incompatible history is never resubmitted."
requirements-completed: [OPS-04, OPS-05]
coverage:
  - id: D1
    description: "Trusted first-party Codex OAuth requests receive the trusted Responses profile plus stable bum-owned identity headers, while xAI, BYOK, and custom routes receive none."
    requirement: OPS-04
    verification:
      - kind: unit
        ref: "session::acp_session::codex_reconstruct_refresh_tests::trusted_codex_reconstruct_enables_profile_and_metadata"
        status: pass
      - kind: integration
        ref: "model_switch_gate::{trusted_codex_wire_headers_are_sent_and_stable,codex_wire_headers_do_not_leak_to_xai_byok_or_custom,trusted_to_untrusted_switch_strips_codex_identity_headers}"
        status: pass
    human_judgment: false
  - id: D2
    description: "Both encrypted-content spellings/casing variants are terminal only for API HTTP 400 and cannot compact or resubmit the turn."
    requirement: OPS-05
    verification:
      - kind: unit
        ref: "error::tests::encrypted_content_error_accepts_spaced_case_insensitive_400_only"
        status: pass
      - kind: unit
        ref: "session::acp_session::auth_error_no_retry_tests::encrypted_content_400_is_classified_terminal"
        status: pass
      - kind: integration
        ref: "model_switch_gate::encrypted_content_400_is_terminal_before_compaction_or_resubmit"
        status: pass
    human_judgment: false
  - id: D3
    description: "A real GPT-5.6 turn and real Grok-to-GPT-5.6 session switch remain productive after login."
    requirement: OPS-04
    verification: []
    human_judgment: true
    rationale: "The Phase 10 validation contract requires a redacted live dual-login rerun; fixture coverage cannot establish daily-driver service behavior."
duration: ~3h
completed: 2026-07-18
status: complete
---

# Phase 10 Plan 04: Trusted Codex Activation and Bounded Recovery Summary

**Trusted Codex OAuth now owns bum's Responses profile and stable request identity, while encrypted-history mismatch 400s fail once without compaction or resubmission.**

## Accomplishments

- Activated `ResponsesWireProfile::TrustedCodex` only when a Codex route is both first-party and backed by SessionToken OAuth; `originator: bum` plus `session-id`, `thread-id`, and `x-client-request-id` all use the actor's stable session UUID.
- Removed all reserved Codex identity headers case-insensitively after inherited/URL headers are assembled; trusted reconstruction reinserts only canonical bum-owned values and preserves the existing availability-gated account-ID behavior.
- Hardened first-party Codex authority to include scheme and effective port, so a same-host/different-port custom endpoint cannot inherit trusted metadata.
- Centralized encrypted-content mismatch recognition in sampling types: case-insensitive underscore and spaced variants match only API HTTP 400, and shell recovery checks the shared predicate before auto-compaction.
- Added real outbound-route and sampler-turn regressions, including a strict one-inference-request proof for a low-context-window encrypted-content 400.

## Task Commits

1. **Task 1: Enable the trusted profile and request metadata only during first-party Codex OAuth reconstruction** — `aa81bf4` (feat)
2. **Task 2: Centralize encrypted-content mismatch recognition and route shell recovery through it** — `9f42f42` (fix)

**Plan metadata:** pending this summary commit.

## Verification

- `cargo test -p xai-grok-shell --lib session::acp_session::codex_reconstruct_refresh_tests::trusted_codex_reconstruct_enables_profile_and_metadata -- --exact` — passed.
- `cargo test -p xai-grok-shell --test model_switch_gate trusted_codex_wire_headers_are_sent_and_stable -- --exact` — passed.
- `cargo test -p xai-grok-shell --test model_switch_gate codex_wire_headers_do_not_leak_to_xai_byok_or_custom -- --exact` — passed.
- `cargo test -p xai-grok-shell --test model_switch_gate trusted_to_untrusted_switch_strips_codex_identity_headers -- --exact` — passed.
- `cargo test -p xai-grok-shell --test provider_routing codex_first_party_requires_matching_effective_port -- --exact` — passed.
- `cargo test -p xai-grok-sampling-types --lib error::tests::encrypted_content_error_accepts_spaced_case_insensitive_400_only -- --exact` — passed.
- `cargo test -p xai-grok-shell --lib session::acp_session::auth_error_no_retry_tests::encrypted_content_400_is_classified_terminal -- --exact` — passed.
- `cargo test -p xai-grok-shell --test model_switch_gate encrypted_content_400_is_terminal_before_compaction_or_resubmit -- --exact` — passed (31.68s).
- `cargo test -p xai-grok-sampling-types --lib error::tests -- --nocapture` — **24 passed, 0 failed**.
- `cargo test -p xai-grok-sampler --lib retry::tests::classify_encrypted_content_emits_to_session -- --exact` — passed.
- `git diff --check` — passed before each task commit.

## Decisions Made

- Treat URL scheme, host, effective port, and path as the trusted Codex authority. The explicit configured endpoint may still be trusted, but a loopback/custom port cannot piggyback on a first-party route.
- Keep the shared encrypted-content predicate in `xai-grok-sampling-types`; `SamplingError` delegates to it and the shell's `SamplingErrorInfo` path calls the same predicate with its status/message representation.
- Preserve a strict total inference-request assertion for the encrypted-400 recovery test. Auxiliary title generation is drained before the baseline rather than filtered out afterward.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Security] Effective port was missing from first-party Codex authority**

- **Found during:** Task 1 trusted-route test design.
- **Issue:** A same-host, different-port endpoint could match the configured Codex host/path and receive trusted OAuth metadata.
- **Fix:** Required matching scheme and effective port for configured endpoints; builtin ChatGPT hosts require HTTPS/effective port 443.
- **Files modified:** `crates/codegen/xai-grok-shell/src/agent/config.rs`, `crates/codegen/xai-grok-shell/tests/provider_routing.rs`.
- **Verification:** `codex_first_party_requires_matching_effective_port` passed.
- **Committed in:** `aa81bf4`.

**2. [Rule 1 - Test correctness] Distinguish primary agent turns from auxiliary title inference**

- **Found during:** Task 1 outbound-wire regression execution.
- **Issue:** A normal initial prompt emits an auxiliary one-tool title request in addition to the primary agent request, so selecting every POST would assert against the wrong route/headers.
- **Fix:** Task 1 header tests select primary requests (two or more tools). Task 2 explicitly retains an unfiltered one-request assertion after seeding and draining title traffic.
- **Files modified:** `crates/codegen/xai-grok-shell/tests/model_switch_gate.rs`.
- **Verification:** All four exact model-switch gate regressions passed.
- **Committed in:** `aa81bf4`, `9f42f42`.

**3. [Rule 1 - Fixture correctness] Give the custom endpoint its own API-key fixture**

- **Found during:** Task 1 custom-route wire verification.
- **Issue:** The custom endpoint must actually receive a request to prove header isolation, while the fail-closed SessionToken denial intentionally prevents its use there.
- **Fix:** Kept a fixture-only custom API key for its outbound isolation check; session-token denial remains covered at reconstruction/routing boundaries.
- **Files modified:** `crates/codegen/xai-grok-shell/tests/model_switch_gate.rs`.
- **Verification:** `codex_wire_headers_do_not_leak_to_xai_byok_or_custom` passed.
- **Committed in:** `aa81bf4`.

---

**Total deviations:** 3 auto-fixed (1 security hardening, 2 test-fixture corrections).
**Impact on plan:** All changes preserve the locked trust boundary and make the planned assertions non-vacuous; no product-scope expansion was introduced.

## Review and Issues Encountered

- A source-aware Grok review was run with web search available and without plan-only mode. It reached its turn cap and returned `Cancelled`, so it was treated as unavailable. The required no-tools inline-corpus fallback completed normally. Its apparent method-coverage and sampler-retry concerns were checked against source: the method assertion is present and `xai-grok-sampler/src/retry.rs` already uses `SamplingError::is_encrypted_content_error`; no review finding required a code change.
- `cargo fmt --all -- --check` is red on broad pre-existing formatting drift in unrelated crates (for example fast-worktree, config, pager, tools, and update). No formatter mutation or unrelated reformatting was performed.
- A broader `auth_error_no_retry_tests` module run had 19 passing and 2 failing existing bearer-resolver fixture assertions. They are unrelated to this plan: both call `reconstruct_full_config` only; Task 2 runs after a sampler error, and Task 1 changes the Codex authority gate only. The fixture uses XAI model `test` at `http://localhost`, which leaves its `Unknown` BYOK/non-first-party XAI gate inactive. The exact new terminal-classification test passed.

## User Setup Required

None for the automated implementation. A redacted live OPS-04/OPS-05 dual-login validation remains required by the phase validation contract before claiming daily-driver readiness.

## Next Phase Readiness

- Plan 10-05 can use trusted-route/header isolation and bounded encrypted-400 recovery as automated evidence.
- Do not mark OPS-04 or OPS-05 live-complete until the documented real Codex turn and Grok-to-Codex switch checks are rerun with redacted evidence.

## Self-Check: PASSED

- Trusted profile/identity activation remains shell-owned and fail-closed.
- Reserved identity headers are absent on xAI, BYOK, custom, and trusted-to-untrusted transition routes.
- The encrypted-content recovery test counts every post-baseline inference request and proves exactly one failed primary turn.

---
*Phase: 10-codex-responses-wire-parity*
*Completed: 2026-07-18*
