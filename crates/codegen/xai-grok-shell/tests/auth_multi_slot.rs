use std::collections::BTreeMap;
use std::sync::Arc;

use chrono::{Duration, Utc};
use xai_grok_shell::auth::{AuthManager, AuthMode, GrokAuth, GrokComConfig};

#[tokio::test]
async fn public_auth_manager_loads_xai_from_nested_multi_slot_document() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("auth.json");
    let config = GrokComConfig::default();
    let scope = config.auth_scope();
    let xai_auth = GrokAuth {
        key: "integration-xai-token".to_owned(),
        auth_mode: AuthMode::Oidc,
        create_time: Utc::now(),
        expires_at: Some(Utc::now() + Duration::hours(1)),
        ..Default::default()
    };
    let codex_auth = GrokAuth {
        key: "reserved-codex-token".to_owned(),
        auth_mode: AuthMode::ApiKey,
        create_time: Utc::now(),
        ..Default::default()
    };
    let xai = BTreeMap::from([(scope, xai_auth)]);
    let codex = BTreeMap::from([("codex::fixture".to_owned(), codex_auth)]);
    let document = serde_json::json!({
        "version": 1,
        "providers": {
            "xai": xai,
            "codex": codex,
        }
    });
    std::fs::write(&path, serde_json::to_vec_pretty(&document).unwrap()).unwrap();

    let manager = Arc::new(AuthManager::new(dir.path(), config));
    let auth = manager.auth().await.unwrap();

    assert_eq!(auth.key, "integration-xai-token");
    let on_disk: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(
        on_disk
            .pointer("/providers/codex/codex::fixture/key")
            .and_then(|value| value.as_str()),
        Some("reserved-codex-token")
    );
}
