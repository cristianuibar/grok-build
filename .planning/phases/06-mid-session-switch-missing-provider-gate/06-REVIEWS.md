---
phase: 6
reviewers: [codex]
reviewed_at: 2026-07-17T01:11:00Z
cycle: 3
plans_reviewed:
  - 06-01-PLAN.md
  - 06-02-PLAN.md
  - 06-03-PLAN.md
  - 06-04-PLAN.md
  - 06-05-PLAN.md
  - 06-06-PLAN.md
verdict: FAIL
finding_counts:
  high: 0
  medium: 1
  low: 0
cycle1_high_still_open: 0
cycle2_high_still_open: 0
replan_commit: 53d80ac
raw_output: /tmp/gsd-review-grok-build-1557405645-cycle3-out.md
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

---

## Codex Review — Cycle 2

**Phase:** 06 — Mid-session switch & missing-provider gate  
**Requirements:** MOD-03, MOD-06  
**Reviewer:** Codex CLI (`gpt-5.6-sol` / high, sandbox read-only)  
**Cycle:** 2 (post-replan `8dfdef6`)  
**Reviewed:** 2026-07-17  
**Raw output:** `/tmp/gsd-review-grok-build-1557405645-cycle2-out.md`

### Overall verdict

FAIL

The replan materially improved Phase 6: H1 is now transactional, H2 has a provider-aware deferred path covering external login, and most cycle-1 MEDIUM/LOW findings are incorporated. However, H3 remains incomplete: Plan 04 defines dual-slot metadata and a refresh effect but does not assign concrete startup, login, logout, and ordinary focus-refresh triggers to executable tasks. The badge cache can therefore remain stale. There is also a same-wave conflict between Plans 02 and 04 and several residual validation gaps.

### Cycle-1 HIGH verification

- **H1 — RESOLVED.** Plan 02 explicitly forbids changes to `models.current`, `app.models`, toast, and `PersistSetting` until `SwitchModelComplete(Ok)` (`06-02-PLAN.md` must_haves + Task 1 transactional rewrite of `set_default_model`, `p6_transactional_default_*` tests). Live code confirms this is the correct seam: current `set_default_model` optimistically calls `set_default_model_inner`, shows the toast, and emits `PersistSetting` before `SwitchModel` (`setters.rs`). A deferred settings-intent hole remains (MEDIUM below), but the original optimistic-mutation defect is fully planned.

- **H2 — RESOLVED.** Plan 03 replaces the deferred tuple with a provider-aware intent, defines a shared readiness check, and explicitly covers in-process `AuthComplete`, bounded polling after CLI-primary login, and `FocusGained` refresh (`06-03-PLAN.md` contract + Task 2). Live code confirms `FocusGained` exists (`event_loop.rs`) and current `handle_auth_complete` is in-process-only. Implementation-trigger tests remain under-specified (MEDIUM), but the product contract itself is present.

- **H3 — PARTIAL (still HIGH).** Plan 04 now specifies dual `xai`/`codex` usable booleans in `AuthMeta`, populated through `AuthStatusReport`/`store_usable`, plus an AppView cache and explicit refresh effect. This fixes the missing **producer design**. The remaining HIGH hole is **refresh ownership**: Task 1 defines metadata, cache setters, and an effect but never instructs where startup, normal login, logout, and non-deferred focus refreshes are emitted; declared files omit `actions.rs`, task-result dispatch, auth dispatch, and `event_loop.rs`. Plan 03 only wires refresh while a deferred switch is pending. Live `AuthMeta` remains xAI-only (`auth/meta.rs`), and its current producer is tied to xAI `auth_manager.current()` paths in `mvp_agent`.

### Cycle-1 MEDIUM/LOW verification

| Cycle-1 finding | Status | Plan evidence |
|---|---|---|
| 06-01 real `model_switch::apply` harness | RESOLVED | New `tests/model_switch_gate.rs`; real ACP/MvpAgent apply path and no-side-effect assertions |
| 06-02 omitted `interactions.rs` exhaustive match | RESOLVED | File added to frontmatter and Task 2 |
| 06-03 deferred intent lacks provider | RESOLVED | `DeferredModelSwitch` includes `required_provider`; wrong-provider tests specified |
| 06-04 BYOK badge mislabel | RESOLVED | `hasOwnCredentials` ACP metadata plus badge suppression test |
| 06-04 AppCtx constructor surface | RESOLVED | Task directs compiler-driven updates to all production/test literals |
| 06-05 history/non-cancel session proof | PARTIAL | Real session harness required, but “lightest” / code-path-only substitutes still allowed |
| 06-06 false-green filters | PARTIAL | Unique `p6_` names + discovery planned; Plan 06 Task 2 verify still only aggregate `p6_` presence |
| 06-02 free-form provider parsing | RESOLVED | Strict `xai`/`codex` parsing with malformed → `Other` and test |
| 06-06 stale wave/conditional edit metadata | RESOLVED | Wave table corrected; conditional test rename documented |

### Per-plan analysis

#### 06-01 — Shell gate and typed ACP error

**Verdict:** PASS  
**Strengths:** Correct authoritative boundary; gate after resolve, before prepare/`SetSessionModel`/`ModelChanged`; dual-slot `store_usable`; BYOK skip; separate incompatible-agent code; real ACP/MvpAgent integration target with `p6_` prefixes.  
**Findings:** None.

#### 06-02 — Transactional pager gate and QuestionView

**Verdict:** PASS_WITH_WARNINGS  
**Strengths:** H1 fully specified; MissingProvider distinct; provider parse strict; `interactions.rs` included; Keep current defined.  
**Findings:**
- [MEDIUM] Deferred settings intent loses `persist_default` — Plan 02 no-session `set_default_model` path stashes a switch, but Plan 03’s `DeferredModelSwitch` carries only model, effort, and provider. After session appears, settings-specific `app.models` update / success toast / default persist intent cannot be recovered. Add `persist_default` (or equivalent) to deferred struct + test no-session → session-created → Ok path.

#### 06-03 — Login recovery and external CLI observation

**Verdict:** PASS_WITH_WARNINGS  
**Strengths:** Provider-aware deferred; wrong-provider auth cannot consume; Codex does not start xAI OAuth; external CLI not AuthComplete-only.  
**Findings:**
- [MEDIUM] External-login trigger not tested end to end — `p6_external_cli_status_refresh_applies_deferred` only injects a completed status refresh; production action leaves poll vs session flag open with no required `FocusGained`/timer emission assertion. Pin one bounded scheduling contract, stale-generation/cancellation, and test that Login now / FocusGained actually emits `RefreshProviderAuthStatus`.

#### 06-04 — Auth usability cache and badges

**Verdict:** FAIL  
**Strengths:** Dual-slot semantics, refreshable usable, BYOK meta, pure cache reads, full catalog, exact badge copy.  
**Findings:**
- [HIGH] Badge-cache refresh triggers remain unowned — must_haves promise startup/login/logout/explicit refresh, but Task 1 only defines metadata, setters, and an effect. No task wires lifecycle emission surfaces; files omit `actions.rs`, task-result, auth dispatch, event_loop. Add explicit startup, AuthComplete, logout, focus, and manual-refresh wiring with stale/error behavior + tests.
- [MEDIUM] Same-wave effect/action conflict — Plans 02 and 04 both modify `app/effects/mod.rs` in wave 2; `Effect::RefreshProviderAuthStatus` also needs `actions.rs` which Plan 04 does not declare. Serialize or assign shared Effect/TaskResult contract to one prerequisite plan.
- [MEDIUM] Settings DynamicEnum badge made optional contrary to UI-SPEC component inventory (`06-UI-SPEC.md` settings model enum = same badge policy). Extend settings path or amend UI contract.
- [LOW] Task 2 verify masking — `(pager && shell) || fallback shell` can green a failed `p6_needs_login` run. Keep pager test unconditional under `set -e`.

#### 06-05 — Dual-provider continuity proofs

**Verdict:** PASS_WITH_WARNINGS  
**Strengths:** Both switch directions, same-provider, BYOK, next-route, history, in-flight named as outcomes; real apply harness required over pure routing.  
**Findings:**
- [MEDIUM] History and in-flight observables still under-specified — “lightest” history surface and code-path-only cancel assertions still allowed. Select named chat-history/session-storage + paced turn fixture (e.g. MockInferenceServer) and assert no cancel while prompt open.

#### 06-06 — Validation and phase gate

**Verdict:** PASS_WITH_WARNINGS  
**Strengths:** Correct wave table; unique `p6_` prefixes; shell/pager separation; `06-VALIDATION.md` has solid per-group `discover` helper.  
**Findings:**
- [MEDIUM] Plan 06 Task 2 automated verify can pass with required subgroups absent — aggregate `p6_` count per crate only; leading doc checks can be bypassed by `&&` placement. Execute per-group discovery for every named filter (or run the VALIDATION aggregate script as written).

### Cross-cutting findings

- Wave 2 is not conflict-free: Plans 02 and 04 overlap on `effects/mod.rs`; Plan 04 has undeclared `actions.rs` need.
- `persist_default` must span immediate and deferred switch-intent fields.
- Badge freshness and deferred retry should share one status snapshot contract with centralized lifecycle trigger ownership (not split only into deferred-pending Plan 03 path).
- Approved badge contract covers slash **and** settings model pickers.
- Threat models prevent credential leakage/cross-slot use; residual hole is stale/runaway external-login polling — bound attempts + generation token.

### Symbol verification table

| Plan claim | Code reality | Status |
|---|---|---|
| `model_switch::apply` resolves before prepare/apply | Resolve then prepare / SetSessionModel / broadcast | OK |
| Dual-provider usable semantics | `AuthStatusReport` + `credential_usable`/`store_usable` | OK |
| `ModelProvider::{Xai,Codex}` and labels | Present in `agent/config.rs` | OK |
| BYOK `has_own_credentials` | Public on `ModelEntry` | OK |
| `MODEL_SWITCH_MISSING_PROVIDER` typed error | Not live; Plan 01 | PLANNED |
| Transactional `set_default_model` | Live still optimistic; Plan 02 | PLANNED |
| Provider-aware deferred switch | Live still `(ModelId, Option<Effort>)` tuple | PLANNED |
| Dual-slot `AuthMeta` | Live xAI account/subscription only | PLANNED |
| `RefreshProviderAuthStatus` | No live Effect/TaskResult | PLANNED |
| `Event::FocusGained` refresh seam | Exists in event_loop | OK |
| ACP model `provider` metadata | Emitted | OK |
| ACP `hasOwnCredentials` | Not emitted | PLANNED |
| MissingProvider QuestionView kind | Not live | PLANNED |
| Exhaustive `LocalQuestionKind` consumer | `interactions.rs` match exists | OK |
| `AppCtx` auth snapshot | No provider_auth field yet | PLANNED |
| `model_switch_gate.rs` / `p6_` tests | Not present yet | PLANNED |
| Real MvpAgent ACP harness patterns | `session_load_perf`, `git_contention_e2e` | OK |
| Named history/cancel observation API (Plan 05) | No concrete symbol selected | MISSING |
| `switch_changes_next_sample_route` | Pure routing test only | OK |

### Residual checker / test executability notes

- Cargo tests not executed (read-only review; target lock writes). Symbols verified with `rg`/reads.
- No `p6_` tests or `model_switch_gate.rs` yet — expected pre-execute RED.
- Documented cargo forms are syntactically valid; VALIDATION has a good per-filter discover helper.
- Plan 04 Task 2 `&&`/`||` masking defect; Plan 06 Task 2 aggregate-only discovery; Plan 05 history/in-flight still need concrete observables.

### Scope creep check

No Phase 7 or Phase 8 scope creep. Provider-status polling, `bum login` copy, and model-picker metadata are necessary Phase 6 work.

### Recommended plan edits before execute

1. Add explicit startup/login/logout/focus/manual refresh trigger ownership + tests to Plan 04 (close H3).
2. Serialize Plans 02 and 04, or move shared Effect/TaskResult definitions into one prerequisite plan.
3. Carry `persist_default` through `DeferredModelSwitch`.
4. Pin and test one bounded external-login polling/focus contract, including stale-generation handling.
5. Apply `needs login` to settings DynamicEnum picker or amend UI-SPEC.
6. Replace Plan 05 fallback observables with named history + paced real-turn non-cancel assertions.
7. Make Plan 06 execute independent discovery asserts for every required subgroup.
8. Fix Plan 04 Task 2 verify masking.

### Finding counts (cycle 2)

- HIGH: 1
- MEDIUM: 6
- LOW: 1
- cycle1_high_still_open: 1 (H3 partial)

---

## Synthesis (orchestrator) — Cycle 2

**Overall:** Cycle 2 **FAIL** — replan closed H1 (transactional switch) and H2 (external CLI deferred via status refresh/poll/focus). H3 producer shape is planned but **refresh trigger ownership** is not task-wired, so badge cache can stay false/stale after real auth lifecycle events. Six new residual MEDIUMs + one LOW remain actionable in plans.

### Cycle-1 HIGH disposition

| # | Topic | Cycle 2 |
|---|--------|---------|
| H1 | Transactional switch / no optimistic current | **RESOLVED** in 06-02 |
| H2 | External CLI → deferred apply | **RESOLVED** in 06-03 |
| H3 | AuthMeta dual-slot usable producer | **PARTIAL** — producer + effect planned; lifecycle emission ownership still **HIGH** |

### Unresolved HIGH (cycle 2)

1. **Badge-cache refresh triggers unowned (06-04, residual of H3)** — AuthMeta dual usable fields + `RefreshProviderAuthStatus` are planned, but no executable task wires startup / login (`AuthComplete` / `apply_auth_meta` consumers), logout, non-deferred focus, or manual refresh emission surfaces (`actions.rs`, task_result, auth dispatch, event_loop). Plan 03 only consumes refresh while deferred is pending. Without this, badges stay default-false after real dual-login/logout.

### Unresolved actionable Non-HIGH (not yet fully in PLAN tasks)

1. **[M] `persist_default` on deferred struct (02/03)** — no-session settings stash loses default-persist intent when Plan 03 reshapes deferred.
2. **[M] External-login scheduling E2E test (03)** — pin poll/focus emission + generation/cancel; not only injected TaskResult.
3. **[M] Same-wave `effects/mod.rs` (+ undeclared `actions.rs`) conflict (02 ∥ 04)** — serialize or single-owner shared Effect contract.
4. **[M] Settings DynamicEnum badge vs UI-SPEC (04)** — optional in plan; required in UI inventory.
5. **[M] Plan 05 concrete history/mid-turn observables** — still allows lightest/code-path substitutes.
6. **[M] Plan 06 Task 2 per-group discovery** — aggregate `p6_` only in verify block.
7. **[L] Plan 04 Task 2 verify `||` masks pager failure.**

### Fully incorporated cycle-1 items (do not re-count)

- Shell apply harness; interactions.rs; provider-aware deferred; BYOK badge meta; AppCtx churn; provider parse; wave table alignment; unique `p6_` naming + VALIDATION discovery pattern (content — Plan 06 execute verify still weak).

### Criteria checklist (cycle 2)

| Criterion | Result |
|-----------|--------|
| Shell gate before prepare/SetSessionModel/ModelChanged | OK planned (01 PASS) |
| Usable creds both xai + codex | OK planned via `store_usable` |
| BYOK / `has_own_credentials` | Shell + badge suppress planned |
| Typed MISSING_PROVIDER ≠ IncompatibleAgent | OK planned |
| QuestionView + no optimistic current | **H1 RESOLVED** (transactional 02) |
| deferred after Login now + external CLI | **H2 RESOLVED** (03 contract); scheduling test residual MEDIUM |
| Dual-slot usable producer + refresh | **H3 PARTIAL** — producer OK; trigger ownership HIGH |
| Badge does not hide models | OK planned |
| Test commands / discovery | Mostly OK; 04 mask + 06 aggregate residual |
| Threat models | Adequate; poll generation residual |
| Hallucinated symbols | Core OK; planned symbols marked PLANNED |
| Phase 7/8 scope creep | None |
| Same-wave file ownership | **MEDIUM** 02/04 effects overlap |

### Next step

Replan focusing on: (1) close H3 with explicit refresh-trigger tasks/files/tests on 06-04, (2) wave-2 Effect ownership, (3) `persist_default` on deferred, (4) remaining MEDIUM/LOW list above — then cycle 3 review.

---

## Codex Review — Cycle 3

### Overall verdict

FAIL

The replan resolves all prior HIGH findings and six of seven actionable cycle-2 residuals. Shell gating, auth lifecycle refresh, external-login observation, badge coverage, session observables, wave ownership, and subgroup discovery are executable as planned. One MEDIUM cross-plan contract gap remains: a settings-originated switch blocked with `MissingProvider` loses its `persist_default` intent while passing through QuestionView into `DeferredModelSwitch`. Execution as written would auto-switch after login but fail to persist the selected default or show the settings success behavior.

### Cycle-2 residual verification

- **H3 startup/login/logout/focus refresh triggers — RESOLVED.** Plan 04 Task 1 owns `AuthMeta` dual-slot production and `RefreshProviderAuthStatus`. Plan 04 Task 2 explicitly owns startup/session-ready, successful `AuthComplete`, logout/status-clear, unconditional `FocusGained`, and manual refresh behavior, with `p6_provider_auth_*` and `p6_refresh_*` tests. Live seams exist in `handle_auth_complete`, `dispatch_logout`, session lifecycle, and `Event::FocusGained`.

- **`persist_default` on deferred — PARTIAL.** Plan 02 Task 1 threads `persist_default` through `Effect::SwitchModel` and `TaskResult::SwitchModelComplete`; Plan 03 defines it on `DeferredModelSwitch` and preserves it during session-created/status-refresh retry. However, Plan 02’s `LocalQuestionKind::MissingProviderLogin` and `MissingProviderLoginAnswered` omit the flag, while Plan 03 Task 1 action 3 sets it to `false` unless a deferred entry already exists. For a live session, `set_default_model` emits `SwitchModel` rather than stashing a deferred entry, so the settings intent is unavailable when Login now is answered.

- **External-login tests and generation cancellation — RESOLVED.** Plan 03’s pinned scheduling contract and Task 2 require production emission from Login now and `FocusGained`, bounded polling, refresh completion, and stale-generation rejection through `p6_focus_gained`, `p6_external_cli`, and `p6_refresh_generation`.

- **Wave ownership 02→04 — RESOLVED.** Plan 04 is wave 3 with `depends_on: [06-01, 06-02]`. Its ownership table makes Plan 02 sole owner of the transactional SwitchModel contract and Plan 04 sole owner of `RefreshProviderAuthStatus`.

- **Settings DynamicEnum badge required — RESOLVED.** Plan 04 Task 3 action 5 mandates the same badge policy for `ActiveModelCatalog`; `p6_needs_login_settings_dynamic_enum` is required behavior.

- **Concrete Plan 05 observables — RESOLVED.** The non-negotiable session harness requires non-empty user/assistant history with count plus stable identity, `MockInferenceServer::hold_agent_completions`, a real in-flight switch, and an observed no-cancel path. Code-path-only and pure-routing substitutes are explicitly forbidden.

- **Plan 06 per-subgroup discovery — RESOLVED.** Plan 06’s required subgroup section, Task 2 action, and automated verification use `discover()` independently for shell and pager filters. Aggregate `grep -c p6_` is explicitly forbidden.

- **No `||` masking in Plan 04 — RESOLVED.** Task verifications run cargo commands unconditionally under `set -euo pipefail`. Remaining `|| true` occurrences only normalize `grep -c` before an explicit numeric discovery assertion; they do not mask cargo failures.

### Cycle-1 HIGH verification

- **H1 transactional switch / no optimistic current — RESOLVED.** Plan 02 Task 1 forbids current-model, app-model, toast, and default persistence changes before `SwitchModelComplete(Ok)`, with dedicated transactional tests.
- **H2 external CLI login drives deferred retry — RESOLVED.** Plan 03 requires bounded polling, unconditional focus refresh, refresh-result consumption, and stale-generation cancellation.
- **H3 authoritative dual-slot status producer — RESOLVED.** Plan 04 Tasks 1–2 cover the producer, cache, refresh effect, and all required lifecycle triggers.

### Per-plan analysis

#### 06-01 — Shell missing-provider gate

**Verdict:** PASS

**Strengths:** Correct authority and gate position; dual-slot `store_usable` semantics; refreshable credentials accepted; BYOK exemption; distinct typed ACP error; real apply-path harness and no-side-effect assertions.

**Findings:** None.

#### 06-02 — Transactional pager gate and QuestionView

**Verdict:** PASS

**Strengths:** Fully resolves optimistic state/persistence; strict provider parsing; separate MissingProvider QuestionView; exhaustive `LocalQuestionKind` consumer included; Keep current is non-mutating.

**Findings:** None independently; its QuestionView payload participates in the shared Plan 03 finding below.

#### 06-03 — Login recovery and deferred retry

**Verdict:** FAIL

**Strengths:** Provider-aware deferred state, bounded external-login polling, production refresh emissions, wrong-provider rejection, stale-generation handling, and session-created retry are well specified.

**Findings:**

- **[MEDIUM] Settings-origin `persist_default` is lost across QuestionView → Login now** — Plan 02 Task 1 carries the flag only as far as `SwitchModelComplete`; its QuestionView kind/action carries model, effort, and provider only. Plan 03 Task 1 action 3 then creates the deferred intent with `persist_default: false` unless an entry already exists, but the live-session settings path has not stashed one — carry the flag through `LocalQuestionKind`/answer Action, or stash a complete deferred gate intent when MissingProvider completes; add an immediate-session settings → block → Login now → refresh → successful retry persistence test.

#### 06-04 — Dual-slot cache and badges

**Verdict:** PASS

**Strengths:** Closes H3 lifecycle ownership; serializes shared file edits; preserves the mixed catalog; handles refreshable credentials and BYOK; requires both slash and settings badges; avoids filesystem reads during picker construction.

**Findings:** None.

#### 06-05 — Session continuity proofs

**Verdict:** PASS

**Strengths:** Uses a real session/apply path, non-empty history snapshots, held inference, observable non-cancellation, next-provider routing, both switch directions, same-provider switching, and BYOK.

**Findings:** None.

#### 06-06 — Validation and phase gate

**Verdict:** PASS

**Strengths:** Correct wave map, unique prefixes, independent discovery assertions, shell/pager separation, incompatible-agent regression, and fixture-only policy.

**Findings:** None.

### Cross-cutting findings

- No unresolved same-wave conflict remains. Plans 02→04→03 serialize all shared `actions.rs` and `effects/mod.rs` work. Plans 01→05 serialize `model_switch_gate.rs`.
- The only incomplete shared contract is:

  `persist_default` in SwitchModelComplete → MissingProvider QuestionView → Login-now answer → DeferredModelSwitch.

- Auth status remains display-only in the pager; Plan 01’s shell gate stays authoritative.
- Poll generation, provider isolation, malformed provider handling, and secret-free errors are adequately covered by tasks and threat models.
- No new credential-leakage or cross-slot threat-model gap was found.

### Symbol verification table

| Plan claim | Code reality | Status |
|---|---|---|
| `model_switch::apply` is the switch authority | `pub(crate)` handler exists at `model_switch.rs:13` | OK |
| Gate can precede preparation and mutation | Resolve at line 34; prepare at 116; `SetSessionModel` at 196; broadcast later | OK |
| Dual-provider usability exists | `AuthStatusReport`, `credential_usable`, and `store_usable` exist | OK |
| `AuthMeta` already carries dual slots | Live `AuthMeta` is xAI account/subscription only | PLANNED |
| BYOK seam exists | `ModelEntry::has_own_credentials` exists | OK |
| ACP model metadata exposes provider | `to_acp_model_info` emits `provider` | OK |
| ACP metadata exposes `hasOwnCredentials` | Not live | PLANNED |
| Pager deferred state is provider-aware | Live state remains `Option<(ModelId, Option<Effort>)>` | PLANNED |
| Transactional settings switch is live | Live `set_default_model` still mutates and persists optimistically | PLANNED |
| `Event::FocusGained` exists | Present at `event_loop.rs:2665` | OK |
| MissingProvider QuestionView kind exists | Not live | PLANNED |
| Exhaustive local-kind consumer exists | `interactions.rs:81–86` matches current variants | OK |
| Settings DynamicEnum seam exists | `dynamic_enum_choices` and `PagerLocalSnapshot` exist | OK |
| Real MvpAgent ACP harness patterns exist | `session_load_perf.rs` and `git_contention_e2e.rs` construct both ACP sides | OK |
| Held inference observable exists | `MockInferenceServer::hold_agent_completions` and release exist | OK |
| History observables exist | Chat-state exposes conversation, length, item, and count queries | OK |
| Existing routing regression exists | `switch_changes_next_sample_route` is present but remains pure routing only | OK |
| Phase 6 `p6_` tests exist | None exist before execution | PLANNED |
| Settings-origin persist intent reaches deferred Login now | No live field; updated plan omits the handoff | MISSING |

### Residual checker / test executability notes

- Cargo tests were not run because this was a read-only review and Cargo would write target locks/artifacts.
- Documented cargo command forms are syntactically valid.
- Absence of `p6_` tests and `model_switch_gate.rs` is expected before execution.
- Plan 06’s `discover()` ordering is valid for both integration targets and crate-wide filters.
- Plan 04 no longer has cargo-failure fallback masking.
- A required regression is still missing from plan behavior: live-session `SetDefaultModel` → MissingProvider → Login now → provider becomes usable → retry succeeds → exactly one default persistence effect, app model update, and settings success toast. Keep current must produce none of those effects.

### Scope creep check

No Phase 7 subagent orchestration or Phase 8 rebrand work is included. Provider-status refresh, `bum login` copy, model metadata, and settings badge plumbing are necessary Phase 6 work.

### Recommended plan edits before execute

1. Complete the settings-origin deferred contract:

   - Add `persist_default` to `LocalQuestionKind::MissingProviderLogin` and `MissingProviderLoginAnswered`, or stash a complete `DeferredModelSwitch` when the missing-provider result opens.
   - Change Plan 03 Task 1 so Login now consumes the carried value instead of defaulting to `false`.
   - Add a `p6_deferred_*` test covering the full live-session settings block/login/retry/success path and asserting one default persist; add the Keep current counterpart asserting zero persistence.

### Finding counts

- HIGH: 0
- MEDIUM: 1
- LOW: 0
- cycle2_high_still_open: 0
- cycle1_high_still_open: 0

VERDICT: FAIL  
CURRENT_HIGH: 0  
CURRENT_ACTIONABLE: 1

---

## Synthesis (orchestrator) — Cycle 3 (FINAL)

**Overall:** Cycle 3 **FAIL** with **0 HIGH** — replan `53d80ac` closed all prior HIGHs and every named cycle-2 residual except a narrow **`persist_default` handoff** through QuestionView on the **live-session settings** path. Plans 01, 02, 04, 05, 06 are execute-ready; Plan 03 is execute-ready for recovery/external CLI except that settings-origin Login-now deferred may incorrectly force `persist_default: false`.

### Cycle-2 residual disposition

| Residual | Cycle 3 |
|----------|---------|
| H3 refresh trigger ownership (04 Task 2) | **RESOLVED** |
| `persist_default` on deferred | **PARTIAL** — field exists on Effect/Complete + DeferredModelSwitch; **live-session** MissingProvider → Login now drops the flag |
| External-login emission + generation cancel | **RESOLVED** |
| Wave serialize 02→04 | **RESOLVED** (04 wave 3 depends_on 02) |
| Settings DynamicEnum badge required | **RESOLVED** |
| Plan 05 concrete observables | **RESOLVED** (named history + MockInferenceServer; no substitutes) |
| Plan 06 per-subgroup discovery | **RESOLVED** |
| Plan 04 no `\|\|` cargo masking | **RESOLVED** |

### Cycle-1 HIGH disposition (final)

| # | Topic | Cycle 3 |
|---|--------|---------|
| H1 | Transactional switch / no optimistic current | **RESOLVED** |
| H2 | External CLI → deferred apply | **RESOLVED** |
| H3 | Dual-slot AuthMeta + lifecycle refresh | **RESOLVED** |

### Unresolved HIGH (cycle 3)

None.

### Unresolved actionable Non-HIGH

1. **[M] Settings-origin `persist_default` lost across QuestionView → Login now (02/03)** — Live-session `set_default_model` emits `SwitchModel { persist_default: true }` then opens `MissingProviderLogin { model_id, effort, provider }` **without** the flag; Plan 03 Login now sets `persist_default: false` unless a pre-existing deferred stash already holds true (no-session path only). Fix: carry `persist_default` on kind/answer **or** stash complete `DeferredModelSwitch` on MissingProvider open; Login now consumes it; add `p6_deferred_*` live-session settings block→login→retry persist test (+ Keep current zero-persist).

### Fully resolved this cycle (do not re-count)

- H3 lifecycle emissions; wave 02→04 ownership; DynamicEnum badge; Plan 05 named observables; Plan 06 per-subgroup discover; Plan 04 verify no cargo masking; external login FocusGained/poll/generation; all cycle-1 HIGHs.

### Criteria checklist (cycle 3)

| Criterion | Result |
|-----------|--------|
| Shell gate before prepare/SetSessionModel/ModelChanged | OK planned (01 PASS) |
| Usable creds both xai + codex | OK planned |
| BYOK / has_own_credentials | Shell + badge suppress planned |
| Typed MISSING_PROVIDER ≠ IncompatibleAgent | OK planned |
| QuestionView + no optimistic current | **H1 RESOLVED** |
| deferred after Login now + external CLI | **H2 RESOLVED**; scheduling tests planned |
| Dual-slot usable producer + refresh ownership | **H3 RESOLVED** |
| Badge does not hide models; settings DynamicEnum | OK planned |
| Test commands / per-subgroup discovery | OK planned |
| Threat models | Adequate |
| Hallucinated symbols | Core OK; planned symbols PLANNED |
| Phase 7/8 scope creep | None |
| Same-wave file ownership | **RESOLVED** (02→04) |
| Settings persist_default through Login now | **MEDIUM residual** |

### Execute readiness

- **No HIGH blockers.** Final cycle residual is one MEDIUM contract hole on settings default-persist after blocked switch + Login now.
- Prefer a **tiny replan** of Plans 02/03 for the handoff field + test **or** execute with that item as first-class must-fix during Plan 02 Task 1 / Plan 03 Task 1 (executor must not default Login-now deferred to `false` when `SwitchModelComplete` had `persist_default: true`).
- Phase gate should not ship without the live-session settings→Login now→persist path covered.

### Next step

Optional micro-replan of 02/03 for `persist_default` on QuestionView/answer/deferred Login now + one E2E `p6_` test; otherwise start `/gsd-execute-phase 6` with that residual as a hard execute acceptance check.

**Counts:** HIGH 0 · MEDIUM 1 · LOW 0 · cycle1/2 high still open 0 · CURRENT_ACTIONABLE 1

---

## Micro-replan note — Cycle 3 residual MEDIUM incorporated

**Date:** 2026-07-17  
**Scope:** Plans `06-02-PLAN.md` + `06-03-PLAN.md` only.

**Finding closed in plan text:** Live-session settings `persist_default` was dropped across MissingProvider QuestionView → Login now because kind/answer carried only model/effort/provider and Login now defaulted deferred to `persist_default: false` unless a no-session stash already existed.

**Preferred contract now planned:**
1. **Plan 02** — On `SwitchModelComplete(Err MissingProvider)`, stash full `DeferredModelSwitch { model_id, effort, required_provider: Some(...), persist_default }` from the complete intent, then open QuestionView. Session-only switches stash `persist_default: false`; settings path stashes `true`. Keep current clears the stash (zero-persist tests).
2. **Plan 03** — Login now **consumes** the gate-open stash and must not overwrite `persist_default` to false; post-login/status-refresh apply re-issues `Effect::SwitchModel` with deferred’s flag. E2E unit: live-session settings → block → Login now → usable → retry persists default once; Keep current zero-persist.

After this micro-replan, the cycle-3 MEDIUM is **planned** (execute-ready). Residual actionable count for execute: treat as closed in plan space; verify via `p6_missing_provider_live_session_settings_*`, `p6_login_now_preserves_gate_open_persist_default_true`, `p6_live_session_settings_login_now_retry_persists_default`, `p6_live_session_settings_keep_current_zero_persist` / `p6_keep_current_after_settings_*`.
