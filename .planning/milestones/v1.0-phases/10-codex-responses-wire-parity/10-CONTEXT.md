# Phase 10: Codex Responses wire parity - Context

**Gathered:** 2026-07-18
**Status:** Ready for planning

<domain>
## Phase Boundary

Make bum's trusted ChatGPT/Codex HTTP Responses path match the official Codex HTTP contract closely enough for productive GPT turns to remain reliable. The phase owns Codex-specific request defaults, trusted request identity headers, safe cross-provider history handling, and terminal SSE reconstruction. It must not alter the xAI path, BYOK/custom OpenAI endpoints, WebSocket transport, tool-name parity, or product identity beyond identifying requests as **bum**.

Phase 10 may implement deterministic wire and reliability fixes while Phase 9's real dual-login validation remains incomplete. It cannot claim the daily-driver bar is green until the rebuilt binary passes the redacted live OPS-04 and OPS-05 reruns.

</domain>

<decisions>
## Implementation Decisions

### Provider Boundary and Delivery Scope
- Apply a dedicated Responses wire profile only to trusted first-party ChatGPT/Codex routes; keep xAI, BYOK, and custom endpoints unchanged.
- Include the observed Codex post-success retry storm and foreign encrypted-reasoning replay on provider switch because both block productive GPT turns.
- Keep WebSocket incremental transport, Codex tool naming/freeform patch parity, and broader attribution polish out of this phase for Phase 12.
- Require automated proof plus a real redacted OPS-04/OPS-05 rerun; do not mark Phase 9 or Phase 10 green on fixture-only evidence.

### Trusted Codex Wire Profile
- Make Codex defaults explicit: `store: false`, `stream: true`, include `reasoning.encrypted_content`, and preserve full HTTP input history rather than using `previous_response_id`.
- Send `tool_choice: "auto"` and enable parallel tool calls when bum exposes tools; omit tool controls when no tools are present.
- Omit `reasoning.summary` by default instead of always forcing `concise`; future model capability policy belongs to Phase 11.
- Preserve the existing system-to-top-level-`instructions` conversion and the invariant that Responses `input` never contains `role: system`.

### Request Identity and Privacy
- Emit `originator: bum`, never `codex_cli_rs`, and do not copy Codex product telemetry, attestation, installation, or beta headers.
- Send `session-id`, `thread-id`, and `x-client-request-id` only on trusted first-party ChatGPT/Codex routes.
- Derive stable UUID continuity values from bum session state without storing or sharing third-party identity data.
- Continue to send `ChatGPT-Account-ID` only when available and trusted; never leak Codex-only headers to xAI, BYOK, or custom URLs.

### Reliability Recovery and Validation
- On a true provider transition, remove provider-scoped encrypted reasoning blobs while retaining ordinary user, assistant, and tool history.
- Recognize both `encrypted_content` and `encrypted content` HTTP 400 forms and use bounded recovery, never an open-ended retry loop.
- Accumulate visible text deltas and use them only if the terminal response lacks assistant text; terminal completion must not trigger an empty-response retry after visible success.
- Add serialized-request, header-isolation, fake-SSE, encrypted-switch, and error-matcher regressions before running the redacted live UAT.

### the agent's Discretion
- Exact Rust type ownership and injection boundary for the trusted-Codex wire profile and stable UUID derivation.
- Whether existing generic sampler defaults move into the provider-aware policy or remain generic after tests prove no behavior leaks.
- Exact test module placement, fixture naming, and focused Cargo commands consistent with established crate conventions.
- The smallest safe bounded-recovery behavior after an encrypted-content mismatch, provided it never hides a real error or loops.

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `ConversationRequest -> rs::CreateResponse` already moves system history into `instructions` and excludes system roles from Responses `input` in `crates/codegen/xai-grok-sampling-types/src/conversation.rs`.
- Sampler construction already supplies bearer auth, JSON/SSE headers, `store: false`, streaming, and encrypted-reasoning include defaults in `crates/codegen/xai-grok-sampler/src/client.rs`.
- The shell has trusted first-party Codex route reconstruction plus scoped `ChatGPT-Account-ID` injection.
- Phase 9 provides the live UAT checklist, non-secret preflight, and redacted evidence pattern.

### Established Patterns
- Provider routing and credentials fail closed; credentials and provider-specific headers must not cross slots.
- Rust unit and integration tests use focused `cargo test -p <crate>` commands; full workspace checks are too expensive for each iteration.
- Session history is actor-owned and provider changes currently preserve history, so transition sanitation must occur before an incompatible request is emitted.

### Integration Points
- Request conversion: `xai-grok-sampling-types/src/conversation.rs`.
- HTTP/SSE construction and Responses terminal mapping: `xai-grok-sampler/src/client.rs` and `src/stream/responses.rs`.
- Provider route reconstruction and model switching: `xai-grok-shell/src/session/acp_session_impl/`.
- Contract reference: `.planning/research/CODEX-RESPONSES-WIRE.md` and the local official Codex source snapshot under `/home/cristian/bum/codex/codex-rs`.

</code_context>

<specifics>
## Specific Ideas

- Treat the local official Codex source as an interoperability reference, not product identity to copy.
- Keep full HTTP history authoritative; `previous_response_id` is not the primary ChatGPT HTTP continuity contract.
- Preserve same-provider Codex continuity while preventing foreign encrypted blobs from reaching Codex after a switch.
- Live proof uses the rebuilt `bum` binary with real dual OAuth and redacted notes only; no tokens, auth documents, or raw secret-bearing transcripts are committed.

</specifics>

<deferred>
## Deferred Ideas

- Phase 11 effort soft-clamping, extended reasoning menu fidelity, and model-specific reasoning-summary capability policy.
- Phase 12 Responses-over-WebSocket, stock Codex tool/freeform-apply-patch naming parity, and wider product attribution/notices work.
- Completing Phase 9's broader OPS-03..OPS-06 daily-driver validation beyond the Phase 10 OPS-04/OPS-05 rerun.
- Copying Codex CLI product identity, telemetry, attestation, installation metadata, or private headers.

</deferred>
