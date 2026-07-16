---
phase: 6
reviewers: [codex]
reviewed_at: 2026-07-17T00:35:00Z
cycle: 1
plans_reviewed:
  - 06-01-PLAN.md
  - 06-02-PLAN.md
  - 06-03-PLAN.md
  - 06-04-PLAN.md
  - 06-05-PLAN.md
  - 06-06-PLAN.md
verdict: FAIL
finding_counts:
  high: 3
  medium: 7
  low: 2
---

# Cross-AI Plan Review — Phase 6

**Phase:** 06 — Mid-session switch & missing-provider gate  
**Requirements:** MOD-03, MOD-06  
**Reviewer:** Codex CLI (`gpt-5.6-sol` / high, sandbox read-only)  
**Cycle:** 1  
**Reviewed:** 2026-07-17  

## Codex Review — Cycle 1

### Overall verdict
FAIL

The shell-side design is mostly sound: the proposed credential gate sits at the correct point, uses the dual-provider auth store, preserves BYOK, and introduces a distinct typed ACP error. The phase is not ready to execute because three product-level gaps remain: the pager still mutates and persists the selected model before the shell accepts it, external Codex CLI login cannot currently trigger the deferred switch, and the proposed pager credential cache has no authoritative data source. Several test plans also rely on a provider-routing fixture that does not exercise `model_switch::apply`, session history, or in-flight turns.

### Per-plan analysis

#### 06-01 — Shell gate + typed ACP error
**Verdict:** PASS_WITH_WARNINGS  
**Strengths:** The gate is correctly placed after `resolve_model_id` and before compatibility preparation, `prepare_prepared_sampling_config_for_model`, `SetSessionModel`, state mutation, and `ModelChanged`. `store_usable` has the correct refresh-token-aware semantics for both `providers.xai` and `providers.codex`. The BYOK exemption uses the existing catalog-derived `has_own_credentials` seam. The missing-provider ACP code is separate from `MODEL_SWITCH_INCOMPATIBLE_AGENT`, and its proposed payload contains no credentials.

**Findings:**
- [MEDIUM] The named integration-test file does not currently exercise the gate or its side effects — `provider_routing.rs::switch_changes_next_sample_route` composes credential preparation and routing transforms but does not instantiate `MvpAgent`, call `model_switch::apply`, observe `SetSessionModel`, or subscribe to `ModelChanged`; meanwhile `apply` is `pub(crate)` (`model_switch.rs:13`) — the proposed tests could pass without proving fail-closed behavior — add handler-local/MvpAgent tests or describe an explicit public ACP session harness with assertions that the model, prepared state, and event stream remain unchanged.

#### 06-02 — Pager MissingProvider QuestionView + no optimistic current
**Verdict:** FAIL  
**Strengths:** The proposed `SwitchModelError::MissingProvider` mapping is distinct from `IncompatibleAgent`, and mapping the known typed code before the generic fallback is correct. The existing `QuestionViewState`, `LocalQuestionKind`, `with_no_freeform`, prompt restoration, and local-answer translation mechanisms are appropriate for the modal. “Keep current” naturally leaves the current session model untouched if the settings transaction is corrected.

**Findings:**
- [HIGH] The plan does not actually eliminate optimistic state or configuration mutation — `set_default_model` calls `set_default_model_inner` before emitting `Effect::SwitchModel`; that function immediately changes `models.current` and `app.models`, then `PersistSetting` is emitted before `SwitchModel` (`setters.rs:1449`, `1457`, `1520`, `1540–1569`) — rolling the pointer back after an ACP error still produces the forbidden flash, and a rejected target can remain persisted as the default — defer all current-model mutation, success toast, and persistence until `SwitchModelComplete(Ok)`; carry a “persist default after successful switch” intent through the effect/result contract.
- [MEDIUM] Adding `LocalQuestionKind::MissingProvider` has an omitted exhaustive-match consumer — `app/acp_handler/interactions.rs:78–87` explicitly matches every local question kind but is absent from the plan’s file list — execution will otherwise fail to compile or require an unplanned edit — add this file and its cancellation behavior to Task 2.
- [LOW] The pager payload retains `provider` as a free-form `String` — the shell currently derives it from trusted catalog metadata, which is good, but the pager should still parse only `xai` or `codex` before choosing behavior or rendering provider-specific copy — introduce a small enum/parser and route malformed values to `Other`.

#### 06-03 — Login now + deferred switch retry
**Verdict:** FAIL  
**Strengths:** Reusing the existing deferred-switch machinery is directionally correct. Stashing before authentication avoids losing the target, and applying only after successful authentication maintains the fail-closed shell boundary. The proposed normal in-process `AuthComplete` path fits the existing pager authentication lifecycle.

**Findings:**
- [HIGH] External Codex CLI login cannot produce the planned retry signal — `handle_auth_complete` only handles an in-process `Effect::Authenticate` result for the currently matching `AuthState::Authenticating`; there is no auth-file watcher, provider-status poll, focus-triggered refresh, or cross-process event that turns `bum login --provider codex` into pager `AuthComplete` — the Codex CLI-primary branch would tell the user to log in but never auto-apply the deferred switch — add an explicit provider-status refresh loop or auth-store watcher, including focus regain and post-command polling, and invoke deferred apply only when the required slot becomes usable.
- [MEDIUM] The deferred intent does not retain the blocked provider — `AgentSession.deferred_model_switch` is only `(ModelId, Option<ReasoningEffort>)` (`agent.rs:698`) — any unrelated successful authentication can trigger a retry across agents; the shell will block it again, but this creates spurious retries and modals — replace the tuple with a provider-aware struct or store the required provider alongside it, then consume it only after that provider is usable.

#### 06-04 — Dual-slot usable cache + needs-login badge
**Verdict:** FAIL  
**Strengths:** The plan preserves the full mixed model catalog and treats the badge as annotation rather than filtering. `credential_usable`/`store_usable` are the correct source semantics, including refreshable credentials. Keeping auth-file access out of `build_model_items` is also architecturally correct.

**Findings:**
- [HIGH] The proposed cache has no authoritative producer — pager `AuthMeta` contains xAI account/subscription metadata, not `providers.xai/providers.codex` usability; `apply_auth_meta` therefore cannot populate or refresh the two booleans, and the plan adds no status effect, startup fetch, watcher, or shell-to-pager snapshot contract — badges would remain false/stale after startup, login, logout, or external CLI login — add a typed provider-auth status result and define refresh triggers and stale/error behavior.
- [MEDIUM] Provider-only badge logic mislabels BYOK models — the shell intentionally bypasses provider OAuth for `ModelEntry::has_own_credentials`, but ACP model metadata currently exposes `provider` without an own-credentials/login-required flag (`config.rs:5408–5464`) — a usable BYOK model would display `needs login` when its provider slot is empty — expose a trusted `hasOwnCredentials` or `requiresProviderLogin` field and suppress the badge for such models.
- [MEDIUM] Extending `AppCtx` has a wider compilation/test surface than declared — production literals exist outside the plan’s files, including `slash/mod.rs:326` and `app/modals.rs:65`, plus numerous test literals — adding mandatory provider-status fields will require updating them all — list the affected constructors or design an accessor/default that avoids broad literal churn.

#### 06-05 — Dual-login switching + BYOK/history coverage
**Verdict:** PASS_WITH_WARNINGS  
**Strengths:** The scenarios cover both switching directions, same-provider switching, BYOK exemption, next-turn routing, history preservation, and non-cancellation of an in-flight turn. This is the right acceptance surface for MOD-03 and MOD-06.

**Findings:**
- [MEDIUM] The proposed file lacks the session-level harness needed for its strongest assertions — current `provider_routing.rs` tests routing components and contains no `MvpAgent`, ACP `set_model`, chat-state history, session actor, or in-flight turn control — “history intact” and “mid-turn switch does not cancel” could become tautological unit tests rather than product proofs — specify a real session/MvpAgent harness, snapshot history before and after, observe absence of a cancellation command, and verify the following prompt uses the new provider.

#### 06-06 — Validation + phase gate
**Verdict:** PASS_WITH_WARNINGS  
**Strengths:** The phase gate covers shell, pager, routing, formatting, clippy, and manual UX review. Dependencies correctly place final validation after the implementation plans, and the validation checklist reflects both requirements.

**Findings:**
- [MEDIUM] Several test filters can report false green — Cargo returns success when a filter matches zero tests; the discovery command only uses `-- --list` without asserting a match, while broad filters such as `auth_complete`, `deferred_model_switch`, and `needs_login` already match unrelated existing tests — validation could pass without running the new Phase 6 cases — give all new tests a unique Phase 6 prefix and make discovery assert at least one matching line before executing each filtered group.
- [LOW] Validation metadata is internally stale — `06-VALIDATION.md` groups 06-04 in wave 3 even though its plan declares wave 2, and 06-06 permits a test rename despite listing only documentation files — align the wave table and explicitly permit the relevant test file if a rename is part of the gate task.

### Cross-cutting findings

- The shell is the right enforcement boundary, but pager state and persistence must also be transactional. Otherwise the product fails closed at execution while still presenting and saving a model that was never accepted.
- The deferred switch, badge cache, and external-login flow need one shared provider-status contract. At present, 06-03 assumes a completion event that does not exist and 06-04 assumes usable-status data that `AuthMeta` does not carry.
- BYOK is handled correctly in 06-01 but not propagated into 06-04’s presentation logic.
- Wave ordering and `depends_on` declarations are otherwise coherent: 06-03 correctly depends on both modal work and credential-cache work, and no same-wave implementation files conflict.
- The error threat model is adequate at the shell boundary: provider comes from resolved catalog metadata, the suggestion is reconstructed, and no token or auth-store value enters the ACP error. Pager-side provider validation remains worthwhile hardening.

### Symbol verification

| Plan claim | Code reality | Status |
|---|---|---|
| `model_switch::apply` resolves before preparation/apply side effects | `resolve_model_id` precedes compatibility work; prepare starts at `model_switch.rs:115`, `SetSessionModel` at `195` | OK |
| `ModelSwitchIncompatibleAgentError` and `MODEL_SWITCH_INCOMPATIBLE_AGENT` exist | Both are defined with ACP conversions in `agent/config.rs:5465–5517` | OK |
| `ModelSwitchMissingProviderError` exists | Not present yet; planned addition | MISSING |
| `ModelProvider::{Xai,Codex}` and `as_str`/display support exist | Defined in `agent/config.rs:3394–3417` | OK |
| `ModelEntry::has_own_credentials` exists | Defined in `agent/config.rs:4040–4051` | OK |
| Dual-slot auth document supports `providers.xai` and `providers.codex` | Nested provider parsing exists in `auth/storage.rs:121–141` | OK |
| `credential_usable` and `store_usable` exist | Defined in `auth/status.rs:121–147` | OK |
| Pager `SwitchModelError` mapping seam exists | Enum and ACP mapping exist but currently handle only incompatible/generic errors | OK |
| `deferred_model_switch` stores target and effort | Exact tuple exists at `app/agent.rs:698`; required provider is absent | OK |
| `QuestionViewState`, `LocalQuestionKind`, and `with_no_freeform` exist | Existing agent-type mismatch path provides the proposed analogue | OK |
| External CLI login produces pager `AuthComplete` | No cross-process notification, watcher, or poller found | MISSING |
| `AuthMeta` can populate dual provider usability | It contains xAI account/subscription metadata only | MISSING |
| ACP model metadata exposes provider | `to_acp_model_info` includes `provider` | OK |
| ACP model metadata exposes BYOK/login requirement | No `hasOwnCredentials` or equivalent is emitted | MISSING |
| `build_model_items` and `AppCtx` are available for badge plumbing | Both exist; current signatures carry no provider-status snapshot | OK |
| `provider_routing::switch_changes_next_sample_route` exists | Exists at `provider_routing.rs:995`, but is a pure routing test rather than a session/apply test | OK |
| `provider_routing.rs` has an existing history/in-flight MvpAgent harness | No such harness or symbols were found | MISSING |
| `apply_deferred_switch_outcome` exists | Existing lifecycle helper is present | OK |

### Test command executability

The documented `cargo test -p …` and `cargo test -p … --test provider_routing …` forms are syntactically valid, and the `provider_routing` integration target exists.

A live `cargo test -p xai-grok-shell --test provider_routing -- --list` could not execute in this review environment because Cargo attempted to open `target/debug/.cargo-lock` and the filesystem is read-only. Symbol/filter existence was therefore checked with `rg`.

The following situation is acceptable RED-until-implementation:

- New filters containing `model_switch_missing_provider`, dual-login, BYOK, history, and provider-auth usability currently have no corresponding tests.

The following validation patterns are unsafe:

- `-- --list` alone does not fail when the intended test is absent.
- `auth_complete`, `deferred_model_switch`, and `incompatible_agent` already match pre-existing tests.
- `needs_login` already matches an unrelated startup-auth test.
- Broad filters therefore need unique test names plus a non-empty discovery assertion.

### Residual checker warnings reassessed

The Codex Login-now/external-CLI warning is blocking, not merely residual. The planned pager flow has no mechanism to observe credentials written by a separate `bum login --provider codex` process, so it cannot satisfy the locked auto-retry behavior.

The checker’s history and mid-turn concerns remain valid. Plan 06-05 names the right outcomes but does not identify a harness capable of observing them.

The earlier wave concern is resolved in the plans themselves: 06-04 is wave 2 and 06-03 depends on it. Only `06-VALIDATION.md` retains the stale wave assignment.

### Scope creep check

No Phase 7 or Phase 8 scope creep was found. The plans remain focused on provider gating, login continuation, model-picker status, and switching validation. They do not introduce subagents, workflow customization, broader rebranding, or unrelated telemetry/update work. Using `bum` in login copy is required product naming, not Phase 8 expansion.

### Recommended plan edits before execute

1. Rewrite 06-02’s settings flow as a transaction: do not mutate `models.current`, `app.models`, toast, or persisted default until shell `SwitchModelComplete(Ok)`.
2. Add a typed dual-provider auth-status acquisition path shared by 06-03 and 06-04, with startup, in-process auth, logout, external-login, and focus-regain refresh triggers.
3. Store the required provider in `deferred_model_switch` and consume the intent only after that provider becomes usable.
4. Expose trusted BYOK/login-required metadata to the pager and suppress `needs login` for own-credential models.
5. Add `app/acp_handler/interactions.rs` and all affected `AppCtx` constructors/tests to the declared file surfaces.
6. Replace the assumed `provider_routing.rs` gate/history harness with an explicit handler-local or MvpAgent ACP session harness.
7. Define concrete observables for no side effects, history preservation, no in-flight cancellation, and next-turn provider selection.
8. Give new tests unique names and make validation fail when discovery returns zero matching tests.
9. Validate the typed provider discriminator at the pager boundary.
10. Correct the stale wave assignment and conditional test-file edit in `06-VALIDATION.md`/06-06.

### Finding counts
- HIGH: 3
- MEDIUM: 7
- LOW: 2

---

## Synthesis (orchestrator)

**Overall:** Cycle 1 **FAIL** — 3 HIGH product gaps block execute readiness. Shell gate design (Plan 01) is largely solid; pager transactionality, dual-slot status feed, and external-CLI deferred retry must be planned before `/gsd-execute-phase 6`.

### Unresolved HIGH

1. **Optimistic pager model mutation (06-02)** — Settings/live switch path mutates `models.current` / `app.models` and may `PersistSetting` before shell `SwitchModelComplete(Ok)`. Rollback-after-error still flashes and can persist a rejected default. Needs transactional switch: mutate/persist only on success.
2. **External CLI login → deferred retry (06-03)** — Codex Login-now CLI-primary path has no auth-file watcher / focus poll / status refresh that yields pager `AuthComplete`. Deferred auto-retry after `bum login --provider codex` is not implementable as written. Elevate checker residual to plan requirement.
3. **No authoritative dual-slot usable producer (06-04)** — `AuthMeta` is xAI account/subscription only; cannot feed `providers.xai`/`providers.codex` usable flags. Badge cache needs a typed provider-auth status path + refresh triggers shared with deferred apply.

### Unresolved actionable MEDIUM/LOW

1. **[M] 06-01 harness** — `provider_routing` does not exercise `model_switch::apply` / side-effect absence; need handler-local or MvpAgent ACP harness.
2. **[M] 06-02 file list** — Add `app/acp_handler/interactions.rs` (exhaustive `LocalQuestionKind` match).
3. **[M] 06-03 deferred shape** — Store required provider with deferred intent; apply only when that slot is usable.
4. **[M] 06-04 BYOK badge** — Expose `hasOwnCredentials` / equivalent; suppress `needs login` for own-credential models.
5. **[M] 06-04 AppCtx surface** — List all constructors/tests or use default/accessor to avoid broad churn (`slash/mod.rs`, `app/modals.rs`, tests).
6. **[M] 06-05 session proofs** — History/non-cancel/next-turn need real session harness, not pure routing unit tests.
7. **[M] 06-06 false-green filters** — Unique Phase 6 test prefixes + discovery assert ≥1 match before execute.
8. **[L] 06-02 provider parse** — Parse only `xai`/`codex` at pager boundary; malformed → `Other`.
9. **[L] 06-06/VALIDATION wave table** — Align 06-04 wave 2; clarify allowed test-file edits if renames are in gate.

### Criteria checklist (cycle 1)

| Criterion | Result |
|-----------|--------|
| Shell gate before prepare/SetSessionModel/ModelChanged | OK in plan design (verify at execute) |
| Usable creds both xai + codex (not xAI AuthManager alone) | OK planned via `store_usable` |
| BYOK / `has_own_credentials` | Shell OK; pager badge gap (MEDIUM) |
| Typed MISSING_PROVIDER ≠ IncompatibleAgent | OK planned |
| QuestionView + no optimistic current | **HIGH gap** on settings path |
| deferred after Login now + external CLI | In-process OK; **external CLI HIGH** |
| Badge does not hide models | OK planned |
| Test commands executable | Syntax OK; false-green filter risk |
| Threat models | Shell adequate; pager provider parse LOW |
| Hallucinated symbols | Core analogs OK; AuthMeta/CLI AuthComplete/BYOK meta MISSING |
| Phase 7/8 scope creep | None |

### Next step

Replan with `--reviews` feedback incorporating the three HIGHs and actionable MEDIUM/LOW items above, then re-run cycle 2 review.
