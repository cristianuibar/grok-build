//! Phase 3 CLI model-list row formatter integration tests.
//!
//! Proves public `xai_grok_pager::models::format_cli_model_row` against UI-SPEC
//! (`  * {id} ({name})` / `  - {id} ({name})`). Do not use `cargo test -p xai-grok-pager --lib`.

use xai_grok_pager::models::format_cli_model_row;

#[test]
fn current_row_uses_star_id_and_name() {
    let row = format_cli_model_row(true, "grok-build", "Grok Build (xAI)");
    assert_eq!(row, "  * grok-build (Grok Build (xAI))");
    assert!(!row.contains("(default)"), "UI-SPEC uses star marker only, not (default) suffix");
}

#[test]
fn non_current_row_uses_dash_id_and_name() {
    let row = format_cli_model_row(false, "gpt-5.6-sol", "GPT-5.6 Sol (Codex)");
    assert_eq!(row, "  - gpt-5.6-sol (GPT-5.6 Sol (Codex))");
    assert!(
        row.contains("(Codex)"),
        "provider-bearing name must appear in CLI parenthetical"
    );
}

#[test]
fn empty_name_falls_back_to_id_inside_parens() {
    let current = format_cli_model_row(true, "orphan-id", "");
    assert_eq!(current, "  * orphan-id (orphan-id)");

    let other = format_cli_model_row(false, "orphan-id", "");
    assert_eq!(other, "  - orphan-id (orphan-id)");
}

#[test]
fn codex_suffix_on_non_default_gpt_row() {
    let row = format_cli_model_row(false, "gpt-5.6-terra", "GPT-5.6 Terra (Codex)");
    assert_eq!(row, "  - gpt-5.6-terra (GPT-5.6 Terra (Codex))");
    assert!(row.starts_with("  - "));
}
