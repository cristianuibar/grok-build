---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: milestone
current_phase: 8
current_phase_name: Quiet fork & rebrand polish
current_plan: Not started
status: planning
stopped_at: Completed 07-06-PLAN.md
last_updated: "2026-07-17T09:51:16.046Z"
last_activity: 2026-07-17
last_activity_desc: Phase 7 complete, transitioned to Phase 8
progress:
  total_phases: 9
  completed_phases: 7
  total_plans: 35
  completed_plans: 35
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-07-17)

**Core value:** One CLI (`bum`) can log into both xAI and Codex and freely switch between Grok and GPT-5.6 models in a real coding session — including cross-provider subagent orchestration.
**Current focus:** Phase 8 — Quiet fork & rebrand polish (ready to plan)

## Current Position

Phase: 8 of 9 (Quiet fork & rebrand polish)
Plan: Not started
Status: Ready to plan
Last activity: 2026-07-17 — Phase 7 complete (AGENT-01..06), transitioned to Phase 8
Current Plan: Not started
Total Plans in Phase: TBD

Progress: [████████░░] 78% (7/9 phases)

## Performance Metrics

**Velocity:**

- Total plans completed: 35 (phases 1–7)
- Phase 5: 6 plans (Wave 0 RED → storage → OAuth → logout/status → refresh → gate)
- Phase 6: 6 plans (shell gate → QV → dual free switch → badges → deferred login → phase gate)
- Phase 7: 6 plans (harness → effort → spawn gate → async preflight → isolation → phase gate)

**By Phase:**

| Phase | Plans | Notes |
|-------|-------|-------|
| 01 | 5/5 | complete |
| 02 | 4/4 | complete |
| 03 | 3/3 | complete |
| 04 | 5/5 | complete |
| 05 | 6/6 | complete — AUTH-02..05 |
| 06 | 6/6 | complete — MOD-03, MOD-06 |
| 07 | 6/6 | complete — AGENT-01..06 |

---
**Per-Plan Metrics:**

| Plan | Duration | Tasks | Files |
|------|----------|-------|-------|
| Phase 06 P01 | 16min | 2 tasks | 5 files |
| Phase 06 P02 | 25min | 2 tasks | 32 files |
| Phase 06 P05 | 12min | 2 tasks | 1 files |
| Phase 06 P04 | 29min | 3 tasks | 30 files |
| Phase 06 P03 | 13min | 2 tasks | 9 files |
| Phase 06 P06 | 25min | 2 tasks | 3 files |
| Phase 07 P01 | 8min | 2 tasks | 3 files |
| Phase 07 P02 | 5min | 2 tasks | 3 files |
| Phase 07 P03 | 32min | 2 tasks | 8 files |
| Phase 07 P04 | 15min | 2 tasks | 7 files |
| Phase 07 P05 | 10min | 2 tasks | 3 files |
| Phase 07 P06 | 4min | 2 tasks | 2 files |

## Decisions

- [Phase 7]: Cross-provider spawn uses child model → catalog provider → credentials/backend; Task exposes reasoning_effort; missing child provider fails closed before pending/worktree via async preflight_spawn + pure oauth_provider_slot_usable
- [Phase 7]: Effort hard-fail only when Task/harness explicitly set effort (role/persona/definition defaults soft-skip under Tool model stamp)
- [Phase 7]: Dual-direction Authorization isolation proven with dual fake tokens; live dual-login NL E2E deferred to Phase 9
- [Phase 6]: Missing-provider gate lives in model_switch::apply (catalog provider, store_usable/credential_usable, BYOK skip)
- [Phase ?]: Introduce DeferredModelSwitch early with persist_default + required_provider for gate-open handoff
- [Phase ?]: Transactional set_default_model: no current/persist/toast until SwitchModelComplete(Ok)
- [Phase ?]: MissingProvider QuestionView Login now / Keep current; Keep current clears deferred zero-persist
- [Phase ?]: 06-05: History preserve via chat_history.jsonl snapshot; mid-turn non-cancel via hold_agent_completions + cancel_notifications spy
- [Phase ?]: 06-05: Next-sample route proven by post-switch Authorization carrying target provider token
- [Phase ?]: Stale-on-error RefreshProviderAuthStatus keeps last dual-slot cache
- [Phase ?]: needs login badge shared helper for slash /model and settings DynamicEnum; BYOK hasOwnCredentials suppress
- [Phase ?]: Lifecycle badge refresh: startup post-render + SessionCreated + AuthComplete + FocusGained (not deferred-gated)
- [Phase ?]: Login now consumes gate-open DeferredModelSwitch preserving persist_default
- [Phase ?]: Codex Login now is CLI-primary with bounded poll; never starts xAI OAuth
- [Phase ?]: try_apply_deferred only when required provider slot usable; stale poll generation ignored
- [Phase ?]: Phase 6 gate uses --lib discover for shell unit + pager because bare package list fails on unrelated integration targets
- [Phase ?]: Phase 6 validation complete: per-subgroup p6_ discovery ≥1 for MOD-03/MOD-06
- [Phase 7]: Plan 01: no TaskToolInput.reasoning_effort until Plan 02; p7_ green-only protocol
- [Phase 7]: Plan 01: Tool unknown-model is existing task_model_override_error reject; route isolation via public resolve_provider_route
- [Phase 7]: Plan 01 scaffold does not mark AGENT-01..06 complete (product proofs in Plans 02–06)
- [Phase ?]: Plan 02: Task effort parse uses local allowlist (tools↔sampling-types cycle); store canonical max→xhigh
- [Phase ?]: Plan 02: Preserve none/minimal on Task effort sanitize; NL effort guidance via task_effort_guidance
- [Phase ?]: Plan 03: authoritative missing-provider spawn gate before insert_pending/worktree in handle_subagent_request
- [Phase ?]: Plan 03: pure oauth_provider_slot_usable(auth_path, provider, live_xai) shared by model_switch and subagent
- [Phase ?]: Plan 03: Tool reasoning_effort hard-fail on parse/unsupported; Harness soft-skip
- [Phase ?]: Production design is async SubagentBackend.preflight_spawn + shell live effective-model resolve (C2-H1); sync slug-only Fn is non-goal
- [Phase ?]: Shared resolve_effective_child_model_for_spawn + missing_provider_spawn_gate_message for dual-layer preflight/spawn gate
- [Phase ?]: In-crate p7_isolation_spawn_sample_cancel harness for D-12 Authorization both dirs (C2-M4)
- [Phase ?]: auth_json_path_override dual-slot child credential resolve for deterministic isolation fixtures
- [Phase ?]: Both Grok→Codex and Codex→Grok Authorization proofs mandatory — no one-direction waiver
- [Phase ?]: Plan 06: green-only phase gate; AGENT-01 lifecycle cited existing tests; p7_preflight covers credential gate
- [Phase ?]: Plan 06: no product code — Plans 01-05 greened all p7_ filters; VALIDATION nyquist_compliant

## Session

**Last session:** 2026-07-17T09:31:00.276Z
**Stopped at:** Completed 07-06-PLAN.md
**Resume file:** None
