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

---

# CYCLE 2 — Convergence Review

```yaml
cycle: 2
reviewed_at: 2026-07-21T10:28:36.000Z
revision_commit: c389058b3931bdf3ccd16ee03aed3fddb662d7c5
plans_reviewed:
  - .planning/phases/11-codex-effort-catalog-fidelity/11-01-PLAN.md (revised)
  - .planning/phases/11-codex-effort-catalog-fidelity/11-02-PLAN.md (revised)
reviewers:
  codex:
    model: gpt-5.6-sol
    reasoning_effort: high
    service_tier: standard
    sandbox: read-only
    cli_version: codex-cli 0.144.5
    source_grounding: true
  claude-sonnet-5:
    role: orchestrator + independent second grounding pass (this session)
    source_grounding: true
verdict: NOT CONVERGED
```

Cycle 1 (above) raised 4 HIGH + 7 actionable MEDIUM/LOW findings against the original plans.
Commit `c389058` revised both plans specifically to address them (see the diff — 248 lines
changed across the two PLAN.md files only; no source code has been touched yet, this remains a
pre-execution plan review). This cycle checks two things per cycle-1 finding: (a) is the
revision's fix grounded in real source as cited, and (b) does the fix design actually close the
described gap when traced through — not merely whether the plan *claims* resolution. It then
hunts for new issues the revision itself introduced.

Two independent passes were run and cross-checked: Codex CLI (`gpt-5.6-sol`, high reasoning
effort, standard service tier, `-s read-only`, full repo access, `--skip-git-repo-check`, cwd =
repo root) via `codex exec`, and a second, fully independent grounding pass done directly in
this session (grep/read against the live source, performed *before* reading Codex's output).
Both passes independently converged on the same primary blocking gap (finding **NEW-A** below)
without either seeing the other's work first — strong corroborating evidence it is real, not an
artifact of one reviewer's framing.

## Per-finding resolution table (cycle-1 findings vs. this revision)

| # | Cycle-1 finding | Verdict this cycle | Evidence |
|---|---|---|---|
| HIGH#1 | 11-01: initial-session threading gap (`spawn.rs:404`) | **PARTIALLY RESOLVED** | The two sites the revision targets are real and correctly fixed: `spawn.rs:404`'s `chat_state_sampling_config` literal (field `reasoning_effort: sampling_config.reasoning_effort,` confirmed at `spawn.rs:413` exact) and `sampler_turn.rs::reconstruct_full_config` (doc comment + signature confirmed at `sampler_turn.rs:246-250`; fallback literal's `reasoning_effort: None,` confirmed at `sampler_turn.rs:288` exact; final return literal confirmed at `sampler_turn.rs:393`, with `reasoning_effort: cfg.reasoning_effort,` confirmed at `sampler_turn.rs:406` exact — plan's citation is exact down to the line). But a **third** production construction site is not enumerated by either plan and is not covered by the mechanical `cargo check` compile-ripple sweep's safe-default reasoning — see **NEW-A** below. The plan's own success criterion ("every Codex Responses request built through ANY real production path") is therefore still not true once cross-provider subagents are considered. |
| HIGH#2 | 11-02: raw preference conflated with catalog default (`handle.reasoning_effort = applied_effort`) | **PARTIALLY RESOLVED** | The new field is genuinely new (current `SessionHandle` has only `reasoning_effort` at `handle.rs:108`, confirmed exact; `#[derive(Clone)]` confirmed at `handle.rs:40`). The one production `SessionHandle {...}` literal is confirmed fully exhaustive at `spawn.rs:1594-1631` (no `..Default::default()`), with the field to seed confirmed at `spawn.rs:1613` exact. The plan's control-flow premise — an early immutable snapshot (`session_handle_waiting_for_load`, ~`model_switch.rs:50`) distinct from a later mutable re-fetch (`agent.sessions.borrow_mut().get_mut(&session_id)`, confirmed exact at `model_switch.rs:256`) — is real and mechanically sound for the interactive-switch path. But the session **resume/restore** path reaches the same `model_switch::apply` through a different door that reintroduces the exact conflation this finding was about — see **NEW-B** below. |
| HIGH#3 | 11-02: existing handle already stores effective/default, not raw preference | **PARTIALLY RESOLVED** | Same evidence and same caveat as HIGH#2 — the fresh, uninterrupted Sol→Terra repro (`11-02-PLAN.md`'s `p11_no_preference_switch_uses_target_catalog_default_not_prior_model_default`) is correctly designed and would pass. Resume reintroduces stickiness via **NEW-B**. |
| HIGH#4 | 11-02: null-vs-absent ACP response parsing | **RESOLVED** | `resolve_switch_model_response_effort`'s described 3-way match is on `meta.and_then(\|m\| m.get("reasoning_effort"))` — an `Option<&Value>` — matched *before* any `.as_str()` chaining, so `Some(&Value::Null)` stays distinguishable from `None` (a `.and_then(|v| v.as_str())`-first design would have silently re-collapsed the two, since `.as_str()` on `Value::Null` also yields `None` — the plan's described match order avoids this footgun). Today's discard/fallback bug is confirmed real at `effects/mod.rs:1653` (`.map(\|_\| ())` before the `TaskResult::SwitchModelComplete` construction). |
| M1 | 11-01: summary omission not catalog-driven | **RESOLVED** | The 3-hop catalog template is real and matches exactly: `DefaultModelJson` (`config.rs:3437-3468`, no `pub`, confirmed), `default_models()` mapping (`config.rs:3520-3535` region, confirmed), `ModelEntryConfig` field pair (`config.rs:3584-3588`, confirmed with matching serde attrs), `ModelInfo` field pair (`config.rs:3893-3903`, confirmed), final conversion (`config.rs:3975-3985`, confirmed). `ConfigModelOverride` (`config.rs:3681`) confirmed to exist as the correctly-out-of-scope override layer, with its sibling-field participation pattern confirmed real at `config.rs:3798-3812`. The choke point's proposed `summary` branch (`if req.reasoning_summary_omit {...}`) is independent of `reasoning_effort_supported`; the trusted-Codex path's independent unconditional summary-clear is confirmed unchanged at `client.rs:1233-1257`. |
| M2 | 11-01: no integrated initial-session wire test | **PARTIALLY RESOLVED** | The test is added and the harness region is real and reusable. But the test is not regression-sensitive as specified: it asserts `reasoning.effort == "low"` with *no override applied* against Sol, whose catalog `reasoning_effort` default is independently confirmed `"low"` in `default_models.json` — that assertion would pass identically even if `reasoning_effort_supported` were never threaded to `spawn.rs` at all, because it is simply Sol's pre-existing unclamped default. Likewise `reasoning.summary` absent is expected to route through the Trusted-Codex profile, which already unconditionally clears summary today (`client.rs:1242`) independent of the new `reasoning_summary_omit` flag. The test as specified could pass on the pre-revision (buggy) code. Needs either an unsupported initial effort (to force a real clamp) or a generic/non-trusted Codex route (to isolate the new summary-omit flag from the trusted path's pre-existing unconditional clear). |
| M3 | 11-02: "picker phase soft-skips" claim was wrong | **RESOLVED** | Both new citations resolve to the *exact executable line* implementing the described collapse, not just a signature/doc-comment: `types.rs:1003` is `(!options.is_empty()).then_some(options)` inside `parse_reasoning_efforts_meta` (confirmed exact — this is the literal collapse mechanism); `model_state.rs:204` is `parse_reasoning_efforts_meta(info.meta.as_ref()).unwrap_or_else(legacy_effort_options)` (confirmed exact). Both citations are more precise than cycle 1's own (which cited the doc-comment/signature lines) — an improvement, not a regression. |
| M4 | 11-02: clamping mutates persisted preference indirectly | **RESOLVED** | `campaigns.rs:392-410`'s `persist_models_default` body confirmed character-for-character: `cfg.models.default = if s.is_empty() { None } else { Some(s) }` (the `value` param genuinely clears), but `if let Some(effort) = reasoning_effort { cfg.models.default_reasoning_effort = Some(effort); }` has **no else branch** — passing `reasoning_effort: None` is confirmed a true no-op/skip, not a clear, despite the doc comment's stale blanket claim "`None` clears the field." This is exactly the semantics the M4 fix (`reasoning_effort: if effort_clamped { None } else { resolved_effort }`) depends on being true. |
| M5 | 11-02: clamp notice not atomic with the actual clamp | **NOT RESOLVED** | The plan does thread one `supported` binding into both the clamp-display decision and the new response-meta key (confirmed: `model_switch.rs`'s described `supported` local is used for both `effective_effort`/`effort_clamped` computation and the `"reasoning_effort_supported"` meta key — same variable, same statement, genuinely not two separately-computed values as before). But `agent.models_manager.model_reasoning_efforts(model_id.0.as_ref())` (confirmed at `models.rs:476-482`) is a **direct map-key lookup** keyed on `model_id`, whereas the actual wire-side sampling config is built by resolving the model through alias/routing-slug logic (confirmed a *separate* resolution path exists at `agent_ops.rs:1057-1082`, which clones a resolved `ModelEntry` rather than doing a raw key lookup). If the model is addressed by a routing slug/alias that differs from its catalog map key, `model_reasoning_efforts` can silently return `vec![]` while the actual `sampling_config_for_model` call (which operates on the already-resolved `ModelEntry`) sees the real, non-empty list — reproducing a version of the original atomicity gap through a different mechanism (alias-resolution mismatch instead of pager-side re-derivation). |
| L1 | 11-01: threat-model invariant overstated | **RESOLVED** | Corrected wording at `11-01-PLAN.md`'s threat table now names all three sources (`None`, a `supported` member, or `catalog_default` when the list is empty), matching the helper's actual contract in Task 1's action text and the official `turn_context.rs:243-263` algorithm it ports. |
| L2 | 11-02: GateHarness doesn't expose SessionHandle | **RESOLVED** | Confirmed exact: `GateHarness::set_model(...) -> Result<acp::SetSessionModelResponse, acp::Error>` (`model_switch_gate.rs:611-620`) returns the full ACP response object, including `.meta`, on every call. The redirected test design (assert via response metadata, never `SessionHandle` internals) is genuinely implementable as specified. `GateHarness`'s own fields hold no agent/handle reference, confirmed at `model_switch_gate.rs:499-505` (`MvpAgent::new(...)` is a local binding moved into `AgentSideConnection::new`, not stored on the harness). |

## New concerns introduced by the revision

- **HIGH — NEW-A: a third production construction site silently drops the new catalog fields
  (recurrence of HIGH#1's exact bug class).** `crates/codegen/xai-grok-shell/src/agent/subagent/mod.rs:929-956`
  (`read_parent_sampling_config`, called from `resolve_subagent_sampling_config` at `mod.rs:789`
  — the default `"inherit_parent"` behavior for any subagent spawned without an explicit
  per-agent model override — and directly from `handle_request.rs:746` as a resolved-model-not-found
  fallback) builds a **fully exhaustive** `xai_grok_sampler::SamplerConfig { ... }` literal (every
  field of the current struct definition present, confirmed field-by-field against
  `xai-grok-sampler/src/config.rs:64-149`; no `..Default::default()` or other rest pattern) from
  the parent session's live `cfg: xai_grok_sampling_types::SamplingConfig` (obtained via the same
  `chat_state.get_sampling_config().await` accessor `sampler_turn.rs::reconstruct_full_config`
  uses). It already faithfully copies `reasoning_effort: cfg.reasoning_effort,` (confirmed at
  `subagent/mod.rs:942`) from the parent — proving the surrounding code cares about carrying the
  parent's real sampling state forward, not blanking it — but once Task 1 adds
  `reasoning_effort_supported`/`reasoning_summary_omit` to both types, this literal will hit a
  "missing field" compile error like `spawn.rs`/`sampler_turn.rs` did before their explicit fixes.
  Neither plan's `files_modified`/`read_first`/`action` mentions `agent/subagent/mod.rs` (only
  the unrelated *test* file `agent/subagent/tests/mod.rs` is touched, for the `SessionHandle`
  field, a different struct). If an executor follows Task 1's generic compile-ripple instruction
  ("add `reasoning_effort_supported: None, reasoning_summary_omit: false,` at every resulting
  missing-field site") **literally and uniformly**, this specific site would get the wrong
  values — a real Codex-model subagent spawned by inheritance would silently fall back to legacy
  unclamped/summary-always-sent behavior, exactly the class of bug HIGH#1 was raised to close, now
  at the cross-provider-subagent path (PROJECT.md validates this as delivered, actively-used
  functionality — Phase 7, "Parent/child can use different providers"). The correct fix mirrors
  `spawn.rs`/`sampler_turn.rs`: copy `cfg.reasoning_effort_supported.clone()` /
  `cfg.reasoning_summary_omit` from the parent's live config, not blank-fill. Independently found
  by both reviewers in this cycle before either saw the other's output.

- **HIGH — NEW-B: session resume/restore reintroduces the raw-preference/catalog-default
  conflation via a different path than the one the revision fixed.** Fresh-session creation
  persists whatever `sampling_config.reasoning_effort` happens to be — including a bare catalog
  default the user never chose — to disk: `agent_ops.rs:3514-3516`
  (`let initial_reasoning_effort = chat_history.is_empty().then_some(sampling_config.reasoning_effort);`
  then sent via `PersistenceMsg::CurrentModel { ..., reasoning_effort: initial_reasoning_effort, ... }`,
  confirmed exact). On resume, `load_session` reads that persisted value back
  (`summary.reasoning_effort`) and feeds it into `model_switch::apply` as if it were an
  ACP-request-supplied `effort_override`: `acp_agent.rs:1868-1885` builds a `restore_meta` ACP
  `Meta` map keyed on `REASONING_EFFORT_META_KEY` from the persisted value and calls
  `crate::agent::handlers::model_switch::apply(self, acp::SetSessionModelRequest::new(session_id, model_id).meta(restore_meta)).await`
  — confirmed exact, this is the *same* `apply()` the revision modifies. Under the revised logic
  (`stored_preference = effort_override.or(handle.reasoning_effort_preference)`,
  `11-02-PLAN.md:172`), a persisted-but-never-user-chosen value parses into a non-`None`
  `effort_override` and becomes the new `stored_preference`, which then gets written into
  `handle.reasoning_effort_preference` at the tail of `apply()` — i.e. resuming a session that
  merely got Sol's `low` default at creation (no explicit user choice ever made) now records
  `low` as an explicit "raw preference" in the new dedicated field, and a subsequent no-override
  switch to Terra would incorrectly inherit sticky `low` instead of Terra's own `medium` default
  — the exact HIGH#2/HIGH#3 repro, reopened via resume. Neither plan's `read_first`/`action`
  addresses session restore/resume at all. Preference provenance needs to be persisted
  separately (or restore needs to feed the persisted value through a non-`effort_override`,
  non-user-provenance path).

- **HIGH — NEW-C: deleting the `model_supports_reasoning_effort` gate entirely lets a stored
  effort preference reach non-Codex wire requests unconditionally, including model variants that
  reject the parameter outright.** Confirmed today's (pre-phase-11, unmodified) code at
  `model_switch.rs:162-179` gates `effort_override` behind
  `agent.models_manager.model_supports_reasoning_effort(model_id.0.as_ref())` before ever
  assigning it to `prepared.sampler_config.reasoning_effort` — an unsupported override is
  silently dropped with a `tracing::warn!`, never reaching the wire. `11-02-PLAN.md`'s revised
  Task 1 (`11-02-PLAN.md:172-176`) explicitly deletes this gate with no replacement check:
  `stored_preference = effort_override.or(handle.reasoning_effort_preference)` is stamped into
  `prepared.sampler_config.reasoning_effort` for **any** target model, Codex or not. For a
  Codex target this is intentional and correct (11-01's choke point clamps it against the
  catalog menu). For a target where `reasoning_effort_supported == None` (every current Grok/xAI
  model, per 11-01's own semantics table at `11-01-PLAN.md:147`), the choke point's `None` branch
  is explicitly "byte-identical to before this phase" — i.e. `req.reasoning_effort.map(...)` is
  sent on the wire completely unguarded. `crates/codegen/xai-grok-shell/src/session/acp_session_impl/laziness.rs`'s
  own comment (near its `ConversationRequest` construction) confirms this is not hypothetical:
  "`grok-4.5` (and other tool-flavoured Grok variants) reject the field at the proxy with `400
  Bad Request: Model does not support parameter reasoningEffort`" when `reasoning_effort` is
  present at all. Concretely: a user sets an explicit Codex effort (e.g. `xhigh`), it becomes
  `handle.reasoning_effort_preference`; a later switch to a Grok model with no explicit override
  now carries that raw `xhigh` straight through to a wire request the target model may hard-reject
  — a regression versus today's gated behavior, not merely an unclamped value. The fix needs a
  target-provider (or target-catalog-support) check before stamping `stored_preference` into the
  request candidate for a non-Codex target — the gate should be removed for Codex targets only,
  not universally.

- **MEDIUM — required-field compile-ripple enumeration is still incomplete (three more real
  sites found beyond the plan's own list).** `#[serde(default)]` only affects JSON
  *deserialization*; it does nothing for a Rust struct literal built directly in code — every
  literal below needs an explicit new-field line or a rest pattern regardless of the serde
  attribute the plan adds. `crates/codegen/xai-grok-shell/src/remote/client.rs:829-895` builds a
  fully field-by-field `ModelEntryConfig { ... }` (confirmed: explicitly parses
  `reasoningEffort`/`supportsReasoningEffort`/`reasoningEfforts` from a *remotely-fetched* model
  list, a genuinely separate live parsing path from the embedded `default_models.json` ->
  `DefaultModelJson` 3-hop template the plan modifies; no rest pattern found through the literal's
  full extent). `crates/codegen/xai-grok-shell/src/agent/config.rs:5144-5150`-ish builds a
  fallback/synthetic `ModelInfo { ... }` for a bare-bearer-token session with no catalog match.
  `crates/codegen/xai-grok-shell/src/agent/mvp_agent/tests.rs:1131` (`make_test_handle`) is a
  **second** test-only `SessionHandle { ... }` literal, contradicting 11-02-PLAN.md's claim that
  `agent/subagent/tests/mod.rs:1080` is "the ONE test-only `SessionHandle` literal." Lower
  practical risk than NEW-A/B/C: all three are exhaustive literals that `cargo check
  --workspace --all-targets` (the plans' own hard acceptance gate) will force an executor to
  touch regardless of whether the plan names them, and the semantically-safe blank default
  (`None`/`false`) is very likely the objectively correct value at all three (none of them are a
  "silently carry forward the parent's real value" site like NEW-A). Worth correcting in the
  plan's own file inventory for documentation accuracy, not a functional blocker on its own.

- **MEDIUM — the two sibling plans now contradict each other on empty-menu picker behavior.**
  `11-01-PLAN.md`'s `must_haves.truths` still asserts (unchanged by the revision, confirmed —
  this line was not touched by the `c389058` diff): "the effort menu soft-skips" for a model
  advertising an empty/absent supported-effort list. `11-02-PLAN.md`'s revision adds a
  "CORRECTED CLAIM (review finding M3)" explicitly stating the opposite is true today: an empty
  catalog array does **not** soft-skip, it falls back to the legacy 4-level menu (verified above,
  M3 RESOLVED). 11-01's own copy needs the same correction; as written the two plans assert
  contradictory picker behavior for the identical case.

- **LOW — clamp-notice timing wording is imprecise.** `11-02-PLAN.md`'s must-have says the
  notice is "emitted synchronously at request-build from embedded catalog data," but the action
  text computes the display clamp and renders the notice during model-*switch* handling
  (`model_switch.rs`'s mirrored display clamp, rendered by `lifecycle.rs` after the switch
  response returns) — not at the point a request is actually built/sent. Documentation
  precision only; does not affect the design's correctness.

A note on the shared-name concern flagged in the review prompt: the ACP response-meta JSON key
`"reasoning_effort_supported"` (a string array) and 11-01's internal Rust field of the same name
(`Option<Vec<ReasoningEffort>>`) are sufficiently distinguished in context — this was checked and
is not a type-confusion risk. The real M5 gap is that 11-02 re-derives the list through a
different (alias-unaware) resolution path than the one 11-01's choke point actually uses, not a
naming collision.

## Verification coverage (cycle 2 — newly cited symbols only)

Scope per task instructions: every symbol **newly cited** by the `c389058` revision (i.e. added
or materially re-cited with a more precise line number vs. cycle 1), excluding symbols under
either plan's "Artifacts this phase produces" section (not yet in source by definition). Cycle
1's own ~48-citation table above is not re-verified here (unchanged citations, already VERIFIED).
Authority: ripgrep/`sed`/`grep` against this repo's live working tree, run independently in this
session (not merely re-stated from the Codex sub-review, though the Codex pass corroborated
every item below independently).

**Result: 0 MISSING, 0 AMBIGUOUS, 34 VERIFIED among newly-cited symbols.**

| Symbol / citation | Plan | File:Line cited | Verified against |
|---|---|---|---|
| `chat_state_sampling_config = xai_grok_sampling_types::SamplingConfig {` | 11-01 | `spawn.rs:404` | exact |
| `reasoning_effort: sampling_config.reasoning_effort,` (spawn.rs, chat-state literal) | 11-01 | `spawn.rs:413` | exact |
| `SessionHandle {` literal open | 11-01/11-02 | `spawn.rs:1594` (cited range 1593-1631) | exact, confirmed fully exhaustive (no `..Default::default()`) |
| `reasoning_effort: sampling_config.reasoning_effort,` (SessionHandle literal) | 11-02 | `spawn.rs:1613` | exact |
| `reconstruct_full_config` doc comment + signature | 11-01 | `sampler_turn.rs:246-250` | exact |
| `.unwrap_or_else(\|\| xai_grok_sampling_types::SamplingConfig { ... reasoning_effort: None, ... })` | 11-01 | `sampler_turn.rs:279-290`, field at `:288` | exact |
| final return `SamplingConfig {` literal | 11-01 | `sampler_turn.rs:393` | exact |
| `reasoning_effort: cfg.reasoning_effort,` (reconstruct_full_config return) | 11-01 | `sampler_turn.rs:406` | exact |
| `use xai_grok_sampler::SamplerConfig as SamplingConfig;` (the alias `spawn.rs`/`sampler_turn.rs` inherit via `use super::*;`) | 11-01 | `acp_session.rs:71` | exact |
| `session_setup.rs` struct-update literal (`..current_config`, needs no change) | 11-01 | `session_setup.rs:506-512` (assignment at `:429`) | exact, confirmed `..current_config` rest pattern present |
| `compaction.rs` test-actor `SamplingConfig{...}` literal | 11-01 | `compaction.rs:2177-2195` (`:2179` open) | exact |
| `DefaultModelJson` struct | 11-01 | `config.rs:3437-3468` | exact, confirmed no `pub`, matches `supports_reasoning_effort`/`reasoning_efforts` template shape |
| `default_models()` mapping (`supports_reasoning_effort: m.supports_reasoning_effort, reasoning_efforts: m.reasoning_efforts`) | 11-01 | `config.rs:3529-3530` | exact (present in cited region) |
| `ModelEntryConfig` field pair | 11-01 | `config.rs:3584-3588` | exact |
| `ModelInfo` field pair | 11-01 | `config.rs:3897-3900` | exact |
| `ModelEntryConfig` -> `ModelInfo` conversion | 11-01 | `config.rs:3979-3981` | exact |
| `ConfigModelOverride` struct (correctly out of scope) | 11-01 | `config.rs:3681` | exact, confirmed real struct |
| override-layer sibling-field participation pattern | 11-01 | `config.rs:3803-3808` | exact |
| GPT-5.6 sol/terra/luna `reasoning_effort` defaults (`low`/`medium`/`medium`) | 11-01/11-02 | `default_models.json:29,64,99` | exact |
| `SessionHandle` struct + existing `reasoning_effort` field | 11-02 | `handle.rs:41` (struct), `:108` (field), `:40` (`#[derive(Clone)]`) | exact |
| test-only `SessionHandle {` literal, `agent/subagent/tests/mod.rs` | 11-02 | `agent/subagent/tests/mod.rs:1080` | exact — `let handle = SessionHandle {` is precisely line 1080 |
| `persist_models_default` full body | 11-02 | `campaigns.rs:390-410` | exact, confirmed `None`-skips-not-clears semantics for `reasoning_effort` param specifically (no `else` branch) |
| `handle_switch_model_complete` signature + body regions | 11-02 | `lifecycle.rs:1128-1226`, sub-regions `:1146-1176`, `:1180-1209` (persist_default-gated path, confirmed a distinct mechanism, correctly left alone), `:1211-1223` (notice + `Effect::PersistPreferredModel { reasoning_effort: resolved_effort }`) | exact |
| `parse_reasoning_efforts_meta`'s collapse mechanism | 11-02 | `xai-grok-sampling-types/src/types.rs:1003` (`(!options.is_empty()).then_some(options)`) | exact — points at the executable collapse line, not just the function signature (`:991`) |
| `reasoning_effort_options_for`'s fallback mechanism | 11-02 | `xai-grok-pager/src/acp/model_state.rs:204` (`.unwrap_or_else(legacy_effort_options)`) | exact — same improvement in precision over cycle 1's citation |
| `session_handle_waiting_for_load` early snapshot / late mutable re-fetch | 11-02 | `model_switch.rs:49-50` (snapshot), `:256` (`agent.sessions.borrow_mut().get_mut(&session_id)`) | exact (off-by-one on the snapshot line is the multi-line method-call span, not an error) |
| `model_supports_reasoning_effort` / `model_reasoning_efforts` accessors | 11-02 | `models.rs:451`, `:476` (cited range 451-483) | exact |
| `GateHarness::set_model` return type | 11-02 | `model_switch_gate.rs:611-620` | exact — confirmed `Result<acp::SetSessionModelResponse, acp::Error>`, full response object |
| `GateHarness` struct fields hold no agent/handle reference | 11-02 | `model_switch_gate.rs:499-505` | exact |
| `p6_dual_login_*` test region (reuse pattern for new `p11_*` tests) | 11-02 | `model_switch_gate.rs:792-961` | exact — region confirmed to contain `p6_missing_provider_apply_blocks_codex_when_codex_slot_empty`, same-provider Sol→Terra switch test, `p6_dual_login_next_sample_uses_target_provider` |
| dispatch test region (`SwitchModelComplete` constructions + `PersistPreferredModel` assertions) | 11-02 | `dispatch/tests/task_result.rs:581-1013` | exact |
| UI-SPEC locked clamp-notice copy | 11-02 | `11-UI-SPEC.md:71-79` | exact — `reasoning effort clamped to {level} ({model} supports {list})`, confirmed verbatim |
| Phase 11 ROADMAP success criteria / MOD-01/MOD-02/OPS-04 requirement text | both | `ROADMAP.md:258-289`, `REQUIREMENTS.md:26-27,47` | exact |

### Newly-discovered production sites NOT cited by either plan (the review's own findings, not plan citations — listed here for completeness of this cycle's grounding trail)

- `agent/subagent/mod.rs:929-956` (`inherited: xai_grok_sampler::SamplerConfig { ... }`, fully
  exhaustive, confirmed field-by-field against `xai-grok-sampler/src/config.rs:64-149`) — see
  **NEW-A**.
- `agent_ops.rs:3514-3516` and `acp_agent.rs:1868-1885` (session persist/restore round-trip
  through `model_switch::apply`) — see **NEW-B**.
- `model_switch.rs:162-179` (today's gate, confirmed still present pre-revision; deleted with no
  replacement by `11-02-PLAN.md:172-176`) and `laziness.rs`'s `grok-4.5` proxy-rejection comment
  — see **NEW-C**.
- `remote/client.rs:829-895`, `config.rs:~5144-5150`, `mvp_agent/tests.rs:1131` — additional
  compile-ripple sites, see the MEDIUM finding above.

### Explicitly out of scope for this pass (same categories cycle 1 excluded, still applicable)

Compile-time verification (no code changes exist yet), the external `acp` crate's own surface,
and RESEARCH/CONTEXT-only citations not repeated by the plans remain out of scope for the same
reasons cycle 1 gave.

## Cycle 2 risk assessment

| Scope | Risk | Reason |
|---|---|---|
| 11-01 | **MEDIUM-HIGH** | The choke-point/clamp/summary design itself is sound and well-grounded (M1, L1, and the two originally-cited sites of HIGH#1 all check out exactly). The remaining risk is coverage completeness: NEW-A's subagent-inheritance gap and the compile-ripple inventory gaps mean "every Codex Responses request" is not yet true as an unqualified claim. |
| 11-02 | **HIGH** | The core state-model redesign (dedicated raw-preference field, null-vs-absent parsing, clamp-skip persistence) is correctly designed and grounded for the interactive mid-session-switch path it targets — HIGH#2/HIGH#3/HIGH#4/M4 all check out for that path specifically. But NEW-B (resume reintroduces the conflation) and NEW-C (deleted gate lets an unsupported preference reach a rejecting Grok model) are both live, concrete regressions on paths outside the interactive-switch path the revision focused on, and M5's atomicity gap persists via a different mechanism. |
| Phase 11 overall | **HIGH — not converged** | Both plans' core wire-shape and state-model designs are now well-grounded and correctly address every cycle-1 finding *on the specific path each finding was reported against*. The pattern across all three new HIGH findings is the same: the fix is correct where cycle 1 looked, but the phase's actual production surface is wider (subagent inheritance, session resume, cross-provider switch-away) than either the original plans or cycle 1's review scoped. A third revision cycle should explicitly enumerate all `SamplerConfig`/`SamplingConfig` construction sites (not just the two HIGH#1 named) and all entry points into `model_switch::apply` (not just the interactive `/model` path) before execution. |

This cycle was read-only for both reviewers; no source changes, builds, or tests were run —
expected and correct for a pre-execution plan review.

---

# CYCLE 3 — Convergence Review (final cycle)

```yaml
cycle: 3
reviewed_at: 2026-07-21T11:13:52.000Z
revision_commit: 33cafea7141b4d354cf0fb1a4d1f2ab2d4ec72e6
plans_reviewed:
  - .planning/phases/11-codex-effort-catalog-fidelity/11-01-PLAN.md (revised, cycle-2 findings)
  - .planning/phases/11-codex-effort-catalog-fidelity/11-02-PLAN.md (revised, cycle-2 findings)
reviewers:
  codex:
    model: gpt-5.6-sol
    reasoning_effort: high
    service_tier: standard
    sandbox: read-only
    cli_version: codex-cli 0.144.5
    source_grounding: true
  claude-sonnet-5:
    role: orchestrator + independent second grounding pass (this session)
    source_grounding: true
verdict: NOT CONVERGED
```

Cycle 2 raised 3 new HIGH findings (NEW-A subagent inheritance, NEW-B session resume/restore,
NEW-C deleted Grok-path gate) plus 5 actionable MEDIUM/LOW findings (A1 non-regression-sensitive
test, A2 alias-resolution atomicity gap, A3 incomplete compile-ripple inventory, A4 cross-plan
contradiction, A5 imprecise wording) against the `c389058` revision. Commit `33cafea` revises both
plans again to address every cycle-2 finding. This is the **third and final** cycle: it checks (a)
whether each cycle-2 finding is genuinely resolved, grounded against real source; (b) whether the
six cycle-1 findings cycle 2 had already confirmed resolved (HIGH#4, M1, M3, M4, L1, L2) survived
this second round of edits unregressed; and (c) whether `33cafea` itself introduces anything new.

Two independent passes were run and cross-checked, as in cycle 2: Codex CLI (`gpt-5.6-sol`, high
reasoning effort, standard service tier, `-s read-only`, `--skip-git-repo-check`,
`--dangerously-bypass-hook-trust`, cwd = repo root) via `codex exec`, and a second, fully
independent grounding pass done directly in this session (grep/read against the live source and
the `git diff c389058 33cafea` revision diff, performed *before* reading Codex's output). Both
passes independently found the same primary blocking gap (**finding 2 / NEW-B redux** below)
without either seeing the other's work first, and each pass additionally caught one thing the
other missed (this session found the two extra `PersistenceMsg::CurrentModel` production writers
by direct grep; Codex additionally traced the gap one layer deeper, into the storage trait /
`ModelPatch` / `Summary` schema, and independently caught a stale contradictory `read_first`
instruction under NEW-C that this session's first pass had missed) — cross-checking both passes
against source resolved every disagreement below in favor of the better-grounded reading.

## Per-finding resolution table (cycle-2 findings vs. this revision)

| # | Cycle-2 finding | Verdict this cycle | Evidence |
|---|---|---|---|
| 1 | HIGH NEW-A: subagent-inheritance construction site (`agent/subagent/mod.rs::read_parent_sampling_config`) | **RESOLVED** | The revision's fix is exactly targeted: the `SamplerConfig { ... }` literal opens at `subagent/mod.rs:929` (confirmed exact, function `read_parent_sampling_config` at `mod.rs:903`), already copies `reasoning_effort: cfg.reasoning_effort,` at `mod.rs:942` (confirmed), and 11-01's Task 1 action text (`11-01-PLAN.md:190`) explicitly instructs copying `reasoning_effort_supported`/`reasoning_summary_omit` from the parent's live `cfg` at this exact site, explicitly excluding it from the generic blank-fill sweep. Independently swept both reviewer passes for a fourth uncovered `SamplerConfig`/`SamplingConfig` production construction site (grepped both crates for every literal of either type) — none found; the only other non-plan-listed `SamplerConfig` literal is `tools/config.rs` (a static default baseline with no live model data, correctly left to the generic `None`/`false` sweep). `resolve_subagent_sampling_config`'s caller graph (`mod.rs:789` default inherit path, `handle_request.rs:746` not-found fallback) is unchanged from cycle 2's citation and confirmed accurate. |
| 2 | HIGH NEW-B: session resume/restore preference persistence (`agent_ops.rs`/`acp_agent.rs`) | **NOT RESOLVED** | The revision's new Task 2 fixes the two sites cycle 2 named — session-creation write (`agent/mvp_agent/agent_ops.rs:3515-3524`, confirmed exact: `let initial_reasoning_effort = chat_history.is_empty().then_some(sampling_config.reasoning_effort);` feeding `PersistenceMsg::CurrentModel { ..., reasoning_effort: initial_reasoning_effort }`) and restore-read (`agent/mvp_agent/acp_agent.rs:1868-1885`, confirmed exact: `restore_meta` built from `summary.reasoning_effort` and fed into `model_switch::apply` via `.meta(restore_meta)`) — and the plan is correct that these two sites, in isolation, are fixable as described. But the fix is incomplete on two independent axes, both confirmed by direct source read: **(a) two more production writers of the exact same `PersistenceMsg::CurrentModel` message are never mentioned by either plan** — `session/acp_session_impl/model_switch.rs:238-244` (`SessionActor::handle_set_session_model`, confirmed: `.send(PersistenceMsg::CurrentModel { model_id, agent_name, reasoning_effort: Some(sampling_config.reasoning_effort) })`, fired on **every** mid-session switch, not just creation — reached via `SessionCommand::SetSessionModel` sent from `agent/handlers/model_switch.rs:243`, the exact command Task 1 of this same plan modifies) and `agent/subagent/handle_request.rs:1364-1370` (subagent launch, confirmed: `reasoning_effort: Some(effective_sampling_config.reasoning_effort)`, writes to the subagent's own `session::persistence::new_with_explicit_dir` storage, so lower blast radius than the switch-path writer but still a real, unlisted production site). Neither plan's `files_modified`/`read_first`/action text mentions `session/acp_session_impl/model_switch.rs` at all (zero hits), and `handle_request.rs` is cited only for its *different* `read_parent_sampling_config` call at line 746, never for its `PersistenceMsg::CurrentModel` send at line ~1368. **(b) even at the one site the plan does cover, the field is not threaded past the enum**: `PersistenceMsg::CurrentModel` dispatches (via the persistence actor's match arm, `persistence.rs:1603-1614`) to the `SessionStorage` trait method `update_current_model_and_agent(&self, info, model_id, agent_name: Option<&str>, reasoning_effort: Option<Option<ReasoningEffort>>) -> io::Result<()>` (confirmed exact signature, `storage/mod.rs:530-535`) — a fixed 4-parameter trait method with no room for a 5th argument without a signature change across all implementors. The concrete implementation builds a `ModelPatch { model_id, agent_name, reasoning_effort }` (confirmed exact, `storage/summary_write.rs:44-48`, no preference field), applied to the on-disk `Summary` struct, which itself carries only `pub reasoning_effort: Option<ReasoningEffort>` (confirmed exact, `persistence.rs:892-893`) — **no `reasoning_effort_preference` field exists anywhere in the storage trait, `ModelPatch`, or `Summary`**. Task 2's action text ("Add a new field to the `PersistenceMsg::CurrentModel` variant/struct... Run `cargo check --workspace --all-targets` and fix mechanically") stops at the enum and never mentions the trait, `ModelPatch`, `Summary`, or JSONL storage (`storage/jsonl/mod.rs`) — and critically, `cargo check` will **not** catch this gap the way it catches a missing struct field, because dropping an unused value at the persistence actor's match arm is a silent no-op (at most an unused-variable warning), not a compile error. This is a materially different (harder) failure mode than a typical compile-ripple miss: the plan's own `<done>` criterion ("a resumed session's raw effort preference reflects only genuine prior explicit user choices") requires the *positive* case (an explicit preference surviving a real restart) to work, but no test in the revised Task 3 exercises that path — `p11_resumed_session_with_no_explicit_preference_does_not_inherit_persisted_catalog_default` only proves the negative/no-preference case. Net effect if executed as written: an explicit effort preference set via `/effort` or a switch mid-session would work correctly in-memory (Task 1 is sound) but would not reliably survive a process restart — either silently dropped (if the compile-ripple sweep at the two uncovered writer sites is filled with `None`, the plan's own precedent for "no signal available" sites) or never reach disk at all (if the persistence-actor/trait/storage plumbing is never touched). Separately, `11-02-PLAN.md`'s `files_modified` (frontmatter, 2 entries), `artifacts` (frontmatter, 2 entries), and Task 2's `<files>` tag (2 entries) all cite non-existent paths `crates/codegen/xai-grok-shell/src/agent/agent_ops.rs` and `crates/codegen/xai-grok-shell/src/acp_agent.rs` — the real files are `crates/codegen/xai-grok-shell/src/agent/mvp_agent/agent_ops.rs` and `crates/codegen/xai-grok-shell/src/agent/mvp_agent/acp_agent.rs` (confirmed via direct filesystem lookup; the wrong paths do not exist). The `<read_first>`/`<action>` text itself is self-correcting (it explicitly says "confirm, do not assume" and gives `grep -rn` commands that resolve to the real paths regardless), so this specific inaccuracy is unlikely to cause a wrong edit on its own — it is folded into this finding rather than counted separately because it is symptomatic of the same "which files does this task actually touch" gap. One partial mitigation worth crediting: `agent/config.rs:1415`'s `Config.reasoning_effort_override: Option<ReasoningEffort>` (`#[serde(skip)]`, populated from a CLI `--effort` flag) is a genuine, already-existing "explicit choice, distinct from a resolved catalog default" signal at the session-creation call site — so Task 2's own fallback question ("does this call site have a way to distinguish explicit choice from catalog default?") has a real, better-than-`None` answer available for the ONE site the plan does cover, even though the harder resume-survival gap above remains open regardless. |
| 3 | HIGH NEW-C: restored Grok-path support gate (`model_switch.rs`) | **PARTIALLY RESOLVED** | The actual `<action>` fix is correct and complete: it branches on `model.info().provider`, stamping the preference unconditionally for `ModelProvider::Codex` targets and restoring the exact pre-phase `model_supports_reasoning_effort` gate + `tracing::warn!` for `ModelProvider::Xai` targets (`11-02-PLAN.md:206`). Independently confirmed the pre-phase gate this restores matches current (unmodified) source exactly: `agent/handlers/model_switch.rs:161-178` today reads `if let Some(eff) = effort_override { if agent.models_manager.model_supports_reasoning_effort(model_id.0.as_ref()) { ... prepared.sampler_config.reasoning_effort = Some(eff); } else { tracing::warn!(...) } }` — byte-for-byte the shape the revision's restore instruction describes. The new test `p11_grok_path_gate_restored_unsupported_preference_dropped_not_sent` (`11-02-PLAN.md:296`) correctly exercises the exact regression end to end. **However**, the task's own `<read_first>` section still contains a stale, contradictory instruction left over from the cycle-1 revision and untouched by `33cafea` (confirmed via diff: this exact line is unchanged context, not part of the `33cafea` diff): `11-02-PLAN.md:189` — "the exact block to replace (162-181 effort_override gate — **DELETE** the `model_supports_reasoning_effort` silent-drop branch **entirely, do not merely rename it**...)". This is verbatim the instruction that produced cycle-2's NEW-C regression in the first place; it now sits ~15-40 lines above the corrected `<action>` text that properly says to restore (not delete) the gate for non-Codex targets. An executor who anchors on the strong, unambiguous `read_first` imperative before reaching the more nuanced branch-on-provider `<action>` text risks reproducing the exact bug this finding was raised to close. The design and the primary instructions are sound; the supporting orientation text was not updated to match. |
| 4 | A1: initial-session tests made regression-sensitive | **RESOLVED** | Confirmed the test harness has no existing `new_session(initial_effort)` helper (`new_session_with_model(&self, model_id: &str)`, `model_switch_gate.rs:590-604`, takes only a model id, meta carries only `{"modelId": ...}` — no effort key) — the plan's own `read_first` honestly flags this as new, narrowly-scoped harness work rather than glossing over a pre-existing gap. A generic/non-trusted credential path is buildable: the harness already constructs BYOK/API-key credentials (`agent_config_with_byok_and_custom`, `byok.api_key`/`custom.api_key`) distinct from the OAuth `sample_oidc` path used for the trusted-profile tests, giving the second new test a real, low-risk route to `ResponsesWireProfile::Disabled`. |
| 5 | A2: alias-resolved supported list (M5 atomicity gap, take 2) | **RESOLVED** | Confirmed `agent/handlers/model_switch.rs::apply` resolves `let model = agent.resolve_model_id(&model_id)?;` at line 53 — BEFORE the effort-override block — via `resolve_model_id` (`agent/mvp_agent/agent_ops.rs:1057-1082`), which matches by catalog map key OR by scanning the `.model` field (confirmed alias/routing-slug-aware, not a raw key lookup), and this same `model` binding is what's passed to `agent.prepare_prepared_sampling_config_for_model(&model, ...)` two lines before the plan's `stored_preference` computation. Deriving `supported` from `model.info().reasoning_efforts` on this already-resolved binding (rather than a second `model_reasoning_efforts(model_id)` raw lookup) is therefore provably the same resolved entry 11-01's wire-populating code uses — not merely asserted to be the same. |
| 6 | A3: compile-ripple inventory completeness | **PARTIALLY RESOLVED** | The three originally-cited sites are all confirmed accurate in current source: `remote/client.rs:829-895`'s field-by-field `ModelEntryConfig` literal, `agent/config.rs:~5144-5150`'s fallback `ModelInfo` literal (both fixed with `default_reasoning_summary_none: false`), and `agent/mvp_agent/tests.rs:1131`'s `make_test_handle` (confirmed a second, genuinely distinct `SessionHandle` literal, fixed with `reasoning_effort_preference: None`). A handful of additional test-only `SamplerConfig`/`SamplingConfig` literals remain unlisted (`xai-grok-sampler/src/actor/state.rs:83`, `xai-grok-shell/src/test_support/lsp_runtime.rs:39`, `session/helpers/session_compact.rs:1587-1588`, `xai-chat-state/src/commands.rs:372-373`) but all four are confirmed test-support/test-only code and already fall under the plan's own generic "there are more... `cargo check --all-targets` will enumerate exactly" catch-all — not a new gap. The finding is only "partially" resolved because the identical class of problem — a plan asserting a closed inventory when a production writer is missing — has recurred one revision later on the field this same revision introduces (`reasoning_effort_preference` on `PersistenceMsg::CurrentModel`); see finding 2 above for full detail (not re-litigated here to avoid double-counting the same defect under two labels). |
| 7 | A4: 11-01/11-02 empty-menu picker claim consistency | **RESOLVED** | Confirmed both plans now state the identical corrected claim: an empty/absent `reasoningEfforts` catalog array collapses to `None` via `parse_reasoning_efforts_meta` (`xai-grok-sampling-types/src/types.rs:1003`, confirmed exact: `(!options.is_empty()).then_some(options)`) and the picker falls back to the legacy 4-level menu via `reasoning_effort_options_for` (`xai-grok-pager/src/acp/model_state.rs:204`, confirmed exact fallback call) — `11-01-PLAN.md:27` and `11-02-PLAN.md:43,49` are now textually consistent (both cite the same two lines, same mechanism, same "out of scope to change" framing). |
| 8 | A5: clamp-notice timing wording | **RESOLVED** | `11-02-PLAN.md:37`'s corrected must-have accurately separates switch-time notice computation (`model_switch.rs::apply`, rendered by `lifecycle.rs::handle_switch_model_complete`) from 11-01's per-request wire-time clamp — confirmed both mechanisms exist as described and are independently triggered as the corrected wording states. |

## Cycle-1 fixes: regression check

Confirmed via direct inspection of the `git diff c389058 33cafea` hunks (not just plan-text
re-reading) that none of these six previously-resolved cycle-1 items were touched by this
revision's edits:

- **HIGH#4 (null-vs-absent ACP response parsing):** intact, untouched. The `resolve_switch_model_response_effort` action text (`11-02-PLAN.md:213`, "Tri-state-safe response parsing... fixes HIGH#4") is unchanged context in the `33cafea` diff.
- **M1 (catalog-driven summary omission):** intact, untouched. 11-01's choke-point `summary:` logic and the `default_reasoning_summary_none` catalog-threading text are unchanged context in the diff.
- **M3 (picker "soft-skip" claim correction, originally landed in 11-02):** intact, untouched at its original 11-02 location — the `33cafea` diff only *adds* the matching correction to 11-01 (finding A4 above), it does not modify 11-02's own already-correct text.
- **M4 (clamp-safe persistence, skip-not-clear semantics):** intact, untouched. `11-02-PLAN.md:217`'s "Atomic notice + clamp-safe persistence (fixes M5 and M4)" action text is unchanged context in the diff; `campaigns.rs:390-410`'s underlying skip-not-clear body is unmodified source (no code has changed).
- **L1 (threat-model invariant wording):** intact, untouched. `11-01-PLAN.md:295`'s corrected T-11-01 threat-register row is unchanged context in the diff.
- **L2 (GateHarness / response-metadata-only test redirection):** intact, untouched. Task 3's read_first and action text continue to assert exclusively via ACP response metadata / dispatched `TaskResult` fields, never `SessionHandle` internals; `GateHarness::set_model`'s full-response return type is unchanged production source.

No regressions found among the six.

## New concerns introduced by this revision

- **HIGH — see finding 2 above (session resume/restore persistence chain).** Not re-listed
  separately here since it is fully detailed in the per-finding table; flagged here only so it is
  not missed as "new since 33cafea" — the two extra `PersistenceMsg::CurrentModel` writers and the
  storage-trait/`ModelPatch`/`Summary` plumbing gap are defects *in this revision's own new Task 2*,
  not carried over from any earlier cycle (Task 2 did not exist before `33cafea`).

- **MEDIUM — stale contradictory `read_first` instruction under NEW-C.** See finding 3 above.
  Concrete fix: rewrite `11-02-PLAN.md:189`'s "DELETE the `model_supports_reasoning_effort`
  silent-drop branch entirely, do not merely rename it" to match the corrected branch-on-provider
  instruction that already exists correctly in the task's `<action>` text (restore for `Xai`
  targets, unconditional stamp for `Codex` targets).

- **MEDIUM — required-field / required-plumbing inventory still incomplete one revision later.**
  Covered in full under finding 2 (the two extra `PersistenceMsg::CurrentModel` writers) and
  finding 6 (the harmless remainder of minor test-only literals, already covered generically).

Task ordering and cross-plan consistency were independently re-checked and found clean: no stale
"Task 2" textual cross-reference anywhere in `11-02-PLAN.md` still means the pre-renumbering task
(all 8 occurrences checked individually — objective prose, read_first citations, action text, and
the verification section all correctly track the renumbering); Task 3's resume test correctly
depends on the new Task 2 executing first; and the shared-name `"reasoning_effort_supported"` ACP
JSON key vs. 11-01's Rust struct field of the same name remains suffiently distinguished in context
(re-checked, same conclusion as cycle 2 — not a type-confusion risk).

## Verification coverage (cycle 3 — newly cited symbols only)

Scope per task instructions: every symbol newly cited by the `33cafea` revision, plus every symbol
this cycle's independent hunt for missed production sites surfaced (the `PersistenceMsg::CurrentModel`
writer sweep). Cycle 1's ~48-citation table and cycle 2's 34-citation table are not re-verified here
(unchanged citations, already VERIFIED). Authority: direct `sed`/`grep`/`awk` reads against this
repo's live working tree, run independently in this session before reading Codex's output, with
Codex's independent pass corroborating every item below except where noted.

**Result: 0 MISSING, 0 AMBIGUOUS, 29 VERIFIED among newly-cited/newly-surfaced symbols.**

| Symbol / citation | Plan / source | File:Line cited | Verified against |
|---|---|---|---|
| `read_parent_sampling_config` fn signature | 11-01 | `subagent/mod.rs:903` | exact |
| `SamplerConfig {` literal open, `read_parent_sampling_config` | 11-01 | `subagent/mod.rs:929` | exact |
| `reasoning_effort: cfg.reasoning_effort,` (existing, pre-phase) | 11-01 | `subagent/mod.rs:942` | exact |
| `resolve_subagent_sampling_config` default-inherit call site | 11-01 | `subagent/mod.rs:789` | exact |
| not-found fallback call site | 11-01 | `subagent/handle_request.rs:746` | exact |
| only other non-plan `SamplerConfig` literal (static baseline, no live data) | independent sweep | `tools/config.rs:203` | exact — correctly out of scope |
| `initial_reasoning_effort` computation + `PersistenceMsg::CurrentModel` send | 11-02 | `agent/mvp_agent/agent_ops.rs:3515-3524` | exact |
| `restore_meta` construction + `model_switch::apply` call | 11-02 | `agent/mvp_agent/acp_agent.rs:1868-1885` | exact |
| **2nd uncovered `PersistenceMsg::CurrentModel` writer** — `SessionActor::handle_set_session_model`, fires every switch | independent finding | `session/acp_session_impl/model_switch.rs:97,238-244` | exact — confirmed NOT in either plan's file lists |
| `SessionCommand::SetSessionModel` send (feeds the writer above) | independent finding | `agent/handlers/model_switch.rs:243` | exact |
| **3rd uncovered `PersistenceMsg::CurrentModel` writer** — subagent launch | independent finding | `agent/subagent/handle_request.rs:1364-1370` | exact — own storage dir (`new_with_explicit_dir`, line 1016), lower blast radius |
| `PersistenceMsg::CurrentModel` enum variant definition | independent finding | `session/persistence.rs:313-320` | exact, `reasoning_effort: Option<Option<ReasoningEffort>>` |
| persistence actor's match arm (dispatch to storage) | independent finding | `session/persistence.rs:1603-1614` | exact |
| `SessionStorage::update_current_model_and_agent` trait signature | independent finding | `session/storage/mod.rs:530-535` | exact — 4 fixed params, no preference slot |
| `ModelPatch` struct (no preference field) | independent finding | `session/storage/summary_write.rs:44-48` | exact |
| `Summary` struct's `reasoning_effort` field (no preference field) | independent finding | `session/persistence.rs:787,892-893` | exact |
| `Config.reasoning_effort_override` (CLI explicit-choice signal) | independent finding | `agent/config.rs:1415` | exact — `#[serde(skip)]`, genuine explicit-choice signal available at creation |
| pre-phase `model_supports_reasoning_effort` gate (current, unmodified) | 11-02 | `agent/handlers/model_switch.rs:161-178` | exact, byte-for-byte match to what the revision restores |
| stale "DELETE... entirely" `read_first` instruction (unchanged since c389058) | independent finding | `11-02-PLAN.md:189` | exact — confirmed unchanged context line in `33cafea` diff |
| `p11_grok_path_gate_restored_unsupported_preference_dropped_not_sent` | 11-02 | `11-02-PLAN.md:296` | exact |
| `new_session_with_model` harness fn (no effort param) | 11-01/A1 | `model_switch_gate.rs:590-604` | exact |
| BYOK/API-key credential path in harness | 11-01/A1 | `model_switch_gate.rs` (`agent_config_with_byok_and_custom`, `byok.api_key`) | exact |
| `resolve_model_id` (alias/routing-slug-aware resolution) | 11-02/A2 | `agent/mvp_agent/agent_ops.rs:1057-1082` | exact |
| `let model = agent.resolve_model_id(...)` pre-effort-block binding | 11-02/A2 | `agent/handlers/model_switch.rs:53` | exact |
| `model.info` field AND `model.info()` method (both exist, coexist in same file) | independent check | `agent/config.rs:4025,4043`; used both ways in `model_switch.rs:56,82` | exact — plan's `model.info().reasoning_efforts` syntax is valid |
| `remote/client.rs` `ModelEntryConfig` remote-parsing literal | 11-01/A3 | `remote/client.rs:829-895` | exact |
| fallback/synthetic `ModelInfo` literal | 11-01/A3 | `agent/config.rs:~5144-5150` | exact |
| `make_test_handle` 2nd test-only `SessionHandle` literal | 11-02/A3 | `agent/mvp_agent/tests.rs:1114-1131` | exact |
| `parse_reasoning_efforts_meta` collapse mechanism | 11-01+11-02/A4 | `types.rs:1003` | exact |
| `reasoning_effort_options_for` legacy fallback | 11-01+11-02/A4 | `model_state.rs:204` | exact |

### Explicitly out of scope for this pass (same categories cycles 1-2 excluded, still applicable)

Compile-time verification (no code changes exist yet — this remains a pre-execution plan review),
the external `acp` crate's own surface, and RESEARCH/CONTEXT-only citations not repeated by the
plans remain out of scope for the same reasons prior cycles gave.

## Cycle 3 risk assessment

| Scope | Risk | Reason |
|---|---|---|
| 11-01 | **LOW-MEDIUM** | Every cycle-2 finding against 11-01 (NEW-A, A1, A3's three original sites, A4) is now resolved and independently re-verified against source. No new issues found in 11-01 this cycle. Residual risk is only the ordinary "cargo check will enumerate a few more test-only compile-ripple sites" class already explicitly anticipated by the plan's own text. |
| 11-02 | **HIGH** | NEW-C's core mechanism is fixed (branch-on-provider gate restore) but a stale contradictory `read_first` line remains. NEW-B (session resume/restore) is NOT resolved: the new Task 2 covers one of three real production writers of the persisted preference, and even that one writer's fix stops at the enum variant without threading through the storage trait, `ModelPatch`, or the on-disk `Summary` schema — none of which `cargo check` will force to the surface, since the gap is a silently-dropped value, not a missing field. |
| Phase 11 overall | **HIGH — NOT CONVERGED** | Two consecutive cycles have now found a live defect in the raw-preference/catalog-default identity model on a path outside whatever the immediately-prior fix targeted (cycle 2: resume read-path and the deleted gate; cycle 3: resume write-path and its storage plumbing, plus a documentation-consistency echo of the gate finding). Before execution, `11-02-PLAN.md`'s Task 2 needs a further revision that: (1) enumerates and fixes all three `PersistenceMsg::CurrentModel` production writers, not just the session-creation one; (2) threads the new preference field through `SessionStorage::update_current_model_and_agent`, `ModelPatch`, `Summary`, and JSONL storage, not just the enum variant; (3) adds a test that an explicit preference — not only the no-preference case — survives a real persist/resume round-trip; (4) fixes the file-path inaccuracies in `files_modified`/`artifacts`/Task 2's `<files>` tag; and (5) removes the stale "DELETE... entirely" `read_first` line under Task 1's NEW-C fix. Everything else in both plans — the wire-shape design (11-01, fully converged), the interactive mid-session-switch state model (11-02 Task 1, fully converged), and the alias-resolution/test-design fixes (A1/A2/A4/A5, fully converged) — is sound and does not need to be revisited. |

This cycle was read-only for both reviewers; no source changes, builds, or tests were run —
expected and correct for a pre-execution plan review.
