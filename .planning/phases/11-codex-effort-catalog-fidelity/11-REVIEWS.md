---
phase: 11
reviewers: [codex]
reviewed_at: 2026-07-21T09:44:18.493Z
plans_reviewed:
  - .planning/phases/11-codex-effort-catalog-fidelity/11-01-PLAN.md
  - .planning/phases/11-codex-effort-catalog-fidelity/11-02-PLAN.md
reviewer_config:
  codex:
    model: gpt-5.6-sol
    reasoning_effort: high
    service_tier: standard
    sandbox: read-only
    cli_version: codex-cli 0.144.5
source_grounding: true
---

# Cross-AI Plan Review — Phase 11

Single-reviewer run (`--codex` explicitly requested). Codex reviewed with full repo read access
(`-s read-only`, `--skip-git-repo-check`, cwd = repo root) and was instructed to verify every
claim against source rather than review plan text in isolation.

## Codex Review

Overall verdict: the architecture is directionally sound, but both plans need revision before execution. Plan 11-01 misses the initial-session production path; Plan 11-02 conflates raw preference, catalog default, and effective effort, producing incorrect sticky behavior and an empty-menu metadata bug.

### 11-01-PLAN.md

#### Summary

The plan chooses the correct final wire choke point and ports official Codex's middle-of-list clamp accurately. However, it does not actually cover every Codex request: initial session creation builds chat-state sampling configuration through a separate production path that the plan's compile-ripple instructions would incorrectly initialize to `None`. Its summary behavior is also provider-driven rather than catalog-driven.

#### Strengths

- The clamp algorithm matches official Codex: keep a supported value; otherwise use index `(len - 1) / 2`, then the model default. See `codex/codex-rs/core/src/session/turn_context.rs:243`.
- `From<&ConversationRequest> for CreateResponse` is the correct final serialization choke point. All Responses payloads currently place effort and summary there at `crates/codegen/xai-grok-sampling-types/src/conversation.rs:2104`, with the existing unconditional fields at `conversation.rs:2159`.
- `None` versus `Some(empty)` is a useful representation for separating legacy/non-Codex behavior from a catalog that explicitly has no supported levels.
- The proposed regression tests preserve the existing distinction between trusted Codex, which clears summary at `crates/codegen/xai-grok-sampler/src/client.rs:1242`, and the generic legacy path, whose current fixture expects `"concise"` at `client.rs:2493` (verified exact at `client.rs:2505`).
- Cloning a four-element effort list per configuration/request is negligible and introduces no meaningful performance or security concern.

#### Concerns

- **HIGH — Initial Codex sessions bypass the planned production threading.** Initial session creation directly builds `xai_grok_sampling_types::SamplingConfig` at `crates/codegen/xai-grok-shell/src/session/acp_session_impl/spawn.rs:404`. It does not call `apply_prepared_sampling_to_chat_state_fields`. Following the plan's blanket instruction to add `reasoning_effort_supported: None` to "other" literals would make initial Codex sessions lose the supported list. Only model-switched sessions would receive it through `crates/codegen/xai-grok-shell/src/agent/config.rs:4687`. Consequently, the request builder at `crates/codegen/xai-chat-state/src/actor/request_builder.rs:128` would see `None` and preserve legacy, unclamped behavior.
- **MEDIUM — Summary omission is not catalog-driven.** `ModelInfo` has effort/default/menu fields but no reasoning-summary capability or default at `crates/codegen/xai-grok-shell/src/agent/config.rs:3895`. The embedded GPT-5.6 entries likewise advertise only effort data at `crates/codegen/xai-grok-models/default_models.json:28`. Using `reasoning_effort_supported.is_some()` as a proxy for `default_reasoning_summary == none` silently forces summary omission for every Codex-provider entry, including custom/BYOK Codex models whose summary preference is unknown.
- **MEDIUM — No integrated initial-session wire test catches the missing path.** The planned tests cover conversion, generic client serialization, catalog population, and later model switching, but not "start directly on GPT-5.6 and send the first turn." That is exactly the route through `spawn.rs` where the threading breaks.
- **LOW — The threat-model membership invariant is overstated.** The helper can return `catalog_default` when the supported list is empty, so it does not always return `None` or a member of `supported`. Production callers may avoid that branch, but the stated invariant and mitigation should reflect the actual function contract.

#### Suggestions

- Add `spawn.rs:404` to `files_modified` and explicitly copy: `reasoning_effort_supported: sampling_config.reasoning_effort_supported.clone()`.
- Audit production literals individually. Do not mechanically assign `None` where the literal transforms a live model configuration into chat state.
- Add an integration test that starts directly on a Codex model, sends a turn, and asserts the actual mock-server body contains the clamped effort and omitted summary.
- Either introduce explicit catalog metadata such as `default_reasoning_summary`, or narrow the plan's claim to the current GPT-5.6 family. Do not describe provider inference as catalog alignment.
- Correct the threat-model invariant to permit the catalog-default fallback explicitly.

#### Risk Assessment

**HIGH as written.** The wire implementation itself is sensible, but the omitted initial-session path means the central claim — "every Codex Responses request" — is false. With that path fixed, risk drops to MEDIUM, primarily because summary behavior remains implicit.

---

### 11-02-PLAN.md

#### Summary

The plan correctly makes the shell response authoritative and reuses established TUI notification infrastructure, but its state model is not correct. It treats the target model's catalog default as a user preference, cannot distinguish an intentional resolved `None` from legacy missing metadata, and derives notice details from a different/fallback-applied catalog view. These issues directly violate several must-haves.

#### Strengths

- Returning resolved effort through `SetSessionModelResponse.meta` is better than the current client echo. The pager currently discards the response at `crates/codegen/xai-grok-pager/src/app/effects/mod.rs:1681` and reports its requested value at `effects/mod.rs:1716`.
- The plan correctly identifies the existing reset point: the shell currently starts from the target model configuration at `crates/codegen/xai-grok-shell/src/agent/handlers/model_switch.rs:162` and stores the resulting effort at `model_switch.rs:256` (verified exact at line 258).
- Reusing `RenderBlock::system` is consistent with existing successful-switch notices at `crates/codegen/xai-grok-pager/src/app/dispatch/session/lifecycle.rs:1211`.
- The proposed behavior and TUI test tiers target the appropriate integration suites.

#### Concerns

- **HIGH — Raw preference and catalog default are conflated.** The plan says `applied_effort` becomes the raw `stored_preference`, but if `stored_preference` is `None`, it deliberately leaves the target catalog default in `prepared.sampler_config.reasoning_effort`. That default originates at `crates/codegen/xai-grok-shell/src/agent/config.rs:5293`. Assigning `handle.reasoning_effort = applied_effort` therefore turns an implicit default into an explicit sticky preference. Example: no preference → switch to Sol (`low`) → switch to Terra; Terra incorrectly receives sticky `low` instead of its catalog default `medium`.
- **HIGH — The existing handle already stores effective/default effort, not a raw preference.** New session handles initialize `reasoning_effort` directly from sampling configuration at `crates/codegen/xai-grok-shell/src/session/acp_session_impl/spawn.rs:1613`. Reusing that field cannot distinguish user choice from a catalog default. The proposed "single authoritative raw preference" semantics require a state-model change, not merely different switch-handler assignments.
- **HIGH — Intentional `None` is lost in ACP response parsing.** For an empty supported list, serializing `"reasoning_effort": effective_effort.map(...)` produces JSON `null`. The proposed pager parser treats null exactly like a missing/unparseable key and falls back to the client-requested effort. Thus the shell omits effort, while the pager displays and persists the unsupported raw value. The current request/response seam is at `crates/codegen/xai-grok-pager/src/app/effects/mod.rs:1653`.
- **MEDIUM — The empty-menu "soft-skip" claim contradicts current TUI behavior.** Empty or unusable `reasoningEfforts` parses as `None` at `crates/codegen/xai-grok-sampling-types/src/types.rs:987`, after which the pager falls back to the legacy menu whenever `supportsReasoningEffort` is true at `crates/codegen/xai-grok-pager/src/acp/model_state.rs:190`. The shell explicitly omits empty effort arrays while retaining the support flag at `crates/codegen/xai-grok-shell/src/agent/config.rs:5485` (verified exact — `if !info.reasoning_efforts.is_empty()` gates the key insertion; `supportsReasoningEffort` is inserted independently). Therefore an empty catalog does not presently soft-skip.
- **MEDIUM — Clamping mutates persisted preference indirectly.** The lifecycle persists `resolved_effort` at `crates/codegen/xai-grok-pager/src/app/dispatch/session/lifecycle.rs:1219` (verified exact), and the persistence layer writes any `Some` effort into the global default at `crates/codegen/xai-grok-shell/src/util/config/campaigns.rs:393`. Persisting the clamped effective value overwrites the user's raw choice across restarts, contrary to the stated prohibition.
- **MEDIUM — The notice is not request-build-time or atomic with the actual clamp.** The plan recomputes a display-only clamp during model switching, while correctness clamps later during request conversion. It then reconstructs the supported list from the pager's fallback-applied model state. Those are separate catalog views at separate times, so "level and supported list come atomically from the same entry" is not established.
- **LOW — Some proposed integration assertions are not directly available through the current harness.** `GateHarness` retains the ACP client and mock servers, while `MvpAgent` is moved into the agent connection during setup at `crates/codegen/xai-grok-shell/tests/model_switch_gate.rs:499` (verified — `MvpAgent::new(...)` at line 503 is consumed by `acp::AgentSideConnection::new(agent, ...)` two lines later; `GateHarness`'s own fields hold only `client`/mock servers, no agent/handle reference). Tests cannot directly inspect `SessionHandle.reasoning_effort` without additional instrumentation; they should assert via response metadata, round-trip switches, persisted summary, or actual request bodies.

#### Suggestions

- Introduce an explicit raw preference field, separate from effective sampling effort, for example: `reasoning_effort_preference: Option<ReasoningEffort>`. Initialize it only from explicit user/session input, not `ModelInfo.reasoning_effort`.
- Keep three values distinct in the switch handler: (1) raw user preference, (2) target catalog default/request candidate, (3) post-clamp effective wire effort.
- Preserve intentional null metadata. Fall back to the client value only when the response key is absent for legacy compatibility; `key present + null` must resolve to `None`.
- Return the exact supported list — or a fully formed clamp-notice payload — in the ACP response. Do not reconstruct it through the pager's legacy fallback.
- Decide and document empty-menu semantics, then change both ACP metadata emission and pager parsing so absent and explicitly empty menus are distinguishable.
- Add tests for: no preference (Sol → Terra produces Terra's `medium`, not sticky Sol `low`); explicit preference survives Grok → Codex → Grok; empty menu returns JSON null and leaves pager effort unset; restart/resume preserves the raw preference rather than the clamped value; start directly on Codex, not only switch into it.

#### Risk Assessment

**HIGH.** The raw/effective state conflation and null-response fallback can produce incorrect model behavior and misleading UI state even when the wire clamp itself is correct. The plan should be redesigned around explicit preference identity before implementation.

---

## Consensus Summary

Only one external reviewer ran this cycle (`--codex`, per explicit request), so there is no
cross-model agreement/divergence to synthesize. What follows is the single-reviewer risk
picture, cross-checked independently against source during the grounding pass below (see
"Additional spot-verification of reviewer findings").

### Top concerns (both plans HIGH risk as written)

1. **11-01 misses the initial-session (non-switch) code path** (`spawn.rs:404`) — the single
   choke point the plan targets (`conversation.rs:2159`) is real and correctly identified, but
   the field that feeds it (`reasoning_effort_supported`) never reaches a session that starts
   directly on a Codex model rather than switching into one. Independently confirmed: `spawn.rs`
   builds its own `xai_grok_sampling_types::SamplingConfig { ... }` literal that does not route
   through `apply_prepared_sampling_to_chat_state_fields`, and `spawn.rs` is absent from the
   plan's `files_modified` list.
2. **11-02's sticky-preference model conflates "user chose this" with "catalog gave me this by
   default."** `SessionHandle.reasoning_effort` is seeded from resolved sampling config at
   session creation (`spawn.rs:1613`), not from an explicit-choice-only store, so the plan's
   "carry forward `handle.reasoning_effort`" mechanism will make catalog defaults sticky —
   independently confirmed by reading both the seeding site and the catalog-default
   population site (`config.rs:5293`).
3. **11-02's empty-list wire signal (`null`) is indistinguishable from "key absent"** in the
   proposed pager-side parse, so the omit-on-empty-menu behavior (11-01's correctly-implemented
   choke-point rule) gets silently overridden back to the stale client-requested value on the
   pager side. Independently confirmed the parse/fallback code path exists as described at
   `effects/mod.rs:1653` (today unconditionally discards the response before this plan's change).

### Secondary concerns worth folding into the replan

- Catalog has no `default_reasoning_summary` field — 11-01's summary-omit rule is inferred from
  `reasoning_effort_supported.is_some()` (i.e. "is a Codex model"), not from real catalog data.
  Confirmed: `ModelInfo` (config.rs:3845-3905) carries no summary-related field.
- 11-02's clamp notice and the actual wire clamp read two different catalog views (pager-side
  fallback-applied menu vs. shell-side raw catalog list) at two different times — a real
  atomicity gap, not just a wording nit.
- 11-02's persisted "preferred model" effect writes the *clamped* value back to the global
  default (`campaigns.rs:393`), which will leak an unsupported-model's clamped effort into the
  user's next default-model session.

### What is already solid (do not relitigate in the replan)

- The clamp algorithm itself (index `(len-1)/2`, fallback to catalog default) is a faithful,
  verified port of official Codex (`codex-rs/core/src/session/turn_context.rs:243-263`).
- The chosen choke point (`conversation.rs`'s `From<&ConversationRequest> for rs::CreateResponse`)
  is correctly identified as the single place every *switch-originated* Codex Responses request
  is built — the gap is session-start, not the choke point's location.
- Making the ACP `SetSessionModelResponse.meta` carry the server-resolved effort instead of
  letting the pager echo its own request is the right shape of fix for the "not sticky today"
  bug — the concerns are about what value flows through it, not the mechanism.

## Verification coverage

Source-grounding pass per `plan_review.source_grounding=true` (authority: ripgrep against this
repo's working tree plus the sibling `~/bum/codex` ground-truth checkout the plans themselves
cite). Scope: every file path, function/method, struct/enum/field, and test name cited by
**11-01-PLAN.md** and **11-02-PLAN.md** (`<read_first>`, `<action>`, `<acceptance_criteria>`
blocks), excluding symbols listed under each plan's own "Artifacts this phase produces" section
(new-artifact declarations are not yet in source by definition, not a grounding gap).

**Result: 0 MISSING, 0 AMBIGUOUS, ~48 VERIFIED, 5 classes of symbols explicitly out of scope
(listed below).** No MISSING symbols were found, so `gsd-tools.cjs drift-guard severity --status
MISSING --authority grep` was not invoked (nothing to classify — the needs-acknowledgement path
never triggers on an empty MISSING set).

### VERIFIED — representative sample (all citations checked resolved to the exact or
within-cited-range file:line; every citation attempted returned VERIFIED)

| Plan | Symbol / citation | File:Line | Note |
|------|-------------------|-----------|------|
| 11-01 | `impl FromStr for ReasoningEffort` | `xai-grok-sampling-types/src/types.rs:829-845` | exact bounds match |
| 11-01 | `"xhigh" \| "max" => Ok(Self::Xhigh)` | `types.rs:839` | exact |
| 11-01 | `ReasoningEffortOption{id,value,label,description,default}` | `types.rs:894-900` | exact |
| 11-01 | `SamplingConfig::reasoning_effort` field | `types.rs:1050` | exact |
| 11-01 | `ConversationRequest::reasoning_effort` (before `json_schema`) | `conversation.rs:546,548` | exact |
| 11-01 | `impl From<&ConversationRequest> for rs::CreateResponse` | `conversation.rs:2104` | exact |
| 11-01 | choke-point `reasoning: Some(rs::Reasoning{...})` | `conversation.rs:2159-2161` | exact |
| 11-01 | second `reasoning: Some(rs::Reasoning{...})` occurrence | `conversation.rs:4061` | checked and ruled out — builds `rs::Response` (server echo) inside `#[cfg(test)]`, not `rs::CreateResponse`; not a second production choke point the plan missed |
| 11-01 | `SamplerConfig::reasoning_effort` field | `xai-grok-sampler/src/config.rs:94` | exact |
| 11-01 | `impl Default for SamplerConfig` | `config.rs:150` | exact |
| 11-01 | `ModelProvider{Xai,Codex}` | `xai-grok-shell/src/agent/config.rs:3408-3413` | exact |
| 11-01 | `sampling_config_for_model` / `model.info()` | `config.rs:5260-5267` | exact |
| 11-01 | `apply_prepared_sampling_to_chat_state_fields` literal | `config.rs:4683-4712` | exact |
| 11-01 | `build_conversation_request` literal | `xai-chat-state/src/actor/request_builder.rs:37,128-146` | exact |
| 11-01 | 2 test-only `SamplingConfig{...}` compile-ripple sites | `xai-chat-state/src/types.rs:176-187, 219-229` | exact |
| 11-01 | `apply_trusted_codex_response_profile` summary-clear | `xai-grok-sampler/src/client.rs:1233,1242-1244` | exact |
| 11-01 | `trusted_codex_responses_profile_on_off_serializes_exactly` | `client.rs:2445` | exact |
| 11-01 | `assert!(trusted["reasoning"].get("summary").is_none())` | `client.rs:2475` | exact |
| 11-01 | `assert_eq!(generic["reasoning"]["summary"], "concise")` | `client.rs:2505` | exact |
| 11-01 | `ResponsesWireProfile::Disabled` variant | `sampler/config.rs:34` | exact |
| 11-01 | sol/terra/luna `reasoning_efforts` catalog menus | `xai-grok-models/default_models.json:20-123` | exact — sol default `low`, terra/luna default `medium`, all 4-entry `[low,medium,high,xhigh]` |
| 11-01 | official clamp algorithm (ground truth) | `~/bum/codex/codex-rs/core/src/session/turn_context.rs:243-263` | exact, character-for-character match to RESEARCH.md's quote |
| 11-01 | crate-root re-export enabling `xai_grok_sampling_types::clamp_reasoning_effort` | `xai-grok-sampling-types/src/lib.rs:24` (`pub use self::types::*;`) | confirms 11-02's import path will resolve |
| 11-02 | `use xai_grok_sampling_types::parse_reasoning_effort_meta;` | `xai-grok-shell/src/agent/handlers/model_switch.rs:17` | exact |
| 11-02 | `handle = session_handle_waiting_for_load(...)` | `model_switch.rs:49` | exact |
| 11-02 | effort_override gate block | `model_switch.rs:162-180` | exact |
| 11-02 | `handle.reasoning_effort = applied_effort;` | `model_switch.rs:258` | exact |
| 11-02 | `broadcast_model_changed(...)` | `model_switch.rs:262` | exact |
| 11-02 | `set_current_reasoning_effort(applied_effort)` | `model_switch.rs:281` | exact |
| 11-02 | `Ok(acp::SetSessionModelResponse::new().meta(...))` | `model_switch.rs:283` | exact |
| 11-02 | `SessionHandle.reasoning_effort` field | `xai-grok-shell/src/session/handle.rs:41,108` | exact |
| 11-02 | `model_supports_reasoning_effort` / `model_reasoning_efforts` | `xai-grok-shell/src/agent/models.rs:451,476` | exact |
| 11-02 | `Effect::SwitchModel` handler + response-discard bug | `xai-grok-pager/src/app/effects/mod.rs:1653` | exact — `.map(\|_\| ())` discard confirmed present today |
| 11-02 | `Effect::SwitchModel{...}` / `TaskResult::SwitchModelComplete{...}` fields | `xai-grok-pager/src/app/actions.rs:1521-1537, 2302-2313` | exact — `effort` field at line 2305 |
| 11-02 | sole call site of `handle_switch_model_complete` | `xai-grok-pager/src/app/dispatch/task_result.rs:455-462` | exact — independently grepped whole crate, confirmed only 1 real call site (2 other hits are an import and a doc-comment reference) |
| 11-02 | `handle_switch_model_complete` body + notice block | `.../dispatch/session/lifecycle.rs:1128,1148,1211-1224` | exact |
| 11-02 | `reasoning_effort_options_for` accessor | `xai-grok-pager/src/acp/model_state.rs:193` | exact |
| 11-02 | 09-UAT.md Phase-11-effort references | `09-UAT.md:154,163,283` exact; `:270` topically adjacent (no literal "Phase 11" on that line, cited with `~`) | 3/4 exact, 1/4 approximate as the plan's own `~` hedges |
| 11-02 | `p6_*` test convention / `trusted_codex_wire_headers_are_sent_and_stable` | `model_switch_gate.rs:92-149 (p6_*), 912-961 (dual-login), 1033 (wire headers)` | exact |

*(Full working list ran to ~48 individually-checked citations across both plans; every one
resolved to VERIFIED. None are omitted from the counts above — the table shows a representative
selection rather than all 48 rows for readability.)*

### Additional spot-verification of reviewer findings (not plan citations — Codex's own claims,
independently re-derived from source rather than taken on trust)

Because this cycle has only one external reviewer, the HIGH/MEDIUM findings above were
independently re-grepped rather than accepted at face value:

- `spawn.rs:404` — confirmed a standalone `xai_grok_sampling_types::SamplingConfig { ... }`
  literal exists in the initial-session path, separate from `apply_prepared_sampling_to_chat_state_fields`.
- `spawn.rs:1613` — confirmed `SessionHandle { ..., reasoning_effort: sampling_config.reasoning_effort, ... }`
  seeds the handle from resolved (possibly catalog-default) sampling config at session creation.
- `config.rs:5485` — confirmed `if !info.reasoning_efforts.is_empty() { map.insert(REASONING_EFFORTS_META_KEY, ...) }`
  gates the menu key independently of `supportsReasoningEffort`, i.e. empty-list and absent-list
  collapse to the same wire signal.
- `types.rs:987` / `model_state.rs:190` — confirmed `parse_reasoning_efforts_meta` returns `None`
  for both absent and empty-after-filter menus, and the pager's `reasoning_effort_options_for`
  falls back to `legacy_effort_options` in that case rather than an empty list.
- `campaigns.rs:393` + `effects/mod.rs:1904` + `lifecycle.rs:1218-1221` — confirmed the full
  chain: `Effect::PersistPreferredModel{reasoning_effort: resolved_effort}` (the **clamped**
  value) → `persist_models_default(Some(model), reasoning_effort)`.
- `model_switch_gate.rs:499` (Codex's citation) / `:503` (this pass's exact line) — confirmed
  `MvpAgent::new(...)` is a local binding moved into `acp::AgentSideConnection::new(agent, ...)`;
  `GateHarness`'s own struct fields hold no agent/handle reference, so the LOW finding about test
  assertion reach is accurate.

All six independently re-checked. All six confirmed accurate.

### Explicitly out of scope for this pass (not MISSING — different authority, noted for
completeness)

1. **External `agent_client_protocol` (acp) crate's own API surface** (`acp::ModelId`,
   `acp::SetSessionModelRequest/Response`, `.meta()` builder). Not vendored in-tree; outside this
   repo's grep authority. Not re-derived from the crate's own source — treated as validated by
   the fact that the exact same call shapes already compile in current production code the plans
   modify in place (e.g. `model_switch.rs:283`'s existing `.meta(...)` call).
2. **`11-UI-SPEC.md` locked-copy line citations** (lines 71-79). This is a phase-authored design
   artifact, not implementation source — there is no "drift" risk to grep for in a doc this same
   phase produced. Read in full separately; internally consistent with both plans' TUI-notice
   citations.
3. **RESEARCH.md/CONTEXT.md citations not also cited by the plans themselves.** Task scope is
   "every symbol the plans cite" — RESEARCH.md §B4-B8 and CONTEXT.md are supporting inputs, not
   "the plans." Citations the plans repeat from RESEARCH.md (clamp algorithm, choke point,
   summary-omit line, default_models.json shape) were verified as part of the plan pass above;
   RESEARCH-only citations with no corresponding plan citation (e.g. the full 8-stage "bum effort
   flow end-to-end" pipeline table in §B4) were not separately re-derived.
4. **Compile-time verification (`cargo check --workspace --all-targets`, the plans' own
   acceptance criterion).** Genuinely UNCHECKABLE at plan-review time: no code changes exist yet
   to compile. Expected for a pre-execution plan review, not a grounding gap — this is exactly
   what execution-time verification (`/gsd-execute-phase`) is for.
5. **Per-occurrence classification of the ~34 `SwitchModelComplete` literal sites** across the
   three test files 11-02 names (`task_result.rs` 27, `settings.rs` 6, `prompt.rs` 1 — each file
   confirmed to contain at least one occurrence, matching the plan's "three known files" claim).
   Not classified construction-vs-match-arm per occurrence — the plan itself defers exact
   enumeration to `cargo check`'s own error list rather than claiming a precise manual count, so
   there is no specific claim here to ground beyond "these files exist and contain occurrences,"
   which is confirmed.
