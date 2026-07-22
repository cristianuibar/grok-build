---
phase: 06-mid-session-switch-missing-provider-gate
verified: 2026-07-17T00:38:26Z
status: passed
score: 3/3 must-haves verified
behavior_unverified: 0
overrides_applied: 0
re_verification: false
---

# Phase 6: Mid-session switch & missing-provider gate Verification Report

**Phase Goal:** User freely switches models mid-session; missing credentials fail closed with the correct login prompt  
**Verified:** 2026-07-17T00:38:26Z  
**Status:** passed  
**Re-verification:** No — initial verification  
**Mode:** mvp (ROADMAP goal is not User Story format; flow coverage derived from CONTEXT/PLAN user story + success criteria)

## User Flow Coverage

User story (from CONTEXT / plan objective): **As a** bum user mid-session, **I want to** switch models freely and get a clear fail-closed login path when a provider is missing, **so that** I never depend on silent mid-turn 401 as the primary UX and never restart the CLI to change models.

| Step | Expected | Evidence | Status |
|------|----------|----------|--------|
| Switch model mid-session | Next turn uses newly selected model without restarting CLI | `model_switch::apply` updates session model; `p6_dual_login_next_sample_uses_target_provider` + `switch_changes_next_sample_route` pass | ✓ |
| Select model for unusable provider | Switch blocked; Login prompt names provider | Gate early-return before prepare; `p6_missing_provider_*` shell + pager QuestionView `MissingProviderLogin` | ✓ |
| Choose Login now / Keep current | Login recovery defers apply; Keep stays on previous model | `dispatch_missing_provider_login_answered`; `p6_login_now_*`, `p6_keep_current_*`, `p6_auth_complete_*` | ✓ |
| Both providers logged in | Free Grok ↔ GPT-5.6 in one continuous session | `p6_dual_login_free_switch_*`, history + mid-turn session tests | ✓ |

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
| --- | ------- | ---------- | -------------- |
| 1 | User can switch models mid-session anytime; the next turn uses the newly selected model without restarting the CLI | ✓ VERIFIED | Shell session harness free switch both directions; `p6_dual_login_next_sample_uses_target_provider` proves next sample after switch hits target provider; pure `switch_changes_next_sample_route` pass. Mid-turn switch allowed without cancel: `p6_mid_turn_switch_does_not_cancel_inflight`. |
| 2 | Selecting a model whose provider has no usable credentials blocks the switch and prompts that provider’s login (no silent mid-turn 401 as primary UX) | ✓ VERIFIED | `model_switch::apply` calls `missing_provider_gate_error` after `resolve_model_id` and **before** `prepare_prepared_sampling_config` / SetSessionModel / ModelChanged. Typed `MODEL_SWITCH_MISSING_PROVIDER` ACP payload (camelCase `code`, `provider`, `modelId`, `suggestion: bum login --provider {id}`). Pager maps error → `SwitchModelError::MissingProvider` → `LocalQuestionKind::MissingProviderLogin` with UI-SPEC copy (“Sign in to {Provider}…”, Login now / Keep current, CLI in option description). Tests: shell `p6_missing_provider_*` (3), unit round-trip (2), pager `p6_missing_provider_*` + login/keep/deferred suites. |
| 3 | With both providers logged in, user can move between Grok and GPT-5.6 models in one continuous session | ✓ VERIFIED | Dual fixture tokens: `p6_dual_login_free_switch_xai_to_codex` + `codex_to_xai`; history preserved `p6_history_preserved_across_successful_switch`; same-provider no extra friction `p6_same_provider_codex_switch_with_usable_creds`. |

**Score:** 3/3 truths verified (0 present, behavior-unverified)

### Plan-level supporting truths (merged, non-reducing)

| Area | Status | Spot-check |
|------|--------|------------|
| BYOK skips OAuth-slot gate | ✓ | `p6_byok_*` (3 shell) |
| Expired-but-refreshable usable | ✓ | `p6_missing_provider_apply_allows_codex_when_refreshable_token_present` |
| Failed apply has no side effects | ✓ | `p6_missing_provider_apply_no_side_effects` |
| Transactional default (no optimistic current/persist) | ✓ | `p6_transactional_default_*` (4 pager) |
| Deferred post-login auto-retry + provider-scoped poll | ✓ | `p6_login_now_*`, `p6_auth_complete_*`, `p6_external_cli_*`, `p6_focus_gained_*`, `p6_refresh_generation_*` |
| Full catalog + needs-login badge (not filter) | ✓ | `p6_needs_login_*` (6) |
| AuthMeta dual-slot usable → badge cache | ✓ | shell `AuthMeta.providers`; pager `apply_auth_meta` / `p6_provider_auth_*` (7) |
| IncompatibleAgent not collapsed into missing-provider | ✓ | `incompatible_agent_*` (4) + separate error codes |
| Phase gate / validation docs | ✓ | `06-PHASE-GATE.md`, `06-VALIDATION.md` present with per-subgroup discover |

### Required Artifacts

| Artifact | Expected | Status | Details |
| -------- | ----------- | ------ | ------- |
| `crates/codegen/xai-grok-shell/src/agent/config.rs` | `MODEL_SWITCH_MISSING_PROVIDER` + ACP payload | ✓ VERIFIED | ~5529–5611; pure `missing_provider_gate_error`; unit tests |
| `crates/codegen/xai-grok-shell/src/agent/handlers/model_switch.rs` | Authoritative apply gate | ✓ VERIFIED | Gate before prepare; BYOK + `provider_oauth_slot_usable` |
| `crates/codegen/xai-grok-shell/tests/model_switch_gate.rs` | Session harness proofs | ✓ VERIFIED | Missing-provider, dual-login, history, mid-turn, BYOK |
| `crates/codegen/xai-grok-pager/src/app/actions.rs` | `SwitchModelError::MissingProvider` | ✓ VERIFIED | Typed variant + `MissingProviderLoginAnswered` |
| `crates/codegen/xai-grok-pager/src/views/question_view.rs` | `LocalQuestionKind::MissingProviderLogin` | ✓ VERIFIED | Kind with model/effort/provider |
| `crates/codegen/xai-grok-pager/src/app/dispatch/session/lifecycle.rs` | Open question + Login now / Keep current | ✓ VERIFIED | `open_missing_provider_login_question`, `dispatch_missing_provider_login_answered`, deferred stash |
| `crates/codegen/xai-grok-pager/src/app/effects/mod.rs` | ACP → MissingProvider mapping | ✓ VERIFIED | MissingProvider before IncompatibleAgent (D-07) |
| `crates/codegen/xai-grok-shell/src/auth/meta.rs` | Dual-slot AuthMeta | ✓ VERIFIED | `providers: Option<ProviderAuthMetaSlots>` |
| `crates/codegen/xai-grok-pager/src/app/app_view.rs` | Badge cache from AuthMeta | ✓ VERIFIED | `ProviderAuthUsableSnapshot`, `apply_auth_meta` |
| `.planning/phases/.../06-VALIDATION.md` | Requirement → test map | ✓ VERIFIED | Present |
| `.planning/phases/.../06-PHASE-GATE.md` | Runnable gate | ✓ VERIFIED | Green subgroups re-run in this verification |

### Key Link Verification

| From | To | Via | Status | Details |
| ---- | --- | --- | ------ | ------- |
| `model_switch::apply` | `ModelSwitchMissingProviderError::into_acp_error` | early return after resolve when slot unusable | ✓ WIRED | Lines 73–97 model_switch.rs |
| ACP error | pager Effect::SwitchModel | `from_acp_error` → `SwitchModelError::MissingProvider` | ✓ WIRED | effects/mod.rs ~1686–1699 |
| SwitchModelComplete(MissingProvider) | QuestionView MissingProviderLogin | lifecycle handler + deferred stash | ✓ WIRED | lifecycle.rs ~1216–1259 |
| MissingProviderLoginAnswered(login=true) | deferred + provider poll / xAI login | no immediate apply | ✓ WIRED | lifecycle.rs ~1306–1374 |
| AuthMeta.providers | AppView.provider_auth | apply_auth_meta → slash/settings badges | ✓ WIRED | app_view.rs + slash model badges |
| Catalog ModelEntry.provider | gate provider id | trusted in-process catalog only | ✓ WIRED | `model.info.provider` |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| -------- | ------------- | ------ | ------------------ | ------ |
| missing-provider gate | `slot_usable` | disk `auth.json` provider slot via `read_provider_auth_store` + `provider_slot_usable`; xAI may also use AuthManager | Fixture dual/empty stores in harness | ✓ FLOWING |
| QuestionView copy | provider / model display | parsed ACP error + catalog display name | Structured error fields | ✓ FLOWING |
| needs-login badge | `provider_auth` snapshot | AuthMeta dual-slot + RefreshProviderAuthStatus | Usable flags only (no tokens) | ✓ FLOWING |
| deferred retry | `deferred_model_switch` | gate-open stash + Login now | Applied when required provider becomes usable | ✓ FLOWING |

### Behavioral Spot-Checks

Verifier re-ran phase-gate subgroups (fixture-only). All exit 0.

| Behavior | Command | Result | Status |
| -------- | ------- | ------ | ------ |
| Shell missing-provider apply | `cargo test -p xai-grok-shell --test model_switch_gate p6_missing_provider` | 3 passed | ✓ PASS |
| Shell dual-login free switch | `… p6_dual_login` | 3 passed | ✓ PASS |
| Shell same-provider / history / mid-turn | `p6_same_provider`, `p6_history`, `p6_mid_turn` | 1+1+1 passed | ✓ PASS |
| Shell BYOK | `p6_byok` | 3 passed | ✓ PASS |
| Shell unit error wire | `--lib p6_model_switch_missing_provider` | 2 passed | ✓ PASS |
| Next-sample routing | `--test provider_routing switch_changes_next_sample_route` | 1 passed | ✓ PASS |
| Pager missing-provider / transactional / login / keep / deferred | `--lib p6_*` subgroups | all discovered ≥1 and passed | ✓ PASS |
| Pager badge + AuthMeta + refresh + external CLI | `p6_needs_login`, `p6_provider_auth`, `p6_auth_`, `p6_external_cli`, … | all passed | ✓ PASS |
| IncompatibleAgent non-collapse | `--lib incompatible_agent` | 4 passed | ✓ PASS |

### Probe Execution

| Probe | Command | Result | Status |
| ----- | ------- | ------ | ------ |
| N/A | — | Phase uses cargo gate, not `scripts/**/probe-*.sh` | SKIP |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| ----------- | ---------- | ----------- | ------ | -------- |
| MOD-03 | 06-03, 06-05, 06-06 | Mid-session free switch; next turn uses new model | ✓ SATISFIED | Dual-login free switch + next-sample route tests |
| MOD-06 | 06-01–04, 06-06 | Missing-provider fail-closed + login prompt | ✓ SATISFIED | Shell gate + pager QuestionView + deferred login recovery |

No orphaned Phase 6 requirements outside MOD-03 / MOD-06.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| ---- | ---- | ------- | -------- | ------ |
| — | — | No TBD/FIXME/XXX in phase core paths | — | — |

Debt-marker scan on gate/switch/lifecycle/effects/question_view/actions: clean.

### Human Verification Required

None required for goal closure. Phase gate is fixture-only by design (T-06-17). Optional manual live OAuth smoke (real browser dual login + cross-provider chat) is out of automated gate scope and deferred to daily-driver Phase 9 if desired.

### Gaps Summary

No gaps. All three ROADMAP success criteria are implemented, wired, data-flowing, and behaviorally proven by named tests re-executed in this verification.

### Notes

- **MVP goal format:** ROADMAP phase goal is not User Story regex-valid (`As a…, I want to…, so that…`). Verification used CONTEXT/PLAN user story + success criteria. Recommend `/gsd mvp-phase 6` only if process tooling requires strict User Story goals going forward — not a product gap.
- **Pre-existing exclusions (not Phase 6 gaps):** shell integration `signed_managed_config*` compile issue; pager `tests/settings_e2e.rs` syntax — gate correctly uses `--lib` / targeted integration tests.
- **Out of scope (later phases):** cross-provider subagent spawn (Phase 7); rebrand chrome (Phase 8).

---

_Verified: 2026-07-17T00:38:26Z_  
_Verifier: Claude (gsd-verifier)_
