---
phase: 05-codex-oauth-dual-auth-lifecycle
reviewed: 2026-07-16T19:15:00Z
depth: standard
files_reviewed: 22
files_reviewed_list:
  - crates/codegen/xai-grok-shell/src/auth/codex/mod.rs
  - crates/codegen/xai-grok-shell/src/auth/codex/browser.rs
  - crates/codegen/xai-grok-shell/src/auth/codex/claims.rs
  - crates/codegen/xai-grok-shell/src/auth/codex/device.rs
  - crates/codegen/xai-grok-shell/src/auth/codex/ensure_fresh.rs
  - crates/codegen/xai-grok-shell/src/auth/codex/refresh.rs
  - crates/codegen/xai-grok-shell/src/auth/refresh/codex_refresher.rs
  - crates/codegen/xai-grok-shell/src/auth/storage.rs
  - crates/codegen/xai-grok-shell/src/auth/status.rs
  - crates/codegen/xai-grok-shell/src/auth/model.rs
  - crates/codegen/xai-grok-shell/src/auth/flow.rs
  - crates/codegen/xai-grok-shell/src/auth/mod.rs
  - crates/codegen/xai-grok-shell/src/extensions/auth.rs
  - crates/codegen/xai-grok-shell/src/session/acp_session_impl/sampler_turn.rs
  - crates/codegen/xai-grok-shell/src/session/acp_session_tests/codex_reconstruct_refresh_tests.rs
  - crates/codegen/xai-grok-shell/src/agent/config.rs
  - crates/codegen/xai-grok-shell/tests/auth_codex_lifecycle.rs
  - crates/codegen/xai-grok-pager/src/app/cli.rs
  - crates/codegen/xai-grok-pager/src/app/dispatch/auth.rs
  - crates/codegen/xai-grok-pager/src/slash/commands/logout.rs
  - crates/codegen/xai-grok-pager/tests/auth_cli_parse.rs
  - crates/codegen/xai-grok-pager-bin/src/main.rs
findings:
  critical: 2
  warning: 5
  info: 3
  total: 10
status: clean
fixed_at: 2026-07-16T20:50:00Z
fix_report: 05-REVIEW-FIX.md
---

# Phase 5: Code Review Report

**Reviewed:** 2026-07-16T19:15:00Z  
**Fixed:** 2026-07-16T20:50:00Z (see `05-REVIEW-FIX.md` — CR-01, CR-02, WR-01..03 + logout snapshot)  
**Depth:** standard
**Files Reviewed:** 22
**Status:** issues_found

## Summary

Adversarial review of Phase 5 Codex OAuth + dual-auth lifecycle (AUTH-02..05): browser/device login, multi-slot logout/status, lock-held `ensure_fresh`, Option C reconstruct wiring, and CLI clap surface.

Dual-slot isolation, selective/`--all` logout fail-closed, paste-safe status, permanent-fail `clear_provider_slot_with_lock` (no reacquire), BYOK/custom-host reconstruct gates, and identity-preserving refresh merge look deliberately designed and mostly correct.

Two correctness/reliability defects must be fixed before shipping: **(1)** Codex `ensure_fresh` acquires `auth.json.lock` with unbounded **blocking** flock on the async worker (no timeout/stale recovery unlike xAI), which can hang every Codex reconstruct; **(2)** reconstruct maps *all* `ensure_fresh` `None` outcomes to `api_key = None`, including lock/IO/unavailability — wiping a still-valid prepared SessionToken. Additional security hygiene and data-loss edge cases are warnings.

## Narrative Findings (AI reviewer)

## Critical Issues

### CR-01: Unbounded blocking flock on async path for Codex ensure_fresh

**File:** `crates/codegen/xai-grok-shell/src/auth/codex/ensure_fresh.rs:194-203`
**Severity:** BLOCKER
**Issue:** Production `ensure_fresh_codex_auth_at` acquires the dual-auth file lock via `lock_auth_file_blocking` **directly on the Tokio worker**:

```194:203:crates/codegen/xai-grok-shell/src/auth/codex/ensure_fresh.rs
    let lock = match crate::auth::manager::lock::lock_auth_file_blocking(auth_file) {
        Ok(l) => l,
        Err(e) => {
            tracing::warn!(
                error_len = e.to_string().len(),
                "codex ensure_fresh: failed to acquire auth.json.lock"
            );
            return None;
        }
    };
```

`lock_auth_file_blocking` → `blocking_acquire` uses `file.lock_exclusive()` with **no timeout and no stale-holder recovery** (`manager/lock.rs:310-366`). By contrast, xAI refresh uses `try_lock_auth_file_async` (spawn_blocking + timeout + dead-PID/age break).

Consequences on the reconstruct hot path (`sampler_turn.rs` → every Codex SessionToken request):

1. If another process (or stuck holder) holds `auth.json.lock` during its own IdP exchange, this worker thread blocks in the kernel **indefinitely**.
2. No stale recovery: a dead PID / abandoned lock file can hang all Codex turns forever (xAI path recovers after timeout).
3. Saturating the runtime with blocked workers can stall unrelated async work in the same process.

This is incorrect async lock usage and a production hang risk — not a style preference.

**Fix:** Mirror xAI refresh lock acquisition:

```rust
// Prefer async timed acquire + adopt/fallback, never block the worker.
let lock = match crate::auth::manager::lock::try_lock_auth_file_async(
    auth_file,
    AUTH_LOCK_TIMEOUT, // same order as AuthManager refresh
).await {
    Some(l) => l,
    None => {
        tracing::warn!("codex ensure_fresh: auth.json.lock timeout/unavailable");
        // See CR-02: do not treat as permanent credential loss.
        return EnsureFreshOutcome::Unavailable;
    }
};
```

Hold the guard across IdP as planned, but **never** call `lock_auth_file_blocking` from an async function without `spawn_blocking` + timeout.

---

### CR-02: Reconstruct treats every `ensure_fresh` `None` as permanent auth failure

**File:** `crates/codegen/xai-grok-shell/src/session/acp_session_impl/sampler_turn.rs:298-308`
**Severity:** BLOCKER
**Issue:** On Codex SessionToken + first-party host, reconstruct does:

```298:308:crates/codegen/xai-grok-shell/src/session/acp_session_impl/sampler_turn.rs
        if codex_session_oauth {
            match crate::agent::config::ensure_fresh_codex_auth().await {
                Some(material) => {
                    api_key = Some(material.bearer);
                    codex_account_id = material.account_id;
                }
                None => {
                    // Permanent fail / hard-expired: do not serve stale prepared bearer.
                    api_key = None;
                }
            }
        }
```

But `ensure_fresh_codex_auth_at` returns `None` for **many non-permanent cases**, including:

| `ensure_fresh` path | Line (approx) | Meaning |
|---|---|---|
| lock acquire `Err` | 196-201 | I/O / inode race — credential may still be valid |
| lock not live | 205-207 | advisory fail — not invalid_grant |
| store read `Err` | 213-218 | parse/IO — disk unreadable, prepared key may still work |
| Ok(None) empty slot | 212 | truly logged out |
| hard-expired, no RT | 237-242 | truly unusable |
| permanent IdP failure | 325-340 | correctly cleared slot |
| transient + hard-expired | 347-349 | unusable now |

Collapsing these into `api_key = None` means a **transient lock/IO glitch or CR-01 contention path wipes a still-valid prepared SessionToken** and fails the turn closed as if the user were logged out / permanently rejected. That is incorrect fail-closed semantics: permanent invalidation is right; **availability failures are not**.

**Fix:** Return a ternary (or enum) from ensure_fresh and only clear prepared bearer on hard unusable / permanent:

```rust
pub enum EnsureFreshCodexResult {
    Fresh(CodexAuthMaterial),
    /// Hard-expired, missing slot, permanent IdP clear — do not serve prepared key.
    Unusable,
    /// Lock/IO/timeout — keep prepared SessionToken; do not claim permanent logout.
    Unavailable,
}

// reconstruct:
match ensure_fresh_codex_auth().await {
    EnsureFreshCodexResult::Fresh(m) => {
        api_key = Some(m.bearer);
        codex_account_id = m.account_id;
    }
    EnsureFreshCodexResult::Unusable => {
        api_key = None;
    }
    EnsureFreshCodexResult::Unavailable => {
        // Keep prepared creds.api_key; optionally omit account header if unknown.
    }
}
```

## Warnings

### WR-01: Production-public ensure_fresh test hooks can redirect token spend

**File:** `crates/codegen/xai-grok-shell/src/auth/codex/ensure_fresh.rs:88-137`
**Also:** re-exported via `auth/codex/mod.rs` and `agent/config.rs:4763-4768`
**Severity:** WARNING
**Issue:** `set_ensure_fresh_codex_test_hooks_public`, `set_ensure_fresh_codex_synthetic_*`, and token-URL override hooks are **`pub` in non-test builds** (only `set_ensure_fresh_codex_test_hooks` is `cfg(any(test, feature = "unstable"))`). Production `ensure_fresh_codex_auth()` resolves path via hooks first (`resolve_auth_file`), and synthetic IdP / token_url override short-circuit the real ChatGPT endpoint.

Any in-process caller (test residue, future plugin, malicious dependency) can:

1. Point `token_url` at an attacker host and **exfiltrate the refresh_token** on next reconstruct, or
2. Install synthetic success material and mint arbitrary bearers into request construction.

This is not remote RCE by itself, but shipping credential-path test harnesses on the public production surface is a security hygiene defect.

**Fix:** Gate all hook mutators behind `#[cfg(any(test, feature = "unstable"))]` (and keep integration tests on a `unstable` feature if needed). Production `ensure_fresh` should ignore hook state entirely in release builds:

```rust
#[cfg(any(test, feature = "unstable"))]
fn take_hooks_snapshot() -> EnsureFreshTestHooks { ... }

#[cfg(not(any(test, feature = "unstable")))]
fn take_hooks_snapshot() -> EnsureFreshTestHooks {
    EnsureFreshTestHooks::default()
}
```

---

### WR-02: Persist failure after RT rotation still returns fresh material

**File:** `crates/codegen/xai-grok-shell/src/auth/codex/ensure_fresh.rs:304-323`
**Severity:** WARNING
**Issue:** On `RefreshOutcome::Success`, if `mutate_provider_store_or_prune_with_lock` fails, the code logs and **still returns** `CodexAuthMaterial::from_auth(&new_auth)`. If the IdP rotated the refresh token, disk still holds the **invalidated** RT. Process death before a later successful write permanently logs the user out of Codex with no recovery short of re-login.

Comment acknowledges intentional tradeoff; for auth credentials this is a data-loss risk under disk full / permission errors.

**Fix:** Prefer one of:

1. Treat persist failure as `Unavailable` (do not advertise rotated tokens that are not durable), **or**
2. Retry persist once; on hard failure, return material only if `idp_rotated == false` (old RT still valid on disk).

```rust
RefreshOutcome::Success(new_auth) => {
    if let Err(e) = mutate_provider_store_or_prune_with_lock(...) {
        tracing::error!(...);
        // If RT may have rotated, fail closed to re-login rather than lie about durability.
        return None; // or Unavailable + do not use new_auth if rotated
    }
    Some(CodexAuthMaterial::from_auth(&new_auth))
}
```

---

### WR-03: Browser callback reports success before token exchange

**File:** `crates/codegen/xai-grok-shell/src/auth/codex/browser.rs:328-344` and `388-392`
**Severity:** WARNING
**Issue:** The loopback handler returns HTML **"Signed in to Codex"** as soon as `code` + state validate (`CallbackOutcome::Success`). Token exchange + persist happen **after** the browser page is sent. If exchange fails (network, IdP 4xx, empty access_token), the user has already closed the window believing success while the terminal shows failure and disk is unchanged.

**Fix:** Either:

1. Exchange (or at least validate) before completing the oneshot and render success/failure HTML based on exchange outcome (requires restructuring so the server task performs exchange), or  
2. Soften HTML to "Authorization received — finish in the terminal" until exchange completes.

---

### WR-04: Logout does not invalidate prepare-time Codex snapshot / in-session creds from other processes

**File:** `crates/codegen/xai-grok-shell/src/auth/flow.rs:1056-1113`
**Severity:** WARNING
**Issue:** `logout_provider_slot` / `logout_all_provider_slots` clear disk via blocking storage APIs but never call `invalidate_codex_session_key_snapshot()`. Same-process ACP logout clears `agent.auth_manager` for xAI only.

mtime-based snapshot re-read usually picks up a rewritten `auth.json`, and reconstruct `None` (empty slot) clears bearer — so many paths recover. Gaps remain:

- If mtime/len are unchanged (some FS/timestamp resolutions, or only in-memory managers), prepare can re-stamp a stale Codex key until epoch bump.
- Long-lived session chat-state Credentials keep the old SessionToken until the next reconstruct that observes empty store.

**Fix:** After successful codex or dual clear, call:

```rust
crate::agent::config::invalidate_codex_session_key_snapshot();
```

from `logout_provider_slot` (Codex), `logout_all_provider_slots`, and ACP `handle_logout`. Document that multi-process logout relies on disk SoT + reconstruct fail-closed (CR-02 must not wipe on Unavailable).

---

### WR-05: `lock_auth_file_blocking` during network holds dual-auth writers for up to 30s+

**File:** `crates/codegen/xai-grok-shell/src/auth/codex/ensure_fresh.rs:192-301`
**Severity:** WARNING
**Issue:** Plan-mandated lock-held RT rotation is correct for single-flight, but combined with blocking acquire (CR-01) it means **xAI login/logout/persist and Codex login** can block for the full IdP RTT (30s timeout) whenever a Codex ensure_fresh is in flight. Acceptable for correctness of RT rotation; still a robustness concern for dual-auth UX under concurrent CLI + session use.

**Fix:** After CR-01 (async timed lock), keep lock span as designed. Optionally re-read/adopt after IdP without extending unrelated critical sections beyond persist (current design is intentional — do not drop lock before IdP).

## Info

### IN-01: Bare logout prints usage twice

**File:** `crates/codegen/xai-grok-shell/src/auth/flow.rs:1227-1230`
**Issue:** `eprintln!(bare_logout_usage())` then `anyhow::bail!(bare_logout_usage())` duplicates the UI-SPEC body on stderr (bail also surfaces via main error reporting).
**Fix:** `bail` only, or eprintln + `std::process::exit(2)` without duplicating the string in the error.

### IN-02: Unsupported version parsed by scanning error Display text

**File:** `crates/codegen/xai-grok-shell/src/auth/status.rs:55-59` (and similar in `storage.rs:224-228`)
**Issue:** Version is recovered via `e.to_string().split_whitespace().find_map(|tok| tok.parse::<u32>())`. Fragile if error copy changes.
**Fix:** Structured `AuthStoreReadError::UnsupportedVersion` from `read_auth_document` without string scraping.

### IN-03: OAuth `state` compared with `!=` (non-constant-time)

**File:** `crates/codegen/xai-grok-shell/src/auth/codex/browser.rs:149-153`
**Issue:** CSRF state uses string inequality. Low practical risk for high-entropy state; constant-time compare is still preferred for auth material.
**Fix:** `subtle::ConstantTimeEq` on the UTF-8 bytes (after length check).

---

## What looks solid (not findings)

- Selective / `--all` / bare logout fail-closed (CLI + ACP + TUI toast) — no silent dual-wipe.
- Permanent fail uses `clear_provider_slot_with_lock` only (no reacquire deadlock).
- Codex login returns before xAI `post_login_sync` (no Codex principal into team managed-config).
- `merge_codex_refresh_response` preserves RT / account / email / user_id when IdP omits them.
- Reconstruct BYOK (`AuthType::ApiKey`) and custom non-allowlisted hosts skip ensure_fresh and account header.
- Status formatter tests assert token material never appears; device user_code sanitized against control chars.
- Token exchange error paths avoid logging response bodies that may contain secrets.

---

_Reviewed: 2026-07-16T19:15:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
