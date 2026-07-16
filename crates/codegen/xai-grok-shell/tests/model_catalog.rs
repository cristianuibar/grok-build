//! Phase 3 model catalog integration tests.
//!
//! Wave 0 harness for public catalog APIs only (`resolve_model_list`, `Config`,
//! `ModelProvider`, etc.). Do not use `cargo test -p xai-grok-shell --lib`.

use indexmap::IndexMap;
use xai_grok_shell::agent::config::{
    resolve_model_list, Config, ConfigModelOverride, ModelEntry, ModelEntryConfig, ModelInfo,
    ModelProvider,
};
use xai_grok_shell::agent::models::available_models;

const GPT_IDS: [&str; 3] = ["gpt-5.6-sol", "gpt-5.6-terra", "gpt-5.6-luna"];

/// Build a prefetched remote-style entry via public APIs only.
fn remote_entry(cfg: &Config, slug: &str) -> ModelEntry {
    let mut entry = ModelEntry::fallback(slug, &cfg.endpoints);
    entry.info.provider = ModelProvider::Xai;
    entry.info.supported_in_api = true;
    entry.info.user_selectable = true;
    entry.info.hidden = false;
    entry
}

/// Proves the integration binary links and discovers tests (`--list`).
#[test]
fn harness_smoke() {
    let list = resolve_model_list(&Config::default(), None);
    assert!(
        !list.is_empty(),
        "resolve_model_list(Config::default(), None) must return at least one model"
    );
}

/// D-01/D-02/D-05: embedded catalog includes GPT-5.6 Sol/Terra/Luna with codex provider + UI-SPEC names.
#[test]
fn catalog_includes_gpt56() {
    let list = resolve_model_list(&Config::default(), None);
    let expected = [
        ("gpt-5.6-sol", "GPT-5.6 Sol (Codex)"),
        ("gpt-5.6-terra", "GPT-5.6 Terra (Codex)"),
        ("gpt-5.6-luna", "GPT-5.6 Luna (Codex)"),
    ];
    for (id, name) in expected {
        let entry = list
            .get(id)
            .unwrap_or_else(|| panic!("catalog missing GPT-5.6 entry {id}; keys={:?}", list.keys().collect::<Vec<_>>()));
        assert_eq!(
            entry.info.provider.as_str(),
            "codex",
            "{id} provider must be wire id \"codex\""
        );
        assert_eq!(
            entry.info.name.as_deref(),
            Some(name),
            "{id} display name must match UI-SPEC"
        );
    }
}

/// D-11: no-prefetch order is Grok first, then Sol → Terra → Luna.
#[test]
fn mixed_catalog_order() {
    let list = resolve_model_list(&Config::default(), None);
    let keys: Vec<&String> = list.keys().collect();

    let grok_pos = keys
        .iter()
        .position(|k| k.as_str() == "grok-build")
        .expect("catalog must include grok-build");
    let sol_pos = keys
        .iter()
        .position(|k| k.as_str() == "gpt-5.6-sol")
        .expect("catalog must include gpt-5.6-sol");
    let terra_pos = keys
        .iter()
        .position(|k| k.as_str() == "gpt-5.6-terra")
        .expect("catalog must include gpt-5.6-terra");
    let luna_pos = keys
        .iter()
        .position(|k| k.as_str() == "gpt-5.6-luna")
        .expect("catalog must include gpt-5.6-luna");

    assert!(
        grok_pos < sol_pos,
        "grok-build (pos {grok_pos}) must come before gpt-5.6-sol (pos {sol_pos}); keys={keys:?}"
    );
    assert!(
        sol_pos < terra_pos,
        "gpt-5.6-sol (pos {sol_pos}) must come before gpt-5.6-terra (pos {terra_pos}); keys={keys:?}"
    );
    assert!(
        terra_pos < luna_pos,
        "gpt-5.6-terra (pos {terra_pos}) must come before gpt-5.6-luna (pos {luna_pos}); keys={keys:?}"
    );
}

/// D-13: default model id remains grok-build.
#[test]
fn default_remains_grok_build() {
    assert_eq!(
        xai_grok_models::default_model(),
        "grok-build",
        "default_model() must remain grok-build (D-13)"
    );
}

/// D-07/D-10: Grok entry is provider xai with (xAI) name suffix.
#[test]
fn grok_entry_provider_xai_and_name_suffix() {
    let list = resolve_model_list(&Config::default(), None);
    let grok = list
        .get("grok-build")
        .expect("catalog must include grok-build");
    assert_eq!(grok.info.provider.as_str(), "xai");
    assert_eq!(
        grok.info.name.as_deref(),
        Some("Grok Build (xAI)"),
        "Grok display name must match UI-SPEC"
    );
}

/// D-08: missing provider on ModelInfo / ModelEntryConfig deserializes to xai.
#[test]
fn provider_default_xai() {
    let info: ModelInfo = serde_json::from_value(serde_json::json!({
        "model": "legacy-model",
        "base_url": "https://example.com/v1",
        "context_window": 200000,
        "api_backend": "responses",
        "auth_scheme": "bearer",
        "extra_headers": {},
        "use_concise": false,
        "hidden": false,
        "supports_reasoning_effort": false,
        "supports_backend_search": false,
        "show_model_fingerprint": false,
    }))
    .expect("ModelInfo without provider must deserialize");
    assert_eq!(info.provider.as_str(), "xai");

    let entry: ModelEntryConfig = serde_json::from_value(serde_json::json!({
        "model": "legacy-model",
        "base_url": "https://example.com/v1",
        "context_window": 200000,
    }))
    .expect("ModelEntryConfig without provider must deserialize");
    assert_eq!(entry.provider.as_str(), "xai");

    assert_eq!(ModelProvider::default().as_str(), "xai");
    assert_eq!(ModelProvider::Xai.display_label(), "xAI");
    assert_eq!(ModelProvider::Codex.display_label(), "Codex");
    assert_eq!(ModelProvider::Codex.as_str(), "codex");
}

/// GPT agent_type stays stock (not codex harness) this phase (D-05 / discretion).
#[test]
fn gpt_agent_type_is_stock() {
    let list = resolve_model_list(&Config::default(), None);
    for id in ["gpt-5.6-sol", "gpt-5.6-terra", "gpt-5.6-luna"] {
        let entry = list.get(id).unwrap_or_else(|| panic!("missing {id}"));
        assert_ne!(
            entry.info.agent_type, "codex",
            "{id} agent_type must not be codex harness this phase; got {}",
            entry.info.agent_type
        );
        // Stock default is grok-build-plan (or equivalent default_agent_type()).
        assert!(
            !entry.info.agent_type.is_empty(),
            "{id} agent_type must be non-empty stock default"
        );
    }
}

/// GPT rows are selector-visible for API-key users (supported_in_api true).
#[test]
fn gpt_supported_in_api_true() {
    let list = resolve_model_list(&Config::default(), None);
    for id in ["gpt-5.6-sol", "gpt-5.6-terra", "gpt-5.6-luna"] {
        let entry = list.get(id).unwrap_or_else(|| panic!("missing {id}"));
        assert!(
            entry.info.supported_in_api,
            "{id} must have supported_in_api = true"
        );
    }
}

/// Valid ConfigModelOverride.provider Some(Codex) reaches ModelInfo via resolve_model_list.
#[test]
fn override_provider_codex_reaches_model_info() {
    let mut cfg = Config::default();
    cfg.config_models.insert(
        "override-target".to_owned(),
        ConfigModelOverride {
            provider: Some(ModelProvider::Codex),
            name: Some("Override Target (Codex)".to_owned()),
            ..Default::default()
        },
    );
    let list = resolve_model_list(&cfg, None);
    let entry = list
        .get("override-target")
        .expect("override must create/keep model key");
    assert_eq!(entry.info.provider.as_str(), "codex");
}

/// ConfigModelOverride without provider leaves fallback/default xai.
#[test]
fn override_provider_missing_defaults_xai() {
    let mut cfg = Config::default();
    cfg.config_models.insert(
        "no-provider-override".to_owned(),
        ConfigModelOverride {
            name: Some("No Provider".to_owned()),
            provider: None,
            ..Default::default()
        },
    );
    let list = resolve_model_list(&cfg, None);
    let entry = list
        .get("no-provider-override")
        .expect("override must create model key");
    assert_eq!(
        entry.info.provider.as_str(),
        "xai",
        "missing override provider must keep fallback xai"
    );

    // Explicit missing field on ConfigModelOverride TOML also defaults None → xai on apply.
    let raw: toml::Value = toml::from_str(
        r#"
        [model.foo]
        name = "Foo"
        "#,
    )
    .unwrap();
    let cfg = Config::new_from_toml_cfg(&raw).expect("config should parse");
    assert!(cfg.config_models.contains_key("foo"));
    assert_eq!(cfg.config_models["foo"].provider, None);
    let list = resolve_model_list(&cfg, None);
    assert_eq!(list["foo"].info.provider.as_str(), "xai");
}

/// Invalid provider value warns (InvalidValue) without dropping the model key.
#[test]
fn override_invalid_provider_warns_keeps_model() {
    let raw: toml::Value = toml::from_str(
        r#"
        [model.foo]
        name = "Foo"
        provider = "not-a-provider"
        "#,
    )
    .unwrap();
    let cfg = Config::new_from_toml_cfg(&raw).expect("config should parse");
    assert!(
        cfg.config_models.contains_key("foo"),
        "invalid provider must not drop the model key"
    );
    assert_eq!(
        cfg.config_models["foo"].provider, None,
        "invalid provider field is skipped"
    );
    assert!(
        !cfg.model_override_warnings.is_empty(),
        "expected at least one model override warning"
    );
    let provider_warn = cfg.model_override_warnings.iter().any(|w| {
        w.field.as_deref() == Some("provider")
            || w.reason.to_lowercase().contains("provider")
            || matches!(
                w.kind,
                xai_grok_shell::agent::config_model_override_parse::ModelOverrideWarningKind::InvalidValue
            )
    });
    assert!(
        provider_warn,
        "expected a warning about invalid provider; warnings={:?}",
        cfg.model_override_warnings
    );
}

// ── Plan 02: prefetch survival / collision / visibility ──────────────────

/// D-03/D-09: remote-only prefetch still yields bundled Codex GPT rows.
#[test]
fn codex_defaults_survive_prefetch() {
    let cfg = Config::default();
    let mut prefetched = IndexMap::new();
    prefetched.insert("secret-xyz".to_owned(), remote_entry(&cfg, "secret-xyz"));

    let list = resolve_model_list(&cfg, Some(prefetched));

    for id in GPT_IDS {
        let entry = list.get(id).unwrap_or_else(|| {
            panic!(
                "Codex default {id} must survive prefetch; keys={:?}",
                list.keys().collect::<Vec<_>>()
            )
        });
        assert_eq!(
            entry.info.provider.as_str(),
            "codex",
            "{id} provider must remain codex after prefetch survival"
        );
    }
    assert!(
        list.contains_key("secret-xyz"),
        "remote-only key must remain; keys={:?}",
        list.keys().collect::<Vec<_>>()
    );
    // Pure catalog path — no AuthManager / providers.codex involved.
}

/// Enterprise/xAI prune intent: remote-only prefetch must not resurrect grok-build.
#[test]
fn prefetch_still_prunes_bundled_xai() {
    let cfg = Config::default();
    let mut prefetched = IndexMap::new();
    prefetched.insert("remote-only".to_owned(), remote_entry(&cfg, "remote-only"));

    let list = resolve_model_list(&cfg, Some(prefetched));
    assert!(
        !list.contains_key("grok-build"),
        "grok-build must stay pruned when absent from prefetched; keys={:?}",
        list.keys().collect::<Vec<_>>()
    );
}

/// Custom models endpoint must not inject bundled GPT defaults.
#[test]
fn custom_endpoint_skips_codex_inject() {
    let mut cfg = Config::default();
    cfg.endpoints.models_base_url = Some("https://enterprise.example/v1".into());

    let empty = resolve_model_list(&cfg, Some(IndexMap::new()));
    for id in GPT_IDS {
        assert!(
            !empty.contains_key(id),
            "custom endpoint empty prefetch must not inject {id}; keys={:?}",
            empty.keys().collect::<Vec<_>>()
        );
    }

    let mut prefetched = IndexMap::new();
    prefetched.insert(
        "enterprise-model".to_owned(),
        remote_entry(&cfg, "enterprise-model"),
    );
    let list = resolve_model_list(&cfg, Some(prefetched));
    for id in GPT_IDS {
        assert!(
            !list.contains_key(id),
            "custom endpoint remote prefetch must not inject {id}; keys={:?}",
            list.keys().collect::<Vec<_>>()
        );
    }
    assert!(list.contains_key("enterprise-model"));
}

/// Q1 RESOLVED: empty Some({}) still receives Codex defaults when !has_custom_endpoint.
#[test]
fn empty_prefetch_still_gets_codex_defaults() {
    let cfg = Config::default();
    assert!(
        !cfg.endpoints.has_custom_endpoint(),
        "default Config must not have a custom models endpoint"
    );
    let list = resolve_model_list(&cfg, Some(IndexMap::new()));
    for id in GPT_IDS {
        assert!(
            list.contains_key(id),
            "empty prefetch must still inject {id}; keys={:?}",
            list.keys().collect::<Vec<_>>()
        );
        assert_eq!(list[id].info.provider.as_str(), "codex");
    }
    // xAI bundled defaults stay pruned on empty Some (only Codex re-append).
    assert!(
        !list.contains_key("grok-build"),
        "empty prefetch must not resurrect grok-build"
    );
}

/// Bundled Codex entry is authoritative: remote cannot rebind gpt-5.6-sol to xai.
#[test]
fn prefetched_collision_cannot_rebind_codex_default_to_xai() {
    let cfg = Config::default();
    let mut hijacked = remote_entry(&cfg, "gpt-5.6-sol");
    hijacked.info.provider = ModelProvider::Xai;
    hijacked.info.name = Some("Hijacked Sol".to_owned());

    let mut prefetched = IndexMap::new();
    prefetched.insert("gpt-5.6-sol".to_owned(), hijacked);
    prefetched.insert("remote-grok".to_owned(), remote_entry(&cfg, "remote-grok"));

    let list = resolve_model_list(&cfg, Some(prefetched));
    let sol = list
        .get("gpt-5.6-sol")
        .expect("gpt-5.6-sol must exist after collision merge");
    assert_eq!(
        sol.info.provider.as_str(),
        "codex",
        "bundled provider must win over remote xai rebind"
    );
    assert_eq!(
        sol.info.name.as_deref(),
        Some("GPT-5.6 Sol (Codex)"),
        "bundled UI-SPEC name must win over hijacked remote name"
    );
}

/// D-11 / cycle 2: remove-then-append — Sol→Terra→Luna after remote, not mid-list Terra.
#[test]
fn prefetched_codex_collision_order_appends_sol_terra_luna_after_remote() {
    let cfg = Config::default();
    let mut terra = remote_entry(&cfg, "gpt-5.6-terra");
    terra.info.provider = ModelProvider::Xai;
    terra.info.name = Some("Hijacked Terra".to_owned());

    // Prefetch lists terra first (bad index if replace-in-place), then a remote-only key.
    let mut prefetched = IndexMap::new();
    prefetched.insert("gpt-5.6-terra".to_owned(), terra);
    prefetched.insert("remote-grok".to_owned(), remote_entry(&cfg, "remote-grok"));

    let list = resolve_model_list(&cfg, Some(prefetched));
    let keys: Vec<&str> = list.keys().map(|k| k.as_str()).collect();

    let remote_pos = keys
        .iter()
        .position(|k| *k == "remote-grok")
        .expect("remote-grok must remain");
    let sol_pos = keys
        .iter()
        .position(|k| *k == "gpt-5.6-sol")
        .expect("gpt-5.6-sol must be appended");
    let terra_pos = keys
        .iter()
        .position(|k| *k == "gpt-5.6-terra")
        .expect("gpt-5.6-terra must be re-appended");
    let luna_pos = keys
        .iter()
        .position(|k| *k == "gpt-5.6-luna")
        .expect("gpt-5.6-luna must be appended");

    assert!(
        remote_pos < sol_pos && remote_pos < terra_pos && remote_pos < luna_pos,
        "remote-grok (pos {remote_pos}) must appear before all GPT keys; keys={keys:?}"
    );
    assert!(
        sol_pos < terra_pos && terra_pos < luna_pos,
        "GPT family order must be sol→terra→luna (JSON order), not prefetched terra-first; keys={keys:?}"
    );
    assert_eq!(
        list["gpt-5.6-terra"].info.provider.as_str(),
        "codex",
        "re-appended terra must use bundled provider"
    );
}

/// D-12: GPT visible for session auth and API-key auth; no Codex credential gate.
#[test]
fn gpt_visible_session_and_api_key() {
    let cfg = Config::default();
    let catalog = resolve_model_list(&cfg, None);

    let session = available_models(&catalog, true);
    let api_key = available_models(&catalog, false);

    for id in GPT_IDS {
        let in_session = session.keys().any(|k| k.0.as_ref() == id);
        let in_api = api_key.keys().any(|k| k.0.as_ref() == id);
        assert!(
            in_session,
            "session auth must list {id}; keys={:?}",
            session.keys().map(|k| k.0.as_ref()).collect::<Vec<_>>()
        );
        assert!(
            in_api,
            "API-key auth must list {id}; keys={:?}",
            api_key.keys().map(|k| k.0.as_ref()).collect::<Vec<_>>()
        );
    }
    // Pure catalog functions only — no AuthManager / providers.codex involvement.
}

/// Mixed flat list: prefetched remote + Codex append; GPT after prefetched when missing.
#[test]
fn mixed_list_with_prefetch() {
    let cfg = Config::default();
    let mut prefetched = IndexMap::new();
    let mut remote = remote_entry(&cfg, "grok-build");
    remote.info.name = Some("Remote Grok Build".to_owned());
    prefetched.insert("grok-build".to_owned(), remote);

    let list = resolve_model_list(&cfg, Some(prefetched));
    assert!(list.contains_key("grok-build"));
    for id in GPT_IDS {
        assert!(
            list.contains_key(id),
            "mixed list must include {id}; keys={:?}",
            list.keys().collect::<Vec<_>>()
        );
    }

    let keys: Vec<&str> = list.keys().map(|k| k.as_str()).collect();
    let grok_pos = keys.iter().position(|k| *k == "grok-build").unwrap();
    let sol_pos = keys.iter().position(|k| *k == "gpt-5.6-sol").unwrap();
    assert!(
        grok_pos < sol_pos,
        "prefetched grok-build must precede appended GPT family; keys={keys:?}"
    );
}
