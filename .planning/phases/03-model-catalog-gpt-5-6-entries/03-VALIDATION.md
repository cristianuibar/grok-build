---
phase: 3
slug: model-catalog-gpt-5-6-entries
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-07-16
updated: 2026-07-16
---

# Phase 3 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Wave 0 uses **integration tests on public APIs** — do **not** repair the broken shell/pager `--lib` suites.
> Review cycle 1: cargo verify hygiene + Task 1 sequencing + collision + override + required UAT.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Cargo built-in `cargo test` (crate integration + models unit) |
| **Config file** | none global — per-crate; `rust-toolchain.toml` (1.92.0) |
| **Quick run command** | `cargo test -p xai-grok-shell --test model_catalog -- --nocapture` |
| **Full suite command** | `cargo test -p xai-grok-models --lib && cargo test -p xai-grok-shell --test model_catalog -- --nocapture && cargo test -p xai-grok-pager --test format_cli_model_row -- --nocapture && cargo check -p xai-grok-shell && cargo check -p xai-grok-pager` |
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

- `xai_grok_pager::models::format_cli_model_row` (make `pub` in Plan 03)

**Wire strings in tests:** assert `"xai"` / `"codex"` literals via `ModelProvider::as_str()` — do **not** import private `PROVIDER_XAI` / `PROVIDER_CODEX` into integration tests.

---

## Sampling Rate

- **After every task commit:** Run that task’s `<automated>` command (prefer `&&` chains — never `;` that masks failures)
- **After every plan wave:** Run the full suite command above for crates touched in the wave
- **Before `/gsd-verify-work`:** Full suite green + required manual `/model` + settings checkpoint (Plan 03 Task 4)
- **Max feedback latency:** ~120 seconds preferred (per integration binary after warm compile)

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 03-01-01 | 01 | 1 | MOD-01, MOD-02 | T-03-01 | Wave 0: binary compiles; `--list` discovers smoke + RED key tests | integration scaffold | `cargo test -p xai-grok-shell --test model_catalog -- --list && cargo test -p xai-grok-shell --test model_catalog harness_smoke -- --nocapture` | ❌ W0 | ⬜ pending |
| 03-01-01 | 01 | 1 | MOD-01 | T-03-04 | Catalog includes gpt-5.6 keys (behavior RED then GREEN) | integration | `cargo test -p xai-grok-shell --test model_catalog catalog_includes_gpt56 -- --nocapture` | ❌ W0 | ⬜ pending |
| 03-01-01 | 01 | 1 | MOD-02 | — | Mixed order Grok then Sol→Terra→Luna (no prefetch) | integration | `cargo test -p xai-grok-shell --test model_catalog mixed_catalog_order -- --nocapture` | ❌ W0 | ⬜ pending |
| 03-01-02 | 01 | 1 | MOD-01, MOD-02 | T-03-01/02 | Schema + JSON + remote provider=Xai + override chain | integration + check | `cargo check -p xai-grok-shell && cargo test -p xai-grok-shell --test model_catalog -- --nocapture` | ❌ after impl | ⬜ pending |
| 03-01-02 | 01 | 1 | MOD-01 | T-03-02 | Override provider valid/missing/invalid | integration | covered by full model_catalog binary (override_* tests) | ❌ after impl | ⬜ pending |
| 03-01-03 | 01 | 1 | MOD-01 | — | default_model stays grok-build; models crate ok | unit | `cargo test -p xai-grok-models --lib && cargo check -p xai-grok-shell` | ✅ partial | ⬜ pending |
| 03-02-01 | 02 | 2 | MOD-02 | T-03-05 | Prefetch survival RED tests | integration | `cargo test -p xai-grok-shell --test model_catalog codex_defaults_survive_prefetch -- --nocapture` | ❌ W0 | ⬜ pending |
| 03-02-01 | 02 | 2 | MOD-02 | T-03-05b | Prefetch collision cannot rebind Codex default to xai | integration | `cargo test -p xai-grok-shell --test model_catalog prefetched_collision_cannot_rebind_codex_default_to_xai -- --nocapture` | ❌ W0 | ⬜ pending |
| 03-02-02 | 02 | 2 | MOD-02 | T-03-05/06 | Union-append + collision authority; enterprise skip | integration | `cargo test -p xai-grok-shell --test model_catalog -- --nocapture` | ❌ after impl | ⬜ pending |
| 03-02-03 | 02 | 2 | MOD-01/02 | T-03-07 | GPT visible session + API-key; no Codex login gate | integration | `cargo test -p xai-grok-shell --test model_catalog gpt_visible -- --nocapture` | ❌ W0 | ⬜ pending |
| 03-03-01 | 03 | 3 | MOD-01 | T-03-09 | to_acp_model_info meta.provider + name pass-through | integration | `cargo test -p xai-grok-shell --test model_catalog to_acp_model_info -- --nocapture` | ❌ W0 | ⬜ pending |
| 03-03-01 | 03 | 3 | MOD-01/02 | — | Pure ACP/list name suffix projection | integration | `cargo test -p xai-grok-shell --test model_catalog acp_list_projection -- --nocapture` | ❌ W0 | ⬜ pending |
| 03-03-02 | 03 | 3 | MOD-01/02 | T-03-08 | CLI `* id (name)` / `- id (name)` pure formatter | integration | `cargo test -p xai-grok-pager --test format_cli_model_row -- --nocapture && cargo check -p xai-grok-pager` | ❌ W0 | ⬜ pending |
| 03-03-03 | 03 | 3 | MOD-01/02 | — | Phase gate automated sweep + UI-SPEC reconcile | integration + check | Full suite command | depends 01–03 | ⬜ pending |
| 03-03-04 | 03 | 3 | MOD-01/02 | — | Required manual `/model` + settings names | human checkpoint | Plan 03 Task 4 | n/a | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Scaffold during **plan 03-01 Task 1** (not a separate 03-00 plan):

- [ ] `crates/codegen/xai-grok-shell/tests/model_catalog.rs` — integration binary; **must compile and `--list` successfully** with `harness_smoke` PASS and behavior-RED key tests (no compile-fail for missing `ModelProvider`)
- [ ] Prefer constructing prefetch fixtures via `ModelEntry::fallback` + public field mutation (or `ModelEntry::from_config_entry` + `ModelEntryConfig` literals) — **do not** depend on private `prefetch_model_entry` / `test_model_entry` helpers inside `config.rs`
- [ ] Provider-field and UI-SPEC name asserts added in Plan 01 Task 2 after schema lands (still via integration binary)
- [ ] `crates/codegen/xai-grok-pager/tests/format_cli_model_row.rs` — created in plan 03-03 when `format_cli_model_row` is made `pub` (Wave 0 for CLI only; not required before Plan 01)
- [ ] No new test framework install
- [ ] **Do not** schedule repair of shell/pager `--lib` compile debt in this phase

*If a verify still says `--lib`, treat it as a plan bug and retarget to `--test model_catalog` / `--test format_cli_model_row`.*
*If a verify pipes through bare `| tail` or multi-filter cargo, treat as plan bug.*

---

## Phase Requirements → Test Map (MOD-01 / MOD-02)

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|--------------|
| MOD-01 | Catalog contains Sol/Terra/Luna with provider=codex and Codex-labeled names | integration | `cargo test -p xai-grok-shell --test model_catalog catalog_includes_gpt56 -- --nocapture` | ❌ Wave 0 |
| MOD-01 | Names match UI-SPEC provider suffix `(Codex)` / Grok `(xAI)` | integration | name asserts + `acp_list_projection` | ❌ Wave 0 |
| MOD-01 | Every entry has explicit provider; missing defaults to xai | integration | covered in full model_catalog (provider_default_xai) | ❌ Wave 0 |
| MOD-01 | Config override provider valid / missing / invalid keep-model | integration | override_* tests in model_catalog | ❌ after 01-02 |
| MOD-01 | ACP projection name + meta.provider | integration | `cargo test -p xai-grok-shell --test model_catalog to_acp_model_info -- --nocapture` | ❌ Wave 0 |
| MOD-02 | Mixed list: Grok + GPT; order Grok then Sol→Terra→Luna (no prefetch) | integration | `cargo test -p xai-grok-shell --test model_catalog mixed_catalog_order -- --nocapture` | ❌ Wave 0 |
| MOD-02 | Prefetched xAI-only map still includes GPT rows | integration | `cargo test -p xai-grok-shell --test model_catalog codex_defaults_survive_prefetch -- --nocapture` | ❌ Wave 0 |
| MOD-02 | Prefetch collision cannot rebind gpt-5.6-* to xai | integration | `… prefetched_collision_cannot_rebind_codex_default_to_xai` | ❌ Wave 0 |
| MOD-02 | Custom models endpoint does not inject GPT | integration | `… custom_endpoint_skips_codex_inject` in model_catalog | ❌ Wave 0 |
| MOD-02 | Empty `Some(IndexMap::new())` still injects Codex when `!has_custom_endpoint` (Q1) | integration | `… empty_prefetch_still_gets_codex_defaults` | ❌ Wave 0 |
| MOD-01/02 | GPT visible for session and API-key auth; not filtered by Codex credentials | integration | `cargo test -p xai-grok-shell --test model_catalog gpt_visible -- --nocapture` | ❌ Wave 0 |
| Default | `default_model()` remains `grok-build` | unit | `cargo test -p xai-grok-models --lib` (+ assert in model_catalog) | ✅ partial |
| CLI | `bum models` row format `*\|- id (name)` | integration | `cargo test -p xai-grok-pager --test format_cli_model_row -- --nocapture` | ❌ Wave 0 (Plan 03) |
| Selector UAT | Interactive `/model` + settings names | human | Plan 03 Task 4 checkpoint | n/a |

---

## Manual / Required Human Verifications

| Behavior | Requirement | Why Manual | Required? | Test Instructions |
|----------|-------------|------------|-----------|-------------------|
| Interactive slash `/model` picker rows show provider-suffixed names | MOD-01, MOD-02 | No public pure `build_model_items`; pure ACP projection is automated substitute | **Required** (Plan 03 Task 4) | Launch `bum`, open `/model`, confirm `Grok Build (xAI)` and GPT-5.6 Sol/Terra/Luna `(Codex)` in one flat list |
| Settings DynamicEnum active catalog **names** | MOD-01, MOD-02 | Settings inherits names only (descriptions empty OK per UI-SPEC) | **Required** with `/model` checkpoint | Open settings Default model enum; confirm same names; empty descriptions OK |
| Live remote prefetch against real xAI session | MOD-02 | Network/session not required for phase gate; merge proven offline | Optional | Login xAI, open picker, confirm GPT family still listed after remote models load |

**Automated substitute for inherit surfaces (required in Plan 03 Task 1/3):** assert via `model_catalog` that resolved catalog / `to_acp_model_info` **display names include** the UI-SPEC substrings `(xAI)` and `(Codex)` for the shipped default rows (`acp_list_projection_*`). That proves the name feed slash/settings consume without structural picker changes. Manual checkpoint remains required for interactive confirmation.

---

## Phase Gate

```bash
cargo test -p xai-grok-models --lib -- --nocapture
cargo test -p xai-grok-shell --test model_catalog -- --nocapture
cargo test -p xai-grok-pager --test format_cli_model_row -- --nocapture
cargo check -p xai-grok-shell
cargo check -p xai-grok-pager
```

All five automated lines green **and** Plan 03 Task 4 human checkpoint approved before `/gsd-verify-work` for Phase 3. Do **not** require shell/pager `--lib` green.

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify targeting integration tests or models `--lib` / `cargo check` (no shell/pager `--lib` gates)
- [ ] No automated command uses bare `| tail` without pipefail; no multi-filter cargo invocations
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references (`model_catalog.rs`; pager format test in Plan 03)
- [ ] Wave 0 Task 1: `--list` succeeds; RED is behavior assert not compile-fail
- [ ] Verify commands use `&&` not `;`
- [ ] No watch-mode flags
- [ ] Feedback latency < 120s preferred after warm compile
- [ ] Required manual `/model` + settings checkpoint completed (Plan 03 Task 4)
- [ ] `nyquist_compliant: true` set after validate-phase
- [ ] `wave_0_complete: true` after Plan 01 Task 1 lands the compiling integration binary

**Approval:** pending
