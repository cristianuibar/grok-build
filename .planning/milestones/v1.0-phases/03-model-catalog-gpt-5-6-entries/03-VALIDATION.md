---
phase: 3
slug: model-catalog-gpt-5-6-entries
status: complete
nyquist_compliant: true
wave_0_complete: true
created: 2026-07-16
updated: 2026-07-16
---

# Phase 3 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Wave 0 uses **integration tests on public APIs** — do **not** repair the broken shell/pager `--lib` suites.
> Review cycle 1: cargo verify hygiene + Task 1 sequencing + collision + override.
> Review cycle 2: remove-then-append collision order; automated settings DynamicEnum; optional advisory UAT.
>
> **Nyquist close-out (2026-07-16):** Phase gate re-executed green —
> models `--lib` 1/1, `model_catalog` 24/24, `format_cli_model_row` 4/4,
> `dynamic_enum_model_names` 2/2, `cargo check` shell+pager ok.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Cargo built-in `cargo test` (crate integration + models unit) |
| **Config file** | none global — per-crate; `rust-toolchain.toml` (1.92.0) |
| **Quick run command** | `cargo test -p xai-grok-shell --test model_catalog -- --nocapture` |
| **Full suite command** | `cargo test -p xai-grok-models --lib && cargo test -p xai-grok-shell --test model_catalog -- --nocapture && cargo test -p xai-grok-pager --test format_cli_model_row -- --nocapture && cargo test -p xai-grok-pager --test dynamic_enum_model_names -- --nocapture && cargo check -p xai-grok-shell && cargo check -p xai-grok-pager` |
| **Estimated runtime** | ~30–120 seconds after first compile (integration binaries only) |

### Cargo verify hygiene (locked — review HIGH)

| Rule | Detail |
|------|--------|
| One TESTNAME filter | Cargo accepts **one** positional TESTNAME filter per invocation. Do not pass multiple filters like `catalog_includes_gpt56 mixed_catalog_order`. |
| Multi-test coverage | Run the full binary: `cargo test -p xai-grok-shell --test model_catalog -- --nocapture` **or** chain separate single-filter invocations with `&&`. |
| Exit status | Never pipe cargo through bare `\| tail` (or any pipe) without `set -o pipefail`. Prefer **no pipe** so cargo’s exit code is the shell status. |
| Chains | Use `&&` only — never `;` that masks failures. |
| Forbidden gates | `cargo test -p xai-grok-shell --lib` / `cargo test -p xai-grok-pager --lib` |

### Harness policy (locked — RESEARCH Q3)

| Allowed | Forbidden |
|---------|-----------|
| `cargo test -p xai-grok-shell --test model_catalog …` | `cargo test -p xai-grok-shell --lib …` for Phase 3 gates |
| `cargo test -p xai-grok-pager --test format_cli_model_row …` | `cargo test -p xai-grok-pager --lib …` for Phase 3 gates |
| `cargo test -p xai-grok-pager --test dynamic_enum_model_names …` | Treating interactive UAT environment-blocker as phase pass |
| `cargo test -p xai-grok-models --lib` | Fixing entire shell (~32) / pager (~169) lib-test compile errors |
| `cargo check -p xai-grok-shell` / `cargo check -p xai-grok-pager` | Full workspace test as a phase gate |

**Why:** Shell and pager lib unit-test modules currently fail to compile (cross-crate `cfg(test)` leakage, missing test-only re-exports). Integration tests link the library as a normal dep and exercise **public** APIs only.

**Public API surface for shell catalog proofs:**

- `xai_grok_shell::agent::config::resolve_model_list`
- `xai_grok_shell::agent::config::Config` / `Config::new_from_toml_cfg` / `EndpointsConfig` / `ModelEntry` / `ModelInfo` / `ModelProvider` (after Plan 01)
- `xai_grok_shell::agent::config::ConfigModelOverride` / `config_models` / `model_override_warnings`
- `xai_grok_shell::agent::config::to_acp_model_info`
- `xai_grok_shell::agent::models::available_models`
- `xai_grok_models::default_model` / models list (default id sanity)

**Public API surface for CLI format proofs:**

- `xai_grok_pager::models::format_cli_model_row` (made `pub` in Plan 03)

**Public API surface for settings DynamicEnum name proofs:**

- `xai_grok_pager::settings::dynamic_enum_choices` (already `pub`)
- `xai_grok_pager::settings::{DynamicEnumSource, PagerLocalSnapshot, OwnedEnumChoice}`

**Wire strings in tests:** assert `"xai"` / `"codex"` literals via `ModelProvider::as_str()` — do **not** import private `PROVIDER_XAI` / `PROVIDER_CODEX` into integration tests.

---

## Sampling Rate

- **After every task commit:** Run that task’s `<automated>` command (prefer `&&` chains — never `;` that masks failures)
- **After every plan wave:** Run the full suite command above for crates touched in the wave
- **Before `/gsd-verify-work`:** Full suite green (ACP + CLI + settings DynamicEnum + cargo checks). Plan 03 Task 4 interactive UAT is optional advisory only.
- **Max feedback latency:** ~120 seconds preferred (per integration binary after warm compile)

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 03-01-01 | 01 | 1 | MOD-01, MOD-02 | T-03-01 | Wave 0: binary compiles; `--list` discovers smoke + RED key tests | integration scaffold | `cargo test -p xai-grok-shell --test model_catalog -- --list && cargo test -p xai-grok-shell --test model_catalog harness_smoke -- --nocapture` | ✅ | ✅ green |
| 03-01-01 | 01 | 1 | MOD-01 | T-03-04 | Catalog includes gpt-5.6 keys | integration | `cargo test -p xai-grok-shell --test model_catalog catalog_includes_gpt56 -- --nocapture` | ✅ | ✅ green |
| 03-01-01 | 01 | 1 | MOD-02 | — | Mixed order Grok then Sol→Terra→Luna (no prefetch) | integration | `cargo test -p xai-grok-shell --test model_catalog mixed_catalog_order -- --nocapture` | ✅ | ✅ green |
| 03-01-02 | 01 | 1 | MOD-01, MOD-02 | T-03-01/02 | Schema + JSON + remote provider=Xai + override chain | integration + check | `cargo check -p xai-grok-shell && cargo test -p xai-grok-shell --test model_catalog -- --nocapture` | ✅ | ✅ green |
| 03-01-02 | 01 | 1 | MOD-01 | T-03-02 | Override provider valid/missing/invalid | integration | covered by full model_catalog binary (override_* tests) | ✅ | ✅ green |
| 03-01-03 | 01 | 1 | MOD-01 | — | default_model stays grok-build; models crate ok | unit | `cargo test -p xai-grok-models --lib && cargo check -p xai-grok-shell` | ✅ | ✅ green |
| 03-02-01 | 02 | 2 | MOD-02 | T-03-05 | Prefetch survival | integration | `cargo test -p xai-grok-shell --test model_catalog codex_defaults_survive_prefetch -- --nocapture` | ✅ | ✅ green |
| 03-02-01 | 02 | 2 | MOD-02 | T-03-05b | Prefetch collision cannot rebind Codex default to xai | integration | `cargo test -p xai-grok-shell --test model_catalog prefetched_collision_cannot_rebind_codex_default_to_xai -- --nocapture` | ✅ | ✅ green |
| 03-02-01 | 02 | 2 | MOD-02 | T-03-05c | Collision order: remote first then Sol→Terra→Luna (remove-then-append) | integration | `cargo test -p xai-grok-shell --test model_catalog prefetched_codex_collision_order -- --nocapture` | ✅ | ✅ green |
| 03-02-02 | 02 | 2 | MOD-02 | T-03-05/06 | Remove-then-append + collision authority; enterprise skip | integration | `cargo test -p xai-grok-shell --test model_catalog -- --nocapture` | ✅ | ✅ green |
| 03-02-03 | 02 | 2 | MOD-01/02 | T-03-07 | GPT visible session + API-key; no Codex login gate | integration | `cargo test -p xai-grok-shell --test model_catalog gpt_visible -- --nocapture` | ✅ | ✅ green |
| 03-03-01 | 03 | 3 | MOD-01 | T-03-09 | to_acp_model_info meta.provider + name pass-through | integration | `cargo test -p xai-grok-shell --test model_catalog to_acp_model_info -- --nocapture` | ✅ | ✅ green |
| 03-03-01 | 03 | 3 | MOD-01/02 | — | Pure ACP/list name suffix projection | integration | `cargo test -p xai-grok-shell --test model_catalog acp_list_projection -- --nocapture` | ✅ | ✅ green |
| 03-03-02 | 03 | 3 | MOD-01/02 | T-03-08 | CLI `* id (name)` / `- id (name)` pure formatter | integration | `cargo test -p xai-grok-pager --test format_cli_model_row -- --nocapture && cargo check -p xai-grok-pager` | ✅ | ✅ green |
| 03-03-02 | 03 | 3 | MOD-01/02 | — | Settings DynamicEnum names via public dynamic_enum_choices | integration | `cargo test -p xai-grok-pager --test dynamic_enum_model_names -- --nocapture` | ✅ | ✅ green |
| 03-03-03 | 03 | 3 | MOD-01/02 | — | Phase gate automated sweep + UI-SPEC reconcile | integration + check | Full suite command | ✅ | ✅ green |
| 03-03-04 | 03 | 3 | MOD-01/02 | — | Optional advisory `/model` visual (not phase gate) | human optional | Plan 03 Task 4 | n/a | ⬜ optional skip |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky · ⬜ optional skip*

### Last executed results (2026-07-16)

| Suite | Command | Result |
|-------|---------|--------|
| models lib | `cargo test -p xai-grok-models --lib -- --nocapture` | 1 passed |
| model_catalog | `cargo test -p xai-grok-shell --test model_catalog -- --nocapture` | **24 passed** |
| format_cli_model_row | `cargo test -p xai-grok-pager --test format_cli_model_row -- --nocapture` | **4 passed** |
| dynamic_enum_model_names | `cargo test -p xai-grok-pager --test dynamic_enum_model_names -- --nocapture` | **2 passed** |
| shell check | `cargo check -p xai-grok-shell` | ok |
| pager check | `cargo check -p xai-grok-pager` | ok |

---

## Wave 0 Requirements

Scaffold during **plan 03-01 Task 1** (not a separate 03-00 plan):

- [x] `crates/codegen/xai-grok-shell/tests/model_catalog.rs` — integration binary; **must compile and `--list` successfully** with `harness_smoke` PASS and behavior-RED key tests (no compile-fail for missing `ModelProvider`)
- [x] Prefer constructing prefetch fixtures via `ModelEntry::fallback` + public field mutation (or `ModelEntry::from_config_entry` + `ModelEntryConfig` literals) — **do not** depend on private `prefetch_model_entry` / `test_model_entry` helpers inside `config.rs`
- [x] Provider-field and UI-SPEC name asserts added in Plan 01 Task 2 after schema lands (still via integration binary)
- [x] `crates/codegen/xai-grok-pager/tests/format_cli_model_row.rs` — created in plan 03-03 when `format_cli_model_row` is made `pub` (Wave 0 for CLI only; not required before Plan 01)
- [x] `crates/codegen/xai-grok-pager/tests/dynamic_enum_model_names.rs` — created in plan 03-03 Task 2 (settings name projection; not required before Plan 01)
- [x] No new test framework install
- [x] **Do not** schedule repair of shell/pager `--lib` compile debt in this phase

*If a verify still says `--lib`, treat it as a plan bug and retarget to `--test model_catalog` / `--test format_cli_model_row`.*
*If a verify pipes through bare `| tail` or multi-filter cargo, treat as plan bug.*

---

## Phase Requirements → Test Map (MOD-01 / MOD-02)

| Req ID | Behavior | Test Type | Automated Command | File Exists? | Status |
|--------|----------|-----------|-------------------|--------------|--------|
| MOD-01 | Catalog contains Sol/Terra/Luna with provider=codex and Codex-labeled names | integration | `cargo test -p xai-grok-shell --test model_catalog catalog_includes_gpt56 -- --nocapture` | ✅ | ✅ green |
| MOD-01 | Names match UI-SPEC provider suffix `(Codex)` / Grok `(xAI)` | integration | name asserts + `acp_list_projection` | ✅ | ✅ green |
| MOD-01 | Every entry has explicit provider; missing defaults to xai | integration | covered in full model_catalog (provider_default_xai) | ✅ | ✅ green |
| MOD-01 | Config override provider valid / missing / invalid keep-model | integration | override_* tests in model_catalog | ✅ | ✅ green |
| MOD-01 | ACP projection name + meta.provider | integration | `cargo test -p xai-grok-shell --test model_catalog to_acp_model_info -- --nocapture` | ✅ | ✅ green |
| MOD-02 | Mixed list: Grok + GPT; order Grok then Sol→Terra→Luna (no prefetch) | integration | `cargo test -p xai-grok-shell --test model_catalog mixed_catalog_order -- --nocapture` | ✅ | ✅ green |
| MOD-02 | Prefetched xAI-only map still includes GPT rows | integration | `cargo test -p xai-grok-shell --test model_catalog codex_defaults_survive_prefetch -- --nocapture` | ✅ | ✅ green |
| MOD-02 | Prefetch collision cannot rebind gpt-5.6-* to xai | integration | `… prefetched_collision_cannot_rebind_codex_default_to_xai` | ✅ | ✅ green |
| MOD-02 | Collision order: remote then Sol→Terra→Luna (remove-then-append) | integration | `… prefetched_codex_collision_order` | ✅ | ✅ green |
| MOD-02 | Custom models endpoint does not inject GPT | integration | `… custom_endpoint_skips_codex_inject` in model_catalog | ✅ | ✅ green |
| MOD-02 | Empty `Some(IndexMap::new())` still injects Codex when `!has_custom_endpoint` (Q1) | integration | `… empty_prefetch_still_gets_codex_defaults` | ✅ | ✅ green |
| MOD-01/02 | GPT visible for session and API-key auth; not filtered by Codex credentials | integration | `cargo test -p xai-grok-shell --test model_catalog gpt_visible -- --nocapture` | ✅ | ✅ green |
| Default | `default_model()` remains `grok-build` | unit | `cargo test -p xai-grok-models --lib` (+ assert in model_catalog) | ✅ | ✅ green |
| CLI | `bum models` row format `*\|- id (name)` | integration | `cargo test -p xai-grok-pager --test format_cli_model_row -- --nocapture` | ✅ | ✅ green |
| Settings | DynamicEnum names include `(xAI)`/`(Codex)`; empty description OK | integration | `cargo test -p xai-grok-pager --test dynamic_enum_model_names -- --nocapture` | ✅ | ✅ green |
| Selector UAT | Interactive `/model` visual | human optional | Plan 03 Task 4 (advisory; not phase gate) | n/a | ⬜ optional |

---

## Manual / Human Verifications

| Behavior | Requirement | Why Manual | Required? | Test Instructions |
|----------|-------------|------------|-----------|-------------------|
| Interactive slash `/model` picker rows show provider-suffixed names | MOD-01, MOD-02 | Visual polish only; ACP + settings automated cover name feed | **Optional advisory** (Plan 03 Task 4) | Launch `bum`, open `/model`, confirm `Grok Build (xAI)` and GPT-5.6 Sol/Terra/Luna `(Codex)` in one flat list |
| Settings DynamicEnum active catalog **names** | MOD-01, MOD-02 | **Automated** via `dynamic_enum_choices` integration test; interactive optional | Automated required; human optional | Automated: `--test dynamic_enum_model_names`. Optional: open settings Default model enum |
| Live remote prefetch against real xAI session | MOD-02 | Network/session not required for phase gate; merge proven offline | Optional | Login xAI, open picker, confirm GPT family still listed after remote models load |

**Automated gate for inherit surfaces (required in Plan 03 Tasks 1–3):**
1. `model_catalog` — resolved catalog / `to_acp_model_info` display names include `(xAI)` and `(Codex)` (`acp_list_projection_*`)
2. `dynamic_enum_model_names` — public `dynamic_enum_choices(ActiveModelCatalog, …)` displays include those suffixes; empty model-row description OK
3. `format_cli_model_row` — CLI `id (name)` rows

Interactive Task 4 is advisory only. Environment inability to launch TUI is a documented skip — **not** a phase pass and **not** a phase fail when automated is green.

---

## Phase Gate

```bash
cargo test -p xai-grok-models --lib -- --nocapture
cargo test -p xai-grok-shell --test model_catalog -- --nocapture
cargo test -p xai-grok-pager --test format_cli_model_row -- --nocapture
cargo test -p xai-grok-pager --test dynamic_enum_model_names -- --nocapture
cargo check -p xai-grok-shell
cargo check -p xai-grok-pager
```

All six automated lines green before `/gsd-verify-work` for Phase 3. Plan 03 Task 4 interactive check is optional. Do **not** require shell/pager `--lib` green.

**Gate result (Nyquist 2026-07-16):** all six lines green (1 + 24 + 4 + 2 tests; both checks ok).

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify targeting integration tests or models `--lib` / `cargo check` (no shell/pager `--lib` gates)
- [x] No automated command uses bare `| tail` without pipefail; no multi-filter cargo invocations
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references (`model_catalog.rs`; pager format + dynamic_enum tests in Plan 03)
- [x] Wave 0 Task 1: `--list` succeeds; RED is behavior assert not compile-fail (observing catalog_includes_gpt56 RED before Task 2 is optional)
- [x] Verify commands use `&&` not `;`
- [x] No watch-mode flags
- [x] Feedback latency < 120s preferred after warm compile
- [x] Automated settings + ACP + CLI projection green; interactive Task 4 optional only
- [x] `nyquist_compliant: true` set after validate-phase
- [x] `wave_0_complete: true` after Plan 01 Task 1 lands the compiling integration binary

**Approval:** approved (Nyquist auditor re-ran phase gate 2026-07-16)
