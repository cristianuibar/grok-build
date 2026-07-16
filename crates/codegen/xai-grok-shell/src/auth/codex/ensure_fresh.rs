//! Lock-held Codex ensure-fresh: re-read → IdP → guard-held persist.
//!
//! Production invoker is `SessionActor::reconstruct_full_config` (AUTH-05).
//! This module owns the outer refresh chain; [`super::refresh`] / CodexRefresher
//! stay pure (data-only).

use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use chrono::Duration;
use tokio::sync::Mutex as AsyncMutex;

use super::{codex_token_exchange, CODEX_AUTH_SCOPE, CodexRefreshResult};
use crate::auth::error::RefreshTokenFailedReason;
use crate::auth::model::{is_expired, is_expired_with_buffer, select_provider_access_token};
use crate::auth::refresh::{CodexRefresher, RefreshOutcome, RefreshReason, TokenRefresher};
use crate::auth::storage::{
    clear_provider_slot_with_lock, mutate_provider_store_or_prune_with_lock, read_provider_auth_store,
};
use crate::auth::{AuthProvider, GrokAuth, PROVIDER_CODEX};

/// Typed material returned into request construction after ensure_fresh.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodexAuthMaterial {
    pub bearer: String,
    pub account_id: Option<String>,
}

impl CodexAuthMaterial {
    pub fn from_auth(auth: &GrokAuth) -> Self {
        Self {
            bearer: auth.key.clone(),
            account_id: auth.organization_id.clone(),
        }
    }
}

/// Options for ensure_fresh (test hooks + production defaults).
#[derive(Debug, Clone, Default)]
pub struct EnsureFreshCodexOptions {
    /// Override token endpoint URL (mock IdP).
    pub token_url_override: Option<String>,
}

/// Process-wide test hooks so `reconstruct_full_config` can target a temp
/// `auth.json` + mock IdP without mutating `BUM_HOME` (OnceLock).
#[derive(Default)]
struct EnsureFreshTestHooks {
    auth_file: Option<PathBuf>,
    token_url: Option<String>,
    /// When set, skip HTTP and return this outcome from the IdP step.
    /// Counter increments once per simulated IdP spend.
    synthetic: Option<SyntheticIdp>,
}

#[derive(Clone)]
struct SyntheticIdp {
    outcome: SyntheticOutcome,
    call_counter: std::sync::Arc<std::sync::atomic::AtomicUsize>,
}

#[derive(Clone)]
enum SyntheticOutcome {
    Success(GrokAuth),
    Permanent,
    Transient,
}

static TEST_HOOKS: OnceLock<Mutex<EnsureFreshTestHooks>> = OnceLock::new();

fn test_hooks() -> &'static Mutex<EnsureFreshTestHooks> {
    TEST_HOOKS.get_or_init(|| Mutex::new(EnsureFreshTestHooks::default()))
}

/// Install test hooks for crate-local reconstruct / isolation tests.
///
/// Call [`clear_ensure_fresh_codex_test_hooks`] when done (or RAII via guard).
#[cfg(any(test, feature = "unstable"))]
pub fn set_ensure_fresh_codex_test_hooks(
    auth_file: Option<PathBuf>,
    token_url: Option<String>,
) {
    let mut g = test_hooks().lock().unwrap_or_else(|e| e.into_inner());
    g.auth_file = auth_file;
    g.token_url = token_url;
}

/// Public test surface for integration binary (same crate public API).
pub fn set_ensure_fresh_codex_test_hooks_public(
    auth_file: Option<PathBuf>,
    token_url: Option<String>,
) {
    let mut g = test_hooks().lock().unwrap_or_else(|e| e.into_inner());
    g.auth_file = auth_file;
    g.token_url = token_url;
}

/// Clear all ensure_fresh test hooks.
pub fn clear_ensure_fresh_codex_test_hooks() {
    let mut g = test_hooks().lock().unwrap_or_else(|e| e.into_inner());
    *g = EnsureFreshTestHooks::default();
}

/// Install a synthetic IdP that returns success with `fresh` and counts spends.
pub fn set_ensure_fresh_codex_synthetic_success(
    auth_file: PathBuf,
    fresh: GrokAuth,
    call_counter: std::sync::Arc<std::sync::atomic::AtomicUsize>,
) {
    let mut g = test_hooks().lock().unwrap_or_else(|e| e.into_inner());
    g.auth_file = Some(auth_file);
    g.token_url = None;
    g.synthetic = Some(SyntheticIdp {
        outcome: SyntheticOutcome::Success(fresh),
        call_counter,
    });
}

/// Synthetic permanent invalid_grant.
pub fn set_ensure_fresh_codex_synthetic_permanent(auth_file: PathBuf) {
    let mut g = test_hooks().lock().unwrap_or_else(|e| e.into_inner());
    g.auth_file = Some(auth_file);
    g.synthetic = Some(SyntheticIdp {
        outcome: SyntheticOutcome::Permanent,
        call_counter: std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0)),
    });
}

/// Synthetic transient failure.
pub fn set_ensure_fresh_codex_synthetic_transient(auth_file: PathBuf) {
    let mut g = test_hooks().lock().unwrap_or_else(|e| e.into_inner());
    g.auth_file = Some(auth_file);
    g.synthetic = Some(SyntheticIdp {
        outcome: SyntheticOutcome::Transient,
        call_counter: std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0)),
    });
}

fn take_hooks_snapshot() -> EnsureFreshTestHooks {
    let g = test_hooks().lock().unwrap_or_else(|e| e.into_inner());
    EnsureFreshTestHooks {
        auth_file: g.auth_file.clone(),
        token_url: g.token_url.clone(),
        synthetic: g.synthetic.clone(),
    }
}

/// In-process mutex so concurrent ensure_fresh tasks single-flight IdP spend.
fn refresh_mutex() -> &'static AsyncMutex<()> {
    static M: OnceLock<AsyncMutex<()>> = OnceLock::new();
    M.get_or_init(|| AsyncMutex::new(()))
}

fn product_auth_file() -> PathBuf {
    crate::util::grok_home::grok_home().join("auth.json")
}

/// Resolve auth.json path: explicit arg > test hook > product home.
fn resolve_auth_file(explicit: Option<&Path>) -> PathBuf {
    if let Some(p) = explicit {
        return p.to_path_buf();
    }
    let hooks = take_hooks_snapshot();
    hooks.auth_file.unwrap_or_else(product_auth_file)
}

/// Production entry: product home auth.json (or test hook override).
pub async fn ensure_fresh_codex_auth() -> Option<CodexAuthMaterial> {
    let path = resolve_auth_file(None);
    ensure_fresh_codex_auth_at(&path, EnsureFreshCodexOptions::default()).await
}

/// Public path-taking ensure_fresh for isolation / concurrency tests.
pub async fn ensure_fresh_codex_auth_at(
    auth_file: &Path,
    options: EnsureFreshCodexOptions,
) -> Option<CodexAuthMaterial> {
    let hooks = take_hooks_snapshot();
    let token_url = options
        .token_url_override
        .or(hooks.token_url)
        .filter(|s| !s.is_empty());
    // Synthetic IdP is path-scoped: only apply when the hook's auth_file matches
    // this call's path (avoids cross-fixture bleed if hooks are left installed).
    let synthetic = hooks.synthetic.filter(|_| {
        hooks
            .auth_file
            .as_ref()
            .is_some_and(|p| p.as_path() == auth_file)
    });

    let _in_process = refresh_mutex().lock().await;

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

    if !lock.still_live(auth_file) {
        tracing::warn!("codex ensure_fresh: lock not live after acquire");
        return None;
    }

    let store = match read_provider_auth_store(auth_file, PROVIDER_CODEX) {
        Ok(Some(s)) => s,
        Ok(None) => return None,
        Err(e) => {
            tracing::debug!(
                error_len = e.to_string().len(),
                "codex ensure_fresh: read providers.codex failed"
            );
            return None;
        }
    };

    let Some(auth) = select_provider_access_token(&store) else {
        return None;
    };

    // No selectable OIDC session (ApiKey-only slot is not session OAuth).
    if auth.auth_mode != crate::auth::AuthMode::Oidc {
        return None;
    }

    // Fresh enough (outside 5-min early-invalidation buffer) → no IdP.
    if !is_expired(&auth) {
        return Some(CodexAuthMaterial::from_auth(&auth));
    }

    // Near-expiry / expired but no refresh_token.
    if auth.refresh_token.as_ref().is_none_or(|t| t.is_empty()) {
        // Hard-unexpired access (buffer-only expiry) → keep material.
        if !is_expired_with_buffer(&auth, Duration::zero()) {
            return Some(CodexAuthMaterial::from_auth(&auth));
        }
        return None;
    }

    // Sibling adopt: if selected token is already non-expired we returned above.
    // Re-read is under lock so a sibling that finished between our mutex wait
    // and lock acquire is visible as a non-expired selection.

    // IdP refresh while holding lock.
    let outcome = if let Some(syn) = synthetic.as_ref() {
        syn.call_counter
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        match &syn.outcome {
            SyntheticOutcome::Success(fresh) => {
                // Preserve identity merge rules via exchange helper path:
                // treat `fresh` as the new auth (caller supplies complete GrokAuth).
                let mut merged = fresh.clone();
                if merged.refresh_token.is_none() {
                    merged.refresh_token = auth.refresh_token.clone();
                }
                if merged.organization_id.is_none() {
                    merged.organization_id = auth.organization_id.clone();
                }
                if merged.email.is_none() {
                    merged.email = auth.email.clone();
                }
                if merged.user_id.is_empty() {
                    merged.user_id = auth.user_id.clone();
                }
                if merged.oidc_issuer.is_none() {
                    merged.oidc_issuer = auth.oidc_issuer.clone();
                }
                if merged.oidc_client_id.is_none() {
                    merged.oidc_client_id = auth.oidc_client_id.clone();
                }
                RefreshOutcome::success(merged)
            }
            SyntheticOutcome::Permanent => RefreshOutcome::permanent(
                RefreshTokenFailedReason::RefreshTokenRejected,
                Some(auth.key.clone()),
            ),
            SyntheticOutcome::Transient => {
                RefreshOutcome::transient("synthetic codex refresh transient")
            }
        }
    } else if let Some(ref url) = token_url {
        // Prefer pure exchange with URL override (injectable mock server).
        let refresher = CodexRefresher::new(auth.clone()).with_token_url(url.clone());
        refresher.refresh(RefreshReason::PreRequest).await
    } else {
        // Product path: fixed issuer token endpoint.
        match codex_token_exchange(&auth, None).await {
            CodexRefreshResult::Success(a) => RefreshOutcome::Success(a),
            CodexRefreshResult::TerminalError { reason } => {
                RefreshOutcome::permanent(reason, Some(auth.key.clone()))
            }
            CodexRefreshResult::Failed => {
                RefreshOutcome::transient("Codex token refresh failed")
            }
        }
    };

    match outcome {
        RefreshOutcome::Success(new_auth) => {
            let to_store = (*new_auth).clone();
            let scope = CODEX_AUTH_SCOPE.to_owned();
            if let Err(e) = mutate_provider_store_or_prune_with_lock(
                auth_file,
                &lock,
                AuthProvider::Codex,
                move |store| {
                    store.insert(scope, to_store);
                },
            ) {
                tracing::warn!(
                    error_len = e.to_string().len(),
                    "codex ensure_fresh: failed to persist rotated tokens"
                );
                // Still return material — tokens are valid even if disk write failed.
            }
            // Best-effort invalidate prepare-time mtime snapshot.
            crate::agent::config::invalidate_codex_session_key_snapshot();
            Some(CodexAuthMaterial::from_auth(&new_auth))
        }
        RefreshOutcome::PermanentFailure { error, .. } => {
            tracing::warn!(
                reason = ?error.reason,
                "codex ensure_fresh: permanent failure — clearing codex slot only"
            );
            // NEVER call public clear_provider_slot (would reacquire lock → deadlock).
            if let Err(e) =
                clear_provider_slot_with_lock(auth_file, &lock, AuthProvider::Codex)
            {
                tracing::warn!(
                    error_len = e.to_string().len(),
                    "codex ensure_fresh: clear_provider_slot_with_lock failed"
                );
            }
            crate::agent::config::invalidate_codex_session_key_snapshot();
            None
        }
        RefreshOutcome::TransientFailure { message } => {
            tracing::warn!(
                message_len = message.len(),
                "codex ensure_fresh: transient failure"
            );
            if is_expired_with_buffer(&auth, Duration::zero()) {
                // Hard-expired: no usable credential.
                None
            } else {
                // Buffer-only expiry: keep old bearer.
                Some(CodexAuthMaterial::from_auth(&auth))
            }
        }
    }
}

/// Return IdP call counter from last synthetic install (for concurrent tests).
pub fn synthetic_idp_call_count(
    counter: &std::sync::Arc<std::sync::atomic::AtomicUsize>,
) -> usize {
    counter.load(std::sync::atomic::Ordering::SeqCst)
}
