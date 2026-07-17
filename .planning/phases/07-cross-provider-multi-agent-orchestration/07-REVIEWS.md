---
phase: 7
reviewers: [codex]
reviewed_at: 2026-07-17T07:55:02Z
plans_reviewed: ['07-01-PLAN.md', '07-02-PLAN.md', '07-03-PLAN.md', '07-04-PLAN.md', '07-05-PLAN.md', '07-06-PLAN.md']
cycle: 1
---

# Cross-AI Plan Review — Phase 7

**Phase:** 07-cross-provider-multi-agent-orchestration  
**Reviewers:** Codex only (`--codex`)  
**Reviewed at:** 2026-07-17T07:55:02Z  
**Plans:** 07-01-PLAN.md, 07-02-PLAN.md, 07-03-PLAN.md, 07-04-PLAN.md, 07-05-PLAN.md, 07-06-PLAN.md

---

## Codex Review

# Phase 7 Plan Review

## Overall summary

The phase direction is sound: it extends the existing Task → coordinator → child-session path, preserves catalog-driven provider routing, and explicitly addresses the background-success race. The biggest planning issues are execution feasibility and test validity: Plan 01’s proposed RED tests cannot compile against a field that does not yet exist, Plan 03 places the authoritative credential gate after worktree side effects, and Plan 05 does not currently include the source/test visibility changes needed for an external integration test to drive the private subagent resolver or a real child sample.

## 07-01 — RED harness

### Summary

The plan correctly identifies the desired contracts, but its “intentional RED” mechanism conflicts with Rust compilation and the phase’s own discovery requirements.

### Strengths

- It correctly targets the actual Task construction point, which currently hardcodes `reasoning_effort: None` in [`task/mod.rs:305-316`](/home/cristian/bum/grok-build/crates/codegen/xai-grok-tools/src/implementations/grok_build/task/mod.rs:305).
- It correctly recognizes that background Tasks return success before coordinator completion: [`task/mod.rs:328-355`](/home/cristian/bum/grok-build/crates/codegen/xai-grok-tools/src/implementations/grok_build/task/mod.rs:328).
- Creating a dedicated `cross_provider_subagent.rs` target is appropriate; that file does not currently exist.

### Concerns

- **HIGH — proposed tests cannot compile.** `TaskToolInput` has no `reasoning_effort` member today; its fields end at `model` then `task_id` in [`xai-tool-types/src/task.rs:96-104`](/home/cristian/bum/grok-build/crates/common/xai-tool-types/src/task.rs:96). A test literal referencing the planned field will fail compilation, so `cargo test -- --list` cannot discover any tests. This defeats the stated Wave-1 requirement.
- **HIGH — deliberately failing tests will poison broad `p7_` execution.** The later plans run `cargo test ... p7_`; if Plan 01 adds normally compiled assertions that fail, this command remains red until all successors land. That is workable only if execution tooling explicitly supports expected-red commits, which the plan does not establish.
- **MEDIUM — the “Tool unknown model fallback” contract is inaccurate for the Task path.** The shell already rejects Tool-provenance unknown models before the later fallback block through `task_model_override_error` in [`handle_request.rs:28-43`](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/subagent/handle_request.rs:28), and Task validates explicit models even earlier in [`task/mod.rs:276-290`](/home/cristian/bum/grok-build/crates/codegen/xai-grok-tools/src/implementations/grok_build/task/mod.rs:276). The fallback at [`handle_request.rs:436-451`](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/subagent/handle_request.rs:436) is principally relevant to non-Tool/internal resolution paths.

### Suggestions

- Make Plan 01 a **green compile-only scaffold**: add test helpers and a smoke test, but defer assertions requiring the new field until Plan 02, or land the field/schema in Plan 01 and keep behavioral wiring for Plan 02.
- Separate “discovery scaffold” from “behavioral assertion” filters, so the phase gate never executes intentionally failing tests.
- Reframe the unknown-model test as a regression test of the existing Tool-provenance rejection, and reserve fallback changes for harness/internal provenance only if actually required.

### Risk assessment

**HIGH.** As written, Plan 01 can prevent the tools crate from compiling and blocks the intended validation flow.

---

## 07-02 — Task schema, effort wire, NL guidance

### Summary

This is the most direct plan and aligns well with current code, but it needs a complete call-site migration and stricter verification hygiene.

### Strengths

- It addresses the exact gap: `TaskToolInput` lacks effort while Task hardcodes `None` ([`task.rs:96-104`](/home/cristian/bum/grok-build/crates/common/xai-tool-types/src/task.rs:96), [`task/mod.rs:305-316`](/home/cristian/bum/grok-build/crates/codegen/xai-grok-tools/src/implementations/grok_build/task/mod.rs:305)).
- It reuses the established parser rather than inventing a vocabulary. `ReasoningEffort::from_str` supports `none`, `minimal`, `low`, `medium`, `high`, `xhigh`, and `max` ([`types.rs:829-845`](/home/cristian/bum/grok-build/crates/codegen/xai-grok-sampling-types/src/types.rs:829)).
- Adding model-facing guidance via `build_task_description` is correctly scoped: the builder already appends model guidance after the shared Task description ([`builder.rs:1280-1300`](/home/cristian/bum/grok-build/crates/codegen/xai-grok-agent/src/builder.rs:1280)).

### Concerns

- **MEDIUM — adding a struct field requires updating all Rust literals.** There are 42 `TaskToolInput { ... }` literals, largely in [`task/mod.rs`](/home/cristian/bum/grok-build/crates/codegen/xai-grok-tools/src/implementations/grok_build/task/mod.rs) plus [`xai-tool-types/src/task.rs:1126-1137`](/home/cristian/bum/grok-build/crates/common/xai-tool-types/src/task.rs:1126). The plan says to update selected tests but does not explicitly require a workspace search and full call-site migration.
- **MEDIUM — canonicalization is underspecified.** The parser maps `max` to `Xhigh` ([`types.rs:829-845`](/home/cristian/bum/grok-build/crates/codegen/xai-grok-sampling-types/src/types.rs:829)); carrying the raw `"max"` into overrides means the shell parses it again. Prefer storing `parsed.to_string()` so the runtime contract is canonical.
- **LOW — the proposed verify command masks failures.** It uses `... || cargo test ... || cargo test ...`, contrary to the phase’s own no-swallow policy. A failed first intended test could be hidden by a later command matching another test.

### Suggestions

- Add `rg -n "TaskToolInput \{"` to the implementation checklist and update every literal with `reasoning_effort: None`.
- Store canonical effort tokens after parsing, and test `max → xhigh`.
- Replace chained fallback verification with separate discovery-and-execute commands for the exact added builder test.

### Risk assessment

**MEDIUM.** The implementation is localized and uses good primitives, but incomplete literal migration will fail compilation.

---

## 07-03 — authoritative shell gate

### Summary

The plan chooses the right authority boundary, but its proposed placement is too late for its stated side-effect goal and it partially duplicates already-existing Tool model validation.

### Strengths

- Shell authority is appropriate because `handle_subagent_request` owns final child configuration and calls `spawn_session_on_thread` ([`handle_request.rs:1042-1055`](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/subagent/handle_request.rs:1042)).
- Reusing `missing_provider_gate_error` preserves the Phase 6 BYOK/usable-slot policy ([`config.rs:5602-5613`](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/config.rs:5602)).
- The plan correctly fixes the current invalid-effort behavior: malformed effort is merely logged and ignored today ([`handle_request.rs:472-486`](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/subagent/handle_request.rs:472)).
- The model resolver already isolates provider keys: Codex gets only `codex_session_owned`, while xAI gets only `session_key` ([`subagent/mod.rs:1006-1052`](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/subagent/mod.rs:1006)).

### Concerns

- **HIGH — credential gating after “effective model resolve” is after worktree work.** `handle_subagent_request` creates/rehydrates an isolated worktree before it resolves the effective model; worktree setup starts around [`handle_request.rs:217`](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/subagent/handle_request.rs:217), while model resolve is at [`handle_request.rs:424-432`](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/subagent/handle_request.rs:424). Thus the proposed gate is before child session launch but not before expensive/durable pre-spawn effects as claimed.
- **MEDIUM — extracting the Phase 6 helper is not a trivial move.** `provider_oauth_slot_usable` is private and depends on `MvpAgent`, including its live xAI `AuthManager` fallback ([`model_switch.rs:24-47`](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/handlers/model_switch.rs:24)). `SubagentSpawnContext` has `auth` and `auth_manager`-related state but is not an `MvpAgent`; the plan should specify a pure helper with explicit disk path/live-auth inputs.
- **MEDIUM — Tool unknown-model handling is already present.** As noted above, [`task_model_override_error`](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/subagent/handle_request.rs:28) already rejects Tool explicit unknown models. The plan should avoid changing that area merely to satisfy a test based on a stale premise.
- **MEDIUM — unsupported effort behavior needs catalog-level validation.** `model_supports_reasoning_effort` is only consulted today when an effort is present ([`handle_request.rs:472-477`](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/subagent/handle_request.rs:472)); the plan’s hard-fail decision is reasonable, but tests need a supported Codex fixture and an unsupported fixture to ensure it doesn’t reject valid Sol launches.

### Suggestions

- Split model resolution into a side-effect-free preflight helper and run provider gating before pending insertion/worktree creation. Reuse its output later rather than resolving twice.
- Extract a `provider_slot_usable(auth_path, provider, live_xai_auth)` helper in auth/config, then adapt both switch and spawn callers.
- Scope the unknown-model work to a regression test of existing Tool behavior unless a concrete harness path proves a remaining fallback.
- Add a test that a missing provider creates neither a worktree nor a child session.

### Risk assessment

**HIGH.** The security direction is right, but ordering currently permits unnecessary local side effects before a deliberate fail-closed decision.

---

## 07-04 — eager Task credential preflight

### Summary

This correctly closes the background “started” race, but its contract cannot fully cover omitted-model spawns with the resource shape proposed.

### Strengths

- It targets the actual race: background mode spawns a task and immediately returns the “started” text ([`task/mod.rs:328-355`](/home/cristian/bum/grok-build/crates/codegen/xai-grok-tools/src/implementations/grok_build/task/mod.rs:328)).
- Injecting a shell-owned resource is preferable to having `xai-grok-tools` read `auth.json`; current Task model validation is injected from shell in [`agent_rebuild.rs:310-319`](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/session/agent_rebuild.rs:310).
- It preserves shell authority as a second line of defense.

### Concerns

- **HIGH — `Fn(model_slug)` cannot validate omitted-model inheritance.** Task intentionally turns omitted `model` into `None` ([`task/mod.rs:166-181`](/home/cristian/bum/grok-build/crates/codegen/xai-grok-tools/src/implementations/grok_build/task/mod.rs:166)), while the proposed gate only accepts a slug. This leaves the stated “same-provider-if-somehow-missing” and omit-model requirement unenforceable at Task time unless the resource also carries the current parent/effective model.
- **MEDIUM — potential semantic drift from the shell gate.** Plan 04’s closure independently resolves catalog/auth status while Plan 03 owns final model resolution, including role/config model pins. The actual child model may differ from explicit Task model because the shell precedence rules include agent/config pins and inheritance ([`subagent/mod.rs:768-829`](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/subagent/mod.rs:768)). Preflighting only `Task.model` can approve one provider while the authoritative child resolves to another.
- **MEDIUM — failure for a missing resource needs compatibility analysis.** Existing unit setups only insert `TaskModelValidator` ([`task/mod.rs:837`](/home/cristian/bum/grok-build/crates/codegen/xai-grok-tools/src/implementations/grok_build/task/mod.rs:837)); making an absent credential resource fail closed will require updating every Task test fixture and any non-shell tool bridge.

### Suggestions

- Define a preflight request/resource that receives the full Task context and returns the **effective target provider/model**, not merely an explicit slug.
- Prefer an async backend `preflight_spawn` RPC if the effective model requires shell-only role/pin resolution; it avoids duplicating precedence in the tools crate.
- Explicitly migrate all resource fixtures and non-interactive construction paths.

### Risk assessment

**HIGH.** Without effective-model-aware preflight, the UX gate can disagree with the authoritative gate—the exact divergence the plan is trying to prevent.

---

## 07-05 — dual-token routing proofs

### Summary

The required observable Authorization proof is exactly the right bar, but the plan needs an explicit test seam and stronger acceptance language.

### Strengths

- It correctly rejects resolve-only checks as insufficient. Existing routing integration tests already demonstrate mock HTTP and distinct fixture tokens ([`provider_routing.rs:1-35`](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/tests/provider_routing.rs:1), [`provider_routing.rs:40-79`](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/tests/provider_routing.rs:40)).
- It appropriately tests both directions and missing-child-slot no-request behavior.
- It retains the important invariant that the resolver never assigns parent xAI credentials to Codex ([`subagent/mod.rs:1027-1035`](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/subagent/mod.rs:1027)).

### Concerns

- **HIGH — the planned integration target cannot directly call the relevant helpers.** `resolve_model_override_to_config` is private ([`subagent/mod.rs:1006`](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/subagent/mod.rs:1006)), and `handle_subagent_request` is `pub(crate)` ([`handle_request.rs:59`](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/subagent/handle_request.rs:59)). An external `tests/cross_provider_subagent.rs` crate cannot use either. The plan modifies only test files, so it does not establish a public/test-only seam or move the tests inside the crate.
- **MEDIUM — “at least one direction mandatory” conflicts with the stated D-12 requirement.** The phase decision and plan must-haves require both Grok→Codex and Codex→Grok. Allowing one outbound-header direction weakens AGENT-04 and should be a blocker, not an acceptable completion path.
- **MEDIUM — parent stability cannot be shown by a “resolve-only parent handle unchanged” test.** That proves no mutation in a helper, not that real spawn leaves parent model state untouched. The test needs an actual spawn path.

### Suggestions

- Add a test seam to Plan 03/05: either a crate-internal integration-style test module under `src/agent/subagent/tests`, or a deliberate public test harness that exposes a real child spawn/sampling operation.
- Make outbound Authorization and base URL assertions mandatory in **both** directions.
- Require a real parent `ChatState`/session assertion after spawn, not a resolver-only proxy.

### Risk assessment

**HIGH.** The desired proof is good, but the planned files cannot currently execute it.

---

## 07-06 — regression gate and documentation

### Summary

The final gate is well structured and properly treats same-provider behavior as a hard regression boundary, but it inherits the earlier test-seam and red-test problems.

### Strengths

- It uses existing focused regression tests rather than an expensive unfiltered shell suite. The referenced names exist: `reasoning_effort_explicit_overrides_role` ([`tests/mod.rs:1294`](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/subagent/tests/mod.rs:1294)), `role_default_used_when_no_explicit_override` ([`tests/mod.rs:1236`](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/subagent/tests/mod.rs:1236)), `persona_resolved_from_config` ([`tests/mod.rs:1351`](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/subagent/tests/mod.rs:1351)), and `resume_model_pinning_overrides_default_resolution` ([`tests/rest.rs:2038`](/home/cristian/bum/grok-build/crates/codegen/xai-grok-shell/src/agent/subagent/tests/rest.rs:2038)).
- The per-filter discovery helper is a strong defense against vacuous filters.
- It appropriately excludes live OAuth from the phase gate.

### Concerns

- **MEDIUM — the gate’s completion claim is stronger than the planned proof.** AGENT-04 requires child turns to use child credentials/backend. Unless Plan 05 adds the needed in-crate seam, the documented `p7_isolation` filter will not be a real child-turn proof.
- **MEDIUM — “all AGENT-01..06 automated proofs green” is incompatible with Plan 01’s intentional-red approach unless the test strategy is corrected first.**
- **LOW — role/persona tests verify precedence resolution, not full same-provider spawn/resume/parent-child routing.** They are valuable regressions, but AGENT-01 also needs at least one actual same-provider spawn/resume lifecycle assertion.

### Suggestions

- Make Plan 06 contingent on a checklist that verifies both-direction wire tests are real child samples, not resolver tests.
- Add one same-provider lifecycle test that exercises coordinator spawn plus resume/result routing.
- Do not mark Nyquist compliant until Plan 01’s scaffolding is converted to normal green, compiling tests.

### Risk assessment

**MEDIUM.** The phase-gate design is strong, but it can only certify what the earlier plans genuinely test.

## Bottom line

Approve the architecture, but revise before execution. The minimum changes are:

1. Replace Plan 01’s non-compiling/intentional-red tests with compiling scaffolds.
2. Move or precompute the authoritative credential gate before worktree side effects.
3. Make Task eager preflight resolve the actual effective child model, including omitted-model/role-pin cases.
4. Add an in-crate or public test seam so Plan 05 can run real child sampling and assert outbound Authorization in both directions.

---

## Consensus Summary

Single reviewer (Codex only). Plan-level consensus is the Codex verdict; treat HIGH items as blocking for `/gsd-plan-phase 7 --reviews`.

### Agreed Strengths
- Phase extends existing Task → coordinator → child-session path rather than inventing a parallel multi-agent API.
- Correctly targets background Task "started" race and catalog-driven provider routing.
- Shell authority for spawn gate is the right boundary; reuses Phase 6 `missing_provider_gate_error` / usable-slot policy.
- Plan 06 Nyquist discovery helper and same-provider regression anchors are well structured.
- Dual fake-token Authorization + base URL proofs are the right bar for AGENT-04/05.

### Agreed Concerns (blocking / high priority)
1. **Plan 01 intentional-RED vs Rust compile/discovery** — tests referencing `TaskToolInput.reasoning_effort` cannot compile before Plan 02; failing `p7_` tests poison later wave verification.
2. **Plan 03 gate placement after worktree side effects** — authoritative credential gate must run before worktree create/rehydrate, not only before `spawn_session_on_thread`.
3. **Plan 04 eager preflight is slug-only** — cannot cover omit-model inheritance / role pins; risks UX gate ≠ authoritative shell gate.
4. **Plan 05 missing test seam** — `resolve_model_override_to_config` is private and `handle_subagent_request` is `pub(crate)`; external `tests/cross_provider_subagent.rs` cannot drive real child sampling/Authorization assertions without an in-crate or public harness.
5. **Plan 01 dual HIGH on red strategy** — need green compile-only scaffold (or land schema early) and separate discovery filters from intentional failures.

### Actionable Non-HIGH (incorporate in PLAN.md)
- Full `TaskToolInput { ... }` call-site migration + canonical effort tokens (`max` → `xhigh`) in Plan 02.
- Extract pure `provider_slot_usable(...)` helper (not MvpAgent-bound private switch helper).
- Scope Tool unknown-model work to regression of existing rejection, not stale fallback premise.
- Mandatory both-direction Authorization proofs (do not weaken D-12 to "one direction").
- Parent stability after real spawn, not resolve-only proxy.
- Unsupported-effort fixtures (supported Sol + unsupported model).
- Resource-fixture migration when credential preflight resource becomes required.
- One same-provider spawn/resume lifecycle assertion beyond pure precedence unit tests.
- Drop `||` verification chains that swallow failures.
- Plan 02 verification must not use fallback OR chains that hide the intended test failure.

### Divergent Views
N/A — single reviewer (Codex).

### Bottom line
Architecture is sound; **revise Plans 01, 03, 04, and 05 before execution**. Minimum fixes: compiling scaffolds, pre-worktree gate, effective-model-aware Task preflight, and real dual-direction wire test seam.

