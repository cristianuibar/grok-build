---
phase: 05-codex-oauth-dual-auth-lifecycle
fixed_at: 2026-07-16T20:50:00Z
review_path: .planning/phases/05-codex-oauth-dual-auth-lifecycle/05-REVIEW.md
iteration: 1
findings_in_scope: 5
fixed: 5
skipped: 0
status: all_fixed
---

# Phase 5: Code Review Fix Report

**Fixed at:** 2026-07-16T20:50:00Z  
**Source review:** `05-REVIEW.md`  
**Iteration:** 1  

**Summary:**
- Findings in scope: CR-01, CR-02, WR-01, WR-02, WR-03 (+ logout snapshot invalidate)
- Fixed: 5
- Skipped: 0 (Info left optional)

## Fixed Issues

### CR-01: Unbounded blocking flock on async ensure_fresh

**Files:** `auth/codex/ensure_fresh.rs`  
**Commit:** `64de2ec`  
**Fix:** Use `try_lock_auth_file_async(..., AUTH_LOCK_TIMEOUT)` instead of `lock_auth_file_blocking`. Timeout / not-live → `EnsureFreshCodexResult::Unavailable` (no hang, no wipe).

### CR-02: Reconstruct wipes prepared bearer on any None

**Files:** `ensure_fresh.rs`, `sampler_turn.rs`  
**Commit:** `64de2ec`  
**Fix:** Ternary `EnsureFreshCodexResult::{Fresh, Unusable, Unavailable}`. Reconstruct clears `api_key` only on `Unusable`; keeps prepared SessionToken on `Unavailable`.

### WR-01: Production-public test hooks

**Files:** `ensure_fresh.rs`, `mod.rs`, `agent/config.rs`  
**Commit:** `64de2ec`  
**Fix:** Synthetic IdP / hook setters gated `cfg(any(test, feature = "unstable", debug_assertions))`.

### WR-02: Persist fail after RT rotation still returns material

**Files:** `ensure_fresh.rs`  
**Commit:** `64de2ec`  
**Fix:** On persist failure after rotation, return `Unavailable` (do not advertise unpersisted RT).

### WR-03: Browser “Signed in” before token exchange

**Files:** `auth/codex/browser.rs`  
**Commit:** `64de2ec`  
**Fix:** Callback HTML title/message: “Authorization received” / finish in terminal (not “Signed in”).

### Bonus: Logout snapshot invalidate

**Files:** `auth/flow.rs`  
**Commit:** `64de2ec`  
**Fix:** Selective Codex logout and `--all` call `invalidate_codex_session_key_snapshot()`.

## Re-verification

| Gate | Result |
|------|--------|
| `auth_codex_lifecycle` (31) | PASS |
| `--lib` Option C reconstruct trio | PASS |
| `auth_multi_slot` | PASS |
| `provider_routing` (50) | PASS |
| `cargo check -p xai-grok-pager-bin` | PASS |
