---
phase: 06-mid-session-switch-missing-provider-gate
plan: 04
subsystem: ui
tags: [model-switch, missing-provider, needs-login-badge, dual-slot-auth, AuthMeta]

requires:
  - phase: 06-mid-session-switch-missing-provider-gate
    provides: MODEL_SWITCH_MISSING_PROVIDER typed ACP error + transactional SwitchModel (Plans 01–02)
  - phase: 05-codex-oauth-dual-auth-lifecycle
    provides: AuthStatusReport / store_usable dual-slot semantics (D-02)
provides:
  - AuthMeta.providers dual-slot usable booleans (display hints only)
  - AppView provider_auth cache + set_provider_auth_usable pure API
  - Effect::RefreshProviderAuthStatus + TaskResult::ProviderAuthStatusRefreshed
  - Lifecycle refresh ownership (startup, AuthComplete, logout, FocusGained) closing H3
  - needs login badge on slash /model and settings DynamicEnum ActiveModelCatalog
  - hasOwnCredentials ACP meta BYOK badge suppress
affects:
  - 06-03 Login now deferred apply reuses RefreshProviderAuthStatus + provider_auth cache
  - 06-06 verification of badge + gate end-to-end

tech-stack:
  added: []
  patterns:
    - "Dual-slot usable producer: AuthMeta.providers + AuthStatusReport disk refresh (booleans only)"
    - "Stale-on-error refresh: ProviderAuthStatusRefreshed None keeps last cache"
    - "Shared format_model_display_with_auth_badge for slash + settings DynamicEnum"
    - "SlashController.provider_auth fan-out from AppView.set_provider_auth_usable"

key-files:
  created: []
  modified:
    - crates/codegen/xai-grok-shell/src/auth/meta.rs
    - crates/codegen/xai-grok-shell/src/auth/mod.rs
    - crates/codegen/xai-grok-shell/src/agent/mvp_agent/mod.rs
    - crates/codegen/xai-grok-shell/src/agent/config.rs
    - crates/codegen/xai-grok-pager/src/app/app_view.rs
    - crates/codegen/xai-grok-pager/src/app/actions.rs
    - crates/codegen/xai-grok-pager/src/app/effects/mod.rs
    - crates/codegen/xai-grok-pager/src/app/dispatch/task_result.rs
    - crates/codegen/xai-grok-pager/src/app/dispatch/auth.rs
    - crates/codegen/xai-grok-pager/src/app/dispatch/session/lifecycle.rs
    - crates/codegen/xai-grok-pager/src/app/event_loop.rs
    - crates/codegen/xai-grok-pager/src/slash/command.rs
    - crates/codegen/xai-grok-pager/src/slash/mod.rs
    - crates/codegen/xai-grok-pager/src/slash/commands/model.rs
    - crates/codegen/xai-grok-pager/src/settings/registry.rs
    - crates/codegen/xai-grok-pager/src/app/dispatch/settings/ui.rs

key-decisions:
  - "Stale-on-error for RefreshProviderAuthStatus (None flags keep last known cache; never log auth file)"
  - "Startup refresh path: post-render one-shot + SessionCreated/WorktreeSessionCreated emission"
  - "TUI bare /logout remains CLI-pointer; LogoutComplete fail-closes cache; FocusGained covers post-CLI logout"
  - "Display suffix · needs login (no ArgItem right_label expansion)"
  - "Shared badge helper between slash build_model_items and settings dynamic_enum_choices"

patterns-established:
  - "ProviderAuthUsableSnapshot UNKNOWN = both false until meta/refresh"
  - "FocusGained badge refresh never gated on deferred_model_switch"
  - "hasOwnCredentials true omits needs login even when slot unusable"

requirements-completed: [MOD-06]

coverage:
  - id: D1
    description: AuthMeta dual-slot usable serde + AppView cache apply independently
    requirement: MOD-06
    verification:
      - kind: unit
        ref: cargo test -p xai-grok-shell --lib p6_auth_meta
        status: pass
      - kind: unit
        ref: cargo test -p xai-grok-pager --lib p6_provider_auth
        status: pass
    human_judgment: false
  - id: D2
    description: Lifecycle refresh ownership — startup/session, AuthComplete, FocusGained, logout
    requirement: MOD-06
    verification:
      - kind: unit
        ref: cargo test -p xai-grok-pager --lib p6_provider_auth_refresh
        status: pass
      - kind: unit
        ref: cargo test -p xai-grok-pager --lib p6_refresh
        status: pass
    human_judgment: false
  - id: D3
    description: needs login badge on slash + settings DynamicEnum; full catalog; BYOK suppress
    requirement: MOD-06
    verification:
      - kind: unit
        ref: cargo test -p xai-grok-pager --lib p6_needs_login
        status: pass
      - kind: unit
        ref: cargo test -p xai-grok-shell --lib has_own_credentials
        status: pass
    human_judgment: false

duration: 29min
completed: 2026-07-16
status: complete
---

# Phase 6 Plan 04: Dual-slot badge cache & needs-login picker cues Summary

**Authoritative dual-slot usable producer (AuthMeta + disk refresh) feeds an AppView cache refreshed on startup/login/logout/focus; unusable non-BYOK models show exact ` · needs login` on slash `/model` and settings DynamicEnum without filtering the mixed catalog.**

## Performance

- **Duration:** ~29 min
- **Started:** 2026-07-16T23:08:32Z
- **Completed:** 2026-07-16T23:37:37Z
- **Tasks:** 3/3
- **Files modified:** ~30

## Accomplishments

- Extended `AuthMeta` with nested `providers.{xai,codex}.usable` (booleans only; old clients omit field)
- Populate AuthMeta from `AuthStatusReport` / `ProviderAuthMetaSlots::from_auth_file` on authenticate meta emit
- `AppView.provider_auth` cache with pure getters/setters; `apply_auth_meta` updates when providers present
- `Effect::RefreshProviderAuthStatus` + TaskResult apply with **stale-on-error** (None keeps last cache)
- Lifecycle ownership (H3 residual): post-render startup, SessionCreated, AuthComplete, LogoutComplete, FocusGained
- ACP `hasOwnCredentials` on BYOK models; shared `format_model_display_with_auth_badge` for slash + settings
- Full mixed catalog preserved; exact badge text `needs login`; `(current) · needs login` ordering locked by tests

## Task Commits

1. **Task 1: AuthMeta dual-slot producer + AppView cache + RefreshProviderAuthStatus** - `286460b` (feat)
2. **Task 2: Lifecycle refresh triggers** - `bed5c17` (feat)
3. **Task 3: hasOwnCredentials + slash/settings needs-login badge** - `a2f7b34` (feat)

## Files Created/Modified

- `crates/codegen/xai-grok-shell/src/auth/meta.rs` — ProviderAuthMetaSlots + AuthMeta.providers + tests
- `crates/codegen/xai-grok-shell/src/agent/mvp_agent/mod.rs` — fill providers from product-home auth.json
- `crates/codegen/xai-grok-shell/src/agent/config.rs` — hasOwnCredentials in to_acp_model_info
- `crates/codegen/xai-grok-pager/src/app/app_view.rs` — ProviderAuthUsableSnapshot cache + fan-out to slash controllers
- `crates/codegen/xai-grok-pager/src/app/actions.rs` — RefreshProviderAuthStatus Effect + TaskResult
- `crates/codegen/xai-grok-pager/src/app/effects/mod.rs` — disk AuthStatusReport load (booleans only)
- `crates/codegen/xai-grok-pager/src/app/dispatch/*` — task_result apply, auth complete emit, session ready emit, logout clear
- `crates/codegen/xai-grok-pager/src/app/event_loop.rs` — startup post-render + FocusGained refresh
- `crates/codegen/xai-grok-pager/src/slash/commands/model.rs` — badge helpers + build_model_items + p6_needs_login tests
- `crates/codegen/xai-grok-pager/src/settings/registry.rs` — ModelAuthHint + DynamicEnum badge policy
- `crates/codegen/xai-grok-pager/src/app/dispatch/settings/ui.rs` — snapshot wires provider_auth + model_auth_hints

## Decisions Made

- **Stale-on-error:** refresh IO/parse failure → keep last known dual-slot cache (document for Plan 03)
- **Startup path:** post-render `RefreshProviderAuthStatus` always + SessionCreated/WorktreeSessionCreated also emit
- **Logout:** in-process LogoutComplete sets both unusable; bare TUI `/logout` is CLI-pointer only — FocusGained covers post-CLI
- **Badge surface:** display suffix ` · needs login` (discretion lock; no ArgItem.right_label)
- **Shared helper** avoids slash/settings badge drift

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Critical] Fan-out provider_auth into SlashController**
- **Found during:** Task 3 (AppCtx only would stay UNKNOWN at production suggest_args)
- **Issue:** Production `SlashController::app_ctx` builds from controller state, not AppView borrow
- **Fix:** Store `provider_auth` on SlashController; `set_provider_auth_usable` fans out to welcome/agents/dashboard
- **Files modified:** `slash/mod.rs`, `app_view.rs`
- **Commit:** `a2f7b34`

**2. [Rule 2 - Critical] Settings DynamicEnum needs parallel model_auth_hints**
- **Found during:** Task 3 (available_models is name+id only)
- **Issue:** Badge needs provider + hasOwnCredentials without restructuring `(String, ModelId)` everywhere
- **Fix:** `ModelAuthHint` vec parallel to available_models; production builders fill from ACP meta
- **Files modified:** `settings/registry.rs`, `dispatch/settings/ui.rs`
- **Commit:** `a2f7b34`

## Known Stubs

None for plan goals. Plan 03 owns deferred Login-now apply body (consumes this refresh Effect).

## Verification

```
cargo test -p xai-grok-shell --lib p6_auth_meta -- --nocapture
cargo test -p xai-grok-pager --lib p6_provider_auth -- --nocapture
cargo test -p xai-grok-pager --lib p6_refresh -- --nocapture
cargo test -p xai-grok-pager --lib p6_needs_login -- --nocapture
cargo test -p xai-grok-shell --lib has_own_credentials -- --nocapture
cargo check -p xai-grok-pager
```

All green (no `|| cargo` masking).

## Self-Check: PASSED

- SUMMARY path exists
- Commits `286460b`, `bed5c17`, `a2f7b34` on main
- must_haves covered: dual usable producer, cache refresh lifecycle, needs login badge slash+settings, BYOK suppress
