//! ChatGPT / Codex OAuth (browser PKCE + device-code) for the `providers.codex` slot.
//!
//! In-tree mirror of openai/codex `codex-rs/login` wire contract:
//! - issuer `https://auth.openai.com`
//! - public `client_id`
//! - loopback bind `127.0.0.1:1455` (fallback `1457`), redirect host **`localhost`**
//! - device endpoints under `{issuer}/api/accounts/deviceauth/*`
//!
//! Never imports `~/.codex`. Never writes the xAI slot. Never runs Platform
//! `obtain_api_key` token-exchange (COVERAGE OPT-OUT).

mod browser;
mod claims;
mod device;

pub use browser::{
    CodexAuthorizeSession, CodexLoginError, CodexTokenResponse, apply_codex_oauth_callback,
    build_codex_authorize_url, codex_redirect_uri, exchange_codex_authorization_code,
    persist_codex_tokens, run_codex_browser_login,
};
pub use claims::{
    CONSERVATIVE_EXPIRES_FALLBACK, CodexTokenClaims, decode_codex_claims, resolve_expires_at,
};
pub use device::{
    codex_device_token_url, codex_device_usercode_url, codex_device_verify_url,
    run_codex_device_login, run_codex_device_login_with_base, run_codex_device_poll_only_with_base,
};

use std::path::Path;

use crate::auth::GrokAuth;

/// OpenAI / ChatGPT auth issuer (fixed paths — no OIDC discovery).
pub const CODEX_ISSUER: &str = "https://auth.openai.com";
/// Public Codex CLI client id (ChatGPT OAuth).
pub const CODEX_CLIENT_ID: &str = "app_EMoamEEZ73f0CkXaXp7hrann";
/// Stable multi-slot scope key under `providers.codex`.
pub const CODEX_AUTH_SCOPE: &str = "https://auth.openai.com::app_EMoamEEZ73f0CkXaXp7hrann";
/// Preferred loopback bind port (openai/codex DEFAULT_PORT).
pub const CODEX_PREFERRED_PORT: u16 = 1455;
/// Fallback loopback bind port when 1455 is busy.
pub const CODEX_FALLBACK_PORT: u16 = 1457;
/// Product originator query param on the authorize URL.
pub const CODEX_ORIGINATOR: &str = "bum";
/// OAuth scopes (mirror openai/codex authorize URL).
pub const CODEX_SCOPES: &str =
    "openid profile email offline_access api.connectors.read api.connectors.invoke";

/// Run Codex login (browser PKCE default; device when `force_device`).
///
/// Persists only `providers.codex` under `auth_file`. Does not touch xAI.
pub async fn run_codex_login(
    auth_file: &Path,
    force_device: bool,
) -> Result<GrokAuth, CodexLoginError> {
    if force_device {
        run_codex_device_login(auth_file).await
    } else {
        run_codex_browser_login(auth_file).await
    }
}
