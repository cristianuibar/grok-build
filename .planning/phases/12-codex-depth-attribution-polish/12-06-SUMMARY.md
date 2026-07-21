---
phase: 12-codex-depth-attribution-polish
plan: "06"
subsystem: documentation
tags: [embedded-docs, product-identity, sandbox, permissions, hooks]

requires:
  - phase: 12-01
    provides: Executable identity classifier and complete embedded-document inventory contract
  - phase: 12-05
    provides: bum identity patterns for daily-driver workflow documentation
provides:
  - bum-first sandbox, plan-mode, background-task, terminal, and permission guidance
  - Explicitly bounded sandbox claims for the Codex-derived patch implementation
  - bum identity and isolated global paths in both compiled REFERENCE_DOCS entries
affects: [12-07, phase-12-verification]

tech-stack:
  added: []
  patterns: [per-file identity classification, global-versus-project path separation, capability-bound safety prose]

key-files:
  created: []
  modified:
    - crates/codegen/xai-grok-pager/docs/user-guide/18-sandbox.md
    - crates/codegen/xai-grok-pager/docs/user-guide/19-plan-mode.md
    - crates/codegen/xai-grok-pager/docs/user-guide/20-background-tasks.md
    - crates/codegen/xai-grok-pager/docs/user-guide/21-terminal-support.md
    - crates/codegen/xai-grok-pager/docs/user-guide/22-permissions-and-safety.md
    - crates/codegen/xai-grok-pager/docs/hooks-and-plugins.md
    - crates/codegen/xai-grok-pager/docs/custom-hooks.md

key-decisions:
  - "State explicitly that the Codex-derived patch implementation receives bum's configured permission and sandbox enforcement, not stronger stock Codex CLI guarantees."
  - "Use ~/.bum for user-global state while retaining project-local .grok, /etc/grok, GROK_*, terminal brands, and xai-grok-* crate paths as labeled compatibility or platform contracts."

patterns-established:
  - "Safety documentation distinguishes model/tool ancestry from the containment guarantees enforced by bum."
  - "Non-numbered runtime references follow the same bum identity contract as the 22 numbered embedded guides."

requirements-completed: [ID-02, OPS-04]

coverage:
  - id: D1
    description: Sandbox, planning, and background-task guides use bum commands and global paths without overstating containment.
    requirement: OPS-04
    verification:
      - kind: other
        ref: "12-PHASE-GATE.md --docs-files guides 18-sandbox.md 19-plan-mode.md 20-background-tasks.md"
        status: pass
      - kind: unit
        ref: "cargo test -p xai-grok-pager --lib p12_embedded_docs_use_bum_product_identity -- --nocapture"
        status: pass
    human_judgment: false
  - id: D2
    description: Terminal diagnostics and permission examples target bum while preserving real platform and compatibility contracts.
    requirement: ID-02
    verification:
      - kind: other
        ref: "12-PHASE-GATE.md --docs-files guides 21-terminal-support.md 22-permissions-and-safety.md"
        status: pass
    human_judgment: false
  - id: D3
    description: Both compiled reference documents identify bum for commands and user-global paths without changing hook or plugin semantics.
    requirement: ID-02
    verification:
      - kind: other
        ref: "12-PHASE-GATE.md --docs-files hooks-and-plugins.md custom-hooks.md"
        status: pass
      - kind: other
        ref: "REFERENCE_DOCS exact two-entry registration check in crates/codegen/xai-grok-pager/src/docs.rs"
        status: pass
    human_judgment: false

duration: 4 min
completed: 2026-07-21
status: complete
---

# Phase 12 Plan 06: Safety and Reference Documentation Summary

**bum-owned safety, terminal, permission, and compiled reference guidance with isolated global paths and bounded patch-tool containment claims**

## Performance

- **Duration:** 4 min
- **Started:** 2026-07-21T22:20:47Z
- **Completed:** 2026-07-21T22:25:02Z
- **Tasks:** 3
- **Files modified:** 7

## Accomplishments

- Corrected sandbox, plan-mode, and background-task product subjects, commands, session paths, event paths, and global profile paths to bum and `~/.bum`.
- Bound the Codex-derived patch implementation to bum's actual permission and OS-sandbox layers without claiming stronger stock Codex CLI containment.
- Corrected terminal diagnostics, clipboard workflows, wrapper commands, permission examples, managed paths, hooks, and interactive-grant wording while preserving real platform and compatibility identifiers.
- Brought both non-numbered documents compiled through `REFERENCE_DOCS` under the same bum identity and isolated-home contract as the numbered guide.

## Task Commits

Each task was committed atomically:

1. **Task 1: Correct sandbox, plan-mode, and background-task guide identity** - `e014ee1` (docs)
2. **Task 2: Correct terminal and permission guide identity** - `cd2e20e` (docs)
3. **Task 3: Sweep both embedded reference documents** - `4c6e88c` (docs)

## Files Created/Modified

- `crates/codegen/xai-grok-pager/docs/user-guide/18-sandbox.md` - bum commands, isolated home paths, compatibility labels, and bounded patch-tool containment language.
- `crates/codegen/xai-grok-pager/docs/user-guide/19-plan-mode.md` - bum session plan path.
- `crates/codegen/xai-grok-pager/docs/user-guide/20-background-tasks.md` - bum-owned background, monitoring, and scheduler behavior.
- `crates/codegen/xai-grok-pager/docs/user-guide/21-terminal-support.md` - bum diagnostics, clipboard, wrappers, troubleshooting, and product-home paths.
- `crates/codegen/xai-grok-pager/docs/user-guide/22-permissions-and-safety.md` - bum permission subjects, commands, user-global paths, and labeled inherited compatibility paths.
- `crates/codegen/xai-grok-pager/docs/hooks-and-plugins.md` - bum modal, marketplace command, and global plugin/hook paths.
- `crates/codegen/xai-grok-pager/docs/custom-hooks.md` - bum hook examples, global paths, logs, and labeled internal environment contracts.

## Decisions Made

- Kept `.grok/` for repository-local hook, sandbox, and permission discovery because it is an active compatibility contract, while every user-global path now uses `~/.bum`.
- Kept `/etc/grok` system-policy paths because they remain live runtime contracts and labeled them as inherited system compatibility paths rather than bum product identity.
- Kept `GROK_SANDBOX`, `GROK_CLIPBOARD_NO_DATA_CONTROL`, and hook/plugin `GROK_*` variables because they are live internal compatibility names, with their role stated explicitly.
- Kept Grok Desktop, OS, terminal, model/provider, and `xai-grok-*` crate names where they identify real platforms, ecosystems, or internal code rather than the executable.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## Known Stubs

None. The only stub-pattern match is the documented `todo_write` read-only tool name, not incomplete content or unwired behavior.

## User Setup Required

None - no external service configuration or credentials are required.

## Next Phase Readiness

- All 22 numbered guides and both `REFERENCE_DOCS` entries now participate in the bum identity contract.
- Plan 12-07 can run the complete embedded inventory, capability disclosure, notice, originator, scope, and validation closeout gates.

## Self-Check: PASSED

- All seven modified documentation files and this summary exist on disk.
- Task commits `e014ee1`, `cd2e20e`, and `4c6e88c` exist in git history and contain exactly the seven planned files.
- The combined seven-file identity gate and committed whitespace/scope checks pass.
- `p12_embedded_docs_use_bum_product_identity` passes 1/1 with 7,158 tests filtered out.
- Both reference documents remain registered exactly once in the existing `REFERENCE_DOCS` table.
- Stub and threat-surface scans found no incomplete implementation or unplanned runtime trust-boundary additions.

---
*Phase: 12-codex-depth-attribution-polish*
*Completed: 2026-07-21*
