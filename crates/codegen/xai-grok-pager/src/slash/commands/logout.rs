//! `/logout` -- dual-safe fail-closed pointer to selective CLI logout.
//!
//! Bare `/logout` must not dual-wipe both provider slots (AUTH-03 / D-03 / D-05).
//! Use `bum logout --provider xai|codex` or `bum logout --all`.

use crate::app::actions::Action;
use crate::slash::command::{CommandExecCtx, CommandResult, SlashCommand};

pub struct LogoutCommand;

impl SlashCommand for LogoutCommand {
    fn name(&self) -> &str {
        "logout"
    }

    fn description(&self) -> &str {
        "Log out of a provider (CLI: bum logout --provider …)"
    }

    fn usage(&self) -> &str {
        "/logout"
    }

    fn run(&self, _ctx: &mut CommandExecCtx, _args: &str) -> CommandResult {
        CommandResult::Action(Action::Logout)
    }
}
