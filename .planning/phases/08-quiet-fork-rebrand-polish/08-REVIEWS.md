---
phase: 8
reviewers: [codex]
reviewed_at: 2026-07-17T10:21:16Z
plans_reviewed: ['08-01-PLAN.md', '08-02-PLAN.md', '08-03-PLAN.md', '08-04-PLAN.md', '08-05-PLAN.md', '08-06-PLAN.md']
cycle: 1
current_high: 3
current_actionable: 5
verdict: REPLAN
---

# Cross-AI Plan Review — Phase 8

**Phase:** 08-quiet-fork-rebrand-polish  
**Reviewers:** Codex only (`--codex`)  
**Cycle:** 1  
**Plans:** 08-01-PLAN.md, 08-02-PLAN.md, 08-03-PLAN.md, 08-04-PLAN.md, 08-05-PLAN.md, 08-06-PLAN.md

---

## Codex review (cycle 1)

### HIGH

- [C1-H1] 08-02/08-03/08-06: ID-02 inventory is materially incomplete. Reachable strings still instruct users to run `grok` or identify the product as Grok, including `auth/error.rs`, `auth/device_code.rs`, `mcp_doctor.rs`, `plugin_cmd.rs`, `headless.rs`, and several `pager-bin/main.rs` messages such as “Grok agent server” and “Grok crashed.” The final inventory only covers UI-SPEC checklist files. Fix: add a runtime-string inventory task covering all reachable CLI/help/error copy, with explicit model/internal exclusions and a non-vacuous residual gate.

- [C1-H2] 08-05/08-06: OPS-02 misses the always-on internal OTLP trace pipeline. `instrumentation.rs` defaults to `InstrumentationMode::Server`; `build_default_otel_layer_config()` enables export unless telemetry is explicitly disabled; absence is deliberately not explicit disablement; the endpoint defaults to `https://cli-chat-proxy.grok.com/v1/traces`. Thus `TelemetryMode::Disabled` plus Sentry/Mixpanel changes do not stop default trace phone-home. Fix: add an OPS-02 task that disables the internal OTLP exporter by fork default in both TUI and non-TUI paths, with `p8_internal_otel_off_by_default` coverage.

- [C1-H3] 08-05: remote settings can silently re-enable phone-home without local consent. `resolve_telemetry_mode()` accepts remote `telemetry_mode`/`telemetry_enabled`, while `resolve_feedback()` still applies remote `feedback_enabled`; the latter enables feedback-manager config fetches, analytics, and prompts. Telemetry hardening is discretionary and feedback remote hardening is absent. Fix: make remote policy restrictive-only for both telemetry and feedback, preserve only explicit local/admin opt-in if desired, and add remote-true/default-local-unset regression tests.

### MEDIUM

- [C1-M1] 08-05: `x.ai/debug/trigger_feedback` explicitly bypasses enabled checks, and `force_feedback_request()` can record against the feedback API even when feedback is disabled. Fix: gate this extension behind explicit developer-only configuration or ensure quiet-fork sessions have no network feedback client; add a disabled-mode test.

- [C1-M2] 08-04/08-06: the no-network update proofs are too structural. The independent `finish_update_on_exit()` Ctrl+U path currently falls back to `run_update_if_available()`, while proposed tests may only assert a message/helper and source order. Fix: add an injected call counter or hermetic process test proving startup, explicit update, min-version, leader, and quit-for-update paths never invoke stock update helpers.

### LOW

- [C1-L1] 08-06: `p8_sentry` is listed as “if present,” although 08-05 mandates it. Fix: make it an unconditional discovered subgroup in the phase gate.

- [C1-L2] 08-PLAN-CHECK: the recorded blocker is stale. `08-03-PLAN.md` now has a correct frontmatter fence, and `gsd-tools query verify.plan-structure` reports all six plans valid. Fix: rerun and replace PLAN-CHECK before convergence/execution.

- [C1-L3] 08-01/08-04/08-05/08-06: several automated commands chain with `;`, contradicting VALIDATION’s locked “`&&` only” protocol. `set -e` makes most safe, but the plans do not conform to their own gate contract. Fix: normalize multi-command verifies to `&&`.

### Verdict

REPLAN — default OTLP export, remotely re-enabled telemetry/feedback, and substantial residual Grok CLI copy prevent all three criteria from being assured.

### Summary

- OPS-01 design covers the known `None → true`, first-run persistence, min-version, leader, explicit-update, and Ctrl+U seams, but needs stronger no-network proof.
- OPS-02 does not cover the independent internal OTLP exporter.
- Remote settings remain a silent telemetry/feedback activation path.
- ID-02 only rebrands selected chrome; many reachable `grok` instructions remain.
- Source-grounding found no material symbol/path hallucinations and confirmed all plan frontmatter is currently valid.

CYCLE_SUMMARY: current_high=3 current_actionable=5

## Current HIGH Concerns

- ID-02 leaves numerous reachable Grok/`grok` user-facing strings outside plan ownership.
- Internal OTLP traces still export to the stock xAI proxy by default.
- Remote settings can silently enable telemetry and feedback phone-home.

## Current Actionable Non-HIGH Concerns

- Gate or disable the debug feedback bypass.
- Add hermetic no-network coverage for every update path, including Ctrl+U.
- Make `p8_sentry` unconditional in Plan 06.
- Refresh the stale PLAN-CHECK artifact.
- Replace semicolon command chains with the locked `&&` protocol.
