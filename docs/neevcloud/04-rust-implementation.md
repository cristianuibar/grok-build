# NeevCloud — Rust implementation

Near-copy-paste code for the port. Everything here mirrors `crates/codegen/xai-grok-shell/src/auth/codex/`, which is the closest existing analog (JSON-bodied, non-RFC-8628-shaped, out-of-band refresh, its own slot). Read that tree before this doc; every idiom below is lifted from it, not invented.

Paths are relative to `/home/cristian/bum/grok-build`.

## 0. Decisions you must make once, before any code

| Decision | Value used in this doc | Why |
|---|---|---|
| Wire id | `"neev"` | Must be byte-identical across `PROVIDER_NEEV`, `ModelProvider::as_str`, `AuthProviderArg` (clap kebab-cases variants — `NeevCloud` would become `--provider neev-cloud`; `Neev` → `neev` works by luck like `Codex` → `codex`, `crates/codegen/xai-grok-pager/src/app/cli.rs:12-17`), `usable_for_wire`, and the TUI allowlists. One string, five files. |
| Module | `crates/codegen/xai-grok-shell/src/auth/neev/` | No new crate. Codex added zero crates; root `Cargo.toml:1` says it is auto-generated. |
| Scope key | `NEEV_AUTH_SCOPE = "https://code.neevcloud.com::neev-cli"` | Follows the `{issuer}::{client_id}` convention pinned at `auth/codex/mod.rs:54`. |
| `api_backend` | `chat_completions` | `ApiBackend::ChatCompletions` is `#[default]` (`crates/codegen/xai-grok-sampling-types/src/types.rs:1013`). Codex's `responses` is the wrong wire — do not copy the model rows. |

`AUTH_DOCUMENT_VERSION` stays `1` (`auth/model.rs:73`). A new `providers` key is additive; bumping the version makes every older `bum` fail-closed on the whole file and lose the xai + codex slots.

## 1. Module layout

```
crates/codegen/xai-grok-shell/src/auth/neev/
  mod.rs          consts + NeevLoginError + run_neev_login + pub use barrel
  fidelity.rs     every wire-fidelity string + id generators — owned by 07-wire-fidelity.md
  http.rs         RequestBuilder helpers (timeout, content-type, x-org-id, fidelity headers)
  device.rs       POST /auth/device/code + poll loop
  refresh.rs      pure token exchange (mirrors codex/refresh.rs)
  console.rs      /api/user, /api/orgs, /api/config
  ensure_fresh.rs lock-held refresh chain (mirrors codex/ensure_fresh.rs)
```

No `browser.rs` (device-code only, no PKCE loopback). No `claims.rs` — the token is an opaque 67-char `sk-…`, not a JWT; `decode_codex_claims` (`auth/codex/claims.rs:59`) would return empty claims and `resolve_expires_at` would fall back to `CONSERVATIVE_EXPIRES_FALLBACK` (5 minutes, `claims.rs:16`), re-refreshing a 1-year token every 5 minutes. Take `expires_in` from the token response instead.

## 2. `mod.rs` — consts, error, entry point

```rust
//! NeevCloud device-code login for the `providers.neev` slot.
//!
//! Wire contract reverse-engineered from `@neevcode/neev@0.0.2` and confirmed
//! against the live console. OAuth-shaped but delivers ONE long-lived static
//! bearer: access_token == refresh_token == the `/api/config` `options.apiKey`.
//! That single secret is both the account session and the inference key —
//! never log it, never put a response body in an error string.
//!
//! Never reads neev's own 0644 SQLite store. Never writes the xai/codex slots.

mod console;
mod device;
mod ensure_fresh;
/// Wire-fidelity strings + id generators. `pub` because the gateway headers are
/// applied at the sampler seam (§10), outside `auth::neev`. Spec: docs/neevcloud/07-wire-fidelity.md.
pub mod fidelity;
mod http;
mod refresh;

pub use console::{
    NeevGatewayConfig, NeevOrg, NeevUser, fetch_neev_config, fetch_neev_orgs, fetch_neev_user,
};
pub use device::{
    NeevTokenResponse, neev_device_code_url, neev_device_token_url, neev_device_verify_url,
    run_neev_device_login, run_neev_device_login_with_base, run_neev_device_poll_only_with_base,
};
// NeevTokenResponse must be in the barrel: `device` is a private module, and
// refresh.rs's re-exported `pub fn merge_neev_refresh_response(_, &NeevTokenResponse)`
// would otherwise trip `private_interfaces` and be uncallable from outside.
pub use ensure_fresh::{
    EnsureFreshNeevOptions, EnsureFreshNeevResult, NeevAuthMaterial, ensure_fresh_neev_auth,
    ensure_fresh_neev_auth_at,
};
pub use refresh::{NeevRefreshResult, merge_neev_refresh_response, neev_token_exchange};

use std::path::Path;

use crate::auth::GrokAuth;

/// Production console base. Dev: `https://dev.code.neevcloud.com`.
pub const NEEV_CONSOLE_URL_DEFAULT: &str = "https://code.neevcloud.com";
/// Public device-code client id (constant `HK` in the neev binary).
pub const NEEV_CLIENT_ID: &str = "neev-cli";
/// Stable multi-slot scope key under `providers.neev`.
pub const NEEV_AUTH_SCOPE: &str = "https://code.neevcloud.com::neev-cli";
/// Free-form human label sent as the `client` field on /auth/device/code.
///
/// The version segment is a fidelity string — it comes from `fidelity::NEEV_VERSION`
/// so `grep -rn '0\.0\.2' auth/neev/` keeps returning exactly one file (07 §7).
/// ponytail: platform is not derived; the server only displays this.
pub fn neev_client_label() -> String {
    format!("neev CLI {} on linux", fidelity::NEEV_VERSION)
}

/// Console base URL. `NEEVCLOUD_CONSOLE_URL` overrides; trailing slashes stripped.
///
/// ponytail: env var is deliberately NOT `GROK_`/`BUM_`-prefixed (breaks
/// CONVENTIONS.md) — it is the vendor's own name and must stay verbatim.
pub fn neev_console_url() -> String {
    std::env::var("NEEVCLOUD_CONSOLE_URL")
        .ok()
        .map(|v| v.trim().trim_end_matches('/').to_owned())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| NEEV_CONSOLE_URL_DEFAULT.to_owned())
}

/// Errors from the NeevCloud login path. Mirrors `CodexLoginError`
/// (`auth/codex/browser.rs:31-53`) minus the PKCE/loopback variants.
#[derive(Debug, thiserror::Error)]
pub enum NeevLoginError {
    #[error("OAuth error from NeevCloud: {0}")]
    OAuthError(String),
    #[error("Token exchange failed: {0}")]
    TokenExchange(String),
    /// Cloudflare `error_code: 1010, browser_signature_banned`. Presents as 403,
    /// NOT 401 — a dedicated variant so it never reads as an auth failure.
    /// Defensive only: measured against the live console, reqwest's UA is not
    /// banned (see §3). Cloudflare rules are the vendor's to change, and a 403
    /// that reads as "wrong password" is a bad hour.
    #[error(
        "NeevCloud rejected this client at the CDN (HTTP 403). Not an auth \
         failure — the request never reached the API."
    )]
    BlockedClient,
    #[error("Login timed out. Please try again.")]
    Timeout,
    #[error("Failed to persist NeevCloud credentials: {0}")]
    Persist(String),
    #[error("{0}")]
    Other(String),
}

/// Run NeevCloud login. Device-code only — no browser/loopback flow exists.
///
/// Persists only `providers.neev` under `auth_file`.
pub async fn run_neev_login(auth_file: &Path) -> Result<GrokAuth, NeevLoginError> {
    run_neev_device_login(auth_file).await
}
```

## 3. `http.rs` — request builders

These builders carry the **console identity** (identity A, `neev/latest/0.0.2/cli`). The strings and the
`console_headers` helper are owned by **[07-wire-fidelity.md](07-wire-fidelity.md)** — do not restate them here,
and do not put a fidelity literal anywhere but `fidelity.rs` (07 §7).

**Nothing here is required.** Measured live against `code.neevcloud.com/api/orgs`, authenticated:

| Client UA | Result |
|---|---|
| curl default (`curl/8.x`) | 200 |
| `neev/latest/0.0.2/cli` | 200 |
| empty UA | 200 |
| `Python-urllib/3.13` | **403** `error_code: 1010, browser_signature_banned` |

Cloudflare bans a small set of **known-bot signatures**, not unknown ones — the same urllib client returns 200 the moment it sends any other UA, and an absent UA sails through. `xai_grok_http::shared_client()` (`crates/codegen/xai-grok-http/src/lib.rs:282`) sets `.user_agent(process_user_agent_string())` at `:289` → `grok-shell/<ver> (linux; x86_64)`, and that passes fine. So the UA below is **not** a fix for anything: it is the adopted fidelity decision (07 §1) — forward-compat insurance against NeevCloud gating on client identity later. It buys nothing today, and it can never be the cause of a bug.

The per-request `.headers(…)` is the only mechanism available: `set_client_name` (`xai-grok-http/src/lib.rs:164`) is process-global and `expect`s on a second call, so using it would rewrite xAI's and Codex's UA too. See 07 §3.

```rust
//! Request builders for NeevCloud-bound calls. Console identity (07 §2A).

use std::time::Duration;

use reqwest::RequestBuilder;

use super::fidelity::console_headers;

const NEEV_TIMEOUT: Duration = Duration::from_secs(30);

pub(super) fn neev_post(url: &str) -> RequestBuilder {
    crate::http::shared_client()
        .post(url)
        // `.headers()` overrides the client-level UA for these calls only.
        .headers(console_headers(false))
        .header("Content-Type", "application/json")
        .timeout(NEEV_TIMEOUT)
}

pub(super) fn neev_get(url: &str, bearer: &str) -> RequestBuilder {
    crate::http::shared_client()
        .get(url)
        .headers(console_headers(false))
        .bearer_auth(bearer)
        .timeout(NEEV_TIMEOUT)
}

/// `x-org-id` is required for /api/config and /api/mcp/list — console calls
/// only. The `/zen/go/v1/chat/completions` gateway does NOT want it.
///
/// `accept_json` mirrors neev's `acceptJson` pipe: `application/json` on
/// /api/config, `*/*` everywhere else (07 §2A).
pub(super) fn neev_get_org(
    url: &str,
    bearer: &str,
    org_id: &str,
    accept_json: bool,
) -> RequestBuilder {
    crate::http::shared_client()
        .get(url)
        .headers(console_headers(accept_json))
        .bearer_auth(bearer)
        .header("x-org-id", org_id)
        .timeout(NEEV_TIMEOUT)
}
```

`crate::http` is the in-shell alias for `xai_grok_http` — the same one `auth/codex/device.rs:114` uses. Do not build a second `reqwest::Client`; that forfeits the pool self-healing documented at `xai-grok-http/src/lib.rs:274-281`. Note `neev_get_org` cannot be `neev_get(…).header(…)`, because `accept_json` has to reach `console_headers` before the map is applied.

The gateway identity (identity B) is a different client with a different UA and the four `x-opencode-*` headers; it is wired at the sampler seam, not here — 07 §2B/§4 and §10 below.

The `Authorization` and `x-org-id` headers stay where they are — they are protocol, not fidelity, and `console_headers` deliberately sets neither.

## 4. `device.rs` — device/code + poll loop

```rust
//! NeevCloud device-code login (RFC 8628 grant type, JSON bodies).
//!
//! 1. POST `{base}/auth/device/code` → device_code, user_code,
//!    verification_uri_complete (a PATH), expires_in, interval
//! 2. Poll POST `{base}/auth/device/token` until access_token
//! 3. GET /api/user + /api/orgs to fill identity, then persist `providers.neev`

use std::path::Path;
use std::time::{Duration, Instant};

use chrono::Utc;
use serde::{Deserialize, Serialize};

use super::console::{fetch_neev_orgs, fetch_neev_user};
use super::http::neev_post;
use super::{NEEV_AUTH_SCOPE, NEEV_CLIENT_ID, NeevLoginError, neev_client_label, neev_console_url};
use crate::auth::{
    AuthMode, AuthProvider, GrokAuth, ProviderStoreMutation, mutate_provider_store_or_prune,
};

const DEFAULT_POLL_INTERVAL_SECS: u64 = 5;
const SLOW_DOWN_INCREMENT_SECS: u64 = 5;
/// Floor on the server's `expires_in`, mirroring the xAI device flow's
/// MIN_DEVICE_CODE_EXPIRY_FALLBACK_SECS (auth/device_code.rs:22).
const MIN_DEVICE_CODE_EXPIRY_FALLBACK_SECS: u64 = 600;
/// Reused verbatim from the xAI flow (auth/device_code.rs:19) — identical literal.
const DEVICE_GRANT_TYPE: &str = "urn:ietf:params:oauth:grant-type:device_code";

pub fn neev_device_code_url(base: &str) -> String {
    format!("{}/auth/device/code", base.trim_end_matches('/'))
}

pub fn neev_device_token_url(base: &str) -> String {
    format!("{}/auth/device/token", base.trim_end_matches('/'))
}

/// The server returns `verification_uri_complete` as a **PATH**, not an absolute
/// URL — the browser link is `{base}{path}`. This is a real quirk; the Codex
/// analog (`codex_device_verify_url`, auth/codex/device.rs:40) builds an
/// absolute URL from the issuer, so a structural copy prints a broken link.
///
/// Concatenate FIRST, then validate, so the https/control-char guard still runs.
pub fn neev_device_verify_url(base: &str, verification_uri_complete: &str) -> String {
    format!(
        "{}{}",
        base.trim_end_matches('/'),
        verification_uri_complete
    )
}
```

### Wire types

Every field below is from the verified protocol. Nothing else.

```rust
#[derive(Serialize)]
struct DeviceCodeReq<'a> {
    client_id: &'a str,
    /// Free-form human label: `neev CLI ${version} on ${platform}`.
    client: &'a str,
}

#[derive(Deserialize)]
struct DeviceCodeResp {
    device_code: String,
    user_code: String,
    /// A PATH (e.g. `/device?code=…`), not an absolute URL.
    verification_uri_complete: String,
    #[serde(default)]
    expires_in: Option<u64>,
    #[serde(default)]
    interval: Option<serde_json::Value>, // number or string — see parse_interval
}

#[derive(Serialize)]
struct TokenPollReq<'a> {
    grant_type: &'a str,
    device_code: &'a str,
    client_id: &'a str,
}

#[derive(Serialize)]
pub(super) struct RefreshReq<'a> {
    pub grant_type: &'a str, // "refresh_token"
    pub refresh_token: &'a str,
    pub client_id: &'a str,
}

/// Success body for both the device poll and the refresh.
///
/// Verified: access_token == refresh_token == the /api/config apiKey, all three
/// byte-identical, ~1 year expiry. Fields stay independent anyway — do not
/// collapse them; if the server ever diverges the code should just work.
#[derive(Debug, Clone, Deserialize)]
pub struct NeevTokenResponse {
    pub access_token: String,
    #[serde(default)]
    pub refresh_token: Option<String>,
    #[serde(default)]
    pub expires_in: Option<u64>,
}

/// Standard RFC 8628 error body.
#[derive(Deserialize)]
struct PollErrorBody {
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    error_description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct NeevDeviceCode {
    pub verification_url: String,
    pub user_code: String,
    device_code: String,
    interval: u64,
    expires_in: u64,
}

/// Interval may arrive as a JSON number or a string (Codex tolerates both,
/// auth/codex/device.rs:93 — same defensiveness, same cost).
fn parse_interval(value: Option<serde_json::Value>) -> u64 {
    match value {
        Some(serde_json::Value::Number(n)) => n.as_u64().unwrap_or(DEFAULT_POLL_INTERVAL_SECS),
        Some(serde_json::Value::String(s)) => s.trim().parse().unwrap_or(DEFAULT_POLL_INTERVAL_SECS),
        _ => DEFAULT_POLL_INTERVAL_SECS,
    }
    .max(1)
}
```

### Request

```rust
pub async fn request_neev_device_code(
    base: &str,
    client_id: &str,
) -> Result<NeevDeviceCode, NeevLoginError> {
    let resp = neev_post(&neev_device_code_url(base))
        .json(&DeviceCodeReq {
            client_id,
            client: &neev_client_label(),
        })
        .send()
        .await
        .map_err(|e| NeevLoginError::Other(format!("device code request failed: {e}")))?;

    let status = resp.status();
    if status.as_u16() == 403 {
        return Err(NeevLoginError::BlockedClient);
    }
    if !status.is_success() {
        // Do not include body (may leak tokens) — auth/codex/device.rs:252.
        return Err(NeevLoginError::Other(format!(
            "device code request failed with HTTP {status}"
        )));
    }

    let dc: DeviceCodeResp = resp
        .json()
        .await
        .map_err(|e| NeevLoginError::Other(format!("invalid device code response: {e}")))?;

    // Charset guard against control chars from a hostile/compromised console
    // (mirrors auth/codex/device.rs:142 and auth/device_code.rs:175-184).
    if !dc
        .user_code
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-')
    {
        return Err(NeevLoginError::Other(
            "server returned invalid user_code format".into(),
        ));
    }

    let verification_url = neev_device_verify_url(base, &dc.verification_uri_complete);
    // Validate the CONCATENATED url — a bare path would fail Url::parse.
    validate_verification_url(&verification_url)?;

    Ok(NeevDeviceCode {
        verification_url,
        user_code: dc.user_code,
        device_code: dc.device_code,
        interval: parse_interval(dc.interval),
        expires_in: dc
            .expires_in
            .unwrap_or(MIN_DEVICE_CODE_EXPIRY_FALLBACK_SECS)
            .max(MIN_DEVICE_CODE_EXPIRY_FALLBACK_SECS),
    })
}

/// https-only (loopback http tolerated for mock IdP tests), no control chars.
///
/// `validate_verification_uri` (auth/device_code.rs:521) does the same job for
/// the xAI flow, but it is a **private `fn`** in that module (not `pub(super)`/
/// `pub(crate)`), so `auth::neev` cannot call it. Either widen it to
/// `pub(super)` and reuse — it differs only in returning `anyhow::Result` and
/// using `url::Url` + `is_ascii_control` — or keep this local copy.
fn validate_verification_url(url: &str) -> Result<(), NeevLoginError> {
    if url.chars().any(|c| c.is_control()) {
        return Err(NeevLoginError::Other(
            "server returned invalid verification URL".into(),
        ));
    }
    let parsed = reqwest::Url::parse(url)
        .map_err(|_| NeevLoginError::Other("server returned invalid verification URL".into()))?;
    let ok = parsed.scheme() == "https"
        || (parsed.scheme() == "http"
            && matches!(parsed.host_str(), Some("127.0.0.1") | Some("localhost")));
    if !ok {
        return Err(NeevLoginError::Other(
            "server returned a non-https verification URL".into(),
        ));
    }
    Ok(())
}
```

### Poll loop

Sleep-first ordering is deliberate: an immediate poll on a fresh code only ever returns `authorization_pending` and risks `slow_down` (rationale at `auth/device_code.rs:231`, same ordering at `auth/codex/device.rs:200`).

```rust
async fn poll_for_token(
    base: &str,
    client_id: &str,
    device_code: &NeevDeviceCode,
) -> Result<NeevTokenResponse, NeevLoginError> {
    let url = neev_device_token_url(base);
    let mut poll_interval = Duration::from_secs(device_code.interval.max(1));
    let deadline = Duration::from_secs(device_code.expires_in);
    let start = Instant::now();

    loop {
        if start.elapsed() >= deadline {
            return Err(NeevLoginError::Timeout);
        }

        tokio::time::sleep(poll_interval).await;

        let resp = neev_post(&url)
            .json(&TokenPollReq {
                grant_type: DEVICE_GRANT_TYPE,
                device_code: &device_code.device_code,
                client_id,
            })
            .send()
            .await
            .map_err(|e| NeevLoginError::Other(format!("device token poll failed: {e}")))?;

        let status = resp.status();
        if status.is_success() {
            return resp.json::<NeevTokenResponse>().await.map_err(|e| {
                NeevLoginError::TokenExchange(format!("invalid device token success body: {e}"))
            });
        }
        if status.as_u16() == 403 {
            return Err(NeevLoginError::BlockedClient);
        }

        let text = resp.text().await.unwrap_or_default();
        let err_body: Option<PollErrorBody> = serde_json::from_str(&text).ok();
        let error_code = err_body
            .as_ref()
            .and_then(|b| b.error.as_deref())
            .unwrap_or("");

        match error_code {
            "authorization_pending" => continue,
            "slow_down" => {
                poll_interval += Duration::from_secs(SLOW_DOWN_INCREMENT_SECS);
                continue;
            }
            "access_denied" => {
                return Err(NeevLoginError::OAuthError(
                    "Authorization denied. The request was rejected.".into(),
                ));
            }
            "expired_token" => {
                return Err(NeevLoginError::Other(
                    "Device code expired. Run `bum login --provider neev` again.".into(),
                ));
            }
            _ => {
                // Never echo the body — the token IS the account secret.
                return Err(NeevLoginError::Other(format!(
                    "device auth failed with HTTP {status}"
                )));
            }
        }
    }
}
```

Note the divergence from Codex: `auth/codex/device.rs:249-251` treats 403/404 as *pending* and `continue`s. Do **not** copy that. Any unexpected 403 — CDN, misrouted request, revoked client — spins silently until MAX_WAIT (15 min) and then reports a timeout, hiding the actual cause. Surface it immediately. This is independent of anything UA-related.

### Persist

`GrokAuth` needs zero new fields; every NeevCloud datum maps onto an existing one.

| NeevCloud | `GrokAuth` field |
|---|---|
| `access_token` | `key` |
| `refresh_token` (== access_token) | `refresh_token` — **must** be `Some`, see below |
| `expires_in` | `expires_at: Some(now + expires_in)` |
| console base | `oidc_issuer` |
| `"neev-cli"` | `oidc_client_id` |
| `/api/user` `id` (`acc_01…`) | `user_id` |
| `/api/user` `email` | `email` |
| `/api/orgs[0].id` (`wrk_01…`) | `organization_id` ← the `x-org-id` value |
| `/api/orgs[0].name` | `organization_name` |
| `/api/user` `plan.label` | *not stored* — `ProviderAuthStatus.plan` is derived; leave `None`, `format_auth_status` renders an em-dash |

`refresh_token: None` on an `AuthMode::Oidc` credential silently degrades to the unrefreshable `LegacySession` (`auth/token_type.rs:28-29`) and reports `usable: false`. The two tokens being byte-identical makes it tempting to store only `key`. Store both.

```rust
fn grok_auth_from_neev_tokens(
    base: &str,
    client_id: &str,
    tokens: &NeevTokenResponse,
    user: Option<&super::NeevUser>,
    org: Option<&super::NeevOrg>,
) -> GrokAuth {
    let now = Utc::now();
    GrokAuth {
        key: tokens.access_token.clone(),
        auth_mode: AuthMode::Oidc,
        create_time: now,
        user_id: user.map(|u| u.id.clone()).unwrap_or_default(),
        email: user.and_then(|u| u.email.clone()).filter(|e| !e.is_empty()),
        organization_id: org.map(|o| o.id.clone()),
        organization_name: org.map(|o| o.name.clone()),
        // Byte-identical to `key` per the verified protocol; stored anyway so
        // token_type.rs sees a refreshable Oidc session.
        refresh_token: tokens
            .refresh_token
            .clone()
            .or_else(|| Some(tokens.access_token.clone())),
        expires_at: Some(
            now + chrono::Duration::seconds(tokens.expires_in.unwrap_or(0) as i64),
        ),
        oidc_issuer: Some(base.trim_end_matches('/').to_owned()),
        oidc_client_id: Some(client_id.to_owned()),
        ..Default::default()
    }
}

/// Persist under `providers.neev` only. Sibling slots preserved by the storage
/// layer; 0600 + flock + atomic rename come free (auth/storage.rs:692, :725).
/// Never open auth.json directly.
fn persist_neev_auth(auth_file: &Path, auth: GrokAuth) -> Result<GrokAuth, NeevLoginError> {
    let to_store = auth.clone();
    let outcome = mutate_provider_store_or_prune(auth_file, AuthProvider::Neev, move |store| {
        store.insert(NEEV_AUTH_SCOPE.to_owned(), to_store);
    })
    .map_err(|e| NeevLoginError::Persist(e.to_string()))?;
    match outcome {
        ProviderStoreMutation::DocumentWritten | ProviderStoreMutation::Unchanged => {}
        // The store deletes auth.json when every slot empties
        // (persist_document_or_prune). Codex guards this at browser.rs:205-212.
        ProviderStoreMutation::FileDeleted => {
            return Err(NeevLoginError::Persist(
                "unexpected file delete after NeevCloud persist".into(),
            ));
        }
    }
    Ok(auth)
}
```

### Entry points — keep the injectable base on every one

```rust
/// Poll → identity → persist. Only writes after a successful token exchange
/// (auth/codex/device.rs:180).
pub async fn complete_neev_device_login(
    base: &str,
    client_id: &str,
    device_code: NeevDeviceCode,
    auth_file: &Path,
) -> Result<GrokAuth, NeevLoginError> {
    let tokens = poll_for_token(base, client_id, &device_code).await?;

    // Identity is best-effort: a working token with no /api/user must still log
    // in. org_id is the exception worth caring about — without it /api/config
    // 401s on every cold start.
    let user = fetch_neev_user(base, &tokens.access_token, None).await.ok().flatten();
    let org = fetch_neev_orgs(base, &tokens.access_token)
        .await
        .ok()
        .and_then(|orgs| orgs.into_iter().next());
    if org.is_none() {
        tracing::warn!("neev: /api/orgs returned no org — /api/config will be unavailable");
    }

    let auth = grok_auth_from_neev_tokens(base, client_id, &tokens, user.as_ref(), org.as_ref());
    persist_neev_auth(auth_file, auth)
}

pub async fn run_neev_device_login(auth_file: &Path) -> Result<GrokAuth, NeevLoginError> {
    run_neev_device_login_with_base(auth_file, &neev_console_url(), NEEV_CLIENT_ID).await
}

/// Device login with injectable console base (mock-console tests).
pub async fn run_neev_device_login_with_base(
    auth_file: &Path,
    base: &str,
    client_id: &str,
) -> Result<GrokAuth, NeevLoginError> {
    eprintln!("Signing in with NeevCloud...");
    let device_code = request_neev_device_code(base, client_id).await?;

    eprintln!("To sign in, open this URL in your browser:");
    eprintln!();
    eprintln!("  {}", device_code.verification_url);
    eprintln!();
    eprintln!("Then enter this code:");
    eprintln!();
    eprintln!("  {}", device_code.user_code);
    eprintln!();
    eprintln!(
        "\x1b[90mOnly continue with a code you requested. \
         Don't share it with anyone.\x1b[0m"
    );
    eprintln!();
    eprintln!("Waiting for authorization...");

    let _ = webbrowser::open(&device_code.verification_url);
    complete_neev_device_login(base, client_id, device_code, auth_file).await
}

/// Poll harness for tests: no CLI copy, no browser open.
pub async fn run_neev_device_poll_only_with_base(
    auth_file: &Path,
    base: &str,
    client_id: &str,
) -> Result<GrokAuth, NeevLoginError> {
    let device_code = request_neev_device_code(base, client_id).await?;
    complete_neev_device_login(base, client_id, device_code, auth_file).await
}
```

The prompt is plain `eprintln!` to stderr, exactly like `auth/codex/device.rs:274-290`. There is no ratatui screen, no `/login` slash command, and `AuthUrlMode::Device` has zero pager consumers — do not wire NeevCloud into it.

## 5. `refresh.rs` — pure exchange

Near-no-op (the "refresh" returns the same bytes) but still required: the reconstruct path expects the shape, and skipping it makes the neev slot diverge from the dual-auth lifecycle. `TokenRefresher::refresh` MUST NOT mutate — the outer `ensure_fresh` owns lock/persist/clear (`auth/refresh/mod.rs:142-145`).

```rust
//! Pure-data NeevCloud refresh-token exchange. No disk I/O, no store mutation.

use super::device::{NeevTokenResponse, RefreshReq, neev_device_token_url};
use super::http::neev_post;
use super::{NEEV_CLIENT_ID, neev_console_url};
use crate::auth::GrokAuth;
use crate::auth::error::RefreshTokenFailedReason;

#[derive(Debug)]
pub enum NeevRefreshResult {
    Success(Box<GrokAuth>),
    TerminalError { reason: RefreshTokenFailedReason },
    Failed,
}

/// Same OAuth2 classification as Codex (auth/codex/refresh.rs:25).
pub fn classify_terminal(error_code: &str) -> Option<RefreshTokenFailedReason> {
    match error_code {
        "invalid_grant" => Some(RefreshTokenFailedReason::RefreshTokenRejected),
        "invalid_client" => Some(RefreshTokenFailedReason::ClientRejected),
        _ => None,
    }
}

/// Identity-preserving merge (mirror auth/codex/refresh.rs:46). NeevCloud's
/// refresh body carries no identity at all, so every field below is preserved
/// from `prev` in practice — the merge exists so a server change can't silently
/// blank organization_id and break `x-org-id`.
pub fn merge_neev_refresh_response(prev: &GrokAuth, tokens: &NeevTokenResponse) -> GrokAuth {
    let now = chrono::Utc::now();
    let mut next = prev.clone();
    next.key = tokens.access_token.clone();
    next.refresh_token = tokens
        .refresh_token
        .clone()
        .or_else(|| prev.refresh_token.clone());
    if let Some(secs) = tokens.expires_in {
        next.expires_at = Some(now + chrono::Duration::seconds(secs as i64));
    }
    next.create_time = now;
    next
}

/// Exchange `refresh_token` at `{base}/auth/device/token`. Pure data.
///
/// `token_url_override` points tests at a mock console without rewriting the
/// stored issuer (mirror auth/codex/refresh.rs:85).
pub async fn neev_token_exchange(
    auth: &GrokAuth,
    token_url_override: Option<&str>,
) -> NeevRefreshResult {
    let Some(refresh_tok) = auth.refresh_token.as_ref().filter(|t| !t.is_empty()) else {
        tracing::debug!("neev refresh: missing refresh_token");
        return NeevRefreshResult::Failed;
    };
    let client_id = auth
        .oidc_client_id
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or(NEEV_CLIENT_ID);
    let base = auth
        .oidc_issuer
        .clone()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(neev_console_url);
    let token_url = token_url_override
        .map(str::to_owned)
        .unwrap_or_else(|| neev_device_token_url(&base));

    let resp = match neev_post(&token_url)
        .json(&RefreshReq {
            grant_type: "refresh_token",
            refresh_token: refresh_tok.as_str(),
            client_id,
        })
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!(error_len = e.to_string().len(), "neev: refresh request failed");
            return NeevRefreshResult::Failed;
        }
    };

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        if let Some(error_code) = serde_json::from_str::<serde_json::Value>(&body)
            .ok()
            .and_then(|v| v.get("error")?.as_str().map(str::to_owned))
            && let Some(reason) = classify_terminal(&error_code)
        {
            tracing::warn!(
                error_code = %error_code,
                http_status = %status.as_u16(),
                body_len = body.len(), // length only — never the body
                "neev: terminal refresh error"
            );
            return NeevRefreshResult::TerminalError { reason };
        }
        tracing::warn!(
            http_status = %status.as_u16(),
            body_len = body.len(),
            "neev: refresh failed (transient)"
        );
        return NeevRefreshResult::Failed;
    }

    let tokens = match resp.json::<NeevTokenResponse>().await {
        Ok(t) if !t.access_token.is_empty() => t,
        Ok(_) => {
            tracing::warn!("neev: refresh response missing access_token");
            return NeevRefreshResult::Failed;
        }
        Err(e) => {
            tracing::warn!(error_len = e.to_string().len(), "neev: bad refresh response");
            return NeevRefreshResult::Failed;
        }
    };

    NeevRefreshResult::Success(Box::new(merge_neev_refresh_response(auth, &tokens)))
}
```

The `let … && let …` chain is let-chains — edition 2024, already used at `auth/codex/refresh.rs:141-144`.

`auth/refresh/neev_refresher.rs` is a 60-line copy of `auth/refresh/codex_refresher.rs` (swap the exchange fn and the transient message). **Do not** wire it into `build_refresher` (`auth/refresh/mod.rs:148`) — that only ever builds `Oidc` or `External`, and Codex deliberately bypasses it. `CodexRefresher` is constructed out-of-band inside `ensure_fresh_codex_auth` (`auth/codex/ensure_fresh.rs:359`, `:376`). Copy that.

## 6. `console.rs` — /api/user, /api/orgs, /api/config

```rust
//! NeevCloud console API. Bearer + (for /api/config) `x-org-id`.

use serde::Deserialize;

use super::http::{neev_get, neev_get_org};
use super::NeevLoginError;

#[derive(Debug, Clone, Deserialize)]
pub struct NeevPlan {
    #[serde(rename = "type", default)]
    pub plan_type: Option<String>,
    #[serde(default)]
    pub label: Option<String>, // "Free"
    #[serde(default)]
    pub status: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NeevUser {
    pub id: String, // acc_01…
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(rename = "workspaceName", default)]
    pub workspace_name: Option<String>,
    #[serde(default)]
    pub plan: Option<NeevPlan>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NeevOrg {
    pub id: String, // wrk_01…
    pub name: String,
}

/// `config.provider.opencode.options` — the inference credentials.
#[derive(Debug, Clone, Deserialize)]
pub struct NeevProviderOptions {
    #[serde(rename = "baseURL")]
    pub base_url: String, // https://code.neevcloud.com/zen/go/v1
    #[serde(rename = "apiKey")]
    pub api_key: String, // byte-identical to the access_token
}

#[derive(Debug, Clone, Deserialize)]
pub struct NeevModelLimit {
    #[serde(default)]
    pub context: Option<u64>,
    #[serde(default)]
    pub output: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NeevModelSpec {
    #[serde(default)]
    pub name: Option<String>, // "MiniMax M2"
    #[serde(default)]
    pub tool_call: bool,
    #[serde(default)]
    pub reasoning: bool,
    #[serde(default)]
    pub limit: Option<NeevModelLimit>,
    // `cost` and `provider.npm` are deliberately not modeled — the bum catalog
    // has no pricing or npm concept. serde ignores unknown fields by default.
}

/// The provider key is literally `"opencode"` (upstream fork heritage) even
/// though the display name is "NeevCode Zen".
#[derive(Debug, Clone, Deserialize)]
pub struct NeevProviderEntry {
    #[serde(default)]
    pub name: Option<String>,
    pub options: NeevProviderOptions,
    #[serde(default)]
    pub whitelist: Vec<String>,
    #[serde(default)]
    pub models: std::collections::BTreeMap<String, NeevModelSpec>,
}

#[derive(Debug, Clone, Deserialize)]
struct NeevConfigProviders {
    opencode: NeevProviderEntry,
}

#[derive(Debug, Clone, Deserialize)]
struct NeevConfigInner {
    provider: NeevConfigProviders,
}

#[derive(Debug, Clone, Deserialize)]
struct NeevConfigEnvelope {
    config: NeevConfigInner,
}

/// Gateway config distilled to what bum actually consumes.
#[derive(Debug, Clone)]
pub struct NeevGatewayConfig {
    pub base_url: String,
    pub display_name: Option<String>,
    /// Whitelisted, tool-call-capable models only.
    pub models: Vec<(String, NeevModelSpec)>,
}

pub async fn fetch_neev_user(
    base: &str,
    bearer: &str,
    org_id: Option<&str>,
) -> Result<Option<NeevUser>, NeevLoginError> {
    let url = format!("{}/api/user", base.trim_end_matches('/'));
    let req = match org_id {
        // accept_json: false — neev only pipes acceptJson on /api/config (07 §2A).
        Some(org) => neev_get_org(&url, bearer, org, false),
        None => neev_get(&url, bearer),
    };
    let resp = req
        .send()
        .await
        .map_err(|e| NeevLoginError::Other(format!("/api/user request failed: {e}")))?;
    parse_console_json(resp, "/api/user").await
}

pub async fn fetch_neev_orgs(base: &str, bearer: &str) -> Result<Vec<NeevOrg>, NeevLoginError> {
    let url = format!("{}/api/orgs", base.trim_end_matches('/'));
    let resp = neev_get(&url, bearer)
        .send()
        .await
        .map_err(|e| NeevLoginError::Other(format!("/api/orgs request failed: {e}")))?;
    Ok(parse_console_json::<Vec<NeevOrg>>(resp, "/api/orgs")
        .await?
        .unwrap_or_default())
}

/// **404 = "no org config" = `Ok(None)`, not an error.** A config-less org is a
/// normal account. Precedent: auth/device_code.rs:166 maps 404 to a typed
/// NotEnabled rather than a failure.
///
/// Requires `x-org-id` — without it this 401s.
pub async fn fetch_neev_config(
    base: &str,
    bearer: &str,
    org_id: &str,
) -> Result<Option<NeevGatewayConfig>, NeevLoginError> {
    let url = format!("{}/api/config", base.trim_end_matches('/'));
    // accept_json: true — matches neev's acceptJson pipe on this endpoint (07 §2A).
    let resp = neev_get_org(&url, bearer, org_id, true)
        .send()
        .await
        .map_err(|e| NeevLoginError::Other(format!("/api/config request failed: {e}")))?;

    let Some(envelope) = parse_console_json::<NeevConfigEnvelope>(resp, "/api/config").await? else {
        return Ok(None);
    };
    let p = envelope.config.provider.opencode;

    // Gateway host is server-named — validate before it can become a bearer
    // destination. See §8.
    if !is_first_party_neev_url(&p.options.base_url, base) {
        tracing::warn!("neev: /api/config returned an off-console baseURL; ignoring");
        return Ok(None);
    }

    let models = p
        .whitelist
        .iter()
        .filter_map(|id| p.models.get(id).map(|m| (id.clone(), m.clone())))
        // ponytail: the harness is tool-driven and there is no capability flag
        // to carry `tool_call` downstream — so filter instead of modeling it.
        .filter(|(_, m)| m.tool_call)
        .collect();

    Ok(Some(NeevGatewayConfig {
        base_url: p.options.base_url,
        display_name: p.name,
        models,
    }))
    // NOTE: options.api_key is intentionally dropped — it is byte-identical to
    // the access_token already in the neev slot. See §8.
}

async fn parse_console_json<T: serde::de::DeserializeOwned>(
    resp: reqwest::Response,
    what: &str,
) -> Result<Option<T>, NeevLoginError> {
    let status = resp.status();
    if status.as_u16() == 404 {
        return Ok(None);
    }
    if status.as_u16() == 403 {
        return Err(NeevLoginError::BlockedClient);
    }
    if !status.is_success() {
        return Err(NeevLoginError::Other(format!(
            "{what} failed with HTTP {status}"
        )));
    }
    resp.json::<T>()
        .await
        .map(Some)
        .map_err(|e| NeevLoginError::Other(format!("invalid {what} response: {e}")))
}
```

Do not let a 404 enter a retry loop — `xai_grok_http::send_with_retry_escaping_pool` takes an `is_retryable` predicate; these calls use the plain `.send()` path, which is correct here.

`/api/usage/report` (§8 of the protocol) and `/api/mcp/list` (§9) are out of scope: fire-and-forget metering with no bum consumer, and bum has its own MCP crate. Add when billing matters.

## 7. `ensure_fresh.rs`

A structural clone of `auth/codex/ensure_fresh.rs` with the names swapped. The parts that matter:

```rust
/// Typed material returned into request construction (mirror CodexAuthMaterial,
/// auth/codex/ensure_fresh.rs:24).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NeevAuthMaterial {
    pub bearer: String,
    /// `wrk_…` for `x-org-id` on console calls. NOT sent to the gateway.
    pub org_id: Option<String>,
}

impl NeevAuthMaterial {
    pub fn from_auth(auth: &GrokAuth) -> Self {
        Self {
            bearer: auth.key.clone(),
            org_id: auth.organization_id.clone(),
        }
    }
}

/// Ternary outcome — the distinction is load-bearing (CR-02,
/// auth/codex/ensure_fresh.rs:38-51). Collapsing `Unavailable` into `Unusable`
/// logs the user out on a network blip.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnsureFreshNeevResult {
    /// Fresh (or still hard-unexpired) material safe to put on the wire.
    Fresh(NeevAuthMaterial),
    /// Hard-expired, missing slot, permanent IdP clear — do not serve prepared key.
    Unusable,
    /// Lock/IO/timeout — keep the prepared token; not a permanent logout.
    Unavailable,
}
```

The body follows `ensure_fresh_codex_auth_at` (`:231`) line for line:

1. `refresh_mutex().lock().await` — in-process single-flight (`:199`).
2. `try_lock_auth_file_async(auth_file, AUTH_LOCK_TIMEOUT)` → `None` ⇒ `Unavailable`.
3. `lock.still_live(auth_file)` recheck ⇒ `Unavailable` if stale.
4. `read_provider_auth_store(auth_file, PROVIDER_NEEV)` — `Ok(None)` ⇒ `Unusable`, `Err` ⇒ `Unavailable`.
5. `select_provider_access_token(&store)` ⇒ `Unusable` if none; `auth_mode != Oidc` ⇒ `Unusable`.
6. `if !is_expired(&auth) { return Fresh(NeevAuthMaterial::from_auth(&auth)) }` — this is the normal path for a year. `is_expired` already applies the 300s early-invalidation buffer (`auth/model.rs:437`, `DEFAULT_EARLY_INVALIDATION_SECS = 300` at `:8`), which **is** neev's 5-minute window. Add no constant.
7. Otherwise `neev_token_exchange` → `RefreshOutcome` → persist via `mutate_provider_store_or_prune_with_lock(auth_file, &lock, AuthProvider::Neev, …)`; permanent ⇒ `clear_provider_slot_with_lock`.

Copy the `#[cfg(any(test, feature = "unstable", debug_assertions))]` synthetic-hook block verbatim (`:123-177`, `:318-355`) — it is the blessed test gate; do not invent a feature flag.

A revoked-but-unexpired token looks perfectly fresh here for a year and only fails at the gateway with a 401. That is correct; do not build revocation detection into the store — let the sampler's 401 path drive re-login.

## 8. `options.apiKey` and `options.baseURL` — do not persist either

Two independent reasons, both load-bearing:

- **`apiKey` is byte-identical to `access_token`**, which is already in the neev slot at 0600. Persisting it elsewhere duplicates the account secret. `ModelEntry.api_key` is `Serialize` and the models cache manager writes the whole entry map to `models_cache.json` — a non-auth-store file. That reproduces exactly the world-readable-token weakness the port exists to avoid.
- **BYOK semantics.** Setting `ModelEntry.api_key` flips `has_own_credentials()` → `ModelByok::Byok` and the config-override path auto-sets `supported_in_api` when `api_key.is_some()`, un-hiding neev models for users with no neev account.

Leave `api_key: None` and let the bearer flow from the auth slot through `resolve_provider_route`'s `credential_slot`, the same path xai and codex use. Free, precisely because the two secrets are the same string. If they ever diverge server-side, revisit — leave that in a comment.

`baseURL` is server-owned and can move. It is fetched at login/prepare and held in `EndpointsConfig`; the validation guard:

```rust
/// First-party NeevCloud gateway hosts.
///
/// The baseURL comes from a **server response**, so the console names the host
/// that receives the account bearer. Mirror the codex allowlist discipline
/// (is_first_party_codex_url, agent/config.rs:4499): host AND path prefix —
/// host-only is documented there as insufficient.
///
/// Unlike Codex, the gateway shares a host with the console (`/zen/go/v1` on
/// code.neevcloud.com), so the path check is the only thing distinguishing them.
pub fn is_first_party_neev_url(url: &str, console_base: &str) -> bool {
    let (Ok(parsed), Ok(console)) = (reqwest::Url::parse(url), reqwest::Url::parse(console_base))
    else {
        return false;
    };
    let (Some(host), Some(cfg_host)) = (parsed.host_str(), console.host_str()) else {
        return false;
    };
    parsed.scheme() == "https"
        && host.eq_ignore_ascii_case(cfg_host)
        && parsed.path().starts_with("/zen/go/v1")
}
```

Honest note: for xai/codex, `session_oauth_allowed` distinguishes a session token from a BYOK key. Since NeevCloud's access_token *is* its apiKey, that distinction is cosmetic here. Implement it anyway — defense in depth against a hostile `/api/config`, and it keeps the match arms honest.

## 9. Provider registration — the 2→3 widening

Sequencing matters. Do this in order:

**Step 1 — exhaustive-match the `matches!` guards *before* adding the variant.** These are two-way tests that compile fine and silently take the **xAI** branch for a third variant. A neev logout clearing the xai slot, or a neev login triggering xAI's `post_login_sync`, are the failure modes:

| Site | Guard |
|---|---|
| `auth/flow.rs:901` | `matches!(provider, Some(AuthProvider::Codex))` → else falls to xAI |
| `auth/flow.rs:780-782` | `report_signed_in_for_provider` label match |
| `auth/flow.rs:1077`, `:1085`, `:1251`, `:1266` | logout / `api_key_still_set` / post-logout copy |
| `xai-grok-shell/src/extensions/auth.rs:163` | `if matches!(provider, crate::auth::AuthProvider::Xai)` (note: `src/extensions/`, **not** `src/auth/extensions/`) |

Convert each to an exhaustive `match` with no `_` arm. This is the highest-value structural change in the port. (Line numbers from research; `// VERIFY:` each before editing.)

**Step 2 — add the variant and let the compiler enumerate the rest.**

```rust
// auth/model.rs — next to PROVIDER_XAI:17 / PROVIDER_CODEX:20
/// Stable wire key for the NeevCloud provider slot in multi-slot `auth.json`.
pub const PROVIDER_NEEV: &str = "neev";

pub enum AuthProvider {
    Xai,
    Codex,
    /// NeevCloud device-code slot (`providers.neev`).
    Neev,
}

// as_str():36  → Self::Neev => PROVIDER_NEEV
// label():44   → Self::Neev => "NeevCloud"
// parse():52   → PROVIDER_NEEV => Some(Self::Neev)

/// All provider slots in status display order (xAI, Codex, NeevCloud).
pub fn all() -> [Self; 3] {
    [Self::Xai, Self::Codex, Self::Neev]
}
```

`all()` is the keystone. `storage.rs:562` (`clear_all_provider_slots`, i.e. `bum logout --all`) iterates it and generalizes for free. The on-disk `AuthDocument.providers: BTreeMap<String, AuthStore>` (`auth/model.rs:80`) needs **no schema change** — a third key is additive.

**Compile-hard sites (good — they are your checklist):**

| File | Change |
|---|---|
| `auth/status.rs:38` | `providers: [ProviderAuthStatus; 2]` → `; 3`; add the third element to the literal arrays in `from_document:75` and `empty:91`. `format_auth_status:234` iterates — no change. |
| `auth/meta.rs:31` | `ProviderAuthMetaSlots { xai, codex }` → add `#[serde(default)] pub neev: ProviderSlotUsableMeta` (default keeps older ACP clients deserializing; there is a regression test for exactly that). `from_report:40`'s exhaustive match fails to compile — good. |
| `xai-grok-pager-bin/src/main.rs` | Two near-identical `AuthProviderArg → AuthProvider` maps (login ~`:1754`, logout ~`:1779`). Both are wildcard-free. |
| `auth/storage.rs` `write_fixture_auth_document` | `#[cfg(test)]` helper with positional `(xai, codex)` slots — add a third param. |

**Silent sites (compiler will NOT catch these — grep `"codex"` as a literal before declaring done):**

| File | Symptom if missed |
|---|---|
| `xai-grok-pager/src/app/cli.rs:12` `AuthProviderArg` | `--provider neev` doesn't parse |
| `pager/src/app/actions.rs:43` `parse_provider_wire_id` | returns `None` → TUI silently drops the missing-provider question |
| `pager/src/app/app_view.rs:539` `ProviderAuthUsableSnapshot` + `usable_for_wire:572` | `/model` needs-login badge never lights up |
| `pager/src/app/dispatch/session/lifecycle.rs:302-305` `provider_label` | `_ =>` falls back to xAI — mislabels rather than fails |
| `pager/src/app/dispatch/session/lifecycle.rs:1433` `let is_codex = provider == "codex"` | **"Login now" on a neev model launches an xAI OAuth flow.** Generalize: `let is_cli_primary = matches!(provider.as_str(), "codex" \| "neev");` |
| `pager/src/slash/commands/model.rs:199-202` `model_meta_provider` | catalog meta allowlist |
| `agent/config.rs:5563`, `:5587` provider_label `_ =>` | error copy says xAI |

### `flow.rs` login branch

Insert **before** the xAI path so it early-returns ahead of `post_login_sync` — never feed a non-xAI principal into xAI team managed-config (the rationale for the Codex branch at `auth/flow.rs:901`):

```rust
if matches!(provider, Some(super::AuthProvider::Neev)) {
    if devbox {
        anyhow::bail!("Devbox login is not available for NeevCloud. Use `bum login --provider neev`.");
    }
    if oauth {
        anyhow::bail!("NeevCloud has no browser sign-in. Use `bum login --provider neev`.");
    }
    let _ = device_auth; // device-code is the only NeevCloud flow.
    let auth_file = grok_home::grok_home().join("auth.json");
    let auth = super::neev::run_neev_login(&auth_file)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    report_signed_in_for_provider(&auth, Some(super::AuthProvider::Neev));
    return Ok(());
}
```

Bare `bum logout` stays fail-closed (Err + usage, zero mutation) so a third slot can never be silently wiped. `bum auth status` needs no new code beyond the arity fix — `format_auth_status` iterates and has a `format_never_emits_token_material` test. Keep it that way.

## 10. Routing arm

```rust
// agent/config.rs:3394 — wire id MUST equal PROVIDER_NEEV (convention-only,
// unenforced by the type system: two parallel enums bridged by a &'static str).
pub enum ModelProvider {
    #[default]
    Xai,
    Codex,
    /// NeevCloud Zen gateway (OpenAI-compatible chat completions).
    Neev,
}
// as_str():3404        → Self::Neev => "neev"
// display_label():3412 → Self::Neev => "NeevCloud"
```

`ModelProvider` derives `Default = Xai` with `#[serde(default)]` on every `provider` field: a neev row that **omits** `provider` silently becomes an xAI model pointed at a neev base_url. An unknown variant string does error (no `#[serde(other)]`), so typos fail loudly; omissions do not.

`EndpointsConfig` (`agent/config.rs:148`) gains two fields, mirroring the `codex_base_url:162` / `resolve_codex_base_url:331` / env-default `:569` triple exactly:

```rust
/// NeevCloud console base (`https://code.neevcloud.com`). Env: `NEEVCLOUD_CONSOLE_URL`.
/// This is the CONSOLE, not the inference gateway.
pub neev_console_url: String,
/// Gateway base from `/api/config` `options.baseURL`, cached at login/prepare.
/// Blank → `{console}/zen/go/v1`, so catalog stamping never yields a blank base.
pub neev_base_url: String,

pub fn resolve_neev_console_url(&self) -> String {
    let s = self.neev_console_url.trim().trim_end_matches('/');
    if s.is_empty() { crate::auth::neev::NEEV_CONSOLE_URL_DEFAULT.to_owned() } else { s.to_owned() }
}

pub fn resolve_neev_base_url(&self) -> String {
    if self.neev_base_url.trim().is_empty() {
        format!("{}/zen/go/v1", self.resolve_neev_console_url())
    } else {
        self.neev_base_url.clone()
    }
}
```

Then the three matches inside `resolve_provider_route` (`agent/config.rs:4458`) go non-exhaustive and the build tells you where:

```rust
let credential_slot = match provider {
    ModelProvider::Xai => crate::auth::PROVIDER_XAI,
    ModelProvider::Codex => crate::auth::PROVIDER_CODEX,
    ModelProvider::Neev => crate::auth::PROVIDER_NEEV,
};
let base_url = model_base_url_override
    .map(str::trim)
    .filter(|s| !s.is_empty())
    .map(str::to_owned)
    .unwrap_or_else(|| match provider {
        ModelProvider::Xai => endpoints.resolve_inference_base_url(),
        ModelProvider::Codex => endpoints.resolve_codex_base_url(),
        ModelProvider::Neev => endpoints.resolve_neev_base_url(),
    });
let session_oauth_allowed = match provider {
    ModelProvider::Xai => crate::util::is_first_party_xai_url(&base_url),
    ModelProvider::Codex => is_first_party_codex_url(&base_url, endpoints),
    ModelProvider::Neev => crate::auth::neev::is_first_party_neev_url(
        &base_url,
        &endpoints.resolve_neev_console_url(),
    ),
};
```

`resolve_provider_route`'s doc (`:4440-4447`) explicitly forbids parallel base-URL tables and declares the function pure. NeevCloud's runtime-fetched baseURL tempts a bypass — resist. Seed `EndpointsConfig.neev_base_url` from the console config at login/ensure-fresh, and let the pure function read it. Do not make it async; every caller would have to change.

The rest of the routing widening (each fails to compile, so the compiler is the checklist):

- `resolve_credentials_for_provider:4812` — signature grows `neev_session_key: Option<&str>`; slot match at `:4834`. The two `provider == ModelProvider::Xai` `XAI_API_KEY` fall-throughs (`:4844`, `:4868`) already exclude non-Xai, so neev lands in the fail-closed `else` branches — no D-15 leak.
- Callers: `resolve_credentials:4546`, `session_key_for_model_provider:4564`, `prepare_sampling_credentials:4579`, `try_resolve_model_credentials:4966`, `agent/models.rs:959` (via `prepare_sampling_credentials`), `agent/subagent/mod.rs:1033`, `agent/mvp_agent/agent_ops.rs:1108` + `:1115`. The tuple is **positional** `(xai, codex)` — the failure mode of getting it wrong is "neev requests silently authenticate with the Codex token", which `models.rs:952-953`'s own comment names as the thing to never do.
- `snapshot_neev_session_key_from_auth_store` + `invalidate_neev_session_key_snapshot`, cloned from `:4700` / `:4756`. The cache keys on (path, mtime, len, epoch) — **call the invalidate after login** or the first post-login model switch still sees an empty slot (a same-second write can collide on mtime+len).
- `agent/handlers/model_switch.rs:26-29` — `Neev => crate::auth::PROVIDER_NEEV`. Nothing else: `provider_slot_usable` / `credential_usable` are provider-agnostic, `missing_provider_gate_error:5602` is already generic (its suggestion string becomes `bum login --provider neev` for free), and the xAI-only `AuthManager` fallback at `:38-42` correctly excludes neev — extending it would let a stale xAI session gate a neev switch.
- `xai-grok-shell/src/session/acp_session_impl/sampler_turn.rs:294-322` — clone the `codex_session_oauth` gate (it calls `crate::agent::config::ensure_fresh_codex_auth()` at `:299`, so the neev twin must be re-exported from `agent::config` the same way): `provider == Some(Neev) && auth_type == SessionToken && is_first_party_neev_url(…)` → `ensure_fresh_neev_auth().await` → `Fresh` ⇒ overwrite `api_key`; `Unusable` ⇒ `api_key = None`; `Unavailable` ⇒ keep prepared. Do **not** clone `inject_chatgpt_account_id_header:4779` — the gateway wants only `Authorization: Bearer` plus the fidelity headers below; `x-org-id` belongs on console calls.

### Gateway fidelity (identity B) — the one sampler-layer change

Where the neev `SamplerConfig` is built, fold in identity B (07 §2B). `extra_headers` is an
`IndexMap<String, String>` (`xai-grok-sampler/src/client.rs:2098`):

```rust
for (name, value) in crate::auth::neev::fidelity::gateway_headers(&session_id, &request_id) {
    cfg.extra_headers.insert(name.to_string(), value);
}
// The gateway UA canNOT ride extra_headers — the sampler clobbers it. See below.
cfg.extra_headers
    .insert("user-agent".to_owned(), fidelity::gateway_user_agent().into_owned());
```

`session_id` is one `fidelity::new_session_id()` per bum session (cache it — do not regenerate per request);
`request_id` is a fresh `fidelity::new_request_id()` per outbound request. Never replay a captured id (07 §6).

**The last line only works after a ~2-line sampler change.** `SamplingClient::new` applies `extra_headers`
(`client.rs:443-449`) and *then* runs an unconditional `headers.insert(USER_AGENT, …)` at `:500` — `insert`
replaces, so a UA passed through `extra_headers` is silently dropped. The four `x-opencode-*` headers are
unaffected and ride fine. 07 §4 lays out both options — `origin_client` (supported, but composes a
`… grok-shell/<ver> (os; arch)` UA that cannot be byte-exact) versus making `:500` a conditional insert — and
recommends the conditional insert. Apply it, or the gateway UA is `grok-shell/…` no matter what this code says.
That is also the first thing to check in the 07 §8 capture diff.

Otherwise nothing in the sampler/HTTP/stream layer changes. `SamplingClient::endpoint()` is `format!("{base}/{path}")` after `trim_end_matches('/')`, so `https://code.neevcloud.com/zen/go/v1` + `chat/completions` composes correctly; `AuthScheme::Bearer` is `#[default]`; the `Usage` type (`sampling-types/src/types.rs:536-570`) already has `prompt_tokens`/`completion_tokens`/`total_tokens` + `completion_tokens_details.reasoning_tokens`, field-for-field the verified neev response. Keep `supports_backend_search: false` (the default) so the xAI-only `search_parameters` field stays off the wire.

## 11. Catalog ingestion

`resolve_model_list` is **sync** and takes only `&Config` — it cannot do async HTTP. So: fetch `/api/orgs` → `/api/config` at login, persist the distilled result to a non-secret sidecar, and let the sync path read it.

```rust
/// `~/.bum/neev_models.json` — a NON-SECRET sidecar. Contains no apiKey (see §8),
/// so it does not belong in auth.json and does not need the credential store.
/// Still write it via crate::util::secure_file::open_secure_file (the 0600
/// truncating open auth/storage.rs:702 uses; on Windows pair it with
/// set_windows_secure_permissions, storage.rs:718) — 0600 is free and there is
/// no reason to broadcast the account's model whitelist.
#[derive(Serialize, Deserialize)]
struct NeevCatalogCache {
    base_url: String,
    models: Vec<NeevCatalogRow>,
}

#[derive(Serialize, Deserialize)]
struct NeevCatalogRow {
    /// Lowercase REQUEST id from `whitelist[]` (`minimax-m2`). The response
    /// echoes `MiniMax-M2` — never round-trip-compare model ids.
    model: String,
    name: String,
    context: u64,
    output: Option<u32>,
}
```

Mapping `/api/config` → `ModelEntryConfig`:

| `/api/config` | catalog | note |
|---|---|---|
| `whitelist[]` | include filter | only build entries for whitelisted ids |
| `models.<id>.limit.context` | `context_window: NonZeroU64` | **required, non-zero**; use the `unwrap_or_else(\|\| NonZeroU64::new(200_000).expect("200000 is non-zero"))` fallback idiom at `agent/config.rs:3479-3481` — never `NonZeroU64::new(v).unwrap()` on a server value |
| `models.<id>.limit.output` | `max_completion_tokens: Option<u32>` | exact fit |
| `models.<id>.name` | `name` | "MiniMax M2" — the pretty casing lives here, not in `model` |
| `provider.opencode.name` | `description` | "NeevCode Zen" |
| `options.baseURL` | `model_base_url_override` into `resolve_provider_route` | already trims + empty-filters + takes precedence (`:4467-4470`) |
| `api_backend` | **omit** | `ChatCompletions` is `#[default]` |
| `auth_scheme` | **omit** | `Bearer` is `#[default]` |
| `supported_in_api` | **`false`** | `visible_for_auth()` = `!hidden && (is_session_auth \|\| supported_in_api)` (`:4003`) — `false` correctly hides neev models from API-key-only users, same as `grok-build` |
| `tool_call` | **drop the field, use as a filter** | no capability flag exists to carry it downstream |
| `reasoning` | **drop** | `supports_reasoning_effort` describes the low/med/high *menu*, not "reasons internally". Mapping it would make the picker offer an effort submenu the gateway never advertised |
| `cost.input` / `cost.output` | **drop** | `grep -n "cost\|price\|per_token"` over `agent/config.rs` returns zero hits. No consumer exists. |

**The injection seam is the trap.** In `resolve_model_list` (`agent/config.rs:3132`) static defaults are added only in the `else` of the no-prefetch branch, and the Codex re-append (`:3185-3201`) runs only **inside** `if let Some(mut prefetched)`. Codex survives both paths because it is baked into `default_models.json`. Neev rows come from a runtime cache, so an append placed inside the prefetch branch is invisible to every `resolve_model_list(cfg, None)` caller — `xai-grok-shell/src/cli_models.rs:40` (the `models` CLI listing) and ~15 tests.

Append **after** the whole `if let Some(prefetched)` block, unconditional, guarded by `if !cfg.endpoints.has_custom_endpoint()` exactly like the Codex re-append, and use remove-then-insert (`shift_remove` then `insert`, `:3192-3201`) so a remote catalog cannot rebind a neev key to `provider: xai`. `IndexMap` order is semantic.

Also re-check `WR-02` (`:3202-3212`): `has_xai` is computed over `resolved.values()`, and adding neev rows changes what "first entry" means for `resolve_default_model`'s `first_or_fallback`. Verify the guard still holds with three providers.

`default_models.json` gets **no neev rows** — the list is per-account. If you decide to bundle a static fallback row instead, note that `xai-grok-models/src/lib.rs`'s `default_model_is_grok_build_and_catalog_has_four_plus` (`:76-98`) is additive-safe: it asserts `models.len() >= 4` plus `contains` for the four current ids, so an extra row passes unchanged. Nothing to update.

Open question worth deciding before building: fetch `/api/config` once at login and cache, or refresh on every startup via the existing prefetch thread (`agent/models.rs:1597`)? Login-only is the smaller diff and the entries are stable; startup-refresh picks up whitelist changes without a re-login. Default to login-only + refresh-on-401 unless told otherwise.

## 12. Tests

Conventions: behavior-phrase names (no `test_` prefix), `#[cfg(test)] mod tests` inline for pure logic, `crates/codegen/xai-grok-shell/tests/auth_neev_lifecycle.rs` for lifecycle (mirror `auth_codex_lifecycle.rs`, including its module doc restating the `BUM_HOME`/`OnceLock` hygiene — `grok_home()` is a `OnceLock`, so every neev API must stay path-taking). `serial_test::serial` + `EnvVarGuard` on anything env-touching. Run per-crate:

```
cargo test -p xai-grok-shell --test auth_neev_lifecycle <name>
```

### The poll loop

`start_paused = true` makes `tokio::time::sleep` auto-advance, so a 5s interval and a 600s deadline cost no wall-clock. Drive it through `run_neev_device_poll_only_with_base` against an axum mock (the harness at `auth/device_code.rs:789`, `spawn_token_server`, is the one to copy).

```rust
// crates/codegen/xai-grok-shell/tests/auth_neev_lifecycle.rs

/// Mock console that returns a scripted sequence of token-endpoint responses.
fn spawn_neev_console(script: Vec<(u16, &'static str)>) -> (String, JoinHandle<()>) { /* axum */ }

#[tokio::test(start_paused = true)]
#[serial_test::serial]
async fn poll_pending_then_success_persists_only_neev_slot() {
    let tmp = tempfile::tempdir().unwrap();
    let auth_file = tmp.path().join("auth.json");
    seed_xai_and_codex_slots(&auth_file);

    let (base, _srv) = spawn_neev_console(vec![
        (400, r#"{"error":"authorization_pending"}"#),
        (400, r#"{"error":"authorization_pending"}"#),
        (200, r#"{"access_token":"sk-fake","refresh_token":"sk-fake","expires_in":31536000}"#),
    ]);

    let auth = run_neev_device_poll_only_with_base(&auth_file, &base, NEEV_CLIENT_ID)
        .await
        .expect("login");

    assert_eq!(auth.key, "sk-fake");
    // Oidc + Some(refresh_token) or token_type.rs:28-29 degrades it to
    // LegacySession and status.rs reports usable:false.
    assert_eq!(auth.auth_mode, AuthMode::Oidc);
    assert_eq!(auth.refresh_token.as_deref(), Some("sk-fake"));

    // AuthDocument is pub(crate) — an integration test cannot read it. Use the
    // public path-taking API, exactly like auth_codex_lifecycle.rs does.
    assert!(read_provider_auth_store(&auth_file, PROVIDER_NEEV).unwrap().is_some());
    // Slot isolation — the single most valuable assertion in the file.
    assert_eq!(read_provider_auth_store(&auth_file, PROVIDER_XAI).unwrap(), Some(seeded_xai()));
    assert_eq!(read_provider_auth_store(&auth_file, PROVIDER_CODEX).unwrap(), Some(seeded_codex()));
    #[cfg(unix)]
    assert_eq!(fs::metadata(&auth_file).unwrap().permissions().mode() & 0o777, 0o600);
}

#[tokio::test(start_paused = true)]
#[serial_test::serial]
async fn slow_down_increments_interval_by_five_seconds() {
    // Script: slow_down, then success. Assert the gap between poll 1 and poll 2
    // is server_interval, and poll 2 → poll 3 is server_interval + 5.
    // The mock records tokio::time::Instant per hit; paused time makes the
    // arithmetic exact rather than flaky.
}

#[tokio::test(start_paused = true)]
#[serial_test::serial]
async fn expired_token_is_terminal_and_writes_nothing() {
    let (base, _srv) = spawn_neev_console(vec![(400, r#"{"error":"expired_token"}"#)]);
    let err = run_neev_device_poll_only_with_base(&auth_file, &base, NEEV_CLIENT_ID)
        .await
        .unwrap_err();
    assert!(err.to_string().contains("bum login --provider neev"));
    assert!(!auth_file.exists()); // persist-only-after-success
}

#[tokio::test(start_paused = true)]
#[serial_test::serial]
async fn access_denied_is_terminal_and_writes_nothing() { /* mirror */ }

/// Codex treats 403 as *pending* (auth/codex/device.rs:249-251) — copying that
/// here spins for 10 minutes and then reports a timeout instead of the real
/// cause, whatever the 403 turns out to be.
#[tokio::test(start_paused = true)]
#[serial_test::serial]
async fn poll_403_surfaces_as_blocked_client_not_pending() {
    let (base, _srv) = spawn_neev_console(vec![(403, r#"{"error_code":1010}"#)]);
    let err = run_neev_device_poll_only_with_base(&auth_file, &base, NEEV_CLIENT_ID)
        .await
        .unwrap_err();
    assert!(matches!(err, NeevLoginError::BlockedClient));
}

/// Deadline comes from the server's expires_in (floored at 600s), not a flat
/// 15-minute MAX_WAIT like Codex.
#[tokio::test(start_paused = true)]
#[serial_test::serial]
async fn poll_times_out_at_expires_in() {
    // Mock: expires_in = 600, always authorization_pending.
    let started = tokio::time::Instant::now();
    let err = run_neev_device_poll_only_with_base(&auth_file, &base, NEEV_CLIENT_ID)
        .await
        .unwrap_err();
    assert!(matches!(err, NeevLoginError::Timeout));
    assert!(started.elapsed() >= Duration::from_secs(600));
}
```

### Token expiry

`is_expired` (`model.rs:437`) and `is_expired_with_buffer` (`:444`) are `pub(crate)`; `early_invalidation` (`:429`) is `pub(super)` (visible to `auth::*` descendants only). Either way they are unreachable from an integration binary, so these tests live **inline** in `auth/neev/device.rs`.

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::model::{is_expired, is_expired_with_buffer};

    fn tokens(expires_in: Option<u64>) -> NeevTokenResponse {
        NeevTokenResponse {
            access_token: "sk-fake".into(),
            refresh_token: Some("sk-fake".into()),
            expires_in,
        }
    }

    /// The whole point of not reusing codex/claims.rs: an opaque sk-… token has
    /// no JWT `exp`, and resolve_expires_at's CONSERVATIVE_EXPIRES_FALLBACK
    /// (5 min, claims.rs:16) would re-refresh a 1-year token every 5 minutes.
    #[test]
    fn expires_at_comes_from_expires_in_not_a_jwt() {
        let before = Utc::now();
        let auth = grok_auth_from_neev_tokens(
            "https://code.neevcloud.com",
            NEEV_CLIENT_ID,
            &tokens(Some(31_536_000)), // 1 year
            None,
            None,
        );
        let exp = auth.expires_at.expect("expires_at set");
        let secs = (exp - before).num_seconds();
        assert!((31_536_000 - 5..=31_536_000 + 5).contains(&secs), "got {secs}s");
        assert!(!is_expired(&auth));
    }

    /// 300s early-invalidation is already the default (model.rs:8) and IS neev's
    /// 5-minute window. This pins that no new constant crept in.
    #[test]
    fn token_inside_the_five_minute_buffer_is_early_expired_but_not_hard_expired() {
        let mut auth = grok_auth_from_neev_tokens(
            "https://code.neevcloud.com", NEEV_CLIENT_ID, &tokens(Some(0)), None, None,
        );
        auth.expires_at = Some(Utc::now() + chrono::Duration::seconds(120)); // < 300s

        assert!(is_expired(&auth), "inside buffer → refresh");
        assert!(
            !is_expired_with_buffer(&auth, chrono::Duration::zero()),
            "still hard-valid → ensure_fresh must return Fresh, not Unusable"
        );
    }

    /// A missing expires_in must not mint a token that never expires.
    #[test]
    fn absent_expires_in_yields_an_immediately_expired_credential() {
        let auth = grok_auth_from_neev_tokens(
            "https://code.neevcloud.com", NEEV_CLIENT_ID, &tokens(None), None, None,
        );
        // expires_at = now + 0 → Some, so TOKEN_TTL's 30-day create_time
        // fallback (model.rs:444) never applies. Fail closed, not open.
        assert!(is_expired_with_buffer(&auth, chrono::Duration::zero()));
    }

    /// Both fields, always — Oidc without a refresh_token silently degrades to
    /// the unrefreshable LegacySession (token_type.rs:28-29) and reports
    /// usable:false in auth status.
    #[test]
    fn refresh_token_is_populated_even_when_the_server_omits_it() {
        let mut t = tokens(Some(31_536_000));
        t.refresh_token = None;
        let auth = grok_auth_from_neev_tokens(
            "https://code.neevcloud.com", NEEV_CLIENT_ID, &t, None, None,
        );
        assert_eq!(auth.refresh_token.as_deref(), Some("sk-fake"));
    }

    #[test]
    fn verification_url_concatenates_the_returned_path_onto_the_base() {
        // The server returns a PATH. codex_device_verify_url builds an absolute
        // URL from the issuer — a structural copy prints a broken link.
        assert_eq!(
            neev_device_verify_url("https://code.neevcloud.com/", "/device?code=ABCD-1234"),
            "https://code.neevcloud.com/device?code=ABCD-1234"
        );
    }

    #[test]
    fn bare_path_verification_url_is_rejected_after_concat() {
        assert!(validate_verification_url("/device?code=ABCD").is_err());
        assert!(validate_verification_url("http://evil.example/device").is_err());
        assert!(validate_verification_url("https://code.neevcloud.com/device").is_ok());
    }
}
```

### Fidelity

`fidelity.rs` carries its own inline tests (07 §3, §6): the console UA matches the captured string, `gateway_headers`
carries no UA, and the ids match the captured shape and never repeat. Add one to `xai-grok-sampler` asserting
`extra_headers["user-agent"]` survives into `default_headers` — that is the regression test for the §10 clobber fix,
and it belongs next to `new_applies_extra_headers` (`client.rs:2262-2266`).

**No test may assert bum's live traffic matches production neev** (07 §7): that test goes red when the vendor ships.
Assert against the captured constants only. The capture-diff (07 §8) is a manual pre-release ritual, not CI.

`merge_neev_refresh_response` identity-preservation and `classify_terminal` get the same two inline tests `auth/codex/refresh.rs:209-235` has. Then mirror the neev arms into `tests/auth_multi_slot.rs`, `tests/provider_routing.rs` (copy `resolve_provider_route_codex_default:203` and `codex_model_routes_to_codex_backend_with_codex_token:453`), `tests/model_catalog.rs` (copy `to_acp_model_info_sets_meta_provider:615`, and assert `api_key.is_none()` on neev entries so the §8 leak cannot regress), and `tests/model_switch_gate.rs`.

## 13. Build order

1. Exhaustive-match the `matches!` guards (§9 step 1). Nothing else compiles-and-is-wrong afterwards.
2. Add `AuthProvider::Neev` + `PROVIDER_NEEV` + `all() -> [Self; 3]`. Follow the compiler errors; fix the silent sites from the grep list.
3. `auth/neev/fidelity.rs` (self-contained, unit-tested, no deps on the rest — 07 §3/§6), then `http.rs` (timeout + org header + `console_headers`), then `device.rs`.
4. `console.rs`, `refresh.rs`, `ensure_fresh.rs`.
5. `ModelProvider::Neev` + `EndpointsConfig` + `resolve_provider_route` + the credential-tuple widening.
6. Catalog ingestion + the after-the-if-let injection seam.
7. Gateway fidelity last (§10): the `client.rs:500` conditional insert + `extra_headers` wiring. It is the only edit to vendored xAI code, it changes nothing functional, and doing it last keeps it out of the blast radius while the wire is still being debugged.

Doing (3) before (1)–(2) means debugging silent xAI-fallthrough bugs on top of live wire behaviour.

## 14. Deliberately skipped

- `neev/browser.rs` — device-code only. No PKCE, no loopback ports, no `localhost`-vs-`127.0.0.1` redirect quirk.
- `b3` / `traceparent` fidelity headers — trace context, not client identity; zero fidelity value at real cost. Rationale and the existing-injector survey: 07 §5.
- `Accept-Encoding` / `Connection` fidelity — reqwest owns both; a knowingly accepted gap on two headers nobody identifies clients by (07 §2A).
- The `ulid` crate — `fidelity.rs` hand-rolls 20 lines on the existing `rand` dep for a header the gateway does not validate (07 §6).
- `neev/claims.rs` — opaque token, not a JWT.
- Wiring `NeevRefresher` into `build_refresher` — Codex doesn't either; it would drag the xAI refresh path into the blast radius for zero gain.
- `POST /api/usage/report` metering — fire-and-forget in neev, no bum consumer. Add when billing matters.
- `GET /api/mcp/list` — bum has its own MCP crate.
- Cost/pricing catalog fields — zero consumers exist.
- Per-provider `/v1/models` prefetch fan-out — the fetch pipeline is single-URL/single-credential by construction and NeevCloud has no `/v1/models` anyway. Add if the whitelist starts churning faster than login.
- Any new `Provider` trait or registry — `AuthProvider` + `TokenRefresher` already are it. A third impl is not the moment to generalize. Add one when a fourth provider arrives.

## 15. Process

`AGENTS.md:509-519` forbids direct edits outside a GSD entry point. This is a new phase (the tree is at Phase 7 of 9), so it needs `NN-CONTEXT.md` (with the §10 base-URL decision locked), `NN-RESEARCH.md` (the verified protocol block *is* the research), `NN-PATTERNS.md` as a closest-analog table where nearly every row's analog is the codex twin at "exact" quality, then fine-grained `NN-MM-PLAN.md`/`SUMMARY.md` pairs. Enter via `/gsd-execute-phase`.

One scope flag worth raising before any of this: `PROJECT.md`'s Out of Scope says *"Supporting arbitrary third-party providers beyond xAI + Codex/OpenAI in v1 — multi-provider architecture should not block more later, but only these two ship in v1."* NeevCloud is exactly that third provider. The architecture does not block it (which is what that line reserves), but v1-vs-post-v1 is a decision, not an implementation detail.

Gate before calling it done: `cargo fmt --all` + `cargo clippy -p xai-grok-shell` + the per-test invocations above. Never an unfiltered workspace run. Never `std::fs::canonicalize` (clippy denies it — use `dunce::canonicalize`).
