---
phase: 10-codex-responses-wire-parity
verified: 2026-07-18T14:58:27Z
status: passed
score: 9/9 must-haves verified
behavior_unverified: 0
overrides_applied: 0
re_verification: false
deferred:
  - truth: "Workspace `cargo fmt --all -- --check` is green"
    addressed_in: "workspace hygiene chore (out of Phase 10 product scope)"
    evidence: "10-VALIDATION.md deferred items; deferred-items.md — pre-existing ~68-file fmt drift; not OPS wire-parity blocker"
---

# Phase 10: Codex Responses wire parity — Verification Report

**Phase Goal:** bum’s ChatGPT Codex HTTP path matches official Codex CLI wire contracts beyond the system→instructions fix, so productive GPT turns stay green under real dual login  
**Verified:** 2026-07-18T14:58:27Z  
**Status:** passed  
**Re-verification:** No — initial verification  
**Mode:** mvp (ROADMAP); verification is goal-backward against ROADMAP success criteria + plan must_haves (OPS-04 / OPS-05 deepen)

## User Flow Coverage (MVP)

User story (plans 10-01..10-07): *As a bum daily-driver user, I want to complete GPT-5.6 turns and Grok↔GPT-5.6 switches reliably, so that I can keep coding in one CLI without a restart.*

| Step | Expected | Evidence | Status |
|------|----------|----------|--------|
| 1. Log in to Codex + select GPT-5.6 | Trusted first-party ChatGPT OAuth path elects `ResponsesWireProfile::TrustedCodex` + bum identity headers | `sampler_turn.rs` `codex_session_oauth` gate (`ModelProvider::Codex` + `SessionToken` + `is_first_party_codex_url`); `trusted_codex_reconstruct_enables_profile_and_metadata` PASS | ✓ |
| 2. Send a coding turn on GPT-5.6 | Request body matches ChatGPT path (`store:false`, `stream:true`, encrypted include, tools auto, no forced `reasoning.summary`, no `role:system` in input) | `apply_trusted_codex_response_profile` + `trusted_codex_responses_profile_on_off_serializes_exactly` PASS; live OPS-04 PASS | ✓ |
| 3. Turn completes once (tools/read/edit usable) | No post-success empty retry storm; delta-only terminal still yields assistant text | `text_acc` + inject fallback in `stream/responses.rs`; `delta_only_terminal_empty_completes_once_without_retry` PASS; live OPS-04 “no post-success retry” | ✓ |
| 4. Switch Grok → GPT-5.6 in same process | Foreign encrypted reasoning stripped; ordinary history retained; no encrypted-content 400 / store-false id 404 | `clear_encrypted_reasoning` / `sanitize_provider_history` in `model_switch.rs`; `strip_input_item_ids_for_store_false`; unit + live OPS-05 PASS | ✓ |
| Outcome | Productive GPT turns stay green under real dual login | `10-VALIDATION.md` §Live OPS evidence: OPS-04 PASS + OPS-05 PASS (2026-07-18) | ✓ |

**Outcome clause verified:** productive GPT turns remain green under real dual login (automated wire contracts + redacted operator live evidence).

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | Codex Responses requests set `store`/`stream`/`include`/`tool_choice` consistent with ChatGPT path (not Azure-store assumptions) — **ROADMAP SC1** | ✓ VERIFIED | `ResponsesWireProfile::TrustedCodex` → `apply_trusted_codex_response_profile`: `store=Some(false)`, clears `previous_response_id`, tools-only `tool_choice=auto` + `parallel_tool_calls=true`. Stream path forces `stream=true`. Generic path still includes `reasoning.encrypted_content`. Serialization test asserts exact body for profile on/off. Re-run: `trusted_codex_responses_profile_on_off_serializes_exactly` ok. |
| 2 | Optional Codex-compatible headers (`session-id` / `thread-id` / originator as **bum**, not `codex_cli_rs`) documented and applied where they improve reliability — **ROADMAP SC2** | ✓ VERIFIED | Documented in `.planning/research/CODEX-RESPONSES-WIRE.md` (prefer bum originator). Applied only under `codex_session_oauth` in `sampler_turn.rs`: `originator=bum`, stable UUID for `session-id`/`thread-id`/`x-client-request-id`. Reserved headers stripped before trusted reinsert. `CODEX_ORIGINATOR = "bum"`. `codex_cli_rs` only appears in tests as collision input to prove rewrite. Re-run: `trusted_codex_wire_headers_are_sent_and_stable` ok; reconstruct unit ok. |
| 3 | Reasoning summary defaults omit or match GPT-5.6 catalog (not always force `concise` if server rejects) — **ROADMAP SC3** | ✓ VERIFIED | Trusted profile sets `reasoning.summary = None`. Disabled profile still serializes `summary: "concise"` (generic path unchanged). Asserted in `trusted_codex_responses_profile_on_off_serializes_exactly` (`trusted["reasoning"].get("summary").is_none()` vs generic `"concise"`). Re-run: ok. |
| 4 | Automated tests assert no `role: system` in Responses `input` and non-empty `instructions` when system history present — **ROADMAP SC4** | ✓ VERIFIED | Conversion lifts system into top-level `instructions` and filters system from `input` (`conversation.rs` + comments). Tests: `responses_api_never_emits_system_role_in_input` and trusted serialization (`instructions == "Base agent prompt."`, no system role in input). Re-run: both ok. |
| 5 | Live OPS-04 re-run stays PASS after wire tweaks — **ROADMAP SC5** | ✓ VERIFIED | `10-VALIDATION.md` §Live OPS evidence: operator 2026-07-18 **PASS** — GPT-5.6 turns complete once, no post-success retry, read/edit/tool usable. Automated half 26/26 PASS before live. Accepted as human evidence already satisfied (not re-opened). |
| 6 | Visible SSE text deltas become assistant text when terminal completion omits output; genuine empty still boundedly retried; delta-only success does not storm-retry — **10-02 / OPS-04** | ✓ VERIFIED | `stream/responses.rs` `text_acc` + `inject_streaming_text_fallback`. Unit: `text_delta_then_completed_uses_fallback_when_terminal_empty` ok; actor: `delta_only_terminal_empty_completes_once_without_retry` ok. VALIDATION also records terminal-wins, multi-output, incomplete, and bounded empty-retry PASS. |
| 7 | Cross-provider switch removes only encrypted reasoning; same-provider Codex retains it; ordinary user/assistant/tool history kept — **10-03 / OPS-05** | ✓ VERIFIED | `model_switch.rs` `clear_encrypted_reasoning` (field-level only); no `strip_reasoning_blocks` on switch path. Unit: `cross_provider_transition_sanitizes_existing_and_late_reasoning` ok. Gate tests listed (`held_grok_response_after_codex_switch_is_sanitized`, `codex_to_codex_transition_retains_encrypted_reasoning`). Live OPS-05 PASS (Grok→gpt-5.6-luna). |
| 8 | Encrypted-content HTTP 400 variants share one bounded predicate and are terminal before compaction/resubmit; `store:false` strips residual item ids — **10-04 + live fix** | ✓ VERIFIED | `is_encrypted_content_error` (400 only; spaced/case variants) in `error.rs`; shell uses predicate at failure path. Tests: predicate unit + `encrypted_content_400_is_classified_terminal` ok. `strip_input_item_ids_for_store_false` wired in `client.rs` create/stream; unit ok. Live OPS-05 retest after id-strip fix PASS. |
| 9 | Trusted profile is disabled-by-default; only first-party Codex SessionToken OAuth enables it; production/fixture literals do not silently opt in — **10-01/04/06/07** | ✓ VERIFIED | `ResponsesWireProfile` default `Disabled`; shell production/fixtures set `Disabled` except `reconstruct_full_config` under `codex_session_oauth`. Default + propagation test ok; reconstruct enables TrustedCodex + headers only on trust gate. Fixtures in `common/mod.rs`, `provider_routing.rs`, subagent, cancel, etc. use Disabled. |

**Score:** 9/9 truths verified (0 present, behavior-unverified)

### Deferred Items

| # | Item | Addressed In | Evidence |
|---|------|-------------|----------|
| 1 | Workspace `cargo fmt --all -- --check` green | Workspace hygiene chore (not Phase 10 wire scope) | `10-VALIDATION.md` / `deferred-items.md`: ~68 files; pre-existing drift; focused behavior 26/26 still green |

### Required Artifacts

| Artifact | Expected | Status | Details |
| -------- | -------- | ------ | ------- |
| `crates/codegen/xai-grok-sampler/src/config.rs` | `ResponsesWireProfile` on `SamplerConfig` | ✓ VERIFIED | Enum `Disabled`/`TrustedCodex`; default Disabled; serde snake_case |
| `crates/codegen/xai-grok-sampler/src/client.rs` | Profile-aware request shaping + store-false id strip | ✓ VERIFIED | `apply_trusted_codex_response_profile`; `strip_input_item_ids_for_store_false` on serialize path; serialization tests |
| `crates/codegen/xai-grok-sampler/src/stream/responses.rs` | Delta accumulator + terminal fallback | ✓ VERIFIED | `text_acc`, `inject_streaming_text_fallback`, multi-output order tests |
| `crates/codegen/xai-grok-sampler/tests/test_actor.rs` | Retry-storm / Empty-retry actor fixtures | ✓ VERIFIED | Profile field present; delta-only once + empty bounded retry tests exist and pass |
| `crates/codegen/xai-grok-sampling-types/src/conversation.rs` | System→instructions; store-false id strip | ✓ VERIFIED | Conversion + `strip_input_item_ids_for_store_false` + system-role tests |
| `crates/codegen/xai-grok-sampling-types/src/error.rs` | Shared encrypted-content 400 predicate | ✓ VERIFIED | `is_encrypted_content_error` + unit coverage |
| `crates/codegen/xai-grok-shell/src/session/acp_session_impl/sampler_turn.rs` | Trust gate, profile elect, bum headers | ✓ VERIFIED | `codex_session_oauth`, reserved header strip, originator/session/thread insert, profile elect |
| `crates/codegen/xai-grok-shell/src/session/acp_session_impl/model_switch.rs` | Encrypted-reasoning transition sanitizer | ✓ VERIFIED | `clear_encrypted_reasoning`, late-response sanitize, switch unit tests |
| `crates/codegen/xai-grok-shell/tests/model_switch_gate.rs` | Header isolation + encrypted-400 gate | ✓ VERIFIED | Gate suite lists trusted headers, no-leak, trusted→untrusted, encrypted terminal, held late response |
| `crates/codegen/xai-grok-shell/src/agent/config.rs` | Production `SamplerConfig` literal + first-party URL | ✓ VERIFIED | `responses_wire_profile: Disabled`; `is_first_party_codex_url` |
| Shell fixtures (`tests/common`, `provider_routing`, cancel/auth/subagent) | Explicit disabled profile | ✓ VERIFIED | Grep shows Disabled at known literals |
| `.planning/phases/10-codex-responses-wire-parity/10-VALIDATION.md` | Automated + live evidence | ✓ VERIFIED | 26/26 automated PASS; live OPS-04/OPS-05 PASS; honesty rules; deferred fmt |
| `.planning/research/CODEX-RESPONSES-WIRE.md` | Wire contract documentation | ✓ VERIFIED | store/stream/headers/summary/`store:false` id rules documented |

### Key Link Verification

| From | To | Via | Status | Details |
| ---- | -- | --- | ------ | ------- |
| Session-owned trusted profile | Responses HTTP body | `SamplerConfig.responses_wire_profile` → client defaults → `apply_response_defaults` / stream defaults | ✓ WIRED | TrustedCodex only after shell elect; generic conversion first |
| Codex SessionToken OAuth trust gate | Profile + extra_headers | `codex_session_oauth` + `is_first_party_codex_url` in `reconstruct_full_config` | ✓ WIRED | Provider Codex + SessionToken + first-party URL required |
| Reserved identity header strip | Untrusted routes | `extra_headers.retain(!is_reserved_codex_identity_header)` then trusted reinsert | ✓ WIRED | Prevents smuggling; tests rewrite `codex_cli_rs` collisions |
| `ResponseOutputTextDelta` | Assistant text on empty terminal | `text_acc` → `inject_streaming_text_fallback` on completed | ✓ WIRED | Fallback only when terminal empty; terminal wins otherwise |
| Provider transition | Next request history | `handle_set_session_model` → `sanitize_provider_history` / late-response barrier | ✓ WIRED | Encrypted field clear only; no broad `strip_reasoning_blocks` on switch |
| Encrypted-content 400 | Terminal recovery | `is_encrypted_content_error` from sampling-types in shell failure path | ✓ WIRED | Shared predicate; gate proves no compaction/resubmit loop |
| Serialized body `store:false` | Input item ids removed | `strip_input_item_ids_for_store_false` in client send paths | ✓ WIRED | Prevents `rs_*` 404 after provider switch |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| -------- | ------------- | ------ | ------------------ | ------ |
| Trusted request body | `CreateResponse` after profile | Conversation conversion + `apply_trusted_codex_response_profile` | Real history + tools; store false; instructions from system | ✓ FLOWING |
| Identity headers | `extra_headers` map | Session UUID from `provider_transition.trusted_codex_request_id` | Real UUID strings; originator literal `"bum"` | ✓ FLOWING |
| SSE assistant text | `text_acc` → conversation items | Stream events + terminal response | Live deltas or terminal; not hardcoded empty | ✓ FLOWING |
| History after switch | Conversation items | Actor sanitize clears only `encrypted_content` | Ordinary text/tool history retained | ✓ FLOWING |
| Error classification | HTTP 400 message | `is_encrypted_content_error(status, message)` | Message-driven, not stub true | ✓ FLOWING |

### Behavioral Spot-Checks

Verifier re-ran a focused sample (2026-07-18); not the full 26-command chain (already recorded in `10-VALIDATION.md`).

| Behavior | Command | Result | Status |
| -------- | ------- | ------ | ------ |
| Profile default Disabled | `cargo test -p xai-grok-sampler --lib client::tests::responses_wire_profile_defaults_and_propagates_to_client_defaults -- --exact` | 1 ok | ✓ PASS |
| Trusted body serialization | `… trusted_codex_responses_profile_on_off_serializes_exactly -- --exact` | 1 ok | ✓ PASS |
| No system role in input | `cargo test -p xai-grok-sampling-types --lib conversation::tests::responses_api_never_emits_system_role_in_input -- --exact` | 1 ok | ✓ PASS |
| store=false id strip | `… strip_input_item_ids_for_store_false_removes_ids_only_when_store_false -- --exact` | 1 ok | ✓ PASS |
| Encrypted-content predicate | `… encrypted_content_error_accepts_spaced_case_insensitive_400_only -- --exact` | 1 ok | ✓ PASS |
| SSE delta fallback | `… text_delta_then_completed_uses_fallback_when_terminal_empty -- --exact` | 1 ok | ✓ PASS |
| Actor no retry-storm | `cargo test -p xai-grok-sampler --test test_actor delta_only_terminal_empty_completes_once_without_retry -- --exact` | 1 ok | ✓ PASS |
| Cross-provider sanitize | `cargo test -p xai-grok-shell --lib …cross_provider_transition_sanitizes_existing_and_late_reasoning -- --exact` | 1 ok | ✓ PASS |
| Reconstruct profile+headers | `… trusted_codex_reconstruct_enables_profile_and_metadata -- --exact` | 1 ok | ✓ PASS |
| Encrypted 400 terminal class | `… encrypted_content_400_is_classified_terminal -- --exact` | 1 ok | ✓ PASS |
| Trusted headers gate | `cargo test -p xai-grok-shell --test model_switch_gate trusted_codex_wire_headers_are_sent_and_stable -- --exact` | 1 ok (~32s) | ✓ PASS |
| Consolidated suite (prior) | `10-VALIDATION.md` Plan 10-05 Task 1 | 26/26 behavior PASS | ✓ PASS (recorded) |
| Live OPS-04/OPS-05 (prior) | Operator dual-login on rebuilt `bum` | PASS / PASS | ✓ PASS (recorded) |

### Probe Execution

| Probe | Command | Result | Status |
| ----- | ------- | ------ | ------ |
| N/A | Phase uses focused `cargo test -p …` commands, not `scripts/*/tests/probe-*.sh` | — | SKIP (no conventional probes) |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| ----------- | ---------- | ----------- | ------ | -------- |
| **OPS-04** (deepen) | 10-01, 02, 04, 05, 06, 07 | GPT-5.6 coding session after Codex login stays productive (wire parity + once-complete turns) | ✓ SATISFIED (phase deepen) | Trusted wire profile + headers + SSE recovery + automated 26/26 + live OPS-04 PASS |
| **OPS-05** (deepen) | 10-03, 04, 05 | Grok↔GPT-5.6 same-session switch without restart / encrypted 400 / store-false 404 | ✓ SATISFIED (phase deepen) | Encrypted sanitation + id strip + encrypted-400 terminal + live OPS-05 PASS |

**Note:** `.planning/REQUIREMENTS.md` still lists OPS-04/OPS-05 as Phase 9 checklist items with unchecked boxes — that is the parent daily-driver gate ownership, not a missing Phase 10 implementation. Phase 10’s deepen contract is met; formal REQ checkbox closure remains with Phase 9 hybrid green.

**ROADMAP staleness (info):** Phase 10 plans line still says “human live OPS checkpoint still open”; `10-VALIDATION.md` + `10-05-SUMMARY.md` now record live PASS. Update ROADMAP/STATE on phase close (orchestrator).

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| ---- | ---- | ------- | -------- | ------ |
| — | — | No `TBD`/`FIXME`/`XXX` in phase-critical wire sources scanned | — | — |
| — | — | No production `originator: codex_cli_rs` | — | Test-only collision fixtures intentionally use it |
| — | — | Workspace `cargo fmt --check` FAIL | ⚠️ Warning (deferred) | Hygiene only; not goal-blocking |

### Human Verification Required

None pending for phase close.

Live dual-login OPS-04 and OPS-05 were already completed and recorded as **PASS** in `10-VALIDATION.md` §Live OPS evidence (operator 2026-07-18, post store=false id-strip fix). Per verification instructions, those items are treated as human verification already satisfied and are not re-opened.

### Gaps Summary

No goal-blocking gaps. All five ROADMAP success criteria and the supporting plan must_haves that deliver the user-story outcome are present, substantive, wired, and behaviorally evidenced (automated + live).

**Non-blocking deferred:** workspace-wide rustfmt check (`nyquist_compliant: false` in VALIDATION remains correct for hygiene).

**Confirmation-bias counter notes (non-gaps):**
1. Generic/Disabled profile still forces `reasoning.summary = "concise"` — intentional; only TrustedCodex omits (SC3 scoped to Codex path).
2. REQUIREMENTS.md checkboxes for OPS-04/05 remain open under Phase 9 ownership — Phase 10 deepen is satisfied without claiming full Phase 9 hybrid close.
3. Live OPS-05 used `gpt-5.6-luna` (not necessarily `sol`) — still GPT-5.6 Codex family; acceptance criteria met.

---

_Verified: 2026-07-18T14:58:27Z_  
_Verifier: Claude (gsd-verifier)_
