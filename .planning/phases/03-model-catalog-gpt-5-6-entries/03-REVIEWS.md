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
