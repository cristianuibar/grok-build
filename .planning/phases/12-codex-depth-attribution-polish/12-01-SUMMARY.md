---
phase: 12-codex-depth-attribution-polish
plan: "01"
subsystem: testing
tags: [rust, embedded-docs, product-identity, codex, phase-gate]

requires:
  - phase: 08-quiet-fork-rebrand-polish
    provides: bum product identity and the allowlisted rebrand-gate pattern
  - phase: 10-codex-responses-wire-parity
    provides: trusted Codex route metadata and apply-patch regressions
provides:
  - RED full-inventory contracts for embedded bum identity and Codex capability disclosure
  - GREEN preset regression locking Codex:apply_patch apart from bum search_replace
  - Executable credential-free Phase 12 gate with exact discovery and staged Nyquist transitions
affects: [12-02, 12-03, 12-04, 12-05, 12-06, 12-07, phase-12-verification]

tech-stack:
  added: []
  patterns: [semantic identity classification, exact discover-before-execute, atomic validation transitions]

key-files:
  created:
    - .planning/phases/12-codex-depth-attribution-polish/12-PHASE-GATE.md
  modified:
    - crates/codegen/xai-grok-pager/src/docs.rs
    - crates/codegen/xai-grok-agent/src/config.rs

key-decisions:
  - "Treat provider/model, lineage, legal, internal, hosted-domain, and project-local compatibility references as explicit allowed categories instead of globally banning Grok or Codex words."
  - "Pin the pre-Phase-12 base at 74309d13c79ee98e0d2b0d5f58994bf3481c5ad2 so final scope checks inspect committed phase changes only."
  - "Keep all Phase 12 verification credential-free and preserve existing runtime, transport, OAuth, and tool schemas."

patterns-established:
  - "Wave 0 RED: documentation contracts compile and are discovered exactly once, then fail only on stale shipped prose."
  - "Phase gates enumerate exact test counts and exact shipped-document inventories before execution."

requirements-completed: [ID-02, OPS-04]

coverage:
  - id: D1
    description: Full embedded documentation inventory has non-vacuous bum identity and capability-disclosure contracts before prose changes.
    requirement: ID-02
    verification:
      - kind: unit
        ref: "cargo test -p xai-grok-pager --lib p12_ -- --nocapture (expected Wave 0 RED)"
        status: pass
    human_judgment: false
  - id: D2
    description: Codex and default bum presets retain distinct canonical edit-tool identities.
    requirement: OPS-04
    verification:
      - kind: unit
        ref: "crates/codegen/xai-grok-agent/src/config.rs#p12_codex_toolset_identity"
        status: pass
    human_judgment: false
  - id: D3
    description: Phase 12 has an executable credential-free gate with exact inventories, diff scope, notices, and staged validation closure.
    requirement: OPS-04
    verification:
      - kind: other
        ref: "bash .planning/phases/12-codex-depth-attribution-polish/12-PHASE-GATE.md --verify-scaffold"
        status: pass
    human_judgment: false

duration: 13 min
completed: 2026-07-21
status: complete
---

# Phase 12 Plan 01: Wave 0 Regression Contracts Summary

**Full embedded-doc identity RED contracts, canonical Codex patch-tool protection, and a credential-free exact-discovery phase gate**

## Performance

- **Duration:** 13 min
- **Started:** 2026-07-21T21:38:36Z
- **Completed:** 2026-07-21T21:52:01Z
- **Tasks:** 3
- **Files modified:** 3

## Accomplishments

- Added two full-inventory pager contracts that are intentionally RED on stale embedded prose and report the offending document or missing capability marker.
- Locked `Codex:apply_patch` in the real Codex preset while confirming bum's default preset retains `GrokBuild:search_replace` and does not inherit the Codex patch tool.
- Added an executable Phase 12 gate with pinned Rust counts, exact 22-guide/2-reference/2-entry-point inventories, semantic identity classification, notice/originator checks, committed-diff scope, check-only formatting, and atomic Nyquist closure modes.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add failing embedded-document identity and disclosure contracts** - `33ffad8` (test)
2. **Task 2: Lock the existing Codex and bum edit-tool identities** - `6ef55ad` (test)
3. **Task 3: Create the credential-free Phase 12 gate contract** - `ee9220f` (test)

## Files Created/Modified

- `crates/codegen/xai-grok-pager/src/docs.rs` - Complete embedded inventory identity scan and Authentication capability-marker contract.
- `crates/codegen/xai-grok-agent/src/config.rs` - Canonical Codex/default bum edit-tool preset regression.
- `.planning/phases/12-codex-depth-attribution-polish/12-PHASE-GATE.md` - Executable phase gate and staged validation state machine.

## Decisions Made

- Used semantic forbidden claims with explicit legitimate-reference categories, avoiding a destructive global ban on provider or lineage terms.
- Pinned the committed scope baseline to the pre-Phase-12 `HEAD`; dirty working-tree files never enter final diff evaluation.
- Kept Wave 0 documentation tests intentionally RED until Wave 2 prose changes, while all scaffold and tool-boundary checks are GREEN now.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration or credentials are required.

## Next Phase Readiness

- Ready for Plan 12-02 to add the canonical capability matrix and product entry-point wording.
- Plans 12-03 through 12-06 can use `--docs-files` immediately; Plan 12-07 owns the full Rust/static and staged validation close.
- The two pager `p12_` tests must remain RED until the embedded documentation rewrite lands.

## Self-Check: PASSED

- All three scoped files exist; the gate is executable.
- Task commits `33ffad8`, `6ef55ad`, and `ee9220f` exist in git history.
- Exact task acceptance commands pass, with the pager documentation contracts intentionally RED for the planned Wave 0 reasons.

---
*Phase: 12-codex-depth-attribution-polish*
*Completed: 2026-07-21*
