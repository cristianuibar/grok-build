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
> Seeded from `02-RESEARCH.md` ## Validation Architecture; updated after review incorporation.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Cargo built-in + `tokio::test` (workspace) |
| **Config file** | Per-crate; no jest/vitest |
| **Quick run command** | `cargo test -p xai-grok-shell --lib auth::` |
| **Credential filters** | `auth::credential_provider::` and `util::grok_auth_credentials::` |
| **Full suite command** | `cargo test -p xai-grok-shell --lib` |
| **Estimated runtime** | ~30–90 seconds (auth lib focus) |

Prefer lib unit tests under `auth/storage.rs` and `auth/manager_tests.rs`; avoid full-workspace and ignored binary e2e for the AUTH-01 gate.

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p xai-grok-shell --lib auth::`
- **After every plan wave:** Run `cargo test -p xai-grok-shell --lib` (or auth + credential filters)
- **Before `/gsd-verify-work`:** Full phase gate (below) must be green + multi-slot isolation tests green
- **Max feedback latency:** ~90 seconds

---

## Per-Task Verification Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| AUTH-01 | Legacy flat `auth.json` under temp home reads as xAI slot | unit | `cargo test -p xai-grok-shell --lib auth::storage::` | ❌ Wave 0 |
| AUTH-01 | Write xAI credential emits nested `providers.xai` | unit | same | ❌ Wave 0 |
| AUTH-01 | Seed `providers.codex` fixture → xAI update does not clobber codex | unit | same | ❌ Wave 0 |
| AUTH-01 | Lock-scoped concurrent xAI + codex mutations preserve both | unit | same | ❌ Wave 0 |
| AUTH-01 | Unsupported schema version fails closed (not corrupt-empty recovery) | unit | same | ❌ Wave 0 |
| AUTH-01 | `store_api_key` / `clear_api_key` locked multi-slot mutation | unit | same | ❌ Wave 0 |
| AUTH-01 | Empty xAI after logout / clear does not delete file if codex present | unit | storage/manager | ❌ Wave 0 |
| AUTH-01 | `try_devbox_recovery` clears xAI only; codex survives | unit | storage/manager | ❌ Wave 0 |
| AUTH-01 | `FileDeleted` only after successful `remove_file` | unit | storage/manager | ❌ Wave 0 |
| AUTH-01 | `AuthManager::update` + `get_valid_token` / `auth()` from multi-slot file | unit | `auth::manager` | ⚠️ extend |
| AUTH-01 | `ShellAuthCredentialProvider` Bearer from multi-slot manager | unit | `auth::credential_provider::` | ⚠️ extend |
| AUTH-01 | Agent-turn seam: nested → AuthManager → `sampling_config.api_key` → Bearer | unit | credential/manager tests | ❌ Wave 0 |
| AUTH-01 | `GROK_AUTH_PATH` outside product home cannot receive writes | unit | `auth::manager` | ❌ Wave 0 |
| AUTH-01 | Browser + device-code mock login multi-slot on-disk + codex survival | unit | `oidc/login` + `device_code` | ⚠️ extend |
| AUTH-01 | Path stays under temp product home (never stock `~/.grok` writes) | unit | manager + login tests | ✅ pattern exists |

---

## Wave 0 Gaps

- [ ] `auth/storage.rs` tests: migrate flat → nested; lock-scoped merge-safe write; concurrency; version fail-closed; API key under xAI; FileDeleted honesty
- [ ] `auth/manager_tests.rs`: AuthManager load/update against nested document; logout/delete + try_devbox with sibling provider; GROK_AUTH_PATH write isolation
- [ ] Agent-turn seam: ShellAuthCredentialProvider + sampling_config.api_key → Bearer
- [ ] Mock OIDC + device-code: nested schema + codex survival asserts
- [ ] Optional helper: `write_fixture_auth_document(path, xai, codex)` for tests
- [ ] Audit test helpers that bypass storage (`agent/relay.rs` test `write_test_auth_to_disk`) — still accept flat via migration **or** switch to storage API

---

## Phase Gate

```bash
cargo test -p xai-grok-shell --lib auth:: -- --nocapture
cargo test -p xai-grok-shell --lib auth::credential_provider:: -- --nocapture
cargo test -p xai-grok-shell --lib util::grok_auth_credentials:: -- --nocapture
```

All three green + multi-slot isolation tests green before phase verification seal.
