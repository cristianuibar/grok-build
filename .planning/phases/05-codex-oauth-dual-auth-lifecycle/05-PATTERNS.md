# Phase 5: Codex OAuth & dual auth lifecycle - Pattern Map

**Mapped:** 2026-07-16
**Files analyzed:** 18
**Analogs found:** 18 / 18

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `shell/src/auth/storage.rs` | store | file-I/O | same file (`mutate_xai_store_or_prune`) | exact |
| `shell/src/auth/flow.rs` | service | request-response | same file (`run_cli_login` / `perform_logout`) | exact |
| `shell/src/auth/mod.rs` | config | transform | same file (module barrel + re-exports) | exact |
| `shell/src/auth/model.rs` | model | transform | same file (`GrokAuth`, `PROVIDER_*`) | exact |
| `shell/src/auth/manager.rs` | service | event-driven | same file (`remove_scope` / permanent fail) | exact |
| `shell/src/auth/codex/mod.rs` | service | request-response | `shell/src/auth/oidc/mod.rs` | role-match |
| `shell/src/auth/codex/browser.rs` | service | request-response | `shell/src/auth/oidc/login.rs` | exact |
| `shell/src/auth/codex/device.rs` | service | request-response | `shell/src/auth/device_code.rs` | exact |
| `shell/src/auth/codex/claims.rs` | utility | transform | `shell/src/auth/jwt.rs` + `oidc/protocol` peeks | role-match |
| `shell/src/auth/codex/refresh.rs` | service | request-response | `shell/src/auth/oidc/refresh.rs` | exact |
| `shell/src/auth/refresh/codex_refresher.rs` | service | request-response | `shell/src/auth/refresh/oidc_refresher.rs` | exact |
| `shell/src/auth/refresh/mod.rs` | config | transform | same file (`TokenRefresher` + `build_refresher`) | exact |
| `shell/src/auth/oidc/protocol.rs` | utility | transform | same file (`generate_pkce`) — visibility export | exact |
| `pager/src/app/cli.rs` | route | request-response | same file (`Command::Login` / `Logout`) | exact |
| `pager-bin/src/main.rs` | controller | request-response | same file (`Command::Login` match arm) | exact |
| `pager/src/slash/commands/logout.rs` | component | request-response | same file | exact |
| `pager/src/app/dispatch/auth.rs` | controller | event-driven | same file (`dispatch_logout`) | exact |
| `shell/src/agent/config.rs` | service | CRUD | same file (`snapshot_codex_session_key_from_auth_store`) | exact |
| `shell/tests/auth_codex_lifecycle.rs` | test | batch | `shell/tests/auth_multi_slot.rs` | role-match |
| `shell/tests/auth_multi_slot.rs` | test | batch | same file (extend isolation proofs) | exact |

Paths abbreviated: `shell` = `crates/codegen/xai-grok-shell`, `pager` = `crates/codegen/xai-grok-pager`, `pager-bin` = `crates/codegen/xai-grok-pager-bin`.

---

## Pattern Assignments

### `shell/src/auth/storage.rs` (store, file-I/O)

**Analog:** same file — generalize xAI-only writers to provider-parameterized RMW.

**Target API shape** (from RESEARCH; copy structure of `mutate_xai_store_or_prune`):

```rust
// Pattern to implement — mirror lines 481–521, parameterize PROVIDER_XAI
pub(crate) fn mutate_provider_store_or_prune(
    auth_file: &Path,
    provider: &str, // PROVIDER_XAI | PROVIDER_CODEX
    f: impl FnOnce(&mut AuthStore),
) -> std::io::Result<ProviderStoreMutation>;
```

**Core RMW pattern** (lines 479–521):
```rust
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
```

**Slot apply pattern** (lines 472–477) — do **not** use `write_auth_json` for Codex:
```rust
fn apply_xai_slot(doc: &mut AuthDocument, auth_store: &AuthStore) {
    doc.providers
        .insert(PROVIDER_XAI.to_owned(), auth_store.clone());
    doc.version = Some(AUTH_DOCUMENT_VERSION);
    // Do NOT insert empty PROVIDER_CODEX by default; preserve if present.
}
```

**Per-slot read pattern** (lines 203–248) — reuse for status + refresh:
```rust
pub fn read_provider_auth_store(
    path: &Path,
    provider: &str,
) -> Result<Option<AuthStore>, AuthStoreReadError> {
    match read_auth_document(path) {
        Ok(doc) => Ok(doc.providers.get(provider).cloned()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        // ... redacted diagnostics, fail-closed ...
    }
}
```

**Prune semantics** (lines 527–542) — clear one slot; delete file only when all empty:
```rust
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
```

**Anti-pattern:** Codex login must **not** call `write_auth_json` / `mutate_xai_store_or_prune` (xAI-only). Build `mutate_provider_store_or_prune` / `clear_provider_slot` on top of `mutate_auth_document*`.

---

### `shell/src/auth/flow.rs` (service, request-response)

**Analog:** same file — extend CLI login/logout; add status.

**CLI login entry** (lines 867–941) — add `provider` param; dispatch Codex vs xAI:
```rust
pub async fn run_cli_login(
    config: &crate::agent::config::Config,
    oauth: bool,
    device_auth: bool,
    devbox: bool,
) -> anyhow::Result<()> {
    let login_override = LoginTransportOverride::from_flags(oauth, device_auth);
    // ... device vs loopback; report_signed_in ...
    Ok(())
}
```

**Success copy** (lines 773–778) — update to include provider label per UI-SPEC:
```rust
fn report_signed_in(auth: &GrokAuth) {
    eprint!("\r\x1b[K");
    match auth.email {
        Some(ref email) => eprintln!("✓ Signed in as {email}"),
        None => eprintln!("✓ Signed in"),
    }
}
// Target: "✓ Signed in to {ProviderLabel} as {email}"
```

**Logout core** (lines 960–1006) — today xAI AuthManager only; Phase 5 needs provider-scoped clear:
```rust
pub fn perform_logout(
    auth_manager: &AuthManager,
    scope: Option<&str>,
) -> std::io::Result<LogoutResult> {
    // ...
    if was_logged_in {
        if let Some(scope) = scope {
            auth_manager.remove_scope(scope)?;
        } else {
            auth_manager.clear()?; // clears current xAI scope only
        }
    }
    Ok(LogoutResult { was_logged_in, email, api_key_still_set: ... })
}
```

**CLI logout formatting** (lines 1010–1030):
```rust
pub fn run_cli_logout(config: &crate::agent::config::Config) -> anyhow::Result<()> {
    let result = perform_logout(&auth_manager, None)?;
    if !result.was_logged_in {
        eprintln!("No cached session to log out of.");
        // ...
    }
    // Target: require --provider | --all; bare → usage fail-closed (UI-SPEC)
}
```

**New: auth status** — pure document inspect via `read_auth_document` + `select_provider_access_token`; greppable format from UI-SPEC (no tokens).

---

### `shell/src/auth/codex/mod.rs` (service, request-response)

**Analog:** `shell/src/auth/oidc/mod.rs` (lines 1–15)

**Module barrel pattern:**
```rust
//! OIDC authentication: protocol, login, and refresh submodules.

mod login;
pub(crate) mod protocol;
pub(crate) mod refresh;
#[cfg(test)]
mod test_helpers;

pub use login::{run_login_flow, run_login_flow_with_config};
pub(crate) use protocol::{ /* ... */ };
pub(crate) use refresh::{OidcRefreshResult, oidc_token_exchange};
```

**Wire into** `shell/src/auth/mod.rs` as `pub mod codex;` (or `pub(crate) mod codex`) alongside existing `pub mod oidc` / `pub mod device_code`.

**Constants** (from RESEARCH — implement as consts in mod.rs):
```rust
pub const CODEX_ISSUER: &str = "https://auth.openai.com";
pub const CODEX_CLIENT_ID: &str = "app_EMoamEEZ73f0CkXaXp7hrann";
pub const CODEX_PREFERRED_PORT: u16 = 1455;
pub const CODEX_FALLBACK_PORT: u16 = 1457;
// redirect: http://localhost:{port}/auth/callback  (localhost host string!)
// scopes: openid profile email offline_access api.connectors.read api.connectors.invoke
```

---

### `shell/src/auth/codex/browser.rs` (service, request-response)

**Analog:** `shell/src/auth/oidc/login.rs` — axum loopback PKCE; **ports/paths differ**.

**Imports + loopback server pattern** (lines 11–18, 385–441):
```rust
use axum::{
    Router,
    extract::{Query, State},
    http::{Method, StatusCode},
    response::Html,
    routing::get,
};
use tokio::net::TcpListener;

// xAI binds dynamic/random port + redirect http://127.0.0.1:{port}/callback
let listener = TcpListener::bind(("127.0.0.1", callback_port)).await?;
let redirect_uri = format!("http://127.0.0.1:{}/callback", port);

// CLI stderr copy pattern:
eprintln!("Signing in with {}...", provider_label);
eprintln!("Open this URL to sign in:");
eprintln!("  {}", auth_url);
```

**Codex deltas (must not copy xAI ports/paths blindly):**
| Concern | xAI OIDC | Codex (mirror openai/codex) |
|---------|----------|------------------------------|
| Bind | `127.0.0.1` dynamic port | `127.0.0.1:1455` then `1457` |
| Redirect URI host | `127.0.0.1` | **`localhost`** (string must be localhost) |
| Path | `/callback` | `/auth/callback` |
| Discovery | `.well-known` OIDC | **Fixed** paths on `auth.openai.com` |
| Persist | `AuthManager` xAI scope | `mutate_provider_store` → `providers.codex` |

**PKCE generation analog** — `oidc/protocol.rs` lines 333–345 (export or duplicate):
```rust
pub(super) struct Pkce {
    pub(super) code_verifier: String,
    pub(super) code_challenge: String,
}
pub(super) fn generate_pkce() -> Pkce {
    let random_bytes: [u8; 32] = rand::random();
    let code_verifier = URL_SAFE_NO_PAD.encode(random_bytes);
    let code_challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(code_verifier.as_bytes()));
    Pkce { code_verifier, code_challenge }
}
```

**Token exchange form analog** — `oidc/protocol.rs` lines 401–436:
```rust
pub(super) async fn exchange_code(
    token_endpoint: &str,
    code: &str,
    redirect_uri: &str,
    client_id: &str,
    code_verifier: &str,
) -> anyhow::Result<TokenResponse> {
    let resp = with_alpha_test_key(
        crate::http::shared_client()
            .post(token_endpoint)
            .form(&[
                ("grant_type", "authorization_code"),
                ("code", code),
                ("redirect_uri", redirect_uri),
                ("client_id", client_id),
                ("code_verifier", code_verifier),
            ])
            .timeout(std::time::Duration::from_secs(15)),
        token_endpoint,
    )
    .send()
    .await?;
    // ...
}
```

**State validation** — `protocol::validate_state` (line 567).

**Persist mapping** into `GrokAuth` under `providers.codex`:
- `key` = access_token
- `auth_mode` = `AuthMode::Oidc`
- `refresh_token`, `expires_at`, `oidc_issuer`, `oidc_client_id`
- `organization_id` = `chatgpt_account_id` (from id_token claims)
- `email` from claims when present

---

### `shell/src/auth/codex/device.rs` (service, request-response)

**Analog:** `shell/src/auth/device_code.rs`

**Two-phase API pattern** (lines 1–8, 81–92, 135, 213):
```rust
//! 1. `request_device_code()` -- POST to server, get code + URL
//! 2. `complete_device_code_login()` -- poll until approved, persist credentials

pub struct DeviceCode {
    pub verification_uri: String,
    pub verification_uri_complete: Option<String>,
    pub user_code: String,
    device_code: String,
    interval: i32,
    expires_in: i64,
}
```

**CLI display copy** (lines 366–392) — match UI-SPEC (already aligned):
```rust
eprintln!("To sign in, open this URL in your browser:");
eprintln!("  {}", display_uri);
// ...
eprintln!("  {}", device_code.user_code);
eprintln!(
    "\x1b[90mOnly continue with a code you requested. \
     Don't share it with anyone.\x1b[0m"
);
eprintln!("Waiting for authorization...");
```

**Codex device endpoints** (fixed, not xAI device grant):
| Step | Path |
|------|------|
| Usercode | `{issuer}/api/accounts/deviceauth/usercode` |
| Token poll | `{issuer}/api/accounts/deviceauth/token` |
| Verify URL | `{issuer}/codex/device` |
| Exchange redirect | `{issuer}/deviceauth/callback` |

Do **not** reuse `DEVICE_GRANT_TYPE` / xAI oauth2 device path — Codex deviceauth API differs from RFC 8628 shape used by xAI.

---

### `shell/src/auth/codex/claims.rs` (utility, transform)

**Analog:** `shell/src/auth/jwt.rs` + `oidc/protocol` claim peeks.

**JWT insecure decode pattern** (jwt.rs lines 11–21):
```rust
pub fn parse_jwt_expiration(token: &str) -> Option<DateTime<Utc>> {
    jsonwebtoken::dangerous::insecure_decode::<Claims>(token)
        .ok()
        .and_then(|data| data.claims.exp)
        .and_then(|ts| DateTime::from_timestamp(ts, 0))
}

pub fn is_jwt_expired_or_near(token: &str, threshold: Duration) -> bool {
    parse_jwt_expiration(token)
        .map(|exp| exp <= Utc::now() + threshold)
        .unwrap_or(false)
}
```

Extend with a claims struct for `email`, `chatgpt_account_id` / org fields, plan labels — same `insecure_decode` style. Map account id → `GrokAuth.organization_id`.

---

### `shell/src/auth/codex/refresh.rs` + `refresh/codex_refresher.rs` (service, request-response)

**Analog pure exchange:** `shell/src/auth/oidc/refresh.rs` (lines 1–67)
```rust
//! Pure-data OIDC refresh. Talks to the IdP and returns
//! [`OidcRefreshResult`] without touching [`AuthManager`].

pub(crate) enum OidcRefreshResult {
    Success(Box<GrokAuth>),
    TerminalError { reason: RefreshTokenFailedReason },
    Failed,
}

pub(super) fn classify_terminal(error_code: &str) -> Option<RefreshTokenFailedReason> {
    match error_code {
        "invalid_grant" => Some(RefreshTokenFailedReason::RefreshTokenRejected),
        "invalid_client" => Some(RefreshTokenFailedReason::ClientRejected),
        _ => None,
    }
}
```

**Codex delta:** fixed `POST {issuer}/oauth/token` with `grant_type=refresh_token` — **no OIDC discovery**.

**Analog TokenRefresher trait:** `shell/src/auth/refresh/mod.rs` (lines 136–144):
```rust
#[async_trait::async_trait]
pub(crate) trait TokenRefresher: Send + Sync {
    /// Implementations MUST NOT call auth_manager.update(), clear(),
    /// hot_swap(), or any other state-mutating method. Return the
    /// result and let refresh_chain handle all mutations.
    async fn refresh(&self, reason: RefreshReason) -> RefreshOutcome;
}
```

**Analog OidcRefresher structure** (`oidc_refresher.rs` lines 31–50, 90–115) — disk-sibling adopt on `invalid_grant`:
```rust
pub(crate) struct OidcRefresher {
    auth: Arc<dyn AuthSnapshot>,
    // ...
}

/// One-shot retry with disk's RT after `invalid_grant`.
async fn retry_with_fresh_disk_token(
    &self,
    tried: &crate::auth::GrokAuth,
) -> Option<RefreshOutcome> {
    let disk_now = self.auth.read_disk_auth()?;
    if !crate::auth::is_expired(&disk_now) && disk_now.key != tried.key {
        // adopt sibling AT without consuming RT
        return Some(RefreshOutcome::success(disk_now));
    }
    // ...
}
```

**Permanent-failure isolation (manager)** — lines 475–536: xAI `clear`/`remove_scope` only mutates xAI slot via `mutate_xai_store_or_prune_with_lock`. Codex permanent fail must use **codex slot clear only** — never `AuthManager::clear()` on the xAI manager for Codex failures.

**RefreshOutcome routing** (`refresh/mod.rs` lines 86–108):
```rust
pub(crate) enum RefreshOutcome {
    Success(Box<GrokAuth>),
    PermanentFailure {
        error: crate::auth::error::RefreshTokenFailedError,
        tried_key: Option<String>,
    },
    TransientFailure { message: String },
}
```

**RESEARCH recommendation:** thin Codex handle + shared `TokenRefresher` trait; do **not** rewrite xAI `AuthManager` for dual ownership in Phase 5.

---

### `pager/src/app/cli.rs` (route, request-response)

**Analog:** same file — extend clap shapes.

**Current Login/Logout** (lines 18–42):
```rust
/// Sign out and clear cached credentials
Logout,
/// Sign in to Grok
Login {
    #[arg(long, hide = true)]
    legacy: bool,
    #[arg(long = "oauth", alias = "oidc", conflicts_with_all = ["device_auth"])]
    oauth: bool,
    #[arg(
        long = "device-auth",
        visible_alias = "device-code",
        conflicts_with_all = ["oauth"]
    )]
    device_auth: bool,
    #[arg(skip)]
    devbox: bool,
},
```

**Target shape** (RESEARCH):
```rust
Login {
    provider: Option<ProviderCli>, // None => xai (bare login lock)
    oauth: bool,
    device_auth: bool,
    // ...
}
Logout {
    provider: Option<ProviderCli>,
    all: bool, // conflicts with provider
}
Auth {
    Status,
}
```

**Existing parse tests** (lines 885–925) — extend:
```rust
#[test]
fn bum_login_defaults_to_xai_without_provider_argument() {
    let args = PagerArgs::try_parse_from(["bum", "login"]).expect("bare bum login parses");
    // assert provider is None / xai default
}
```

Add tests for: `--provider codex`, bare logout fail-closed, `auth status`, `--all` vs `--provider` conflict.

---

### `pager-bin/src/main.rs` (controller, request-response)

**Analog:** same file lines 1739–1763

```rust
Command::Login {
    legacy: _,
    oauth,
    device_auth,
    devbox,
} => {
    init_tracing_simple("cli");
    let config = /* load */;
    xai_grok_shell::auth::run_cli_login(&config, oauth, device_auth, devbox).await?;
    // ...
}
Command::Logout => {
    xai_grok_shell::auth::run_cli_logout(&config)?;
    // ...
}
```

Pass new `provider` / `all` fields into shell handlers; add `Command::Auth { Status }` arm → `run_cli_auth_status`.

---

### `pager/src/slash/commands/logout.rs` + `dispatch/auth.rs` (component/controller)

**Analog slash** (`logout.rs` full file):
```rust
impl SlashCommand for LogoutCommand {
    fn name(&self) -> &str { "logout" }
    fn description(&self) -> &str {
        "Log out and return to the login screen"
        // Target: dual-safe — e.g. "Log out of a provider (CLI: bum logout --provider …)"
    }
    fn run(&self, _ctx: &mut CommandExecCtx, _args: &str) -> CommandResult {
        CommandResult::Action(Action::Logout)
    }
}
```

**Analog dispatch** (`dispatch/auth.rs` lines 18–21):
```rust
pub(super) fn dispatch_logout(_app: &mut AppView) -> Vec<Effect> {
    vec![Effect::Logout]
    // Target: fail-closed if would dual-wipe; toast per UI-SPEC
}
```

**Effect handler:** `pager/src/app/effects/mod.rs` line 76 `Effect::Logout` — ensure shell path is provider-safe (no silent dual clear).

---

### `shell/src/agent/config.rs` (service, CRUD)

**Analog:** `snapshot_codex_session_key_from_auth_store` (lines 4700–4748)

```rust
pub fn snapshot_codex_session_key_from_auth_store() -> Option<String> {
    // mtime/len cache ...
    let token = match crate::auth::read_provider_auth_store(&path, crate::auth::PROVIDER_CODEX) {
        Ok(Some(store)) => crate::auth::select_provider_access_token(&store).map(|a| a.key),
        Ok(None) => None,
        Err(e) => {
            tracing::warn!(error = %e, "codex auth store snapshot failed; fail-closed");
            None
        }
    };
    // ...
}
```

**Phase 5 actions:**
1. Invalidate mtime cache on Codex login/logout/refresh mutations (or provide `invalidate_codex_session_key_cache()`).
2. Prefer live re-read / short TTL after AUTH-05 refresher exists.
3. Inject `ChatGPT-Account-ID` from `GrokAuth.organization_id` on Codex sampling path when account id is persisted.

**Token selection** — `model.rs` lines 354–375 (`select_provider_access_token`) already ranks Oidc > ApiKey; Codex login must write selectable Oidc entry.

---

### `shell/src/auth/model.rs` (model, transform)

**Analog constants already present** (lines 16–35, 70–126):
```rust
pub const PROVIDER_XAI: &str = "xai";
pub const PROVIDER_CODEX: &str = "codex";
pub const AUTH_DOCUMENT_VERSION: u32 = 1;

pub struct GrokAuth {
    pub key: String,
    pub auth_mode: AuthMode,
    // ...
    pub organization_id: Option<String>, // ← store chatgpt_account_id here
    pub refresh_token: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub oidc_issuer: Option<String>,
    pub oidc_client_id: Option<String>,
    pub email: Option<String>,
    // ...
}
```

**Debug redaction** (lines 128–140) — never log full tokens:
```rust
.field("key", &token_suffix(&self.key))
.field("refresh_token", &self.refresh_token.as_deref().map(token_suffix))
```

Optional: scope key constant for Codex slot entry (`openai::chatgpt` vs `issuer::client_id`) — lock one for fixtures (RESEARCH A4).

---

### `shell/tests/auth_codex_lifecycle.rs` (test, batch)

**Analog:** `shell/tests/auth_multi_slot.rs` (lines 1–49)

```rust
use xai_grok_shell::auth::{AuthManager, AuthMode, GrokAuth, GrokComConfig};

#[tokio::test]
async fn public_auth_manager_loads_xai_from_nested_multi_slot_document() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("auth.json");
    // write nested providers.xai + providers.codex
    // assert AuthManager sees xAI only; codex sibling preserved on disk
}
```

**Wave 0 test cases to copy structure for** (RESEARCH Validation Architecture):
| Test name (suggested) | Req |
|-----------------------|-----|
| `codex_login_persists_slot` | AUTH-02 |
| `codex_authorize_url_*` | AUTH-02 |
| `selective_logout_isolates` | AUTH-03 |
| `bare_logout_fail_closed` | AUTH-03 |
| `logout_all_clears_both` | AUTH-03 |
| `auth_status_format_*` | AUTH-04 |
| `codex_refresh_isolates` | AUTH-05 |
| `codex_invalid_grant_no_xai_wipe` | AUTH-05 |

Use `tempfile` + `wiremock`/`axum` mock IdP (same stack as oidc test_helpers). Prefer `$BUM_HOME` isolation via existing env guards / `grok_home` test patterns from multi_slot.

**Also extend** `auth_multi_slot.rs` for concurrent sibling-preserve after new writers land.

---

## Shared Patterns

### Multi-slot atomic RMW
**Source:** `shell/src/auth/storage.rs` lines 416–449 (`mutate_auth_document*`), 479–542 (`mutate_xai_store_or_prune*`)
**Apply to:** Codex login persist, selective logout, Codex refresh write, permanent-fail wipe
```rust
// Always: lock → still_live → read document → mutate ONE provider → still_live → write/prune
// Never: rewrite entire providers map from a single-slot mental model
// Never: write_auth_json for Codex (xAI-only helper)
```

### Provider wire ids
**Source:** `shell/src/auth/model.rs` lines 16–20
**Apply to:** CLI `--provider`, status labels (map to UI labels `xAI`/`Codex`), storage keys
```rust
pub const PROVIDER_XAI: &str = "xai";
pub const PROVIDER_CODEX: &str = "codex";
```

### CLI stderr login UX
**Source:** `shell/src/auth/oidc/login.rs` lines 428–447; `device_code.rs` lines 366–392; UI-SPEC
**Apply to:** Codex browser + device flows
- Indented URL/code lines (2 spaces)
- Dim safety note ANSI `\x1b[90m...\x1b[0m`
- `✓ Signed in to {ProviderLabel}…` success (provider-labeled)
- Never print tokens in errors

### PKCE S256
**Source:** `shell/src/auth/oidc/protocol.rs` lines 333–345
**Apply to:** Codex browser authorize URL
- Prefer exporting `generate_pkce` / `Pkce` for reuse (`pub(crate)` from oidc or shared util)
- Challenge method always `S256`

### TokenRefresher isolation
**Source:** `shell/src/auth/refresh/mod.rs` + `oidc_refresher.rs`
**Apply to:** Codex refresher
- Pure IdP call returns `RefreshOutcome`; caller persists
- On permanent fail: wipe/mark **only** that provider’s slot
- Disk-sibling adopt before wipe (token family protection)

### Fail-closed bare logout
**Source:** CONTEXT/UI-SPEC (product contract); current `run_cli_logout` is **legacy single-provider** — must change
**Apply to:** CLI logout + TUI `/logout`
```
Specify a provider or clear all:
  bum logout --provider xai
  bum logout --provider codex
  bum logout --all
```
Non-zero exit; **no** mutation of `auth.json`.

### Secrets / redaction
**Source:** `GrokAuth::Debug` (`model.rs` 128–140); `AuthStoreReadError` Display (`storage.rs` 168–188)
**Apply to:** status CLI, tracing, error bodies
- Status: greppable key:value only (`logged_in`, `usable`, `account`, `plan`)
- Logs: `token_suffix` / lengths / error codes — never raw bearer

### Error style
**Source:** `shell/src/auth/error.rs`; `device_code.rs` `DeviceCodeError`
**Apply to:** new Codex modules
- `thiserror` enums for domain errors
- `anyhow::Result` at CLI/flow boundaries with `.context()`
- Permanent refresh → `RefreshTokenFailedReason` + provider-scoped re-login copy:
  `{ProviderLabel} session is no longer valid. Run \`bum login --provider {id}\`.`

### AuthManager scope for xAI only
**Source:** `manager.rs` clear/remove_scope (475–536)
**Apply to:** dual lifecycle design
- Keep existing AuthManager for xAI scope runtime
- Codex: storage-level ops + thin refresher (RESEARCH open Q1)
- Do not call xAI `AuthManager::clear()` for Codex logout/fail

---

## No Analog Found

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| — | — | — | All Phase 5 files have in-tree analogs. Codex OAuth **wire contract** (ports, client_id, deviceauth paths) has no in-tree implementation — use RESEARCH constants + openai/codex mirror, but **code structure** copies oidc/login + device_code + storage RMW. |

**External contract reference (not code analog):** openai/codex `codex-rs/login` (PKCE, 1455/1457, `localhost` redirect, deviceauth) documented in `05-RESEARCH.md`. Prefer axum over Codex’s `tiny_http`.

---

## Metadata

**Analog search scope:**
- `crates/codegen/xai-grok-shell/src/auth/**`
- `crates/codegen/xai-grok-shell/src/agent/config.rs` (Codex snapshot)
- `crates/codegen/xai-grok-shell/tests/auth_multi_slot.rs`
- `crates/codegen/xai-grok-pager/src/app/cli.rs`
- `crates/codegen/xai-grok-pager/src/app/dispatch/auth.rs`
- `crates/codegen/xai-grok-pager/src/slash/commands/logout.rs`
- `crates/codegen/xai-grok-pager-bin/src/main.rs`

**Files scanned:** ~35 auth-related modules + CLI/TUI wiring + multi_slot tests
**Pattern extraction date:** 2026-07-16

**Key takeaway for planner:** Isolation is the hard pattern, not OAuth. Every write path must be provider-parameterized RMW; every permanent failure must be slot-scoped; bare logout must fail closed; Codex browser must use Codex ports/redirect strings while reusing axum + PKCE helpers from xAI.
