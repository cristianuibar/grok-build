//! Phase 3 settings DynamicEnum name-inherit integration tests.
//!
//! Proves public `dynamic_enum_choices(ActiveModelCatalog, …)` carries
//! provider-suffixed display names from `PagerLocalSnapshot.available_models`.
//! Do not use `cargo test -p xai-grok-pager --lib`.

use std::sync::Arc;

use agent_client_protocol as acp;
use xai_grok_pager::settings::{
    dynamic_enum_choices, DynamicEnumSource, PagerLocalSnapshot,
};

fn model_id(id: &str) -> acp::ModelId {
    acp::ModelId::new(Arc::from(id))
}

#[test]
fn dynamic_enum_choices_include_provider_suffixes() {
    let snapshot = PagerLocalSnapshot {
        available_models: vec![
            ("Grok Build (xAI)".to_owned(), model_id("grok-build")),
            ("GPT-5.6 Sol (Codex)".to_owned(), model_id("gpt-5.6-sol")),
            ("GPT-5.6 Terra (Codex)".to_owned(), model_id("gpt-5.6-terra")),
        ],
        ..PagerLocalSnapshot::default()
    };

    let choices = dynamic_enum_choices(DynamicEnumSource::ActiveModelCatalog, &snapshot);

    assert!(
        !choices.is_empty(),
        "ActiveModelCatalog must return at least the sentinel choice"
    );
    assert_eq!(
        choices[0].display, "(no override)",
        "index 0 must remain the (no override) sentinel"
    );
    assert_eq!(choices[0].canonical, "");

    let displays: Vec<&str> = choices.iter().map(|c| c.display.as_str()).collect();
    assert!(
        displays.iter().any(|d| d.contains("(xAI)")),
        "settings choices must include a display with (xAI); got {displays:?}"
    );
    assert!(
        displays.iter().any(|d| d.contains("(Codex)")),
        "settings choices must include a display with (Codex); got {displays:?}"
    );

    // Model rows (non-sentinel) may have empty descriptions (names-only inherit).
    for choice in choices.iter().skip(1) {
        assert!(
            choice.description.is_empty(),
            "model-row description empty OK; got {:?} for {}",
            choice.description,
            choice.display
        );
        assert!(
            !choice.canonical.is_empty(),
            "model-row canonical should be the display name from snapshot"
        );
    }
}

#[test]
fn dynamic_enum_choices_empty_catalog_is_sentinel_only() {
    let snapshot = PagerLocalSnapshot::default();
    let choices = dynamic_enum_choices(DynamicEnumSource::ActiveModelCatalog, &snapshot);
    assert_eq!(choices.len(), 1);
    assert_eq!(choices[0].display, "(no override)");
}
