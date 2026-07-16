---
phase: 03-model-catalog-gpt-5-6-entries
plan: 01
subsystem: models
tags: [model-catalog, ModelProvider, gpt-5.6, default_models, codex, xai]

requires:
  - phase: 02-multi-slot-credentials-xai-oauth
    provides: "Auth slot wire ids xai/codex for provider binding alignment"
provides:
  - "ModelProvider enum (xai|codex) on full catalog type chain"
  - "Mixed embedded default_models.json with Grok + GPT-5.6 Sol/Terra/Luna"
  - "Wave 0 integration harness tests/model_catalog.rs on public APIs"
  - "ConfigModelOverride.provider parse/apply + fully_populated_override drift guard"
affects:
  - 03-02 (prefetch merge / collision)
  - 03-03 (CLI / ACP meta surface)
  - 04 (provider-aware routing)

tech-stack:
  added: []
  patterns:
    - "Explicit ModelProvider field separate from agent_type harness"
    - "Integration binary model_catalog for Phase 3 gates (avoid broken shell --lib)"
    - "Missing provider serde-defaults to xai; invalid TOML provider warns without drop"

key-files:
  created:
    - crates/codegen/xai-grok-shell/tests/model_catalog.rs
  modified:
    - crates/codegen/xai-grok-models/default_models.json
    - crates/codegen/xai-grok-models/src/lib.rs
    - crates/codegen/xai-grok-shell/src/agent/config.rs
    - crates/codegen/xai-grok-shell/src/agent/config_model_override_parse.rs
    - crates/codegen/xai-grok-shell/src/agent/models.rs
    - crates/codegen/xai-grok-shell/src/remote/client.rs
    - crates/codegen/xai-grok-shell/src/agent/mvp_agent/tests.rs
    - crates/codegen/xai-grok-shell/src/agent/subagent/tests/mod.rs

key-decisions:
  - "ModelProvider uses serde snake_case wire ids xai/codex matching auth slots"
  - "GPT agent_type stays stock (not codex harness) this phase"
  - "Remote parse always forces provider=Xai to block spoofing"
  - "Phase 3 proofs via cargo test --test model_catalog only"

patterns-established:
  - "Wave 0 integration harness under tests/ for catalog contracts when shell --lib is broken"
  - "Provider binding asserted with public wire string literals, never private PROVIDER_* imports"

requirements-completed: [MOD-01, MOD-02]

coverage:
  - id: D1
    description: "Embedded catalog includes grok-build + gpt-5.6-sol/terra/luna with provider tags"
    requirement: MOD-01
    verification:
      - kind: integration
        ref: "cargo test -p xai-grok-shell --test model_catalog catalog_includes_gpt56"
        status: pass
    human_judgment: false
  - id: D2
    description: "Explicit provider binding xai/codex on defaults and overrides; missing → xai"
    requirement: MOD-02
    verification:
      - kind: integration
        ref: "cargo test -p xai-grok-shell --test model_catalog -- --nocapture"
        status: pass
    human_judgment: false
  - id: D3
    description: "Default model remains grok-build with four+ catalog entries"
    requirement: MOD-01
    verification:
      - kind: unit
        ref: "cargo test -p xai-grok-models --lib"
        status: pass
    human_judgment: false

duration: 11min
completed: 2026-07-16
status: complete
---

# Phase 3 Plan 01: Model catalog foundation Summary

**Mixed provider-labeled catalog with `ModelProvider` (xai|codex) on the full type chain and GPT-5.6 Sol/Terra/Luna rows, proven via `tests/model_catalog` integration binary**

## Performance

- **Duration:** 11 min
- **Started:** 2026-07-16T10:08:49Z
- **Completed:** 2026-07-16T10:19:56Z
- **Tasks:** 3/3
- **Files modified:** 9

## Accomplishments

- Shipped embedded SoT with Grok Build (xAI) + GPT-5.6 Sol/Terra/Luna (Codex) using UI-SPEC names and explicit `provider` fields
- Added public `ModelProvider` end-to-end (`DefaultModelJson` → `ModelEntryConfig` → `ModelInfo` → `ConfigModelOverride` apply)
- Wave 0 `tests/model_catalog.rs` integration harness: 11 green tests covering order, binding, defaults, overrides (valid/missing/invalid)

## Task Commits

Each task was committed atomically:

1. **Task 1: Wave 0 compiling smoke harness + behavior-RED catalog key tests** - `a056664` (test)
2. **Task 2: ModelProvider schema chain + default_models.json + override parse + GREEN catalog proofs** - `037722f` (feat)
3. **Task 3: Models-crate default id sanity + shell check sweep** - `a6c93b4` (test)

## Files Created/Modified

- `crates/codegen/xai-grok-shell/tests/model_catalog.rs` — Phase 3 integration proof vehicle (public APIs only)
- `crates/codegen/xai-grok-models/default_models.json` — mixed catalog SoT (4 models, providers, UI-SPEC names)
- `crates/codegen/xai-grok-shell/src/agent/config.rs` — `ModelProvider` + provider fields + map/apply/fallback/from_config
- `crates/codegen/xai-grok-shell/src/agent/config_model_override_parse.rs` — `fully_populated_override` includes provider
- `crates/codegen/xai-grok-shell/src/remote/client.rs` — remote parse forces `provider: Xai`
- `crates/codegen/xai-grok-shell/src/agent/models.rs` — test helper `make_entry_config*` provider field
- `crates/codegen/xai-grok-shell/src/agent/mvp_agent/tests.rs` / `subagent/tests/mod.rs` — ModelInfo literal fan-out
- `crates/codegen/xai-grok-models/src/lib.rs` — unit test for default id + catalog size

## Decisions Made

- Followed plan/discretion: `ModelProvider` snake_case serde; GPT `agent_type` stock; `context_window: 272000`; `api_backend: responses`; remote always Xai
- Integration tests assert `"xai"`/`"codex"` string literals only (no private `PROVIDER_*` imports)
- No routing, Codex OAuth, missing-provider gate, CLI/ACP meta, or shell `--lib` repair

## Deviations from Plan

None - plan executed as written. Minor Rule 2: added optional models-crate unit test (plan allowed) for LazyLock/default∈ids coverage after JSON expansion. Scripted literal fan-out briefly mis-inserted into non-struct bodies (`info()`/`deref`/`make_entry_config`); fixed before Task 2 commit.

## Threat Flags

None beyond plan register. Remote path mitigates T-03-01 by forcing `provider=Xai`. No new network endpoints or auth paths.

## Known Stubs

None — catalog entries are real data rows; routing intentionally deferred to Phase 4 (not stubbed UI).

## Test Results

```
cargo test -p xai-grok-shell --test model_catalog -- --nocapture
# 11 passed

cargo test -p xai-grok-models --lib
# 1 passed

cargo check -p xai-grok-shell
# ok
```

## Self-Check: PASSED

- All key files present
- Commits `a056664`, `037722f`, `a6c93b4` present
- Automated gates green
