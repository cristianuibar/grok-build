---
phase: 12-codex-depth-attribution-polish
plan: "02"
subsystem: documentation
tags: [embedded-docs, product-identity, codex, provider-capabilities]

requires:
  - phase: 12-01
    provides: Embedded capability-disclosure regression and fail-closed documentation classifier
provides:
  - Canonical embedded provider capability contract for xAI/Grok and ChatGPT/Codex behavior in bum
  - Root README and user-guide entry points linked to the canonical contract
  - Explicit non-stock Codex CLI identity boundary with preserved provider brands and notice links
affects: [12-03, 12-07, phase-12-verification]

tech-stack:
  added: []
  patterns: [single-source capability disclosure, allowlisted product identity classification]

key-files:
  created: []
  modified:
    - README.md
    - crates/codegen/xai-grok-pager/docs/user-guide/README.md
    - crates/codegen/xai-grok-pager/docs/user-guide/02-authentication.md

key-decisions:
  - "Keep one canonical provider capability matrix in the embedded Authentication guide and link to it from both documentation entry points."
  - "Describe Codex compatibility as route-scoped HTTP/SSE, metadata, tool, and JSON patch behavior without implying stock CLI or deferred WebSocket parity."

patterns-established:
  - "Capability claims live in one embedded contract; entry points link to it instead of duplicating the matrix."
  - "Product identity is bum while provider/model, fork-lineage, internal compatibility, and legal names remain accurate."

requirements-completed: [ID-02, OPS-04]

coverage:
  - id: D1
    description: The embedded Authentication guide is the canonical provider capability contract with explicit supported and deferred boundaries.
    requirement: OPS-04
    verification:
      - kind: unit
        ref: "cargo test -p xai-grok-pager --lib p12_capability_disclosure_is_embedded_and_complete -- --nocapture"
        status: pass
    human_judgment: false
  - id: D2
    description: The root README and user-guide index identify bum and make the canonical capability contract discoverable.
    requirement: ID-02
    verification:
      - kind: other
        ref: "bash .planning/phases/12-codex-depth-attribution-polish/12-PHASE-GATE.md --docs-files README.md crates/codegen/xai-grok-pager/docs/user-guide/README.md crates/codegen/xai-grok-pager/docs/user-guide/02-authentication.md"
        status: pass
    human_judgment: false
  - id: D3
    description: Existing Grok Build fork lineage and both third-party notice targets remain intact without equating bum with the stock Codex CLI.
    requirement: ID-02
    verification:
      - kind: other
        ref: "Plan 12-02 exact entry-point rg checks and committed Markdown diff checks"
        status: pass
    human_judgment: false

duration: 6 min
completed: 2026-07-21
status: complete
---

# Phase 12 Plan 02: Provider Capability Contract Summary

**An embedded xAI/Grok and ChatGPT/Codex capability matrix with honest bum identity, supported HTTP/SSE boundaries, and direct entry-point discovery**

## Performance

- **Duration:** 6 min
- **Started:** 2026-07-21T21:54:24Z
- **Completed:** 2026-07-21T22:00:20Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- Published one embedded provider capability matrix covering separate OAuth slots, HTTP/SSE transport, `store: false` continuity, bum-owned trusted-route metadata, standard and optional tool mappings, JSON `{ patch }` behavior, deferred parity, and the existing live-gate boundary.
- Made the contract discoverable from the root README and bum-branded user-guide index without duplicating the matrix.
- Preserved GPT-5.6/Codex/xAI/Grok brands, Grok Build fork lineage, and both existing third-party notice targets while stating that bum is not the stock OpenAI Codex CLI.

## Task Commits

Each task was committed atomically:

1. **Task 1: Write the canonical provider capability contract** - `18eed9b` (docs)
2. **Task 2: Link the capability contract from primary documentation entry points** - `6448ee7` (docs)

## Files Created/Modified

- `crates/codegen/xai-grok-pager/docs/user-guide/02-authentication.md` - Canonical embedded provider capability matrix and bum-correct authentication examples and product-home paths.
- `README.md` - Exact non-stock Codex disclaimer plus direct capability-contract link, with lineage and notice links preserved.
- `crates/codegen/xai-grok-pager/docs/user-guide/README.md` - bum-branded guide index, contract discovery text, and `bum -p` example.

## Decisions Made

- Kept the full matrix in the already embedded Authentication guide so repository and runtime users share one source of truth.
- Distinguished behavioral `codex:apply_patch` compatibility through JSON `{ patch }` from exact stock wire/name parity and avoided claims beyond existing permission and sandbox tests.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration or credentials are required.

## Next Phase Readiness

- Ready for Plan 12-03 to continue the embedded-document identity sweep.
- The focused capability-marker test and all three Plan 12-02 documentation classifier checks are green.
- The full embedded identity test remains intentionally staged until the remaining Wave 2 document plans complete.

## Self-Check: PASSED

- All three modified documentation files exist and contain the planned contract or discovery links.
- Task commits `18eed9b` and `6448ee7` exist in git history.
- Exact test discovery found one capability test; it passed 1/1.
- The `--docs-files` gate passed for both entry points and the embedded Authentication guide.
- Stub and new threat-surface scans found no incomplete implementation or runtime trust-boundary additions.

---
*Phase: 12-codex-depth-attribution-polish*
*Completed: 2026-07-21*
