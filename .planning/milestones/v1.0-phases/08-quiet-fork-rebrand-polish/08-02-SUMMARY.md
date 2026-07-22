---
phase: 08-quiet-fork-rebrand-polish
plan: 02
subsystem: ui
tags: [rebrand, id-02, clap, welcome, hero, project-picker, billing, headless, plugin_cmd, p8]

requires:
  - phase: 08-quiet-fork-rebrand-polish
    provides: Wave 1 p8_ green harness + VALIDATION map (Plan 01)
  - phase: 01-product-identity-isolated-home
    provides: BUM_HOME isolation, binary name bum
provides:
  - Pager clap name=bum about=bum TUI (D-12)
  - Welcome hero badge/subtitle/ZDR/trust product chrome as bum (D-01, D-14)
  - Project picker + billing product strings as bum; SuperGrok + grok.com host preserved
  - Headless auth + plugin_cmd residual CLI instruct bum login / bum plugin
  - Green p8_cli_brand / p8_welcome / p8_project_picker / p8_runtime_cli / p8_billing
affects:
  - 08-03 residual shell OAuth / pager-bin / minimal filters
  - 08-04 auto-update hard-off
  - 08-05 feedback quiet path
  - 08-06 phase residual greps

tech-stack:
  added: []
  patterns:
    - "Product chrome string constants (HERO_SUBTITLE, PRODUCT_BADGE_LABEL, ZDR/TRUST) for unit-testable rebrand"
    - "p8_* green product asserts co-located with surface modules (no intentional-red under p8_)"
    - "Surgical product-name substitution; keep model brands and SuperGrok commercial SKU"

key-files:
  created: []
  modified:
    - crates/codegen/xai-grok-pager/src/app/cli.rs
    - crates/codegen/xai-grok-pager/src/app/mod.rs
    - crates/codegen/xai-grok-pager/src/views/welcome/hero_box.rs
    - crates/codegen/xai-grok-pager/src/views/welcome/mod.rs
    - crates/codegen/xai-grok-pager-minimal/src/welcome.rs
    - crates/codegen/xai-grok-pager/src/project_picker/mod.rs
    - crates/codegen/xai-grok-pager/src/app/dispatch/billing.rs
    - crates/codegen/xai-grok-pager/src/app/dispatch/tests/billing.rs
    - crates/codegen/xai-grok-pager/src/app/app_view.rs
    - crates/codegen/xai-grok-pager/src/headless.rs
    - crates/codegen/xai-grok-pager/src/plugin_cmd.rs
    - crates/codegen/xai-grok-pager/tests/pty_e2e/minimal/minimal_slash_switches_from_fullscreen.rs
    - crates/codegen/xai-grok-pager/tests/pty_e2e/minimal/minimal_new_session_keeps_history_and_resets.rs
    - .planning/phases/08-quiet-fork-rebrand-polish/08-VALIDATION.md

key-decisions:
  - "Hero Full/HeroInline product span is `bum  ` with trailing spacer; stock Beta marketing span omitted (D-14)"
  - "HeroFooter shows channel gray only — no bare Beta fallback when channel empty"
  - "pager-minimal welcome product rebranded with PTY expectations (UI-SPEC pager-minimal row; Rule 2 for banner consistency)"
  - "Billing keeps SuperGrok name and https://grok.com/supergrok host; only product Grok Build → bum"
  - "Explicit residual exclusions: model catalog Grok Build (xAI)/grok-build, SuperGrok SKU, internal types, feedback notes (Plan 05)"

patterns-established:
  - "Extract locked product chrome to pub(crate) constants for p8_ unit asserts without full TUI"
  - "Flip co-located stock string equality tests (billing, help header) in same task as product change"

requirements-completed: [ID-02]

coverage:
  - id: D1
    description: "Clap PagerArgs name=bum and about=bum TUI; help header not stock Grok Build TUI"
    requirement: ID-02
    verification:
      - kind: unit
        ref: "cargo test -p xai-grok-pager --lib p8_cli_brand"
        status: pass
    human_judgment: false
  - id: D2
    description: "Hero subtitle Thanks for using bum.; badge/ZDR/trust product chrome bum; no /feedback advertise"
    requirement: ID-02
    verification:
      - kind: unit
        ref: "cargo test -p xai-grok-pager --lib p8_welcome"
        status: pass
    human_judgment: false
  - id: D3
    description: "Project picker + billing free-usage/purchase/get-most-of product bum; SuperGrok+host kept"
    requirement: ID-02
    verification:
      - kind: unit
        ref: "cargo test -p xai-grok-pager --lib p8_"
        status: pass
      - kind: unit
        ref: "cargo test -p xai-grok-pager --lib billing"
        status: pass
    human_judgment: false
  - id: D4
    description: "Headless auth instructs bum login; plugin_cmd instructs bum plugin (C1-H1 residual)"
    requirement: ID-02
    verification:
      - kind: unit
        ref: "cargo test -p xai-grok-pager --lib p8_runtime_cli"
        status: pass
    human_judgment: false
  - id: D5
    description: "Model catalog still Grok Build (xAI) for grok-build (D-02)"
    requirement: ID-02
    verification:
      - kind: unit
        ref: "cargo test -p xai-grok-pager --test dynamic_enum_model_names"
        status: pass
    human_judgment: false

duration: 11min
completed: 2026-07-17
status: complete
---

# Phase 8 Plan 02: Pager TUI product chrome Summary

**Pager clap, welcome/hero, project picker, billing product strings, and headless/plugin_cmd residual CLI present as bum; SuperGrok commercial SKU and model brands preserved; green p8_cli_brand / p8_welcome / p8_runtime_cli proofs.**

## Performance

- **Duration:** 11 min
- **Started:** 2026-07-17T10:48:01Z
- **Completed:** 2026-07-17T10:59:08Z
- **Tasks:** 3/3
- **Files modified:** 14

## Accomplishments

- Clap `PagerArgs` identity is `name=bum`, `about=bum TUI` (D-12); legacy help header asserts updated
- Welcome hero subtitle locked to `Thanks for using bum.` without `/feedback`; Full/HeroInline badge product `bum  `; Beta marketing omitted; ZDR + trust L1 product subject bum
- Project picker UI-SPEC multiline question; free-usage / purchase / get-most-of product word bum with SuperGrok + grok.com host preserved
- Headless auth errors instruct `bum login`; plugin_cmd help/errors instruct `bum plugin …` and `bum discovers skills…` (C1-H1)
- 12 green `--lib p8_` tests; billing suite green; `dynamic_enum_model_names` green (D-02)

## Task Commits

1. **Task 1: Clap + error context identity → bum (D-12)** - `b01d731` (feat)
2. **Task 2: Welcome hero badge, subtitle, ZDR/trust product lines** - `f80bd47` (feat)
3. **Task 3: Project picker + billing + pager runtime CLI residual** - `94d945c` (feat)

**Plan metadata:** `16b523d` (docs: complete plan)

## Files Created/Modified

- `crates/codegen/xai-grok-pager/src/app/cli.rs` — clap name/about + `p8_cli_brand_*`
- `crates/codegen/xai-grok-pager/src/app/mod.rs` — help header asserts for bum
- `crates/codegen/xai-grok-pager/src/views/welcome/hero_box.rs` — `HERO_SUBTITLE` + p8_welcome subtitle tests
- `crates/codegen/xai-grok-pager/src/views/welcome/mod.rs` — badge/ZDR/trust constants + p8_welcome badge/trust tests
- `crates/codegen/xai-grok-pager-minimal/src/welcome.rs` — minimal welcome product name bum
- `crates/codegen/xai-grok-pager/src/project_picker/mod.rs` — picker question + p8_project_picker test
- `crates/codegen/xai-grok-pager/src/app/dispatch/billing.rs` — free-usage / purchase / get-most-of product bum
- `crates/codegen/xai-grok-pager/src/app/dispatch/tests/billing.rs` — expected strings + `p8_billing_free_usage_product_is_bum`
- `crates/codegen/xai-grok-pager/src/app/app_view.rs` — subscribe gate fixture product bum
- `crates/codegen/xai-grok-pager/src/headless.rs` — `bum login` auth messages + `p8_runtime_cli_headless_*`
- `crates/codegen/xai-grok-pager/src/plugin_cmd.rs` — `bum plugin` instructions + `p8_runtime_cli_plugin_cmd_*`
- `crates/codegen/xai-grok-pager/tests/pty_e2e/minimal/*.rs` — welcome banner expectations → bum
- `.planning/phases/08-quiet-fork-rebrand-polish/08-VALIDATION.md` — Plan 02 filters greened

## Decisions Made

- **Omit Beta marketing chrome** on Full badge; HeroFooter no longer falls back to bare `"Beta"` when channel empty (UI-SPEC D-14)
- **pager-minimal in scope for Task 2** even though not in PLAN files list — required so PTY banner expectations and UI-SPEC pager-minimal row stay consistent
- **No new brand.rs** — constants local to welcome module / existing string sites only

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing critical] Rebranded pager-minimal welcome product name**
- **Found during:** Task 2
- **Issue:** PTY e2e files listed for update assert welcome banner text produced by `xai-grok-pager-minimal` welcome card, which still printed stock `Grok Build`
- **Fix:** Set minimal welcome product span to `bum`; update PTY expectations
- **Files modified:** `crates/codegen/xai-grok-pager-minimal/src/welcome.rs`, PTY minimal tests
- **Verification:** unit p8_welcome green; PTY files still `#[ignore]` (not executed this plan)
- **Committed in:** `f80bd47`

**2. [Rule 1 - Bug] Updated legacy clap help/name unit tests**
- **Found during:** Task 1
- **Issue:** `cli_command_name_is_grok` / help header still asserted stock product
- **Fix:** Renamed/asserted bum usage line and about
- **Files modified:** `crates/codegen/xai-grok-pager/src/app/mod.rs`
- **Committed in:** `b01d731`

**3. [Rule 2 - Missing critical] Updated billing string equality tests in same commit**
- **Found during:** Task 3
- **Issue:** Plan required flipping `dispatch/tests/billing.rs` with product strings
- **Fix:** Expected purchase / get-most-of strings → bum; added free-usage p8_ assert
- **Committed in:** `94d945c`

## Explicit residual exclusions (document for Plan 06 greps)

| Exclusion | Reason |
|-----------|--------|
| Model catalog `Grok Build (xAI)` / id `grok-build` | D-02 model brand |
| SuperGrok / SuperGrok Heavy commercial labels | Provider commercial SKU |
| `https://grok.com/supergrok…` billing hosts | External commercial URL |
| Internal crate/package names `xai-grok-*` | D-03 no crate renames |
| `GROK_*` env knobs | D-13 no env mass rename |
| Feedback thank-you / notes dispatch | Plan 05 / D-15 |
| Shell auth/OAuth/mcp residual | Plan 03 (parallel wave) |
| Agent system prompts | Deferred per plan scope |

## TDD Gate Compliance

- Plan frontmatter `type: execute` with per-task `tdd="true"`.
- Implementation + green `p8_` asserts landed together per task (no intentional-red under `p8_` per phase protocol).
- RED-only commits skipped to honor green-only `p8_` harness rule from Plan 01 / user constraints.

## Test Results

```text
cargo test -p xai-grok-pager --lib p8_cli_brand  → 3 passed
cargo test -p xai-grok-pager --lib p8_welcome    → 4 passed
cargo test -p xai-grok-pager --lib p8_           → 12 passed
cargo test -p xai-grok-pager --lib billing       → 56 passed
cargo test -p xai-grok-pager --test dynamic_enum_model_names → 2 passed
```

## Known Stubs

None — product chrome strings are live locked copy, not placeholders.

## Self-Check: PASSED

- Key files present (cli, hero_box, project_picker, headless, plugin_cmd, SUMMARY)
- Commits found: `b01d731`, `f80bd47`, `94d945c`
- Locked copy present: bum TUI, Thanks for using bum., project picker, bum login, bum plugin
