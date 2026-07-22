---
phase: 05-codex-oauth-dual-auth-lifecycle
verified: 2026-07-16T20:50:00Z
status: passed
score: 4/4 must-haves verified
behavior_unverified: 0
overrides_applied: 0
human_verification: []
optional_live_smoke_deferred: true
optional_live_smoke: "Optional (non-gate): BUM_HOME temp dir; bum login --provider codex; bum auth status; bum logout --provider codex; confirm xAI untouched if dual-logged"
post_review_fix: "CR-01/CR-02 + WR hooks/HTML/snapshot invalidate; auth_codex_lifecycle 31/31; Option C lib trio green; multi_slot + provider_routing green"
---

# Phase 5: Codex OAuth & dual auth lifecycle — Verification Report

**Phase Goal:** User can log into both xAI and ChatGPT/Codex, manage them independently, and keep both sessions healthy  
**Verified:** 2026-07-16T20:50:00Z  
**Status:** passed  
**Re-verification:** Yes — after code-review fixes (CR-01 timed lock, CR-02 Unavailable reconstruct, test-hook cfg, browser HTML, logout snapshot invalidate). Automated gates re-run green.  
**Mode:** mvp (roadmap); goal text is outcome-style (not `As a…, I want…, so that…` user-story form). User Flow Coverage mapped from roadmap success criteria / outcome.  
**Optional live ChatGPT smoke:** deferred (non-gate; CI mocks cover AUTH-02..05).

## Goal Achievement

### User Flow Coverage

User story / outcome: *User can log into both xAI and ChatGPT/Codex, manage them independently, and keep both sessions healthy.*

| Step | Expected | Evidence | Status |
|------|----------|----------|--------|
| Login Codex | `bum login --provider codex` runs ChatGPT OAuth (PKCE browser / device) and persists only `providers.codex` under product home | `run_cli_login_for_provider` Codex early-return (`flow.rs`); `auth/codex/browser.rs` PKCE + `persist_codex_tokens` via `mutate_provider_store_or_prune(Codex)`; `auth/codex/device.rs` deviceauth; clap `auth_cli_parse` + lifecycle `codex_login_persists_slot` / authorize / device / state-mismatch | ✓ |
| Logout one | Selective logout clears only named provider | `run_cli_logout_at_path` → `clear_provider_slot` / `logout_provider_slot`; TUI/ACP bare fail-closed; tests `selective_logout_isolates`, `bare_logout_fail_closed`, `logout_all_clears_both` | ✓ |
| Status | Per-provider logged_in / usable, no secrets | `AuthStatusReport` + `run_cli_auth_status`; tests `auth_status_format_paste_safe`, usable semantics, `run_cli_auth_status`; CLI `bum auth status` parses | ✓ |
| Independent refresh | Codex refresh/invalid_grant does not wipe xAI; mid-session reconstruct serves fresh bearer | `ensure_fresh_codex_auth` + lock-held persist/clear; wired from `SessionActor::reconstruct_full_config`; isolation + Option C reconstruct trio GREEN | ✓ |

### Observable Truths (roadmap contract)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | User can log in to ChatGPT/Codex via ChatGPT OAuth (PKCE browser + device-code where applicable); credentials only under bum auth store | ✓ VERIFIED | Codex module + CLI dispatch; persist only `AuthProvider::Codex`; no `~/.codex` import path; lifecycle: `codex_login_persists_slot`, authorize PKCE/localhost, deviceauth multi-step, state-mismatch writes nothing (31/31 lifecycle green) |
| 2 | User can log out of one provider without clearing the other | ✓ VERIFIED | Blocking disk clear per provider / `--all` atomic; bare logout fail-closed; `selective_logout_isolates` GREEN; TUI + ACP require provider or all |
| 3 | User can inspect per-provider auth status (logged in / usable) | ✓ VERIFIED | Pure `logged_in` ≠ `usable`; both slots always listed; paste-safe (no tokens); CLI handler + clap wired |
| 4 | Access tokens refresh independently without wiping/invalidating the other provider | ✓ VERIFIED | `codex_refresh_isolates`, `codex_invalid_grant_no_xai_wipe`, concurrent single IdP spend, transient/hard-expired paths GREEN; production hook `reconstruct_full_config` → `ensure_fresh_codex_auth` with SessionToken + first-party host gates; Option C mid-session / BYOK / custom-endpoint GREEN |

**Score:** 4/4 truths verified (0 present, behavior-unverified)

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `crates/codegen/xai-grok-shell/tests/auth_codex_lifecycle.rs` | AUTH-02..05 integration harness | ✓ VERIFIED | 1105 lines; 31 tests all pass |
| `crates/codegen/xai-grok-shell/src/auth/storage.rs` | Public mutate/clear/clear_all + with_lock clear | ✓ VERIFIED | `mutate_provider_store_or_prune`, `clear_provider_slot`, `clear_provider_slot_with_lock`, `clear_all_provider_slots` present and used |
| `crates/codegen/xai-grok-shell/src/auth/status.rs` | Pure paste-safe dual status | ✓ VERIFIED | 329 lines; logged_in/usable semantics; no token dump |
| `crates/codegen/xai-grok-shell/src/auth/model.rs` | `AuthProvider` + PROVIDER_* | ✓ VERIFIED | `Xai` \| `Codex`; parse rejects unknown |
| `crates/codegen/xai-grok-shell/src/auth/codex/{mod,browser,device,claims,refresh,ensure_fresh}.rs` | Codex OAuth + refresh chain | ✓ VERIFIED | PKCE 1455/1457, deviceauth, pure refresh, lock-held ensure_fresh |
| `crates/codegen/xai-grok-shell/src/auth/refresh/codex_refresher.rs` | Data-only TokenRefresher | ✓ VERIFIED | No storage mutate imports |
| `crates/codegen/xai-grok-shell/src/auth/flow.rs` | CLI login/logout/status dual-safe | ✓ VERIFIED | Codex branch before xAI post_login_sync; logout disk authority; status Write path |
| `crates/codegen/xai-grok-shell/src/session/acp_session_impl/sampler_turn.rs` | reconstruct ensure_fresh wiring | ✓ VERIFIED | Lines ~288–325: Codex + SessionToken + first-party → ensure_fresh + account header |
| `crates/codegen/xai-grok-shell/src/session/acp_session_tests/codex_reconstruct_refresh_tests.rs` | Option C seam | ✓ VERIFIED | 3 tests pass via `--lib` |
| `crates/codegen/xai-grok-pager/src/app/cli.rs` + `pager-bin/main.rs` | Clap + match arms | ✓ VERIFIED | `--provider`, logout `--all`, `auth status`; main calls shell handlers |
| `crates/codegen/xai-grok-pager/src/app/dispatch/auth.rs` | TUI logout fail-closed | ✓ VERIFIED | Bare `/logout` does not dual-wipe |
| `crates/codegen/xai-grok-shell/src/extensions/auth.rs` | ACP logout dual-safe | ✓ VERIFIED | Requires provider or `all=true` |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `Command::Login` (pager-bin) | `run_cli_login_for_provider` | clap provider → Codex early branch | ✓ WIRED | main.rs ~1739–1761 |
| Codex token success | `providers.codex` | `mutate_provider_store_or_prune(AuthProvider::Codex)` | ✓ WIRED | browser `persist_codex_tokens` |
| `Command::Logout` | `clear_provider_slot` / `clear_all` | blocking disk authority | ✓ WIRED | bare fail-closed |
| `Command::Auth status` | `run_cli_auth_status` | path-taking format | ✓ WIRED | main.rs ~1790–1797 |
| `SessionActor::reconstruct_full_config` | `ensure_fresh_codex_auth` | SessionToken + first-party Codex URL only | ✓ WIRED | sampler_turn.rs; Option C tests assert api_key + headers |
| `ensure_fresh_codex_auth` | lock + Codexpersist/clear | mutex → auth.json.lock → re-read → IdP → with_lock persist/clear | ✓ WIRED | ensure_fresh.rs; permanent path uses `clear_provider_slot_with_lock` (no reacquire) |
| Request headers | `ChatGPT-Account-ID` | trusted host + OAuth material.account_id | ✓ WIRED | inject helper + lifecycle header tests |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|--------------------|--------|
| reconstruct SamplingConfig.api_key | `material.bearer` | `ensure_fresh_codex_auth` → store/IdP | Yes (fixture + synthetic IdP in tests; production reads auth.json) | ✓ FLOWING |
| reconstruct extra_headers | `codex_account_id` | `CodexAuthMaterial.account_id` / organization_id | Yes when trusted Codex OAuth | ✓ FLOWING |
| auth status output | logged_in/usable/account | `read_auth_document` / provider store | Yes from disk fixture | ✓ FLOWING |
| logout | disk slots | `clear_provider_slot*` | Yes mutates auth.json | ✓ FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| AUTH-02..05 lifecycle suite | `cargo test -p xai-grok-shell --test auth_codex_lifecycle` | 31 passed, 0 failed | ✓ PASS |
| Option C mid-session | `cargo test -p xai-grok-shell --lib codex_reconstruct_refreshes_mid_session_expiry` | ok | ✓ PASS |
| Option C BYOK | `cargo test -p xai-grok-shell --lib codex_byok_key_not_overridden` | ok | ✓ PASS |
| Option C custom endpoint | `cargo test -p xai-grok-shell --lib codex_oauth_bearer_absent_on_custom_endpoint` | ok | ✓ PASS |
| AUTH-01 regression | `cargo test -p xai-grok-shell --test auth_multi_slot` | 1 passed | ✓ PASS |
| MOD-04/05 regression | `cargo test -p xai-grok-shell --test provider_routing` | 50 passed | ✓ PASS |
| Clap dual-auth surface | `cargo test -p xai-grok-pager --test auth_cli_parse` | 6 passed | ✓ PASS |
| Compile shell + bin | `cargo check -p xai-grok-shell -p xai-grok-pager-bin` | Finished ok | ✓ PASS |

**Note:** `cargo test -p xai-grok-pager <filter>` against `--lib` fails with **pre-existing** ~169 compile errors unrelated to Phase 5 (documented in 05-04 SUMMARY). Phase clap proofs run via integration binary `tests/auth_cli_parse.rs` — GREEN.

### Probe Execution

| Probe | Command | Result | Status |
|-------|---------|--------|--------|
| — | — | No `scripts/*/tests/probe-*.sh` for this phase | SKIP |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| AUTH-02 | 05-01, 05-03, 05-06 | ChatGPT/Codex OAuth into bum store only | ✓ SATISFIED | Login path + lifecycle login/device/authorize/state tests |
| AUTH-03 | 05-02, 05-04, 05-06 | Selective logout | ✓ SATISFIED | Storage clear + CLI/TUI/ACP + selective_logout tests |
| AUTH-04 | 05-02, 05-04, 05-06 | Per-provider status | ✓ SATISFIED | status.rs + run_cli_auth_status + format/usable tests |
| AUTH-05 | 05-05, 05-06 | Independent refresh | ✓ SATISFIED | ensure_fresh + reconstruct seam + isolation/invalid_grant/concurrent tests |

No orphaned Phase 5 requirements found outside AUTH-02..05.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| — | — | No TBD/FIXME/XXX in Phase 5 auth production paths scanned | — | — |
| pager lib tests | — | Pre-existing compile break (~169 errors) | ℹ️ Info | Not introduced by Phase 5; clap covered by `auth_cli_parse` integration binary |

**Deferred-scope audit (Plan 06):**

- No product import of stock `~/.codex` credentials (comment + no load path)
- No Platform API-key primary Codex login path
- No Phase 6 mid-session “login to Codex to use GPT” product gate shipped in this phase

### Human Verification Required

### 1. Optional live ChatGPT OAuth smoke

**Test:** With a real ChatGPT account, set `BUM_HOME` to a temp directory. Run `bum login --provider codex` (browser). Then `bum auth status`. Optionally log in xAI first, then `bum logout --provider codex`.  
**Expected:** Codex signs in; status lists both providers without secrets; Codex logout leaves xAI credentials intact.  
**Why human:** Live IdP + browser loopback cannot be exercised by CI mocks. Phase contract marks this as **optional** smoke only — automated suite is the phase gate.

### Gaps Summary

No automated gaps. All four roadmap success criteria are implemented, wired, and exercised by named tests (including AUTH-05 production reconstruct seam, not ensure_fresh-in-isolation alone).

Status is **passed**. Optional live ChatGPT smoke remains a non-gate developer check only (deferred; CI mocks cover AUTH-02..05).

---

_Verified: 2026-07-16T19:08:06Z_  
_Verifier: Claude (gsd-verifier)_
