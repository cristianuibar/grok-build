---
phase: 03-model-catalog-gpt-5-6-entries
plan: 02
subsystem: models
tags: [model-catalog, prefetch, ModelProvider, codex, collision, remove-then-append, available_models]

requires:
  - phase: 03-model-catalog-gpt-5-6-entries
    provides: "ModelProvider + mixed default_models.json + model_catalog integration harness"
provides:
  - "resolve_model_list remove-then-append of bundled provider=Codex after remote prefetch"
  - "Collision authority: bundled codex + UI-SPEC name win over remote xai rebind on gpt-5.6-*"
  - "Collision order: remote/xAI first then Sol→Terra→Luna (not mid-list Terra)"
  - "Enterprise has_custom_endpoint skips Codex inject"
  - "Empty Some({}) still injects Codex when !has_custom_endpoint (Q1)"
  - "Dual-auth GPT visibility proofs via available_models (no Codex credential gate)"
affects:
  - 03-03 (CLI / ACP meta surface)
  - 04 (provider-aware routing)
  - 06 (missing-provider gate)

tech-stack:
  added: []
  patterns:
    - "Prefetch replace preserves xAI prune; only ModelProvider::Codex rows are re-appended"
    - "shift_remove all three gpt-5.6-* keys then insert Sol→Terra→Luna (remove-then-append)"
    - "Phase 3 proofs stay on cargo test --test model_catalog (not shell --lib)"

key-files:
  created: []
  modified:
    - crates/codegen/xai-grok-shell/src/agent/config.rs
    - crates/codegen/xai-grok-shell/tests/model_catalog.rs

key-decisions:
  - "Collision policy is remove-then-append, never IndexMap replace-in-place (D-11 / cycle 2)"
  - "Match provider == ModelProvider::Codex only — never agent_type == codex (D-05)"
  - "Empty Some(IndexMap) still gets Codex re-append when !has_custom_endpoint (Q1)"
  - "available_models unchanged; GPT visibility is supported_in_api, not Codex login (D-12)"

patterns-established:
  - "Codex union-append layer sits immediately after resolved = prefetched, before config_models"
  - "Double-gate inject with !has_custom_endpoint so enterprise custom catalogs stay clean"

requirements-completed: [MOD-01, MOD-02]

coverage:
  - id: D1
    description: "Codex GPT defaults survive remote-only prefetch with provider=codex"
    requirement: MOD-01
    verification:
      - kind: integration
        ref: "cargo test -p xai-grok-shell --test model_catalog codex_defaults_survive_prefetch"
        status: pass
    human_judgment: false
  - id: D2
    description: "Collision authority + Sol→Terra→Luna append order after remote (remove-then-append)"
    requirement: MOD-01
    verification:
      - kind: integration
        ref: "cargo test -p xai-grok-shell --test model_catalog prefetched_codex_collision_order"
        status: pass
    human_judgment: false
  - id: D3
    description: "Bundled cannot be rebound to xai on gpt-5.6-sol; UI-SPEC name wins"
    requirement: MOD-02
    verification:
      - kind: integration
        ref: "cargo test -p xai-grok-shell --test model_catalog prefetched_collision_cannot_rebind"
        status: pass
    human_judgment: false
  - id: D4
    description: "Enterprise custom endpoint skips Codex inject; empty prefetch still injects when default endpoints"
    requirement: MOD-01
    verification:
      - kind: integration
        ref: "cargo test -p xai-grok-shell --test model_catalog -- --nocapture"
        status: pass
    human_judgment: false
  - id: D5
    description: "GPT visible for session and API-key auth; grok-build remains session-only; no Codex login filter"
    requirement: MOD-02
    verification:
      - kind: integration
        ref: "cargo test -p xai-grok-shell --test model_catalog gpt_visible"
        status: pass
    human_judgment: false

duration: 3min
completed: 2026-07-16
status: complete
---

# Phase 3 Plan 02: Prefetch Codex union-append Summary

**Remote prefetch no longer erases GPT-5.6 Sol/Terra/Luna — remove-then-append restores bundled Codex rows after replace with authoritative provider/name and stable end-of-list order**

## Performance

- **Duration:** 3 min
- **Started:** 2026-07-16T10:22:23Z
- **Completed:** 2026-07-16T10:25:10Z
- **Tasks:** 3/3
- **Files modified:** 2

## Accomplishments

- After `resolved = prefetched`, bundled `provider == Codex` rows are `shift_remove`d then re-appended in JSON order Sol→Terra→Luna when `!has_custom_endpoint`
- Collision proofs: remote cannot rebind `gpt-5.6-sol` to xai; terra-first prefetch still ends remote → sol → terra → luna
- Enterprise custom models endpoint never receives forced GPT bundles; empty `Some({})` still injects Codex on default endpoints (Q1)
- Dual-auth visibility: GPT listed for session and API-key; grok-build stays session-only; pure catalog path (no Codex credential gate)
- Full `cargo test -p xai-grok-shell --test model_catalog` green: **19 passed**

## Task Commits

Each task was committed atomically:

1. **Task 1: RED prefetch-survival + collision + enterprise + visibility tests** - `339b0f8` (test)
2. **Task 2: Remove-then-append Codex bundled defaults after prefetch replace** - `4600325` (feat)
3. **Task 3: Dual-auth GPT visibility proof (grok session-only contrast)** - `f773d26` (test)

## Files Created/Modified

- `crates/codegen/xai-grok-shell/src/agent/config.rs` — Codex remove-then-append layer in `resolve_model_list` after prefetch replace
- `crates/codegen/xai-grok-shell/tests/model_catalog.rs` — survival, prune, enterprise skip, empty inject, collision authority/order, mixed list, dual-auth visibility

## Decisions Made

- **Remove-then-append only** for the three bundled Codex keys — never replace-in-place (preserves D-11 order when prefetch listed Terra early)
- **Provider field match only** (`ModelProvider::Codex`) — not `agent_type` (D-05 stock harness)
- **Q1:** empty prefetched map still gets Codex inject when not on custom endpoint
- **models.rs untouched** — visibility already correct via `supported_in_api` / `visible_for_auth`

## Deviations from Plan

None - plan executed exactly as written.

Minor Task 3 strengthening only: added explicit grok-build session-only contrast asserts inside `gpt_visible_session_and_api_key` (already planned behavior; no production change).

## TDD Gate Compliance

1. RED: `339b0f8` — `test(03-02): add failing prefetch survival collision visibility tests` (`codex_defaults_survive_prefetch` failed with keys=`["secret-xyz"]` before merge)
2. GREEN: `4600325` — `feat(03-02): remove-then-append Codex defaults after prefetch replace` (full binary 19/19 pass)
3. Test strengthen: `f773d26` — visibility contrast (still GREEN)

## Known Stubs

None.

## Threat Flags

None beyond plan register (T-03-05 / T-03-05b / T-03-05c mitigated by remove-then-append + Codex-only re-append; T-03-06 gated by `has_custom_endpoint`).

## Test Results

```text
cargo test -p xai-grok-shell --test model_catalog -- --nocapture
# 19 passed; 0 failed

cargo test -p xai-grok-shell --test model_catalog codex_defaults_survive_prefetch -- --nocapture  # pass
cargo test -p xai-grok-shell --test model_catalog prefetched_codex_collision_order -- --nocapture  # pass
cargo test -p xai-grok-shell --test model_catalog gpt_visible -- --nocapture  # pass
```

No shell `--lib` gates; no bare `| tail` cargo pipes.

## Self-Check: PASSED

- FOUND: `crates/codegen/xai-grok-shell/src/agent/config.rs`
- FOUND: `crates/codegen/xai-grok-shell/tests/model_catalog.rs`
- FOUND: commits `339b0f8`, `4600325`, `f773d26`
- FOUND: SUMMARY at `.planning/phases/03-model-catalog-gpt-5-6-entries/03-02-SUMMARY.md`
