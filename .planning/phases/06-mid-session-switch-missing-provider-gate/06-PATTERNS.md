# Phase 6: Mid-session switch & missing-provider gate - Pattern Map

**Mapped:** 2026-07-17
**Files analyzed:** 15
**Analogs found:** 15 / 15

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `crates/codegen/xai-grok-shell/src/agent/handlers/model_switch.rs` | service / handler | request-response | same file (IncompatibleAgent gate block) | exact |
| `crates/codegen/xai-grok-shell/src/agent/config.rs` | model / error type | transform | `ModelSwitchIncompatibleAgentError` in same file | exact |
| `crates/codegen/xai-grok-shell/src/auth/status.rs` (reuse / thin helper) | utility | transform | `store_usable` / `credential_usable` in same file | exact |
| `crates/codegen/xai-grok-shell/tests/` (missing-provider / gate) | test | request-response | `tests/provider_routing.rs` + `tests/auth_codex_lifecycle.rs` | role-match |
| `crates/codegen/xai-grok-pager/src/app/actions.rs` | model (enum) | event-driven | `SwitchModelError` + `AgentTypeMismatchAnswered` in same file | exact |
| `crates/codegen/xai-grok-pager/src/app/effects/mod.rs` | middleware / effect | request-response | `Effect::SwitchModel` error map in same file | exact |
| `crates/codegen/xai-grok-pager/src/app/dispatch/session/lifecycle.rs` | controller | event-driven | `handle_switch_model_complete` + `open_agent_type_mismatch_question` | exact |
| `crates/codegen/xai-grok-pager/src/views/question_view.rs` | component | event-driven | `LocalQuestionKind::AgentTypeMismatch` | exact |
| `crates/codegen/xai-grok-pager/src/app/agent_view/mod.rs` | component / controller | event-driven | `translate_local_submit` AgentTypeMismatch arm | exact |
| `crates/codegen/xai-grok-pager/src/app/dispatch/router.rs` | route | event-driven | `Action::AgentTypeMismatchAnswered` / `Action::SwitchModel` arms | exact |
| `crates/codegen/xai-grok-pager/src/app/dispatch/auth.rs` | controller | event-driven | `handle_auth_complete` reauth_stashed_prompt retry | role-match |
| `crates/codegen/xai-grok-pager/src/app/dispatch/settings/setters.rs` | controller | request-response | `set_default_model` optimistic `set_current` + rollback via prev | exact |
| `crates/codegen/xai-grok-pager/src/slash/commands/model.rs` | component | transform | `build_model_items` `(current)` suffix | exact |
| `crates/codegen/xai-grok-pager/src/app/app_view.rs` (optional usable cache) | store | request-response | `apply_auth_meta` / auth lifecycle fields on AppView | role-match |
| `crates/codegen/xai-grok-pager/src/app/dispatch/tests/task_result.rs` | test | event-driven | IncompatibleAgent QuestionView suite | exact |

## Pattern Assignments

### `crates/codegen/xai-grok-shell/src/agent/handlers/model_switch.rs` (service, request-response)

**Analog:** same file — agent-type mismatch early-return block (lines 65–88)

**Imports pattern** (lines 1–11):
```rust
//! Applies a model switch to a session — the ungated path. `set_session_model`
//! enforces the `allowed_models` gate before delegating here; internal callers
//! (`new_session`, `load_session`) call `apply` directly.
use crate::agent::config;
use crate::agent::mvp_agent::{
    MvpAgent, agent_name_after_model_switch, harnesses_are_compatible, resolve_required_agent_type,
};
use crate::session::SessionCommand;
use agent_client_protocol::{self as acp};
use tokio::sync::oneshot;
use xai_grok_sampling_types::parse_reasoning_effort_meta;
```

**Core gate pattern** — insert missing-provider check **after** `resolve_model_id` (line 35), **before** agent-type mismatch and `prepare_prepared_sampling_config_for_model` (lines 65–116). Mirror the IncompatibleAgent early return:

```rust
// Analog: lines 65-88 — typed reject + telemetry + into_acp_error
if is_mismatch && turn_count > 0 {
    xai_grok_telemetry::session_ctx::log_event(xai_grok_telemetry::events::ModelSwitched {
        session_id: session_id.0.to_string(),
        previous_model_id: previous_model_id.to_string(),
        new_model_id: model_id.0.to_string(),
        success: false,
        error_code: Some(config::MODEL_SWITCH_INCOMPATIBLE_AGENT.to_string()),
        required_agent_type: Some(required.clone()),
        current_agent_type: active_agent_type.clone(),
    });
    let err_payload = config::ModelSwitchIncompatibleAgentError {
        code: config::MODEL_SWITCH_INCOMPATIBLE_AGENT.to_string(),
        active_agent_type: active_agent_type.unwrap_or_else(|| "unknown".to_owned()),
        required_agent_type: required.clone(),
        model_id: model_id.0.to_string(),
        suggestion: "start_new_session".to_string(),
    };
    return Err(err_payload.into_acp_error());
}
```

**Prescribed missing-provider gate** (from RESEARCH, copy structure of above):
```rust
// After: let model = agent.resolve_model_id(&model_id)?;
// Skip OAuth-slot gate when model.has_own_credentials() (BYOK)
// Use catalog ModelEntry.info.provider (ModelProvider), never client-supplied provider
if !model.has_own_credentials() && !provider_slot_usable(agent, model.info.provider) {
    let provider_id = model.info.provider.as_str(); // "xai" | "codex"
    xai_grok_telemetry::session_ctx::log_event(/* ModelSwitched success:false,
        error_code: Some(MODEL_SWITCH_MISSING_PROVIDER) */);
    let err = config::ModelSwitchMissingProviderError {
        code: config::MODEL_SWITCH_MISSING_PROVIDER.to_string(),
        provider: provider_id.to_owned(),
        model_id: model_id.0.to_string(),
        suggestion: format!("bum login --provider {provider_id}"),
    };
    return Err(err.into_acp_error());
}
// then existing agent-type mismatch checks…
```

**Public entry already funnels here** — `MvpAgent::set_session_model` (acp_agent.rs ~3118–3143):
```rust
async fn set_session_model(
    &self,
    args: acp::SetSessionModelRequest,
) -> Result<acp::SetSessionModelResponse, acp::Error> {
    let model = self.resolve_model_id(&args.model_id)?;
    if !model.info.user_selectable {
        return Err(
            acp::Error::invalid_params()
                .data("This model isn't allowed by your allowed_models setting."),
        );
    }
    let session_id = args.session_id.clone();
    let res = crate::agent::handlers::model_switch::apply(self, args).await;
    // …
    res
}
```

**Error handling:** return `acp::Error` via typed payload `.into_acp_error()`; log `ModelSwitched { success: false, error_code }`. Do **not** call prepare/SetSessionModel/broadcast on gate fail.

**BYOK short-circuit analog** (`config.rs` lines 4046–4051):
```rust
/// `true` when the model has a non-empty `api_key` or an `env_key` that
/// resolves to a non-empty value.
pub fn has_own_credentials(&self) -> bool {
    self.own_credential().is_some()
}
```

---

### `crates/codegen/xai-grok-shell/src/agent/config.rs` (model / error type, transform)

**Analog:** `ModelSwitchIncompatibleAgentError` (lines 5465–5517)

**Typed ACP error pattern** (copy structure exactly):
```rust
/// Error code for model switch rejection due to agent type mismatch.
pub const MODEL_SWITCH_INCOMPATIBLE_AGENT: &str = "MODEL_SWITCH_INCOMPATIBLE_AGENT";
/// Error code for model switch failure during the zero-turn full harness
/// rebuild path.
pub const MODEL_SWITCH_REBUILD_FAILED: &str = "MODEL_SWITCH_REBUILD_FAILED";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ModelSwitchIncompatibleAgentError {
    pub code: String,
    pub active_agent_type: String,
    pub required_agent_type: String,
    pub model_id: String,
    pub suggestion: String,
}

impl ModelSwitchIncompatibleAgentError {
    pub fn into_acp_error(self) -> acp::Error {
        let message = format!(
            "Cannot switch to model '{}': it requires agent '{}' but the active agent is '{}'. \
             Start a new session to use this model.",
            self.model_id, self.required_agent_type, self.active_agent_type,
        );
        acp::Error::new(acp::ErrorCode::InvalidRequest.into(), message)
            .data(serde_json::to_value(&self).ok())
    }
    pub fn from_acp_error(err: &acp::Error) -> Option<Self> {
        let data = err.data.as_ref()?;
        let code = data.get("code")?.as_str()?;
        if code != MODEL_SWITCH_INCOMPATIBLE_AGENT {
            return None;
        }
        serde_json::from_value(data.clone()).ok()
    }
    pub fn user_message(&self) -> String { /* TUI-friendly */ }
}
```

**Phase 6 twin** (UI-SPEC contract — mirror field style):
```rust
pub const MODEL_SWITCH_MISSING_PROVIDER: &str = "MODEL_SWITCH_MISSING_PROVIDER";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ModelSwitchMissingProviderError {
    pub code: String,       // always MODEL_SWITCH_MISSING_PROVIDER
    pub provider: String,   // "xai" | "codex"
    pub model_id: String,
    pub suggestion: String, // "bum login --provider {id}"
}
// into_acp_error message (UI-SPEC):
// "Cannot switch to model '{modelId}': no usable {ProviderLabel} credentials. Run: bum login --provider {id}"
// from_acp_error: match code == MODEL_SWITCH_MISSING_PROVIDER
```

**Provider labels** (lines 3402–3417) — reuse, do not invent:
```rust
impl ModelProvider {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Xai => "xai",
            Self::Codex => "codex",
        }
    }
    pub fn display_label(self) -> &'static str {
        match self {
            Self::Xai => "xAI",
            Self::Codex => "Codex",
        }
    }
}
```

**Catalog provider stamp for client badges** (`to_acp_model_info`, lines 5427–5432):
```rust
// Provider binding for multi-provider catalog (D-10 machine surface).
// Copied only from trusted in-process ModelInfo.provider — never client input.
map.insert(
    "provider".to_string(),
    serde_json::Value::String(info.provider.as_str().to_owned()),
);
```

---

### `crates/codegen/xai-grok-shell/src/auth/status.rs` (utility, transform)

**Analog:** pure usable helpers in same file (lines 113–147)

**Usable credentials pattern** (gate must reuse these semantics — refreshable = usable):
```rust
/// - `true` when access is hard-unexpired (`Duration::zero()` buffer), **or**
/// - `true` when a nonblank refresh_token is present (refreshable OAuth)
/// - `false` when access is hard-expired and no refresh_token
pub fn credential_usable(auth: &GrokAuth) -> bool {
    if auth.key.trim().is_empty() && auth.refresh_token.as_ref().is_none_or(|t| t.trim().is_empty()) {
        return false;
    }
    let hard_unexpired =
        !auth.key.trim().is_empty() && !is_expired_with_buffer(auth, Duration::zero());
    if hard_unexpired {
        return true;
    }
    auth.refresh_token
        .as_ref()
        .is_some_and(|t| !t.trim().is_empty())
}

pub fn store_usable(store: &AuthStore) -> bool {
    store
        .values()
        .filter(|a| a.auth_mode != AuthMode::WebLogin)
        .any(credential_usable)
}
```

**AuthProvider labels** (`auth/model.rs` lines 35–48):
```rust
impl AuthProvider {
    pub fn as_str(self) -> &'static str { /* "xai" | "codex" */ }
    pub fn label(self) -> &'static str {
        match self {
            Self::Xai => "xAI",
            Self::Codex => "Codex",
        }
    }
}
```

**Disk slot read** (`auth/storage.rs` lines 207–211):
```rust
pub fn read_provider_auth_store(
    path: &Path,
    provider: &str,
) -> Result<Option<AuthStore>, AuthStoreReadError> {
    match read_auth_document(path) {
        Ok(doc) => Ok(doc.providers.get(provider).cloned()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        // …
    }
}
```

**Anti-pattern:** do **not** use `AuthManager::has_usable_token` alone for Codex — it is xAI-scoped. Resolve `ModelProvider` → slot via store/`AuthStatusReport`/`store_usable`. For xAI, live AuthManager + disk may both count if matching status semantics.

---

### `crates/codegen/xai-grok-pager/src/app/actions.rs` (model enum, event-driven)

**Analog:** `SwitchModelError` + `AgentTypeMismatchAnswered` (lines 16–30, 721–730)

**Error enum pattern:**
```rust
#[derive(Debug, Clone)]
pub enum SwitchModelError {
    IncompatibleAgent {
        error: xai_grok_shell::agent::config::ModelSwitchIncompatibleAgentError,
        prev_model_id: Option<acp::ModelId>,
    },
    /// Any other failure (network, auth, server error, etc.).
    Other(String),
}
```

**Add sibling variant** (do not collapse into IncompatibleAgent or Other):
```rust
MissingProvider {
    error: xai_grok_shell::agent::config::ModelSwitchMissingProviderError,
    prev_model_id: Option<acp::ModelId>,
},
```

**Answer Action pattern** (mirror lines 721–730):
```rust
AgentTypeMismatchAnswered {
    start_new: bool,
    model_id: acp::ModelId,
    effort: Option<ReasoningEffort>,
},
// New:
MissingProviderLoginAnswered {
    login: bool, // true = Login now, false = Keep current model
    model_id: acp::ModelId,
    effort: Option<ReasoningEffort>,
    provider: String, // "xai" | "codex"
},
```

---

### `crates/codegen/xai-grok-pager/src/app/effects/mod.rs` (middleware, request-response)

**Analog:** `Effect::SwitchModel` map (lines 1601–1651)

**ACP send + typed deserialize pattern:**
```rust
Effect::SwitchModel {
    agent_id,
    session_id,
    model_id,
    effort,
    prev_model_id,
} => {
    let tx = acp_tx.clone();
    tasks.spawn(async move {
        let meta = effort.map(|eff| { /* REASONING_EFFORT_META_KEY */ });
        let req = acp::SetSessionModelRequest::new(session_id, model_id.clone()).meta(meta);
        let result = acp_send(req, &tx)
            .await
            .map(|_| ())
            .map_err(|e| {
                use xai_grok_shell::agent::config::ModelSwitchIncompatibleAgentError;
                if let Some(typed) = ModelSwitchIncompatibleAgentError::from_acp_error(&e) {
                    SwitchModelError::IncompatibleAgent {
                        error: typed,
                        prev_model_id: prev_model_id.clone(),
                    }
                } else {
                    SwitchModelError::Other(sanitize_user_error(&e.to_string()))
                }
            });
        TaskResult::SwitchModelComplete {
            agent_id, model_id, effort, result, prev_model_id,
        }
    });
}
```

**Extend match order** (MissingProvider before IncompatibleAgent before Other):
```rust
.map_err(|e| {
    use xai_grok_shell::agent::config::{
        ModelSwitchIncompatibleAgentError, ModelSwitchMissingProviderError,
    };
    if let Some(typed) = ModelSwitchMissingProviderError::from_acp_error(&e) {
        SwitchModelError::MissingProvider {
            error: typed,
            prev_model_id: prev_model_id.clone(),
        }
    } else if let Some(typed) = ModelSwitchIncompatibleAgentError::from_acp_error(&e) {
        SwitchModelError::IncompatibleAgent { /* existing */ }
    } else {
        SwitchModelError::Other(sanitize_user_error(&e.to_string()))
    }
})
```

Always keep `sanitize_user_error` on untyped paths — never surface tokens.

---

### `crates/codegen/xai-grok-pager/src/app/dispatch/session/lifecycle.rs` (controller, event-driven)

**Analog A:** `open_agent_type_mismatch_question` (lines 205–259)  
**Analog B:** `handle_switch_model_complete` (lines 1036–1098)  
**Analog C:** `deferred_model_switch` take/apply (lines 27–111)

**QuestionView open pattern** (copy builder + no freeform):
```rust
pub(in crate::app::dispatch) fn open_agent_type_mismatch_question(
    app: &mut AppView,
    model_id: acp::ModelId,
    effort: Option<ReasoningEffort>,
    model_name: &str,
) -> Vec<Effect> {
    use crate::views::question_view::{LocalQuestionKind, QuestionViewState};
    use xai_grok_tools::implementations::grok_build::ask_user_question::{
        Question, QuestionOption,
    };
    // guard: ActiveView::Agent, agent present, question_view not already open
    let question = Question {
        question: format!("Switching to {model_name} requires starting a new session. Continue?"),
        id: None,
        options: vec![
            QuestionOption {
                label: "Yes".into(),
                description: format!("Start a new session with {model_name}"),
                preview: None,
                id: None,
            },
            QuestionOption {
                label: "No".into(),
                description: "Continue the current session".into(),
                preview: None,
                id: None,
            },
        ],
        multi_select: Some(false),
    };
    let stashed = agent.prompt.stash();
    let state = QuestionViewState::new(
        format!("agent-type-mismatch-{}", uuid::Uuid::new_v4()),
        vec![question],
        stashed,
    )
    .with_local_kind(LocalQuestionKind::AgentTypeMismatch { model_id, effort })
    .with_no_freeform();
    agent.question_view = Some(state);
    agent.prompt.set_text("");
    vec![]
}
```

**Phase 6 modal copy** (06-UI-SPEC — replace question/options only):
```text
Question: Sign in to {ProviderLabel} to use {ModelDisplayName}.
Option 1: Login now — Run login for {ProviderLabel} (CLI: bum login --provider {id})
Option 2: Keep current model — Dismiss and stay on the active model
LocalQuestionKind::MissingProviderLogin { model_id, effort, provider }
```

**Complete handler pattern** (lines 1036–1098) — always clear `model_switch_pending` first:
```rust
pub(in crate::app::dispatch) fn handle_switch_model_complete(...) -> Vec<Effect> {
    if let Some(agent) = app.agents.get_mut(&agent_id) {
        agent.session.model_switch_pending = false;
        let mut effects = match result {
            Ok(()) => {
                // set_current only on success
                agent.session.models.set_current(model_id.clone(), effort);
                // scrollback: "Switched to {display_name}" / with effort
                // optional Effect::PersistPreferredModel
            }
            Err(SwitchModelError::IncompatibleAgent { .. }) => {
                if let Some(ref prev) = prev_model_id {
                    agent.session.models.set_current(prev.clone(), None);
                }
                agent.active_modal = None;
                let display_name = agent.session.models.display_name_for(&model_id);
                return open_agent_type_mismatch_question(app, model_id, effort, &display_name);
            }
            Err(SwitchModelError::Other(msg)) => {
                agent.scrollback.push_block(RenderBlock::system(
                    format!("Couldn't switch model: {msg}"),
                ));
                vec![]
            }
        };
        effects.extend(maybe_drain_queue(agent));
        effects
    } else {
        vec![]
    }
}
```

**MissingProvider arm rules:**
1. Always restore `prev_model_id` if Some (settings optimistic path).
2. **Never** call `set_current` for the blocked target (no optimistic flash).
3. Open MissingProvider QuestionView (not IncompatibleAgent; not Other scrollback-only).
4. Clear `model_switch_pending` (already done at top).

**Deferred switch pattern** (reuse for Login now):
```rust
// AgentSession.deferred_model_switch: Option<(ModelId, Option<ReasoningEffort>)>
// take_deferred_model_switch / apply_deferred_model_switch already resolve effort tokens
// On Login now: agent.session.deferred_model_switch = Some((model_id, effort));
// On AuthComplete mid-session: apply deferred → re-issue Effect::SwitchModel with model_switch_pending
```

**Success answer dispatcher analog** (`dispatch_agent_type_mismatch_answered`, lines 1100–1119):
```rust
pub(in crate::app::dispatch) fn dispatch_agent_type_mismatch_answered(
    app: &mut AppView,
    start_new: bool,
    model_id: acp::ModelId,
    effort: Option<ReasoningEffort>,
) -> Vec<Effect> {
    if start_new {
        let effects = dispatch_new_session_inner(app, Some(model_id.clone()));
        // stash deferred_model_switch on new agent when effort present
        effects
    } else {
        vec![]
    }
}
// MissingProvider: login=true → deferred + provider login effect/path;
// login=false → clear deferred; stay on current; no model apply
```

---

### `crates/codegen/xai-grok-pager/src/views/question_view.rs` (component, event-driven)

**Analog:** `LocalQuestionKind::AgentTypeMismatch` (lines 123–130)

```rust
/// Modal shown when the shell rejects a model switch due to agent
/// type incompatibility. Carries the target model + effort so the
/// answer handler can create a new session with it.
AgentTypeMismatch {
    model_id: agent_client_protocol::ModelId,
    effort: Option<xai_grok_shell::sampling::types::ReasoningEffort>,
},
// New:
MissingProviderLogin {
    model_id: agent_client_protocol::ModelId,
    effort: Option<xai_grok_shell::sampling::types::ReasoningEffort>,
    provider: String, // "xai" | "codex"
},
```

**Builders already exist:** `QuestionViewState::with_local_kind` + `with_no_freeform` (lines 274+).

---

### `crates/codegen/xai-grok-pager/src/app/agent_view/mod.rs` (component, event-driven)

**Analog:** `translate_local_submit` AgentTypeMismatch arm (lines 1643–1650)

```rust
LocalQuestionKind::AgentTypeMismatch { model_id, effort } => {
    let start_new = *idx == 0;
    InputOutcome::Action(Action::AgentTypeMismatchAnswered {
        start_new,
        model_id: model_id.clone(),
        effort,
    })
}
// New:
LocalQuestionKind::MissingProviderLogin { model_id, effort, provider } => {
    let login = *idx == 0; // 0 = Login now, 1 = Keep current model
    InputOutcome::Action(Action::MissingProviderLoginAnswered {
        login,
        model_id: model_id.clone(),
        effort,
        provider: provider.clone(),
    })
}
```

Exhaustiveness: every `LocalQuestionKind` match site must handle the new variant (compiler will force).

---

### `crates/codegen/xai-grok-pager/src/app/dispatch/router.rs` (route, event-driven)

**Analog:** SwitchModel + AgentTypeMismatchAnswered (lines 798–817, 1091–1095)

```rust
Action::SwitchModel { model_id, effort } => {
    // no optimistic set_current on this path
    let Some(session_id) = agent.session.session_id.clone() else {
        agent.session.deferred_model_switch = Some((model_id, effort));
        return vec![];
    };
    agent.session.model_switch_pending = true;
    vec![Effect::SwitchModel {
        agent_id: id,
        session_id,
        model_id,
        effort,
        prev_model_id: None, // SwitchModel path does not optimistically mutate current
    }]
}
Action::AgentTypeMismatchAnswered { start_new, model_id, effort } => {
    dispatch_agent_type_mismatch_answered(app, start_new, model_id, effort)
}
// New: Action::MissingProviderLoginAnswered { … } => dispatch_missing_provider_login_answered(…)
```

**TEA purity:** keep SwitchModel dispatch free of auth I/O; gate remains shell-side.

---

### `crates/codegen/xai-grok-pager/src/app/dispatch/auth.rs` (controller, event-driven)

**Analog:** `handle_auth_complete` mid-session return + `reauth_stashed_prompt` retry (lines 270–332)

```rust
if let Some(return_view) = app.auth_return_view.take() {
    restore_auth_return_view(app, return_view);
    clear_startup_actions(app);
    let mut retry_effects = Vec::new();
    for agent in app.agents.values_mut() {
        strip_trailing_auth_error_blocks(agent);
        if let Some(prompt) = agent.reauth_stashed_prompt.take() {
            agent.scrollback.push_block(RenderBlock::system(
                "Re-authenticated. Retrying\u{2026}".to_string(),
            ));
            agent.session.enqueue_in_flight_prompt_front(prompt);
            retry_effects.extend(maybe_drain_queue(agent));
        }
    }
    // …
}
```

**Phase 6 extension:** on mid-session AuthComplete, also apply `deferred_model_switch` (same family as session-created apply in lifecycle ~829/919):
```rust
// For each agent with deferred_model_switch after successful provider login:
//   take deferred → set model_switch_pending → push Effect::SwitchModel
// Mirror apply_deferred_model_switch from lifecycle.rs
// Prefer provider-aware: only retry if the missing provider is now usable
```

**Login now depth:** Phase 5 Codex is primarily CLI; always show `bum login --provider {id}` in option description. TUI `Action::Login` is xAI interactive today — do not claim Codex browser TUI if unbuilt; CLI + poll/deferred retry is acceptable.

---

### `crates/codegen/xai-grok-pager/src/app/dispatch/settings/setters.rs` (controller, request-response)

**Analog:** `set_default_model` optimistic current (lines 1443–1575)

```rust
// set_default_model_inner optimistically:
agent.session.models.set_current(id.clone(), None);
// then emits Effect::SwitchModel with prev_model_id for rollback
effects.push(Effect::SwitchModel {
    agent_id: aid,
    session_id: sid,
    model_id: new_id,
    effort: None,
    prev_model_id: prev_id.clone(),
});
```

**Phase 6 pitfall fix:** On `SwitchModelError::MissingProvider`, always restore `prev_model_id` in complete handler (stricter than IncompatibleAgent UX — UI-SPEC: **no optimistic current flash** for this error class). Preferred: either always rollback on MissingProvider, or stop optimistically setting current for session switch until Ok. Existing IncompatibleAgent rollback test is the template.

---

### `crates/codegen/xai-grok-pager/src/slash/commands/model.rs` (component, transform)

**Analog:** `build_model_items` (lines 153–182)

```rust
fn build_model_items(models: &ModelState) -> Vec<ArgItem> {
    let current_id = models.current.as_ref();
    let mut items: Vec<ArgItem> = Vec::with_capacity(models.available.len());
    for (id, info) in &models.available {
        let is_current = current_id == Some(id);
        let display = if is_current {
            format!("{} (current)", info.name)
        } else {
            info.name.clone()
        };
        items.push(ArgItem {
            display,
            match_text: info.name.clone(),
            insert_text: /* trailing space if reasoning */,
            description: info.description.clone().unwrap_or_default(),
        });
    }
    items
}
```

**`ArgItem` has no `right_label` today** (`slash/command.rs` lines 81–90). Prefer display suffix MVP:
```rust
// UI-SPEC badge: exact "needs login"; prefer right_label later
let display = if needs_login {
    if is_current {
        format!("{} (current) · needs login", info.name)
    } else {
        format!("{} · needs login", info.name)
    }
} else if is_current {
    format!("{} (current)", info.name)
} else {
    info.name.clone()
};
// needs_login from AppView dual-slot usable cache + meta.provider ("xai"|"codex")
// Do NOT filter rows; do NOT replace description with badge
```

**Purity:** do not open `auth.json` inside `suggest_args`. Cache usable flags on AppView; refresh on AuthComplete / logout / startup.

---

### Shell / pager tests (test, request-response / event-driven)

**Shell MOD-03 route analog:** `crates/codegen/xai-grok-shell/tests/provider_routing.rs` `switch_changes_next_sample_route` (lines 995–1068) — dual fake tokens, assert base_url + credential slot change.

**Shell usable pure tests:** `auth/status.rs` `#[cfg(test)]` + `tests/auth_codex_lifecycle.rs` (`credential_usable`, `AuthStatusReport`).

**Pager IncompatibleAgent suite template:** `dispatch/tests/task_result.rs` (lines 810–911):

```rust
#[test]
fn /* name */() {
    let mut app = test_app_with_agent();
    let id = AgentId(0);
    // model_switch_pending = true
    let err = xai_grok_shell::agent::config::ModelSwitchIncompatibleAgentError { /* … */ };
    let effects = dispatch(
        Action::TaskComplete(TaskResult::SwitchModelComplete {
            agent_id: id,
            model_id,
            effort: None,
            result: Err(SwitchModelError::IncompatibleAgent {
                error: err,
                prev_model_id: None,
            }),
            prev_model_id: None,
        }),
        &mut app,
    );
    assert!(effects.is_empty());
    assert!(!app.agents[&id].session.model_switch_pending);
    assert!(app.agents[&id].question_view.is_some());
    assert!(matches!(
        qv.local_kind,
        Some(LocalQuestionKind::AgentTypeMismatch { .. })
    ));
}
// Mirror for MissingProvider → LocalQuestionKind::MissingProviderLogin
// Mirror incompatible_agent_rollback_restores_previous_model for MissingProvider
```

**Wave 0 gaps to cover** (from RESEARCH): pure gate decision table; serde round-trip; apply returns missing-provider; QuestionView open; deferred after AuthComplete; badge; BYOK skip; IncompatibleAgent suite still green.

---

## Shared Patterns

### Single-authority switch gate (shell)

**Source:** `model_switch::apply` + `MvpAgent::set_session_model`  
**Apply to:** all entry points (picker, `/model`, settings live switch, ACP `session/set_model`)

- Gate in shell after `resolve_model_id`, before prepare/SetSessionModel.
- Never pager-only pre-check as sole authority.
- Fail closed: typed ACP error, no ModelChanged broadcast, no sampling config mutate.

### Typed ACP model-switch errors

**Source:** `config.rs` `ModelSwitchIncompatibleAgentError`  
**Apply to:** new `ModelSwitchMissingProviderError` + pager `SwitchModelError` + effects map

- Stable `code` string constant  
- `#[serde(rename_all = "camelCase")]` payload in `acp::Error.data`  
- `into_acp_error` / `from_acp_error` pair  
- **Separate** codes: `MODEL_SWITCH_INCOMPATIBLE_AGENT` vs `MODEL_SWITCH_MISSING_PROVIDER`

### QuestionView recovery (TEA-local)

**Source:** `open_agent_type_mismatch_question` + `LocalQuestionKind` + `translate_local_submit`  
**Apply to:** MissingProvider Login now / Keep current model

- `QuestionViewState::with_local_kind(...).with_no_freeform()`  
- Option index 0 = primary CTA, 1 = dismiss  
- Maps to typed Action; router calls dispatch helper  
- Different kind / Action / dispatcher from IncompatibleAgent

### deferred_model_switch

**Source:** `AgentSession.deferred_model_switch` + lifecycle take/apply  
**Apply to:** post-login auto-retry of blocked target; also pre-session SwitchModel stash

```rust
// field: Option<(acp::ModelId, Option<ReasoningEffort>)>
// set on Login now; take on AuthComplete mid-session or session-created
// re-issue Effect::SwitchModel with model_switch_pending = true
```

### Usable credentials (Phase 5)

**Source:** `auth/status.rs` `credential_usable` / `store_usable`  
**Apply to:** shell gate + pager badge cache

- Expired-but-refreshable = usable  
- Hard-expired without refresh = not usable  
- BYOK (`has_own_credentials`) skips OAuth-slot gate  
- Codex must not use xAI-only `AuthManager::has_usable_token` alone

### TEA purity + success UX

**Source:** router SwitchModel (no optimistic current) + lifecycle success scrollback  
**Apply to:** all pager switch paths

- Dispatch free of network/FS  
- Success: `Switched to {ModelDisplayName}` / with effort; provider already in Phase 3 name  
- Mid-turn switch allowed; next-turn only; do not cancel in-flight turn  
- Always clear `model_switch_pending` on complete  

### Secrets

**Source:** auth status paste-safe contract + `sanitize_user_error`  
**Apply to:** all error messages, suggestion strings, badges, logs

- Never print tokens, auth.json body, or Authorization headers  
- Suggestion is CLI only: `bum login --provider {id}`

### Provider wire ids / labels

| Wire | Label | Source |
|------|-------|--------|
| `xai` | `xAI` | `ModelProvider` / `AuthProvider` |
| `codex` | `Codex` | same |

Product CLI in new copy: always `bum`.

---

## No Analog Found

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| — | — | — | Full coverage: every Phase 6 touch reuses an in-tree twin (IncompatibleAgent path, status usable, deferred switch, badge suffix). Login-now **Codex in-TUI OAuth** depth is under-specified (CLI-primary is the safe analog). |

**Partial / discretion gaps (not “no analog”):**

| Concern | Closest analog | Gap |
|---------|----------------|-----|
| AppView dual-slot usable cache for badges | `apply_auth_meta` / auth lifecycle on AppView | No existing `provider_auth_usable: [bool; 2]` field — add lightweight snapshot |
| ArgItem `right_label` | modal `PickerRow` right_label (elsewhere) | Slash `ArgItem` lacks field — use ` · needs login` suffix MVP |
| Codex Login now from TUI | Phase 5 CLI `bum login --provider codex` + mid-session auth return | TUI Login is xAI-centric; CLI fallback always required |

---

## Metadata

**Analog search scope:**  
`crates/codegen/xai-grok-shell/src/agent/handlers/`, `agent/config.rs`, `agent/mvp_agent/`, `auth/`, `tests/`;  
`crates/codegen/xai-grok-pager/src/app/{actions,effects,dispatch,agent_view}/`, `views/question_view.rs`, `slash/commands/model.rs`

**Files scanned:** ~25 primary analogs (handlers, config error block, status, lifecycle, effects SwitchModel, actions enums, question_view kinds, agent_view translate, router, auth complete, settings setters, model slash, provider_routing + task_result tests)

**Pattern extraction date:** 2026-07-17
