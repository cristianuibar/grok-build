---
phase: 06-mid-session-switch-missing-provider-gate
plan: 02
subsystem: ui
tags: [model-switch, missing-provider, question-view, transactional-settings, multi-provider]

requires:
  - phase: 06-mid-session-switch-missing-provider-gate
    provides: MODEL_SWITCH_MISSING_PROVIDER typed ACP error + apply gate (Plan 01)
  - phase: 05-codex-oauth-dual-auth-lifecycle
    provides: dual-slot auth semantics
provides:
  - SwitchModelError::MissingProvider mapped from ACP before IncompatibleAgent
  - Transactional set_default_model / SwitchModel persist_default (D-05)
  - QuestionView LocalQuestionKind::MissingProviderLogin (Login now / Keep current)
  - Gate-open DeferredModelSwitch stash with required_provider + persist_default
  - Keep current zero-persist dismiss path
affects:
  - 06-03 Login now recovery consuming DeferredModelSwitch
  - 06-04 RefreshProviderAuthStatus sibling Effect (wave 3)

tech-stack:
  added: []
  patterns:
    - "Transactional model switch: no models.current / PersistSetting / toast until SwitchModelComplete(Ok)"
    - "persist_default intent threaded Effect → Complete → Ok handler / gate-open DeferredModelSwitch"
    - "MissingProvider QuestionView sibling of AgentTypeMismatch (D-07)"
    - "Pager parses provider wire id only as xai|codex; malformed → Other"

key-files:
  created: []
  modified:
    - crates/codegen/xai-grok-pager/src/app/actions.rs
    - crates/codegen/xai-grok-pager/src/app/agent.rs
    - crates/codegen/xai-grok-pager/src/app/effects/mod.rs
    - crates/codegen/xai-grok-pager/src/app/dispatch/session/lifecycle.rs
    - crates/codegen/xai-grok-pager/src/app/dispatch/settings/setters.rs
    - crates/codegen/xai-grok-pager/src/views/question_view.rs
    - crates/codegen/xai-grok-pager/src/app/agent_view/mod.rs
    - crates/codegen/xai-grok-pager/src/app/acp_handler/interactions.rs
    - crates/codegen/xai-grok-pager/src/app/dispatch/tests/task_result.rs
    - crates/codegen/xai-grok-pager/src/app/dispatch/tests/settings.rs
    - crates/codegen/xai-grok-pager-render/Cargo.toml
    - crates/codegen/xai-grok-pager/Cargo.toml

key-decisions:
  - "Introduce DeferredModelSwitch early (model_id, effort, required_provider, persist_default) instead of tuple"
  - "Live-session MissingProvider stashes full DeferredModelSwitch before opening QuestionView (cycle 3 handoff)"
  - "Login now leaves deferred intact for Plan 03; Keep current clears it"
  - "set_default_model never mutates current/default until SwitchModelComplete(Ok) with persist_default"

patterns-established:
  - "Gate-open stash: SwitchModelComplete(Err MissingProvider) → deferred_model_switch = full struct → open QuestionView"
  - "Provider wire parse at effects map_err boundary (xai|codex only)"

requirements-completed: [MOD-06]

coverage:
  - id: D1
    description: MissingProvider opens QuestionView with UI-SPEC Login now / Keep current copy
    requirement: MOD-06
    verification:
      - kind: unit
        ref: cargo test -p xai-grok-pager --lib p6_switch_model_complete_missing_provider
        status: pass
    human_judgment: false
  - id: D2
    description: Transactional default path — no optimistic current/persist before Ok
    requirement: MOD-06
    verification:
      - kind: unit
        ref: cargo test -p xai-grok-pager --lib p6_transactional_default
        status: pass
    human_judgment: false
  - id: D3
    description: Live-session settings MissingProvider stashes deferred.persist_default true
    requirement: MOD-06
    verification:
      - kind: unit
        ref: cargo test -p xai-grok-pager --lib p6_missing_provider_live_session_settings
        status: pass
    human_judgment: false
  - id: D4
    description: Keep current clears deferred with zero default PersistSetting
    requirement: MOD-06
    verification:
      - kind: unit
        ref: cargo test -p xai-grok-pager --lib p6_keep_current
        status: pass
    human_judgment: false
  - id: D5
    description: IncompatibleAgent suite still opens AgentTypeMismatch (paths not collapsed)
    requirement: MOD-06
    verification:
      - kind: unit
        ref: cargo test -p xai-grok-pager --lib incompatible_agent
        status: pass
    human_judgment: false

duration: 25min
completed: 2026-07-16
status: complete
---

# Phase 6 Plan 02: Transactional missing-provider QuestionView Summary

**Pager maps `MODEL_SWITCH_MISSING_PROVIDER` to a QuestionView (Login now / Keep current) with transactional settings switch — rejected targets never flash as current or persist as default.**

## Performance

- **Duration:** ~25 min
- **Started:** 2026-07-16T22:32:39Z
- **Completed:** 2026-07-16T22:58:10Z
- **Tasks:** 2/2
- **Files modified:** 32

## Accomplishments

- Typed `SwitchModelError::MissingProvider` sibling of IncompatibleAgent; effects try MissingProvider first then IncompatibleAgent
- Transactional `set_default_model`: only `Effect::SwitchModel { persist_default: true }` until Ok applies set_current / PersistSetting / toast
- `LocalQuestionKind::MissingProviderLogin` with exact UI-SPEC copy + CLI in Login now description
- Gate-open `DeferredModelSwitch` stash (required_provider + persist_default) for Plan 03 Login now handoff
- Keep current + cancel dismiss clear deferred with zero default persist
- Enabled `pager-render` `test-helpers` feature so `cargo test -p xai-grok-pager --lib` can compile unit tests

## Task Commits

1. **Task 1+2 support: test-helpers for cargo unit tests** - `27c03c5` (chore)
2. **Tasks 1–2: Transactional SwitchModel + MissingProvider QV + Keep current** - `7da986c` (feat)

## Files Created/Modified

- `crates/codegen/xai-grok-pager/src/app/actions.rs` — MissingProvider error, persist_default fields, MissingProviderLoginAnswered, parse_provider_wire_id
- `crates/codegen/xai-grok-pager/src/app/agent.rs` — DeferredModelSwitch struct
- `crates/codegen/xai-grok-pager/src/app/effects/mod.rs` — map_err MissingProvider-first + provider parse
- `crates/codegen/xai-grok-pager/src/app/dispatch/session/lifecycle.rs` — handle_switch_model_complete transactional Ok / MissingProvider stash + open_missing_provider_login_question + dispatch_missing_provider_login_answered
- `crates/codegen/xai-grok-pager/src/app/dispatch/settings/setters.rs` — transactional set_default_model
- `crates/codegen/xai-grok-pager/src/views/question_view.rs` — LocalQuestionKind::MissingProviderLogin
- `crates/codegen/xai-grok-pager/src/app/agent_view/mod.rs` — translate_local_submit options 0/1
- `crates/codegen/xai-grok-pager/src/app/acp_handler/interactions.rs` — exhaustive LocalQuestionKind match
- `crates/codegen/xai-grok-pager/src/app/dispatch/tests/task_result.rs` — p6_missing_provider / p6_keep_current suite
- `crates/codegen/xai-grok-pager/src/app/dispatch/tests/settings.rs` — p6_transactional_default suite
- `crates/codegen/xai-grok-pager-render/**` + Cargo.toml — test-helpers feature for cargo unit tests

## Decisions Made

- Introduced `DeferredModelSwitch` in this plan (Plan 03 shape) so gate-open stash carries `persist_default`
- Login now is a no-op apply (defers recovery body to Plan 03) while preserving stash
- Provider labels only for parsed `xai` / `codex`

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] pager unit tests could not compile under cargo**
- **Found during:** Task 1 verification
- **Issue:** `xai-grok-pager-render` test helpers were `#[cfg(test)]` only (not visible to pager lib tests); Cargo.toml commented intent but features list was empty
- **Fix:** Added `test-helpers` feature and gated public helpers with `cfg(any(test, feature = "test-helpers"))`; enabled from pager dev-dependency
- **Files modified:** pager-render sources + both Cargo.toml files
- **Commit:** `27c03c5`

**2. [Rule 2 - Critical] Cancel dismiss must clear gate-open deferred**
- **Found during:** Task 2 review of dismiss path
- **Issue:** Plan requires cancel to clear deferred like Keep current; `dismiss_question_view` only restored prompt
- **Fix:** Clear `deferred_model_switch` when dismissing MissingProviderLogin local kind
- **Files modified:** `agent_view/interactions.rs`
- **Commit:** `7da986c`

## Known Stubs

- **Login now body** intentionally leaves deferred stash for Plan 03 (no model apply, no auth effect yet) — Plan 03 owns recovery

## Verification

```
cargo test -p xai-grok-pager --lib p6_ -- --nocapture   # 12 passed
cargo test -p xai-grok-pager --lib incompatible_agent -- --nocapture  # 4 passed
cargo check -p xai-grok-pager  # green
```

## Self-Check: PASSED

- SUMMARY path exists
- Commits `27c03c5`, `7da986c` exist on main
- All must_have p6_ filters green
