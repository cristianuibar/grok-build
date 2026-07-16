---
phase: 4
slug: provider-aware-request-routing
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-07-16
---

# Phase 4 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Wave 0 uses **integration tests on public APIs** — do **not** repair the broken shell `--lib` suite.
> Prove MOD-04/MOD-05 with **fake tokens only** — no live ChatGPT / Codex OAuth required.
>
> **Cargo verify hygiene:** one TESTNAME filter per `cargo test`; never bare `| tail` without `set -o pipefail`; chain with `&&` only.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Cargo built-in `cargo test` (crate integration tests) |
| **Config file** | none global — per-crate; `rust-toolchain.toml` (1.92.0) |
| **Quick run command** | `cargo test -p xai-grok-shell --test provider_routing -- --nocapture` |
| **Full suite command** | `cargo test -p xai-grok-shell --test provider_routing -- --nocapture && cargo test -p xai-grok-shell --test model_catalog -- --nocapture && cargo check -p xai-grok-shell -p xai-grok-sampler` |
| **Estimated runtime** | ~30–120 seconds after first compile (integration binaries only) |

### Cargo verify hygiene (locked)

| Rule | Detail |
|------|--------|
| One TESTNAME filter | Cargo accepts **one** positional TESTNAME filter per invocation |
| Multi-test coverage | Run full binary without filter **or** chain single-filter invocations with `&&` |
| Exit status | Never pipe cargo through bare `\| tail` without `set -o pipefail`. Prefer **no pipe** |
| Chains | Use `&&` only — never `;` that masks failures |
| Forbidden gates | `cargo test -p xai-grok-shell --lib` |

### Harness policy

| Allowed | Forbidden |
|---------|-----------|
| `cargo test -p xai-grok-shell --test provider_routing …` | `cargo test -p xai-grok-shell --lib …` for Phase 4 gates |
| `cargo test -p xai-grok-shell --test model_catalog …` (regression) | Fixing entire shell lib-test compile errors |
| `cargo check -p xai-grok-shell -p xai-grok-sampler` | Live ChatGPT login as phase gate |
| Public pure helpers + catalog APIs | Full ACP e2e binary as required gate |

**Public API surface for Phase 4 proofs:**

- `xai_grok_shell::agent::config::{Config, EndpointsConfig, resolve_model_list, ModelEntry, ModelInfo, ModelProvider}`
- `resolve_credentials` / `resolve_credentials_for_provider` (Plan 03)
- `resolve_provider_route` / `ProviderRoute` / `CODEX_BASE_URL_DEFAULT` (Plan 02)
- `sampling_config_for_model` / `inject_url_derived_headers` / `ResolvedCredentials`
- Pure policy helpers: `session_key_for_model_provider`, `should_attach_xai_auth_manager_bearer_resolver` (Plan 04)
- Optional: `read_provider_access_token` for multi-slot fixture reads

**Wire strings:** assert `"xai"` / `"codex"` literals — do **not** import private `PROVIDER_*` into integration tests.

**Fake tokens:** `xai-fake-token`, `codex-fake-token` (or similar) only.

---

## Sampling Rate

- **After every task commit:** Run that task’s `<automated>` command
- **After every plan wave:** Full `provider_routing` binary (+ model_catalog after Plan 03+)
- **Before `/gsd-verify-work`:** Full suite command green
- **Max feedback latency:** ~120 seconds preferred after warm compile

---

## Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|--------------|
| MOD-04 | xAI model → proxy/xAI base + xAI credential | integration | `cargo test -p xai-grok-shell --test provider_routing xai_model_routes -- --nocapture` | ❌ Wave 0 |
| MOD-05 | Codex model → Codex base + Codex credential (fake) | integration | `cargo test -p xai-grok-shell --test provider_routing codex_model_routes -- --nocapture` | ❌ Wave 0 |
| MOD-04/05 | Explicit model base_url override wins | integration | `… model_override_base_url_wins` | ❌ Wave 0 |
| SC-3 | Switch changes next sample base_url + key | integration | `… switch_changes_next_sample_route` | ❌ Wave 0 |
| Safety | No X-XAI headers on Codex base | integration | `… no_proxy_headers_on_codex` | ❌ Wave 0 |
| Safety | Never cross-slot tokens | integration | `… never_cross_slot` | ❌ Wave 0 |
| Safety | Bearer resolver attach policy | integration | `… should_attach_xai_auth_manager_bearer_resolver` | ❌ Plan 04 |
| Regression | model_catalog still green after stamping | integration | `cargo test -p xai-grok-shell --test model_catalog -- --nocapture` | ✅ exists |

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------------|-----------------|-----------|-------------------|-------------|--------|
| 04-01-01 | 01 | 1 | MOD-04, MOD-05 | T-04-01 | Wave 0: binary compiles; --list; smoke green; RED dual-route contracts | integration scaffold | `cargo test -p xai-grok-shell --test provider_routing -- --list && cargo test -p xai-grok-shell --test provider_routing provider_routing_harness_smoke -- --nocapture` | ❌ W0 | ⬜ pending |
| 04-01-02 | 01 | 1 | MOD-04, MOD-05 | T-04-02 | never_cross_slot intent + proxy header locks + api_backend from model | integration | `cargo test -p xai-grok-shell --test provider_routing xai_proxy_headers_still_apply -- --nocapture` | ❌ W0 | ⬜ pending |
| 04-02-01 | 02 | 2 | MOD-05 | T-04-04 | CODEX_BASE_URL_DEFAULT + resolve_codex_base_url | integration | `cargo check -p xai-grok-shell && cargo test -p xai-grok-shell --test provider_routing resolve_codex_base_url -- --nocapture` | ❌ | ⬜ pending |
| 04-02-02 | 02 | 2 | MOD-04, MOD-05 | T-04-03 | Pure resolve_provider_route (xai/codex/override) | integration | `cargo test -p xai-grok-shell --test provider_routing resolve_provider_route -- --nocapture && cargo check -p xai-grok-shell` | ❌ | ⬜ pending |
| 04-03-01 | 03 | 3 | MOD-04, MOD-05 | T-04-02 | default_models stamps by provider | integration | `cargo test -p xai-grok-shell --test provider_routing codex_model_routes -- --nocapture && cargo test -p xai-grok-shell --test provider_routing xai_model_routes -- --nocapture` | ❌ | ⬜ pending |
| 04-03-02 | 03 | 3 | MOD-04, MOD-05 | T-04-02/06 | Provider-aware credentials; never cross-slot; Codex skips XAI_API_KEY | integration | `cargo test -p xai-grok-shell --test provider_routing never_cross_slot -- --nocapture && cargo test -p xai-grok-shell --test provider_routing model_override_base_url_wins -- --nocapture && cargo check -p xai-grok-shell` | ❌ | ⬜ pending |
| 04-04-01 | 04 | 4 | MOD-04, MOD-05 | T-04-02 | prepare + ModelsManager slot-aware; switch next sample | integration | `cargo check -p xai-grok-shell && cargo test -p xai-grok-shell --test provider_routing switch_changes_next_sample_route -- --nocapture && cargo test -p xai-grok-shell --test provider_routing session_key_for_model_provider -- --nocapture` | ❌ | ⬜ pending |
| 04-04-02 | 04 | 4 | MOD-04, MOD-05 | T-04-02 | reconstruct does not attach xAI bearer for Codex | integration | `cargo check -p xai-grok-shell -p xai-grok-sampler && cargo test -p xai-grok-shell --test provider_routing should_attach_xai_auth_manager_bearer_resolver -- --nocapture && cargo test -p xai-grok-shell --test provider_routing never_cross_slot -- --nocapture` | ❌ | ⬜ pending |
| 04-05-01 | 05 | 5 | MOD-04, MOD-05 | T-04-06 | Observability without secrets; empty Codex route; header locks | integration | `cargo test -p xai-grok-shell --test provider_routing empty_codex -- --nocapture && cargo test -p xai-grok-shell --test provider_routing no_proxy_headers_on_codex -- --nocapture && cargo test -p xai-grok-shell --test provider_routing xai_proxy_headers_still_apply -- --nocapture` | ❌ | ⬜ pending |
| 04-05-02 | 05 | 5 | MOD-04, MOD-05 | T-04-02 | Phase gate full suite | integration + check | Full suite command | ❌ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `crates/codegen/xai-grok-shell/tests/provider_routing.rs` — MOD-04/MOD-05 route + switch + header + cross-slot contracts
- [ ] Dual-endpoint Codex expectations live in provider_routing (not shell --lib `default_models_dual_endpoint_routing`)
- [ ] Framework install: none

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Live dual-provider daily-driver turn | Phase 9 | Needs real ChatGPT OAuth (Phase 5) | Deferred — not Phase 4 gate |

*All Phase 4 success criteria have automated verification with fake tokens.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 120s (warm)
- [ ] `nyquist_compliant: true` set in frontmatter after phase execution gate green
- [ ] `wave_0_complete: true` after Plan 01 harness lands

**Approval:** pending
