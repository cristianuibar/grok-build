//! Tests for feedback / remember / btw / recap dispatchers.

use super::*;
use crate::app::dispatch::{recap_unavailable_toast, scrollback_has_user_messages};

#[test]
fn recap_unavailable_toast_empty_vs_with_messages() {
    assert_eq!(recap_unavailable_toast(false), "No messages yet");
    assert_eq!(recap_unavailable_toast(true), "Couldn't generate recap");
}

#[test]
fn manual_recap_with_no_messages_toasts_empty_state_and_skips_request() {
    let mut app = test_app_with_agent();
    app.session_recap_available = true;
    let id = AgentId(0);
    {
        let agent = app.agents.get_mut(&id).unwrap();
        agent.prompt.set_text("/recap");
        assert!(!scrollback_has_user_messages(&agent.scrollback));
    }

    let effects = dispatch(Action::SendRecap { auto: false }, &mut app);

    assert!(
        effects.is_empty(),
        "empty session must not fire x.ai/recap: {effects:?}"
    );
    let agent = app.agents.get(&id).unwrap();
    assert!(agent.pending_recap_entry.is_none(), "no loading spinner");
    assert_eq!(
        agent.toast.as_ref().map(|(s, _)| s.as_str()),
        Some("No messages yet"),
        "empty session should say No messages yet, not Couldn't generate recap"
    );
    assert_eq!(agent.prompt.text(), "", "slash command text is cleared");
}

#[test]
fn manual_recap_with_messages_requests_and_shows_spinner() {
    let mut app = test_app_with_agent();
    app.session_recap_available = true;
    let id = AgentId(0);
    {
        let agent = app.agents.get_mut(&id).unwrap();
        agent
            .scrollback
            .push_block(RenderBlock::user_prompt("hello"));
        assert!(scrollback_has_user_messages(&agent.scrollback));
    }

    let effects = dispatch(Action::SendRecap { auto: false }, &mut app);

    assert!(
        matches!(effects.as_slice(), [Effect::SendRecap { auto: false, .. }]),
        "expected SendRecap effect, got {effects:?}"
    );
    let agent = app.agents.get(&id).unwrap();
    assert!(
        agent.pending_recap_entry.is_some(),
        "manual recap shows a loading spinner when there is something to summarize"
    );
    assert!(agent.toast.is_none());
}

/// Regression: during session/load, scrollback is batched so
/// `turn_count()` stays 0 until `end_batch`, but UserPrompt entries may already
/// be present. Manual `/recap` must still request a recap.
#[test]
fn manual_recap_during_batch_load_with_prompts_still_requests() {
    let mut app = test_app_with_agent();
    app.session_recap_available = true;
    let id = AgentId(0);
    {
        let agent = app.agents.get_mut(&id).unwrap();
        agent.scrollback.begin_batch();
        agent
            .scrollback
            .push_block(RenderBlock::user_prompt("hello from resume"));
        // Batched push defers rebuild_turns — turn index is stale, entries aren't.
        assert_eq!(agent.scrollback.turn_count(), 0);
        assert!(scrollback_has_user_messages(&agent.scrollback));
    }

    let effects = dispatch(Action::SendRecap { auto: false }, &mut app);

    assert!(
        matches!(effects.as_slice(), [Effect::SendRecap { auto: false, .. }]),
        "batched resume with user prompts must still fire x.ai/recap: {effects:?}"
    );
    let agent = app.agents.get(&id).unwrap();
    assert!(agent.pending_recap_entry.is_some());
    assert!(agent.toast.is_none());
    // Clean up batch for the test fixture (not required for the assertion).
    app.agents.get_mut(&id).unwrap().scrollback.end_batch();
}

/// While session replay is still streaming, don't claim "No messages yet" even
/// if scrollback looks empty — history may arrive on the next notification.
#[test]
fn manual_recap_while_loading_replay_still_requests() {
    let mut app = test_app_with_agent();
    app.session_recap_available = true;
    let id = AgentId(0);
    {
        let agent = app.agents.get_mut(&id).unwrap();
        agent.session.loading_replay = true;
        assert!(!scrollback_has_user_messages(&agent.scrollback));
    }

    let effects = dispatch(Action::SendRecap { auto: false }, &mut app);

    assert!(
        matches!(effects.as_slice(), [Effect::SendRecap { auto: false, .. }]),
        "loading_replay must not short-circuit to No messages yet: {effects:?}"
    );
    let agent = app.agents.get(&id).unwrap();
    assert!(agent.pending_recap_entry.is_some());
    assert!(agent.toast.is_none());
}

#[test]
fn recap_request_transport_failure_with_no_turns_uses_empty_toast() {
    let mut app = test_app_with_agent();
    let id = AgentId(0);
    let session_id = app.agents[&id].session.session_id.clone().unwrap();
    {
        let agent = app.agents.get_mut(&id).unwrap();
        let spinner = agent
            .scrollback
            .push(crate::scrollback::entry::ScrollbackEntry::running(
                RenderBlock::session_event(SessionEvent::Recap {
                    summary: String::new(),
                    auto: false,
                }),
            ));
        agent.pending_recap_entry = Some(spinner);
        assert!(!scrollback_has_user_messages(&agent.scrollback));
    }

    dispatch(
        Action::TaskComplete(TaskResult::RecapRequested {
            session_id,
            auto: false,
            error: Some("transport down".into()),
        }),
        &mut app,
    );

    let agent = app.agents.get(&id).unwrap();
    assert!(agent.pending_recap_entry.is_none());
    assert_eq!(
        agent.toast.as_ref().map(|(s, _)| s.as_str()),
        Some("No messages yet")
    );
}

#[test]
fn recap_request_transport_failure_with_turns_uses_generic_toast() {
    let mut app = test_app_with_agent();
    let id = AgentId(0);
    let session_id = app.agents[&id].session.session_id.clone().unwrap();
    {
        let agent = app.agents.get_mut(&id).unwrap();
        agent
            .scrollback
            .push_block(RenderBlock::user_prompt("hello"));
        let spinner = agent
            .scrollback
            .push(crate::scrollback::entry::ScrollbackEntry::running(
                RenderBlock::session_event(SessionEvent::Recap {
                    summary: String::new(),
                    auto: false,
                }),
            ));
        agent.pending_recap_entry = Some(spinner);
        assert!(scrollback_has_user_messages(&agent.scrollback));
    }

    dispatch(
        Action::TaskComplete(TaskResult::RecapRequested {
            session_id,
            auto: false,
            error: Some("transport down".into()),
        }),
        &mut app,
    );

    let agent = app.agents.get(&id).unwrap();
    assert!(agent.pending_recap_entry.is_none());
    assert_eq!(
        agent.toast.as_ref().map(|(s, _)| s.as_str()),
        Some("Couldn't generate recap")
    );
}

// ── OPS-02 / D-15 quiet-fork /feedback short-circuit ────────────────────────

fn scrollback_system_texts(agent: &AgentView) -> Vec<String> {
    agent
        .scrollback
        .iter_entries()
        .filter_map(|(_, e)| match &e.block {
            RenderBlock::System(s) => Some(s.text.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn p8_feedback_enter_shows_disabled_message() {
    let mut app = test_app_with_agent();
    let id = AgentId(0);

    let effects = dispatch(Action::EnterFeedbackMode, &mut app);

    assert!(
        effects.is_empty(),
        "enter must not emit effects: {effects:?}"
    );
    let texts = scrollback_system_texts(app.agents.get(&id).unwrap());
    assert!(
        texts.iter().any(|t| t.contains("Feedback is disabled in bum")),
        "expected disabled message in scrollback, got {texts:?}"
    );
    assert!(
        texts
            .iter()
            .any(|t| t.contains("no phone-home") || t.contains("Local logs under ~/.bum")),
        "expected phone-home or local-logs hint, got {texts:?}"
    );
    assert!(
        !texts
            .iter()
            .any(|t| t.contains("Grok Build team") || t.contains("Thanks for the feedback")),
        "stock team thank-you must not appear: {texts:?}"
    );
}

#[test]
fn p8_feedback_enter_returns_no_effects() {
    let mut app = test_app_with_agent();
    let effects = dispatch(Action::EnterFeedbackMode, &mut app);
    assert!(
        effects.is_empty(),
        "EnterFeedbackMode must return empty Effect vec, got {effects:?}"
    );
    assert!(
        !effects
            .iter()
            .any(|e| matches!(e, Effect::SendFeedback { .. })),
        "must never emit SendFeedback: {effects:?}"
    );
}

#[test]
fn p8_feedback_enter_stays_normal_mode() {
    use crate::app::agent_view::PromptInputMode;

    let mut app = test_app_with_agent();
    let id = AgentId(0);
    {
        let agent = app.agents.get_mut(&id).unwrap();
        agent.prompt_input_mode = PromptInputMode::Normal;
    }

    dispatch(Action::EnterFeedbackMode, &mut app);

    let agent = app.agents.get(&id).unwrap();
    assert_eq!(
        agent.prompt_input_mode,
        PromptInputMode::Normal,
        "must not leave Normal for Feedback collect mode"
    );
    assert_ne!(agent.prompt_input_mode, PromptInputMode::Feedback);
}

#[test]
fn p8_feedback_send_returns_no_send_effect() {
    let mut app = test_app_with_agent();
    let id = AgentId(0);

    let effects = dispatch(
        Action::SendFeedback("great product, love the fork".into()),
        &mut app,
    );

    assert!(
        effects.is_empty(),
        "send path must return empty effects: {effects:?}"
    );
    assert!(
        !effects
            .iter()
            .any(|e| matches!(e, Effect::SendFeedback { .. })),
        "must never emit Effect::SendFeedback: {effects:?}"
    );

    let texts = scrollback_system_texts(app.agents.get(&id).unwrap());
    assert!(
        texts.iter().any(|t| t.contains("Feedback is disabled in bum")),
        "send path should show disabled message: {texts:?}"
    );
    assert!(
        !texts
            .iter()
            .any(|t| t.contains("Grok Build team") || t.contains("Thanks for the feedback")),
        "stock thank-you must not appear: {texts:?}"
    );
    assert_eq!(
        app.agents.get(&id).unwrap().prompt_input_mode,
        crate::app::agent_view::PromptInputMode::Normal
    );
}
