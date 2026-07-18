//! Agent — a fully built agent: definition + session context.

use std::sync::Arc;

use xai_grok_sampling_types::HostedTool;
use xai_grok_tools::bridge::ToolBridge;
use xai_grok_tools::types::definition::ToolDefinition;

use crate::compaction::CompactionPolicy;
use crate::config::{AgentDefinition, CompletionRequirement, PermissionMode};
use crate::prompt::context::PromptContext;
use crate::system_reminder::ReminderPolicy;

/// A fully built agent: definition + session context.
///
/// NOT portable — tied to a specific session via its ToolBridge,
/// rendered system prompt, and session-level policies.
///
/// Created by AgentBuilder from an AgentDefinition + session context.
///
/// The Agent is effectively immutable after construction. It holds
/// Arc<ToolBridge> — mutations to tool state (MCP registration,
/// completion tracking, retry config) go through ToolBridge's
/// internal locks.
pub struct Agent {
    /// The definition this agent was built from.
    definition: AgentDefinition,

    /// The context that produced the current system prompt.
    /// Stored for inspection, re-rendering, and serialization.
    prompt_context: PromptContext,

    /// The rendered system prompt (cached from prompt_context.render()).
    system_prompt: String,

    /// The tool bridge — owns ToolRegistry + ToolState + SessionContext.
    tool_bridge: Arc<ToolBridge>,

    /// Session-level policies.
    reminder_policy: ReminderPolicy,
    compaction_policy: CompactionPolicy,

    /// Backend-hosted tools to include in API requests.
    /// These are sent as native Responses API types (e.g., `WebSearch`)
    /// and executed server-side by the agentic sampler.
    hosted_tools: Vec<HostedTool>,

    /// Build-time toggle for server-side search tools. ANDed at request
    /// time with the per-model `SessionActor::supports_backend_search`.
    backend_search_enabled: bool,
}

impl Agent {
    /// Create a new Agent.
    ///
    /// Normally called by `AgentBuilder::build()`. Exposed publicly for
    /// test helpers that need to construct an Agent with a pre-built ToolBridge.
    pub fn new(
        definition: AgentDefinition,
        prompt_context: PromptContext,
        system_prompt: String,
        tool_bridge: Arc<ToolBridge>,
        reminder_policy: ReminderPolicy,
        compaction_policy: CompactionPolicy,
        hosted_tools: Vec<HostedTool>,
        backend_search_enabled: bool,
    ) -> Self {
        Self {
            definition,
            prompt_context,
            system_prompt,
            tool_bridge,
            reminder_policy,
            compaction_policy,
            hosted_tools,
            backend_search_enabled,
        }
    }

    // ── From definition ──────────────────────────────────────────────

    /// Agent name (unique identifier).
    pub fn name(&self) -> &str {
        &self.definition.name
    }

    /// Agent description.
    pub fn description(&self) -> &str {
        &self.definition.description
    }

    /// The full agent definition.
    pub fn definition(&self) -> &AgentDefinition {
        &self.definition
    }

    /// Permission mode for this agent.
    pub fn permission_mode(&self) -> &PermissionMode {
        &self.definition.permission_mode
    }

    /// Completion requirement, if any.
    pub fn completion_requirement(&self) -> Option<&CompletionRequirement> {
        self.definition.completion_requirement.as_ref()
    }

    // ── Session-level ────────────────────────────────────────────────

    /// The rendered system prompt.
    pub fn system_prompt(&self) -> &str {
        &self.system_prompt
    }

    /// Compact system prompt for post-compaction use.
    ///
    /// Returns a static string — the compact prompt never changes at runtime.
    pub fn compact_system_prompt(&self) -> &str {
        crate::prompt::template::COMPACT_SYSTEM_PROMPT
    }

    /// The tool bridge for this agent.
    pub fn tool_bridge(&self) -> &Arc<ToolBridge> {
        &self.tool_bridge
    }

    /// Compaction policy.
    pub fn compaction_policy(&self) -> &CompactionPolicy {
        &self.compaction_policy
    }

    /// Reminder policy.
    pub fn reminder_policy(&self) -> &ReminderPolicy {
        &self.reminder_policy
    }

    /// Cached AGENTS.md section (derived from prompt_context).
    pub fn agents_md_section(&self) -> Option<String> {
        self.prompt_context.format_agents_md_section()
    }

    /// AGENTS.md content formatted for user-message injection.
    ///
    /// Returns the `<system-reminder>` block to prepend as a user message,
    /// respecting audience (compacted for subagents) and template.
    pub fn agents_md_user_reminder(&self) -> Option<String> {
        self.prompt_context.agents_md_user_reminder()
    }

    /// Personas content formatted for user-message injection.
    ///
    /// Returns the `<system-reminder>` block to prepend as a user message,
    /// respecting audience (suppressed for subagents) and template.
    pub fn personas_user_reminder(&self) -> Option<String> {
        self.prompt_context.personas_user_reminder()
    }

    /// The structured prompt context for inspection and re-rendering.
    pub fn prompt_context(&self) -> &PromptContext {
        &self.prompt_context
    }

    /// Audience this agent's prompt was rendered for (Primary or Subagent).
    ///
    /// Used by the runtime turn-end TodoGate together with
    /// [`crate::AgentDefinition::carries_task_completion_discipline`] to
    /// decide whether the active prompt actually carries the discipline
    /// rules the gate's reminder text invokes.
    pub fn prompt_audience(&self) -> crate::prompt::context::PromptAudience {
        self.prompt_context.audience
    }

    /// Tool definitions for the sampling API — delegates to ToolBridge.
    pub async fn tool_definitions(&self) -> Vec<ToolDefinition> {
        self.tool_bridge.tool_definitions().await
    }

    /// Backend-hosted tools that should be included in API requests.
    /// These are sent as native types (e.g., `rs::Tool::WebSearch`) and
    /// executed server-side by the agentic sampler.
    pub fn hosted_tools(&self) -> &[HostedTool] {
        &self.hosted_tools
    }

    /// Build-time toggle for server-side search tools. Callers should
    /// AND this with the per-model `supports_backend_search` flag to
    /// decide whether to ship `hosted_tools` on a request. Do not use
    /// `hosted_tools().is_empty()` as a proxy — the list also depends
    /// on web-search config.
    pub fn backend_search_enabled(&self) -> bool {
        self.backend_search_enabled
    }

    /// Built-in tool definitions only (excludes MCP tools).
    pub async fn tool_definitions_builtins_only(&self) -> Vec<ToolDefinition> {
        self.tool_bridge.tool_definitions_builtins_only().await
    }

    /// Whether auto-compact should trigger given current token usage.
    ///
    /// `context_window` comes from the session's SamplingConfig (model-provided).
    pub fn should_auto_compact(
        &self,
        total_tokens: u64,
        context_window: std::num::NonZeroU64,
    ) -> bool {
        let cw = context_window.get();
        xai_token_estimation::exceeds_threshold(
            total_tokens,
            cw,
            self.compaction_policy.auto_compact_threshold_percent as u8,
        )
    }

    /// Update completion and retry policies from a new definition.
    ///
    /// Does NOT rebuild the tool registry or re-render prompts.
    /// Used for mid-session mode switching.
    pub async fn update_policies_from_definition(&self, _def: &AgentDefinition) {
        // TODO: completion requirements and retry configs are now part of
        // ToolServerConfig and handled at registry finalization time.
        // Mid-session policy updates are not yet supported in the new architecture.
    }

    /// Re-render the system prompt from current ToolBridge state
    /// (tool name overrides, disabled tools). Called by hosts after
    /// mid-session tool-override updates.
    pub async fn finalize_prompt(&mut self) {
        self.prompt_context.build_timestamp_utc = chrono::Utc::now().to_rfc3339();

        self.system_prompt = self
            .prompt_context
            .render(&self.tool_bridge)
            .await
            .unwrap_or_default();
    }

    /// Update the system-prompt identity label and re-render (requires `&mut self`).
    pub async fn set_system_prompt_label_and_rerender(&mut self, label: impl Into<String>) {
        let label = label.into();
        self.prompt_context.system_prompt_label = label.clone();
        self.finalize_prompt().await;
        // Prefer a cheap prefix rebind if render failed empty.
        if self.system_prompt.is_empty() {
            self.system_prompt = rebind_system_prompt_label_prefix(
                self.system_prompt.as_str(),
                "",
                &label,
            );
        }
    }

    /// Current system-prompt identity label (not UI picker name).
    pub fn system_prompt_label(&self) -> &str {
        &self.prompt_context.system_prompt_label
    }

    /// Sync identity rebind for `RefCell` session actors: no `.await`, no exclusive
    /// long holds. Rewrites the leading `You are {label}.` in the cached prompt.
    ///
    /// Prefer this during mid-session model switch when a concurrent turn may
    /// already hold `agent.borrow()` — full re-render needs exclusive access.
    pub fn rebind_system_prompt_label(&mut self, label: impl Into<String>) {
        let new_label = label.into();
        let old = self.prompt_context.system_prompt_label.clone();
        if old == new_label {
            return;
        }
        self.system_prompt =
            rebind_system_prompt_label_prefix(&self.system_prompt, &old, &new_label);
        self.prompt_context.system_prompt_label = new_label;
    }

    /// Install a prompt context + rendered system prompt string.
    pub fn install_rendered_prompt(&mut self, ctx: PromptContext, system_prompt: String) {
        self.prompt_context = ctx;
        self.system_prompt = system_prompt;
    }

    /// Re-render the system prompt for a different definition, reusing
    /// the existing ToolBridge. Used for mid-session mode switching.
    pub async fn render_prompt_for_definition(&self, definition: &AgentDefinition) -> String {
        let mut ctx = self.prompt_context.clone();
        ctx.prompt_mode = definition.prompt_mode.clone();
        ctx.prompt_body = definition.prompt_body.clone();
        ctx.system_prompt = definition.system_prompt.clone();
        ctx.build_timestamp_utc = chrono::Utc::now().to_rfc3339();

        // Clear agents_md if the new definition doesn't want it
        if !definition.agents_md {
            ctx.agents_md_files.clear();
        }

        ctx.render(&self.tool_bridge).await.unwrap_or_default()
    }
}

/// Rewrite the leading identity prefix `You are {old}.` → `You are {new}.`.
///
/// Falls back to replacing the first `You are ….` span when the old label is
/// unknown or the prompt was built with a different default.
pub fn rebind_system_prompt_label_prefix(prompt: &str, old_label: &str, new_label: &str) -> String {
    if old_label == new_label {
        return prompt.to_owned();
    }
    if !old_label.is_empty() {
        let old_prefix = format!("You are {old_label}.");
        let new_prefix = format!("You are {new_label}.");
        if let Some(rest) = prompt.strip_prefix(&old_prefix) {
            return format!("{new_prefix}{rest}");
        }
        // Older templates: "You are {label} released by xAI."
        let old_xai = format!("You are {old_label} released by xAI.");
        let new_xai = format!("You are {new_label}.");
        if let Some(rest) = prompt.strip_prefix(&old_xai) {
            return format!("{new_xai}{rest}");
        }
    }
    if let Some(rest) = prompt.strip_prefix("You are ") {
        if let Some(dot) = rest.find(". ") {
            return format!("You are {new_label}. {}", &rest[dot + 2..]);
        }
        if let Some(dot) = rest.find('.') {
            return format!("You are {new_label}.{}", &rest[dot + 1..]);
        }
    }
    prompt.to_owned()
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU64;

    /// Standalone function testing the same logic as Agent::should_auto_compact
    fn should_auto_compact_check(total_tokens: u64, context_window: u64, threshold: u32) -> bool {
        let cw = NonZeroU64::new(context_window).expect("test context_window must be non-zero");
        let usage_percent = (total_tokens * 100) / cw.get();
        usage_percent >= threshold as u64
    }

    #[test]
    fn test_should_auto_compact_below_threshold() {
        // 80% of 100K window with 85% threshold → false
        assert!(!should_auto_compact_check(80_000, 100_000, 85));
    }

    #[test]
    fn test_should_auto_compact_above_threshold() {
        // 90% of 100K window with 85% threshold → true
        assert!(should_auto_compact_check(90_000, 100_000, 85));
    }

    #[test]
    fn test_should_auto_compact_at_threshold() {
        // Exactly 85% of 100K window with 85% threshold → true
        assert!(should_auto_compact_check(85_000, 100_000, 85));
    }

    #[test]
    fn test_should_auto_compact_empty_usage() {
        // 0 tokens used → false
        assert!(!should_auto_compact_check(0, 100_000, 85));
    }

    #[test]
    fn test_should_auto_compact_100_percent_threshold() {
        // 100% threshold → only triggers when fully used
        assert!(!should_auto_compact_check(99_999, 100_000, 100));
        assert!(should_auto_compact_check(100_000, 100_000, 100));
    }

    #[test]
    fn rebind_system_prompt_label_prefix_rewrites_modern_and_legacy() {
        use super::rebind_system_prompt_label_prefix;
        let modern = "You are Grok. You are an interactive CLI tool.";
        assert_eq!(
            rebind_system_prompt_label_prefix(modern, "Grok", "GPT-5.6 Sol"),
            "You are GPT-5.6 Sol. You are an interactive CLI tool."
        );
        let legacy = "You are Grok released by xAI. You are an interactive CLI tool.";
        assert_eq!(
            rebind_system_prompt_label_prefix(legacy, "Grok", "GPT-5.6 Sol"),
            "You are GPT-5.6 Sol. You are an interactive CLI tool."
        );
    }
}
