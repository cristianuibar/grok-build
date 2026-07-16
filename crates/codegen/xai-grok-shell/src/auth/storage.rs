use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

#[cfg(test)]
use super::model::PROVIDER_CODEX;
use super::model::{
    API_KEY_SCOPE, AUTH_DOCUMENT_VERSION, AuthDocument, AuthMode, AuthStore, GrokAuth,
    PROVIDER_XAI, lookup_auth,
};

/// RAII guard for an exclusive advisory lock on `auth.json.lock`.
/// The lock is released when the inner `File` is dropped (closing the FD).
pub(crate) struct AuthFileLock {
    pub(super) _file: File,
}

/// Result of a lock-scoped xAI slot mutation that may prune `auth.json`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum XaiStoreMutation {
    /// The nested document was written because at least one provider has scopes.
    DocumentWritten,
    /// The file did not exist and the mutation left every provider empty.
    Unchanged,
    /// The file was successfully removed because every provider became empty.
    FileDeleted,
}

impl AuthFileLock {
    /// Returns `true` while this guard still refers to the **live**
    /// `auth.json.lock` inode.
    ///
    /// A waiter that finds a holder stuck past the stale-lock timeout breaks
    /// the lock by `unlink`ing the file and recreating it on a fresh inode
    /// (see [`crate::auth::manager::lock`]). The usual cause of a "stuck"
    /// holder is a process **suspended across system sleep** while holding the
    /// lock: it stays alive (so the kernel never releases its flock) yet makes
    /// no progress, so siblings break it. When such a holder resumes, its
    /// flock lives on the now-deleted inode — it no longer holds the live lock
    /// even though this `AuthFileLock` still exists.
    ///
    /// Callers about to perform an irreversible, lock-protected action
    /// (sending a refresh token to the IdP, writing `auth.json`) MUST
    /// re-validate first; otherwise two processes can spend the same refresh
    /// token and trip token-family revocation.
    ///
    /// Non-Unix has no inode concept, so this conservatively returns `true`.
    #[cfg(unix)]
    pub(crate) fn still_live(&self, auth_json_path: &Path) -> bool {
        use std::os::unix::fs::MetadataExt;
        let lock_path = auth_json_path.with_file_name("auth.json.lock");
        let (Ok(fd_meta), Ok(path_meta)) = (self._file.metadata(), std::fs::metadata(&lock_path))
        else {
            // Lock file gone or unreadable → we no longer hold the live lock.
            return false;
        };
        fd_meta.ino() == path_meta.ino() && fd_meta.dev() == path_meta.dev()
    }

    #[cfg(not(unix))]
    pub(crate) fn still_live(&self, _auth_json_path: &Path) -> bool {
        true
    }
}

// ── Errors ────────────────────────────────────────────────────────────

fn unsupported_version_error(version: u32) -> std::io::Error {
    std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        format!(
            "unsupported auth.json document version {version} \
             (max supported is {AUTH_DOCUMENT_VERSION})"
        ),
    )
}

fn lock_not_live_error() -> std::io::Error {
    std::io::Error::new(
        std::io::ErrorKind::WouldBlock,
        "auth.json.lock is no longer live; refusing write",
    )
}

// ── Read path ─────────────────────────────────────────────────────────

/// Parse on-disk bytes into an [`AuthDocument`].
///
/// - Empty / whitespace → empty document
/// - Nested (`providers` object present) → `AuthDocument`; reject
///   `version > AUTH_DOCUMENT_VERSION` with [`ErrorKind::Unsupported`]
/// - Legacy flat scope map → wrapped as in-memory `providers.xai`
///
/// Does not open stock credential homes (D-13).
pub(crate) fn read_auth_document(auth_file: &Path) -> std::io::Result<AuthDocument> {
    let mut file = File::open(auth_file)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    parse_auth_document_str(&contents)
}

fn parse_auth_document_str(contents: &str) -> std::io::Result<AuthDocument> {
    let trimmed = contents.trim();
    if trimmed.is_empty() {
        return Ok(AuthDocument::default());
    }

    let value: serde_json::Value = serde_json::from_str(trimmed)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    let Some(obj) = value.as_object() else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "auth.json root must be a JSON object",
        ));
    };

    // Nested multi-slot form: top-level `providers` object present.
    if obj.get("providers").is_some_and(|p| p.is_object()) {
        let doc: AuthDocument = serde_json::from_value(value)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        if let Some(v) = doc.version
            && v > AUTH_DOCUMENT_VERSION
        {
            return Err(unsupported_version_error(v));
        }
        return Ok(doc);
    }

    // Legacy flat scope → GrokAuth map: treat as xAI slot only.
    let store: AuthStore = serde_json::from_value(value)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    let mut doc = AuthDocument {
        version: None,
        providers: Default::default(),
    };
    doc.providers.insert(PROVIDER_XAI.to_owned(), store);
    Ok(doc)
}

/// Read the xAI provider slot from `auth.json` (legacy flat or nested).
///
/// Public surface stays an [`AuthStore`] scope map so AuthManager and other
/// callers remain storage-agnostic (D-12).
pub fn read_auth_json(auth_file: &Path) -> std::io::Result<AuthStore> {
    let doc = read_auth_document(auth_file)?;
    Ok(doc.providers.get(PROVIDER_XAI).cloned().unwrap_or_default())
}

/// Read auth.json, returning an empty map if the file does not exist.
///
/// Non-empty corrupt JSON, permission errors, etc. are returned as errors
/// so the caller can decide whether to skip the write (to avoid clobbering
/// sibling scopes).
///
/// Kept for the test-only `persist_and_swap` and as a strict reader.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "used from tests only; remove expect when wired in production"
    )
)]
pub(crate) fn read_auth_json_or_empty(auth_file: &Path) -> std::io::Result<AuthStore> {
    match read_auth_json(auth_file) {
        Ok(map) => Ok(map),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(AuthStore::new()),
        Err(e) => Err(e),
    }
}

/// Best-effort backup of a corrupt (unparseable) auth.json.
///
/// If the file exists and `read_auth_json` fails with `InvalidData`,
/// it is renamed to `auth.json.corrupt.<millis>` (sibling in the same
/// directory) and the backup path is returned. Used before recovery
/// writes so the original bytes are never silently lost.
///
/// Unsupported schema versions and other non-`InvalidData` errors are
/// **not** backed up here — fail-closed surfaces must not wipe them.
pub(crate) fn backup_corrupt_auth_file(path: &Path) -> Option<PathBuf> {
    if !path.exists() {
        return None;
    }
    match read_auth_json(path) {
        Ok(_) => return None,
        Err(e) if e.kind() == std::io::ErrorKind::InvalidData => {}
        Err(_) => return None,
    }

    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();

    let file_name = path
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "auth.json".to_string());

    let backup_name = format!("{}.corrupt.{}", file_name, ts);
    let backup = path.with_file_name(backup_name);

    match std::fs::rename(path, &backup) {
        Ok(()) => {
            tracing::warn!(
                original = %path.display(),
                backup = %backup.display(),
                "auth: backed up corrupt auth.json before recovery write"
            );
            // Must reach unified.jsonl: the tracing line above is invisible
            // in production captures, and this is the only record of both
            // the corruption and where the original bytes went.
            xai_grok_telemetry::unified_log::error(
                "auth: corrupt auth.json backed up",
                None,
                Some(serde_json::json!({
                    "original": path.display().to_string(),
                    "backup": backup.display().to_string(),
                })),
            );
            Some(backup)
        }
        Err(e) => {
            tracing::warn!(error = %e, "auth: failed to rename corrupt auth.json for backup");
            xai_grok_telemetry::unified_log::error(
                "auth: corrupt auth.json backup failed",
                None,
                Some(serde_json::json!({
                    "original": path.display().to_string(),
                    "error": e.to_string(),
                })),
            );
            None
        }
    }
}

/// Read auth.json for an upcoming write, with recovery for corrupt files.
///
/// - Missing/empty → empty map (safe to write fresh)
/// - Valid JSON → parsed map (xAI slot)
/// - Non-empty corrupt JSON → backs up to `auth.json.corrupt.<millis>`,
///   then returns empty map so the caller can write the new credential.
/// - Unsupported schema version → error (**not** empty recovery)
///
/// Other I/O errors (PermissionDenied, etc.) are still returned as errors.
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

/// Document reader for merge-safe writes.
///
/// - Missing → empty document (safe to write fresh)
/// - Valid → parsed document
/// - Unsupported version → error (fail closed)
/// - Corrupt JSON → backup the raw file, then **preserve any still-parseable
///   sibling provider maps** from the raw JSON. Only wipe to empty when no
///   sibling provider payload can be recovered (legacy single-store recovery).
///   This prevents a torn/corrupt multi-slot document from deleting live
///   Codex (or other) credentials during an xAI-only write.
fn read_auth_document_for_write(auth_file: &Path) -> std::io::Result<AuthDocument> {
    match read_auth_document(auth_file) {
        Ok(doc) => Ok(doc),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(AuthDocument::default()),
        Err(e) if e.kind() == std::io::ErrorKind::InvalidData => {
            let preserved = preserve_sibling_providers_from_corrupt(auth_file);
            let _ = backup_corrupt_auth_file(auth_file);
            Ok(preserved.unwrap_or_default())
        }
        Err(e) => Err(e),
    }
}

/// Best-effort recovery of parseable `providers.*` maps from a file that
/// failed typed document deserialize. Returns `None` when nothing useful
/// can be recovered (caller treats as empty).
fn preserve_sibling_providers_from_corrupt(auth_file: &Path) -> Option<AuthDocument> {
    let bytes = std::fs::read(auth_file).ok()?;
    let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
    let providers = value.get("providers")?.as_object()?;
    let mut doc = AuthDocument::default();
    if let Some(v) = value.get("version").and_then(|v| v.as_u64()) {
        // Cap at AUTH_DOCUMENT_VERSION for rewrite; unsupported versions
        // never reach this path (they fail closed before recovery).
        if v <= u64::from(AUTH_DOCUMENT_VERSION) {
            doc.version = Some(v as u32);
        }
    }
    for (key, raw) in providers {
        if let Ok(store) = serde_json::from_value::<AuthStore>(raw.clone())
            && !store.is_empty()
        {
            doc.providers.insert(key.clone(), store);
        }
    }
    if doc.providers.is_empty() {
        None
    } else {
        Some(doc)
    }
}

// ── Dual mutation APIs ────────────────────────────────────────────────

/// Acquiring full-document RMW: exclusive `auth.json.lock`, then the same
/// body as [`mutate_auth_document_with_lock`]. For unlocked callers
/// (`write_auth_json`, enrichment-after-timeout, tests, Plan 02 API key).
pub(crate) fn mutate_auth_document<F>(auth_file: &Path, f: F) -> std::io::Result<()>
where
    F: FnOnce(&mut AuthDocument) -> std::io::Result<()>,
{
    let lock = crate::auth::manager::lock::lock_auth_file_blocking(auth_file)?;
    mutate_auth_document_with_lock(auth_file, &lock, f)
    // lock dropped here
}

/// Guard-held full-document RMW: **never** opens/re-acquires `auth.json.lock`.
/// Requires `lock.still_live` before read and again immediately before
/// persist. For callers that already hold the advisory lock (manager
/// cleanup, scope removal, refresh persist, enrichment-with-lock).
pub(crate) fn mutate_auth_document_with_lock<F>(
    auth_file: &Path,
    lock: &AuthFileLock,
    f: F,
) -> std::io::Result<()>
where
    F: FnOnce(&mut AuthDocument) -> std::io::Result<()>,
{
    if !lock.still_live(auth_file) {
        return Err(lock_not_live_error());
    }
    let mut doc = read_auth_document_for_write(auth_file)?;
    f(&mut doc)?;
    if !lock.still_live(auth_file) {
        return Err(lock_not_live_error());
    }
    write_auth_document(auth_file, &doc)
}

/// Replace the xAI slot only (acquiring lock). Preserves sibling providers;
/// sets `version` to [`AUTH_DOCUMENT_VERSION`]. Does not insert empty codex.
pub(super) fn write_auth_json(auth_file: &Path, auth_store: &AuthStore) -> std::io::Result<()> {
    mutate_auth_document(auth_file, |doc| {
        apply_xai_slot(doc, auth_store);
        Ok(())
    })
}

/// Replace the xAI slot only under a held [`AuthFileLock`] (never re-acquires).
pub(crate) fn write_auth_json_with_lock(
    auth_file: &Path,
    lock: &AuthFileLock,
    auth_store: &AuthStore,
) -> std::io::Result<()> {
    mutate_auth_document_with_lock(auth_file, lock, |doc| {
        apply_xai_slot(doc, auth_store);
        Ok(())
    })
}

fn apply_xai_slot(doc: &mut AuthDocument, auth_store: &AuthStore) {
    doc.providers
        .insert(PROVIDER_XAI.to_owned(), auth_store.clone());
    doc.version = Some(AUTH_DOCUMENT_VERSION);
    // Do NOT insert empty PROVIDER_CODEX by default; preserve if present.
}

/// Mutate the xAI slot under an acquired lock, pruning `auth.json` only when
/// every provider slot is empty.
pub(crate) fn mutate_xai_store_or_prune<F>(
    auth_file: &Path,
    f: F,
) -> std::io::Result<XaiStoreMutation>
where
    F: FnOnce(&mut AuthStore),
{
    let lock = crate::auth::manager::lock::lock_auth_file_blocking(auth_file)?;
    mutate_xai_store_or_prune_with_lock(auth_file, &lock, f)
}

/// Guard-held form of [`mutate_xai_store_or_prune`].
///
/// This never re-acquires `auth.json.lock`; callers must pass the live guard
/// they already hold.
pub(crate) fn mutate_xai_store_or_prune_with_lock<F>(
    auth_file: &Path,
    lock: &AuthFileLock,
    f: F,
) -> std::io::Result<XaiStoreMutation>
where
    F: FnOnce(&mut AuthStore),
{
    if !lock.still_live(auth_file) {
        return Err(lock_not_live_error());
    }

    let mut doc = read_auth_document_for_write(auth_file)?;
    let file_exists = auth_file.try_exists()?;
    let mut xai = doc.providers.remove(PROVIDER_XAI).unwrap_or_default();
    f(&mut xai);
    if !xai.is_empty() {
        doc.providers.insert(PROVIDER_XAI.to_owned(), xai);
    }
    doc.version = Some(AUTH_DOCUMENT_VERSION);

    if !lock.still_live(auth_file) {
        return Err(lock_not_live_error());
    }
    persist_document_or_prune(auth_file, &doc, file_exists, remove_auth_file)
}

fn remove_auth_file(path: &Path) -> std::io::Result<()> {
    std::fs::remove_file(path)
}

fn persist_document_or_prune(
    auth_file: &Path,
    doc: &AuthDocument,
    file_exists: bool,
    remove: fn(&Path) -> std::io::Result<()>,
) -> std::io::Result<XaiStoreMutation> {
    if doc.providers.values().all(AuthStore::is_empty) {
        if !file_exists {
            return Ok(XaiStoreMutation::Unchanged);
        }
        remove(auth_file)?;
        return Ok(XaiStoreMutation::FileDeleted);
    }

    write_auth_document(auth_file, doc)?;
    Ok(XaiStoreMutation::DocumentWritten)
}

// ── Low-level nested serialize (no lock) ──────────────────────────────

/// Persist `auth.json` as nested [`AuthDocument`], preferring a crash-safe
/// atomic write but falling back to a non-atomic in-place write when the
/// disk is full.
///
/// The atomic path (temp + rename) needs free space >= the file size,
/// because the old file and a full temp copy coexist until the rename. On a
/// nearly-full disk that temp copy can fail with `StorageFull` (ENOSPC)
/// even though the credentials themselves are tiny. When that happens we
/// retry with an in-place truncate+write, which only needs the freed blocks
/// of the old file — far less than the temp-copy approach.
///
/// The in-place path is non-atomic, with two accepted trade-offs:
/// - If the in-place write itself fails (e.g. a concurrent process grabs the
///   just-freed blocks, or a crash mid-write), the prior bytes are restored
///   best-effort so a torn/empty file never *replaces* the previous on-disk
///   credential — on-disk state ends up no worse than before the attempt.
/// - Unlocked concurrent readers can still observe a torn (partial) file
///   during the brief write window; a partial file is healed on the next
///   read via [`read_auth_json_or_empty_recovering_corrupt`] (backup +
///   relogin). This window is inherent to any sub-1×-free single-file
///   replace and is preferable to persisting nothing at all, which would
///   leave every concurrent process with a stale, already-revoked token.
fn write_auth_document(auth_file: &Path, doc: &AuthDocument) -> std::io::Result<()> {
    write_auth_document_with(auth_file, doc, write_auth_document_atomic)
}

/// Dispatch helper: run `atomic`, and on `StorageFull` fall back to an
/// in-place write. Split out (with `atomic` injectable) so the disk-full
/// fallback is unit-testable without an actually-full filesystem.
fn write_auth_document_with(
    auth_file: &Path,
    doc: &AuthDocument,
    atomic: fn(&Path, &AuthDocument) -> std::io::Result<()>,
) -> std::io::Result<()> {
    match atomic(auth_file, doc) {
        Err(e) if e.kind() == std::io::ErrorKind::StorageFull => {
            tracing::warn!(
                path = %auth_file.display(),
                "auth: disk full during atomic write, falling back to in-place write"
            );
            // Must reach unified.jsonl: a silent in-memory-only credential
            // (the prior behavior) leaves sibling processes with a stale
            // refresh token and no record of why. Surface it loudly.
            xai_grok_telemetry::unified_log::warn(
                "auth: disk full, falling back to non-atomic in-place write",
                None,
                Some(serde_json::json!({
                    "path": auth_file.display().to_string(),
                })),
            );
            write_auth_document_in_place(auth_file, doc)
        }
        other => other,
    }
}

/// Serialize `doc` to `path` (truncate + rewrite), owner-only (0o600)
/// and `fsync`'d. Shared core of the atomic path (which targets the temp
/// file) and the in-place fallback (which targets `auth.json` directly).
///
/// Uses streaming `to_writer_pretty` through a `BufWriter` to avoid
/// allocating the entire JSON string in memory — eliminates OOM risk under
/// severe memory pressure.
///
/// **No lock acquisition** — only the dual mutators gate production RMW.
fn write_document_to(path: &Path, doc: &AuthDocument) -> std::io::Result<()> {
    use crate::util::secure_file::open_secure_file;

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let file = open_secure_file(path)?;
    let mut writer = std::io::BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, doc)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    writer.flush()?;
    writer
        .into_inner()
        .map_err(|e| e.into_error())?
        .sync_all()?;
    #[cfg(windows)]
    {
        crate::util::secure_file::set_windows_secure_permissions(path)?;
    }
    Ok(())
}

/// Atomic write: tmp + rename. Unix `rename(2)` replaces atomically;
/// Windows `rename` requires removing the target first.
fn write_auth_document_atomic(auth_file: &Path, doc: &AuthDocument) -> std::io::Result<()> {
    let tmp = auth_file.with_extension(format!("json.{}.tmp", std::process::id()));
    write_document_to(&tmp, doc)?;
    #[cfg(windows)]
    {
        let _ = std::fs::remove_file(auth_file);
    }
    std::fs::rename(&tmp, auth_file)?;
    Ok(())
}

/// Non-atomic fallback: truncate and rewrite `auth.json` in place.
///
/// Used only when [`write_auth_document_atomic`] fails with `StorageFull`.
/// Opening with truncation first frees the old content's blocks before the
/// new bytes are written, so this needs only the file size in free space
/// rather than the temp-copy approach's file-size-of-headroom.
///
/// Truncation is destructive, so the prior bytes are snapshotted first and
/// restored best-effort if the rewrite fails partway — a failed fallback
/// must not leave an empty/torn file where a parseable (if stale) credential
/// used to be. A partial file that survives (because even the restore failed)
/// is healed on the next read via [`read_auth_json_or_empty_recovering_corrupt`].
fn write_auth_document_in_place(auth_file: &Path, doc: &AuthDocument) -> std::io::Result<()> {
    write_auth_document_in_place_with(auth_file, doc, write_document_to)
}

/// Inner of [`write_auth_document_in_place`] with `write` injectable so the
/// rollback-on-failure path is unit-testable without an actually-full disk.
fn write_auth_document_in_place_with(
    auth_file: &Path,
    doc: &AuthDocument,
    write: fn(&Path, &AuthDocument) -> std::io::Result<()>,
) -> std::io::Result<()> {
    // Snapshot the prior bytes so a torn/empty write can be rolled back to
    // the previous on-disk credential. `None` when the file is absent.
    let prior = std::fs::read(auth_file).ok();
    match write(auth_file, doc) {
        Ok(()) => Ok(()),
        Err(e) => {
            if let Some(prior) = prior
                && let Err(restore_err) = restore_prior_bytes(auth_file, &prior)
            {
                tracing::warn!(
                    error = %restore_err,
                    "auth: failed to restore prior auth.json after in-place write failure"
                );
            }
            Err(e)
        }
    }
}

/// Best-effort rollback: rewrite `bytes` (owner-only, `fsync`'d) after a
/// failed in-place write so a torn/empty file does not replace the prior
/// credential.
fn restore_prior_bytes(auth_file: &Path, bytes: &[u8]) -> std::io::Result<()> {
    use crate::util::secure_file::open_secure_file;

    let mut file = open_secure_file(auth_file)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    #[cfg(windows)]
    {
        crate::util::secure_file::set_windows_secure_permissions(auth_file)?;
    }
    Ok(())
}

// ── Token / API-key helpers (xAI-slot adapters) ───────────────────────

/// Read a single auth token from `auth.json` by scope key.
/// Falls back to the legacy `https://accounts.x.ai/sign-in` scope key
/// when the requested scope is not found (devbox auth.json migration).
pub fn read_token_by_scope(grok_home: &Path, scope: &str) -> anyhow::Result<String> {
    let path = grok_home.join("auth.json");
    let store =
        read_auth_json(&path).map_err(|_| anyhow::anyhow!("Not logged in. Run `grok login`."))?;
    lookup_auth(&store, scope).map(|a| a.key).ok_or_else(|| {
        anyhow::anyhow!("Your auth token is invalid. Run `grok login` to re-authenticate.")
    })
}

/// Read the API key from the `xai::api_key` scope in auth.json.
pub fn read_api_key(grok_home: &Path) -> Option<String> {
    let path = grok_home.join("auth.json");
    let map = read_auth_json(&path).ok()?;
    map.get(API_KEY_SCOPE).map(|a| a.key.clone())
}

/// Store a plain API key in auth.json under the `xai::api_key` scope.
///
/// Uses the corrupt-recovery reader so a malformed auth.json (e.g. from a
/// previous crash) can be healed when the user sets an API key.
pub fn store_api_key(grok_home: &Path, api_key: &str) -> std::io::Result<()> {
    let path = grok_home.join("auth.json");
    mutate_auth_document(&path, |doc| {
        doc.providers
            .entry(PROVIDER_XAI.to_owned())
            .or_default()
            .insert(
                API_KEY_SCOPE.to_owned(),
                GrokAuth {
                    key: api_key.to_owned(),
                    auth_mode: AuthMode::ApiKey,
                    ..Default::default()
                },
            );
        doc.version = Some(AUTH_DOCUMENT_VERSION);
        Ok(())
    })
}

/// Remove the `xai::api_key` scope from auth.json.
pub fn clear_api_key(grok_home: &Path) -> std::io::Result<()> {
    let path = grok_home.join("auth.json");
    mutate_xai_store_or_prune(&path, |xai| {
        xai.remove(API_KEY_SCOPE);
    })?;
    Ok(())
}

// ── Test helpers ──────────────────────────────────────────────────────

/// Write a nested multi-slot fixture without going through production
/// merge (used to seed sibling slots for isolation tests).
#[cfg(test)]
pub(crate) fn write_fixture_auth_document(
    path: &Path,
    xai: AuthStore,
    codex: Option<AuthStore>,
) -> std::io::Result<()> {
    let mut providers = std::collections::BTreeMap::new();
    providers.insert(PROVIDER_XAI.to_owned(), xai);
    if let Some(codex) = codex {
        providers.insert(PROVIDER_CODEX.to_owned(), codex);
    }
    let doc = AuthDocument {
        version: Some(AUTH_DOCUMENT_VERSION),
        providers,
    };
    write_auth_document(path, &doc)
}

#[cfg(test)]
mod write_fallback_tests {
    use super::*;

    fn sample_xai_store() -> AuthStore {
        let mut map = AuthStore::new();
        map.insert(
            API_KEY_SCOPE.to_owned(),
            GrokAuth {
                key: "secret-key".to_owned(),
                auth_mode: AuthMode::ApiKey,
                ..Default::default()
            },
        );
        map
    }

    fn sample_doc() -> AuthDocument {
        let mut doc = AuthDocument {
            version: Some(AUTH_DOCUMENT_VERSION),
            providers: Default::default(),
        };
        doc.providers
            .insert(PROVIDER_XAI.to_owned(), sample_xai_store());
        doc
    }

    fn read_key(path: &Path) -> Option<String> {
        read_auth_json(path)
            .ok()
            .and_then(|m| m.get(API_KEY_SCOPE).map(|a| a.key.clone()))
    }

    fn fake_storage_full(_: &Path, _: &AuthDocument) -> std::io::Result<()> {
        Err(std::io::Error::from(std::io::ErrorKind::StorageFull))
    }

    fn fake_permission_denied(_: &Path, _: &AuthDocument) -> std::io::Result<()> {
        Err(std::io::Error::from(std::io::ErrorKind::PermissionDenied))
    }

    /// Simulates an in-place write that truncates the file (destroying the
    /// old content, as `open_secure_file` does) and then fails partway — the
    /// torn-write case the rollback must recover from.
    fn fake_truncate_then_fail(path: &Path, _: &AuthDocument) -> std::io::Result<()> {
        crate::util::secure_file::open_secure_file(path)?; // truncates to 0 bytes
        Err(std::io::Error::from(std::io::ErrorKind::StorageFull))
    }

    #[test]
    fn in_place_write_roundtrips() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("auth.json");
        write_auth_document_in_place(&path, &sample_doc()).unwrap();
        assert_eq!(read_key(&path).as_deref(), Some("secret-key"));
    }

    #[cfg(unix)]
    #[test]
    fn in_place_write_is_owner_only() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("auth.json");
        write_auth_document_in_place(&path, &sample_doc()).unwrap();
        let mode = std::fs::metadata(&path).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600, "in-place write must stay 0o600");
    }

    /// A `StorageFull` (ENOSPC) failure on the atomic path must fall back to
    /// the in-place write so the credential still lands on disk.
    #[test]
    fn falls_back_to_in_place_on_storage_full() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("auth.json");
        write_auth_document_with(&path, &sample_doc(), fake_storage_full).unwrap();
        assert_eq!(
            read_key(&path).as_deref(),
            Some("secret-key"),
            "disk-full atomic write must fall back to a successful in-place write"
        );
    }

    /// Non-ENOSPC errors must propagate unchanged and must NOT trigger the
    /// in-place fallback (e.g. a permission error should not write the file).
    #[test]
    fn propagates_non_storage_full_errors() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("auth.json");
        let err =
            write_auth_document_with(&path, &sample_doc(), fake_permission_denied).unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::PermissionDenied);
        assert!(!path.exists(), "non-ENOSPC failure must not write the file");
    }

    /// The normal (real atomic + acquiring merge) path still works end to end.
    #[test]
    fn atomic_write_roundtrips() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("auth.json");
        write_auth_json(&path, &sample_xai_store()).unwrap();
        assert_eq!(read_key(&path).as_deref(), Some("secret-key"));
        // Nested on-disk shape after production write.
        let raw: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert!(raw.get("providers").is_some());
        assert_eq!(raw.get("version").and_then(|v| v.as_u64()), Some(1));
    }

    /// A fallback write that truncates then fails must roll back to the prior
    /// bytes instead of leaving an empty/torn file — otherwise a second
    /// disk-full failure would destroy a previously-valid credential.
    #[test]
    fn in_place_restores_prior_bytes_on_failure() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("auth.json");
        // Seed a valid prior credential.
        write_auth_document_in_place(&path, &sample_doc()).unwrap();
        assert_eq!(read_key(&path).as_deref(), Some("secret-key"));

        let mut replacement_store = AuthStore::new();
        replacement_store.insert(
            API_KEY_SCOPE.to_owned(),
            GrokAuth {
                key: "replacement-key".to_owned(),
                auth_mode: AuthMode::ApiKey,
                ..Default::default()
            },
        );
        let mut replacement = AuthDocument {
            version: Some(AUTH_DOCUMENT_VERSION),
            providers: Default::default(),
        };
        replacement
            .providers
            .insert(PROVIDER_XAI.to_owned(), replacement_store);
        let err = write_auth_document_in_place_with(&path, &replacement, fake_truncate_then_fail)
            .unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::StorageFull);
        assert_eq!(
            read_key(&path).as_deref(),
            Some("secret-key"),
            "a failed in-place write must restore the prior credential, not leave an empty file"
        );
    }

    /// Rollback after a failed write must keep the file owner-only (0o600).
    #[cfg(unix)]
    #[test]
    fn in_place_restore_is_owner_only() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("auth.json");
        write_auth_document_in_place(&path, &sample_doc()).unwrap();
        let _ = write_auth_document_in_place_with(&path, &sample_doc(), fake_truncate_then_fail);
        let mode = std::fs::metadata(&path).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600, "restored file must stay 0o600");
    }
}

#[cfg(test)]
mod multi_slot_tests {
    use super::*;
    use crate::auth::manager::lock::try_lock_auth_file_nonblocking;
    use std::sync::{Arc, Barrier};
    use std::time::Duration;

    fn xai_store(key: &str) -> AuthStore {
        let mut map = AuthStore::new();
        map.insert(
            API_KEY_SCOPE.to_owned(),
            GrokAuth {
                key: key.to_owned(),
                auth_mode: AuthMode::ApiKey,
                ..GrokAuth::test_default()
            },
        );
        map
    }

    fn codex_store(key: &str) -> AuthStore {
        let mut map = AuthStore::new();
        map.insert(
            "codex::fixture".to_owned(),
            GrokAuth {
                key: key.to_owned(),
                auth_mode: AuthMode::ApiKey,
                ..GrokAuth::test_default()
            },
        );
        map
    }

    fn raw_json(path: &Path) -> serde_json::Value {
        let s = std::fs::read_to_string(path).expect("read raw auth.json");
        serde_json::from_str(&s).expect("parse raw auth.json")
    }

    fn fake_remove_denied(_: &Path) -> std::io::Result<()> {
        Err(std::io::Error::from(std::io::ErrorKind::PermissionDenied))
    }

    #[test]
    fn store_api_key_only_touches_xai() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("auth.json");
        write_fixture_auth_document(
            &path,
            AuthStore::new(),
            Some(codex_store("codex-survives-store")),
        )
        .unwrap();

        store_api_key(dir.path(), "new-api-key").unwrap();

        assert_eq!(read_api_key(dir.path()).as_deref(), Some("new-api-key"));
        let raw = raw_json(&path);
        assert_eq!(raw.get("version").and_then(|v| v.as_u64()), Some(1));
        assert_eq!(
            raw.pointer("/providers/codex/codex::fixture/key")
                .and_then(|v| v.as_str()),
            Some("codex-survives-store")
        );
        assert_eq!(
            raw.pointer(&format!("/providers/{PROVIDER_XAI}/{API_KEY_SCOPE}/key"))
                .and_then(|v| v.as_str()),
            Some("new-api-key")
        );
    }

    #[test]
    fn clear_api_key_preserves_nonempty_codex_slot() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("auth.json");
        write_fixture_auth_document(
            &path,
            xai_store("remove-me"),
            Some(codex_store("codex-survives-clear")),
        )
        .unwrap();

        clear_api_key(dir.path()).unwrap();

        assert!(path.exists(), "codex scopes require auth.json to remain");
        assert_eq!(read_api_key(dir.path()), None);
        let raw = raw_json(&path);
        assert!(
            raw.pointer(&format!("/providers/{PROVIDER_XAI}/{API_KEY_SCOPE}"))
                .is_none()
        );
        assert_eq!(
            raw.pointer("/providers/codex/codex::fixture/key")
                .and_then(|v| v.as_str()),
            Some("codex-survives-clear")
        );
    }

    #[test]
    fn clear_api_key_deletes_file_when_all_providers_are_empty() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("auth.json");
        write_fixture_auth_document(&path, xai_store("remove-me"), None).unwrap();

        clear_api_key(dir.path()).unwrap();

        assert!(!path.exists(), "last provider scope should prune auth.json");
    }

    #[test]
    fn clear_api_key_is_idempotent_when_file_is_missing() {
        let dir = tempfile::tempdir().unwrap();
        clear_api_key(dir.path()).unwrap();
        assert!(!dir.path().join("auth.json").exists());
    }

    #[test]
    fn file_deleted_requires_successful_remove() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("auth.json");
        std::fs::write(&path, b"seed").unwrap();
        let doc = AuthDocument {
            version: Some(AUTH_DOCUMENT_VERSION),
            providers: Default::default(),
        };

        let err = persist_document_or_prune(&path, &doc, true, fake_remove_denied).unwrap_err();

        assert_eq!(err.kind(), std::io::ErrorKind::PermissionDenied);
        assert!(
            path.exists(),
            "a failed remove must not report or simulate FileDeleted"
        );
    }

    #[test]
    fn held_lock_prune_preserves_codex_without_reacquiring() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("auth.json");
        write_fixture_auth_document(
            &path,
            xai_store("remove-me"),
            Some(codex_store("held-codex")),
        )
        .unwrap();
        let lock = try_lock_auth_file_nonblocking(&path).expect("held lock");
        let started = std::time::Instant::now();

        let outcome = mutate_xai_store_or_prune_with_lock(&path, &lock, AuthStore::clear).unwrap();

        assert_eq!(outcome, XaiStoreMutation::DocumentWritten);
        assert!(
            started.elapsed() < Duration::from_secs(2),
            "guard-held prune must not self-deadlock"
        );
        assert_eq!(
            raw_json(&path)
                .pointer("/providers/codex/codex::fixture/key")
                .and_then(|v| v.as_str()),
            Some("held-codex")
        );
    }

    #[test]
    fn legacy_flat_reads_as_xai_slot() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("auth.json");
        let flat = xai_store("legacy-flat-key");
        // Write raw flat (no providers key) — not nested fixture.
        {
            use crate::util::secure_file::open_secure_file;
            let file = open_secure_file(&path).unwrap();
            serde_json::to_writer_pretty(file, &flat).unwrap();
        }
        let store = read_auth_json(&path).unwrap();
        assert_eq!(
            store.get(API_KEY_SCOPE).map(|a| a.key.as_str()),
            Some("legacy-flat-key")
        );
        // Raw file still flat until a production write rewrites nested.
        let raw = raw_json(&path);
        assert!(raw.get("providers").is_none());
    }

    #[test]
    fn write_emits_nested_providers_xai() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("auth.json");
        write_auth_json(&path, &xai_store("nested-key")).unwrap();
        let raw = raw_json(&path);
        assert!(raw.get("providers").is_some(), "must have providers");
        assert_eq!(raw.get("version").and_then(|v| v.as_u64()), Some(1));
        let xai_key = raw
            .pointer(&format!("/providers/{PROVIDER_XAI}/{API_KEY_SCOPE}/key"))
            .and_then(|v| v.as_str());
        assert_eq!(xai_key, Some("nested-key"));
        assert_eq!(
            read_auth_json(&path)
                .unwrap()
                .get(API_KEY_SCOPE)
                .map(|a| a.key.as_str()),
            Some("nested-key")
        );
    }

    #[test]
    fn seeded_codex_slot_survives_xai_write() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("auth.json");
        let codex = codex_store("codex-seed");
        write_fixture_auth_document(&path, xai_store("old-xai"), Some(codex.clone())).unwrap();

        write_auth_json(&path, &xai_store("new-xai")).unwrap();

        let raw = raw_json(&path);
        let codex_key = raw
            .pointer("/providers/codex/codex::fixture/key")
            .and_then(|v| v.as_str());
        assert_eq!(codex_key, Some("codex-seed"));
        let xai_key = raw
            .pointer(&format!("/providers/{PROVIDER_XAI}/{API_KEY_SCOPE}/key"))
            .and_then(|v| v.as_str());
        assert_eq!(xai_key, Some("new-xai"));
        // Round-trip fixture codex payload equality via document read.
        let doc = read_auth_document(&path).unwrap();
        let codex_on_disk = doc
            .providers
            .get(PROVIDER_CODEX)
            .expect("codex slot present");
        assert_eq!(
            codex_on_disk.get("codex::fixture").map(|a| a.key.as_str()),
            Some("codex-seed")
        );
        let _ = &codex; // seeded payload used above
    }

    #[test]
    fn concurrent_xai_and_codex_mutations_preserve_both() {
        let dir = tempfile::tempdir().unwrap();
        let path = Arc::new(dir.path().join("auth.json"));
        // Seed empty nested doc so both mutators merge into the same file.
        write_fixture_auth_document(&path, AuthStore::new(), None).unwrap();

        let barrier = Arc::new(Barrier::new(2));
        let path_x = Arc::clone(&path);
        let barrier_x = Arc::clone(&barrier);
        let t_xai = std::thread::spawn(move || {
            barrier_x.wait();
            mutate_auth_document(&path_x, |doc| {
                doc.providers
                    .insert(PROVIDER_XAI.to_owned(), xai_store("concurrent-xai"));
                doc.version = Some(AUTH_DOCUMENT_VERSION);
                Ok(())
            })
            .expect("xai mutate");
        });
        let path_c = Arc::clone(&path);
        let barrier_c = Arc::clone(&barrier);
        let t_codex = std::thread::spawn(move || {
            barrier_c.wait();
            mutate_auth_document(&path_c, |doc| {
                doc.providers
                    .insert(PROVIDER_CODEX.to_owned(), codex_store("concurrent-codex"));
                doc.version = Some(AUTH_DOCUMENT_VERSION);
                Ok(())
            })
            .expect("codex mutate");
        });
        t_xai.join().expect("xai thread");
        t_codex.join().expect("codex thread");

        let raw = raw_json(&path);
        assert_eq!(
            raw.pointer(&format!("/providers/{PROVIDER_XAI}/{API_KEY_SCOPE}/key"))
                .and_then(|v| v.as_str()),
            Some("concurrent-xai")
        );
        assert_eq!(
            raw.pointer("/providers/codex/codex::fixture/key")
                .and_then(|v| v.as_str()),
            Some("concurrent-codex")
        );
    }

    #[test]
    fn already_locked_mutate_does_not_self_deadlock() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("auth.json");
        write_fixture_auth_document(&path, xai_store("pre-xai"), Some(codex_store("held-codex")))
            .unwrap();

        let lock = try_lock_auth_file_nonblocking(&path).expect("acquire lock");
        let path_owned = path.clone();
        let handle = std::thread::spawn(move || {
            // Run mutator on this thread so a hang is join-detectable.
            mutate_auth_document_with_lock(&path_owned, &lock, |doc| {
                doc.providers
                    .insert(PROVIDER_XAI.to_owned(), xai_store("post-xai"));
                doc.version = Some(AUTH_DOCUMENT_VERSION);
                Ok(())
            })
        });
        // Wall-clock bound: self-deadlock would hang the join forever.
        let result = match handle.join() {
            Ok(r) => r,
            Err(_) => panic!("with-lock mutator thread panicked"),
        };
        // Use a short sleep+poll is unnecessary if join returned; assert ok.
        result.expect("with-lock mutate under held lock must succeed");

        let raw = raw_json(&path);
        assert_eq!(
            raw.pointer(&format!("/providers/{PROVIDER_XAI}/{API_KEY_SCOPE}/key"))
                .and_then(|v| v.as_str()),
            Some("post-xai")
        );
        assert_eq!(
            raw.pointer("/providers/codex/codex::fixture/key")
                .and_then(|v| v.as_str()),
            Some("held-codex")
        );
    }

    #[test]
    fn acquiring_and_with_lock_are_distinct() {
        // Surface: both APIs exist. Behavior: with-lock never opens a second
        // lock fd (succeeds under held lock); acquiring under held lock
        // cannot complete without blocking (modeled by non-blocking acquire
        // returning None for a second handle on the same process).
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("auth.json");
        write_fixture_auth_document(&path, xai_store("a"), Some(codex_store("b"))).unwrap();

        let lock = try_lock_auth_file_nonblocking(&path).expect("first lock");
        assert!(
            try_lock_auth_file_nonblocking(&path).is_none(),
            "second non-blocking acquire must fail while first holds the lock"
        );
        write_auth_json_with_lock(&path, &lock, &xai_store("via-with-lock")).unwrap();
        drop(lock);

        // Acquiring path works once lock is free.
        write_auth_json(&path, &xai_store("via-acquiring")).unwrap();
        let store = read_auth_json(&path).unwrap();
        assert_eq!(
            store.get(API_KEY_SCOPE).map(|a| a.key.as_str()),
            Some("via-acquiring")
        );
        let doc = read_auth_document(&path).unwrap();
        assert!(doc.providers.contains_key(PROVIDER_CODEX));
    }

    #[test]
    fn unsupported_version_fails_closed() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("auth.json");
        let mut providers = std::collections::BTreeMap::new();
        providers.insert(PROVIDER_XAI.to_owned(), xai_store("future"));
        let future = AuthDocument {
            version: Some(AUTH_DOCUMENT_VERSION + 99),
            providers,
        };
        // Bypass version gate on write: serialize raw nested with high version.
        {
            use crate::util::secure_file::open_secure_file;
            let file = open_secure_file(&path).unwrap();
            serde_json::to_writer_pretty(file, &future).unwrap();
        }

        let err = read_auth_json(&path).unwrap_err();
        assert_eq!(
            err.kind(),
            std::io::ErrorKind::Unsupported,
            "unsupported version must not map to InvalidData"
        );
        assert!(
            err.to_string().contains("unsupported"),
            "error should mention unsupported: {err}"
        );

        // Recovery path must NOT wipe / empty-recover unsupported schemas.
        let recover_err = read_auth_json_or_empty_recovering_corrupt(&path).unwrap_err();
        assert_eq!(recover_err.kind(), std::io::ErrorKind::Unsupported);
        assert!(
            path.exists(),
            "unsupported version must leave the file in place"
        );
        // No corrupt backup should appear.
        let siblings: Vec<_> = std::fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        assert!(
            !siblings.iter().any(|n| n.contains("corrupt")),
            "unsupported version must not trigger corrupt backup: {siblings:?}"
        );
    }

    #[test]
    fn nested_fixture_readable_via_read_auth_json() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("auth.json");
        write_fixture_auth_document(&path, xai_store("from-nested"), None).unwrap();
        let store = read_auth_json(&path).unwrap();
        assert_eq!(
            store.get(API_KEY_SCOPE).map(|a| a.key.as_str()),
            Some("from-nested")
        );
    }

    #[test]
    fn fixture_helper_writes_nested_with_optional_codex() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("auth.json");
        write_fixture_auth_document(&path, xai_store("x"), Some(codex_store("c"))).unwrap();
        let raw = raw_json(&path);
        assert_eq!(raw.get("version").and_then(|v| v.as_u64()), Some(1));
        assert!(raw.pointer("/providers/xai").is_some());
        assert!(raw.pointer("/providers/codex").is_some());
    }

    /// Smoke: with-lock path completes well under a generous wall clock.
    #[test]
    fn already_locked_mutate_completes_quickly() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("auth.json");
        write_fixture_auth_document(&path, xai_store("t0"), Some(codex_store("c0"))).unwrap();
        let lock = try_lock_auth_file_nonblocking(&path).expect("lock");
        let started = std::time::Instant::now();
        write_auth_json_with_lock(&path, &lock, &xai_store("t1")).unwrap();
        assert!(
            started.elapsed() < Duration::from_secs(2),
            "with-lock write hung: {:?}",
            started.elapsed()
        );
    }
}
