---
phase: 03-model-catalog-gpt-5-6-entries
verified: 2026-07-16T11:06:31Z
status: passed
score: 10/10 must-haves verified
behavior_unverified: 0
overrides_applied: 0
re_verification: false
mvp_goal_format_note: "ROADMAP Phase 3 goal is technical prose, not 'As a…, I want to…, so that…'. User-flow coverage derived from plan-level user stories + roadmap success criteria / outcome. Consider `/gsd mvp-phase 3` if strict MVP goal formatting is desired."
---

# Phase 3: Model catalog & GPT-5.6 entries Verification Report

**Phase Goal:** Model selector presents a mixed, provider-labeled catalog including GPT-5.6 family options  
**Verified:** 2026-07-16T11:06:31Z  
**Status:** passed  
**Re-verification:** No — initial verification  
**Mode:** mvp (roadmap)

## User Flow Coverage

User-facing outcome (from plan goals + roadmap SC): *As a bum user, I want to see Grok and GPT-5.6 Sol/Terra/Luna in one provider-labeled model catalog, so that I can pick either family from the same list before routing and Codex OAuth land.*

| Step | Expected | Evidence | Status |
|------|----------|----------|--------|
| Open catalog (defaults) | List includes `grok-build` + `gpt-5.6-sol/terra/luna` | `default_models.json`; `catalog_includes_gpt56`, `mixed_catalog_order` PASS | ✓ |
| See provider labels | Grok named `Grok Build (xAI)`; GPT family `GPT-5.6 * (Codex)` | JSON names; `grok_entry_provider_xai_and_name_suffix`, `acp_list_projection_includes_provider_suffixes` PASS | ✓ |
| One mixed list | Flat list, Grok first then Sol→Terra→Luna; no session-global provider filter | `mixed_catalog_order`, `mixed_list_with_prefetch`, `gpt_visible_session_and_api_key` PASS | ✓ |
| Provider binding usable later | Every entry has `provider` `xai`\|`codex`; ACP `meta.provider` set | `ModelProvider` on type chain; `to_acp_model_info_*` PASS | ✓ |
| Logged-in / prefetch path | GPT rows still present after remote replace | `codex_defaults_survive_prefetch`, collision tests PASS | ✓ |
| CLI / settings surfaces | `id (name)` rows; DynamicEnum displays carry suffixes | `format_cli_model_row`, `dynamic_enum_model_names` PASS | ✓ |

Interactive slash `/model` visual polish is **optional advisory** (VALIDATION / Plan 03 Task 4) — not required for phase pass when automated projection gates are green.

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
| --- | ------- | ---------- | -------------- |
| 1 | Model selector lists GPT-5.6 Sol/Terra/Luna labeled as Codex/OpenAI (roadmap SC1) | ✓ VERIFIED | `default_models.json` entries `gpt-5.6-sol/terra/luna` with names `GPT-5.6 * (Codex)` and `provider: "codex"`; `catalog_includes_gpt56` PASS |
| 2 | Grok + GPT in one mixed list, not a global provider mode filter (roadmap SC2) | ✓ VERIFIED | Single flat catalog; `mixed_catalog_order` (Grok then Sol→Terra→Luna); `gpt_visible_session_and_api_key` filters only via `visible_for_auth` / `supported_in_api` — no Codex credential gate |
| 3 | Every catalog entry carries explicit provider binding for later routing (roadmap SC3) | ✓ VERIFIED | `ModelProvider` on `DefaultModelJson` / `ModelEntryConfig` / `ModelInfo` / `ConfigModelOverride`; shipped JSON sets field; `to_acp_model_info` inserts `meta.provider` `"xai"`\|`"codex"`; missing → `xai` (`provider_default_xai`) |
| 4 | Default model remains `grok-build` | ✓ VERIFIED | `default_models.json` `"default": "grok-build"`; `default_model_is_grok_build_and_catalog_has_four_plus` + `default_remains_grok_build` PASS |
| 5 | No-prefetch order is Grok then Sol → Terra → Luna | ✓ VERIFIED | `mixed_catalog_order` PASS |
| 6 | Prefetch re-appends bundled Codex defaults when `!has_custom_endpoint` | ✓ VERIFIED | `resolve_model_list` remove-then-append at config.rs ~3154–3175; `codex_defaults_survive_prefetch`, `empty_prefetch_still_gets_codex_defaults`, `custom_endpoint_skips_codex_inject` PASS |
| 7 | Collision: bundled Codex + UI-SPEC name win; Sol→Terra→Luna after remote | ✓ VERIFIED | `prefetched_collision_cannot_rebind_codex_default_to_xai`, `prefetched_codex_collision_order_appends_sol_terra_luna_after_remote` PASS |
| 8 | GPT visible for session and API-key auth without Codex login | ✓ VERIFIED | GPT `supported_in_api: true`; `gpt_visible_session_and_api_key` PASS |
| 9 | ACP list projects provider-suffixed names + `meta.provider` | ✓ VERIFIED | `to_acp_model_info_sets_meta_provider`, `to_acp_model_info_provider_xai`, `acp_list_projection_includes_provider_suffixes` PASS |
| 10 | CLI + settings surfaces show provider-bearing names | ✓ VERIFIED | `pub format_cli_model_row` wired in `list_available_models`; `format_cli_model_row` 4/4 PASS; `dynamic_enum_choices` name inherit 2/2 PASS |

**Score:** 10/10 truths verified (0 present, behavior-unverified)

### Required Artifacts

| Artifact | Expected | Status | Details |
| -------- | ----------- | ------ | ------- |
| `crates/codegen/xai-grok-models/default_models.json` | Mixed catalog + provider field + GPT family | ✓ VERIFIED | 4 models; all have explicit `provider`; GPT names Codex-suffixed |
| `crates/codegen/xai-grok-shell/src/agent/config.rs` | `ModelProvider`, resolve merge, ACP meta | ✓ VERIFIED | Enum + type chain + remove-then-append + `to_acp_model_info` |
| `crates/codegen/xai-grok-shell/src/agent/config_model_override_parse.rs` | Override parse includes provider | ✓ VERIFIED | `fully_populated_override` includes provider; invalid-provider path tested |
| `crates/codegen/xai-grok-shell/src/remote/client.rs` | Remote parse forces `provider: Xai` | ✓ VERIFIED | Hardcoded `ModelProvider::Xai` on remote parse (~line 836) |
| `crates/codegen/xai-grok-shell/tests/model_catalog.rs` | Integration harness for catalog contracts | ✓ VERIFIED | 23 tests, all PASS |
| `crates/codegen/xai-grok-pager/src/models.rs` | `pub format_cli_model_row` + list wiring | ✓ VERIFIED | Public formatter; used in list loop |
| `crates/codegen/xai-grok-pager/tests/format_cli_model_row.rs` | CLI row format tests | ✓ VERIFIED | 4/4 PASS |
| `crates/codegen/xai-grok-pager/tests/dynamic_enum_model_names.rs` | Settings name-suffix tests | ✓ VERIFIED | 2/2 PASS |
| `.planning/phases/03-model-catalog-gpt-5-6-entries/03-UI-SPEC.md` | UI contract | ✓ VERIFIED | Present (gsd verify.artifacts) |

### Key Link Verification

| From | To | Via | Status | Details |
| ---- | --- | --- | ------ | ------- |
| `default_models.json` | `ModelEntryConfig` / `ModelInfo` | `DefaultModelJson.provider` → map → `ModelInfo::from_config` | ✓ WIRED | config.rs `default_models` + `from_config` copies `provider` |
| `ModelInfo.provider` | wire `"xai"` / `"codex"` | `ModelProvider::as_str()` | ✓ WIRED | Matches auth slot ids; tests assert string literals |
| `ConfigModelOverride.provider` | `ModelInfo.provider` | `apply` when `Some` | ✓ WIRED | `override_provider_codex_reaches_model_info` PASS |
| Prefetch branch | bundled Codex defaults | `shift_remove` then append `provider == Codex` if `!has_custom_endpoint` | ✓ WIRED | config.rs ~3160–3174 |
| `available_models` | `visible_for_auth` | session vs API-key only | ✓ WIRED | No Codex credential filter; visibility tests PASS |
| `ModelInfo.provider` | ACP `meta.provider` | `to_acp_model_info` | ✓ WIRED | Always inserts from trusted in-process provider |
| ACP / catalog names | CLI stdout | `format_cli_model_row(is_current, id, name)` | ✓ WIRED | models.rs list loop |
| `PagerLocalSnapshot.available_models` names | settings DynamicEnum | `dynamic_enum_choices(ActiveModelCatalog, …)` | ✓ WIRED | Integration tests prove display pass-through |

*Note: `gsd-tools query verify.key-links` reports false when `from:` is not a file path; links above were verified by code inspection + tests.*

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| -------- | ------------- | ------ | ------------------ | ------ |
| Default catalog | `models[]` | Embedded `DEFAULT_MODELS_JSON` / `include_str!` | Real shipped entries (not empty stub) | ✓ FLOWING |
| `resolve_model_list` | `IndexMap<String, ModelEntry>` | defaults ± prefetched ± config_models | Real merge; Codex re-append from `default_model_entries` | ✓ FLOWING |
| ACP model list | `name`, `meta.provider` | `to_acp_model_info` from `ModelInfo` | Catalog names + provider wire ids | ✓ FLOWING |
| CLI rows | `id`, `name` | ACP list state → `format_cli_model_row` | Uses live model list names | ✓ FLOWING |
| Settings DynamicEnum | `display` / `canonical` | Snapshot `available_models` names | Pass-through of provider-suffixed names | ✓ FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| -------- | ------- | ------ | ------ |
| Models crate default + catalog size | `cargo test -p xai-grok-models --lib -- --nocapture` | 1 passed | ✓ PASS |
| Full catalog integration suite | `cargo test -p xai-grok-shell --test model_catalog -- --nocapture` | 23 passed | ✓ PASS |
| CLI row formatter | `cargo test -p xai-grok-pager --test format_cli_model_row -- --nocapture` | 4 passed | ✓ PASS |
| Settings DynamicEnum names | `cargo test -p xai-grok-pager --test dynamic_enum_model_names -- --nocapture` | 2 passed | ✓ PASS |
| Shell compile | `cargo check -p xai-grok-shell` | Finished ok | ✓ PASS |
| Pager compile | `cargo check -p xai-grok-pager` | Finished ok | ✓ PASS |

### Probe Execution

| Probe | Command | Result | Status |
| ----- | ------- | ------ | ------ |
| — | — | No phase-declared `scripts/*/tests/probe-*.sh` | SKIP (N/A) |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| ----------- | ---------- | ----------- | ------ | -------- |
| MOD-01 | 03-01, 03-02, 03-03 | GPT-5.6 family in selector, labeled by provider | ✓ SATISFIED | Catalog + names + ACP/CLI/settings projection tests |
| MOD-02 | 03-01, 03-02, 03-03 | Grok + GPT in one mixed list | ✓ SATISFIED | Mixed order + prefetch survival + dual-auth visibility |

No orphaned Phase 3 requirements in REQUIREMENTS.md beyond MOD-01/MOD-02.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| ---- | ---- | ------- | -------- | ------ |
| — | — | No TBD/FIXME/XXX debt markers in phase-touched files | — | — |
| `config.rs` / `models.rs` | various | `todo_gate` / "placeholder" comments | ℹ️ Info | Pre-existing product terms unrelated to stubs |

No blocker debt markers. No hollow stubs on catalog data path.

### Human Verification Required

None required for phase pass.

**Optional advisory (not a gate):** Launch `bum`, open `/model` or settings Default model, confirm visual rows show `Grok Build (xAI)` and GPT-5.6 Sol/Terra/Luna `(Codex)` in one flat list. Environment inability to launch TUI is a documented skip — not pass or fail.

### Gaps Summary

No gaps. All three roadmap success criteria and supporting plan must-haves are present, substantive, wired, and behaviorally proven by the Phase 3 automated gate. Routing (Phase 4), Codex OAuth (Phase 5), and missing-provider UX (Phase 6) remain correctly out of scope.

### Notes

- **MVP goal format:** Phase `mode: mvp` but ROADMAP goal is not the strict user-story sentence form. Verification still completed against success criteria and plan user-story outcomes. Optional: `/gsd mvp-phase 3` to reformat the roadmap goal.
- **Shell/pager `--lib` suites** remain broken compile debt by design; Phase 3 correctly gates on integration binaries only (VALIDATION.md).
- **Phase 4** will consume `ModelInfo.provider` for credential/backend selection — binding is present; routing is deferred.

---

_Verified: 2026-07-16T11:06:31Z_  
_Verifier: Claude (gsd-verifier)_
