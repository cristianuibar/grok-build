//! Phase 3 model catalog integration tests.
//!
//! Wave 0 harness for public catalog APIs only (`resolve_model_list`, `Config`,
//! `ModelProvider`, etc.). Do not use `cargo test -p xai-grok-shell --lib`.

use xai_grok_shell::agent::config::{resolve_model_list, Config};

/// Proves the integration binary links and discovers tests (`--list`).
#[test]
fn harness_smoke() {
    let list = resolve_model_list(&Config::default(), None);
    assert!(
        !list.is_empty(),
        "resolve_model_list(Config::default(), None) must return at least one model"
    );
}

/// D-01/D-02: embedded catalog must include GPT-5.6 Sol/Terra/Luna wire ids.
#[test]
fn catalog_includes_gpt56() {
    let list = resolve_model_list(&Config::default(), None);
    for id in ["gpt-5.6-sol", "gpt-5.6-terra", "gpt-5.6-luna"] {
        assert!(
            list.contains_key(id),
            "catalog missing GPT-5.6 entry {id}; keys={:?}",
            list.keys().collect::<Vec<_>>()
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
