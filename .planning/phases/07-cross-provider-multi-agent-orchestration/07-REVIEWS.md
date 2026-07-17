---
phase: 7
reviewers: [codex]
reviewed_at: 2026-07-17T08:10:00Z
plans_reviewed: ['07-01-PLAN.md', '07-02-PLAN.md', '07-03-PLAN.md', '07-04-PLAN.md', '07-05-PLAN.md', '07-06-PLAN.md']
cycle: 3
prior_cycle: 2
prior_reviewed_at: 2026-07-17T08:03:38Z
replan_commit: 3b27a6a
current_high: 0
current_actionable: 0
plan_patch: c3-m1-m2-fold
---

# Cross-AI Plan Review — Phase 7

**Phase:** 07-cross-provider-multi-agent-orchestration  
**Reviewers:** Codex only (`--codex`)  
**Cycle:** 3 (final convergence after replan `3b27a6a`)  
**Reviewed at:** 2026-07-17T08:10:00Z  
**Plans:** 07-01-PLAN.md, 07-02-PLAN.md, 07-03-PLAN.md, 07-04-PLAN.md, 07-05-PLAN.md, 07-06-PLAN.md

---

## Cycle status (open vs resolved)

| ID | Cycle | Severity | Plan | Status | Notes |
|----|-------|----------|------|--------|-------|
| C1-H1 | 1 | HIGH | 01 | **RESOLVED** | Green compile-safe scaffold |
| C1-H2 | 1 | HIGH | 01 | **RESOLVED** | Expected-green protocol |
| C1-H3 | 1 | HIGH | 03 | **RESOLVED** | Pre-worktree gate + effective-model preflight |
| C1-H4 | 1 | HIGH | 04 | **RESOLVED** via C2-H1 | Escalated to mechanism feasibility; fixed in `3b27a6a` |
| C1-H5 | 1 | HIGH | 05 | **RESOLVED** | In-crate/public test seam + both-dir Authorization |
| C1-M1..M6 | 1 | MEDIUM | * | **RESOLVED** | Prior cycles |
| C1-M7 | 1 | MEDIUM | 04 | **RESOLVED** via C2-M3 | Full backend/fixture inventory required |
| C1-M8 | 1 | MEDIUM | 06 | **RESOLVED** via C2-L2 | Named/cited lifecycle (still LOW residual C2-L2) |
| C1-L1 | 1 | LOW | 02 | **RESOLVED** via C2-M1 | No OR-chain on cargo execute |
| C2-H1 | 2 | HIGH | 04 | **RESOLVED** | Required async `SubagentBackend.preflight_spawn` + live parent resolve; sync slug Fn non-goal |
| C2-M1 | 2 | MEDIUM | 02 | **RESOLVED** | Separate `set -e` cargo commands; no `A \|\| B` execute |
| C2-M2 | 2 | MEDIUM | 03 | **RESOLVED** | Gate before `insert_pending`; named no-pending/active assert |
| C2-M3 | 2 | MEDIUM | 04 | **RESOLVED** | Mandatory inventory + fail-closed missing backend |
| C2-M4 | 2 | MEDIUM | 05 | **RESOLVED** | Minimal harness contract spawn→sample→cancel/join |
| C2-M5 | 2 | MEDIUM | 06 | **RESOLVED** via C3-M2 fold | Must-haves/action + Task 2 `<verify>` discover+execute full subgroup list |
| C2-L1 | 2 | LOW | 01 | **OPEN** | Dual-fixture smoke ≠ isolation proof (label only) |
| C2-L2 | 2 | LOW | 06 | **OPEN** | Lifecycle must be named/cited real coordinator path (not resolve-only helpers) |
| C2-L3 | 2 | LOW | 02 | **RESOLVED** | `none`/`minimal` allowed when parser accepts |
| C3-M1 | 3 | MEDIUM | 04 | **RESOLVED** (plan patch) | Preflight request = full spawn-resolution input (`resume_from` + runtime overrides/persona) or shared resolve helper |
| C3-M2 | 3 | MEDIUM | 06 | **RESOLVED** (plan patch) | Task 2 `<verify>` discover+execute shell preflight, both isolation dirs, `p7_missing`, `p7_parent_model`, `p7_tool` |

**Counts for convergence (unresolved HIGH + actionable MEDIUM only):**  
`current_high=0` · `current_actionable=0` (C3-M1/C3-M2 folded into PLAN.md; LOWs tracked but not in actionable count)

**Convergence accepted (post cycle-3 plan patch):** residual MEDIUMs folded into `07-04-PLAN.md` / `07-06-PLAN.md` without a full replan cycle. Safe to execute Phase 7.

---

## Focus answers (cycle 3)

### 1. Is C2-H1 fully RESOLVED in Plan 04 text?

**Yes — RESOLVED.**

Plan 04 replan (`3b27a6a`) makes async coordinator/backend preflight **non-negotiable**:

- must_haves: required async `SubagentBackend.preflight_spawn` (or proven async live-parent equivalent); sync slug Fn **not** production design
- “Required design (C2-H1)” section: oneshot RPC via `ChannelBackend` / `SubagentEvent`, handler uses live `ChatStateHandle` + same path as `resolve_effective_model_config`
- Task 1/2 actions: implement trait method, await before bg started notice, shell handler shares Plan 03 pure helpers; SUMMARY must document async design

This matches source reality: inherit path is async (`subagent/mod.rs:840+`, `read_parent_sampling_config` ~897); Task bg returns after `tokio::spawn` (`task/mod.rs:328+`); existing async oneshot pattern on `SubagentBackend` (`backend.rs:56` `validate_type`).

### 2. Are C2-M1..M5 incorporated?

| ID | Incorporated? | Where |
|----|---------------|--------|
| C2-M1 | **Yes** | 07-02 hard rule + Task 1/2 verify: independent cargo lines under `set -euo pipefail` |
| C2-M2 | **Yes** | 07-03 must_haves + action ordering + `p7_spawn_missing_provider_leaves_no_pending_or_active_child` |
| C2-M3 | **Yes** | 07-04 inventory step + SUMMARY table requirement |
| C2-M4 | **Yes** | 07-05 “Minimal harness contract” + mandatory first deliverable |
| C2-M5 | **Yes** (via C3-M2 fold) | 07-06 must_haves + Task 2 `<verify>` discover+execute shell preflight + Plan 05 filters |

### 3. Remaining HIGH or actionable MEDIUM?

**No HIGH. No actionable MEDIUM.** C3-M1/C3-M2 folded into Plan 04/06 via plan patch (not full replan). LOWs only (C2-L1, C2-L2). **Execute Phase 7.**

---

## Codex Review (Cycle 3)

# Phase 7 Plan Review — Convergence Cycle 3 (final)

## Summary

The revised Phase 7 plan is substantially stronger and is ready to execute with two remaining medium-priority refinements. It correctly targets the real gaps: today the Task tool hardcodes `reasoning_effort: None` and returns the background “started” response immediately after scheduling spawn (`task/mod.rs:305`, `task/mod.rs:324`); meanwhile the shell inserts pending state and can create a worktree before resolving the effective child model (`handle_request.rs:98`, `handle_request.rs:245`, `handle_request.rs:423`). Plans 03–05 directly address those defects with an authoritative pre-side-effect gate, an async preflight, and real child-sample routing tests.

## Strengths

- Plan 01’s green-only scaffold is appropriate. It records the current hardcoded effort behavior rather than adding intentionally failing tests. That precisely matches the current implementation (`task/mod.rs:305`).

- Plan 02 correctly uses the existing Task boundary. `TaskToolInput` currently ends with `model` then `task_id` (`task.rs:96`), while the runtime override already has an effort field (`types.rs:84`). Adding the schema field and threading canonical parsed effort is a localized, compatible change.

- Plan 03 fixes the right authority and ordering. The current `Task`-provenance unknown-model defense already exists before the worktree section (`handle_request.rs:234`); the proposed work should preserve it rather than incorrectly “fixing” a fallback path that applies later. Moving effective-model resolution and the credential gate before `insert_pending` is the right design.

- Reusing the Phase 6 gate is sound. `missing_provider_gate_error` is already pure, catalog-provider based, BYOK-aware, and produces the required `bum login --provider …` suggestion (`config.rs:5549`, `config.rs:5602`). The existing switch-local wrapper confirms the remaining extraction work is necessary (`model_switch.rs:24`).

- Plan 04 resolves the prior high-risk background UX flaw with the correct mechanism: an awaited async `SubagentBackend::preflight_spawn`, not a rebuild-time synchronous slug closure. The current trait and channel already support async request/oneshot patterns (`backend.rs:32`, `backend.rs:183`), and the shell can read the live parent sampling state (`subagent/mod.rs:897`).

- Plan 05 appropriately rejects resolve-only evidence. The existing routing tests prove raw provider configs use distinct bearer tokens on the wire (`provider_routing.rs:1357`, `provider_routing.rs:1400`), but they do not exercise a spawned child. The planned spawn → sample → cancel/join harness closes that exact gap.

- The routing path itself is well-founded: subagent override resolution selects credentials by catalog provider and explicitly supplies no xAI key to Codex (`subagent/mod.rs:1021`). Per-turn Codex refresh is also provider-gated (`sampler_turn.rs:294`).

## Concerns

- **MEDIUM (C3-M1) — RESOLVED (plan patch).** Plan 04 Task 1/2 now require full spawn-resolution input: `resume_from` + runtime overrides/persona/provenance, preferred shared `resolve_effective_child_model_for_spawn` with `handle_subagent_request`, plus resume/persona deny tests.

- **MEDIUM (C3-M2) — RESOLVED (plan patch).** Plan 06 Task 2 `<verify>` discover+executes shell `p7_preflight`/`p7_credential_gate`, both isolation direction names, `p7_missing`, `p7_parent_model`, `p7_tool` (not rg-only mentions).

- **LOW — the existing “resume pin” regression is not a real lifecycle test.** Current `resume_model_pinning_overrides_default_resolution` only compares strings (`rest.rs:2038`). Plan 06 recognizes this and requires a real lifecycle test or a specific cited equivalent; ensure the final citation is an actual coordinator spawn/completion/resume test, not this helper-level test.

## Suggestions

- In Plan 04, define preflight input as a cloneable “spawn resolution input” derived from the eventual `SubagentRequest`, including `resume_from`, `SubagentRuntimeOverrides` (especially persona/model provenance), subagent type, and parent session ID. Better yet, share one `resolve_effective_child_model_for_spawn` function between preflight and `handle_subagent_request`.

- Add explicit Plan 04 tests for:
  - resumed child pinned to Codex with missing Codex credentials;
  - persona-derived cross-provider role pin with omitted Task model;
  - preflight result matching the subsequent authoritative spawn resolver for each case.

- Make Plan 06 invoke the generated `07-PHASE-GATE.md` command/script as its final automated verification, or reproduce every mandatory subgroup in its `<verify>` block. Do not rely on `rg` checks that the document mentions filters.

- Require the Plan 06 lifecycle evidence to observe actual coordinator state/result routing, not only `resolve_effective_overrides`; the existing role/persona tests are pure resolution tests (`tests/mod.rs:1236`, `tests/mod.rs:1351`).

## Cycle 2 Resolution Check

| Item | Status | Evidence |
|---|---|---|
| C2-H1: async preflight required | **RESOLVED** | Plan 04 makes async backend/coordinator preflight mandatory and expressly rejects sync slug-only gates (`07-04-PLAN.md:100–114`, `07-04-PLAN.md:155`). Matches live parent config requirement (`subagent/mod.rs:897`). |
| C2-M1: no `cargo test A \|\| cargo test B` | **RESOLVED** | Plan 02 uses separate `set -euo pipefail` commands (`07-02-PLAN.md:104`, `07-02-PLAN.md:143`, `07-02-PLAN.md:174`). |
| C2-M2: gate before pending/worktree with no-pending assertion | **RESOLVED** | Plan 03 requires resolve → gate → `insert_pending` → worktree, including no-pending/no-active tests. Corrects current `insert_pending` before worktree/model resolution (`handle_request.rs:98`, `handle_request.rs:423`). |
| C2-M3: backend/resource/non-shell bridge inventory | **RESOLVED** | Plan 04 requires grep-based inventory and fail-closed missing backend (`07-04-PLAN.md:161–165`). Production builder installs `ChannelBackend` centrally (`agent_rebuild.rs:318+`). |
| C2-M4: minimal real-spawn/sample/cancel harness | **RESOLVED** | Plan 05 defines and requires the lifecycle and both directional authorization tests (`07-05-PLAN.md:103+`). |
| C2-M5: phase gate must require async eager path | **RESOLVED** | Task 2 `<verify>` discover+executes `p7_eager`, shell preflight, isolation both dirs, `p7_missing`, `p7_parent_model`, `p7_tool` (C3-M2 fold). |

## Risk Assessment

**No remaining HIGH or actionable MEDIUM after plan patch.** Residual risk items (preflight field drift; incomplete Task 2 verify) folded into Plan 04/06.

---

## Consensus Summary (Cycle 3)

Single reviewer (Codex only). Plan-level consensus is the Codex verdict grounded against current source after replan `3b27a6a`.

### Cycle 2 items — resolution verdict
1. **C2-H1 Plan 04 async preflight required** — **RESOLVED** in PLAN.md (required design section + tasks; sync Fn non-goal).
2. **C2-M1 Plan 02 verify OR-chain** — **RESOLVED**.
3. **C2-M2 Plan 03 gate-before-pending** — **RESOLVED**.
4. **C2-M3 Plan 04 fixture inventory** — **RESOLVED**.
5. **C2-M4 Plan 05 harness contract** — **RESOLVED**.
6. **C2-M5 Plan 06 AGENT-05 async dependency** — **RESOLVED** via C3-M2 plan patch (Task 2 verify executes full hard-bar subgroup list).

### Agreed Strengths
- Dual-layer design: shell authoritative gate (Plan 03) + Task async eager preflight (Plan 04).
- Green-only `p7_` protocol from Plan 01 through gate.
- Real child-sample Authorization both directions (Plan 05) remains the correct AGENT-04 bar.
- Plan 04 mechanism now matches async parent inherit architecture (`ChatStateHandle` / `resolve_effective_model_config`).

### Current HIGH Concerns
**None.**

### Current Actionable Non-HIGH Concerns
**None** — C3-M1/C3-M2 folded into PLAN.md (plan patch, not full replan).

1. **C3-M1 / Plan 04** — **RESOLVED:** full spawn-resolution input (`resume_from` + runtime overrides/persona) or shared resolve helper with `handle_subagent_request`.
2. **C3-M2 / Plan 06 (C2-M5 residual)** — **RESOLVED:** Task 2 `<verify>` discover+executes shell preflight, both isolation dirs, `p7_missing`, `p7_parent_model`, `p7_tool`.

### Tracked LOW (non-blocking)
- C2-L1 Plan 01 smoke ≠ isolation proof.
- C2-L2 Plan 06 lifecycle: named coordinator spawn/completion/resume test or concrete citation (not `resume_model_pinning_*` string-only helper alone).

### Divergent Views
N/A — single reviewer (Codex).

### Bottom line
**Cycle 3 convergence accepted with C3-M1/C3-M2 folded into Plan 04/06.** All prior HIGHs and C2-M1..M5 closed in plan text. No remaining actionable MEDIUMs. **Execute Phase 7.**

---

## Cycle 1–2 archive note

Cycle 1 (2026-07-17T07:55:02Z) established five HIGHs; replan `d9a9a51` closed four and left C2-H1. Cycle 2 (2026-07-17T08:03:38Z) confirmed C2-H1 OPEN + C2-M1..M5. Replan `3b27a6a` addressed those; cycle 3 verifies resolution and records residual C3-M1/C3-M2. Plan patch folds C3-M1 into `07-04-PLAN.md` and C3-M2 into `07-06-PLAN.md` Task 2 verify — convergence accepted, `current_actionable=0`. Full historical bodies live in git history for this file.
