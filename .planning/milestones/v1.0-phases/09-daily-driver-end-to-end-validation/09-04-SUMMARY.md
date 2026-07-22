---
phase: 09-daily-driver-end-to-end-validation
plan: 04
subsystem: validation
tags: [uat, live-oauth, dual-provider, subagents, codex-responses, ops-03, ops-04, ops-05, ops-06]

requires:
  - phase: 09-daily-driver-end-to-end-validation
    provides: automated residual gate and live UAT runbook (Plans 02–03)
  - phase: 10-codex-responses-wire-parity
    provides: live OPS-04/OPS-05 fixes and operator evidence promoted into Phase 9
provides:
  - Signed live OPS-03..06 matrix with both cross-provider spawn directions
  - Four in-phase Codex child/tool-stream blocker fixes required for OPS-06
  - Redacted operator evidence with models, dates, disposable workspace, and parent stability
affects:
  - 09-05 hybrid gate closure
  - Phase 7 deferred live cross-provider E2E

key-files:
  modified:
    - .planning/phases/09-daily-driver-end-to-end-validation/09-UAT.md
    - crates/codegen/xai-grok-sampler/src/client.rs
    - crates/codegen/xai-grok-sampler/src/stream/responses.rs
    - crates/codegen/xai-grok-shell/src/agent/config.rs
    - crates/codegen/xai-grok-shell/src/agent/subagent/mod.rs
    - crates/codegen/xai-grok-shell/src/agent/subagent/tests/mod.rs
    - crates/codegen/xai-grok-shell/src/session/acp_session_impl/sampler_turn.rs

key-decisions:
  - "Only live operator evidence counts for OPS-03..06; fixtures remain automated residuals"
  - "Unknown Responses event types are skipped only when the serde failure is an unknown variant; malformed known events still fail"
  - "A subagent's Responses wire profile derives from the child's provider, auth type, and endpoint rather than the parent session"
  - "The Codex-parent grok-build visibility defect is deferred because the operator-approved grok-4.5 child still satisfied the cross-provider outcome"

requirements-completed: [OPS-03, OPS-04, OPS-05, OPS-06]

coverage:
  - id: D1
    description: "Live xAI and Codex productive sessions plus same-process provider switch"
    requirement: OPS-03
    verification: []
    human_judgment: true
    rationale: "Operator PASS recorded in 09-UAT.md for OPS-03..05 on 2026-07-18"
  - id: D2
    description: "Live Grok-parent/Codex-child and Codex-parent/Grok-child tasks both return results with parent models unchanged"
    requirement: OPS-06
    verification: []
    human_judgment: true
    rationale: "Operator PASS recorded in 09-UAT.md for both directions on 2026-07-20"
  - id: D3
    description: "Codex subagent and tool-turn blocker fixes retain regression coverage"
    requirement: OPS-06
    verification:
      - kind: unit
        ref: "xai-grok-sampler lib tests and xai-grok-shell subagent wire-profile tests from commit 4fcd52b"
        status: pass
    human_judgment: false

duration: multi-session live UAT
completed: 2026-07-20
status: complete
---

# Phase 9 Plan 04: Live Dual-Provider UAT Summary

**OPS-03 through OPS-06 passed on live xAI and Codex backends, including cross-provider child tasks in both directions after four scoped Codex wire/stream fixes.**

## Accomplishments

- Confirmed productive xAI and GPT-5.6 sessions, including read/edit/tool work.
- Confirmed the same bum process switches from Grok to GPT-5.6 and continues productively.
- Confirmed Grok parent → Codex child and Codex parent → Grok child both return results while the parent model remains unchanged.
- Preserved redacted evidence only; no credentials, tokens, or auth documents entered planning artifacts.
- Accepted residual Grok help chrome as cosmetic and documented Codex capability gaps without weakening the daily-driver bar.

## In-Phase Blockers Fixed

The first Grok-parent → Codex-child attempt exposed product blockers. Commit `4fcd52b` fixed them:

1. Tolerantly skip genuinely unknown Responses SSE event variants such as `response.metadata`.
2. Derive a child session's `ResponsesWireProfile` from the child's resolved provider/auth/endpoint.
3. Align Trusted Codex requests by removing rejected sampler parameters and adding function-tool strictness plus prompt cache identity.
4. Reconstruct empty terminal Responses output from accumulated `response.output_item.done` events.

The operator then repeated the failed direction and recorded PASS. The closure/evidence rollup was committed in `646b001`; earlier live promotions are in `5b68602`.

## Deferred, Non-Blocking

- A Codex parent can hide session-auth-only xAI slug `grok-build` from the Task allow-list; the approved `grok-4.5` child completed successfully.
- Requested-versus-effective child slug surfacing and long-running child bearer refresh remain follow-up work.

## Outcome

All four OPS rows are live PASS under the required taxonomy. Plan 05 may combine this signed human half with the automated residual half.
