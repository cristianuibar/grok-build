---
phase: 5
slug: codex-oauth-dual-auth-lifecycle
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-07-16
updated: 2026-07-16
replan_source: 05-REVIEWS.md cycle 3
---

# Phase 5 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Wave 0 uses **integration tests on public APIs** — do **not** repair the broken full shell `--lib` suite.
> Prove AUTH-02..05 with **fake tokens + mock IdP only** — no live ChatGPT / Codex OAuth required for CI gates.
>
> **AUTH-05 depth (review cycle 1 + cycle 2 residual + cycle 3):**
> - Pure data-only `CodexRefresher` / identity preserve (Plan 05 Task 1) — **not** outer ensure_fresh persist/isolation
> - Lock-held `ensure_fresh_codex_auth` + dual-slot isolation / invalid_grant (Plan 05 Task 2 only)
> - Production invoker on **`SessionActor::reconstruct_full_config`** (sampler_turn.rs)
> - **Executable seam (cycle 2 Option C):** crate-local actor tests in `session/acp_session_tests/codex_reconstruct_refresh_tests.rs` call `actor.reconstruct_full_config().await` (pattern: `auth_error_no_retry_tests.rs`) and assert `SamplingConfig.api_key` / headers — **not** public `ensure_fresh` alone
> - Reconstruct filters always: `cargo test -p xai-grok-shell --lib <TESTNAME>` (cycle 3: never omit `--lib`; Wave 0 must not define same-named integration RED stubs)
> - OAuth bearer override only for `AuthType::SessionToken` + `session_oauth_allowed`; BYOK/custom endpoint tests required
> - Permanent fail clear via `clear_provider_slot_with_lock` (no lock reacquire)
> - Identity preserve when refresh response omits RT/claims
>
> **Cargo verify hygiene:** one TESTNAME filter per `cargo test`; never bare `| tail` without `set -o pipefail`; chain with `&&` only.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Cargo built-in `cargo test` (crate integration tests + pager unit clap tests) |
| **Config file** | none global — per-crate; `rust-toolchain.toml` (1.92.0) |
| **Quick run command** | `cargo test -p xai-grok-shell --test auth_codex_lifecycle -- --nocapture` |
| **Full suite command** | `cargo test -p xai-grok-shell --test auth_codex_lifecycle -- --nocapture && cargo test -p xai-grok-shell --lib codex_reconstruct_refreshes_mid_session_expiry -- --nocapture && cargo test -p xai-grok-shell --lib codex_byok_key_not_overridden -- --nocapture && cargo test -p xai-grok-shell --lib codex_oauth_bearer_absent_on_custom_endpoint -- --nocapture && cargo test -p xai-grok-shell --test auth_multi_slot -- --nocapture && cargo test -p xai-grok-shell --test provider_routing -- --nocapture && cargo test -p xai-grok-pager bum_login_provider_codex_parses -- --nocapture && cargo test -p xai-grok-pager bum_auth_status_parses -- --nocapture && cargo check -p xai-grok-shell && cargo check -p xai-grok-pager-bin && cargo fmt --all --check` |
| **Estimated runtime** | ~60–240 seconds after first compile (lifecycle + mock HTTP + reconstruct seam + regressions) |

### Cargo verify hygiene (locked)

| Rule | Detail |
|------|--------|
| One TESTNAME filter | Cargo accepts **one** positional TESTNAME filter per invocation |
| Multi-test coverage | Run full binary without filter **or** chain single-filter invocations with `&&` |
| Exit status | Never pipe cargo through bare `\| tail` without `set -o pipefail`. Prefer **no pipe** |
| Chains | Use `&&` only — never `;` that masks failures |
| Forbidden gates | Unfiltered `cargo test -p xai-grok-shell --lib`; crate-wide reconstruct filters **without** `--lib` |
| Option C exception | Explicit `cargo test -p xai-grok-shell --lib <TESTNAME>` only (`codex_reconstruct_refreshes_mid_session_expiry`, `codex_byok_key_not_overridden`, `codex_oauth_bearer_absent_on_custom_endpoint`) |
| Name isolation | Wave 0 integration binary must **not** define the three Option C reconstruct TESTNAMEs (cycle 3 collision) |

### AUTH-05 executable seam (cycle 2 — Option C locked)

| Item | Value |
|------|-------|
| **Seam name** | `SessionActor::reconstruct_full_config` |
| **Test location** | `session/acp_session_tests/codex_reconstruct_refresh_tests.rs` |
| **Pattern** | `auth_error_no_retry_tests.rs` — build actor, `reconstruct_full_config().await`, assert api_key/headers |
| **Required names** | `codex_reconstruct_refreshes_mid_session_expiry`, `codex_byok_key_not_overridden`, `codex_oauth_bearer_absent_on_custom_endpoint` |
| **Run command** | `cargo test -p xai-grok-shell --lib <TESTNAME> -- --nocapture` (one name per invocation) |
| **Not sufficient** | Public `ensure_fresh_codex_auth` isolation alone; crate-wide filter without `--lib` |
| **Wave 0** | Do not create same-named RED stubs in `tests/auth_codex_lifecycle.rs` |

### Harness policy

| Allowed | Forbidden |
|---------|-----------|
| `cargo test -p xai-grok-shell --test auth_codex_lifecycle …` | Unfiltered `cargo test -p xai-grok-shell --lib` for Phase 5 gates |
| `cargo test -p xai-grok-shell --lib <reconstruct TESTNAME>` (Option C) | Crate-wide reconstruct filter without `--lib`; same-named Wave 0 RED stubs |
| Explicit `--lib` reconstruct filters only | Treating ensure_fresh-only as AUTH-05 production proof |
| `cargo test -p xai-grok-shell --test auth_multi_slot …` (AUTH-01 regression) | Fixing entire shell lib-test compile errors |
| `cargo test -p xai-grok-shell --test provider_routing …` (MOD-04/05 regression) | Live ChatGPT login as phase gate |
| `cargo test -p xai-grok-pager <clap_test_name>` | Full ACP e2e as required gate |
| `cargo check -p xai-grok-shell` / `-p xai-grok-pager-bin` | Stock `~/.codex` import as product path |
| Public path-taking helpers + mock IdP HTTP + **reconstruct seam** proofs | Pure unit-only refresh without reconstruct invoker |
| Explicit auth_file paths (OnceLock-safe) | Multi-BUM_HOME mutation in one process |

### CI RED policy (Wave 0)

- Plan 01 commits behavior-RED tests intentionally.
- Sequential GSD execution on phase branch: Plans 02–06 turn contracts GREEN before mainline CI that runs all integration tests.
- Do not `#[ignore]` core AUTH-02..05 contracts.

**Public API surface for Phase 5 proofs:**

- `xai_grok_shell::auth::{PROVIDER_XAI, PROVIDER_CODEX, AuthProvider, GrokAuth, select_provider_access_token}`
- **Public** `mutate_provider_store_or_prune` / `clear_provider_slot` / `clear_all_provider_slots` (path + AuthProvider)
- Codex login persist / device / claims (Plan 03)
- `run_cli_login` / `run_cli_logout` / `run_cli_auth_status` (Plans 03–04; status returns String or Write)
- Pure status formatter (`auth/status.rs`) with logged_in vs usable
- `CodexRefresher` (data-only) + pure `codex/refresh` + **`ensure_fresh_codex_auth` → SessionActor::reconstruct_full_config`**
- `clear_provider_slot_with_lock` (permanent fail while lock held)
- `CodexAuthMaterial { bearer, account_id }`
- Reconstruct OAuth override gates: `AuthType::SessionToken` + `session_oauth_allowed`
- Trusted-host ChatGPT-Account-ID inject (not arbitrary hosts) — same trust rules as bearer
- Pager clap: `Command::Login` / `Logout` / `Auth { Status }`

**Wire strings:** assert `"xai"` / `"codex"` literals in CLI/status as needed.

**Fake tokens:** `xai-fake-token`, `codex-fake-token`, `codex-refresh-token` (or similar) only.

**Deterministic home:** Prefer explicit auth_file paths under tempfile. Do not mutate BUM_HOME across tests in one process (OnceLock). Handler tests that require grok_home use single sandbox or subprocess.

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
| AUTH-02 | Mock/inject Codex login persists only `providers.codex` | integration | `… codex_login_persists_slot` | ❌ Wave 0 RED |
| AUTH-02 | PKCE authorize URL contract | integration | `… codex_authorize_url_includes_pkce_and_localhost_callback` | ❌ Wave 0 RED |
| AUTH-02 | Device endpoints deviceauth | integration | `… codex_device_endpoints_use_deviceauth` | ❌ Wave 0 RED |
| AUTH-02 | Device pending/slowdown/exchange/denied | integration | `… codex_device_pending_slowdown_exchange_denied` | ❌ Wave 0 RED |
| AUTH-02 | State mismatch writes nothing | integration | `… codex_oauth_state_mismatch_writes_nothing` | ❌ Wave 0 RED |
| AUTH-02 | CLI `--provider codex` parses | unit (pager) | `cargo test -p xai-grok-pager bum_login_provider_codex_parses` | ❌ Wave 0 scaffold |
| AUTH-03 | Logout codex leaves xAI; logout xAI leaves codex | integration | `… selective_logout_isolates` | ❌ Wave 0 RED |
| AUTH-03 | Bare logout fail-closed | integration | `… bare_logout_fail_closed` | ❌ Wave 0 RED |
| AUTH-03 | `--all` clears both atomically | integration | `… logout_all_clears_both` | ❌ Wave 0 RED |
| AUTH-04 | Pure status format greppable; no tokens | integration | `… auth_status_format_paste_safe` | ❌ Wave 0 RED |
| AUTH-04 | usable expired+refreshable vs expired+no-refresh | integration | `… auth_status_usable` | ❌ Wave 0 RED |
| AUTH-04 | `run_cli_auth_status` handler path | integration | `… run_cli_auth_status` | ❌ Wave 0 RED |
| AUTH-04 | `bum auth status` clap parses | unit (pager) | `cargo test -p xai-grok-pager bum_auth_status_parses` | ❌ Wave 0 scaffold |
| AUTH-05 | Mock refresh updates only codex (outer ensure_fresh) | integration | `… codex_refresh_isolates` | ❌ Wave 0 RED → Plan 05 Task 2 |
| AUTH-05 | invalid_grant codex only (outer ensure_fresh) | integration | `… codex_invalid_grant_no_xai_wipe` | ❌ Wave 0 RED → Plan 05 Task 2 |
| AUTH-05 | Identity preserve when response omits RT (data-only) | integration | `… codex_refresh_preserves_identity` | ❌ Wave 0 RED → Plan 05 Task 1 |
| AUTH-05 | **reconstruct mid-session** (Option C seam) | crate-local `--lib` | `cargo test -p xai-grok-shell --lib codex_reconstruct_refreshes_mid_session_expiry` | ❌ Plan 05 Task 2 only (no Wave 0 stub) |
| AUTH-05 | BYOK key not overridden by OAuth | crate-local `--lib` | `cargo test -p xai-grok-shell --lib codex_byok_key_not_overridden` | ❌ Plan 05 Task 2 only (no Wave 0 stub) |
| AUTH-05 | OAuth bearer absent on custom endpoint | crate-local `--lib` | `cargo test -p xai-grok-shell --lib codex_oauth_bearer_absent_on_custom_endpoint` | ❌ Plan 05 Task 2 only (no Wave 0 stub) |
| AUTH-05 | Concurrent refresh single IdP spend | integration | `… codex_concurrent_refresh_single_idp_spend` | ❌ Wave 0 RED |
| AUTH-05 | Fresh token skips IdP | integration | `… codex_fresh_token_skips_idp` | ❌ Wave 0 RED |
| AUTH-05 | Transient hard-unexpired keeps token | integration | `… codex_transient_fail_hard_unexpired_keeps_token` | ❌ Wave 0 RED |
| AUTH-05 | Transient hard-expired no credential | integration | `… codex_transient_fail_hard_expired_no_credential` | ❌ Wave 0 RED |
| AUTH-05 | ChatGPT-Account-ID on trusted Codex | integration | `… chatgpt_account_id_header_on_codex` | ❌ Wave 0 RED |
| AUTH-05 | Account header absent xAI / custom endpoint | integration | `… chatgpt_account_id_header_absent` | ❌ Wave 0 RED |
| AUTH-01 reg | Multi-slot sibling preserve | existing | `cargo test -p xai-grok-shell --test auth_multi_slot` | ✅ |
| MOD-04/05 reg | Dual route + never_cross_slot | existing | `cargo test -p xai-grok-shell --test provider_routing` | ✅ |

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------------|-----------------|-----------|-------------------|-------------|--------|
| 05-01-01 | 01 | 1 | AUTH-02..05 | T-05-01/02 | Wave 0 list + smoke + RED contracts incl. reconstruct AUTH-05 | integration scaffold | `--list` + smoke | ❌ | ⬜ pending |
| 05-01-02 | 01 | 1 | AUTH-02/03/04 | T-05-02 | Clap scaffolds + pager-bin check; verify new parser names | unit + check | multi clap filters + check pager-bin | ❌ | ⬜ pending |
| 05-02-01 | 02 | 2 | AUTH-03 | T-05-04/26 | Public AuthProvider mutate/clear + with_lock clear | integration | mutate_provider + clear_provider + multi_slot | ❌ | ⬜ pending |
| 05-02-02 | 02 | 2 | AUTH-04 | T-05-03 | Paste-safe status + usable semantics | integration | auth_status_format_paste_safe + auth_status_usable | ❌ | ⬜ pending |
| 05-03-01 | 03 | 3 | AUTH-02 | T-05-06/09 | Authorize + login persist + state mismatch | integration | authorize + login + state_mismatch | ❌ | ⬜ pending |
| 05-03-02 | 03 | 3 | AUTH-02 | T-05-07/22 | Device multi-step + CLI provider; no xAI post_login_sync | integration + unit | clap provider + deviceauth + device multi-step + login | ❌ | ⬜ pending |
| 05-04-01 | 04 | 4 | AUTH-03 | T-05-11/23 | Blocking clear; selective / all / bare | integration | selective + bare + logout_all | ❌ | ⬜ pending |
| 05-04-02 | 04 | 4 | AUTH-04 | T-05-12/14 | status handler + dual-safe TUI/ACP | integration + unit | clap status + run_cli_auth_status + format | ❌ | ⬜ pending |
| 05-05-01 | 05 | 5 | AUTH-05 | T-05-24 | Pure data-only CodexRefresher + identity preserve (no ensure_fresh persist) | integration | `codex_refresh_preserves_identity` only | ❌ | ⬜ pending |
| 05-05-02 | 05 | 5 | AUTH-05 | T-05-15/16/21/25/26 | **Option C `--lib` reconstruct** + isolation/invalid_grant + BYOK/custom + lock + concurrent + transient | actor `--lib` + integration | `--lib` reconstruct trio + refresh_isolates + invalid_grant + concurrent + fresh_skip + transient | ❌ | ⬜ pending |
| 05-05-03 | 05 | 5 | AUTH-05 | T-05-18 | Trusted-host account header + absences | integration | chatgpt_account_id_header* + no_proxy regression | ❌ | ⬜ pending |
| 05-06-01 | 06 | 6 | AUTH-02..05 | T-05-19/21/25 | Full lifecycle + Option C seam green | integration + actor | full lifecycle + reconstruct seam filters | ❌ | ⬜ pending |
| 05-06-02 | 06 | 6 | AUTH-01 + MOD-04/05 | T-05-20 | Regressions + clap + check + fmt + deferred audit | integration + check | Full suite command | ❌ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `crates/codegen/xai-grok-shell/tests/auth_codex_lifecycle.rs` — AUTH-02..05 integration harness
- [ ] Named RED (or scaffold) contracts include at minimum:
  - `auth_codex_lifecycle_harness_smoke` (GREEN)
  - `codex_login_persists_slot`
  - `codex_authorize_url_includes_pkce_and_localhost_callback`
  - `codex_device_endpoints_use_deviceauth`
  - `codex_device_pending_slowdown_exchange_denied`
  - `codex_oauth_state_mismatch_writes_nothing`
  - `selective_logout_isolates` / `logout_all_clears_both` / `bare_logout_fail_closed`
  - `auth_status_format_paste_safe` / `auth_status_usable_*` / `run_cli_auth_status`
  - `codex_refresh_isolates` / `codex_invalid_grant_no_xai_wipe` (integration; Plan 05 Task 2)
  - `codex_refresh_preserves_identity_when_response_omits_refresh_token` (integration; Plan 05 Task 1 data-only)
  - `codex_concurrent_refresh_single_idp_spend`
  - `codex_fresh_token_skips_idp`
  - `codex_transient_fail_hard_unexpired_keeps_token` / `codex_transient_fail_hard_expired_no_credential`
  - `chatgpt_account_id_header_on_codex` / `chatgpt_account_id_header_absent_*`
- [ ] Option C reconstruct trio (**not** in Wave 0 integration binary) land only under Plan 05 crate-local module and run via `--lib`:
  - `codex_reconstruct_refreshes_mid_session_expiry`
  - `codex_byok_key_not_overridden`
  - `codex_oauth_bearer_absent_on_custom_endpoint`
- [ ] Clap scaffolds in pager cli + pager-bin compile for `--provider`, logout `--all`, `auth status`
- [ ] Framework install: none
- [ ] No unfiltered shell `--lib` gate; Option C only via `cargo test -p xai-grok-shell --lib <TESTNAME>`
- [ ] OnceLock policy documented in harness

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Live `bum login --provider codex` browser smoke | AUTH-02 (optional) | Needs real ChatGPT + free ports 1455/1457 | Optional: temp `BUM_HOME`; login; `bum auth status`; selective logout; confirm no tokens printed |
| Device-code live path | AUTH-02 (optional) | Workspace security + browser | Optional: `bum login --provider codex --device-auth` |

*All Phase 5 success criteria have automated verification with fake tokens / mock IdP (including reconstruct-time refresh and trusted ChatGPT-Account-ID).*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references (including reconstruct seam names + BYOK/custom + identity + concurrent + device multi-step + usable + status handler)
- [ ] AUTH-05 production invoker proven on **Option C SessionActor::reconstruct_full_config seam** via explicit `--lib` TESTNAME — not pure unit, prepare-only, ensure_fresh-only, or crate-wide filter without `--lib`
- [ ] Plan 05 Task 1 verifies pure CodexRefresher/identity only; isolation + invalid_grant live only on Task 2
- [ ] No watch-mode flags
- [ ] Feedback latency < 240s (warm)
- [ ] Cargo hygiene: one TESTNAME; never unfiltered shell `--lib` as phase gate; reconstruct always `--lib <TESTNAME>`
- [ ] `nyquist_compliant: true` set in frontmatter after phase execution gate green
- [ ] `wave_0_complete: true` after Plan 01 harness lands

**Approval:** pending (replan from 05-REVIEWS cycle 3 MEDIUMs)
