//! `/model` (alias `/m`) — switch model + (optionally) reasoning effort.
//! Chained autocomplete: pick a reasoning-supported model → trailing space
//! re-opens the dropdown into a `low|medium|high|xhigh` sub-menu.

use agent_client_protocol as acp;
use xai_grok_shell::sampling::types::supports_reasoning_effort_meta;

use crate::acp::model_state::ModelState;
use crate::app::actions::Action;
use crate::slash::command::{AppCtx, ArgItem, CommandExecCtx, CommandResult, SlashCommand};
use crate::slash::commands::effort_levels::build_effort_arg_items;

/// Switch the active model (and optionally its reasoning effort).
pub struct ModelCommand;

impl SlashCommand for ModelCommand {
    fn name(&self) -> &str {
        "model"
    }

    fn aliases(&self) -> &[&str] {
        &["m"]
    }

    fn description(&self) -> &str {
        "Switch the active model"
    }

    fn session_scoped(&self) -> bool {
        true
    }

    fn offered_when_session_less(&self) -> bool {
        // The dashboard offers `/model` to pick the model for the next
        // spawned agent (intercepted in `dispatch_dashboard_dispatch_slash`).
        true
    }

    fn usage(&self) -> &str {
        "/model <name> [effort]"
    }

    fn takes_args(&self) -> bool {
        true
    }

    fn args_required(&self) -> bool {
        true
    }

    fn arg_placeholder(&self) -> Option<&str> {
        Some("<model> [effort]")
    }

    fn suggest_args(&self, ctx: &AppCtx, args_query: &str) -> Option<Vec<ArgItem>> {
        if ctx.models.is_empty() {
            return None;
        }

        // Effort phase if input is "<reasoning-model> ", else model phase.
        if let Some(model_id) = detect_effort_phase(ctx.models, args_query) {
            return Some(build_effort_items(ctx.models, &model_id));
        }
        Some(build_model_items(ctx.models, ctx.provider_auth))
    }

    fn run(&self, ctx: &mut CommandExecCtx, args: &str) -> CommandResult {
        let trimmed = args.trim();
        if trimmed.is_empty() {
            return CommandResult::Error("Usage: /model <name> [effort]".into());
        }

        // Prefer an exact full-string catalog match first. Model display names
        // often contain spaces ("Grok 4.5"); if we split on the last token
        // first, a shorter catalog entry ("Grok") would steal the prefix and
        // treat "4.5" as an effort level.
        if let Some(id) = ctx.models.resolve_by_name_or_id(trimmed) {
            return CommandResult::Action(Action::SetDefaultModel(id));
        }

        // Trailing effort token + reasoning model → session-scoped switch
        // (not persisted as default). Resolve via the shared gate so a rejected
        // level (e.g. `none` on grok-4.5) surfaces the effort error with the
        // model's offered ids — not "Unknown model: … none".
        if let Some((prefix, token)) = split_trailing_token(trimmed)
            && let Some(id) = resolve_model(ctx.models, prefix)
            && ctx
                .models
                .available
                .get(&id)
                .map(supports_reasoning_effort)
                .unwrap_or(false)
        {
            return match ctx.models.resolve_effort_for_model(&id, token) {
                Ok(effort) => CommandResult::Action(Action::SwitchModel {
                    model_id: id,
                    effort: Some(effort),
                }),
                Err(err) => CommandResult::Error(err.message()),
            };
        }

        CommandResult::Error(format!("Unknown model: {trimmed}"))
    }
}

/// Look up a model by case-insensitive display name OR model id match.
fn resolve_model(models: &ModelState, name: &str) -> Option<acp::ModelId> {
    models.resolve_by_name_or_id(name)
}

fn supports_reasoning_effort(info: &acp::ModelInfo) -> bool {
    supports_reasoning_effort_meta(info.meta.as_ref())
}

/// Split `args` into `(prefix, last_token)` on the final whitespace run.
/// Returns `None` when there is no interior whitespace to split on. The token is
/// resolved to an effort against the picked model's options by the caller.
fn split_trailing_token(args: &str) -> Option<(&str, &str)> {
    let (prefix, last) = args.rsplit_once(char::is_whitespace)?;
    let prefix = prefix.trim_end();
    if prefix.is_empty() || last.is_empty() {
        return None;
    }
    Some((prefix, last))
}

/// Returns the matched model id when `args_query` is `"<reasoning-model> ..."`.
/// Longest-name-first to disambiguate names that share a prefix.
fn detect_effort_phase(models: &ModelState, args_query: &str) -> Option<acp::ModelId> {
    let mut candidates: Vec<(&acp::ModelId, &str)> = models
        .available
        .iter()
        .filter(|(_, info)| supports_reasoning_effort(info))
        .map(|(id, info)| (id, info.name.as_str()))
        .collect();
    candidates.sort_by_key(|(_, name)| std::cmp::Reverse(name.len()));

    for (id, name) in candidates {
        if args_query.len() > name.len()
            && args_query.is_char_boundary(name.len())
            && args_query[..name.len()].eq_ignore_ascii_case(name)
            && args_query[name.len()..].starts_with(char::is_whitespace)
        {
            return Some(id.clone());
        }
    }
    None
}

/// Exact badge copy (UI-SPEC D-08). Secondary/dim rendering is a view concern;
/// display text carries the literal ` · needs login` suffix.
pub const NEEDS_LOGIN_BADGE: &str = "needs login";

/// Whether a model row should show the needs-login badge.
///
/// Badge only when provider is `xai`|`codex`, slot is unusable, and the model
/// does **not** have own credentials (BYOK suppress). Unknown provider → no badge.
pub fn should_show_needs_login_badge(
    provider: Option<&str>,
    has_own_credentials: bool,
    provider_auth: crate::app::app_view::ProviderAuthUsableSnapshot,
) -> bool {
    if has_own_credentials {
        return false;
    }
    let Some(provider) = provider else {
        return false;
    };
    match provider_auth.usable_for_wire(provider) {
        Some(false) => true,
        Some(true) | None => false,
    }
}

/// Format model display with optional `(current)` and ` · needs login` suffixes.
///
/// Shared by slash `/model` and settings DynamicEnum (D-04 / D-08).
pub fn format_model_display_with_auth_badge(
    name: &str,
    is_current: bool,
    provider: Option<&str>,
    has_own_credentials: bool,
    provider_auth: crate::app::app_view::ProviderAuthUsableSnapshot,
) -> String {
    let mut display = if is_current {
        format!("{name} (current)")
    } else {
        name.to_owned()
    };
    if should_show_needs_login_badge(provider, has_own_credentials, provider_auth) {
        display.push_str(" · ");
        display.push_str(NEEDS_LOGIN_BADGE);
    }
    display
}

/// Read trusted catalog `provider` wire id from ACP model meta (`xai`|`codex`).
pub fn model_meta_provider(meta: Option<&serde_json::Map<String, serde_json::Value>>) -> Option<&str> {
    let p = meta?.get("provider")?.as_str()?;
    match p {
        "xai" | "codex" => Some(p),
        _ => None,
    }
}

/// Read trusted catalog `hasOwnCredentials` from ACP model meta.
pub fn model_meta_has_own_credentials(
    meta: Option<&serde_json::Map<String, serde_json::Value>>,
) -> bool {
    meta.and_then(|m| m.get("hasOwnCredentials"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

/// One row per logical model. Reasoning models get a trailing space in
/// `insert_text` so the prompt widget chains into the effort sub-menu.
///
/// Does **not** filter by auth (D-04 full mixed catalog). Badge is display-only.
pub fn build_model_items(
    models: &ModelState,
    provider_auth: crate::app::app_view::ProviderAuthUsableSnapshot,
) -> Vec<ArgItem> {
    let current_id = models.current.as_ref();
    let mut items: Vec<ArgItem> = Vec::with_capacity(models.available.len());
    for (id, info) in &models.available {
        let is_current = current_id == Some(id);
        let supports = supports_reasoning_effort(info);
        let meta = info.meta.as_ref();
        let provider = model_meta_provider(meta);
        let has_own = model_meta_has_own_credentials(meta);

        let display = format_model_display_with_auth_badge(
            &info.name,
            is_current,
            provider,
            has_own,
            provider_auth,
        );

        // Trailing space on reasoning models: signals "more input
        // expected" to the prompt widget so Enter advances to effort
        // phase instead of submitting.
        let insert_text = if supports {
            format!("{} ", info.name)
        } else {
            info.name.clone()
        };

        items.push(ArgItem {
            display,
            match_text: info.name.clone(),
            insert_text,
            description: info.description.clone().unwrap_or_default(),
        });
    }
    items
}

/// One row per effort level for the `/model` chained effort phase.
/// `insert_text` is `"ModelName high"` so selecting a row completes both tokens.
fn build_effort_items(models: &ModelState, model_id: &acp::ModelId) -> Vec<ArgItem> {
    let info = match models.available.get(model_id) {
        Some(info) => info,
        None => return Vec::new(),
    };
    let model_name = info.name.clone();
    let is_current_model = models.current.as_ref() == Some(model_id);
    let options = models.reasoning_effort_options_for(model_id);
    build_effort_arg_items(
        &options,
        models.reasoning_effort,
        is_current_model,
        |option| format!("{model_name} {}", option.id),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use xai_grok_shell::sampling::types::ReasoningEffort;

    fn model_with_reasoning(id: &str, name: &str) -> (acp::ModelId, acp::ModelInfo) {
        let id = acp::ModelId::new(Arc::from(id));
        let mut meta = serde_json::Map::new();
        meta.insert(
            "supportsReasoningEffort".into(),
            serde_json::Value::Bool(true),
        );
        let info = acp::ModelInfo::new(id.clone(), name.to_string())
            .meta(serde_json::Value::Object(meta).as_object().cloned());
        (id, info)
    }

    fn plain_model(id: &str, name: &str) -> (acp::ModelId, acp::ModelInfo) {
        let id = acp::ModelId::new(Arc::from(id));
        let info = acp::ModelInfo::new(id.clone(), name.to_string());
        (id, info)
    }

    static EMPTY_BUNDLE: crate::app::bundle::BundleState = crate::app::bundle::BundleState {
        has_cache: false,
        version: String::new(),
        personas: Vec::new(),
        roles: Vec::new(),
        agents: Vec::new(),
        skills: Vec::new(),
        persona_details: Vec::new(),
        role_details: Vec::new(),
    };

    fn dummy_exec_ctx(models: &ModelState) -> CommandExecCtx<'_> {
        CommandExecCtx {
            models,
            session_id: None,
            bundle_state: &EMPTY_BUNDLE,
            screen_mode: crate::app::ScreenMode::Inline,
            pager_state: crate::settings::PagerLocalSnapshot {
                multiline_mode: false,
                yolo_mode: false,
                ..crate::settings::PagerLocalSnapshot::default()
            },
        }
    }

    #[test]
    fn split_trailing_token_splits_on_final_whitespace() {
        assert_eq!(
            split_trailing_token("Reasoning X high"),
            Some(("Reasoning X", "high"))
        );
        assert_eq!(
            split_trailing_token("reasoning-x  xhigh"),
            Some(("reasoning-x", "xhigh"))
        );
        // No interior whitespace → nothing to split off.
        assert!(split_trailing_token("reasoning-x-pro").is_none());
    }

    fn model_with_provider(
        id: &str,
        name: &str,
        provider: &str,
        has_own: bool,
    ) -> (acp::ModelId, acp::ModelInfo) {
        let id = acp::ModelId::new(Arc::from(id));
        let mut meta = serde_json::Map::new();
        meta.insert("provider".into(), serde_json::Value::String(provider.into()));
        if has_own {
            meta.insert("hasOwnCredentials".into(), serde_json::Value::Bool(true));
        }
        let info = acp::ModelInfo::new(id.clone(), name.to_string())
            .meta(Some(meta));
        (id, info)
    }

    #[test]
    fn empty_query_returns_one_row_per_logical_model() {
        let mut state = ModelState::default();
        let (rid, rinfo) = model_with_reasoning("reasoning-x", "Reasoning X");
        let (pid, pinfo) = plain_model("grok-4.5", "Grok 4.5");
        state.available.insert(rid, rinfo);
        state.available.insert(pid, pinfo);

        let cmd = ModelCommand;
        let ctx = AppCtx {
            models: &state,
            cwd: std::path::Path::new("."),
            has_session_announcements: false,
            screen_mode: crate::app::ScreenMode::Fullscreen,
            provider_auth: crate::app::app_view::ProviderAuthUsableSnapshot::UNKNOWN,
        };
        let items = cmd.suggest_args(&ctx, "").unwrap();
        assert_eq!(items.len(), 2, "model phase: one row per logical model");

        // Reasoning model has trailing space in insert_text -- this is the
        // signal the prompt widget reads to keep the dropdown open after
        // Enter so the effort sub-menu can render.
        let reasoning = items
            .iter()
            .find(|i| i.match_text == "Reasoning X")
            .unwrap();
        assert_eq!(reasoning.insert_text, "Reasoning X ");

        // Plain model has no trailing space -- Enter commits immediately.
        let plain = items.iter().find(|i| i.match_text == "Grok 4.5").unwrap();
        assert_eq!(plain.insert_text, "Grok 4.5");
    }

    #[test]
    fn trailing_space_after_reasoning_model_enters_effort_phase() {
        let mut state = ModelState::default();
        let (id, info) = model_with_reasoning("reasoning-x", "Reasoning X");
        state.available.insert(id, info);

        let cmd = ModelCommand;
        let ctx = AppCtx {
            models: &state,
            cwd: std::path::Path::new("."),
            has_session_announcements: false,
            screen_mode: crate::app::ScreenMode::Fullscreen,
            provider_auth: crate::app::app_view::ProviderAuthUsableSnapshot::UNKNOWN,
        };
        // Args query has a trailing space -> effort phase. Items come out
        // ordered xhigh -> low (strongest first) per EFFORT_LEVELS.
        let items = cmd.suggest_args(&ctx, "Reasoning X ").unwrap();
        assert_eq!(items.len(), 4);
        assert_eq!(items[0].insert_text, "Reasoning X xhigh");
        assert_eq!(items[1].insert_text, "Reasoning X high");
        assert_eq!(items[2].insert_text, "Reasoning X medium");
        assert_eq!(items[3].insert_text, "Reasoning X low");
        // Display is just the level so the user sees a clean column.
        assert_eq!(items[0].display, "xhigh");
        // match_text carries the sort-key prefix that forces the matcher's
        // alphabetical tiebreak to render rows in EFFORT_LEVELS order.
        assert!(items[0].match_text.starts_with("a "));
        assert!(items[3].match_text.starts_with("d "));
    }

    #[test]
    fn partial_effort_query_still_in_effort_phase() {
        let mut state = ModelState::default();
        let (id, info) = model_with_reasoning("reasoning-x", "Reasoning X");
        state.available.insert(id, info);

        let cmd = ModelCommand;
        let ctx = AppCtx {
            models: &state,
            cwd: std::path::Path::new("."),
            has_session_announcements: false,
            screen_mode: crate::app::ScreenMode::Fullscreen,
            provider_auth: crate::app::app_view::ProviderAuthUsableSnapshot::UNKNOWN,
        };
        // Still in effort phase; matcher upstream narrows to high / xhigh.
        let items = cmd.suggest_args(&ctx, "Reasoning X h").unwrap();
        assert_eq!(items.len(), 4);
    }

    #[test]
    fn partial_model_query_stays_in_model_phase() {
        let mut state = ModelState::default();
        let (id, info) = model_with_reasoning("reasoning-x", "Reasoning X");
        state.available.insert(id, info);

        let cmd = ModelCommand;
        let ctx = AppCtx {
            models: &state,
            cwd: std::path::Path::new("."),
            has_session_announcements: false,
            screen_mode: crate::app::ScreenMode::Fullscreen,
            provider_auth: crate::app::app_view::ProviderAuthUsableSnapshot::UNKNOWN,
        };
        // No trailing space, user is still typing the model name.
        let items = cmd.suggest_args(&ctx, "Reason").unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].insert_text, "Reasoning X ");
    }

    #[test]
    fn run_parses_model_plus_effort_when_supported() {
        let mut state = ModelState::default();
        let (id, info) = model_with_reasoning("reasoning-x", "Reasoning X");
        state.available.insert(id, info);
        let mut ctx = dummy_exec_ctx(&state);
        let result = ModelCommand.run(&mut ctx, "Reasoning X xhigh");
        match result {
            CommandResult::Action(Action::SwitchModel { model_id, effort }) => {
                assert_eq!(model_id.0.as_ref(), "reasoning-x");
                assert_eq!(effort, Some(ReasoningEffort::Xhigh));
            }
            other => panic!("expected SwitchModel with effort, got {other:?}"),
        }
    }

    #[test]
    fn p6_needs_login_badges_unusable_provider() {
        let mut state = ModelState::default();
        let (gid, ginfo) = model_with_provider("grok-4", "Grok 4", "xai", false);
        let (pid, pinfo) = model_with_provider("gpt-5.6", "GPT-5.6", "codex", false);
        state.available.insert(gid, ginfo);
        state.available.insert(pid, pinfo);
        let auth = crate::app::app_view::ProviderAuthUsableSnapshot {
            xai: true,
            codex: false,
        };
        let items = build_model_items(&state, auth);
        let gpt = items.iter().find(|i| i.match_text == "GPT-5.6").unwrap();
        assert!(
            gpt.display.contains(NEEDS_LOGIN_BADGE),
            "unusable codex row must badge: {}",
            gpt.display
        );
        let grok = items.iter().find(|i| i.match_text == "Grok 4").unwrap();
        assert!(
            !grok.display.contains(NEEDS_LOGIN_BADGE),
            "usable xai row must not badge: {}",
            grok.display
        );
    }

    #[test]
    fn p6_needs_login_never_filters_by_auth() {
        let mut state = ModelState::default();
        let (gid, ginfo) = model_with_provider("grok-4", "Grok 4", "xai", false);
        let (pid, pinfo) = model_with_provider("gpt-5.6", "GPT-5.6", "codex", false);
        state.available.insert(gid, ginfo);
        state.available.insert(pid, pinfo);
        let auth = crate::app::app_view::ProviderAuthUsableSnapshot {
            xai: true,
            codex: false,
        };
        let items = build_model_items(&state, auth);
        assert_eq!(items.len(), state.available.len());
    }

    #[test]
    fn p6_needs_login_current_and_badge_suffix() {
        let mut state = ModelState::default();
        let (pid, pinfo) = model_with_provider("gpt-5.6", "GPT-5.6", "codex", false);
        state.available.insert(pid.clone(), pinfo);
        state.set_current(pid, None);
        let auth = crate::app::app_view::ProviderAuthUsableSnapshot {
            xai: true,
            codex: false,
        };
        let items = build_model_items(&state, auth);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].display, "GPT-5.6 (current) · needs login");
    }

    #[test]
    fn p6_needs_login_usable_provider_no_badge() {
        let mut state = ModelState::default();
        let (pid, pinfo) = model_with_provider("gpt-5.6", "GPT-5.6", "codex", false);
        state.available.insert(pid, pinfo);
        let auth = crate::app::app_view::ProviderAuthUsableSnapshot {
            xai: true,
            codex: true,
        };
        let items = build_model_items(&state, auth);
        assert!(!items[0].display.contains(NEEDS_LOGIN_BADGE));
    }

    #[test]
    fn p6_needs_login_byok_suppresses_badge() {
        let mut state = ModelState::default();
        let (pid, pinfo) = model_with_provider("local-gpt", "Local GPT", "codex", true);
        state.available.insert(pid, pinfo);
        let auth = crate::app::app_view::ProviderAuthUsableSnapshot {
            xai: false,
            codex: false,
        };
        let items = build_model_items(&state, auth);
        assert!(
            !items[0].display.contains(NEEDS_LOGIN_BADGE),
            "BYOK must suppress badge: {}",
            items[0].display
        );
    }

    #[test]
    fn p6_needs_login_settings_dynamic_enum() {
        use crate::settings::{DynamicEnumSource, ModelAuthHint, PagerLocalSnapshot, dynamic_enum_choices};
        let snapshot = PagerLocalSnapshot {
            available_models: vec![
                (
                    "Grok 4".into(),
                    acp::ModelId::new(Arc::from("grok-4")),
                ),
                (
                    "GPT-5.6".into(),
                    acp::ModelId::new(Arc::from("gpt-5.6")),
                ),
            ],
            provider_auth: crate::app::app_view::ProviderAuthUsableSnapshot {
                xai: true,
                codex: false,
            },
            model_auth_hints: vec![
                ModelAuthHint {
                    provider: Some("xai".into()),
                    has_own_credentials: false,
                },
                ModelAuthHint {
                    provider: Some("codex".into()),
                    has_own_credentials: false,
                },
            ],
            ..PagerLocalSnapshot::default()
        };
        let choices = dynamic_enum_choices(DynamicEnumSource::ActiveModelCatalog, &snapshot);
        // +1 for "(no override)"
        assert_eq!(choices.len(), 3);
        let gpt = choices.iter().find(|c| c.canonical == "GPT-5.6").unwrap();
        assert!(
            gpt.display.contains(NEEDS_LOGIN_BADGE),
            "settings DynamicEnum must badge unusable non-BYOK: {}",
            gpt.display
        );
        let grok = choices.iter().find(|c| c.canonical == "Grok 4").unwrap();
        assert!(!grok.display.contains(NEEDS_LOGIN_BADGE));
    }

    #[test]
    fn run_rejects_unoffered_effort_with_effort_error_not_unknown_model() {
        // Regression: previously `resolve_effort_token_for` returned None and
        // the handler fell through to `Unknown model: Reasoning X none`.
        let mut state = ModelState::default();
        let (id, info) = model_with_reasoning("reasoning-x", "Reasoning X");
        state.available.insert(id, info);
        let mut ctx = dummy_exec_ctx(&state);
        let result = ModelCommand.run(&mut ctx, "Reasoning X none");
        match result {
            CommandResult::Error(msg) => {
                assert!(
                    msg.contains("unknown effort level 'none'"),
                    "expected effort error, got {msg}"
                );
                assert!(
                    msg.contains("use one of:"),
                    "expected offered levels in message, got {msg}"
                );
                assert!(
                    !msg.to_lowercase().contains("unknown model"),
                    "must not misreport as unknown model: {msg}"
                );
                let offered = msg.split_once("; ").map(|(_, r)| r).unwrap_or("");
                assert!(
                    !offered.contains("none"),
                    "must not list none as offered: {msg}"
                );
            }
            other => panic!("expected Error, got {other:?}"),
        }
    }

    #[test]
    fn run_prefers_full_multi_word_model_name_over_prefix_plus_effort() {
        // Catalog has both "Grok" (reasoning) and "Grok 4.5". `/model Grok 4.5`
        // must select the full name, not treat "4.5" as an effort on "Grok".
        let mut state = ModelState::default();
        let (short_id, short_info) = model_with_reasoning("grok", "Grok");
        let (long_id, long_info) = model_with_reasoning("grok-4.5", "Grok 4.5");
        state.available.insert(short_id, short_info);
        state.available.insert(long_id.clone(), long_info);
        let mut ctx = dummy_exec_ctx(&state);
        let result = ModelCommand.run(&mut ctx, "Grok 4.5");
        match result {
            CommandResult::Action(Action::SetDefaultModel(resolved_id)) => {
                assert_eq!(resolved_id, long_id);
            }
            other => panic!("expected SetDefaultModel(Grok 4.5), got {other:?}"),
        }
    }

    #[test]
    fn run_rejects_effort_for_non_reasoning_model() {
        let mut state = ModelState::default();
        let (id, info) = plain_model("grok-4.5", "Grok 4.5");
        state.available.insert(id, info);
        let mut ctx = dummy_exec_ctx(&state);
        let result = ModelCommand.run(&mut ctx, "Grok 4.5 high");
        // Falls through to "is the whole string a model name?" — which
        // it isn't, so we get an Unknown error.
        assert!(matches!(result, CommandResult::Error(_)));
    }

    /// The bare `/model <name>` form dispatches
    /// `Action::SetDefaultModel(<ModelId>)` instead of the legacy
    /// `Action::SwitchModel { effort: None }`. The dispatcher routes
    /// the typed setter through both `Effect::SwitchModel`
    /// (session-level mutation) AND `Effect::PersistSetting`
    /// (next-session default).
    ///
    /// The payload is the typed `acp::ModelId` (resolved at the slash
    /// boundary), not a String.
    #[test]
    fn run_bare_model_name_dispatches_set_default_model() {
        let mut state = ModelState::default();
        let (id, info) = plain_model("grok-4.5", "Grok 4.5");
        state.available.insert(id.clone(), info);
        let mut ctx = dummy_exec_ctx(&state);
        let result = ModelCommand.run(&mut ctx, "Grok 4.5");
        match result {
            CommandResult::Action(Action::SetDefaultModel(resolved_id)) => {
                assert_eq!(resolved_id, id);
            }
            other => panic!("expected Action::SetDefaultModel(<id>), got {other:?}"),
        }
    }

    /// Case-insensitive matching against the catalog: `/model grok 4.5`
    /// resolves to the same `ModelId` as `/model Grok 4.5`.
    #[test]
    fn run_set_default_model_resolves_case_insensitively() {
        let mut state = ModelState::default();
        let (id, info) = plain_model("grok-4.5", "Grok 4.5");
        state.available.insert(id.clone(), info);
        let mut ctx = dummy_exec_ctx(&state);
        let result = ModelCommand.run(&mut ctx, "grok 4.5");
        match result {
            CommandResult::Action(Action::SetDefaultModel(resolved_id)) => {
                assert_eq!(resolved_id, id);
            }
            other => panic!("expected Action::SetDefaultModel(<id>), got {other:?}"),
        }
    }
}
