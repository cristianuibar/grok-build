---
phase: 10-codex-responses-wire-parity
plan: "03"
subsystem: shell-session-history
tags: [rust, provider-routing, responses-api, encrypted-reasoning, regression-tests]
requires:
  - phase: 10-codex-responses-wire-parity
    provides: "Responses API conversion and shell fixture compatibility from Plans 01, 02, 06, and 07."
provides:
  - "Provider-scoped encrypted reasoning sanitation that preserves ordinary and non-encrypted conversation history."
  - "Actor-owned provider/epoch tracking plus a late-response insertion barrier."
  - "A deterministic scripted Responses SSE terminal gate and outbound request-capture regressions."
affects: [10-04-trusted-shell-activation, 10-05-validation, OPS-05]
tech-stack:
  added: []
  patterns:
    - "Cross-provider history cleanup is field-level: remove only reasoning.encrypted_content."
    - "Mid-turn history replacement is deferred until late response items are enqueued."
    - "Primary-agent-only scripted fixtures prevent auxiliary model requests from consuming a FIFO scenario."
key-files:
  created: []
  modified:
    - crates/codegen/xai-grok-shell/src/session/acp_session.rs
    - crates/codegen/xai-grok-shell/src/session/acp_session_impl/model_switch.rs
    - crates/codegen/xai-grok-shell/src/session/acp_session_impl/turn.rs
    - crates/codegen/xai-grok-shell/tests/model_switch_gate.rs
    - crates/codegen/xai-grok-test-support/src/mock_server.rs
    - crates/codegen/xai-grok-test-support/src/scripted.rs
key-decisions:
  - "Unknown prior provider remains conservative and clears encrypted reasoning, as locked by the plan."
  - "Known same-provider Codex continuity retains encrypted reasoning even across a model switch."
  - "Do not use broad compaction strip_reasoning_blocks for provider transitions."
requirements-completed: [OPS-05]
coverage:
  - id: D1
    description: "Pure sanitizer removes only encrypted reasoning while retaining user, assistant, tool, reasoning id, summary, and non-encrypted content."
    requirement: OPS-05
    verification:
      kind: unit
      ref: session::acp_session::model_switch::tests::{cross_provider_transition_sanitizes_existing_and_late_reasoning,same_provider_codex_transition_preserves_encrypted_reasoning,unknown_prior_provider_sanitizes_encrypted_reasoning}
      status: pass
    human_judgment: false
  - id: D2
    description: "A held Grok Responses turn released after a Codex switch serializes normal history and visible reasoning but omits encrypted_content in the next Codex request."
    requirement: OPS-05
    verification:
      kind: integration
      ref: held_grok_response_after_codex_switch_is_sanitized
      status: pass
    human_judgment: false
  - id: D3
    description: "A Codex-to-Codex switch retains encrypted reasoning continuity in the next Responses request."
    requirement: OPS-05
    verification:
      kind: integration
      ref: codex_to_codex_transition_retains_encrypted_reasoning
      status: pass
    human_judgment: false
duration: ~2h
completed: 2026-07-18
status: complete
---

# Phase 10 Plan 03: Provider Reasoning Transition Safety Summary

**Provider transitions now strip only foreign encrypted reasoning payloads, preserving useful session history and same-provider Codex continuity.**

## Accomplishments

- Added actor-owned `ProviderTransition` state (active provider plus epoch), captured immediately before each sampler request and rechecked before every response item is persisted.
- Added a narrow local sanitizer for `ConversationItem::Reasoning.encrypted_content`; ordinary user, assistant, tool, reasoning id, summary, and visible reasoning content remain intact.
- Deferred cross-provider persisted-history replacement while a sampler turn is active, then drains it after late items are safely queued and before any next sampler request.
- Added a documented test-only scripted SSE terminal gate, plus primary-agent-only scripted response targeting so title/classifier traffic cannot consume the fixture.
- Added real outbound Responses request-capture coverage for held Grok → Codex safety and Codex → Codex encrypted continuity.

## Task Commits

1. **Task 1: Guard encrypted reasoning on provider switches** — `19c1d76` (feat)
2. **Task 1 follow-up: Defer mid-turn history replacement safely** — `75ba23f` (fix)
3. **Task 2: Capture provider reasoning transition safety** — `3a3390f` (test)

## Verification

- `cargo check -p xai-grok-test-support` — passed.
- `cargo check -p xai-grok-shell --lib` — passed.
- All three exact `session::acp_session::model_switch::tests` sanitizer tests — passed.
- `cargo test -p xai-grok-shell --test model_switch_gate held_grok_response_after_codex_switch_is_sanitized -- --exact` — passed (31.71s).
- `cargo test -p xai-grok-shell --test model_switch_gate codex_to_codex_transition_retains_encrypted_reasoning -- --exact` — passed (31.69s).
- `cargo test -p xai-grok-test-support mock_server::tests::agent_turn_script_waits_for_a_request_with_tools -- --exact` — passed.
- `cargo test -p xai-grok-shell --test model_switch_gate` — **19 passed, 0 failed** (401.01s).
- File-local `rustfmt --edition 2024 --check` passed for the changed test-support files and the newly changed session implementation/constructor paths.
- `git diff --check` passed before each task commit.

## Review and Deviations

Grok CLI review was run with web search enabled and without plan-only mode. It identified a concrete `get_conversation`/`replace_conversation` race: a model switch could otherwise replace history from a stale snapshot while an old-provider response was being inserted. The follow-up commit defers that replacement to a safe turn boundary; late response items are still sanitized before insertion. This is a Rule-1 correctness fix within the planned production files.

The first held-response fixture run also showed that a FIFO scripted `/v1/responses` reply could be consumed by an auxiliary request. `ScriptedResponse::for_agent_turn()` is a minimal test-support extension that leaves the script queued until a request carries at least two tools. Its unit test and the integration fixture prove the held reply belongs to the primary conversation turn; the ordinary assistant-history assertion was retained.

Direct `SessionActor` test constructors received the new actor-state default only, under the explicitly approved constructor-fallout allowance. No production file outside the plan scope changed.

## Issues Encountered

`cargo fmt --all -- --check` remains red on broad pre-existing workspace formatting drift in unrelated crates (for example fast-worktree, config, pager, tools, and update), and in pre-existing regions of large touched files. No formatter mutation or unrelated reformatting was performed. Focused formatter checks and `git diff --check` cover this patch.

## Next Phase Readiness

- Plan 10-04 can activate its trusted Codex shell path knowing that a provider transition will not replay foreign encrypted reasoning into a Codex request.
- Plan 10-05 can cite the pure sanitizer, held cross-provider request capture, and same-provider control as automated OPS-05 evidence.

## Self-Check: PASSED

- The provider sanitizer is local to model switching and does not call `strip_reasoning_blocks`.
- All task commits are present and scoped to planned files plus approved test-constructor fallout.
- No secret, URL heuristic, broad chat-state API, or placeholder behavior was introduced.

---
*Phase: 10-codex-responses-wire-parity*
*Completed: 2026-07-18*
