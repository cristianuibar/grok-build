---
phase: 5
slug: codex-oauth-dual-auth-lifecycle
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-07-16
updated: 2026-07-16
---

# Phase 5 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Wave 0 uses **integration tests on public APIs** — do **not** repair the broken shell `--lib` suite.
> Prove AUTH-02..05 with **fake tokens + mock IdP only** — no live ChatGPT / Codex OAuth required for CI gates.
>
> **AUTH-05 depth:** proofs must include pure `CodexRefresher` isolation **and** a production prepare-time / pre-request call site that spends the refresh token, persists via `mutate_provider_store_or_prune(PROVIDER_CODEX)`, invalidates the Phase 4 snapshot cache, and returns a fresh bearer.
>
> **Cargo verify hygiene:** one TESTNAME filter per `cargo test`; never bare `| tail` without `set -o pipefail`; chain with `&&` only.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Cargo built-in `cargo test` (crate integration tests + pager unit clap tests) |
| **Config file** | none global — per-crate; `rust-toolchain.toml` (1.92.0) |
| **Quick run command** | `cargo test -p xai-grok-shell --test auth_codex_lifecycle -- --nocapture` |
| **Full suite command** | `cargo test -p xai-grok-shell --test auth_codex_lifecycle -- --nocapture && cargo test -p xai-grok-shell --test auth_multi_slot -- --nocapture && cargo test -p xai-grok-shell --test provider_routing -- --nocapture && cargo check -p xai-grok-shell && cargo check -p xai-grok-pager-bin` |
| **Estimated runtime** | ~60–240 seconds after first compile (lifecycle + mock HTTP + regressions) |

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
| `cargo test -p xai-grok-shell --test auth_codex_lifecycle …` | `cargo test -p xai-grok-shell --lib …` for Phase 5 gates |
| `cargo test -p xai-grok-shell --test auth_multi_slot …` (AUTH-01 regression) | Fixing entire shell lib-test compile errors |
| `cargo test -p xai-grok-shell --test provider_routing …` (MOD-04/05 regression) | Live ChatGPT login as phase gate |
| `cargo test -p xai-grok-pager <clap_test_name>` | Full ACP e2e as required gate |
| `cargo check -p xai-grok-shell` / `-p xai-grok-pager-bin` | Stock `~/.codex` import as product path |
| Public pure helpers + mock IdP HTTP + prepare call-site proofs | Pure unit-only refresh without production invoker |

### CI RED policy (Wave 0)

- Plan 01 commits behavior-RED tests intentionally.
- Sequential GSD execution on phase branch: Plans 02–06 turn contracts GREEN before mainline CI that runs all integration tests.
- Do not `#[ignore]` core AUTH-02..05 contracts.

**Public API surface for Phase 5 proofs:**

- `xai_grok_shell::auth::{PROVIDER_XAI, PROVIDER_CODEX, GrokAuth, AuthDocument, AuthStore, select_provider_access_token}`
- `read_provider_auth_store` / `mutate_provider_store_or_prune` / `clear_provider_slot`
- Codex login persist / device / claims (Plan 03)
- `run_cli_login` / `run_cli_logout` / `run_cli_auth_status` (Plans 03–04)
- Pure status formatter (`auth/status.rs`)
- `CodexRefresher` + pure `codex/refresh` + **prepare-time ensure-fresh invoker**
- `snapshot_codex_session_key_from_auth_store` / `invalidate_codex_session_key_cache`
- `inject_url_derived_headers` / prepare sampling headers for `ChatGPT-Account-ID`
- Pager clap: `Command::Login` / `Logout` / `Auth { Status }`

**Wire strings:** assert `"xai"` / `"codex"` literals in CLI/status as needed — do not import private constants into integration tests when avoidable.

**Fake tokens:** `xai-fake-token`, `codex-fake-token`, `codex-refresh-token` (or similar) only.

**Deterministic home:** tempfile `BUM_HOME` / product-home override for auth.json fixtures (mirror `auth_multi_slot.rs`).

---

## Sampling Rate

- **After every task commit:** Run that task’s `<automated>` command
- **After every plan wave:** Full `auth_codex_lifecycle` binary (plus regressions after Plan 05+)
- **Before `/gsd-verify-work`:** Full suite command green
- **Max feedback latency:** ~240 seconds preferred after warm compile

---

## Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|--------------|
| AUTH-02 | Mock/inject Codex login persists only `providers.codex` under temp product home | integration | `cargo test -p xai-grok-shell --test auth_codex_lifecycle codex_login_persists_slot -- --nocapture` | ❌ Wave 0 RED |
| AUTH-02 | PKCE authorize URL contains client_id, S256, localhost, auth/callback, offline_access, originator bum | integration | `cargo test -p xai-grok-shell --test auth_codex_lifecycle codex_authorize_url_includes_pkce_and_localhost_callback -- --nocapture` | ❌ Wave 0 RED |
| AUTH-02 | Device endpoints use Codex deviceauth (not xAI device grant) | integration | `cargo test -p xai-grok-shell --test auth_codex_lifecycle codex_device_endpoints_use_deviceauth -- --nocapture` | ❌ Wave 0 RED |
| AUTH-02 | CLI `--provider codex` parses | unit (pager) | `cargo test -p xai-grok-pager bum_login_provider_codex_parses -- --nocapture` | ❌ Wave 0 scaffold |
| AUTH-03 | Logout codex leaves xAI; logout xAI leaves codex | integration | `cargo test -p xai-grok-shell --test auth_codex_lifecycle selective_logout_isolates -- --nocapture` | ❌ Wave 0 RED |
| AUTH-03 | Bare logout fail-closed (no mutation) | integration | `… bare_logout_fail_closed` | ❌ Wave 0 RED |
| AUTH-03 | `--all` clears both when both present | integration | `… logout_all_clears_both` | ❌ Wave 0 RED |
| AUTH-04 | Pure status format greppable; both providers; no token substrings | integration | `… auth_status_format_paste_safe` | ❌ Wave 0 RED |
| AUTH-04 | `run_cli_auth_status` handler path against dual fixture | integration | `… run_cli_auth_status` | ❌ Wave 0 RED |
| AUTH-04 | `bum auth status` clap parses | unit (pager) | `cargo test -p xai-grok-pager bum_auth_status_parses -- --nocapture` | ❌ Wave 0 scaffold |
| AUTH-05 | Mock refresh updates only codex slot | integration | `… codex_refresh_isolates` | ❌ Wave 0 RED |
| AUTH-05 | invalid_grant wipes/marks codex only | integration | `… codex_invalid_grant_no_xai_wipe` | ❌ Wave 0 RED |
| AUTH-05 | **Production prepare-time path** refreshes near-expiry Codex, persists, invalidates cache, returns fresh bearer | integration | `… codex_pre_request_refresh_updates_snapshot` | ❌ Wave 0 RED |
| AUTH-05 | Codex sampling injects `ChatGPT-Account-ID` when organization_id set | integration | `… chatgpt_account_id_header_on_codex` (lifecycle or provider_routing) | ❌ Wave 0 RED |
| AUTH-01 reg | Multi-slot sibling preserve | existing | `cargo test -p xai-grok-shell --test auth_multi_slot -- --nocapture` | ✅ |
| MOD-04/05 reg | Dual route + never_cross_slot | existing | `cargo test -p xai-grok-shell --test provider_routing -- --nocapture` | ✅ |

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------------|-----------------|-----------|-------------------|-------------|--------|
| 05-01-01 | 01 | 1 | AUTH-02..05 | T-05-01/02 | Wave 0: binary compiles; --list; smoke green; RED lifecycle contracts | integration scaffold | `cargo test -p xai-grok-shell --test auth_codex_lifecycle -- --list && cargo test -p xai-grok-shell --test auth_codex_lifecycle auth_codex_lifecycle_harness_smoke -- --nocapture` | ❌ | ⬜ pending |
| 05-01-02 | 01 | 1 | AUTH-02/03/04 | T-05-02 | Clap scaffolds for --provider, logout --all, auth status | unit | `cargo test -p xai-grok-pager bum_login_defaults_to_xai_without_provider_argument -- --nocapture` | ❌ | ⬜ pending |
| 05-02-01 | 02 | 2 | AUTH-03 | T-05-03 | Provider-slot mutate/clear isolation | integration | `cargo test -p xai-grok-shell --test auth_codex_lifecycle mutate_provider -- --nocapture && cargo test -p xai-grok-shell --test auth_multi_slot -- --nocapture` | ❌ | ⬜ pending |
| 05-02-02 | 02 | 2 | AUTH-04 | T-05-04 | Pure paste-safe dual status format | integration | `cargo test -p xai-grok-shell --test auth_codex_lifecycle auth_status_format_paste_safe -- --nocapture` | ❌ | ⬜ pending |
| 05-03-01 | 03 | 3 | AUTH-02 | T-05-06/09 | Authorize URL + mock login persist only codex | integration | `cargo test -p xai-grok-shell --test auth_codex_lifecycle codex_authorize_url_includes_pkce_and_localhost_callback -- --nocapture && cargo test -p xai-grok-shell --test auth_codex_lifecycle codex_login_persists_slot -- --nocapture` | ❌ | ⬜ pending |
| 05-03-02 | 03 | 3 | AUTH-02 | T-05-07/08 | Deviceauth endpoints + CLI provider dispatch | integration + unit | `cargo test -p xai-grok-pager bum_login_provider_codex_parses -- --nocapture && cargo test -p xai-grok-shell --test auth_codex_lifecycle codex_device_endpoints_use_deviceauth -- --nocapture && cargo test -p xai-grok-shell --test auth_codex_lifecycle codex_login_persists_slot -- --nocapture` | ❌ | ⬜ pending |
| 05-04-01 | 04 | 4 | AUTH-03 | T-05-11 | Selective / --all / bare fail-closed logout | integration | `cargo test -p xai-grok-shell --test auth_codex_lifecycle selective_logout_isolates -- --nocapture && cargo test -p xai-grok-shell --test auth_codex_lifecycle bare_logout_fail_closed -- --nocapture && cargo test -p xai-grok-shell --test auth_codex_lifecycle logout_all_clears_both -- --nocapture` | ❌ | ⬜ pending |
| 05-04-02 | 04 | 4 | AUTH-04 | T-05-12/14 | run_cli_auth_status handler + clap + dual-safe /logout | integration + unit | `cargo test -p xai-grok-pager bum_auth_status_parses -- --nocapture && cargo test -p xai-grok-shell --test auth_codex_lifecycle run_cli_auth_status -- --nocapture && cargo test -p xai-grok-shell --test auth_codex_lifecycle auth_status_format_paste_safe -- --nocapture` | ❌ | ⬜ pending |
| 05-05-01 | 05 | 5 | AUTH-05 | T-05-15/16 | Pure CodexRefresher isolation + invalid_grant no xAI wipe | integration | `cargo test -p xai-grok-shell --test auth_codex_lifecycle codex_refresh_isolates -- --nocapture && cargo test -p xai-grok-shell --test auth_codex_lifecycle codex_invalid_grant_no_xai_wipe -- --nocapture` | ❌ | ⬜ pending |
| 05-05-02 | 05 | 5 | AUTH-05 | T-05-15/16 | **Production prepare-time invoker** → CodexRefresher → persist PROVIDER_CODEX → fresh snapshot | integration | `cargo test -p xai-grok-shell --test auth_codex_lifecycle codex_pre_request_refresh_updates_snapshot -- --nocapture` | ❌ | ⬜ pending |
| 05-05-03 | 05 | 5 | AUTH-05 | T-05-17/18 | Cache invalidate on mutates + ChatGPT-Account-ID header inject | integration | `cargo test -p xai-grok-shell --test auth_codex_lifecycle chatgpt_account_id_header_on_codex -- --nocapture && cargo test -p xai-grok-shell --test provider_routing no_proxy_headers_on_codex -- --nocapture` | ❌ | ⬜ pending |
| 05-06-01 | 06 | 6 | AUTH-02..05 | T-05-19 | Full lifecycle suite green | integration | `cargo test -p xai-grok-shell --test auth_codex_lifecycle -- --nocapture` | ❌ | ⬜ pending |
| 05-06-02 | 06 | 6 | AUTH-01 + MOD-04/05 | T-05-20 | Regressions + check + deferred-scope audit | integration + check | Full suite command | ❌ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `crates/codegen/xai-grok-shell/tests/auth_codex_lifecycle.rs` — AUTH-02..05 integration harness (smoke + behavior-RED contracts)
- [ ] Named RED (or scaffold) contracts include at minimum:
  - `auth_codex_lifecycle_harness_smoke` (GREEN)
  - `codex_login_persists_slot`
  - `codex_authorize_url_includes_pkce_and_localhost_callback`
  - `codex_device_endpoints_use_deviceauth`
  - `selective_logout_isolates` / `logout_all_clears_both` / `bare_logout_fail_closed`
  - `auth_status_format_paste_safe` / `run_cli_auth_status`
  - `codex_refresh_isolates` / `codex_invalid_grant_no_xai_wipe`
  - `codex_pre_request_refresh_updates_snapshot` (production prepare path)
  - `chatgpt_account_id_header_on_codex`
- [ ] Clap scaffolds in `crates/codegen/xai-grok-pager/src/app/cli.rs` for `--provider`, logout `--all`, `auth status`
- [ ] Framework install: none
- [ ] No shell `--lib` gate

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Live `bum login --provider codex` browser smoke | AUTH-02 (optional) | Needs real ChatGPT + free ports 1455/1457 | Optional: temp `BUM_HOME`; login; `bum auth status`; selective logout; confirm no tokens printed |
| Device-code live path | AUTH-02 (optional) | Workspace security + browser | Optional: `bum login --provider codex --device-auth` |

*All Phase 5 success criteria have automated verification with fake tokens / mock IdP (including prepare-time refresh and ChatGPT-Account-ID).*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references (including prepare-time refresh + account header + deviceauth + authorize URL + status handler)
- [ ] AUTH-05 production invoker proven (`codex_pre_request_refresh_updates_snapshot`) — not pure unit only
- [ ] No watch-mode flags
- [ ] Feedback latency < 240s (warm)
- [ ] Cargo hygiene: one TESTNAME; never shell `--lib` as phase gate
- [ ] `nyquist_compliant: true` set in frontmatter after phase execution gate green
- [ ] `wave_0_complete: true` after Plan 01 harness lands

**Approval:** pending (revision from 05-PLAN-CHECK)
