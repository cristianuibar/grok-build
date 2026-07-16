# Phase 2: Multi-slot credentials & xAI OAuth - Pattern Map

**Mapped:** 2026-07-16
**Files analyzed:** 14
**Analogs found:** 14 / 14

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `crates/codegen/xai-grok-shell/src/auth/model.rs` | model | transform (serde types) | self (`AuthStore`, `GrokAuth`, `API_KEY_SCOPE`) | exact |
| `crates/codegen/xai-grok-shell/src/auth/storage.rs` | service / file store | file-I/O (read-merge-write) | self (`read_auth_json` / `write_auth_json` / API key helpers) | exact |
| `crates/codegen/xai-grok-shell/src/auth/manager.rs` | service | request-response + file-I/O | self (`update`, `write_scope_removal`, `new`) | exact |
| `crates/codegen/xai-grok-shell/src/auth/manager/lock.rs` | utility / lock | file-I/O (advisory flock) | self (reuse unchanged) | exact |
| `crates/codegen/xai-grok-shell/src/auth/manager_tests.rs` | test | file-I/O + CRUD | self (`tempfile` + `write_auth_json` fixtures) | exact |
| `crates/codegen/xai-grok-shell/src/auth/storage.rs` (`#[cfg(test)]`) | test | file-I/O | self `write_fallback_tests` module | exact |
| `crates/codegen/xai-grok-shell/src/auth/mod.rs` | config / barrel | transform (re-exports) | self `pub use storage::{…}` | exact |
| `crates/codegen/xai-grok-shell/src/auth/flow.rs` | controller / flow | request-response | self `run_cli_login` / `perform_logout` | exact (minimal touch) |
| `crates/codegen/xai-grok-shell/src/auth/device_code.rs` | service | request-response | self `complete_device_code_login` | exact (doc path fix only) |
| `crates/codegen/xai-grok-shell/src/auth/credential_provider.rs` | service / middleware | request-response | self `ShellAuthCredentialProvider` | exact (verify only) |
| `crates/codegen/xai-grok-shell/src/util/grok_auth_credentials.rs` | utility | request-response (HTTP headers) | self `GrokAuthCredentials::apply` | exact (verify only) |
| `crates/codegen/xai-grok-shell/src/managed_config.rs` | service (consumer) | file-I/O read | self `read_auth_json` callers | role-match (must keep xAI-slot adapter) |
| `crates/codegen/xai-grok-pager/src/app/cli.rs` | config / CLI | request-response | self `Command::Login` flags | exact (no `--provider` yet) |
| `crates/codegen/xai-grok-pager-bin/src/main.rs` | controller | request-response | self `Command::Login` → `run_cli_login` | exact (no change expected) |

## Pattern Assignments

### `crates/codegen/xai-grok-shell/src/auth/model.rs` (model, transform)

**Analog:** same file — add document types beside existing scope map; do **not** change `AuthStore` wire type for callers.

**Imports / existing type alias** (lines 1–14, 234):
```rust
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// auth.json scope key for plain API key auth
pub const API_KEY_SCOPE: &str = "xai::api_key";

pub(crate) type AuthStore = BTreeMap<String, GrokAuth>;
```

**Keep `AuthStore` as scope→`GrokAuth` map.** Add provider constants + on-disk document (discretion shape from RESEARCH):

```rust
/// Stable wire keys for multi-slot auth.json (Phase 2+).
pub const PROVIDER_XAI: &str = "xai";
pub const PROVIDER_CODEX: &str = "codex";

/// Optional schema version for future migrations.
pub const AUTH_DOCUMENT_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuthDocument {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<u32>,
    #[serde(default)]
    pub providers: BTreeMap<String, AuthStore>,
}
```

**Serde field conventions to copy** from `GrokAuth` (lines 49–105):
```rust
#[serde(default, skip_serializing_if = "Option::is_none")]
// and for collections:
#[serde(default, skip_serializing_if = "Vec::is_empty")]
```

**Debug redaction pattern** (lines 107–119) — keep tokens out of logs; document types hold same `GrokAuth` leaves so no new redaction needed at document layer.

**Test fixture helper** (lines 218–231):
```rust
#[cfg(test)]
impl GrokAuth {
    pub fn test_default() -> Self {
        Self {
            key: "test-key".into(),
            user_id: "test-user".into(),
            ..Default::default()
        }
    }
}
```

**Lookup pattern stays on `AuthStore` only** (lines 293–306) — manager continues to call `lookup_auth(&xai_slot, &scope)` after storage extracts the xAI provider map.

---

### `crates/codegen/xai-grok-shell/src/auth/storage.rs` (service, file-I/O) — **primary change surface**

**Analog:** same file. Storage-only multi-slot adapter: public `read_auth_json` / `write_auth_json` continue to speak **xAI `AuthStore`**, while disk bytes become nested `AuthDocument`.

#### Read path (today — lines 50–64)

```rust
pub fn read_auth_json(auth_file: &Path) -> std::io::Result<AuthStore> {
    let mut file = File::open(auth_file)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let trimmed = contents.trim();
    if trimmed.is_empty() {
        return Ok(AuthStore::new());
    }
    let map = serde_json::from_str(trimmed)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    Ok(map)
}
```

**Evolve to:** parse JSON value → detect nested (`providers` object present) vs legacy flat (scope keys) → return `providers.xai` (or empty). Keep signature `Result<AuthStore>` so `managed_config`, `AuthManager`, tests stay green.

Recommended detection (from RESEARCH; implement in this file):
```rust
// Pseudocode — concrete impl belongs here
// 1. serde_json::Value from trimmed
// 2. if value.get("providers").is_some() → AuthDocument → xai slot
// 3. else → treat whole object as legacy flat AuthStore (xAI)
```

#### Corrupt recovery + empty readers (lines 80–170) — preserve control flow

```rust
pub(crate) fn read_auth_json_or_empty(auth_file: &Path) -> std::io::Result<AuthStore> {
    match read_auth_json(auth_file) {
        Ok(map) => Ok(map),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(AuthStore::new()),
        Err(e) => Err(e),
    }
}

pub(crate) fn read_auth_json_or_empty_recovering_corrupt(
    auth_file: &Path,
) -> std::io::Result<AuthStore> {
    match read_auth_json(auth_file) {
        Ok(map) => Ok(map),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(AuthStore::new()),
        Err(e) if e.kind() == std::io::ErrorKind::InvalidData => {
            let _ = backup_corrupt_auth_file(auth_file);
            Ok(AuthStore::new())
        }
        Err(e) => Err(e),
    }
}
```

After multi-slot: recovery still returns **empty xAI slot** semantics to callers; backup of corrupt file unchanged (`auth.json.corrupt.<millis>` pattern lines 94–147).

#### Atomic write + StorageFull fallback (lines 172–326) — keep mechanics, change payload type internally

```rust
pub(super) fn write_auth_json(auth_file: &Path, auth_store: &AuthStore) -> std::io::Result<()> {
    write_auth_json_with(auth_file, auth_store, write_auth_json_atomic)
}

fn write_store_to(path: &Path, auth_store: &AuthStore) -> std::io::Result<()> {
    use crate::util::secure_file::open_secure_file;
    // create_dir_all parent → open_secure_file (0o600) → to_writer_pretty → fsync
}
```

**Critical Phase 2 change:** `write_auth_json` must **not** serialize the xAI map as the whole file. Merge-safe sketch (RESEARCH Pattern 1):

```rust
// Implement inside storage.rs — do not scatter at call sites
fn write_xai_auth_store(path: &Path, xai_store: &AuthStore) -> std::io::Result<()> {
    let mut doc = read_auth_document_or_empty_recovering(path)?;
    doc.providers.insert(PROVIDER_XAI.to_owned(), xai_store.clone());
    doc.version = Some(AUTH_DOCUMENT_VERSION);
    // Do NOT insert empty PROVIDER_CODEX by default; preserve if already present
    write_auth_document_atomic(path, &doc)
}
```

Reuse existing atomic path (tmp `auth.json.<pid>.tmp` + rename) and `write_auth_json_with` StorageFull → in-place fallback (lines 200–224, 257–267). Only the **serialized type** changes from `AuthStore` to `AuthDocument`.

#### API key helpers (lines 339–376) — already read-merge-write within one map; must go through multi-slot write

```rust
pub fn store_api_key(grok_home: &Path, api_key: &str) -> std::io::Result<()> {
    let path = grok_home.join("auth.json");
    let mut map = read_auth_json_or_empty_recovering_corrupt(&path)?;
    map.insert(
        API_KEY_SCOPE.to_owned(),
        GrokAuth {
            key: api_key.to_owned(),
            auth_mode: AuthMode::ApiKey,
            ..Default::default()
        },
    );
    write_auth_json(&path, &map) // after adapter: merges into providers.xai only
}

pub fn clear_api_key(grok_home: &Path) -> std::io::Result<()> {
    let path = grok_home.join("auth.json");
    if let Ok(mut map) = read_auth_json(&path) {
        map.remove(API_KEY_SCOPE);
        if map.is_empty() {
            let _ = std::fs::remove_file(&path); // MUST change: delete only if ALL providers empty
        } else {
            write_auth_json(&path, &map)?;
        }
    }
    Ok(())
}
```

**Empty-file delete pitfall:** `clear_api_key` and manager `write_scope_removal` currently delete when the **xAI map** is empty. After multi-slot, deletion must check the **full document** (no scopes in any provider). Prefer a shared helper e.g. `write_xai_store_or_prune(path, xai_store)` used by both storage API-key clear and manager scope removal.

#### Error / user-facing path strings (lines 330–336)

```rust
read_auth_json(&path).map_err(|_| anyhow::anyhow!("Not logged in. Run `grok login`."))?;
```

Phase 8 owns full string rebrand; only fix path-wrong messages that still point users at `~/.grok` (device_code docs). CLI name `grok login` may stay until Phase 8 unless product-home paths are wrong.

#### Existing unit tests to keep green + extend (lines 378–512)

Pattern: `tempfile::tempdir` + `sample_store()` + injectable `write_auth_json_with` for StorageFull.

```rust
#[cfg(test)]
mod write_fallback_tests {
    fn sample_store() -> AuthStore { /* API_KEY_SCOPE entry */ }
    #[test]
    fn atomic_write_roundtrips() { /* write_auth_json + read_key */ }
    #[test]
    fn falls_back_to_in_place_on_storage_full() { /* fake_storage_full */ }
}
```

**Add sibling test module** (same style) for multi-slot:
- `legacy_flat_reads_as_xai_slot`
- `write_emits_nested_providers_xai`
- `seeded_codex_slot_survives_xai_write`
- `store_api_key_only_touches_xai`
- `clear_api_key_does_not_delete_file_when_codex_present`
- Assert raw JSON has `"providers"` and optional `"version": 1` after write

Optional test helper (prefer over raw `fs::write` of nested JSON):
```rust
#[cfg(test)]
pub(crate) fn write_fixture_auth_document(
    path: &Path,
    xai: AuthStore,
    codex: Option<AuthStore>,
) -> std::io::Result<()> { /* build AuthDocument + atomic write */ }
```

---

### `crates/codegen/xai-grok-shell/src/auth/manager.rs` (service, request-response + file-I/O)

**Analog:** same file. Prefer **storage-only adapter** (CONTEXT discretion): keep single xAI `AuthManager`; leave refresh/sleep-gate/lock logic alone if `read_auth_json`/`write_auth_json` preserve AuthStore semantics.

#### Construction + path priority (lines 262–298)

```rust
pub fn new(grok_home: &Path, grok_com_config: GrokComConfig) -> Self {
    let scope = grok_com_config.auth_scope();
    // GROK_AUTH inline JSON > GROK_AUTH_PATH > {grok_home}/auth.json
    let path = std::env::var("GROK_AUTH_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| grok_home.join("auth.json"));
    let (auth, …) = match read_auth_json(&path) {
        Ok(map) => {
            let found = lookup_auth(&map, &scope);
            // …
        }
        // …
    };
}
```

Already logs `BUM_HOME` (line 274). No dual AuthManager. `grok_home` comes from caller (`xai_grok_config::grok_home()` → `BUM_HOME` / `~/.bum` after Phase 1).

#### Update / save merge pattern (lines 794–875) — **must stay correct via storage**

```rust
pub(crate) async fn update(self: &Arc<Self>, auth: GrokAuth) -> std::io::Result<GrokAuth> {
    let map = match read_auth_json_or_empty_recovering_corrupt(&self.path) {
        Ok(map) => map,
        Err(e) => {
            // Non-recoverable: in-memory only + still spawn enrichment
            self.with_inner_write(|inner| *inner = Some(auth.clone()));
            self.spawn_user_info_enrichment(auth.clone());
            return Ok(auth);
        }
    };
    let mut map = map;
    map.insert(self.scope.clone(), auth.clone());
    let write_result = write_auth_json(&self.path, &map);
    self.with_inner_write(|inner| *inner = Some(auth.clone()));
    self.spawn_user_info_enrichment(auth.clone());
    write_result?;
    Ok(auth)
}
```

Same structure for `save_without_enrichment` and test-only `persist_and_swap` (lines 1022–1035). If storage merge is correct, **no manager rewrite required** for multi-slot isolation.

#### Scope removal / empty file delete (lines 450–493) — **must fix delete semantics**

```rust
fn write_scope_removal(&self, scope: &str) -> std::io::Result<ScopeRemoval> {
    let Ok(mut auth_store) = read_auth_json(&self.path) else {
        return Ok(ScopeRemoval::SkippedUnreadable);
    };
    auth_store.remove(scope);
    if auth_store.is_empty() {
        let _ = std::fs::remove_file(&self.path); // BUG after multi-slot if codex present
        Ok(ScopeRemoval::FileDeleted)
    } else {
        write_auth_json(&self.path, &auth_store)?;
        Ok(ScopeRemoval::EntryRemoved)
    }
}
```

**Required change:** when xAI map is empty but `providers.codex` (or other) has scopes → write document with xAI absent/empty and **do not** `remove_file`. Prefer storage helper that prunes only the empty xAI provider key and deletes file only if `doc.providers` is entirely empty.

#### Disk read for agent path (lines 1061–1123)

```rust
fn read_disk_auth_silent(&self) -> Option<GrokAuth> {
    read_auth_json(&self.path)
        .ok()
        .and_then(|map| lookup_auth(&map, &self.scope))
}

pub(crate) async fn get_valid_token(self: &Arc<Self>) -> Result<String, AuthError> {
    self.auth().await.map(|a| a.key)
}
```

Unchanged call shape after adapter; tests should load nested fixture and assert `get_valid_token` / `current()` return xAI key.

#### Lock discipline (do not weaken)

File lock lives in `manager/lock.rs`; refresh path holds lock before IdP + write. Plain `update()` does not always hold lock today — multi-slot merge-read before write is still process-level via rename. Do not invent a second lock scheme.

---

### `crates/codegen/xai-grok-shell/src/auth/manager/lock.rs` (utility, file-I/O)

**Analog:** same file — **reuse unchanged**.

**Lock path sibling pattern** (from storage `AuthFileLock::still_live`, lines 33–35):
```rust
let lock_path = auth_json_path.with_file_name("auth.json.lock");
```

**Holder format** (lock.rs lines 28–40):
```rust
// Write PID:UNIX_TIMESTAMP into lock file
write!(file, "{pid}:{ts}")?;
file.sync_all()?;
```

Planner: no schema change for lock file; still one file next to multi-slot `auth.json`.

---

### `crates/codegen/xai-grok-shell/src/auth/manager_tests.rs` (test, file-I/O + CRUD)

**Analog:** same file + storage `write_fallback_tests`.

**Standard fixture pattern** (lines 38–61, 101–119):
```rust
let dir = tempfile::tempdir().unwrap();
let cfg = GrokComConfig::default();
let mgr = Arc::new(AuthManager::new(dir.path(), cfg));

// Seed disk:
let mut store = AuthStore::new();
store.insert(LEGACY_SCOPE.to_string(), legacy_auth);
write_auth_json(&auth_path, &store).unwrap();
```

**Assertions after write** (existing style around lines 226, 433):
```rust
let store = read_auth_json(&dir.path().join("auth.json")).unwrap();
// After multi-slot: this still returns xAI AuthStore via adapter.
// For on-disk shape tests, read raw bytes / serde_json::Value and assert "providers".
```

**New tests to add (same module or `storage` tests):**
1. Nested document load: write fixture with `providers.xai` + `providers.codex` → `AuthManager::new` finds xAI scope token
2. `mgr.update(auth).await` does not drop seeded codex scopes (read raw file)
3. `remove_scope` / logout path leaves file when codex non-empty
4. Credential path: after multi-slot load, `get_valid_token` / `ShellAuthCredentialProvider::snapshot` / `GrokAuthCredentials::apply` Bearer header

Keep `make_auth` helper (lines 10–17) and `GrokAuth::test_default()`.

---

### `crates/codegen/xai-grok-shell/src/auth/mod.rs` (barrel)

**Analog:** same file lines 41–43:
```rust
pub use storage::{
    clear_api_key, read_api_key, read_auth_json, read_token_by_scope, store_api_key,
};
```

If adding `read_auth_document` / `PROVIDER_*` constants for Phase 5 prep, re-export deliberately (`pub use model::{…}`). Prefer `pub(crate)` for document internals this phase unless tests need them.

Also exports (lines 28–37):
```rust
pub use flow::{… run_cli_login, run_cli_logout, …};
pub use manager::{AuthManager, shared_api_key_provider};
pub use model::{AuthMode, GrokAuth, lookup_auth};
```

---

### `crates/codegen/xai-grok-shell/src/auth/flow.rs` (controller, request-response)

**Analog:** same file — expect **minimal/no logic change** for AUTH-01 if storage adapter works.

**CLI login entry** (lines 866–941):
```rust
pub async fn run_cli_login(
    config: &crate::agent::config::Config,
    oauth: bool,
    device_auth: bool,
    devbox: bool,
) -> anyhow::Result<()> {
    let login_override = LoginTransportOverride::from_flags(oauth, device_auth);
    // device branch:
    let grok_home = grok_home::grok_home();
    let auth_manager = Arc::new(AuthManager::new(&grok_home, config.grok_com_config.clone()));
    // loopback branch:
    ensure_authenticated_with_override(…, reauth=true, …).await?;
}
```

No `--provider` flag. Defaults to xAI via `GrokComConfig` / `auth_scope()`.

**Logout** (lines 960+) uses `auth_manager` scope removal — inherits multi-slot delete fix from manager/storage.

---

### `crates/codegen/xai-grok-shell/src/auth/device_code.rs` (service, request-response)

**Analog:** same file. Logic persists via `AuthManager::update` (unchanged path).

**Stale doc comment** (line 207) — fix path-wrong product home:
```rust
/// On success, persists credentials to `~/.grok/auth.json` and returns
```
→ product home / `$BUM_HOME/auth.json` (or “the product auth store”). Full chrome rebrand still Phase 8.

Device-code tests already assert auth.json under passed `grok_home` (lines 704–705).

---

### `crates/codegen/xai-grok-shell/src/auth/credential_provider.rs` (service, request-response)

**Analog:** same file — **verify only** after storage change.

**HTTP apply + snapshot** (lines 42–76):
```rust
impl HttpAuth for ShellAuthCredentialProvider {
    fn apply(&self, builder: RequestBuilder, base_url: &str) -> RequestBuilder {
        let mut creds = self.static_credentials.clone();
        if creds.deployment_key.is_none()
            && let Some(auth) = self.auth_manager.current_or_expired()
        {
            creds.user_token = Some(auth.key);
        }
        creds.apply(builder, base_url)
    }
}

fn snapshot(&self) -> CredentialSnapshot {
    let auth = self.auth_manager.current_or_expired();
    // token / user_id / team_id / api_key_id …
}
```

Depends entirely on `AuthManager` seeing the xAI slot after multi-slot load.

---

### `crates/codegen/xai-grok-shell/src/util/grok_auth_credentials.rs` (utility, request-response)

**Analog:** same file — **verify only**.

**Bearer wire contract** (lines 119–134):
```rust
pub fn apply(&self, builder: RequestBuilder, base_url: &str) -> RequestBuilder {
    if let Some(ref key) = self.deployment_key {
        builder.header("Authorization", format!("Bearer {}", key))
    } else if let Some(ref token) = self.user_token {
        builder
            .header("Authorization", format!("Bearer {}", token))
            .header(
                obfstr::obfstr!("X-XAI-Token-Auth"),
                obfstr::obfstr!("xai-grok-cli"),
            )
    } else {
        builder
    }
}
```

**Agent-turn test pattern already in-file** (lines 147–178):
```rust
let dir = tempfile::tempdir().unwrap();
let mgr = Arc::new(AuthManager::new(dir.path(), GrokComConfig::default()));
mgr.hot_swap(auth);
let creds = GrokAuthCredentials::new(None).with_auth_manager(mgr);
assert_eq!(creds.resolve().user_token.as_deref(), Some("test-bearer-token"));
```

Extend with **disk-backed multi-slot**: write nested document under `dir`, construct manager without hot_swap, assert `resolve` / `get_valid_token`.

---

### `crates/codegen/xai-grok-shell/src/managed_config.rs` (consumer)

**Analog:** same file lines 86–107:
```rust
fn read_active_team_auth() -> Option<GrokAuth> {
    let home = crate::util::grok_home::grok_home();
    let store = crate::auth::read_auth_json(&home.join("auth.json")).ok()?;
    let team = store.values().find(|a| a.is_team_principal())?.clone();
    eligible_team_principal(team)
}
```

**Constraint:** `read_auth_json` must continue returning the **xAI scope map**. No managed_config changes if storage adapter is correct. Do not teach managed_config about `providers.codex` this phase.

---

### `crates/codegen/xai-grok-pager/src/app/cli.rs` + `pager-bin/src/main.rs` (CLI entry)

**Analog:** existing login flags (cli.rs ~26–34):
```rust
#[arg(long = "oauth", alias = "oidc", conflicts_with_all = ["device_auth"])]
oauth: bool,
#[arg(long = "device-auth", /* alias device-code */, conflicts_with_all = ["oauth"])]
device_auth: bool,
```

**pager-bin** (main.rs ~1739–1751):
```rust
Command::Login { oauth, device_auth, devbox, … } => {
    xai_grok_shell::auth::run_cli_login(&config, oauth, device_auth, devbox).await?;
}
```

No `--provider` this phase. Expect **zero** CLI surface change unless path/help text wrongly says `~/.grok`.

---

### `crates/codegen/xai-grok-auth` traits (no multi-slot types this phase)

**Analog:** `crates/codegen/xai-grok-auth/src/auth_provider.rs` lines 13–65 — `AuthCredentialProvider` / `CredentialSnapshot` remain provider-agnostic. Do not push multi-slot types into this crate in Phase 2.

---

### `auth/config.rs` (unchanged scope keys)

**Analog:** same file — xAI runtime defaults:
```rust
pub const XAI_OAUTH2_ISSUER: &str = "https://auth.x.ai";

pub fn auth_scope(&self) -> String {
    // oidc issuer::client_id OR oauth2.auth_scope() OR unreachable
}
```

Default client_id obfuscated in `GrokComConfig::default` (`b1a00492-…`). Scope keys stay **inside** `providers.xai`; do not invent `providers.xai::{issuer}::{client_id}` double-nesting.

## Shared Patterns

### Single-file auth store + advisory lock
**Source:** `auth/storage.rs` + `auth/manager/lock.rs`  
**Apply to:** All multi-slot reads/writes  
- File: `$BUM_HOME/auth.json` (or `GROK_AUTH_PATH`)  
- Lock: sibling `auth.json.lock`  
- Atomic write: tmp + rename; StorageFull → in-place + restore prior bytes  
- Mode: 0o600 via `open_secure_file`

### Storage-only multi-slot adapter (Phase 2 discretion)
**Source:** RESEARCH Pattern 1; implement in `storage.rs`  
**Apply to:** Every `write_auth_json` / `read_auth_json` call site (implicit)  
- Runtime type stays `AuthStore` (xAI scopes)  
- On-disk type is `AuthDocument { version, providers }`  
- Write = load document → replace `providers.xai` → preserve siblings → serialize nested  
- Eager migrate on successful write (recommended)

### AuthManager update (scope merge inside xAI slot)
**Source:** `auth/manager.rs` lines 794–848  
**Apply to:** Login, refresh, enrichment, API-key store  
```rust
map.insert(self.scope.clone(), auth.clone());
write_auth_json(&self.path, &map);
// Always update in-memory even if disk write fails
```

### Corrupt recovery before write
**Source:** `storage.rs` `read_auth_json_or_empty_recovering_corrupt` + `backup_corrupt_auth_file`  
**Apply to:** `update`, `store_api_key`, any write that must not die on bad JSON  
- InvalidData → backup `auth.json.corrupt.<ms>` → empty map → write fresh  
- Multi-slot writes after recovery must still only put intended xAI content into a **fresh document** (do not invent fake codex data)

### Empty-store delete (must become multi-provider-aware)
**Source today:** `manager.rs` `write_scope_removal`, `storage.rs` `clear_api_key`  
**Apply to:** Logout, scope removal, API key clear  
- **Before:** delete file if single map empty  
- **After:** delete file only if all provider maps empty / document empty; else write nested remainder

### Credential → HTTP (agent turn proof)
**Source:** `util/grok_auth_credentials.rs` + `auth/credential_provider.rs`  
**Apply to:** AUTH-01 automated proof  
```rust
// Live: AuthManager::get_valid_token() / current_or_expired()
// Wire: Authorization: Bearer <token>
//       X-XAI-Token-Auth: xai-grok-cli  (user/OAuth, not deployment key)
```

### Temp-home tests
**Source:** `manager_tests.rs`, `storage::write_fallback_tests`, Phase 1 `BUM_HOME` test-support  
**Apply to:** All Phase 2 unit tests  
```rust
let dir = tempfile::tempdir().unwrap();
let mgr = Arc::new(AuthManager::new(dir.path(), GrokComConfig::default()));
// Prefer write_auth_json / write_fixture_auth_document over raw flat JSON
// Prefer dir.path() as grok_home — never real ~/.grok
```

### Logging / secrets
**Source:** `model::token_suffix`, manager unified_log fields  
**Apply to:** Any new storage logs  
- Log paths and provider ids, not tokens  
- `GrokAuth` Debug already redacts keys

## No Analog Found

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| — | — | — | No greenfield files; multi-slot is an evolution of existing auth storage. Closest “document envelope” is new `AuthDocument` modeled after RESEARCH + serde conventions on `GrokAuth` / `GrokComConfig`. |

External OAuth protocol, dual AuthManager runtime, Codex login, and model routing have **no** Phase 2 implementation analogs by design (deferred Phases 4–5).

## Anti-patterns (do not copy)

| Anti-pattern | Where seen | Correct pattern |
|--------------|------------|-----------------|
| `serde_json::to_writer(auth_store)` as whole file after nesting | Pre-Phase-2 `write_store_to` | Write `AuthDocument` with merge |
| Delete `auth.json` when xAI map empty | `write_scope_removal`, `clear_api_key` | Delete only if all providers empty |
| Import `~/.grok/auth.json` | N/A (forbidden) | Bum home only; legacy flat under bum home is migration input |
| Dual AuthManager / provider routing now | Future phases | Storage-only adapter |
| Raw `fs::write` flat fixtures that never exercise nested write | Some relay/helpers | Prefer storage API; still accept flat on **read** |
| Live browser OAuth as only AUTH-01 proof | Manual smoke | Fixture + AuthManager + Bearer apply unit tests |

## Metadata

**Analog search scope:**  
`crates/codegen/xai-grok-shell/src/auth/**`, `util/grok_auth_credentials.rs`, `managed_config.rs`, `xai-grok-auth/src/auth_provider.rs`, pager CLI / pager-bin Login, Phase 1 `01-PATTERNS.md` format reference

**Files scanned:** ~25 primary modules (auth tree + consumers + CLI entrypoints)  
**Pattern extraction date:** 2026-07-16  
**Primary recommendation for planner:** Land multi-slot in `storage.rs` first (document type in `model.rs`), adjust empty-delete in manager/storage, extend unit tests; leave OIDC/device-code/lock/credential_provider paths untouched except doc/path hygiene.
