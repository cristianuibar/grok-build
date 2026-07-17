---
phase: 7
reviewers: [codex]
reviewed_at: 2026-07-17T08:03:38Z
plans_reviewed: ['07-01-PLAN.md', '07-02-PLAN.md', '07-03-PLAN.md', '07-04-PLAN.md', '07-05-PLAN.md', '07-06-PLAN.md']
cycle: 2
prior_cycle: 1
prior_reviewed_at: 2026-07-17T07:55:02Z
replan_commit: d9a9a51
current_high: 1
current_actionable: 5
---

# Cross-AI Plan Review — Phase 7

**Phase:** 07-cross-provider-multi-agent-orchestration  
**Reviewers:** Codex only (`--codex`)  
**Cycle:** 2 (convergence after replan `d9a9a51`)  
**Reviewed at:** 2026-07-17T08:03:38Z  
**Plans:** 07-01-PLAN.md, 07-02-PLAN.md, 07-03-PLAN.md, 07-04-PLAN.md, 07-05-PLAN.md, 07-06-PLAN.md

---

## Cycle status (open vs resolved)

| ID | Cycle | Severity | Plan | Status | Notes |
|----|-------|----------|------|--------|-------|
| C1-H1 | 1 | HIGH | 01 | **RESOLVED** | Green compile-safe scaffold; no `TaskToolInput.reasoning_effort`; no intentional-red under `p7_` |
| C1-H2 | 1 | HIGH | 01 | **RESOLVED** | Expected-green protocol; Plans 02–05 add green tests with behavior |
| C1-H3 | 1 | HIGH | 03 | **RESOLVED** | Pre-worktree gate + side-effect-free effective-model preflight in plan text |
| C1-H4 | 1 | HIGH | 04 | **PARTIAL → C2-H1** | Effective-model request is specified, but default sync `Fn` cannot match async shell resolve |
| C1-H5 | 1 | HIGH | 05 | **RESOLVED** | In-crate/public test seam mandatory; both-direction Authorization mandatory |
| C1-M1 | 1 | MEDIUM | 02 | **RESOLVED** | Full `TaskToolInput {` migration + `max→xhigh` canonical store in plan |
| C1-M2 | 1 | MEDIUM | 03 | **RESOLVED** | Pure `provider_slot_usable` with explicit inputs (not MvpAgent-only) |
| C1-M3 | 1 | MEDIUM | 03 | **RESOLVED** | Tool unknown-model scoped to existing `task_model_override_error` regression |
| C1-M4 | 1 | MEDIUM | 05 | **RESOLVED** | Both-direction Authorization hard bar; no one-direction waiver |
| C1-M5 | 1 | MEDIUM | 05 | **RESOLVED** | Parent stability after real spawn (not resolve-only) |
| C1-M6 | 1 | MEDIUM | 03 | **RESOLVED** | Supported Sol + unsupported effort fixtures required |
| C1-M7 | 1 | MEDIUM | 04 | **PARTIAL** | Fixture migration required; full constructor inventory still weak (see C2-M3) |
| C1-M8 | 1 | MEDIUM | 06 | **PARTIAL** | Lifecycle “if cheap” remains soft (see C2-L2) |
| C1-L1 | 1 | LOW | 02 | **OPEN** | Verify still has `\|\|` swallow on xai-tool-types (C2-M1) |
| C2-H1 | 2 | HIGH | 04 | **OPEN** | Sync credential-gate resource cannot deliver live “same effective model” as async shell |
| C2-M1 | 2 | MEDIUM | 02 | **OPEN** | Task 1 verify still uses `\|\|` fallback that can hide intended test failure |
| C2-M2 | 2 | MEDIUM | 03 | **OPEN** | Gate-before-pending must be explicit in task ordering + no-pending entry assert |
| C2-M3 | 2 | MEDIUM | 04 | **OPEN** | Enumerate all Task resource builders / non-shell bridges for fail-closed resource |
| C2-M4 | 2 | MEDIUM | 05 | **OPEN** | Seam still choice-shaped; need minimal harness contract (spawn → one sample → cancel) |
| C2-M5 | 2 | MEDIUM | 06 | **OPEN** | Phase gate inherits Plan 04 feasibility gap for AGENT-05 eager UX proof |
| C2-L1 | 2 | LOW | 01 | **OPEN** | Dual-fixture smoke must not be mistaken for routing/Authorization proof |
| C2-L2 | 2 | LOW | 06 | **OPEN** | Same-provider lifecycle “if cheap” too weak — require named test or cite existing |
| C2-L3 | 2 | LOW | 02 | **OPEN** | Clarify whether parser tokens `none`/`minimal` are allowed Tool values |

**Counts for convergence (unresolved only):**  
`current_high=1` · `current_actionable=5` (C2-M1..C2-M5; LOWs tracked but not in actionable count unless blocking)

---

## Codex Review (Cycle 2)

# Phase 7 Plan Review — Convergence Cycle 2

## Summary

The revision materially improves the plan set: it fixes the compile-poisoning RED strategy, explicitly moves the authoritative gate ahead of worktree work, and makes real child-sampling tests a stated requirement. The remaining blocker is Plan 04’s eager-preflight implementation shape: its proposed synchronous resource cannot, as written, resolve the same live effective model as the async shell path for omit-model and role-pin cases.

## Cycle 1 Resolution Check

| Prior HIGH | Status | Evidence |
|---|---|---|
| Plan 01 tests could not compile before schema exists; intentional RED poisoned `p7_` | **RESOLVED** | 07-01 explicitly forbids `TaskToolInput.reasoning_effort` references and all intentional-red/ignored `p7_` tests; Plan 02 owns field + behavior tests together. This matches the current missing field at `crates/common/xai-tool-types/src/task.rs:96-104`. |
| Plan 03 gate ran after worktree effects | **RESOLVED** | 07-03 requires a side-effect-free effective-model preflight and gate before worktree creation/rehydration and pending insertion. This targets the current ordering: pending insertion at `handle_request.rs:98-118`, worktree work at `handle_request.rs:245-366`, model resolution only at `handle_request.rs:423-429`. |
| Plan 04 slug-only preflight missed inheritance/role pins | **PARTIAL** | 07-04 correctly requires an effective-model-aware request with omitted-model and role-pin coverage. However, its specified `Arc<dyn Fn(...) -> Option<String>>` resource is synchronous, while the canonical inherit path reads live parent chat state asynchronously at `subagent/mod.rs:897-902` and `resolve_effective_model_config` is async at `subagent/mod.rs:775-850`. |
| Plan 05 lacked a callable test seam | **RESOLVED** | 07-05 now mandates an in-crate or deliberately exposed harness before claiming wire assertions. This is necessary because `handle_subagent_request` is `pub(crate)` at `handle_request.rs:59` and `resolve_model_override_to_config` is private at `subagent/mod.rs:1006`. |
| Plan 01 needed green scaffold and separate discovery from future tests | **RESOLVED** | 07-01’s expected-green protocol requires all `p7_` tests to compile and pass today, then requires Plans 02–05 to add green behavior tests only alongside implementation. |

## 07-01 — Green scaffold

### Strengths

- **LOW risk:** The revised scaffold is genuinely compile-safe. It avoids the nonexistent field while locking the current `reasoning_effort: None` behavior that exists at `crates/codegen/xai-grok-tools/src/implementations/grok_build/task/mod.rs:303-316`.
- It correctly treats Tool unknown-model rejection as a regression, not a new fallback fix. The shell already checks Tool-provenance overrides before the fallback block at `handle_request.rs:234-243`; fallback occurs later at `handle_request.rs:435-449`.
- The planned external harness appropriately limits itself to public APIs until Plan 05 establishes an internal seam.

### Concerns

- **LOW — “dual-fixture smoke” is not a routing proof.** Current child route construction is private at `subagent/mod.rs:1006-1043`; Plan 01 should label its fixture test only as infrastructure smoke, as it cannot yet demonstrate child configuration or headers.

### Suggestions

- Keep Plan 01’s `p7_` names scoped to scaffold/current behavior, and reserve isolation names for Plan 05’s real child-sampling tests.

### Risk Assessment

**LOW.** The prior compile/discovery blocker is addressed.

## 07-02 — Task effort contract

### Strengths

- It targets the exact product gap: `TaskToolInput` currently has `model` but no effort field at `crates/common/xai-tool-types/src/task.rs:96-104`, while Task hardcodes `reasoning_effort: None` at `task/mod.rs:308`.
- Canonicalizing aliases before sending the override is sound; the current shell reparses raw strings and warns/ignores failures at `handle_request.rs:472-485`.
- It explicitly requires every struct literal to be migrated. There are currently 45 Rust locations matching `TaskToolInput {` across five source files.

### Concerns

- **MEDIUM — verification still contains an exit-code-swallowing fallback.** Plan 02’s Task 1 verify ends with `cargo test -p xai-tool-types --lib ... || cargo test -p xai-tool-types ...`. That can hide a failure of the intended library test invocation, despite the plan’s stated no-OR policy.

### Suggestions

- Replace the fallback with one confirmed command after inspecting whether `xai-tool-types` has a lib target; if alternate commands are necessary, run both independently under `set -e`.
- State whether `none` and `minimal`, which the existing `ReasoningEffort` parser accepts, are intentionally allowed Tool values or should be rejected despite parser support.

### Risk Assessment

**MEDIUM.** The implementation is localized and well-scoped, but verification hygiene needs correction.

## 07-03 — Authoritative shell gate

### Strengths

- This directly fixes the prior gate-placement flaw: the plan explicitly requires the effective-model decision and credential gate before the current worktree creation/rehydration block at `handle_request.rs:245-366`.
- Extracting a shared slot-usability helper is appropriate because the current switch helper is private and `MvpAgent`-bound at `agent/handlers/model_switch.rs:24-47`.
- It preserves the correct provider-routing primitive: Codex override resolution intentionally passes no parent xAI key at `subagent/mod.rs:1019-1034`.
- It includes the necessary supported and unsupported reasoning-effort fixtures. Current behavior only attempts parsing when the model supports effort, and otherwise silently skips the override at `handle_request.rs:472-485`.

### Concerns

- **MEDIUM — the required preflight must also occur before pending registration, not merely before worktree work.** The current coordinator inserts a pending child at `handle_request.rs:98-118`, well before worktree creation. The plan’s must-haves say this correctly, but the implementation checklist should make moving this boundary explicit; otherwise a rejected child can leave transient pending-state/notifications.

### Suggestions

- Make the first executable steps: resolve definition/role/resume source → resolve effective model → gate → insert pending → worktree/session work.
- Add a test that missing credentials leave both no worktree and no pending/active child entry.

### Risk Assessment

**MEDIUM.** The former HIGH is resolved in plan text, but correctness depends on fully moving the gate ahead of both pending state and filesystem side effects.

## 07-04 — Eager Task preflight

### Strengths

- It correctly recognizes that the Task background path returns immediately after `tokio::spawn` at `task/mod.rs:328-355`, so a shell-only rejection cannot meet the UX requirement.
- It explicitly requires omit-model and role-pin coverage, rather than accepting the previous slug-only closure.
- It correctly keeps tools away from direct auth-store I/O and retains the shell as authority.

### Concerns

- **HIGH — the selected resource contract cannot currently deliver “same effective model” semantics.** The plan proposes `TaskProviderCredentialGate` as `Arc<dyn Fn(TaskCredentialPreflightRequest) -> Option<String>>`, modeled on the synchronous validator at `task/types.rs:772-784`. But the authoritative inherited-model path reads `ChatStateHandle` asynchronously (`subagent/mod.rs:897-902`), and full effective resolution is async (`subagent/mod.rs:775-850` / `resolve_effective_model_config`). A synchronous closure installed during `AgentRebuildSpec::build_agent`—where the existing validator is installed at `session/agent_rebuild.rs:310-317`—cannot query the live parent session at call time. It also has no demonstrated access to the shell’s role/model override state used by `handle_subagent_request`. Plan text still **defaults** to Resources gate and treats `SubagentBackend.preflight_spawn` as discretionary fallback only.
- **MEDIUM — fail-closed on an absent new resource can break non-session tool construction.** Existing Task test setup inserts the backend and session resources but not the model validator in some paths, e.g. `task/mod.rs:1120-1127`; only selected harnesses install `TaskModelValidator` at `task/mod.rs:837` and `946`. The plan says to migrate validator fixtures, but must enumerate all Task resource builders and non-shell bridge paths.

### Suggestions

- Make an async `SubagentBackend::preflight_spawn` RPC the **required** design, not a discretionary fallback. It can use the exact coordinator context, live parent model, resume source, role/persona resolution, and shared slot-usability function.
- If retaining a resource, make it an async trait/function returning a future and prove it receives a live model source—not a model snapshot captured at agent-build time.
- Add an explicit compatibility inventory for every Task bridge/resource constructor.

### Risk Assessment

**HIGH.** The desired behavior is correct, but the planned synchronous mechanism does not yet have a credible path to match the authoritative async resolver.

## 07-05 — Dual-token isolation proofs

### Strengths

- It now makes both directions mandatory and requires real outbound Authorization plus route assertions, which is the right proof bar.
- It correctly requires a test seam before using private shell code; this aligns with the present visibility boundaries at `handle_request.rs:59` and `subagent/mod.rs:1006`.
- The plan’s desired behavior aligns with session reconstruction: xAI bearer resolution is only attached for xAI catalog facts at `sampler_turn.rs:278-284`, while Codex refresh is provider- and first-party-host-gated at `sampler_turn.rs:294-313`.
- It properly treats `SamplingConfig` key-prefix checks as supplemental, not sufficient. Child credentials are seeded from the effective sampling config at `handle_request.rs:761-773`.

### Concerns

- **MEDIUM — the seam is still an implementation choice rather than a specified test API/lifecycle.** “In-crate tests or a public harness” is enough direction, but the plan should specify the minimal harness contract: spawn child, wait until a single sample reaches a mock server, return parent model state, and cleanly cancel/join the child. Without that, the real-spawn proof may degrade into resolver-only testing due to harness complexity.

### Suggestions

- Define the exact in-crate helper signature and timeout/cancellation behavior in Plan 05.
- Ensure mock URLs are configured through the same model route/endpoints used by the child, rather than replacing the route after resolution.

### Risk Assessment

**MEDIUM.** The prior seam blocker is resolved; execution risk remains in building a reliable real-child sampling harness.

## 07-06 — Regression and phase gate

### Strengths

- It correctly makes same-provider regression a hard gate, anchored in existing effort, role, persona, and resume tests.
- Per-filter discovery guards are strong and prevent an empty filter from producing a false green.
- It correctly requires both named directional isolation tests before AGENT-04 can pass.

### Concerns

- **MEDIUM — Plan 06 inherits Plan 04’s unresolved feasibility issue.** A green phase gate cannot establish AGENT-05’s background UX behavior if the eager preflight cannot actually resolve the live effective child model.
- **LOW — the “lifecycle if cheap” qualifier is too weak for AGENT-01.** Existing tests can validate precedence, but the real coordinator has stateful behavior such as pending insertion at `handle_request.rs:98-118` and child spawning at `handle_request.rs:1049-1058`. Require one lifecycle test or document a concrete existing lifecycle test that already exercises this path.

### Suggestions

- Make Phase Gate PASS contingent on Plan 04 using an authoritative-equivalent async preflight mechanism.
- Upgrade same-provider lifecycle coverage from “if cheap” to a required named test, unless a cited existing test covers coordinator spawn, completion, and resume routing.

### Risk Assessment

**MEDIUM.** Gate design is strong, but it should not certify the phase until the eager-preflight design is made implementable.

## Overall Risk Assessment

**HIGH until Plan 04 is revised.** Four of the five prior HIGH findings are addressed in the revised plan text. The remaining effective-model-aware eager preflight is conceptually correct but not executable with the specified synchronous resource against the current async parent-state and model-resolution architecture. The minimum convergence change is to require an async coordinator/backend preflight—or an equivalently async resource—sharing the authoritative effective-model resolver.

---

## Consensus Summary (Cycle 2)

Single reviewer (Codex only). Plan-level consensus is the Codex verdict grounded against current source.

### Cycle 1 HIGHs — resolution verdict
1. **Plan 01 compile / intentional-RED** — **RESOLVED** in PLAN.md (green-only protocol + no pre-schema field refs).
2. **Plan 03 gate-after-worktree** — **RESOLVED** in PLAN.md (preflight + gate before worktree; source still inserts pending early — residual MEDIUM C2-M2).
3. **Plan 04 slug-only preflight** — **PARTIAL**: request is effective-model-aware on paper; **mechanism still HIGH** (C2-H1 sync vs async).
4. **Plan 05 test seam** — **RESOLVED** (in-crate/public seam required; both-dir Authorization mandatory).
5. **Plan 01 dual RED / discovery** — **RESOLVED**.

### Agreed Strengths
- Wave-1 green scaffold no longer poisons later `p7_` gates.
- Authoritative shell gate + pure `provider_slot_usable` extraction is the right authority boundary.
- Dual fake-token Authorization both directions remains the correct AGENT-04 bar; Plan 05 no longer allows one-direction waiver.
- Plan 06 per-subgroup discovery + green-only language is solid once product proofs exist.

### Current HIGH Concerns (unresolved)
1. **C2-H1 / Plan 04 — sync `TaskProviderCredentialGate` cannot match async shell effective-model resolve.**  
   Evidence: `read_parent_sampling_config` / inherit path uses `ChatStateHandle` async at `subagent/mod.rs:897-902`; `resolve_effective_model_config` is async at `subagent/mod.rs:840+`; Task resource pattern is sync `Arc<Fn>` like `TaskModelValidator` at `task/types.rs:772-784`; install site is agent rebuild snapshot (`agent_rebuild.rs:310-317`). Plan still **defaults** to sync Resources gate and treats `SubagentBackend.preflight_spawn` as optional.  
   **Required replan:** make async backend/coordinator preflight (or async resource with live parent state) the **required** design so omit-model + role pins agree with shell authority.

### Current Actionable Non-HIGH Concerns (unresolved)
1. **C2-M1 / Plan 02** — Task 1 verify still ends with `cargo test -p xai-tool-types --lib ... || cargo test -p xai-tool-types ...` (line ~139); drop OR-chain; pick one command under `set -e`.
2. **C2-M2 / Plan 03** — Make gate-before-`insert_pending` explicit in task ordering steps; add named assert that missing credentials leave **no pending/active child entry** (not only no worktree).
3. **C2-M3 / Plan 04** — Inventory every Task resource builder / non-shell bridge that must install the new gate under fail-closed-on-missing-resource (not only “grep validator fixtures”).
4. **C2-M4 / Plan 05** — Specify minimal harness contract: spawn → one outbound sample against mock → capture Authorization/host + parent model → cancel/join; avoid resolve-only degradation.
5. **C2-M5 / Plan 06** — Phase-gate PASS for AGENT-05 must depend on Plan 04’s implementable eager path (async-equivalent), not only green filter names.

### Tracked LOW (non-blocking)
- C2-L1 Plan 01 smoke ≠ isolation proof (label only).
- C2-L2 Plan 06 same-provider lifecycle: require named test or cite existing coordinator path.
- C2-L3 Plan 02: document `none`/`minimal` Tool acceptance policy.

### Divergent Views
N/A — single reviewer (Codex).

### Bottom line
**Do not execute yet.** Architecture remains sound; cycle-1 structural blockers on Plans 01/03/05 are fixed in text. **Replan Plan 04** so eager preflight is async/authoritative-equivalent (not sync Fn + discretionary RPC). Fold C2-M1..M5 into Plans 02/03/04/05/06 in the same replan pass, then re-run cycle 3.

---

## Cycle 1 archive (for audit)

Cycle 1 full write-up (2026-07-17T07:55:02Z) established five HIGHs (Plan 01 compile/RED ×2, Plan 03 worktree order, Plan 04 slug-only, Plan 05 test seam). Replan commit `d9a9a51` addressed those in PLAN.md text; cycle 2 verified four RESOLVED and one PARTIAL escalated to C2-H1 on mechanism feasibility. Historical cycle-1 body is retained in git history for this file prior to cycle 2 rewrite; the resolution table above is authoritative for open vs closed.
