# Phase 10: Pattern Mapping — Codex Responses Wire Parity

## Purpose and boundaries

Phase 10 must add a **typed, trusted-first-party Codex Responses wire profile** without changing the generic Responses conversion used by xAI, BYOK, or custom endpoints.  The shell is the ownership boundary for endpoint trust, OAuth provenance, and session identity; the sampler receives explicit configuration and shapes only requests for that configuration.

The phase has three connected data flows:

```text
Model/auth/endpoint facts + session UUID
    -> SessionActor::reconstruct_full_config
    -> SamplerConfig { trusted Codex wire profile, trusted-only headers }
    -> SamplingClient / ClientDefaults
    -> generic ConversationRequest -> CreateResponse conversion
    -> profile-specific Responses request defaults -> serialized HTTP body

Responses SSE text deltas
    -> stream_responses text accumulator
    -> completed response item conversion + missing-text fallback
    -> RequestTask::drive_l2 observes a non-empty assistant result
    -> no erroneous Empty retry loop

Current provider + requested provider
    -> SessionActor::handle_set_session_model
    -> remove only encrypted Reasoning payloads on a real provider change
    -> retained ordinary history is serialized on the next provider request
```

Keep the following explicitly out of Phase 10: WebSocket transport/tool-name parity (Phase 12), reasoning capability/policy negotiation and UI behavior (Phase 11), generic endpoint heuristics in the sampler, and changes to xAI/BYOK/custom endpoint behavior.

## Expected file map

| File | Role in Phase 10 | Primary symbols / useful ranges | Closest pattern and required data-flow responsibility |
|---|---|---|---|
| `crates/codegen/xai-grok-sampler/src/config.rs` | Production configuration contract | `SamplerConfig` (48–127), `Default` (129–162), `AuthScheme` (18–24), `doom_loop_recovery` (115–122) | Add a serde-defaulted typed profile (for example `ResponsesWireProfile` / `TrustedCodexResponses`) with a disabled default. It must express a semantic capability, not a URL or provider guess. Propagate it through config defaults and update explicit `SamplerConfig` literals. `doom_loop_recovery: Option<_>` is the nearest optional, backward-compatible configuration pattern. |
| `crates/codegen/xai-grok-sampler/src/client.rs` | Production request shaping and serializer-focused unit tests | `SamplingClient::new` (401+), extra-header pass-through (441–450), `apply_response_defaults` (1140–1174), `create_response_stream` (1307–1439), `conversation_stream_responses` (1904–1939), `conversation_responses` (1945–1971), test `minimal_config` (2088–2117) | Copy profile into `ClientDefaults`; apply it after generic `ConversationRequest -> rs::CreateResponse` conversion, at the existing Responses-default seam. For the trusted profile set `store: false`, `stream: true`, `include: ["reasoning.encrypted_content"]`, no `previous_response_id`, and tools-only `tool_choice: "auto"` plus `parallel_tool_calls: true`; omit tool controls when no tools; remove/omit `reasoning.summary`. Do not inspect endpoint URLs here. Add JSON serialization tests for profile-on/profile-off, tools/no-tools, system-to-instructions, and absence of `previous_response_id`. |
| `crates/codegen/xai-grok-sampling-types/src/conversation.rs` | Generic conversion invariant and optional guard tests; normally no production profile logic | `From<&ConversationRequest> for rs::CreateResponse` (2104–2177), `extract_responses_instructions` (2179–2200), `build_responses_input` (2202–2219), reasoning conversion (2272–2280); system test (3591–3657), encrypted reasoning test (4625–4686), tools test (4803–4829), tool-choice test (4915–4952) | This is the closest source for system/instructions and history semantics, but it is shared conversion code. Preserve its generic defaults for non-Codex routes. Extend tests here only if a regression guard is needed; do not put trusted-Codex wire branching in this conversion. It proves that cleared encrypted content will not be re-sent, while ordinary history remains convertible. |
| `crates/codegen/xai-grok-shell/src/session/acp_session_impl/sampler_turn.rs` | Production trust gate, per-session header injection, profile propagation, friendly encrypted-error recovery | `SessionActor::reconstruct_full_config` (227–398), trusted `codex_session_oauth` gate (288–313), account header block (321–329), final `SamplerConfig` literal (354–397), `prepare_sampler_for_turn` (597–602), current error branch (622–705; narrow duplicate check 652–671) | This is the one place with all required facts: provider, credential kind, canonical endpoint trust, and session actor identity. Derive the typed profile only when `provider == Codex`, session-token OAuth is in use, and `is_first_party_codex_url(...)` passes. Add only then the stable local session headers: `originator: bum`, `session-id`, `thread-id`, and `x-client-request-id`, with the latter three based on the actor's session UUID. Keep `ChatGPT-Account-ID` under the same trusted OAuth gate. Route error handling through the shared encrypted-content predicate rather than duplicating a substring test. |
| `crates/codegen/xai-grok-shell/src/agent/config.rs` | Existing canonical endpoint-trust and header-helper source; likely import/default-only production change | `ModelProvider` (3402–3432), `ProviderRoute` (4432–4450), `resolve_provider_route` (4472–4499), `is_first_party_codex_url` (4501–4539), `inject_chatgpt_account_id_header` (4788–4808), `sampling_config_for_model` (5223–5271), `inject_url_derived_headers` (5273–5304) | Reuse `is_first_party_codex_url` as the authoritative trust decision and `inject_chatgpt_account_id_header` as the header-injection style. Do **not** put Bum identity headers in `inject_url_derived_headers` or `sampling_config_for_model`: neither has session ownership, and both are broader than trusted Codex OAuth. |
| `crates/codegen/xai-grok-shell/src/agent/mvp_agent/acp_agent.rs` | Existing session-ID provenance; reference only unless a narrow helper is warranted | session ID validation/generation (838–878) | `Uuid::try_parse` validates client metadata and falls back to `Uuid::now_v7()`. `self.session_info.id` is therefore the correct stable, session-local source for all three UUID-valued trusted-Codex headers. Do not generate one UUID per request. |
| `crates/codegen/xai-grok-sampling-types/src/error.rs` | Production typed encrypted-content predicate and focused unit tests | `SamplingError` (84–132), `is_encrypted_content_error` (211–224), existing tests (642–684) | Expand the existing typed method, retaining the HTTP 400 gate while accepting case-insensitive `encrypted_content` and `encrypted content`. Keep unrelated 400s and non-400 responses false. This is the single source of truth consumed by shell recovery logic. The nearby `is_image_processing_error` is an error-method style analogue, not a semantic replacement. |
| `crates/codegen/xai-grok-sampler/src/stream/responses.rs` | Production SSE text-delta fallback and L2 transformer tests | `stream_responses` (94–516), state initialization (123), text delta handling (184–204), terminal response retention (281–288), completion conversion/fallback (411–508), test fixtures `text_delta_event` (579–588), `completed_event` (590–595), main regression test (628–659) | Add a `text_acc` next to existing `reasoning_acc`, append it while emitting `ChannelToken`, and use it only to fill a *missing* terminal assistant text item after `response_to_conversation_items`. Do not append if terminal output already contains assistant text; preserve terminal usage/status/items. `inject_streaming_reasoning_fallback` is the closest fallback pattern, but there is no exact text counterpart—add a narrowly named helper rather than overloading reasoning behavior. Extend the delta-then-completed test to inspect final assistant content and add a terminal-text/no-duplication control. |
| `crates/codegen/xai-grok-sampler/src/actor/request_task.rs` | Existing retry behavior; normally production-unchanged, used by regression test reasoning | `run_request_task` Empty retry path (179–212), `drive_l2` (500–567), Responses branch in `run_one_attempt` (438–452) | Do not suppress `AttemptOutcome::Empty` generically. The stream fix must make the delta-only terminal response non-empty before this layer. Preserve retries for genuinely empty responses. This file is a diagnostic analogue, not the primary implementation seam. |
| `crates/codegen/xai-grok-sampler/tests/test_actor.rs` | Actor-level retry-storm regression test | `MockServer` (36–65), full `SamplerConfig` literal (71–101), Responses config helpers (772–777), existing doom-loop/retry tests nearby | Add an Axum/SSE fake that sends visible text deltas followed by a completed response with no terminal assistant text. Count requests with `AtomicU32`; assert one successful logical completion and one inference request, proving `drive_l2` does not enter the Empty retry path. Reuse the existing mock server and SSE test idioms. |
| `crates/codegen/xai-grok-shell/src/session/acp_session_tests/codex_reconstruct_refresh_tests.rs` | Route-isolation and session-header unit/integration tests around reconstructed sampler config | fixture `make_codex_actor` (70–113), trusted OAuth test (121–169), BYOK test (174–217), custom endpoint test (222–266) | Extend the existing trust matrix. Trusted first-party session OAuth gets account ID plus `originator: bum` and stable UUID session/thread/request IDs; assert it does not identify as `codex_cli_rs`. BYOK and custom endpoints receive none of the trusted identity headers/profile. Add an xAI/non-Codex control if the fixture can express it cheaply. |
| `crates/codegen/xai-grok-shell/src/session/acp_session_impl/model_switch.rs` | Production cross-provider encrypted-history sanitation plus pure transform tests | `handle_set_session_model` (5–108), current `PreparedSamplingConfig` update (50–65), conversation get/rewrite flow (71–86) | Before committing the target provider, determine the current provider from current model facts and compare it with the target `provider`. On an actual provider change, use the existing chat-state get/replace flow to clear only `ConversationItem::Reasoning.encrypted_content`; retain ordinary messages, tool items, summaries, and unencrypted reasoning. Same-provider switches must not sanitize. `crates/codegen/xai-chat-state/src/compaction_utils.rs::strip_reasoning_blocks` (43–53) is intentionally too broad and must not be reused. A small local pure helper has no exact existing analogue and is preferable to a new broad chat-state API. |
| `crates/codegen/xai-grok-shell/tests/model_switch_gate.rs` | End-to-end model-switch request-capture regressions | `GateHarness` (307–523), model-route/body capture (713–781), history persistence test (804–855), same-provider Codex test (696–709) | Add a request-capture proof that a Grok↔Codex switch retains an ordinary history marker but sends no encrypted reasoning field to the new provider. Add same-provider Codex continuity proof that encrypted content is retained. Pair this with local pure-helper tests in `model_switch.rs`: the former proves timing before outbound serialization; the latter proves exact mutation semantics. |
| `crates/codegen/xai-grok-test-support/src/sse.rs` | Optional reusable fixture only | `responses_api_events` (145–158), delta helper path (169+) | No required production change. Use only if the actor test benefits from a shared delta-only-terminal-empty event builder. If this `src/` file changes, also update `crates/codegen/xai-grok-test-support/README.md` per project convention; otherwise keep the test local and avoid a fixture-only expansion. |

## Configuration and request-shaping contract

### 1. Represent trust as configuration, not inference

Add a narrow type in `xai-grok-sampler/src/config.rs`, such as a profile enum with a disabled default and a `TrustedCodexResponses` value. The exact name can change, but its contract should be:

- It is computed by shell code after the canonical first-party endpoint check and credential-kind check.
- It serializes/defaults safely so existing persisted or test configuration remains compatible.
- It is copied into `ClientDefaults` when `SamplingClient` is constructed.
- It never receives a URL, and sampler code does not call endpoint routing/trust helpers.

Update all explicit `SamplerConfig` construction sites identified by compiler errors, especially the sampler actor fixture (`tests/test_actor.rs:71–101`), sampler `minimal_config` (`src/client.rs:2088–2117`), and shell's `sampling_config_for_model` (`agent/config.rs:5243–5271`) / reconstructed config (`sampler_turn.rs:354–397`). Prefer `..SamplerConfig::default()` where that matches existing local style, but preserve intentional test values.

### 2. Shape only after generic conversion

`ConversationRequest` conversion in `sampling-types/src/conversation.rs` must remain provider-agnostic. At `SamplingClient::conversation_stream_responses` / `conversation_responses`, conversion already occurs before the shared client-defaults/request path. Apply the profile in `apply_response_defaults` (or a profile-specific helper called from it), so a trusted route gets the contract below while generic routes retain current behavior.

| Request property | Trusted Codex Responses profile behavior | Notes |
|---|---|---|
| `stream` | Always `true` for the stream call | Existing `create_response_stream` already forces it; serialization test should prove it. |
| `store` | Explicit `false` | Keep as a profile-owned privacy behavior, rather than allowing a first-party Codex requirement to leak into unrelated Responses callers. |
| `include` | Exactly / at least `reasoning.encrypted_content` | Required to preserve same-provider encrypted continuity. |
| `previous_response_id` | Omit / `None` | Full sanitized conversation history is authoritative in Phase 10. |
| system messages | Convert to `instructions`; never emit `role: "system"` in `input` | Existing generic conversion already implements this invariant. |
| tools present | `tool_choice: "auto"`; `parallel_tool_calls: true` | Apply only when the converted request actually has tools. |
| no tools | Omit tool-control fields | Do not manufacture `tool_choice` or parallel controls. |
| reasoning summary | Omit `reasoning.summary` | This is wire parity, not Phase 11 reasoning policy. |

Do not make this profile a global default merely because the current `apply_response_defaults` writes `store`/`include` generically. The planner should explicitly decide whether existing generic defaults move under the profile or are retained for prior behavior, then add a profile-off regression test to prevent accidental xAI/custom change.

### 3. Trusted session headers belong in `reconstruct_full_config`

The existing local boolean at `sampler_turn.rs:288–313` is the correct composition point:

```text
provider == Codex
&& creds.auth_type == SessionToken
&& is_first_party_codex_url(&cfg.base_url, &endpoints)
```

Use the same boolean for all trusted-Codex-only outcomes:

- enable the typed Responses profile;
- retain the existing `ChatGPT-Account-ID` injection;
- append `originator: bum`;
- append `session-id`, `thread-id`, and `x-client-request-id` using the same `self.session_info.id` UUID string for the lifetime of the session.

`SamplerConfig.extra_headers` is an intentional pass-through (client construction copies entries verbatim at `client.rs:441–450`), so it is suitable once the shell owns the gate. Do not alter `inject_url_derived_headers`; it is a generic URL-header helper and lacks the trusted OAuth/session boundary. Do not add `codex_cli_rs`, telemetry, attestation, or beta headers.

## Error, SSE, and retry patterns

### Encrypted-content recovery

Consolidate matching in `SamplingError::is_encrypted_content_error`:

- only `SamplingError::Api` with status `400 Bad Request` qualifies;
- match case-insensitively;
- accept both the underscore spelling (`encrypted_content`) and whitespace spelling (`encrypted content`);
- reject unrelated 400s and all other status classes.

`sampler_turn.rs` currently replicates a narrow status/string test while producing a friendly user-facing error. Refactor that branch to consume the centralized predicate (or a small shared pure predicate designed for the shell's `SamplingErrorInfo` representation). Do not leave two independently evolving matchers. The friendly recovery remains bounded: it should avoid an error/retry loop, not silently retry arbitrary 400s.

### Delta-only terminal fallback

The current L2 stream transformer emits text deltas but can finish with a completed response whose converted items contain no assistant text. `RequestTask::drive_l2` then correctly classifies the result as empty and the actor retries; therefore the fix belongs before that classification.

Implementation outline tied to existing patterns:

1. Initialize `text_acc: String` alongside `reasoning_acc` in `stream_responses`.
2. In the existing output-text-delta branch, append the same delta that is emitted as `ChannelToken`.
3. At completed-response conversion, inspect returned conversation items.
4. If the terminal conversion contains no assistant text and `text_acc` is non-empty, add/fill one assistant-text item from `text_acc`.
5. If terminal output already has assistant text, do nothing—never concatenate a duplicate.
6. Retain the existing reasoning fallback independently; do not make text behavior depend on reasoning content.

The closest current mechanism is `inject_streaming_reasoning_fallback`, but text fallback has no direct analogue. A dedicated helper with a precise predicate is safer than bending the reasoning helper or changing `RequestTask` retry policy.

## Model-switch sanitation pattern

`handle_set_session_model` already owns the target provider, current session/chat state, and the mutation sequence. Add sanitation there, before the next sampling configuration is committed/used for a new turn:

1. Resolve the old provider from the current model's auth facts while those facts are still available.
2. Compare it to the requested target `provider`.
3. If the provider is unchanged, leave all conversation history exactly intact.
4. If it changed, load the conversation through the existing chat-state handle, clear only encrypted payloads on `ConversationItem::Reasoning`, and replace it through the existing `replace_conversation` path.
5. Keep regular user/assistant messages, tool calls/results, summaries, and non-encrypted reasoning structure so full-history routing remains useful.

Do not call `strip_reasoning_blocks` from `crates/codegen/xai-chat-state/src/compaction_utils.rs`: it removes entire reasoning items and is a compaction-oriented broad transform, not a provider-secret sanitizer. There is no strong exact existing helper for field-level encrypted-payload removal; a local, pure `model_switch.rs` transform plus focused tests is the lowest-risk pattern.

## Test design and acceptance matrix

| Concern | Preferred test location | Assertions |
|---|---|---|
| Typed profile defaults/propagation | `xai-grok-sampler/src/config.rs` and `src/client.rs` unit tests | Disabled default remains backward-compatible; explicit trusted profile reaches request shaping. |
| Trusted request serialization | `xai-grok-sampler/src/client.rs` tests | `store:false`, encrypted include, no `previous_response_id`, no system role in input, no reasoning summary; tools produce `tool_choice:auto` and `parallel_tool_calls:true`; no-tools omit both. Profile-off test demonstrates no accidental generic behavior change. |
| First-party trust and session identity | `codex_reconstruct_refresh_tests.rs` | First-party session OAuth gets profile, account ID, `originator: bum`, and stable UUID header values; no `codex_cli_rs`. BYOK/custom routes get neither trusted headers nor profile. |
| Matcher precision | `xai-grok-sampling-types/src/error.rs` tests | Both spellings and case variants at HTTP 400 match; unrelated 400/non-400 do not. Shell friendly branch uses that behavior rather than its own substring logic. |
| SSE final assistant content | `xai-grok-sampler/src/stream/responses.rs` tests | Delta + terminal-empty completed response produces final assistant text once; delta + terminal-text response does not duplicate it; reasoning fallback remains intact. |
| Actor retry regression | `xai-grok-sampler/tests/test_actor.rs` | A delta-visible/terminal-empty Responses SSE sequence finishes after one HTTP request; request counter proves no `Empty` retry storm. |
| Exact sanitation transform | `model_switch.rs` local tests | Cross-provider transform clears only `encrypted_content`; normal messages and unencrypted reasoning survive; same-provider transform is a no-op. |
| Sanitation timing and same-provider continuity | `model_switch_gate.rs` | Captured next-provider body after a cross-provider switch retains normal history but omits encrypted data. A same-provider Codex switch retains encrypted content in its next request. |

Useful targeted validation after implementation:

```bash
cargo test -p xai-grok-sampling-types conversation::tests --lib
cargo test -p xai-grok-sampling-types encrypted_content --lib
cargo test -p xai-grok-sampler text_delta_then_completed --lib
cargo test -p xai-grok-sampler --test test_actor
cargo test -p xai-grok-shell --lib codex_reconstruct
cargo test -p xai-grok-shell --test model_switch_gate
cargo fmt --all -- --check
```

Run the exact test names discovered during implementation if module filters differ; use the focused crates rather than a full workspace test first.

## File-level implementation cautions

- Root `Cargo.toml` is generated: do not edit it for this phase. A typed profile should need no new dependency.
- `SamplerConfig` explicit literals are likely compiler-visible fallout from adding a field; update only the relevant defaults/fixtures, preserving existing test semantics.
- Headers must be structured values in the current `extra_headers` flow and must never log OAuth material or account IDs.
- A UUID assertion should validate parseability and equality across reconstructed configs in the same actor/session, not assert a wall-clock-derived UUID value.
- The actor regression must retain ordinary `Empty` retry coverage; changing retry classification alone would conceal a malformed response rather than repair the wire/event translation.
- If test-support source is changed for a shared SSE fixture, update its README in the same change. Otherwise an inline test event is the smaller pattern.

## Strong analogs versus intentional new seams

| Need | Strong existing analogue | Planning conclusion |
|---|---|---|
| Provider/endpoint trust | `is_first_party_codex_url` in `agent/config.rs` | Reuse exactly; do not reimplement host/path checks. |
| Trusted account header | `inject_chatgpt_account_id_header` | Extend beside it in session reconstruction, not generic URL/header helpers. |
| Optional typed config | `doom_loop_recovery` in `SamplerConfig` | Use a defaulted semantic profile. |
| Generic system/history conversion | `ConversationRequest -> CreateResponse` | Preserve it; apply provider profile downstream. |
| Typed error classification | `SamplingError::is_encrypted_content_error` | Expand and centralize it. |
| Streaming fallback | `inject_streaming_reasoning_fallback` | Mirror its lifecycle placement, but implement a separate text-only helper. |
| History mutation | `get_conversation` / `replace_conversation` in `model_switch.rs` | Reuse the mutation path; add a narrow item-field sanitizer. |
| Field-level encrypted-history removal | No strong exact analogue; `strip_reasoning_blocks` is too broad | New local pure helper is justified. |
| Delta-only terminal-text fallback | No strong exact analogue | New narrow helper at the Responses stream completion seam is justified. |

This mapping keeps the Phase 10 change localized: shell decides who is trusted, sampler shapes the already-converted request, streaming returns a truthful completed response, and model switching removes only provider-scoped encrypted material when crossing providers.
