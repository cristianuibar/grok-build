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

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Cargo built-in `cargo test` (crate integration + models unit) |
| **Config file** | none global — per-crate; `rust-toolchain.toml` (1.92.0) |
| **Quick run command** | `cargo test -p xai-grok-shell --test model_catalog -- --nocapture` |
| **Full suite command** | `cargo test -p xai-grok-models --lib && cargo test -p xai-grok-shell --test model_catalog -- --nocapture && cargo test -p xai-grok-pager --test format_cli_model_row -- --nocapture` |
| **Estimated runtime** | ~30–120 seconds after first compile (integration binaries only) |

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
- `xai_grok_shell::agent::config::Config` / `EndpointsConfig` / `ModelEntry` / `ModelInfo` / `ModelProvider` (after Plan 01)
- `xai_grok_shell::agent::config::to_acp_model_info`
- `xai_grok_shell::agent::models::available_models`
- `xai_grok_models::default_model` / models list (default id sanity)

**Public API surface for CLI format proofs:**

- `xai_grok_pager::models::format_cli_model_row` (make `pub` in Plan 03)

---

## Sampling Rate

- **After every task commit:** Run that task’s `<automated>` command (prefer `&&` chains — never `;` that masks failures)
- **After every plan wave:** Run the full suite command above for crates touched in the wave
- **Before `/gsd-verify-work`:** Full suite green + `cargo check -p xai-grok-shell` + `cargo check -p xai-grok-pager`
- **Max feedback latency:** ~120 seconds preferred (per integration binary after warm compile)

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 03-01-01 | 01 | 1 | MOD-01, MOD-02 | T-03-01 | Wave 0: integration binary compiles against public config APIs | integration scaffold | `cargo test -p xai-grok-shell --test model_catalog -- --list` | ❌ W0 | ⬜ pending |
| 03-01-01 | 01 | 1 | MOD-01 | T-03-04 | Catalog includes gpt-5.6-sol/terra/luna + Codex names (RED then GREEN) | integration | `cargo test -p xai-grok-shell --test model_catalog catalog_includes_gpt56 -- --nocapture` | ❌ W0 | ⬜ pending |
| 03-01-01 | 01 | 1 | MOD-02 | — | Mixed order Grok then Sol→Terra→Luna (no prefetch) | integration | `cargo test -p xai-grok-shell --test model_catalog mixed_catalog_order -- --nocapture` | ❌ W0 | ⬜ pending |
| 03-01-01 | 01 | 1 | MOD-01 | T-03-04 | Missing provider deserializes to xai | integration | `cargo test -p xai-grok-shell --test model_catalog provider_default_xai -- --nocapture` | ❌ W0 | ⬜ pending |
| 03-01-02 | 01 | 1 | MOD-01, MOD-02 | T-03-01 | Schema + JSON + remote provider=Xai | integration + check | `cargo test -p xai-grok-shell --test model_catalog catalog_includes_gpt56 mixed_catalog_order provider_default_xai -- --nocapture && cargo check -p xai-grok-shell` | ❌ after impl | ⬜ pending |
| 03-01-03 | 01 | 1 | MOD-01 | — | default_model stays grok-build; models crate ok | unit | `cargo test -p xai-grok-models --lib && cargo check -p xai-grok-shell` | ✅ partial | ⬜ pending |
| 03-02-01 | 02 | 2 | MOD-02 | T-03-05 | Prefetch survival RED tests | integration | `cargo test -p xai-grok-shell --test model_catalog codex_defaults_survive_prefetch -- --nocapture` | ❌ W0 | ⬜ pending |
| 03-02-02 | 02 | 2 | MOD-02 | T-03-05/06 | Union-append Codex defaults; enterprise skip | integration | `cargo test -p xai-grok-shell --test model_catalog resolve_model_list codex_defaults_survive_prefetch catalog_includes_gpt56 -- --nocapture` | ❌ after impl | ⬜ pending |
| 03-02-03 | 02 | 2 | MOD-01/02 | T-03-07 | GPT visible session + API-key; no Codex login gate | integration | `cargo test -p xai-grok-shell --test model_catalog available_models gpt_visible -- --nocapture` | ❌ W0 | ⬜ pending |
| 03-03-01 | 03 | 3 | MOD-01 | T-03-09 | to_acp_model_info meta.provider + name pass-through | integration | `cargo test -p xai-grok-shell --test model_catalog to_acp_model_info -- --nocapture` | ❌ W0 | ⬜ pending |
| 03-03-02 | 03 | 3 | MOD-01/02 | T-03-08 | CLI `* id (name)` / `- id (name)` pure formatter | integration | `cargo test -p xai-grok-pager --test format_cli_model_row -- --nocapture && cargo check -p xai-grok-pager` | ❌ W0 | ⬜ pending |
| 03-03-03 | 03 | 3 | MOD-01/02 | — | Phase gate sweep + inherit name suffix assert | integration | Full suite command | depends 01–03 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Scaffold during **plan 03-01 Task 1** (not a separate 03-00 plan):

- [ ] `crates/codegen/xai-grok-shell/tests/model_catalog.rs` — integration binary importing only public `xai_grok_shell::agent::{config,models}` symbols; initially may contain a smoke test that `resolve_model_list(&Config::default(), None)` returns non-empty map, then RED catalog contracts
- [ ] Prefer constructing prefetch fixtures via `ModelEntry::fallback` + public field mutation (or `ModelEntry::from_config_entry` + `ModelEntryConfig` literals) — **do not** depend on private `prefetch_model_entry` / `test_model_entry` helpers inside `config.rs`
- [ ] `crates/codegen/xai-grok-pager/tests/format_cli_model_row.rs` — created in plan 03-03 when `format_cli_model_row` is made `pub` (Wave 0 for CLI only; not required before Plan 01)
- [ ] No new test framework install
- [ ] **Do not** schedule repair of shell/pager `--lib` compile debt in this phase

*If a verify still says `--lib`, treat it as a plan bug and retarget to `--test model_catalog` / `--test format_cli_model_row`.*

---

## Phase Requirements → Test Map (MOD-01 / MOD-02)

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|--------------|
| MOD-01 | Catalog contains Sol/Terra/Luna with provider=codex and Codex-labeled names | integration | `cargo test -p xai-grok-shell --test model_catalog catalog_includes_gpt56 -- --nocapture` | ❌ Wave 0 |
| MOD-01 | Names match UI-SPEC provider suffix `(Codex)` / Grok `(xAI)` | integration | same + name asserts in catalog_includes / grok_entry tests | ❌ Wave 0 |
| MOD-01 | Every entry has explicit provider; missing defaults to xai | integration | `cargo test -p xai-grok-shell --test model_catalog provider_default_xai -- --nocapture` | ❌ Wave 0 |
| MOD-01 | ACP projection name + meta.provider | integration | `cargo test -p xai-grok-shell --test model_catalog to_acp_model_info -- --nocapture` | ❌ Wave 0 |
| MOD-02 | Mixed list: Grok + GPT; order Grok then Sol→Terra→Luna (no prefetch) | integration | `cargo test -p xai-grok-shell --test model_catalog mixed_catalog_order -- --nocapture` | ❌ Wave 0 |
| MOD-02 | Prefetched xAI-only map still includes GPT rows | integration | `cargo test -p xai-grok-shell --test model_catalog codex_defaults_survive_prefetch -- --nocapture` | ❌ Wave 0 |
| MOD-02 | Custom models endpoint does not inject GPT | integration | `… custom_endpoint_skips_codex_inject` in model_catalog | ❌ Wave 0 |
| MOD-02 | Empty `Some(IndexMap::new())` still injects Codex when `!has_custom_endpoint` (Q1) | integration | `… empty_prefetch_still_gets_codex_defaults` | ❌ Wave 0 |
| MOD-01/02 | GPT visible for session and API-key auth; not filtered by Codex credentials | integration | `cargo test -p xai-grok-shell --test model_catalog gpt_visible -- --nocapture` | ❌ Wave 0 |
| Default | `default_model()` remains `grok-build` | unit | `cargo test -p xai-grok-models --lib` (+ assert in model_catalog) | ✅ partial |
| CLI | `bum models` row format `*\|- id (name)` | integration | `cargo test -p xai-grok-pager --test format_cli_model_row -- --nocapture` | ❌ Wave 0 (Plan 03) |

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Interactive slash `/model` picker rows show provider-suffixed names | MOD-01, MOD-02 | Inherit-only UI (`build_model_items` uses `info.name`); no public pure unit path without TUI harness | After Plans 01–03: launch `bum`, open `/model`, confirm rows like `Grok Build (xAI)` and `GPT-5.6 Sol (Codex)` in one flat list |
| Settings DynamicEnum active catalog labels | MOD-01, MOD-02 | Same inherit path via catalog names | Optional: open settings model enum; confirm same names |
| Live remote prefetch against real xAI session | MOD-02 | Network/session not required for phase gate; merge proven offline | Optional: login xAI, open picker, confirm GPT family still listed after remote models load |

**Automated substitute for inherit surfaces (required in Plan 03 Task 3):** assert via `model_catalog` that resolved catalog / `available_models` / `to_acp_model_info` **display names include** the UI-SPEC substrings `(xAI)` and `(Codex)` for the shipped default rows. That proves the name feed slash/settings consume without structural picker changes.

---

## Phase Gate

```bash
cargo test -p xai-grok-models --lib -- --nocapture
cargo test -p xai-grok-shell --test model_catalog -- --nocapture
cargo test -p xai-grok-pager --test format_cli_model_row -- --nocapture
cargo check -p xai-grok-shell
cargo check -p xai-grok-pager
```

All five green before `/gsd-verify-work` for Phase 3. Do **not** require shell/pager `--lib` green.

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify targeting integration tests or models `--lib` / `cargo check` (no shell/pager `--lib` gates)
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references (`model_catalog.rs`; pager format test in Plan 03)
- [ ] Verify commands use `&&` not `;`
- [ ] No watch-mode flags
- [ ] Feedback latency < 120s preferred after warm compile
- [ ] `nyquist_compliant: true` set after validate-phase
- [ ] `wave_0_complete: true` after Plan 01 Task 1 lands the integration binary

**Approval:** pending
