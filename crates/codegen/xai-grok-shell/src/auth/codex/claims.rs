//! Codex / ChatGPT JWT claim extraction (insecure decode — tokens arrive over TLS).
//!
//! Maps ChatGPT id_token / access_token claims into fields used by [`GrokAuth`]:
//! - `chatgpt_account_id` → `organization_id`
//! - `email` when present
//! - `exp` for `expires_at` when the token response omits `expires_in`

use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;

use crate::auth::parse_jwt_expiration;

/// Conservative TTL when neither `expires_in` nor a parseable JWT `exp` is available.
///
/// Short enough that refresh runs soon; never multi-day silent TTL that skips refresh.
pub const CONSERVATIVE_EXPIRES_FALLBACK: Duration = Duration::minutes(5);

/// Useful subset of ChatGPT JWT claims (id_token or access_token).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CodexTokenClaims {
    pub email: Option<String>,
    pub user_id: Option<String>,
    /// ChatGPT workspace / account id → persisted as `GrokAuth.organization_id`.
    pub chatgpt_account_id: Option<String>,
    pub exp: Option<DateTime<Utc>>,
}

#[derive(Deserialize)]
struct IdClaims {
    #[serde(default)]
    email: Option<String>,
    #[serde(default)]
    sub: Option<String>,
    #[serde(default)]
    exp: Option<i64>,
    #[serde(rename = "https://api.openai.com/profile", default)]
    profile: Option<ProfileClaims>,
    #[serde(rename = "https://api.openai.com/auth", default)]
    auth: Option<AuthClaims>,
}

#[derive(Deserialize)]
struct ProfileClaims {
    #[serde(default)]
    email: Option<String>,
}

#[derive(Deserialize)]
struct AuthClaims {
    #[serde(default)]
    chatgpt_user_id: Option<String>,
    #[serde(default)]
    user_id: Option<String>,
    #[serde(default)]
    chatgpt_account_id: Option<String>,
}

/// Insecure-decode ChatGPT JWT claims from `id_token` (preferred) or `access_token`.
pub fn decode_codex_claims(token: &str) -> CodexTokenClaims {
    let Ok(data) = jsonwebtoken::dangerous::insecure_decode::<IdClaims>(token) else {
        return CodexTokenClaims {
            exp: parse_jwt_expiration(token),
            ..Default::default()
        };
    };
    let c = data.claims;
    let email = c
        .email
        .or_else(|| c.profile.as_ref().and_then(|p| p.email.clone()));
    let user_id = c
        .auth
        .as_ref()
        .and_then(|a| a.chatgpt_user_id.clone().or_else(|| a.user_id.clone()))
        .or(c.sub);
    let chatgpt_account_id = c.auth.and_then(|a| a.chatgpt_account_id);
    let exp = c
        .exp
        .and_then(|ts| DateTime::from_timestamp(ts, 0))
        .or_else(|| parse_jwt_expiration(token));
    CodexTokenClaims {
        email,
        user_id,
        chatgpt_account_id,
        exp,
    }
}

/// Resolve `expires_at`: prefer `expires_in`, else JWT `exp`, else conservative short TTL.
pub fn resolve_expires_at(
    now: DateTime<Utc>,
    expires_in: Option<u64>,
    access_token: &str,
    id_token: Option<&str>,
) -> DateTime<Utc> {
    if let Some(secs) = expires_in {
        return now + Duration::seconds(secs as i64);
    }
    // Prefer id_token exp, then access_token exp.
    if let Some(id) = id_token
        && let Some(exp) = decode_codex_claims(id).exp
    {
        return exp;
    }
    if let Some(exp) = decode_codex_claims(access_token).exp {
        return exp;
    }
    // Conservative: require refresh soon rather than multi-day silent TTL.
    now + CONSERVATIVE_EXPIRES_FALLBACK
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine;

    fn jwt_with_payload(payload_json: &str) -> String {
        let enc = base64::engine::general_purpose::URL_SAFE_NO_PAD;
        let header = enc.encode(r#"{"alg":"RS256","typ":"JWT"}"#);
        let payload = enc.encode(payload_json);
        format!("{header}.{payload}.fake-signature")
    }

    #[test]
    fn maps_chatgpt_account_id_and_email() {
        let token = jwt_with_payload(
            r#"{
              "email":"user@example.com",
              "exp": 2000000000,
              "https://api.openai.com/auth": {
                "chatgpt_account_id": "acct-123",
                "chatgpt_user_id": "user-9"
              }
            }"#,
        );
        let claims = decode_codex_claims(&token);
        assert_eq!(claims.email.as_deref(), Some("user@example.com"));
        assert_eq!(claims.chatgpt_account_id.as_deref(), Some("acct-123"));
        assert_eq!(claims.user_id.as_deref(), Some("user-9"));
        assert!(claims.exp.is_some());
    }

    #[test]
    fn expires_at_from_jwt_when_expires_in_missing() {
        let now = DateTime::from_timestamp(1_700_000_000, 0).unwrap();
        let exp_ts = 1_700_003_600;
        let token = jwt_with_payload(&format!(r#"{{"exp":{exp_ts}}}"#));
        let at = resolve_expires_at(now, None, &token, None);
        assert_eq!(at.timestamp(), exp_ts);
    }

    #[test]
    fn conservative_fallback_when_unparseable() {
        let now = Utc::now();
        let at = resolve_expires_at(now, None, "not-a-jwt", None);
        let delta = at.signed_duration_since(now);
        assert!(delta <= CONSERVATIVE_EXPIRES_FALLBACK + Duration::seconds(1));
        assert!(delta >= CONSERVATIVE_EXPIRES_FALLBACK - Duration::seconds(1));
    }
}
