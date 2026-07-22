---
phase: 08-quiet-fork-rebrand-polish
plan: 03
subsystem: identity
tags: [p8, rebrand, oauth, shell, pager-bin, minimal, c1-h1, id-02]

requires:
  - phase: 08-quiet-fork-rebrand-polish
    provides: Wave 1 p8_ harness + VALIDATION map (08-01)
  - phase: 01-product-identity-isolated-home
    provides: Binary name bum, BUM_HOME isolation
provides:
  - OAuth browser return-to-bum callback copy
  - Shell residual CLI recovery (auth/device-code/mcp-doctor/plugin) → bum
  - Open bum browser prompt + residual headless login copy
  - pager-minimal welcome product name locked as bum (p8_minimal_welcome)
  - pager-bin banner bum (pager) + crash/server residual identity
  - Green p8_oauth_return, p8_shell_runtime_cli, p8_minimal_welcome, p8_bin_
affects:
  - 08-04 auto-update hard-off (preserve banner; edit gates only)
  - 08-05 Sentry/OTLP (client labels still internal)
  - 08-06 residual inventory greps (C1-H1 shell/bin sites closed)

tech-stack:
  added: []
  patterns:
    - "Pure format/const helpers for product chrome so p8_ unit tests avoid full async_main"
    - "pager test-helpers feature for dependent crate --lib tests (cfg(test) never propagates)"
    - "Product CLI name bum in recovery strings; leave GROK_* env, model brands, IdP hosts"

key-files:
  created: []
  modified:
    - crates/codegen/xai-grok-shell/src/auth/oidc/login.rs
    - crates/codegen/xai-grok-shell/src/agent/app.rs
    - crates/codegen/xai-grok-shell/src/auth/error.rs
    - crates/codegen/xai-grok-shell/src/auth/device_code.rs
    - crates/codegen/xai-grok-shell/src/mcp_doctor.rs
    - crates/codegen/xai-grok-shell/src/plugin.rs
    - crates/codegen/xai-grok-pager-minimal/src/welcome.rs
    - crates/codegen/xai-grok-pager-minimal/Cargo.toml
    - crates/codegen/xai-grok-pager/Cargo.toml
    - crates/codegen/xai-grok-pager/src/minimal/api.rs
    - crates/codegen/xai-grok-pager/src/app/agent.rs
    - crates/codegen/xai-grok-pager/src/app/agent_view/mod.rs
    - crates/codegen/xai-grok-pager-bin/src/main.rs
    - .planning/phases/08-quiet-fork-rebrand-polish/08-VALIDATION.md

key-decisions:
  - "OAuth success message only; access-denied path and IdP branding unchanged"
  - "Shell residual inventory includes agent/app headless login strings (beyond Open bum)"
  - "Plan 02 already set minimal welcome to bum; Plan 03 locks constant + p8_ proof"
  - "pager-bin residual includes setup/workspace/update CLI instructions (product name only)"
  - "Sentry client grok-pager and OTEL/update gate logic left for Plans 04–05"

patterns-established:
  - "Extract oauth_callback_message / format_pager_banner / agent_server_startup_line / last_session_crash_line for p8_"
  - "xai-grok-pager feature test-helpers re-exports minimal_api + render test helpers to dependents"

requirements-completed: [ID-02]

coverage:
  - id: D1
    description: "OAuth browser success message returns users to bum"
    requirement: ID-02
    verification:
      - kind: unit
        ref: "cargo test -p xai-grok-shell --lib p8_oauth_return"
        status: pass
    human_judgment: false
  - id: D2
    description: "Shell auth/device-code/mcp-doctor/plugin recovery instruct bum CLI"
    requirement: ID-02
    verification:
      - kind: unit
        ref: "cargo test -p xai-grok-shell --lib p8_shell_runtime_cli"
        status: pass
    human_judgment: false
  - id: D3
    description: "Open bum browser prompt (no Open Grok Build)"
    requirement: ID-02
    verification:
      - kind: other
        ref: "rg Open bum agent/app.rs && ! rg Open Grok Build"
        status: pass
    human_judgment: false
  - id: D4
    description: "pager-minimal welcome product span is bum"
    requirement: ID-02
    verification:
      - kind: unit
        ref: "cargo test -p xai-grok-pager-minimal --lib p8_minimal_welcome"
        status: pass
    human_judgment: false
  - id: D5
    description: "pager-bin banner + agent server + crash residual present as bum"
    requirement: ID-02
    verification:
      - kind: unit
        ref: "cargo test -p xai-grok-pager-bin --bin bum p8_bin_"
        status: pass
    human_judgment: false

duration: 45min
completed: 2026-07-17
status: complete
---

# Phase 8 Plan 03: Shell/bin residual chrome rebrand Summary

**OAuth return, shell recovery CLI, minimal welcome, and pager-bin banner/crash/server identity present as bum with green p8_ proofs (C1-H1 / ID-02).**

## Performance

- **Duration:** ~45 min
- **Started:** 2026-07-17T10:48:10Z
- **Completed:** 2026-07-17T11:35:00Z
- **Tasks:** 3/3
- **Files modified:** 14

## Accomplishments

- OAuth success callback: `You can close this window and return to bum.` (error path unchanged)
- Shell residual CLI: auth errors, device-code, mcp doctor, plugin install/list instruct `bum login` / `bum mcp` / `bum plugin`
- Open bum browser prompt; residual headless login/session copy in `agent/app.rs`
- Minimal welcome product constant + green `p8_minimal_welcome` (Plan 02 already set display string)
- Composition-root banner `bum (pager) - v{version}`; serve/crash residual `bum agent server` / `bum crashed`
- VALIDATION map Plan 03 rows greened

## Task Commits

1. **Task 1: OAuth + Open bum + shell runtime CLI residual** - `057a65f` (feat)
2. **Task 2: pager-minimal welcome product name → bum** - `2887ff1` (feat)
3. **Task 3: pager-bin banner + residual server/crash identity** - `9cfcb0e` (feat)

**Plan metadata:** `7d88bda` (docs: complete plan)

## Files Created/Modified

- `crates/codegen/xai-grok-shell/src/auth/oidc/login.rs` — oauth_callback_message + p8_oauth_return
- `crates/codegen/xai-grok-shell/src/agent/app.rs` — Open bum; headless login recovery
- `crates/codegen/xai-grok-shell/src/auth/error.rs` — bum login Display strings + p8_shell_runtime_cli_auth
- `crates/codegen/xai-grok-shell/src/auth/device_code.rs` — bum login device-code recovery + p8 test
- `crates/codegen/xai-grok-shell/src/mcp_doctor.rs` — recovery_copy constants + p8 test
- `crates/codegen/xai-grok-shell/src/plugin.rs` — bum plugin Display strings + p8 test
- `crates/codegen/xai-grok-pager-minimal/src/welcome.rs` — MINIMAL_WELCOME_PRODUCT_NAME + p8_minimal_welcome
- `crates/codegen/xai-grok-pager{,-minimal}/Cargo.toml` + pager minimal_api/agent helpers — test-helpers feature
- `crates/codegen/xai-grok-pager-bin/src/main.rs` — banner/crash/server helpers + residual CLI + p8_bin_
- `08-VALIDATION.md` — Plan 03 filters greened

## Decisions Made

- Combined green product edits + p8_ tests per task (phase green-only filter policy; no intentional-red under p8_)
- Expanded shell residual slightly into `agent/app.rs` headless login strings for C1-H1 completeness
- Enabled `test-helpers` on pager so `cargo test -p xai-grok-pager-minimal --lib` can see helpers (dependency `#[cfg(test)]` is never active)
- Left Sentry `client: "grok-pager"`, update gate functions, and OTEL init untouched (Plans 04–05)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] pager-minimal --lib tests could not compile**
- **Found during:** Task 2
- **Issue:** `xai-grok-pager-minimal` unit tests call `minimal_api::test_agent_view` etc. gated only by dependency `#[cfg(test)]`, which is never enabled for dependents — pre-existing, blocked plan verify command
- **Fix:** Added `test-helpers` feature on `xai-grok-pager` (forwards render test-helpers); re-gated minimal_api / agent / agent_view helpers with `any(test, feature = "test-helpers")`; enabled feature in pager-minimal dev-dep
- **Files modified:** pager Cargo.toml, minimal/api.rs, agent.rs, agent_view/mod.rs, pager-minimal Cargo.toml
- **Committed in:** `2887ff1`

**2. [Rule 2 - Missing critical] residual headless login strings in agent/app.rs**
- **Found during:** Task 1
- **Issue:** Beyond Open Grok Build, same file still instructed `grok login` / `grok agent stdio`
- **Fix:** Rebranded user-facing recovery to bum (left grok.com IdP session wording)
- **Files modified:** agent/app.rs
- **Committed in:** `057a65f`

**3. [Rule 2 - Missing critical] pager-bin residual CLI beyond banner/crash/server**
- **Found during:** Task 3
- **Issue:** setup/workspace/update/eprintln paths still named stock `grok` product CLI
- **Fix:** Product-name rebrand only; no update/OTEL/Sentry behavior change
- **Files modified:** pager-bin main.rs
- **Committed in:** `9cfcb0e`

### Coordination with Plan 02

- Plan 02 concurrent wave already set minimal welcome Span text to `"bum"` (`f80bd47`); Task 2 extracted constant + proof only
- No shared file conflicts with Plan 02 pager TUI clap/hero ownership

## Auth Gates

None.

## Known Stubs

None — product strings are live; tests assert real helpers/constants.

## Threat Flags

None new — OAuth HTML still token-free success message only (T-08-04 mitigated by rename-only).

## Explicit exclusions (for Plan 06 residual gate)

- Model catalog `Grok Build (xAI)` / id `grok-build`
- SuperGrok commercial names
- Internal crate names `xai-grok-*`, types `GrokAuth`, headers `x-grok-client-*`
- Sentry client label `grok-pager` (Plan 05)
- `GROK_*` env family (D-13)
- Agent system prompts (deferred)

## Self-Check: PASSED

- All key files present (shell oauth/auth/mcp/plugin, minimal welcome, pager-bin main, SUMMARY)
- Commits present: `057a65f`, `2887ff1`, `9cfcb0e`
- Content checks: return to bum, Open bum, bum login, MINIMAL_WELCOME_PRODUCT_NAME=bum, bum (pager)
- Filters green: p8_oauth_return (1), p8_shell_runtime_cli (4), p8_minimal_welcome (1), p8_bin_ (3)
