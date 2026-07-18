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
  - "Actor-serialized encrypted-reasoning cleanup and system-head rewrites with no stale whole-history replacement."
  - "A deterministic scripted Responses SSE terminal gate and outbound request-capture regressions."
affects: [10-04-trusted-shell-activation, 10-05-validation, OPS-05]
tech-stack:
  added: []
  patterns:
    - "Cross-provider history cleanup is field-level: remove only reasoning.encrypted_content."
    - "Cross-provider cleanup runs through the chat-state actor; the late-response barrier sanitizes old-provider items inserted after cleanup."
    - "Competing model-switch prompt rewrites use the actor's replace_system_head command rather than caller-side whole-history RMW."
    - "Primary-agent-only scripted fixtures prevent auxiliary model requests from consuming a FIFO scenario."
key-files:
  created: []
  modified:
    - crates/codegen/xai-grok-shell/src/session/acp_session.rs
    - crates/codegen/xai-grok-shell/src/session/acp_session_impl/model_switch.rs
    - crates/codegen/xai-grok-shell/src/session/acp_session_impl/turn.rs
    - crates/codegen/xai-grok-shell/tests/model_switch_gate.rs
    - crates/codegen/xai-chat-state/src/commands.rs
    - crates/codegen/xai-chat-state/src/handle.rs
    - crates/codegen/xai-chat-state/src/actor/mod.rs
    - crates/codegen/xai-chat-state/src/actor/mutations.rs
    - crates/codegen/xai-chat-state/src/actor/tests.rs
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
  - id: D4
    description: "The actor-serialized encrypted-payload clear preserves non-encrypted reasoning fields while serializing with a system-head replacement."
    requirement: OPS-05
    verification:
      kind: unit
      ref: actor::tests::clear_encrypted_reasoning_serializes_with_system_head_replacement
      status: pass
    human_judgment: false
  - id: D5
    description: "A held XAI response survives an XAI-to-Codex switch plus a nearby Codex-to-Codex switch; the next Codex request retains ordinary and visible reasoning history but omits the foreign encrypted payload."
    requirement: OPS-05
    verification:
      kind: integration
      ref: held_grok_response_after_two_switches_is_sanitized
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
- Added the minimal actor-serialized `ClearEncryptedReasoning` command and switched model prompt replacement to `replace_system_head`, removing caller-side whole-history read-modify-write races.
- Removed the temporary deferred-cleanup state: cleanup now runs immediately in the chat-state actor, while the late-response insertion barrier clears an old-provider item if it arrives afterward.
- Added a documented test-only scripted SSE terminal gate, plus primary-agent-only scripted response targeting so title/classifier traffic cannot consume the fixture.
- Added real outbound Responses request-capture coverage for held Grok → Codex safety, a held XAI → Codex → Codex two-switch race, and Codex → Codex encrypted continuity.

## Task Commits

1. **Task 1: Guard encrypted reasoning on provider switches** — `19c1d76` (feat)
2. **Task 1 follow-up: Defer mid-turn history replacement safely** — `75ba23f` (fix)
3. **Task 2: Capture provider reasoning transition safety** — `3a3390f` (test)
4. **Post-review correctness follow-up: Atomically clear provider reasoning** — `656ee49` (fix)

## Verification

- `cargo check -p xai-grok-test-support` — passed.
- `cargo check -p xai-grok-shell --lib` — passed.
- `cargo check -p xai-chat-state` — passed.
- All three exact `session::acp_session::model_switch::tests` sanitizer tests — passed.
- `cargo test -p xai-chat-state --lib` — **338 passed, 0 failed**.
- `cargo test -p xai-chat-state --lib actor::tests::clear_encrypted_reasoning_serializes_with_system_head_replacement -- --exact` — passed.
- `cargo test -p xai-grok-shell --test model_switch_gate held_grok_response_after_codex_switch_is_sanitized -- --exact` — passed (31.71s).
- `cargo test -p xai-grok-shell --test model_switch_gate codex_to_codex_transition_retains_encrypted_reasoning -- --exact` — passed (31.69s).
- `cargo test -p xai-grok-shell --test model_switch_gate held_grok_response_after_two_switches_is_sanitized -- --exact` — passed (31.80s).
- `cargo test -p xai-grok-test-support mock_server::tests::agent_turn_script_waits_for_a_request_with_tools -- --exact` — passed.
- `timeout 900s cargo test -p xai-grok-shell --test model_switch_gate` — **20 passed, 0 failed** (432.22s).
- File-local `rustfmt --edition 2024 --check` passed for the newly added chat-state and model-switch implementation code; the two-switch gate itself is formatter-clean.
- `git diff --check` passed before each task commit.

## Review and Deviations

Grok CLI review was run with web search enabled and without plan-only mode. It identified the initial concrete `get_conversation`/`replace_conversation` race: a model switch could otherwise replace history from a stale snapshot while an old-provider response was being inserted. The first follow-up deferred that replacement, but independent review found the remaining race with a nearby second model switch's caller-side prompt rewrite. The authorized Rule-1 correction adds only a field-specific actor command in `xai-chat-state`, calls it from shell sanitation, and uses the existing actor-owned `replace_system_head` command for prompt rewrites. The temporary pending-cleanup state was removed.

The first held-response fixture run also showed that a FIFO scripted `/v1/responses` reply could be consumed by an auxiliary request. `ScriptedResponse::for_agent_turn()` is a minimal test-support extension that leaves the script queued until a request carries at least two tools. Its unit test and the integration fixture prove the held reply belongs to the primary conversation turn; the ordinary assistant-history assertion was retained.

The scope expansion is limited to the five exact `xai-chat-state` command/handle/actor files necessary for the field-specific mutation and its unit test. Direct `SessionActor` test constructors removed the obsolete deferred-cleanup default under the explicitly approved constructor-fallout allowance. No broad generic mutation API or unrelated crate changed.

## Issues Encountered

`cargo fmt --all -- --check` remains red on broad pre-existing workspace formatting drift in unrelated crates (for example fast-worktree, config, pager, tools, and update), and in pre-existing regions of large touched files. No formatter mutation or unrelated reformatting was performed. Focused formatter checks and `git diff --check` cover this patch.

## Next Phase Readiness

- Plan 10-04 can activate its trusted Codex shell path knowing that a provider transition, even beside a second model switch, will not replay foreign encrypted reasoning into a Codex request.
- Plan 10-05 can cite the pure sanitizer, held cross-provider request capture, and same-provider control as automated OPS-05 evidence.

## Self-Check: PASSED

- The provider sanitizer is local to model switching and does not call `strip_reasoning_blocks`.
- All task commits are present and scoped to planned files plus the authorized five-file chat-state and test-constructor fallout.
- No secret, URL heuristic, broad generic chat-state API, or placeholder behavior was introduced.

---
*Phase: 10-codex-responses-wire-parity*
*Completed: 2026-07-18*
