---
phase: 03-model-catalog-gpt-5-6-entries
plan: 03
subsystem: models
tags: [model-catalog, ACP, meta.provider, CLI, format_cli_model_row, DynamicEnum, settings, MOD-01, MOD-02]

requires:
  - phase: 03-model-catalog-gpt-5-6-entries
    provides: "ModelProvider + mixed default_models.json + prefetch remove-then-append + model_catalog harness"
provides:
  - "ACP to_acp_model_info always inserts meta.provider (xai|codex) alongside agentType"
  - "Pure ACP/list name projection asserts for (xAI)/(Codex) suffixes"
  - "pub format_cli_model_row: star/dash id (name) per UI-SPEC"
  - "Settings dynamic_enum_choices name-suffix integration tests"
  - "UI-SPEC: settings names-only inherit; meta.provider always present; empty/fuzzy empty states locked"
affects:
  - 04 (provider-aware routing)
  - 06 (missing-provider gate)
  - TUI slash /model and Settings Default model (inherit catalog names)

tech-stack:
  added: []
  patterns:
    - "Machine surface: meta.provider from trusted ModelInfo.provider only (never client input)"
    - "Human surface: provider suffix lives in catalog name; UI inherits without structural picker changes"
    - "Phase 3 gates: --test model_catalog / format_cli_model_row / dynamic_enum_model_names + cargo check shell+pager (never --lib)"

key-files:
  created:
    - crates/codegen/xai-grok-pager/tests/format_cli_model_row.rs
    - crates/codegen/xai-grok-pager/tests/dynamic_enum_model_names.rs
  modified:
    - crates/codegen/xai-grok-shell/src/agent/config.rs
    - crates/codegen/xai-grok-shell/tests/model_catalog.rs
    - crates/codegen/xai-grok-pager/src/models.rs
    - .planning/phases/03-model-catalog-gpt-5-6-entries/03-UI-SPEC.md

key-decisions:
  - "meta.provider always inserted (not optional) from ModelInfo.provider.as_str()"
  - "CLI uses star marker only — no (default) text suffix on current row"
  - "Settings DynamicEnum remains names-only inherit; empty per-row descriptions OK"
  - "Interactive /model UAT is optional advisory; automated ACP+CLI+settings gate is phase authority"

patterns-established:
  - "Public pure formatters (format_cli_model_row) enable pager integration tests without --lib repair"
  - "Selector coverage via pure projection: to_acp_model_info names + dynamic_enum_choices displays"

requirements-completed: [MOD-01, MOD-02]

coverage:
  - id: D1
    description: "ACP ModelInfo meta.provider set to xai/codex with agentType retained; name pass-through"
    requirement: MOD-01
    verification:
      - kind: integration
        ref: "cargo test -p xai-grok-shell --test model_catalog to_acp_model_info"
        status: pass
    human_judgment: false
  - id: D2
    description: "Pure ACP/list projection includes (xAI)/(Codex) name suffixes for grok-build and gpt-5.6-sol"
    requirement: MOD-02
    verification:
      - kind: integration
        ref: "cargo test -p xai-grok-shell --test model_catalog acp_list_projection"
        status: pass
    human_judgment: false
  - id: D3
    description: "CLI format_cli_model_row prints star/dash id (name) with empty-name fallback"
    requirement: MOD-01
    verification:
      - kind: integration
        ref: "cargo test -p xai-grok-pager --test format_cli_model_row"
        status: pass
    human_judgment: false
  - id: D4
    description: "Settings dynamic_enum_choices displays include (xAI)/(Codex); model-row descriptions empty OK"
    requirement: MOD-02
    verification:
      - kind: integration
        ref: "cargo test -p xai-grok-pager --test dynamic_enum_model_names"
        status: pass
    human_judgment: false
  - id: D5
    description: "Full Phase 3 automated gate including cargo check shell+pager"
    requirement: MOD-01
    verification:
      - kind: integration
        ref: "cargo test -p xai-grok-models --lib && cargo test -p xai-grok-shell --test model_catalog && cargo test -p xai-grok-pager --test format_cli_model_row && cargo test -p xai-grok-pager --test dynamic_enum_model_names && cargo check -p xai-grok-shell && cargo check -p xai-grok-pager"
        status: pass
    human_judgment: false
  - id: D6
    description: "Optional advisory interactive /model + settings visual check"
    requirement: MOD-02
    verification: []
    human_judgment: true
    rationale: "Visual TUI confirmation only; automated projection asserts already cover MOD-01/02. Skipped when no interactive TUI environment — not a phase pass or fail."

duration: 36min
completed: 2026-07-16
status: complete
---

# Phase 3 Plan 03: ACP meta.provider + CLI format + settings DynamicEnum Summary

**Provider-labeled catalog is observable on every Phase 3 surface: ACP meta.provider + name suffixes, CLI `*|- id (name)` rows, and settings DynamicEnum name inherit — all proven by integration tests without shell/pager --lib repair.**

## Performance

- **Duration:** 36 min
- **Started:** 2026-07-16T10:26:20Z
- **Completed:** 2026-07-16T11:03:12Z
- **Tasks:** 4/4 (Task 4 optional advisory skipped with environment note)
- **Files modified:** 6

## Accomplishments

- ACP `to_acp_model_info` always inserts `meta.provider` (`"xai"` / `"codex"`) from trusted in-process `ModelInfo.provider`, keeping `agentType` and name pass-through with UI-SPEC suffixes
- Pure projection assert `acp_list_projection_includes_provider_suffixes` covers slash/settings name feed without exporting private pager helpers
- Public `format_cli_model_row` + `list_available_models` print `  * {id} ({name})` / `  - {id} ({name})` per UI-SPEC (no `(default)` suffix; no Codex-login filter)
- Settings `dynamic_enum_choices(ActiveModelCatalog)` proven to carry provider-suffixed displays with empty model-row descriptions (names-only inherit)
- Full Phase 3 automated sweep green: models lib, model_catalog (23), format_cli_model_row (4), dynamic_enum_model_names (2), cargo check shell+pager

## Task Commits

Each task was committed atomically:

1. **Task 1 RED: ACP meta.provider tests** - `3e35310` (test)
2. **Task 1 GREEN: meta.provider implementation** - `901a38b` (feat)
3. **Task 2: format_cli_model_row + DynamicEnum tests** - `12b17ad` (feat)
4. **Task 3: UI-SPEC reconcile** - `76c0335` (docs)
5. **Task 4: Optional advisory /model UAT** - skipped (no interactive TUI in executor environment; automated gate is phase authority)

**Plan metadata:** (final docs commit after this SUMMARY)

## Files Created/Modified

- `crates/codegen/xai-grok-shell/src/agent/config.rs` — insert `meta.provider` after `agentType` in `to_acp_model_info`
- `crates/codegen/xai-grok-shell/tests/model_catalog.rs` — ACP meta + list projection integration tests
- `crates/codegen/xai-grok-pager/src/models.rs` — `pub fn format_cli_model_row` + list loop uses ACP name
- `crates/codegen/xai-grok-pager/tests/format_cli_model_row.rs` — CLI row shape tests
- `crates/codegen/xai-grok-pager/tests/dynamic_enum_model_names.rs` — settings name-suffix tests
- `.planning/phases/03-model-catalog-gpt-5-6-entries/03-UI-SPEC.md` — ACP meta.provider always present; settings names-only + empty/fuzzy states already locked

## Decisions Made

- **meta.provider always present** — machine consumers can rely on it; UI still uses name for labels
- **CLI star-only current marker** — dropped legacy ` (default)` parenthetical so provider suffix in name is not confused with default marker
- **Inherit-only for slash/settings** — zero production change to `slash/commands/model.rs` or settings registry; display already uses `info.name` / snapshot names
- **Task 4 advisory only** — environment skip documented; does not waive or pass phase alone

## Deviations from Plan

None - plan executed as written. Task 2 combined RED/GREEN into a single green commit after writing the public formatter and tests together (behavior still proven green under required filters). Task 4 optional checkpoint skipped per plan gate and hard constraints (no TUI).

## Issues Encountered

None blocking. Pre-existing shell/pager `--lib` compile debt left untouched by design (Q3).

## Known Stubs

None — no placeholder empty catalogs or TODO projection paths introduced.

## Threat Flags

None new beyond plan threat model. `meta.provider` mitigation applied: value copied only from trusted in-process `ModelInfo.provider` (T-03-09).

## Optional Interactive UAT (Task 4)

**Status:** Skipped — executor environment has no interactive TUI session launch requirement met; plan marks this checkpoint `gate="optional"` advisory only.

**Environment note:** Automated MOD-01/02 gate is green (ACP + CLI + settings projection + cargo checks). Manual `/model` and Settings visual check can be run later via:

```bash
cargo run -p xai-grok-pager-bin --bin bum
# then /model and Settings → Default model
```

## Test Results (Phase 3 full automated gate)

| Command | Result |
|---------|--------|
| `cargo test -p xai-grok-models --lib` | 1 passed |
| `cargo test -p xai-grok-shell --test model_catalog` | 23 passed |
| `cargo test -p xai-grok-pager --test format_cli_model_row` | 4 passed |
| `cargo test -p xai-grok-pager --test dynamic_enum_model_names` | 2 passed |
| `cargo check -p xai-grok-shell` | ok |
| `cargo check -p xai-grok-pager` | ok |

## Next Phase Readiness

- Catalog + labels complete for Phase 3 (MOD-01/02 visibility surfaces)
- Phase 4 can consume `ModelInfo.provider` / `meta.provider` for routing without reworking picker chrome
- Phase 6 may add missing-provider gates; D-12 still forbids hiding GPT rows solely for Codex logout

---
*Phase: 03-model-catalog-gpt-5-6-entries*
*Completed: 2026-07-16*

## Self-Check: PASSED

- All key files present
- Commits 3e35310 901a38b 12b17ad 76c0335 present
- Full automated gate green (models lib, model_catalog 23, format_cli 4, dynamic_enum 2, cargo check shell+pager)
