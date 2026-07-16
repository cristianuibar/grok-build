---
phase: 2
slug: multi-slot-credentials-xai-oauth
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-07-16
---

# Phase 2 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Seeded from `02-RESEARCH.md` ## Validation Architecture.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Cargo built-in + `tokio::test` (workspace) |
| **Config file** | Per-crate; no jest/vitest |
| **Quick run command** | `cargo test -p xai-grok-shell --lib auth::` |
| **Full suite command** | `cargo test -p xai-grok-shell --lib` |
| **Estimated runtime** | ~30–90 seconds (auth lib focus) |

Prefer lib unit tests under `auth/storage.rs` and `auth/manager_tests.rs`; avoid full-workspace and ignored binary e2e for the AUTH-01 gate.

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p xai-grok-shell --lib auth::`
- **After every plan wave:** Run `cargo test -p xai-grok-shell --lib`
- **Before `/gsd-verify-work`:** Full auth lib suite must be green + multi-slot isolation tests green
- **Max feedback latency:** ~90 seconds

---

## Per-Task Verification Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| AUTH-01 | Legacy flat `auth.json` under temp home reads as xAI slot | unit | `cargo test -p xai-grok-shell --lib auth::storage::` (new) | ❌ Wave 0 |
| AUTH-01 | Write xAI credential emits nested `providers.xai` | unit | same | ❌ Wave 0 |
| AUTH-01 | Seed `providers.codex` fixture → xAI update does not clobber codex | unit | same | ❌ Wave 0 |
| AUTH-01 | `store_api_key` / `read_api_key` / `clear_api_key` only touch xAI slot | unit | same | ❌ Wave 0 |
| AUTH-01 | `AuthManager::update` + `get_valid_token` / `auth()` returns xAI key from multi-slot file | unit | `cargo test -p xai-grok-shell --lib auth::manager::` | ⚠️ extend existing |
| AUTH-01 | Credential provider applies Bearer from manager after multi-slot load | unit | `cargo test -p xai-grok-shell --lib` filter credential/auth | ⚠️ re-verify |
| AUTH-01 | Empty xAI after logout does not delete file if codex scopes present | unit | new storage/manager test | ❌ Wave 0 |
| AUTH-01 | Path stays under temp `BUM_HOME` / passed `grok_home` (never `~/.grok`) | unit | `AuthManager::new(temp, …)` | ✅ pattern exists |
| AUTH-01 | Browser + device-code paths still compile and unit-covered (no live IdP) | unit | existing `flow` / `device_code` tests | ✅ extend if breakage |

---

## Wave 0 Gaps

- [ ] `auth/storage.rs` tests: migrate flat → nested; merge-safe write; codex clobber resistance; API key under xAI
- [ ] `auth/manager_tests.rs` (or sibling): AuthManager load/update against nested document; logout/delete semantics with sibling provider
- [ ] Optional helper: `write_fixture_auth_document(path, xai, codex)` for tests (prefer over raw `fs::write`)
- [ ] Audit test helpers that bypass storage (`agent/relay.rs` test `write_test_auth_to_disk`) — still accept flat via migration **or** switch to storage API

---

## Phase Gate

Auth lib tests green + multi-slot isolation tests green before phase verification seal.
