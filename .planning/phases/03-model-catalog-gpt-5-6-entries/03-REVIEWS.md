# Phase 3 Plan Reviews

**Phase:** 3 — Model catalog & GPT-5.6 entries  
**Cycle:** 1  
**Reviewers:** codex (gpt-5.6-sol/high)  
**Date:** 2026-07-16

---

## Codex Review

### Summary

The plans have a sound architecture and stay within Phase 3’s catalog boundary, but they are not execution-ready because several automated commands can report false success and 03-01 Task 1 has contradictory acceptance criteria. The provider schema and Codex-only union strategy are directionally correct, though collision behavior and override parsing need stronger contracts. UI labeling is covered indirectly, but parts of `03-UI-SPEC.md` are not actually satisfied by the planned work.

### HIGH Concerns

- **Phase validation can silently pass when Cargo fails.** Nearly every `<automated>` command pipes Cargo output through `tail` without `set -o pipefail`, so the pipeline returns `tail`’s success status. Cargo accepts only one positional `TESTNAME` filter, so multi-filter commands are invalid. Affects `03-01` Task 2, `03-02` Tasks 2–3, and `03-VALIDATION.md`.
- **`03-01-PLAN.md` Task 1 cannot satisfy its own acceptance criteria.** Requires tests referencing not-yet-existing `ModelProvider` to be RED by compile failure, while also requiring `cargo test ... --test model_catalog -- --list` to discover the tests. Also references private `PROVIDER_XAI`/`PROVIDER_CODEX` from external integration tests.

### MEDIUM Concerns

- **Union-append mishandles key collisions with canonical Codex IDs.** If remote has `gpt-5.6-sol`, forced `provider=Xai` suppresses trusted bundled Codex row — loses Codex binding and UI-SPEC name. Need collision precedence + test.
- **Provider override parsing under-specified.** Missing `config_model_override_parse.rs` in files_modified; no test for `[model.<id>] provider = "codex"` parse/propagate or invalid provider warn.
- **UI-SPEC settings-description contract not met.** Settings DynamicEnum emits empty descriptions; plan claims inherit without data-flow change. Either carry descriptions or revise UI-SPEC to names-only.
- **MOD-01/02 selector coverage remains indirect.** No test for `/model` `build_model_items` or settings DynamicEnum; manual TUI optional. Need pure projection test or required UAT checkpoint.
- **Phase-check commands not in task verification.** Plan 03 Task 3 mentions cargo check for shell+pager but automated only runs tests.

### LOW Concerns

- Plan file manifests imprecise (lists inspection-only files; omits override parse).
- UI empty-state contract inconsistent (`No models available` vs `/model` returns None).
- `format_cli_model_row` pub expansion should be explicit crate API acceptance.

### Strengths

- Correct `provider` vs `agent_type` distinction.
- Identifies critical `resolved = prefetched` and Codex-only re-append.
- Custom-endpoint guard + enterprise regression planned.
- Ordering contract concrete; no Phase 4–6 scope creep.
- Integration tests over full `--lib` repair is right approach.

### Recommended Plan Changes

1. Fix every automated command: no bare `| tail` without pipefail; one Cargo filter per invocation; join with `&&` or run full binary `cargo test -p xai-grok-shell --test model_catalog`.
2. Rewrite 03-01 Task 1/2 sequencing: compiling smoke harness first; no `--list` success while compile-failing; test against public wire strings `"xai"`/`"codex"`.
3. Add collision policy to 03-02: reserve three bundled Codex IDs or preserve bundled provider/name on collision; add `prefetched_collision_cannot_rebind_codex_default_to_xai`.
4. Complete provider-chain validation in 03-01: include `config_model_override_parse.rs`, update `fully_populated_override()`, tests for valid/missing/invalid provider.
5. Reconcile 03-03 with UI-SPEC: descriptions into settings snapshot OR UI-SPEC names-only; resolve empty-state.
6. Strengthen selector validation: pure projection test or required UAT.
7. Align Task 3 automated with cargo check gates; fix file manifests.

---

## Cycle 1 Status

| Severity | Unresolved count | Notes |
|----------|------------------|-------|
| HIGH | 2 | pipefail/multi-filter; Task 1 RED vs --list contradiction |
| MEDIUM actionable | 5 | collision, override parse, settings desc, selector coverage, phase check in verify |
| LOW | 3 | manifests, empty-state, pub API note |

**Next:** replan with `--reviews` incorporating all HIGH + actionable MEDIUM.

---

## Cycle 1 Replan Status (2026-07-16)

Replanned in place: `03-01`/`03-02`/`03-03` PLAN.md + `03-VALIDATION.md` + `03-UI-SPEC.md`. All HIGH + actionable MEDIUM incorporated into executable plan contracts (see planner return).

---

## Codex Review — Cycle 2

### Summary
Prior HIGH fully resolved. No HIGH remain. Two MEDIUM: collision ordering vs D-11; selector UAT waiver.

### HIGH Concerns
None.

### MEDIUM Concerns (actionable)
1. **Collision ordering:** Replace-in-place can put Terra before xAI. Policy should remove all bundled Codex keys then append in JSON order (Sol→Terra→Luna after remote/xAI).
2. **Selector UAT waiver:** Task 4 allows environment blocker to pass via ACP-only. Need pure pager projection test (`dynamic_enum_choices` / exported seam) OR truly blocking UAT without waive-as-pass.

### LOW
- Task 1 RED not observed in automated (only --list + smoke)
- Fuzzy-empty vs empty-catalog UI-SPEC clarity

### Resolution of Prior Cycle
- HIGH false-green cargo: RESOLVED
- HIGH Task 1 contradiction: RESOLVED
- MEDIUM collision policy (provider authority): RESOLVED (ordering separate)
- MEDIUM override parse: RESOLVED
- MEDIUM settings descriptions: RESOLVED
- MEDIUM selector coverage: PARTIAL
- MEDIUM cargo check: RESOLVED

### Cycle 2 Status
| Severity | Unresolved |
|----------|------------|
| HIGH | 0 |
| MEDIUM actionable | 2 |

---

## Cycle 2 Replan Status (2026-07-16)

Replanned in place (targeted only):

| Concern | Resolution |
|---------|------------|
| MEDIUM 1 Collision ordering | `03-02`: remove-all bundled Codex keys then append Sol→Terra→Luna; test `prefetched_codex_collision_order_appends_sol_terra_luna_after_remote`; PATTERNS sketch updated |
| MEDIUM 2 Selector UAT waiver | `03-03`: automated `--test dynamic_enum_model_names` via public `dynamic_enum_choices`; Task 4 → optional advisory (`gate=optional`); environment skip is not phase pass |
| LOW Task 1 RED observe | `03-01` Task 1 done note: RED observe optional; gate remains `--list` + harness_smoke |
| LOW Fuzzy vs empty catalog | `03-UI-SPEC` selection chrome one-liner; Task 3 UI-SPEC patch |

`03-VALIDATION.md` full suite + phase gate updated for collision order filter + `dynamic_enum_model_names`.


---

## Codex Review — Cycle 3

### Summary
CONVERGED. Both cycle-2 MEDIUM closed. No HIGH, no actionable MEDIUM.

CYCLE_SUMMARY: current_high=0 current_actionable=0

