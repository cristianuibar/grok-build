---
phase: 02-multi-slot-credentials-xai-oauth
reviewed: 2026-07-16T07:59:19Z
depth: deep
commit: 236bea6
files_reviewed: 14
files_reviewed_list:
  - crates/codegen/xai-grok-shell/src/auth/storage.rs
  - crates/codegen/xai-grok-shell/src/auth/model.rs
  - crates/codegen/xai-grok-shell/src/auth/manager.rs
  - crates/codegen/xai-grok-shell/src/auth/manager/enrichment.rs
  - crates/codegen/xai-grok-shell/src/auth/manager/lock.rs
  - crates/codegen/xai-grok-shell/src/auth/manager_tests.rs
  - crates/codegen/xai-grok-shell/src/auth/credential_provider.rs
  - crates/codegen/xai-grok-shell/src/auth/device_code.rs
  - crates/codegen/xai-grok-shell/src/auth/oidc/login.rs
  - crates/codegen/xai-grok-shell/src/auth/recovery.rs
  - crates/codegen/xai-grok-shell/src/auth/refresh/oidc_refresher_tests.rs
  - crates/codegen/xai-grok-shell/tests/auth_multi_slot.rs
  - crates/codegen/xai-grok-pager/src/app/cli.rs
  - crates/codegen/xai-grok-shell/src/agent/relay.rs
findings:
  critical: 0
  high: 2
  medium: 3
  low: 3
  total: 8
status: findings
---

# Phase 2: Code Review Report

**Reviewed:** 2026-07-16T07:59:19Z  
**Depth:** deep  
**Scope:** multi-slot credentials & xAI OAuth (`89040e7^..HEAD` auth/CLI surface)  
**Status:** findings

## Summary

Phase 2 closes the plan-review HIGH gaps for the happy path: dual acquiring/guard-held mutation APIs, `still_live` checks, multi-provider-aware prune, xAI-only devbox recovery, fail-closed unsupported versions, honest `FileDeleted` after successful `remove_file`, `GROK_AUTH_PATH` foreign-path rejection, and nested Bearer proofs.

Adversarial review still finds **two HIGH defects** that can remove or orphan sibling provider credentials under realistic failure/concurrency shapes, plus medium isolation and telemetry gaps. Lock reentrancy on the dual write APIs looks correct; residual risks are outside-lock destructive recovery and stale full-slot replace of `providers.xai`.

Focus areas:

| Focus | Verdict |
|-------|---------|
| Lock reentrancy / self-deadlock | Guard-held APIs OK; acquiring paths used only when unlocked |
| Sibling provider clobber | Happy-path RMW OK; corrupt recovery + outside-lock wipe paths not OK |
| `GROK_AUTH_PATH` isolation / symlink | Foreign override sealed; default-path symlink still followed |
| Token logging redaction | Uses `token_suffix` on persist telemetry |
| `FileDeleted` honesty | Correct at storage layer |
| Unsupported version | Fail-closed; not treated as corrupt |

## Critical Issues

_None._

## High Issues

### HI-01: Document-wide `InvalidData` recovery wipes sibling providers from the live store

**Severity:** HIGH  
**File:** `crates/codegen/xai-grok-shell/src/auth/storage.rs:248-273`, `181-188`  
**Also:** `crates/codegen/xai-grok-shell/src/auth/manager.rs:913-925` (`persist_scope` uses recovering reader)

**Issue:** Any `serde`/`InvalidData` failure on the multi-slot document (including a single bad field under `providers.codex` or a torn partial write) triggers `backup_corrupt_auth_file`, which **`rename`s the entire `auth.json` away**, then returns an empty document/store. The subsequent mutator rewrites only the xAI slot it knows about. Live Codex (and any future provider) credentials disappear from the product path; they survive only as `auth.json.corrupt.<millis>` if the rename succeeded.

This is a multi-slot regression relative to the phase goal “sibling providers must not be clobbered.” Unsupported `version` correctly fails closed and does **not** take this path — but ordinary parse failures do.

Call chain:

1. `read_auth_document_for_write` / `read_auth_json_or_empty_recovering_corrupt`
2. `InvalidData` → `backup_corrupt_auth_file` (rename whole file)
3. Empty doc → `apply_xai_slot` / xAI-only insert → live file has only xAI

**Fix:** Fail closed on write-path parse errors when any provider slot other than the mutator’s is present, or recover per-provider (keep parseable sibling maps). At minimum: under lock, if backup is required, re-parse raw JSON with `serde_json::Value`, preserve sibling `providers.*` objects byte-for-byte, and only clear the failed slot. Do not treat multi-slot `InvalidData` as “empty store + rewrite xAI.”

```rust
// Sketch: write-path reader
fn read_auth_document_for_write(path: &Path) -> io::Result<AuthDocument> {
    match read_auth_document(path) {
        Ok(doc) => Ok(doc),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(AuthDocument::default()),
        Err(e) if e.kind() == ErrorKind::Unsupported => Err(e), // already fail-closed
        Err(e) if e.kind() == ErrorKind::InvalidData => {
            // Prefer: preserve sibling providers from raw Value, or:
            Err(e) // fail closed on write rather than empty-recover multi-slot
        }
        Err(e) => Err(e),
    }
}
```

Also stop calling `read_auth_json_or_empty_recovering_corrupt` from `persist_scope` **before** the lock: a rename without `auth.json.lock` races concurrent writers (see HI-02).

---

### HI-02: `persist_scope` / enrichment replace the entire xAI slot from a snapshot taken outside the mutate closure

**Severity:** HIGH  
**File:** `crates/codegen/xai-grok-shell/src/auth/manager.rs:913-934`  
**Also:** `crates/codegen/xai-grok-shell/src/auth/manager/enrichment.rs:145-200`  
**Storage:** `write_auth_json` / `apply_xai_slot` at `storage.rs:315-338`

**Issue:** Happy-path sibling **providers** are preserved because `write_auth_json*` re-reads the full `AuthDocument` under lock and only swaps `providers.xai`. However the **xAI scope map** written is assembled outside that atomic RMW:

```913:934:crates/codegen/xai-grok-shell/src/auth/manager.rs
        let map = match read_auth_json_or_empty_recovering_corrupt(&self.path) {
            Ok(map) => map,
            // ...
        };
        let mut map = map;
        map.insert(self.scope.clone(), auth.clone());
        let write_result = match held_lock {
            Some(lock) => write_auth_json_with_lock(&self.path, lock, &map),
            None => write_auth_json(&self.path, &map),
        };
```

`apply_xai_slot` then does `doc.providers.insert(PROVIDER_XAI, auth_store.clone())`, replacing every xAI scope with that snapshot.

Consequences:

1. **Concurrent `store_api_key` lost:**  
   `store_api_key` correctly RMW-merges under lock; a racing `AuthManager::update` that read the pre-api-key map overwrites `providers.xai` and drops `xai::api_key`.
2. **Corrupt recovery without lock (compounds HI-01):**  
   `read_auth_json_or_empty_recovering_corrupt` may `rename` the live multi-slot file before `write_auth_json` acquires the lock.
3. Enrichment timeout path reads then `write_auth_json`s the same full-slot replace pattern (profile merge is safer under held lock, still full-slot replace).

Codex survival tests only cover sequential “seed codex → update xAI,” which passes even with this TOCTOU.

**Fix:** Perform the scope insert inside the locked document mutation so the xAI map is re-read under the same lock:

```rust
async fn persist_scope(...) -> io::Result<GrokAuth> {
    let write = |doc: &mut AuthDocument| {
        let xai = doc.providers.entry(PROVIDER_XAI.to_owned()).or_default();
        xai.insert(self.scope.clone(), auth.clone());
        doc.version = Some(AUTH_DOCUMENT_VERSION);
        Ok(())
    };
    match held_lock {
        Some(lock) => mutate_auth_document_with_lock(&self.path, lock, write)?,
        None => mutate_auth_document(&self.path, write)?,
    }
    // memory update + logs...
}
```

Mirror the same pattern for enrichment (mutate only the target scope’s profile fields under lock). Never call rename-based corrupt recovery outside `auth.json.lock`.

## Medium Issues

### ME-01: Default product `auth.json` symlink still followed (isolation incomplete)

**Severity:** MEDIUM  
**File:** `crates/codegen/xai-grok-shell/src/auth/manager.rs:269-291`  
**Also:** `crates/codegen/xai-grok-shell-base/src/util/secure_file.rs:72-82`

**Issue:** `resolve_auth_path` correctly rejects any `GROK_AUTH_PATH` that is not **lexically equal** to `{grok_home}/auth.json`, and tests cover foreign paths + in-home symlink overrides. Comments admit the secure writer follows symlinks — but the **default** product path is never checked with `symlink_metadata`. If `~/.bum/auth.json` itself is a symlink to `~/.grok/auth.json` (or another store), all manager writes land in the stock CLI credential file, violating product isolation.

**Fix:** Before first credential I/O, reject (or refuse to write through) a product auth path whose final component is a symlink; prefer `OpenOptions` + `O_NOFOLLOW` (Unix) in `open_secure_file` for auth writes, or resolve with `symlink_metadata` and error if `is_symlink()`.

---

### ME-02: `ScopeRemoval::Unchanged` reported as `EntryRemoved`

**Severity:** MEDIUM  
**File:** `crates/codegen/xai-grok-shell/src/auth/manager.rs:528-534`

**Issue:** Storage correctly distinguishes `XaiStoreMutation::Unchanged` (file never existed) from `DocumentWritten` / `FileDeleted`. `write_scope_removal` maps both `DocumentWritten` and `Unchanged` to `ScopeRemoval::EntryRemoved`, so unified telemetry claims “entry removed” when nothing was on disk. That undermines the phase’s “honest disk_mutation” goal (FileDeleted was fixed; EntryRemoved was not fully aligned).

**Fix:**

```rust
match mutate_xai_store_or_prune_with_lock(...)? {
    XaiStoreMutation::FileDeleted => Ok(ScopeRemoval::FileDeleted),
    XaiStoreMutation::DocumentWritten => Ok(ScopeRemoval::EntryRemoved),
    XaiStoreMutation::Unchanged => Ok(ScopeRemoval::SkippedUnreadable), // or new variant FileMissing
}
```

---

### ME-03: Auth unit suite still un-runnable; phase verification is check-only

**Severity:** MEDIUM  
**File:** plan summaries (`02-01-SUMMARY.md`, `02-02-SUMMARY.md`) + harness breakage outside this diff

**Issue:** Implementation added substantial concurrency/reentrancy/isolation tests, but summaries report `cargo test -p xai-grok-shell --lib auth::…` blocked by pre-existing harness errors (`EnvVarGuard`, `WorkspaceOps::for_test`, etc.). Critical multi-slot proofs (concurrent mutate, held-lock non-deadlock, unsupported version, GROK_AUTH_PATH) are therefore compile-time reviewed only. That is a ship-risk for an auth/storage phase.

**Fix:** Unblock shell lib tests (or extract `auth::storage` tests into an integration target that builds) before treating Phase 2 as verified. Do not rely on `cargo check` alone for lock/race proofs.

## Low Issues

### LO-01: `write_auth_json` inserts empty `providers.xai` instead of pruning the slot

**Severity:** LOW  
**File:** `crates/codegen/xai-grok-shell/src/auth/storage.rs:334-338`

**Issue:** `apply_xai_slot` always `insert`s the given map, including empty. WebLogin cleanup that removes the last xAI scope can leave `"xai": {}` on disk when Codex is absent (file not deleted) or when Codex remains (empty xAI key kept). Prune path via `mutate_xai_store_or_prune` handles this correctly; the write_auth_json path does not.

**Fix:** If `auth_store.is_empty()`, remove `PROVIDER_XAI` instead of inserting; optionally route empty writes through prune helper.

---

### LO-02: `token_suffix` returns the full secret when `len <= 12`

**Severity:** LOW  
**File:** `crates/codegen/xai-grok-shell/src/auth/model.rs:303-306`  
**Used by:** `manager.rs` persist logs (`key_prefix` / `rt_prefix`)

**Issue:** Short API keys / tokens are logged in full under “prefix” field names. Unlikely for OIDC JWTs; possible for short API keys.

**Fix:** Always redact to a fixed width (e.g. last 4 + length), or log a hash.

---

### LO-03: `write_scope_removal` double-reads the xAI map under the same lock

**Severity:** LOW  
**File:** `crates/codegen/xai-grok-shell/src/auth/manager.rs:524-530`

**Issue:** Reads via `read_auth_json`, mutates a local map, then `mutate_xai_store_or_prune_with_lock` re-reads the document and assigns `*xai = auth_store`. Correct under held lock, but redundant and easier to get wrong if a future edit drops the outer lock. Prefer a single in-closure `xai.remove(scope)`.

## What looks solid

- **Dual mutation APIs** (`mutate_auth_document` / `_with_lock`, `mutate_xai_store_or_prune` / `_with_lock`) with `still_live` before read and before persist.
- **Already-locked call sites** rewired: WebLogin cleanup, scope removal, refresh `update_with_lock`, enrichment-with-lock.
- **Devbox recovery** no longer `remove_file`s whole `auth.json`; `replace_xai_after_devbox_mint` clears only the xAI map under lock.
- **`FileDeleted`** only after successful `remove` (`persist_document_or_prune` + unit test with denied remove).
- **Unsupported version** → `ErrorKind::Unsupported`; recovering reader does not wipe; unit coverage present.
- **Token telemetry** on persist uses `token_suffix`, not raw secrets.
- **Login proofs** (OIDC browser + device code) assert nested `version`/`providers.xai` and seeded Codex survival; CLI bare `bum login` parses without provider arg.
- **Agent-turn seam test** seeds `sampling_config.api_key` from nested multi-slot manager and asserts sampler `Authorization: Bearer`.

## Recommended fix order

1. **HI-01 / HI-02** — lock-scoped RMW for scope insert; never full-document empty-recover when sibling providers exist; no rename outside lock.  
2. **ME-01** — refuse symlink product auth path / `O_NOFOLLOW`.  
3. **ME-02** — honest `ScopeRemoval` mapping.  
4. **ME-03** — restore runnable auth tests.  
5. Low items opportunistically.

---

_Reviewed: 2026-07-16T07:59:19Z_  
_Reviewer: gsd-code-reviewer (adversarial)_  
_Depth: deep_  
_Base: `89040e7^..HEAD` / `236bea6`_
