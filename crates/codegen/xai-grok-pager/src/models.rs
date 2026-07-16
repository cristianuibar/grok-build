//! `grok models` / `bum models` subcommand.

use anyhow::Result;
use tokio_util::sync::CancellationToken;
use xai_grok_shell::agent::config::Config as AgentConfig;
use xai_grok_shell::cli_models::{AuthStatus, list_models};

use crate::client_identity::{PAGER_CLIENT_TYPE, PAGER_CLIENT_VERSION};

/// Format a single `bum models` list row per UI-SPEC:
/// - current: `  * {id} ({name})`
/// - other:   `  - {id} ({name})`
///
/// When `name` is empty, falls back to `id` inside the parenthetical so rows
/// always keep the `id (name)` shape. No `(default)` text suffix — the star
/// marker alone marks the current model; name carries the provider label.
pub fn format_cli_model_row(is_current: bool, id: &str, name: &str) -> String {
    let display_name = if name.is_empty() { id } else { name };
    let marker = if is_current { "*" } else { "-" };
    format!("  {marker} {id} ({display_name})")
}

pub async fn list_available_models(agent_config: &AgentConfig) -> Result<()> {
    match AuthStatus::resolve(agent_config) {
        AuthStatus::ApiKey => println!("You are using XAI_API_KEY."),
        AuthStatus::LoggedIn(host) => println!("You are logged in with {}.", host),
        AuthStatus::ModelCredentials(model) => {
            println!("Model '{model}' is using its own API key.");
        }
        AuthStatus::DeploymentKey => println!("You are authenticated via deployment key."),
        AuthStatus::NotAuthenticated => println!("You are not authenticated."),
    }
    println!();

    let cancel = CancellationToken::new();
    let spawned = crate::acp::spawn::spawn_grok_shell(agent_config.clone(), &cancel, None).await?;

    let state = list_models(&spawned.channel.tx, PAGER_CLIENT_TYPE, PAGER_CLIENT_VERSION).await?;

    println!("Default model: {}", state.current_model_id.0);
    println!();
    println!("Available models:");
    for m in state.available_models {
        let is_current = m.model_id == state.current_model_id;
        println!("{}", format_cli_model_row(is_current, &m.model_id.0, &m.name));
    }

    cancel.cancel();
    Ok(())
}
