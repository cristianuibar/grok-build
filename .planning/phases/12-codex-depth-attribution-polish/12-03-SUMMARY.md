---
phase: 12-codex-depth-attribution-polish
plan: "03"
subsystem: documentation
tags: [embedded-docs, product-identity, bum-home, tui]

requires:
  - phase: 12-01
    provides: Executable identity classifier and embedded-document regression contract
  - phase: 12-02
    provides: Canonical provider capability contract and attribution boundary
provides:
  - bum-first onboarding, session, headless, and TUI shortcut guidance
  - BUM_HOME and ~/.bum global configuration truth across core interaction guides
  - Explicit separation between bum identity and preserved compatibility identifiers
affects: [12-04, 12-07, phase-12-verification]

tech-stack:
  added: []
  patterns: [per-file identity classification, global-versus-project path separation]

key-files:
  created: []
  modified:
    - crates/codegen/xai-grok-pager/docs/user-guide/01-getting-started.md
    - crates/codegen/xai-grok-pager/docs/user-guide/03-keyboard-shortcuts.md
    - crates/codegen/xai-grok-pager/docs/user-guide/04-slash-commands.md
    - crates/codegen/xai-grok-pager/docs/user-guide/05-configuration.md
    - crates/codegen/xai-grok-pager/docs/user-guide/06-theming.md

key-decisions:
  - "Teach source builds for bum instead of the disabled stock x.ai installer and update channel."
  - "Retain Grok model IDs, theme assets, internal GROK_* knobs, and project-local .grok paths only as classified compatibility surfaces."
  - "Use BUM_HOME and ~/.bum consistently for all user-global configuration, credentials, sessions, rules, skills, and appearance files."

patterns-established:
  - "User-global state uses ~/.bum or BUM_HOME; repository-local discovery may retain .grok for compatibility."
  - "Internal legacy tokens are labeled as compatibility names rather than presented as product identity."

requirements-completed: [ID-02]

coverage:
  - id: D1
    description: Core onboarding and TUI guides teach bum commands and the bum global home while retaining legitimate provider, model, lineage, and compatibility references.
    requirement: ID-02
    verification:
      - kind: other
        ref: "bash .planning/phases/12-codex-depth-attribution-polish/12-PHASE-GATE.md --docs-files <five plan files>"
        status: pass
      - kind: other
        ref: "git diff 8c617ae..HEAD --check -- <five plan files>"
        status: pass
    human_judgment: false

duration: 5 min
completed: 2026-07-21
status: complete
---

# Phase 12 Plan 03: Core Guide Identity Summary

**bum-first onboarding and TUI documentation with truthful source installation, isolated `~/.bum` state, and explicitly preserved compatibility identifiers**

## Performance

- **Duration:** 5 min
- **Started:** 2026-07-21T22:01:42Z
- **Completed:** 2026-07-21T22:06:18Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments

- Replaced stock Grok product commands and the disabled x.ai installer/update workflow with bum source-build, login, session, worktree, and headless examples.
- Moved all user-global configuration, credentials, sessions, rules, skills, appearance, and LSP examples to `~/.bum` or `BUM_HOME`.
- Preserved and classified Grok model IDs, theme names/assets, internal `GROK_*` knobs, hosted/provider names, and repository-local `.grok/` compatibility paths.
- Corrected quiet-fork configuration examples so auto-update, feedback upload, and stock telemetry are not presented as enabled bum workflows.

## Task Commits

Each task was committed atomically:

1. **Task 1: Correct onboarding and keyboard documentation identity** - `0205ad5` (docs)
2. **Task 2: Correct slash, configuration, and theming documentation identity** - `1d8506d` (docs)

## Files Created/Modified

- `crates/codegen/xai-grok-pager/docs/user-guide/01-getting-started.md` - bum installation, authentication, sessions, launch options, headless mode, and global rules.
- `crates/codegen/xai-grok-pager/docs/user-guide/03-keyboard-shortcuts.md` - bum TUI subjects and bum-home configuration paths.
- `crates/codegen/xai-grok-pager/docs/user-guide/04-slash-commands.md` - bum interaction subjects, executable wording, and user skill path.
- `crates/codegen/xai-grok-pager/docs/user-guide/05-configuration.md` - `BUM_HOME` global state, quiet-fork defaults, and compatibility classifications.
- `crates/codegen/xai-grok-pager/docs/user-guide/06-theming.md` - bum theming behavior and bum-home appearance files.

## Decisions Made

- Used the repository's documented source-build path because bum has no public stock installer and its x.ai auto-update path is disabled.
- Kept project-local `.grok/` directories unchanged because they are active repository discovery compatibility paths, not user-global product state.
- Kept actual legacy configuration names such as `GROK_*`, the title item `grok`, Grok theme assets, and Grok model IDs while labeling their compatibility/provider roles.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- The identity gate initially interpreted the documented legacy title-item token as a stock executable. The token remains unchanged for compatibility and is now labeled as an internal legacy configuration name in prose that satisfies the classifier.

## Known Stubs

None. Stub-pattern matches were ordinary documented TODO/task-list feature names, not incomplete content or unwired data.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- The first core embedded-guide group is ready for the Wave 3 whole-inventory identity gate.
- Later Phase 12 guide groups can follow the same global `~/.bum` versus project-local `.grok/` classification.

## Self-Check: PASSED

- All five modified guide files exist and passed the Phase 12 per-file identity classifier, individually and as one scoped set.
- Commits `0205ad5` and `1d8506d` exist in git history.
- The committed plan diff contains exactly the five planned guide files and passes `git diff --check`.

---
*Phase: 12-codex-depth-attribution-polish*
*Completed: 2026-07-21*
