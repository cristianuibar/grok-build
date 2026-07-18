use super::*;
use crate::remote::DEFAULT_CONTEXT_WINDOW;
use xai_chat_state::conversation_util::replace_or_insert_system_head;

/// Clear only provider-scoped encrypted reasoning payloads.
///
/// The surrounding conversation structure, including each reasoning item's id,
/// summary, and non-encrypted content, is intentionally retained so a provider
/// transition does not discard useful session context.
fn clear_encrypted_reasoning(conversation: &mut [ConversationItem]) -> bool {
    let mut changed = false;
    for item in conversation {
        if let ConversationItem::Reasoning(reasoning) = item
            && reasoning.encrypted_content.take().is_some()
        {
            changed = true;
        }
    }
    changed
}

fn requires_provider_transition_sanitization(
    previous: Option<crate::agent::config::ModelProvider>,
    target: crate::agent::config::ModelProvider,
) -> bool {
    previous != Some(target)
}

impl SessionActor {
    /// Record a model-route change before any asynchronous mutation can let a
    /// completed sampler turn observe stale provider state.
    pub(super) fn advance_provider_transition(
        &self,
        target: crate::agent::config::ModelProvider,
    ) -> ProviderTransition {
        let previous = self.provider_transition.get();
        let next = ProviderTransition {
            active_provider: Some(target),
            epoch: previous
                .epoch
                .checked_add(1)
                .expect("provider transition epoch overflow"),
        };
        self.provider_transition.set(next);
        previous
    }

    /// Snapshot the active provider immediately before sending a sampler turn.
    pub(super) fn provider_transition_snapshot(&self) -> ProviderTransition {
        self.provider_transition.get()
    }

    /// Clear encrypted reasoning from persisted history while no sampler turn
    /// can concurrently append a response item.
    async fn sanitize_provider_history(&self, transition: ProviderTransition) {
        let mut conversation = self.chat_state_handle.get_conversation().await;
        if clear_encrypted_reasoning(&mut conversation) {
            tracing::info!(
                session_id = %self.session_info.id.0,
                target_provider = ?transition.active_provider,
                transition_epoch = transition.epoch,
                "cleared encrypted reasoning before provider transition"
            );
            self.chat_state_handle.replace_conversation(conversation);
        }
    }

    /// Drain a history cleanup deferred by a mid-turn provider switch.
    ///
    /// The caller runs this only after response items have been enqueued or
    /// immediately before creating the next sampler request. That ordering
    /// keeps `get_conversation` + `replace_conversation` from overwriting a
    /// late response that belongs to the old provider.
    pub(super) async fn sanitize_pending_provider_history(&self) {
        if let Some(transition) = self.pending_provider_history_sanitization.take() {
            self.sanitize_provider_history(transition).await;
        }
    }

    /// Remove encrypted payloads from a late response only when its origin is
    /// unknown or its known provider differs from the active target provider.
    /// A same-provider Codex switch deliberately retains its encrypted
    /// continuity, even though the model-switch epoch advanced.
    pub(super) fn sanitize_late_response_item(
        &self,
        item: &mut ConversationItem,
        turn_snapshot: ProviderTransition,
    ) {
        let current = self.provider_transition.get();
        let known_same_provider = matches!(
            (turn_snapshot.active_provider, current.active_provider),
            (Some(origin), Some(active)) if origin == active
        );
        if !known_same_provider {
            let changed = clear_encrypted_reasoning(std::slice::from_mut(item));
            if changed {
                tracing::debug!(
                    session_id = %self.session_info.id.0,
                    turn_epoch = turn_snapshot.epoch,
                    active_epoch = current.epoch,
                    turn_provider = ?turn_snapshot.active_provider,
                    active_provider = ?current.active_provider,
                    "cleared encrypted reasoning from a late or unknown-provider response"
                );
            }
        }
    }

    pub(super) async fn handle_set_session_model(
        &self,
        sampling_config: xai_grok_sampler::SamplerConfig,
        auth_type: xai_chat_state::AuthType,
        provider: crate::agent::config::ModelProvider,
        use_concise: bool,
        apply_prompt_override: bool,
        skip_prompt_rewrite: bool,
        auto_compact_threshold_percent: u8,
    ) -> Result<acp::ModelId, acp::Error> {
        let model_id = acp::ModelId::new(sampling_config.model.clone());
        // Resolve the previous provider and publish the target route before
        // invalidating model facts or awaiting chat-state work. This is the
        // ordering barrier used by late sampler responses below.
        let previous_transition = self.advance_provider_transition(provider);
        let should_sanitize_history = requires_provider_transition_sanitization(
            previous_transition.active_provider,
            provider,
        );
        if should_sanitize_history {
            let transition = self.provider_transition.get();
            if self.state.lock().await.running_task.is_some() {
                // The in-flight sampler can enqueue response items between a
                // history snapshot and replacement. Let the turn drain this
                // after it has sanitized and enqueued those late items.
                self.pending_provider_history_sanitization
                    .set(Some(transition));
                tracing::debug!(
                    session_id = %self.session_info.id.0,
                    target_provider = ?provider,
                    transition_epoch = transition.epoch,
                    "deferred encrypted reasoning cleanup until in-flight turn reaches a safe point"
                );
            } else {
                self.sanitize_provider_history(transition).await;
            }
        }
        let new_context_window = self.compaction.context_window_override.unwrap_or_else(|| {
            std::num::NonZeroU64::new(sampling_config.context_window).unwrap_or_else(|| {
                std::num::NonZeroU64::new(DEFAULT_CONTEXT_WINDOW)
                    .expect("DEFAULT_CONTEXT_WINDOW is non-zero")
            })
        });
        let prev_threshold = self.compaction.threshold_percent.get();
        if prev_threshold != auto_compact_threshold_percent {
            tracing::info!(
                session_id = % self.session_info.id.0, new_model = % sampling_config
                .model, old_threshold = prev_threshold, new_threshold =
                auto_compact_threshold_percent,
                "auto_compact_threshold_percent updated for model switch"
            );
        }
        self.compaction
            .threshold_percent
            .set(auto_compact_threshold_percent);
        self.supports_backend_search
            .set(sampling_config.supports_backend_search);
        self.compactions_remaining
            .set(sampling_config.compactions_remaining);
        self.compaction_at_tokens
            .set(sampling_config.compaction_at_tokens);
        xai_grok_telemetry::unified_log::info(
            "backend_search: model switch",
            Some(self.session_info.id.0.as_ref()),
            Some(serde_json::json!(
                { "new_model" : & sampling_config.model, "api_backend" :
                format!("{:?}", sampling_config.api_backend),
                "supports_backend_search" : sampling_config.supports_backend_search,
                }
            )),
        );
        // Production transform A: prepared auth_type + api_key written verbatim.
        // Do **not** re-resolve auth_type with xAI AuthManager (D-09 / T-04-16).
        let prepared = crate::agent::config::PreparedSamplingConfig {
            sampler_config: sampling_config,
            auth_type,
            provider,
        };
        let existing = self.chat_state_handle.get_credentials().await;
        let (chat_sampling, credentials) =
            crate::agent::config::apply_prepared_sampling_to_chat_state_fields(
                &prepared,
                &existing,
                new_context_window,
            );
        self.chat_state_handle.update_sampling_config(chat_sampling);
        self.chat_state_handle.update_credentials(credentials);
        // Re-bind for subsequent uses of sampling_config fields below.
        let sampling_config = prepared.sampler_config;
        self.model_auth_facts.replace(None);
        self.signals_handle()
            .record_model_usage(&sampling_config.model);
        if apply_prompt_override && !skip_prompt_rewrite {
            let mut conversation = self.chat_state_handle.get_conversation().await;
            for item in conversation.iter_mut() {
                if let ConversationItem::System(sys) = item {
                    if use_concise {
                        sys.content = std::sync::Arc::<str>::from(
                            xai_grok_agent::prompt::template::COMPACT_SYSTEM_PROMPT,
                        );
                    } else {
                        sys.content =
                            std::sync::Arc::<str>::from(self.agent.borrow().system_prompt());
                    }
                    break;
                }
            }
            self.chat_state_handle.replace_conversation(conversation);
        } else if !apply_prompt_override {
            tracing::info!(
                session_id = % self.session_info.id.0, model_id = % model_id.0,
                "handle_set_session_model: skipping prompt override (apply_prompt_override=false)"
            );
        } else {
            tracing::info!(
                session_id = % self.session_info.id.0, model_id = % model_id.0,
                "handle_set_session_model: skipping prompt rewrite (just rebuilt harness)"
            );
        }
        let agent_name = self.agent.borrow().definition().name.clone();
        let _ = self
            .notifications
            .persistence_tx
            .send(PersistenceMsg::CurrentModel {
                model_id: model_id.clone(),
                agent_name: Some(agent_name),
                reasoning_effort: Some(sampling_config.reasoning_effort),
            });
        Ok(model_id)
    }
    /// Handle [`SessionCommand::RebuildAgentForDefinition`].
    ///
    /// Builds a fresh [`xai_grok_agent::Agent`] from the cached
    /// [`crate::session::agent_rebuild::AgentRebuildSpec`] + the supplied
    /// [`xai_grok_agent::AgentDefinition`], replaces `self.agent`,
    /// rewrites the system message in the conversation, persists the
    /// new prompt artifacts, and updates `active_agent_type`.
    ///
    /// Triggered from `MvpAgent::set_session_model` only when the new
    /// model's `agent_type` differs from the session's current
    /// `active_agent_type` AND `turn_count == 0` (no user message has
    /// been sent yet). Defense-in-depth: rejects if a turn is in flight.
    pub(super) async fn handle_rebuild_agent_for_definition(
        &self,
        definition: xai_grok_agent::AgentDefinition,
    ) -> Result<(), acp::Error> {
        {
            let state = self.state.lock().await;
            if state.running_task.is_some() {
                tracing::warn!(
                    session_id = % self.session_info.id.0, new_agent_type = % definition
                    .name,
                    "handle_rebuild_agent_for_definition: turn in flight, rejecting rebuild"
                );
                return Err(acp::Error::internal_error()
                    .data("rebuild_agent: turn in flight, refusing to rebuild harness"));
            }
        }
        let new_agent_name = definition.name.clone();
        tracing::info!(
            session_id = % self.session_info.id.0, new_agent_type = % new_agent_name,
            "handle_rebuild_agent_for_definition: rebuilding harness"
        );
        let new_agent = self
            .rebuild_spec
            .build_agent(definition)
            .await
            .map_err(|e| {
                tracing::error!(
                    session_id = % self.session_info.id.0, new_agent_type = %
                    new_agent_name, error = % e,
                    "handle_rebuild_agent_for_definition: AgentBuilder::build failed"
                );
                acp::Error::internal_error().data(format!(
                    "rebuild_agent: build failed for agent_type={new_agent_name}: {e}"
                ))
            })?;
        let new_system_prompt = new_agent.system_prompt().to_string();
        let mut new_prompt_context = new_agent.prompt_context().clone();
        new_prompt_context.normalize_for_persistence();
        if let Some(handle) = self.compaction.prefire.take_handle() {
            handle.abort();
            let _ = handle.await;
            self.compaction.prefire.finish();
        }
        self.compaction.prefire.clear();
        *self.agent.borrow_mut() = new_agent;
        *self.active_agent_type.lock() = Some(new_agent_name.clone());
        self.queue_exit_reminder_on_approved_exit.store(
            self.is_cursor_harness(),
            std::sync::atomic::Ordering::Relaxed,
        );
        if let Err(e) = self.workspace_ops.bind_local_session(
            &self.session_id_string(),
            self.tool_context.cwd.as_path().to_path_buf(),
            self.tool_context.hunk_tracker_handle.clone(),
            self.agent.borrow().tool_bridge().toolset(),
            None,
        ) {
            tracing::warn!(
                error = % e, "failed to rebind local session toolset after agent rebuild"
            );
        }
        {
            let bridge = self.agent.borrow().tool_bridge().clone();
            let snapshot = self.tool_metadata_snapshot.clone();
            let tool_index = crate::session::tool_index::Bm25ToolSearchIndex::new(snapshot);
            bridge
                .update_resource(xai_grok_tools::types::tool_index::ToolIndex(
                    std::sync::Arc::new(tool_index),
                ))
                .await;
            if let Some(client) = self.rebuild_spec.managed_gateway_tool_client.clone() {
                bridge.update_resource(client).await;
            }
            let plan_path = self.plan_mode.lock().plan_file_path().to_path_buf();
            bridge
                .update_resource(xai_grok_tools::types::resources::PlanFilePath(plan_path))
                .await;
            if let Some(display_cwd) = self.display_cwd.get() {
                bridge
                    .set_display_cwd(std::path::PathBuf::from(display_cwd))
                    .await;
            }
            bridge
                .update_resource(
                    xai_grok_tools::implementations::grok_build::update_goal::GoalUpdateHandle(
                        self.goal_update_tx.clone(),
                    ),
                )
                .await;
            self.inject_deny_read_globs().await;
        }
        {
            let notified = self.mcp_handshakes_done.notified();
            tokio::pin!(notified);
            let needs_wait = {
                let s = self.mcp_state.lock().await;
                !s.configs.is_empty() && !s.is_initialized()
            };
            if needs_wait {
                const TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);
                tokio::select! {
                    () = & mut notified => {} () = tokio::time::sleep(TIMEOUT) => {
                    tracing::warn!(session_id = % self.session_info.id.0,
                    "handle_rebuild_agent_for_definition: timed out waiting for MCP handshakes");
                    }
                }
            }
        }
        self.re_register_mcp_tools_on_rebuilt_bridge().await;
        if let Some(old_handle) = self.deferred_prefix.take() {
            old_handle.abort();
        }
        let new_user_prefix = self.build_user_message_prefix().await;
        {
            let mut conversation = self.chat_state_handle.get_conversation().await;
            let _ = replace_or_insert_system_head(&mut conversation, &new_system_prompt);
            let drop_startup_skill_reminder = false;
            Self::rewrite_zero_turn_prefix(
                &mut conversation,
                new_user_prefix,
                drop_startup_skill_reminder,
            );
            if !conversation_has_project_instructions(&conversation)
                && let Some(agents_md_reminder) = self.agent.borrow().agents_md_user_reminder()
            {
                let agents_md_at = conversation.len().min(2);
                conversation.insert(
                    agents_md_at,
                    ConversationItem::project_instructions(agents_md_reminder),
                );
            }
            self.inject_baseline_skill_reminder(&mut conversation).await;
            self.chat_state_handle.replace_conversation(conversation);
        }
        save_prompt_context(&self.session_info, &new_prompt_context);
        save_system_prompt(&self.session_info, &new_system_prompt);
        let snapshot = self.chat_state_handle.get_conversation().await;
        persist_chat_history_jsonl_sync(&self.session_info, &snapshot);
        self.mcp_reminder_dirty
            .store(true, std::sync::atomic::Ordering::Relaxed);
        self.send_available_commands_update().await;
        tracing::info!(
            session_id = % self.session_info.id.0, new_agent_type = % new_agent_name,
            "handle_rebuild_agent_for_definition: harness rebuild complete"
        );
        Ok(())
    }
    /// Apply a client-supplied `systemPromptOverride` on session attach without
    /// wiping user/assistant history: swap only the leading `System` message,
    /// atomically inside the `ChatStateActor` (see
    /// `ChatStateCommand::ReplaceSystemHead` for the serialization guarantees).
    /// `system_prompt.txt` (not owned by the persistence actor) is saved
    /// directly, even on a head no-op, so a previously-diverged secondary
    /// artifact self-heals. Skipped entirely on a verbatim mirror-fork
    /// (`preserve_inherited_system`).
    pub(super) async fn handle_replace_system_prompt(&self, system_prompt: String) {
        if self.startup_hints.preserve_inherited_system {
            tracing::debug!(
                session_id = % self.session_info.id.0,
                "handle_replace_system_prompt: skipped (preserve_inherited_system)"
            );
            return;
        }
        let Some(changed) = self
            .chat_state_handle
            .replace_system_head(&system_prompt)
            .await
        else {
            tracing::error!(
                session_id = % self.session_info.id.0,
                "handle_replace_system_prompt: chat-state actor unavailable; override not applied"
            );
            return;
        };
        save_system_prompt(&self.session_info, &system_prompt);
        if changed {
            tracing::info!(
                session_id = % self.session_info.id.0, prompt_len = system_prompt.len(),
                "handle_replace_system_prompt: client override applied"
            );
        } else {
            tracing::debug!(
                session_id = % self.session_info.id.0,
                "handle_replace_system_prompt: head already matches, no-op"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use xai_grok_sampling_types::rs;

    fn encrypted_reasoning() -> ConversationItem {
        ConversationItem::Reasoning(rs::ReasoningItem {
            id: "reasoning_1".to_string(),
            summary: vec![rs::SummaryPart::SummaryText(rs::SummaryTextContent {
                text: "visible summary".to_string(),
            })],
            content: Some(vec![rs::ReasoningTextContent {
                text: "non-encrypted reasoning".to_string(),
            }]),
            encrypted_content: Some("provider-scoped-encrypted-payload".to_string()),
            status: None,
        })
    }

    fn reasoning_after_sanitization(conversation: &[ConversationItem]) -> &rs::ReasoningItem {
        let Some(ConversationItem::Reasoning(reasoning)) = conversation.get(3) else {
            panic!("expected reasoning item at the preserved history position");
        };
        reasoning
    }

    fn representative_history() -> Vec<ConversationItem> {
        vec![
            ConversationItem::user("ordinary user context"),
            ConversationItem::assistant("ordinary assistant context"),
            ConversationItem::tool_result("call_1", "ordinary tool context"),
            encrypted_reasoning(),
        ]
    }

    #[test]
    fn cross_provider_transition_sanitizes_existing_and_late_reasoning() {
        assert!(requires_provider_transition_sanitization(
            Some(crate::agent::config::ModelProvider::Xai),
            crate::agent::config::ModelProvider::Codex,
        ));

        let mut existing_history = representative_history();
        assert!(clear_encrypted_reasoning(&mut existing_history));
        assert!(matches!(existing_history[0], ConversationItem::User(_)));
        assert!(matches!(
            existing_history[1],
            ConversationItem::Assistant(_)
        ));
        assert!(matches!(
            existing_history[2],
            ConversationItem::ToolResult(_)
        ));
        let existing_reasoning = reasoning_after_sanitization(&existing_history);
        assert_eq!(existing_reasoning.id, "reasoning_1");
        assert_eq!(existing_reasoning.summary.len(), 1);
        assert_eq!(
            existing_reasoning
                .content
                .as_ref()
                .and_then(|content| content.first())
                .map(|content| content.text.as_str()),
            Some("non-encrypted reasoning")
        );
        assert!(existing_reasoning.encrypted_content.is_none());

        let mut late_response = vec![encrypted_reasoning()];
        assert!(clear_encrypted_reasoning(&mut late_response));
        let Some(ConversationItem::Reasoning(late_reasoning)) = late_response.first() else {
            panic!("expected a late reasoning response item");
        };
        assert_eq!(late_reasoning.id, "reasoning_1");
        assert_eq!(late_reasoning.summary.len(), 1);
        assert!(late_reasoning.content.is_some());
        assert!(late_reasoning.encrypted_content.is_none());
    }

    #[test]
    fn same_provider_codex_transition_preserves_encrypted_reasoning() {
        assert!(!requires_provider_transition_sanitization(
            Some(crate::agent::config::ModelProvider::Codex),
            crate::agent::config::ModelProvider::Codex,
        ));

        let mut history = representative_history();
        if requires_provider_transition_sanitization(
            Some(crate::agent::config::ModelProvider::Codex),
            crate::agent::config::ModelProvider::Codex,
        ) {
            clear_encrypted_reasoning(&mut history);
        }
        assert_eq!(
            reasoning_after_sanitization(&history)
                .encrypted_content
                .as_deref(),
            Some("provider-scoped-encrypted-payload")
        );
    }

    #[test]
    fn unknown_prior_provider_sanitizes_encrypted_reasoning() {
        assert!(requires_provider_transition_sanitization(
            None,
            crate::agent::config::ModelProvider::Codex,
        ));

        let mut history = representative_history();
        assert!(clear_encrypted_reasoning(&mut history));
        assert!(
            reasoning_after_sanitization(&history)
                .encrypted_content
                .is_none()
        );
    }
}
