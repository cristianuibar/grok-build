---
phase: 12-codex-depth-attribution-polish
plan: "04"
subsystem: documentation
tags: [embedded-docs, product-identity, integrations, custom-models]

requires:
  - phase: 12-01
    provides: Executable identity classifier and embedded-document regression contract
  - phase: 12-03
    provides: Global bum-home versus project-local compatibility path pattern
provides:
  - bum-first MCP, skill, plugin, and hook operational guidance
  - bum-owned custom-model and project-rule configuration paths
  - Explicit classification of retained provider, SDK, environment, model, and project compatibility names
affects: [12-07, phase-12-verification]

tech-stack:
  added: []
  patterns: [per-file identity classification, global-versus-project path separation]

key-files:
  created: []
  modified:
    - crates/codegen/xai-grok-pager/docs/user-guide/07-mcp-servers.md
    - crates/codegen/xai-grok-pager/docs/user-guide/08-skills.md
    - crates/codegen/xai-grok-pager/docs/user-guide/09-plugins.md
    - crates/codegen/xai-grok-pager/docs/user-guide/10-hooks.md
    - crates/codegen/xai-grok-pager/docs/user-guide/11-custom-models.md
    - crates/codegen/xai-grok-pager/docs/user-guide/12-project-rules.md

key-decisions:
  - "Use bum and ~/.bum or BUM_HOME for every executable and user-global extension, model, hook, and rule surface."
  - "Retain project-local .grok paths as explicitly labeled compatibility contracts."
  - "Preserve provider APIs, Grok model IDs, GrokOptions, and internal GROK_* environment names while separating them from product identity."

patterns-established:
  - "Integration documentation labels legacy SDK and environment identifiers as compatibility contracts rather than product chrome."
  - "Provider and model brands remain descriptive while every operational command invokes bum."

requirements-completed: [ID-02]

coverage:
  - id: D1
    description: Extension and integration guides invoke bum and keep user-global state under the bum home without changing project compatibility semantics.
    requirement: ID-02
    verification:
      - kind: other
        ref: "Per-file and combined 12-PHASE-GATE.md --docs-files checks for guides 07-12"
        status: pass
      - kind: other
        ref: "git diff 0258498..HEAD --check -- <six plan files>"
        status: pass
    human_judgment: false

duration: 5 min
completed: 2026-07-21
status: complete
---

# Phase 12 Plan 04: Integration Guide Identity Summary

**bum-owned MCP, skills, plugins, hooks, custom-model, and project-rule guidance with explicit compatibility boundaries**

## Performance

- **Duration:** 5 min
- **Started:** 2026-07-21T22:07:35Z
- **Completed:** 2026-07-21T22:12:57Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments

- Replaced stock executable examples and product subjects across MCP, skills, plugins, hooks, custom-model, and project-rule guidance with bum.
- Moved user-global configuration, credentials, logs, plugins, hooks, skills, and rules to `~/.bum` or `BUM_HOME`.
- Preserved project-local `.grok/` discovery paths and labeled them as compatibility contracts instead of user-global product state.
- Preserved OpenAI, Anthropic, xAI/Grok model, `GrokOptions`, and internal `GROK_*` names in their real provider, SDK, model, and compatibility roles.

## Task Commits

Each task was committed atomically:

1. **Task 1: Correct MCP, skills, and plugin guide identity** - `9993c73` (docs)
2. **Task 2: Correct hooks, custom-model, and project-rules guide identity** - `ecb60c7` (docs)

## Files Created/Modified

- `crates/codegen/xai-grok-pager/docs/user-guide/07-mcp-servers.md` - bum MCP commands, global configuration, OAuth credentials, logs, and scoped project compatibility paths.
- `crates/codegen/xai-grok-pager/docs/user-guide/08-skills.md` - bum skill discovery, creation, inspection, and user-home paths.
- `crates/codegen/xai-grok-pager/docs/user-guide/09-plugins.md` - bum plugin lifecycle commands, global directories, trust guidance, and retained SDK compatibility type.
- `crates/codegen/xai-grok-pager/docs/user-guide/10-hooks.md` - bum hook lifecycle and global paths with retained internal hook environment contracts.
- `crates/codegen/xai-grok-pager/docs/user-guide/11-custom-models.md` - bum model commands and configuration while preserving provider APIs and model brands.
- `crates/codegen/xai-grok-pager/docs/user-guide/12-project-rules.md` - bum global rule discovery and CLI examples with project-local `.grok/` precedence unchanged.

## Decisions Made

- Kept `GROK_*` hook, plugin, and model variables because they are live internal compatibility contracts, and labeled their role in prose.
- Kept `GrokOptions.plugins` because it is the actual Agent SDK type, while making clear that it is a compatibility type used by bum.
- Kept xAI/Grok model IDs and provider display names, including `grok-build`, without using them as the executable or product subject.
- Kept repository-local `.grok/` extension and rule directories because the loader intentionally supports them; only user-global state moved to the bum home.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- The identity classifier initially read the provider phrase “Grok models” as a command-shaped stock executable reference. Rewording it to “models from xAI's Grok family” preserved the provider meaning and cleared the ambiguity.

## Known Stubs

None. The six documentation files contain no incomplete placeholders or unwired behavior.

## User Setup Required

None - no external service configuration or credentials are required.

## Next Phase Readiness

- Guides 07–12 are ready for the Wave 3 complete embedded-document inventory gate.
- Plan 12-07 can validate these pages with the same per-file identity classifier and committed Phase 12 scope contract.

## Self-Check: PASSED

- All six modified guide files and this summary exist on disk.
- Task commits `9993c73` and `ecb60c7` exist in git history.
- Every guide passes its individual identity gate; the exact six-file committed diff passes scope and whitespace checks.

---
*Phase: 12-codex-depth-attribution-polish*
*Completed: 2026-07-21*
