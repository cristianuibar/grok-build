pub(crate) mod attribution;
mod config;
/// ChatGPT / Codex OAuth (browser PKCE + deviceauth) → `providers.codex`.
pub mod codex;
pub mod credential_provider;
#[path = "devbox_login_stub.rs"]
pub(crate) mod devbox_login;
pub mod device_code;
pub mod error;
mod external_auth;
mod flow;
mod jwt;
pub(crate) mod manager;
mod model;
pub mod oidc;
pub(crate) mod recovery;
pub(crate) mod refresh;
mod status;
mod storage;
pub(crate) mod token_type;
pub(crate) use config::LEGACY_AUTH_SCOPE;
pub use config::{
    ForceLoginTeam, GrokComConfig, OAuth2ProviderConfig, OidcAuthConfig, PreferredAuthMethod,
    XAI_OAUTH2_ISSUER, is_xai_oauth2_issuer, xai_oauth2_issuer,
};
pub(crate) use external_auth::{parse_output, refresh_with_command};
pub(crate) use flow::{
    AuthChannels, run_auth_flow, run_auth_flow_with_stderr_bridge,
    try_ensure_session_noninteractive,
};
pub use flow::{
    AuthUrlInfo, AuthUrlMode, DualLogoutResult, LoginTransportOverride, LogoutResult,
    bare_logout_usage, ensure_authenticated, ensure_authenticated_or_noninteractive,
    ensure_authenticated_with_override, format_dual_logout_message, logout_all_provider_slots,
    logout_provider_slot, perform_logout, run_cli_auth_status, run_cli_login,
    run_cli_login_for_provider, run_cli_logout, run_cli_logout_at_path, try_ensure_fresh_auth,
    write_cli_auth_status,
};
pub use jwt::{is_jwt_expired_or_near, parse_jwt_expiration};
mod meta;
pub use error::{AuthError, RefreshTokenError, RefreshTokenFailedReason};
pub use manager::{AuthManager, shared_api_key_provider};
pub use meta::{AuthMeta, GateInfo};
pub use model::{
    AuthMode, AuthProvider, AuthStore, GrokAuth, PROVIDER_CODEX, PROVIDER_XAI, lookup_auth,
    select_provider_access_token,
};
pub(crate) use model::{TOKEN_TTL, UserInfo, is_expired, token_suffix};
pub(crate) use refresh::DiagnosticUploader;
pub use status::{
    AuthStatusReport, ProviderAuthStatus, credential_usable, format_auth_status,
    inspect_provider_store, store_logged_in, store_usable,
};
pub use storage::{
    AuthStoreReadError, ProviderStoreMutation, clear_all_provider_slots, clear_api_key,
    clear_provider_slot, mutate_provider_store_or_prune, read_api_key, read_auth_json,
    read_provider_auth_store, read_token_by_scope, store_api_key,
};
pub(crate) use storage::{
    clear_provider_slot_with_lock, mutate_provider_store_or_prune_with_lock,
};
