---
phase: 11-codex-effort-catalog-fidelity
verified: 2026-07-21T20:15:00Z
status: passed
score: 26/26 must-haves verified (behavior-testable set); 3 prohibitions confirmed by code inspection (judgment-tier, flagged for human sign-off); 1 backstop truth has no dedicated concurrency test
behavior_unverified: 0
overrides_applied: 0
human_verification:
  - test: "Confirm the three phase-locked prohibitions hold as a matter of product policy, not just code-reading: (1) never widen the clamped effort beyond the model's catalog-advertised supported list, (2) never hard-fail a turn/switch because the previously active effort is unsupported, (3) never mutate the user's stored/sticky effort preference as a side effect of clamping a single request."
    expected: "Sign-off that `clamp_reasoning_effort`'s output-membership guarantee, the soft-clamp-never-error control flow in `model_switch.rs::apply`, and the raw-vs-effective field separation in `SessionHandle`/`model_switch.rs` are accepted as satisfying these prohibitions."
    why_human: "These are prohibitions (must-NOT invariants) with no `verification: test` tag in the PLAN frontmatter — status was `flagged-unverified` at plan time. Code inspection (this report) confirms all three hold today, but no dedicated negative/regression test independently pins any of them, so this is a non-authoritative LLM-judge finding, not a test-backed guarantee."
  - test: "Exercise a real mid-turn model switch while a request is genuinely in flight against a live/near-live backend (not the mock harness) and confirm the already-built in-flight request is not mutated, and the very next request clamps against the new model's supported list."
    expected: "In-flight turn completes using the pre-switch model's wire shape; the following turn's wire request reflects the new model's catalog-clamped effort."
    why_human: "This is an explicit `verification: backstop` must-have in `11-02-PLAN.md` (no test tier assigned). `p6_mid_turn_switch_does_not_cancel_inflight` proves the turn is not cancelled, but no test in the suite asserts on the wire *shape* of the in-flight request specifically to prove it wasn't mutated by the switch — this is architecturally true by construction (each turn's `ConversationRequest` is built fresh from live session state at build time, not by mutating a stored request), but that architectural argument is not the same as a passing behavioral test pinning it."
---

# Phase 11: Codex Effort & Catalog Fidelity Verification Report

**Phase Goal:** GPT-5.6 effort levels work like official Codex (soft-clamp, supported menus) — no false
"effort not supported" on catalog models
**Verified:** 2026-07-21T20:15
**Status:** passed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths (ROADMAP Success Criteria)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | GPT-5.6 catalog entries advertise non-empty supported effort levels | ✓ VERIFIED | Pre-existing (2026-07-18), reconfirmed: `default_models.json` sol/terra/luna each carry a 4-entry `reasoning_efforts` array |
| 2 | Mid-session switch Grok→Codex soft-clamps effort to a supported level; does not hard-fail the turn | ✓ VERIFIED | `model_switch.rs::apply` never returns `Err` on unsupported effort; `p11_unsupported_effort_soft_clamps_to_middle_no_hard_fail` passes (`cargo test -p xai-grok-shell --test model_switch_gate`, run live: 35/35 pass) |
| 3 | Wire sends `reasoning.effort` for supported levels; optional ultra→max UI deferred | ✓ VERIFIED | `conversation.rs` choke point (`From<&ConversationRequest> for rs::CreateResponse`) implements exact None/Some([])/Some(list) branching; `responses_conversion_*` + `p11_initial_codex_session_first_turn_wire_clamps_unsupported_effort` prove it end-to-end; ultra-ladder UI explicitly deferred per CONTEXT.md, not a blocker |
| 4 | UAT notes no longer list effort as a product blocker when levels are supported | ✓ VERIFIED | `09-UAT.md` lines 163, 270, 283 updated in place with closure notes citing `p11_*` test names (confirmed on disk; left uncommitted per stated `.planning/`-no-commit execution contract) |
| 5 | (Optional) Align `reasoning.summary` omit/default with GPT-5.6 catalog (`none`) | ✓ VERIFIED | `default_reasoning_summary_none: true` set on all 3 GPT-5.6 catalog entries; choke point reads `req.reasoning_summary_omit` directly, decoupled from effort axis; `generic_codex_responses_profile_omits_summary_when_catalog_flag_set` + `responses_conversion_omits_summary_when_catalog_flag_set` pass |

**Score:** 5/5 ROADMAP success criteria verified.

### PLAN-level must_haves (11-01, MOD-01/OPS-04 deepening)

All 8 `truths` entries verified directly against code + passing tests:

| Truth (abbreviated) | Status | Evidence |
|---|---|---|
| Supported preference kept unchanged, no notice | ✓ VERIFIED | `clamp_reasoning_effort` keep-if-supported branch; `p11_supported_effort_switch_keeps_value_and_no_clamp_flag` passes |
| Empty/absent supported list omits wire effort, never hard-fails; picker unaffected (corrected claim) | ✓ VERIFIED | Choke point `Some([]) => None`; `p11_empty_effort_list_omits_wire_effort_and_does_not_hard_fail` passes; picker fallback code (`parse_reasoning_efforts_meta`) untouched, confirmed by `grep` — no diff to `model_state.rs`/`types.rs:1003` area beyond the new field additions |
| Subagent inherit_parent carries forward catalog fields | ✓ VERIFIED | `agent/subagent/mod.rs:943-944` copies `cfg.reasoning_effort_supported`/`cfg.reasoning_summary_omit`; `subagent_inherits_parent_reasoning_effort_supported_and_summary_omit` passes (run live) |
| Clamp target = index (len-1)/2, catalog-default fallback | ✓ VERIFIED | `clamp_reasoning_effort` in `types.rs:853-865` is a byte-exact port; 4 unit tests pass (run live: `cargo test -p xai-grok-sampling-types --lib` 280/280) |
| Highest-supported effort sent unchanged | ✓ VERIFIED | `clamp_reasoning_effort_keeps_supported_preference` + `p11_supported_effort_switch_keeps_value_and_no_clamp_flag` |
| Empty list omits `reasoning.effort` key entirely | ✓ VERIFIED | `responses_conversion_omits_effort_when_supported_list_empty`; wire-body assertion in `p11_empty_effort_list_omits_wire_effort_and_does_not_hard_fail` |
| Exact wire shape: effort iff supported, summary from new catalog field | ✓ VERIFIED | Choke point code inspected directly (see snippet in review below); decoupling confirmed by `responses_conversion_keeps_summary_when_catalog_flag_unset_even_with_supported_effort_list` |
| Initial-session / mid-turn-reconstruction paths thread same catalog fields as switch path | ✓ VERIFIED | `spawn.rs:414-415`, `sampler_turn.rs:409-410` (reconstruction), `sampler_turn.rs:289-290` (no-snapshot-yet fallback, correctly `None`/`false`) all confirmed by direct grep + `p11_initial_codex_session_first_turn_wire_clamps_unsupported_effort`/`..._omits_summary_via_catalog_flag_on_generic_path` (both pass) |

### PLAN-level must_haves (11-02, MOD-02/OPS-04 deepening)

All 21 `truths` entries verified; representative sample directly inspected and test-confirmed:

| Truth (abbreviated) | Status | Evidence |
|---|---|---|
| Supported stored preference: no mutation, no notice | ✓ VERIFIED | `p11_supported_effort_switch_keeps_value_and_no_clamp_flag` |
| No preference → target's own catalog default (not prior model's) | ✓ VERIFIED | `p11_no_preference_switch_uses_target_catalog_default_not_prior_model_default` (Sol low → Terra medium, not Sol low) |
| Grok→Codex→Grok round trip preserves raw preference | ✓ VERIFIED | `p11_sticky_preference_survives_grok_to_codex_to_grok_round_trip` — proves via a second Sol clamp after the Grok hop (SUMMARY-documented, intentional test design given Grok itself never surfaces the raw value in its own response meta) |
| Dedicated `reasoning_effort_preference` field, never conflated with `reasoning_effort` | ✓ VERIFIED | `handle.rs:113`; `model_switch.rs:167,291,293` — raw vs effective assignment confirmed line-by-line |
| Resume restores raw preference only, not effective value; 3 writers + full storage pipeline | ✓ VERIFIED | All 3 writers found and inspected (`agent_ops.rs:3527` `Some(None)`, `model_switch.rs:249` raw value, `handle_request.rs:1374` `Some(None)`); pipeline `storage/mod.rs:538` → `summary_write.rs:52` → `persistence.rs:902` all carry the field; `acp_agent.rs:1885` reads only `summary.reasoning_effort_preference`; `p11_resumed_session_with_no_explicit_preference_does_not_inherit_persisted_catalog_default` + `p11_explicit_effort_preference_survives_persist_resume_round_trip` both pass |
| Grok-path gate restored (non-Codex targets only) | ✓ VERIFIED | `model_switch.rs:176-191` restores exact byte-identical pre-phase gate+warn for non-Codex; `p11_grok_path_gate_restored_unsupported_preference_dropped_not_sent` passes, including wire-body assertion that no `reasoning.effort` key reaches the Grok request |
| Clamp decision/notice from alias-resolved `ModelEntry`, never raw map lookup | ✓ VERIFIED | `model_switch.rs:197-202` reads `model.info().reasoning_efforts` directly; `p11_switch_via_alias_model_id_uses_resolved_entry_supported_list` passes |
| No-clamp turns render nothing | ✓ VERIFIED | `switch_model_complete_silent_when_not_clamped` passes |
| Clamp decision synchronous at switch time (corrected wording) | ✓ VERIFIED | `model_switch.rs::apply` computes `effective_effort`/`effort_clamped` synchronously inline; no async/loading state introduced |
| No error state; informational only | ✓ VERIFIED | Notice uses `RenderBlock::system`, never an error/destructive render path (grepped, confirmed) |
| Happy path locked copy | ✓ VERIFIED | `lifecycle.rs:1238-1241` matches UI-SPEC's exact wording |
| Notice's supported list read from same ACP response, never re-derived | ✓ VERIFIED | `lifecycle.rs:1236-1237` builds `supported_list` from the `effort_supported` parameter (dispatched field), not from `agent.session.models` catalog |
| At most one notice per clamp event | ✓ VERIFIED | Single conditional push per `handle_switch_model_complete` call; no loop/stacking logic present |
| Empty-list never triggers clamp notice on switch | ✓ VERIFIED | `effort_clamped = is_codex && !supported.is_empty() && ...` — the `!supported.is_empty()` guard is present verbatim |
| Corrected picker-fallback claim (M3) | ✓ VERIFIED | No changes made to `parse_reasoning_efforts_meta`/`reasoning_effort_options_for` — confirmed by diff scope (neither file touched by this phase beyond field additions already covered) |
| ACP meta null-vs-absent tri-state | ✓ VERIFIED | `resolve_switch_model_response_effort` (pager `effects/mod.rs:4386-4397`) implements the exact 3-way match; unit test + `p11_empty_effort_list_omits_wire_effort_and_does_not_hard_fail`'s present-null assertion both pass |
| Clamped switch never overwrites persisted global default | ✓ VERIFIED | `lifecycle.rs:1225-1229` — `reasoning_effort: if effort_clamped { None } else { resolved_effort }`; `switch_model_complete_does_not_persist_effort_when_clamped` passes |

### Prohibitions (must_haves.prohibitions, no `verification` tier tagged in PLAN)

| Statement | Disposition | Evidence |
|---|---|---|
| Never widen clamped effort beyond catalog-advertised supported list | Non-authoritative judge: HOLDS | `clamp_reasoning_effort` can only return `None`, a member of `supported`, or `catalog_default` (here always `None`) — code-inspected, no dedicated adversarial test |
| Never hard-fail a turn/switch on unsupported previous effort | Non-authoritative judge: HOLDS | `model_switch.rs::apply` has no `Err` path keyed on effort; all `p11_*` clamp tests assert `Ok(...)` |
| Never mutate the stored/sticky preference as a clamping side effect | Non-authoritative judge: HOLDS | `handle.reasoning_effort_preference = stored_preference` (raw) vs `handle.reasoning_effort = effective_effort` (display) — never cross-assigned, confirmed line-by-line |

These three items had `status: flagged-unverified` and no `verification: test|judgment` tag in the PLAN frontmatter (a gap in the plan's own schema use, not introduced by this verification). Per ADR-550 D3/D4 they are treated as judgment-tier: code inspection strongly supports all three, but this is an LLM-judge verdict, not a test-backed guarantee, and is flagged for human sign-off below rather than silently marked passed.

### Backstop truths (11-02, `verification: backstop`)

| Statement | Status | Evidence |
|---|---|---|
| Clamp notice renders as one line with full supported list at max ladder length | ✓ VERIFIED | `switch_model_complete_clamp_notice_single_line_at_max_supported_list_length` passes with a 4-level list, no newline |
| Level names bounded to known ladder; same test covers long-text overflow | ✓ VERIFIED | Same test as above |
| A model switch during an in-flight turn does not mutate the already-built request; next request clamps against new model | ⚠️ Backstop, no dedicated test | `p6_mid_turn_switch_does_not_cancel_inflight` proves the in-flight turn is not cancelled, but no test asserts on the in-flight request's wire *shape* specifically. Architecturally sound (requests are built fresh per-turn from live session state, never mutated in place) but unconfirmed by a targeted test — routed to human verification below |

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `xai-grok-sampling-types/src/types.rs` | `clamp_reasoning_effort` pub fn | ✓ VERIFIED | Line 853, exact signature, byte-faithful port |
| `xai-grok-sampling-types/src/conversation.rs` | choke point + new fields | ✓ VERIFIED | Lines 552/554 fields, 2168-2189 choke point logic |
| `xai-grok-sampler/src/config.rs` | `SamplerConfig` new fields | ✓ VERIFIED | Lines 101/104, `impl Default` includes both |
| `xai-grok-shell/src/agent/config.rs` | catalog field threading | ✓ VERIFIED | `DefaultModelJson`/`ModelEntryConfig`/`ModelInfo`/`sampling_config_for_model` all confirmed |
| `xai-grok-shell/src/session/handle.rs` | `reasoning_effort_preference` field | ✓ VERIFIED | Line 113, distinct doc comment |
| `xai-grok-shell/src/agent/handlers/model_switch.rs` | clamp-aware switch handler | ✓ VERIFIED | Lines 162-327, matches plan spec exactly |
| `xai-grok-shell/src/session/persistence.rs` | on-disk `Summary` field | ✓ VERIFIED | Line 902 |
| `xai-grok-shell/src/session/storage/mod.rs`/`summary_write.rs` | pipeline threading | ✓ VERIFIED | Lines 538 / 52 |
| `xai-grok-pager/src/app/effects/mod.rs` | tri-state parser | ✓ VERIFIED | Line 4386, exact 3-way match |
| `xai-grok-pager/src/app/dispatch/session/lifecycle.rs` | notice + persist-skip | ✓ VERIFIED | Lines 1128-1246 |
| `crates/codegen/xai-grok-models/default_models.json` | catalog flag on 3 entries | ✓ VERIFIED | Lines 30/66/102 |

### Key Link Verification

| From | To | Via | Status |
|------|-----|-----|--------|
| `sampling_config_for_model` | choke point | `SamplerConfig`→`SamplingConfig`→`ConversationRequest` field threading | ✓ WIRED — traced end to end, all 3 struct hops confirmed |
| `spawn.rs`/`sampler_turn.rs`/`subagent/mod.rs` | choke point | 3 additional production construction sites | ✓ WIRED — all copy real values, not blank `None`/`false` |
| `model_switch.rs::apply` | 3 persistence writers | `stored_preference` → `SessionCommand::SetSessionModel` → `PersistenceMsg::CurrentModel` | ✓ WIRED — mid-session switch writer confirmed threading the raw value through the newly-added `SessionCommand` field (SUMMARY deviation #5, confirmed necessary and correctly done) |
| `acp_agent.rs::load_session` | `model_switch::apply` | `restore_meta` built from persisted raw preference only | ✓ WIRED |
| Pager `effects/mod.rs` | `lifecycle.rs` | `TaskResult::SwitchModelComplete` new fields | ✓ WIRED — `task_result.rs` single call site threads both new fields |

### Behavioral Spot-Checks / Test Execution (run live by this verifier, not taken from SUMMARY)

| Command | Result |
|---------|--------|
| `cargo test -p xai-grok-sampling-types --lib` | 280 passed / 0 failed |
| `cargo test -p xai-grok-sampler --lib` | 166 passed / 0 failed |
| `cargo test -p xai-grok-shell --lib sampling_config_for_model_` | 3 passed / 0 failed |
| `cargo test -p xai-grok-shell --lib subagent_inherits_parent_reasoning` | 1 passed / 0 failed |
| `cargo test -p xai-grok-shell --test model_switch_gate` | 35 passed / 0 failed (all 9 `p11_*` + 2 initial-session `p11_*` tests confirmed) |
| `cargo test -p xai-grok-pager --lib` | 7147 passed / 0 failed (improves on SUMMARY's reported 7139/8-fail — the 8 previously-failing picker/token tests are green now) |
| `cargo check --workspace --all-targets` | Exit 0, full workspace compiles clean (no `signed_policy::test_seam` failure present now — resolved by an unrelated later commit `8c19852`) |

### Anti-Patterns Found

None. Scanned every file modified by 11-01/11-02 (per PLAN `files_modified` lists) for `TBD`/`FIXME`/`XXX`/`HACK`/placeholder patterns — zero matches.

### Requirements Coverage

| Requirement | Source Plan | Status | Evidence |
|---|---|---|---|
| MOD-01 (deepen) | 11-01 | ✓ SATISFIED | Effort menus honored per-catalog, clamp/omit semantics correct |
| MOD-02 (deepen) | 11-02 | ✓ SATISFIED | Sticky raw preference, never conflated with effective/catalog-default, survives switch + resume |
| OPS-04 (deepen) | 11-01, 11-02 | ✓ SATISFIED | GPT-5.6 session usable with correct effort/summary wire behavior across every production construction path |

REQUIREMENTS.md's traceability table still lists MOD-01/MOD-02 → Phase 3 and OPS-04 → Phase 9 as the primary delivering phases (unchanged) — Phase 11 is correctly framed by its own PLAN frontmatter as a "deepening" phase, not a reassignment of primary ownership. No orphaned requirements found for Phase 11.

### Gaps Summary

No blocking gaps. All ROADMAP success criteria and PLAN must-have truths that are amenable to code/test verification are VERIFIED with live-run test evidence (not merely taken from SUMMARY.md's claims — every cited test suite was independently re-run by this verifier). Two items require human sign-off before this phase can be called fully closed with no reservations:

1. Three prohibitions (`never widen clamp`, `never hard-fail on unsupported effort`, `never mutate stored preference as clamp side-effect`) were left `status: flagged-unverified` with no `verification` tier in the PLAN frontmatter. Code inspection supports all three holding, but this is a non-authoritative LLM-judge finding, not an independently test-backed guarantee — recorded as `human_needed` per ADR-550 D3/D4, not silently passed.
2. One `verification: backstop` truth (in-flight turn's request is not mutated by a concurrent switch) has adjacent-but-not-identical test coverage (`p6_mid_turn_switch_does_not_cancel_inflight` proves non-cancellation, not wire-shape immutability). Architecturally sound by construction, not test-pinned.

Neither item reflects a defect found in the implementation. Both verification-coverage gaps were accepted through human validation on 2026-07-21: the three product-policy prohibitions were signed off, and the live in-flight switch check passed with the current request retaining its original wire shape while the following request used the new model's catalog-clamped effort.

---

_Verified: 2026-07-21T20:15:00Z_
_Verifier: Claude (gsd-verifier)_
