# Phase 10 Research: Codex Responses wire parity

**Status:** Planning research complete (generic-agent workaround)  
**Scope:** Trusted first-party ChatGPT/Codex HTTP Responses only. This report uses local repository evidence and the locked decisions in `10-CONTEXT.md`; it does not change product code or establish live-UAT success.

## Outcome and boundary

Phase 10 should introduce an explicit, provider-scoped Codex Responses wire profile. It must be enabled only after the existing first-party-Codex/OAuth trust gate succeeds. xAI, BYOK, custom OpenAI-compatible endpoints, WebSocket transport, and Codex tool-name/freeform parity stay unchanged.

The profile must make the Codex HTTP contract deliberate rather than relying on sampler-wide defaults:

- preserve full HTTP history and `previous_response_id: None`;
- set `store: false`, `stream: true`, and include `reasoning.encrypted_content`;
- keep the current `instructions` conversion and prohibit `role: system` in `input`;
- send tool controls only when tools are exposed (`tool_choice: auto`, parallel tool calls enabled);
- omit `reasoning.summary` by default;
- add only Codex-safe request identity headers: `originator: bum`, stable per-session `session-id`, `thread-id`, and `x-client-request-id`;
- retain `ChatGPT-Account-ID` only under the existing trusted-host condition.

This is consistent with the local interoperability reference in `.planning/research/CODEX-RESPONSES-WIRE.md` and the Phase 10 decisions in `10-CONTEXT.md`, without copying Codex CLI identity, telemetry, attestation, installation metadata, or beta headers.

## Verified seam map

| Concern | Current behavior and evidence | Recommended Phase 10 seam |
|---|---|---|
| Responses conversion | `From<&ConversationRequest> for rs::CreateResponse` in `crates/codegen/xai-grok-sampling-types/src/conversation.rs:2104` raises all system text to `instructions` (`extract_responses_instructions`, 2179) and omits system items from `input` (`build_responses_input`, 2202). Tests at 3590 and 3623 protect this. | Preserve this generic conversion invariant. Apply Codex-specific request policy after conversion, not by hard-coding it into the generic `From` implementation. |
| Body defaults | Conversion leaves `include`, `store`, and `stream` unset (2147, 2165-2166). Generic sampler defaults add `store: false` and encrypted-reasoning include in `crates/codegen/xai-grok-sampler/src/client.rs:1140`; the streaming call sets `stream: true` at 1318. | Carry a `TrustedCodexResponses` profile from shell route reconstruction into `SamplerConfig`/request construction, then explicitly set the Codex fields. Keep full history and no `previous_response_id`. Decide with tests whether generic defaults remain for xAI. |
| Tools | Conversion maps `tool_choice` at `conversation.rs:2118`; `parallel_tool_calls` is currently `None` (2154). | When bum exposes tools on the trusted Codex profile, send `tool_choice: auto` and `parallel_tool_calls: true`; omit tool controls if there are no tools. |
| Reasoning | Conversion forces `summary: Some(Concise)` at `conversation.rs:2159`, regardless of provider/model. The catalog only carries effort defaults: Sol low, Terra/Luna medium in `crates/codegen/xai-grok-models/default_models.json:19`. | The Codex profile must set reasoning summary to absent. Keep model-specific summary capability and effort soft-clamping for Phase 11. |
| Trusted headers | `reconstruct_full_config` knows route facts and trusted Codex OAuth in `crates/codegen/xai-grok-shell/src/session/acp_session_impl/sampler_turn.rs:288`. It injects `ChatGPT-Account-ID` only through `inject_chatgpt_account_id_header` (`agent/config.rs:4788`); tests cover OAuth, BYOK, and custom-host isolation in `codex_reconstruct_refresh_tests.rs:148`. | Derive stable UUID continuity values from bum session state here (or an adjacent shell-owned type), insert Codex headers only after the same first-party gate, and pass them through existing `extra_headers`. Do not use URL heuristics inside sampler. |
| Retry storm | `stream_responses` emits `ResponseOutputTextDelta` to the UI but does not retain text (`crates/codegen/xai-grok-sampler/src/stream/responses.rs:184`). Final items come from terminal `response.output` (411-514). `drive_l2` retries an empty completed response (`actor/request_task.rs:535`). | Accumulate text deltas and inject them only when terminal output has no assistant text. Preserve terminal usage/stop reason and avoid duplication when terminal output is complete. |
| Foreign encrypted reasoning | Requests serialize `ReasoningItem.encrypted_content` back into input (`conversation.rs:2272`). Model switching only replaces sampling config/credentials and does not sanitize history (`crates/codegen/xai-grok-shell/src/session/acp_session_impl/model_switch.rs:50`). | On a true provider transition, remove provider-scoped encrypted reasoning before an incompatible request is emitted, while retaining ordinary user, assistant, and tool history. Same-provider Codex continuity must remain intact. |
| Error classification | `is_encrypted_content_error` recognizes only `encrypted_content` in `crates/codegen/xai-grok-sampling-types/src/error.rs:211`; the session path repeats the narrow match. | Centralize one case-insensitive, HTTP-400-gated matcher accepting both `encrypted_content` and `encrypted content`. Use bounded recovery/friendly surfacing, never an open-ended retry loop. |

## Recommended design shape

1. **Shell owns provider trust and identity.** Extend the sampling configuration sent from `SessionActor::reconstruct_full_config` with an optional typed wire profile (for example, `TrustedCodexResponses`) rather than a naked URL boolean. Construct it only when the resolved model provider is Codex, credentials are ChatGPT session OAuth, and `is_first_party_codex_url` succeeds.
2. **Sampler owns final HTTP request shaping.** In `conversation_stream_responses` / Responses request creation in `xai-grok-sampler/src/client.rs`, convert the generic conversation first, then apply the typed profile. This prevents a ChatGPT-specific `summary`, header, or `parallel_tool_calls` decision from changing xAI or custom Responses routes.
3. **Session owns transition sanitation.** Before replacing or issuing a request after model/provider switch, compare the previous and new provider and clear only encrypted reasoning blobs on a cross-provider change. The current `ReasoningItem` has no origin provenance, so retain normal history and test the exact target behavior. Do not strip same-provider Codex continuity.
4. **Stream parser owns visible-text recovery.** Maintain a text accumulator alongside the existing reasoning accumulator in `stream/responses.rs`; use it as a fallback only if the terminal response has no assistant text.

## Required regression tests

### Request and header isolation

- Add a serialized trusted-Codex request test proving non-empty `instructions`, no system role in `input`, `store: false`, `stream: true`, encrypted-content include, absent reasoning summary, and no `previous_response_id`.
- Add tool-present and tool-absent cases for `tool_choice`/parallel tool calls.
- Extend the existing Codex reconstruction tests to assert `originator: bum`, stable UUID identity headers, and non-leakage to xAI, BYOK, and custom hosts. Preserve the current `ChatGPT-Account-ID` trust tests.

### Reliability and switching

- In `xai-grok-sampler`, use a fake SSE stream with `response.output_text.delta("hi")` followed by a completed event whose `output` is empty. Assert completed assistant text is `hi` and exactly one request/attempt occurs. Existing `text_delta_then_completed_yields_completed_with_stop` at `stream/responses.rs:628` is a useful fixture seam but presently does not assert text retention.
- Add a terminal-output-present control to prove fallback does not duplicate assistant text.
- Add a shell/model-switch request-capture test for Grok → Codex: ordinary history survives, foreign encrypted content is absent from outbound Codex `input`, and Codex → Codex remains eligible to preserve its own continuity.
- Add matcher cases for underscore, space, case variation, wrong status, and unrelated messages; ensure a 400 encrypted mismatch does not enter a retry storm.

## Validation sequence

## Validation Architecture

The phase uses the existing Rust crate-local unit and integration infrastructure; no new test framework, package, or external service setup is required. Fast feedback comes from focused `cargo test -p <crate> <filter>` invocations. Each implementation task must add or extend the nearest existing regression fixture before its production change is considered complete.

The validation layers are deliberately complementary:

- sampling-types unit tests prove request conversion and encrypted-content error classification;
- sampler unit/SSE tests prove terminal assistant-text reconstruction without duplication;
- sampler actor integration tests count mock HTTP attempts to prevent an empty-response retry storm;
- shell reconstruction and model-switch integration tests prove trusted header isolation and cross-provider history sanitation;
- redacted live OPS-04 and OPS-05 reruns prove the actual dual-login daily-driver path after the rebuilt `bum` binary is used.

No automated fixture substitutes for the two live checks. The executor should retain ordinary empty-response retry coverage, use bounded encrypted-content recovery only, and run `cargo fmt --all -- --check` after the focused suites.

Run focused automated tests while implementing, then perform live validation only with the rebuilt `bum` binary and redacted evidence:

```bash
cargo test -p xai-grok-sampling-types conversation::tests --lib
cargo test -p xai-grok-sampler text_delta_then_completed --lib
cargo test -p xai-grok-sampler --test test_actor
cargo test -p xai-grok-shell --lib codex_reconstruct
cargo test -p xai-grok-shell --test model_switch_gate
cargo fmt --all -- --check
```

Then follow `.planning/phases/09-daily-driver-end-to-end-validation/09-SESSION-HANDOFF.md:89`:

- **OPS-04:** select `gpt-5.6-sol`, send `hi`, then read/edit a file. Visible successful text must complete once, with no retry.
- **OPS-05:** use Grok, switch to `gpt-5.6-sol`, then send `hi`. The turn must not fail with encrypted-content/decryption 400 and ordinary context must remain useful.

Automated fixtures are necessary but not sufficient: neither Phase 9 nor Phase 10 is green until those live dual-login checks pass with no secrets committed.

## Risks and guardrails

- **Provider leakage:** a generic `From<&ConversationRequest>` serves multiple Responses backends. Keep the Codex profile typed and trust-gated, and assert isolation in tests.
- **Continuity regression:** stripping all reasoning indiscriminately would damage same-provider Codex history. Sanitize only encrypted material on a real provider transition.
- **Fallback duplication:** delta reconstruction must fill only missing terminal text, not append to already complete output.
- **Identity/privacy:** UUIDs must be bum session-local and stable only within the intended session; never use or persist third-party identity/telemetry values.
- **False recovery:** encrypted-content matching must remain status-gated and bounded so unrelated 400s are surfaced normally.

## Explicit deferrals

- **Phase 11:** effort soft-clamping, extended reasoning-menu fidelity, and model-specific reasoning-summary capability policy.
- **Phase 12:** Responses-over-WebSocket, stock Codex tool/freeform-`apply_patch` naming parity, and broader attribution/notices work.
- **Outside this phase:** full OPS-03..OPS-06 daily-driver closeout beyond the required OPS-04/OPS-05 rerun, plus any Codex CLI telemetry/attestation/product-identity copying.

## RESEARCH COMPLETE

Artifact: `.planning/phases/10-codex-responses-wire-parity/10-RESEARCH.md`
