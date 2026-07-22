# Phase 7: Cross-provider multi-agent orchestration - Pattern Map

**Mapped:** 2026-07-17
**Files analyzed:** 12
**Analogs found:** 12 / 12

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `crates/common/xai-tool-types/src/task.rs` | model (wire schema) | request-response | same file — `TaskToolInput.model` field + schemars | exact (extend) |
| `crates/codegen/xai-grok-tools/.../task/mod.rs` | service (tool) | request-response | same file — eager `validate_type` / `TaskModelValidator` + overrides wire | exact (extend) |
| `crates/codegen/xai-grok-tools/.../task/types.rs` | model | request-response | same file — `SubagentRuntimeOverrides` + `TaskModelValidator` resource | exact (minimal) |
| `crates/codegen/xai-grok-tools/.../task/backend.rs` | middleware / transport | request-response | same file — `validate_type` preflight RPC | role-match (preflight template) |
| `crates/codegen/xai-grok-agent/src/builder.rs` | utility (description) | transform | same file — `task_model_guidance` / `build_task_description` | exact (extend) |
| `crates/codegen/xai-grok-shell/.../subagent/handle_request.rs` | service (orchestrator) | request-response / event-driven | same file + Phase 6 `handlers/model_switch.rs` gate | exact + role-match |
| `crates/codegen/xai-grok-shell/.../subagent/mod.rs` | service / utility | transform | same file — `resolve_model_override_to_config` | exact (reuse; fail-closed tweak) |
| `crates/codegen/xai-grok-shell/src/agent/config.rs` | utility / policy | transform | same file — `missing_provider_gate_error` | exact (reuse) |
| `crates/codegen/xai-grok-shell/src/auth/status.rs` | utility | transform | same file — `provider_slot_usable` | exact (reuse) |
| `crates/codegen/xai-grok-tools` task unit tests | test | batch | `task/mod.rs` `#[cfg(test)]` model/schema suite | exact |
| `crates/codegen/xai-grok-shell` `p7_*` unit/integration | test | batch | `subagent/tests/mod.rs` + `model_switch_gate.rs` + `provider_routing.rs` | role-match |
| optional `tests/cross_provider_subagent.rs` | test | request-response | `tests/provider_routing.rs` + `tests/model_switch_gate.rs` | role-match |

## Pattern Assignments

### `crates/common/xai-tool-types/src/task.rs` (model / wire schema, request-response)

**Analog:** same file — optional model field pattern (lines 96–109)

**Imports / derive pattern** (lines 1–14):
```rust
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskToolInput {
```

**Optional catalog slug field** (lines 96–104) — copy shape for `reasoning_effort`:
```rust
/// Optional model slug for this subagent.
#[schemars(
    description = "Optional model slug for this agent. If provided, it must resolve to one \
        of the available model slugs. If omitted, the subagent uses the same model as the \
        parent agent. Do not pass if resume_from is set (prior model will be used). Only \
        choose an explicit model when the user directly requests it."
)]
#[serde(default, skip_serializing_if = "Option::is_none")]
pub model: Option<String>,
```

**Phase 7 delta:** Add sibling field after `model`:
```rust
/// Optional reasoning effort for this subagent (`low` / `medium` / `high` / `xhigh`).
#[schemars(
    description = "Optional reasoning effort for this subagent: \"low\", \"medium\", \
        \"high\", or \"xhigh\" (alias \"max\"). If omitted, the child uses the role/model \
        default. Invalid values are rejected. Prefer setting when the user asks for a \
        specific effort (e.g. medium-effort research)."
)]
#[serde(default, skip_serializing_if = "Option::is_none")]
pub reasoning_effort: Option<String>,
```

**Sanitize pattern** (lines 117–129): use `sanitize_optional_arg` for effort strings the same way Task sanitizes model.

**Shared description builder** (lines 842–878): `build_task_description` is naming-template based — effort guidance belongs in agent `builder.rs` appendix (like model guidance), not necessarily in this core paragraph unless schema docs alone are insufficient.

---

### `crates/codegen/xai-grok-tools/.../task/mod.rs` (tool service, request-response)

**Analog:** same file — eager validation before background fire-and-forget

**Imports pattern** (lines 19–27):
```rust
use self::types::*;
use crate::types::output::ToolOutput;
// ...
use xai_tool_types::{SubagentCompletedOutput, SubagentIsolationMode, TaskToolInput};
```

**Resource injection** (lines 120–136):
```rust
let model_validator = res.get::<TaskModelValidator>().cloned();
// Phase 7: similarly inject optional credential-gate resource if eager preflight uses Resources
```

**Resume soft-ignore model** (lines 170–182) — keep; apply same soft-ignore to effort only if product decides (CONTEXT: resume pins source model; effort on resume is discretion — prefer soft-ignore model, allow effort only if not conflicting with pin docs):
```rust
let model = xai_tool_types::sanitize_optional_arg(input.model);
let model = if resume_from.is_some() {
    if let Some(ref ignored) = model {
        tracing::debug!(model = %ignored, "ignoring model override because resume_from is set");
    }
    None
} else {
    model
};
```

**Eager validate_type before bg spawn** (lines 229–273) — **copy this timing** for effort parse + child-provider credential preflight:
```rust
// 2. Eager validation — catch unknown / disabled / not-allowed
//    types before the fire-and-forget background spawn.
match backend.backend().validate_type(&input.subagent_type, &parent_session_id).await {
    SubagentValidateTypeOutcome::Ok => {}
    SubagentValidateTypeOutcome::Unknown { available } => {
        return Err(xai_tool_runtime::ToolError::invalid_arguments(format!(
            "Unknown subagent type: {}{suffix}",
            input.subagent_type
        )));
    }
    // ... Disabled / NotAllowed / ValidationUnavailable
}
```

**Eager model validation** (lines 275–285) — template for credential preflight resource:
```rust
if let Some(ref requested) = model {
    let validator = model_validator.ok_or_else(|| {
        xai_tool_runtime::ToolError::custom(
            "validation_unavailable",
            "Cannot validate Task.model: model catalog validator is unavailable.",
        )
    })?;
    if let Some(error) = validator.error_for(requested) {
        return Err(xai_tool_runtime::ToolError::invalid_arguments(error));
    }
}
```

**Core override wire (MUST CHANGE)** (lines 305–316):
```rust
runtime_overrides: SubagentRuntimeOverrides {
    model,
    model_override_provenance: ModelOverrideProvenance::Tool,
    reasoning_effort: None, // ← AGENT-03: wire from TaskToolInput
    persona: None,
    capability_mode: input.capability_mode,
    isolation: input.isolation,
    harness_agent_type: None,
},
```

**Background fire-and-forget** (lines 328–348) — do **not** rely on this path for AGENT-05; preflight must return `Err` before spawn:
```rust
if input.run_in_background {
    tokio::spawn(async move {
        match bg_backend.backend().spawn(request).await {
            Ok(r) if !r.success => { tracing::error!(/* late rejection */); }
            // ...
        }
    });
}
```

**Test pattern to extend** — schema + thread assertions (lines 1169–1178, 2277–2294):
```rust
#[test]
fn task_tool_input_schema_includes_model() {
    let schema = serde_json::to_value(schemars::schema_for!(TaskToolInput)).unwrap();
    assert_eq!(schema["properties"]["model"]["description"], /* ... */);
}

#[tokio::test]
async fn model_threads_to_runtime_overrides() {
    // assert runtime_overrides.model + provenance Tool
    // Phase 7: assert reasoning_effort: Some("medium") threads the same way
    assert!(request.runtime_overrides.reasoning_effort.is_none()); // today — flip for wired path
}
```

**Invalid-arg error surface:**
```rust
xai_tool_runtime::ToolError::invalid_arguments(format!(/* accepted tokens list */))
```

---

### `crates/codegen/xai-grok-tools/.../task/types.rs` (model, request-response)

**Analog:** same file — overrides + resource registration

**Overrides already carry effort** (lines 76–91):
```rust
pub enum ModelOverrideProvenance {
    #[default]
    Harness,
    Tool,
}

pub struct SubagentRuntimeOverrides {
    pub model: Option<String>,
    pub model_override_provenance: ModelOverrideProvenance,
    /// Override reasoning effort (e.g. "low", "medium", "high").
    pub reasoning_effort: Option<String>,
    // ...
}
```

**TaskModelValidator resource** (lines 768–793) — **copy for spawn credential gate resource**:
```rust
type TaskModelValidationFn = dyn Fn(&str) -> Option<String> + Send + Sync;

#[derive(Clone)]
pub struct TaskModelValidator(Arc<TaskModelValidationFn>);

impl TaskModelValidator {
    pub fn new(validate: impl Fn(&str) -> Option<String> + Send + Sync + 'static) -> Self {
        Self(Arc::new(validate))
    }
    pub fn error_for(&self, model: &str) -> Option<String> {
        (self.0)(model)
    }
}

register_resource!("grok_build", "TaskModelValidator", TaskModelValidator);
```

**Phase 7 pattern:** Add `TaskProviderCredentialGate` (or equivalent) with same `Fn(&str) -> Option<String>` shape: `None` = usable / OK to spawn; `Some(msg)` = fail closed with login hint. Inject from shell when building session Resources (parallel to TaskModelValidator wiring).

---

### `crates/codegen/xai-grok-tools/.../task/backend.rs` (transport preflight, request-response)

**Analog:** same file — `SubagentBackend::validate_type` (lines 56–60)

```rust
/// Validate a subagent type synchronously before spawning.
/// Returns `ValidationUnavailable` on channel close / responder drop / timeout.
async fn validate_type(
    &self,
    subagent_type: &str,
    parent_session_id: &str,
) -> SubagentValidateTypeOutcome;
```

**When to extend backend vs Resources:** Prefer Resources pure gate (like model validator) so tools avoid auth.json I/O. Use backend RPC only if preflight needs coordinator state. RESEARCH recommendation: thin `Fn(model_slug) -> Option<String>` resource.

---

### `crates/codegen/xai-grok-agent/src/builder.rs` (utility, transform)

**Analog:** same file — model slug appendix on Task description

**Naming constants** (lines 1198–1206):
```rust
const TASK_TOOL_NAMING: xai_tool_types::TaskToolNaming<'static> = xai_tool_types::TaskToolNaming {
    task_tool: "${{ tools.by_kind.task }}",
    subagent_type_param: "${{ params.task.subagent_type }}",
    // ...
};
```

**Model guidance pattern** (lines 1250–1270) — mirror for effort:
```rust
const TASK_MODEL_PARAM: &str = "${{ params.task.model }}";
fn task_model_guidance(model_slugs: &[String]) -> String {
    // lists catalog slugs; omit param to inherit parent
    format!(
        "\n\nIf the user explicitly asks for the model of a subagent/task, you may ONLY use model slugs from this list:\n\
         {model_list}\n\n\
         If the user does not explicitly request a model, omit `{TASK_MODEL_PARAM}` to inherit the parent model."
    )
}
```

**Compose description** (lines 1280–1300):
```rust
pub(crate) fn build_task_description(
    subagents: &[SubagentEntry],
    model_slugs: &[String],
) -> String {
    let mut description = xai_tool_types::build_task_description(&descriptors, &TASK_TOOL_NAMING);
    description.push_str(&task_model_guidance(model_slugs));
    // Phase 7: description.push_str(task_effort_guidance()); // NL orchestration AGENT-06
    description
}
```

**Tests to mirror** (lines 1456–1470): `build_task_description_lists_public_model_slugs` — add effort token guidance assert.

---

### `crates/codegen/xai-grok-shell/.../subagent/handle_request.rs` (authoritative orchestrator)

**Analog A:** same file (spawn orchestration)  
**Analog B:** `agent/handlers/model_switch.rs` (missing-provider gate)

**Tool-only model error helper** (lines 28–44):
```rust
pub(super) fn task_model_override_error(
    requested: Option<&str>,
    provenance: ModelOverrideProvenance,
    is_resume: bool,
    available: &indexmap::IndexMap<String, crate::agent::config::ModelEntry>,
    is_session_auth: bool,
) -> Option<String> {
    if provenance != ModelOverrideProvenance::Tool || is_resume {
        return None;
    }
    let requested = requested?;
    crate::agent::models::task_model_error_for_catalog(requested, available, is_session_auth)
}
```

**Role effort inheritance** (lines 151–156):
```rust
if effective_runtime.reasoning_effort.is_none() {
    effective_runtime.reasoning_effort = definition
        .effort
        .map(|e| <&str>::from(e).to_string());
}
```

**Resume pin** (lines 217–224, 451–470) — do not regress:
```rust
if request.runtime_overrides.model.is_some() {
    tracing::debug!(/* Ignoring caller model override on resume */);
}
effective_runtime.model = None;
// later re-resolve source_model via resolve_model_override_to_config
```

**Anti-pattern today: unknown model → parent fallback** (lines 435–449) — for Tool provenance Phase 7 should fail closed instead:
```rust
if model_unknown {
    // today: fall back to parent_config
    // Phase 7 Tool path: send_failure with catalog error — never wrong-provider inherit
}
```

**Effort apply (MUST strengthen reject)** (lines 472–486):
```rust
if let Some(raw) = effective_runtime.reasoning_effort.as_deref()
    && ctx.models_manager.model_supports_reasoning_effort(effective_model_id.0.as_ref())
{
    use xai_grok_sampling_types::ReasoningEffort;
    match raw.parse::<ReasoningEffort>() {
        Ok(eff) => effective_sampling_config.reasoning_effort = Some(eff),
        Err(err) => {
            // today: warn+ignore — Phase 7 Tool provenance: send_failure / reject
            tracing::warn!(value = raw, error = %err,
                "subagent reasoning_effort: parse failed, ignoring override");
        }
    }
}
```

**Authoritative spawn gate placement:** After effective model resolve (+ resume pin), **before** `spawn_session_on_thread` / credential seed. Copy Phase 6 gate call shape from `model_switch.rs`:

```rust
// From handlers/model_switch.rs lines 72–97
let target_provider = model.info.provider;
if let Some(err_payload) = config::missing_provider_gate_error(
    target_provider,
    model_id.0.as_ref(),
    model.has_own_credentials(),
    provider_oauth_slot_usable(agent, target_provider),
) {
    // spawn: send_failure(request, &message_with_suggestion)
    return Err(err_payload.into_acp_error()); // switch path
}
```

**Spawn-side usable helper** — copy `provider_oauth_slot_usable` (model_switch.rs lines 24–44):
```rust
fn provider_oauth_slot_usable(agent: &MvpAgent, provider: config::ModelProvider) -> bool {
    let path = crate::util::grok_home::grok_home().join("auth.json");
    let slot = match provider {
        config::ModelProvider::Xai => crate::auth::PROVIDER_XAI,
        config::ModelProvider::Codex => crate::auth::PROVIDER_CODEX,
    };
    let disk_usable = match crate::auth::read_provider_auth_store(&path, slot) {
        Ok(store) => crate::auth::provider_slot_usable(store.as_ref()),
        Err(_) => false, // fail closed
    };
    if disk_usable { return true; }
    if matches!(provider, config::ModelProvider::Xai) {
        if let Some(auth) = agent.auth_manager.current_or_expired() {
            return crate::auth::credential_usable(&auth);
        }
    }
    false
}
```

Adapt to `SubagentSpawnContext` (has `auth_manager` / models); extract shared helper if both call sites can share without circular deps.

**Failure channel** (mod.rs lines 2065–2071):
```rust
fn send_failure(request: SubagentRequest, error: &str) {
    let _ = request.result_tx.send(SubagentResult {
        success: false,
        error: Some(error.to_string()),
        ..Default::default()
    });
}
```

**Child credential seed** (handle_request.rs lines 761–773) — keep; do not rewrite parent creds into wrong slot:
```rust
let credentials = xai_chat_state::Credentials {
    api_key: effective_sampling_config.api_key.clone(),
    auth_type: inherited_auth_type,
    alpha_test_key: ctx.alpha_test_key.clone(),
    client_version: effective_sampling_config.client_version.clone(),
};
// log key_prefix only — never full tokens
```

**Message copy for spawn:** Align with Phase 6 suggestion string; adapt verb from "switch" to "spawn":
```text
Cannot spawn subagent with model '{id}': no usable {ProviderLabel} credentials. Run: bum login --provider {xai|codex}
```
Reuse `ModelSwitchMissingProviderError::suggestion` (`bum login --provider {id}`) even if ACP code differs.

---

### `crates/codegen/xai-grok-shell/.../subagent/mod.rs` (resolve isolation)

**Analog:** same file — `resolve_model_override_to_config` (lines 1006–1056)

```rust
fn resolve_model_override_to_config(
    model_id: &str,
    ctx: &SubagentSpawnContext,
) -> Option<(xai_grok_sampler::SamplerConfig, acp::ModelId)> {
    use crate::agent::config::{
        resolve_credentials_for_provider, sampling_config_for_model, ModelProvider,
    };
    let entry = crate::agent::config::find_model_by_id(&ctx.available_models, model_id).cloned()?;
    // ...
    let codex_session_owned =
        crate::agent::config::snapshot_codex_session_key_from_auth_store();
    let (xai_key, codex_key) = match entry.info.provider {
        ModelProvider::Xai => (session_key, None),
        ModelProvider::Codex => (None, codex_session_owned.as_deref()),
    };
    let mut credentials =
        resolve_credentials_for_provider(&entry, &endpoints, xai_key, codex_key);
    let config = sampling_config_for_model(/* ... */);
    Some((config, canonical_model_id))
}
```

**Do not:** simplify to parent-only key; cross-apply xAI token into Codex slot; bypass this for explicit Task.model.

---

### `crates/codegen/xai-grok-shell/src/agent/config.rs` (pure policy)

**Analog:** same file — Phase 6 gate (lines 5529–5612)

```rust
pub const MODEL_SWITCH_MISSING_PROVIDER: &str = "MODEL_SWITCH_MISSING_PROVIDER";

pub struct ModelSwitchMissingProviderError {
    pub code: String,
    pub provider: String,
    pub model_id: String,
    pub suggestion: String, // "bum login --provider {id}"
}

impl ModelSwitchMissingProviderError {
    pub fn new(provider: ModelProvider, model_id: impl Into<String>) -> Self {
        let id = provider.as_str();
        Self {
            code: MODEL_SWITCH_MISSING_PROVIDER.to_string(),
            provider: id.to_owned(),
            model_id: model_id.into(),
            suggestion: format!("bum login --provider {id}"),
        }
    }
}

pub fn missing_provider_gate_error(
    provider: ModelProvider,
    model_id: &str,
    has_own_credentials: bool,
    slot_usable: bool,
) -> Option<ModelSwitchMissingProviderError> {
    if has_own_credentials || slot_usable {
        return None;
    }
    Some(ModelSwitchMissingProviderError::new(provider, model_id))
}
```

**Reuse for spawn:** call pure function as-is; optionally add `SUBAGENT_SPAWN_MISSING_PROVIDER` code later (open Q) without changing usable semantics.

**Attach policy (child turns)** — lines 4680–4685:
```rust
pub fn reconstruct_attach_policy_from_facts(
    facts: &ModelAuthFacts,
    session_token_gate_active: bool,
) -> bool {
    should_attach_xai_auth_manager_bearer_resolver(facts.provider, session_token_gate_active)
}
```
Never attach xAI AuthManager to Codex children.

---

### `crates/codegen/xai-grok-shell/src/auth/status.rs` (usable semantics)

**Analog:** same file (lines 121–156)

```rust
pub fn credential_usable(auth: &GrokAuth) -> bool {
    // hard-unexpired OR nonblank refresh_token
}

pub fn store_usable(store: &AuthStore) -> bool {
    store.values()
        .filter(|a| a.auth_mode != AuthMode::WebLogin)
        .any(credential_usable)
}

pub fn provider_slot_usable(store: Option<&AuthStore>) -> bool {
    store.is_some_and(store_usable)
}
```

**Tests already present:** `p6_provider_slot_usable_*` in this module and `model_switch_gate.rs` — reuse decision table for `p7_*` spawn gate pure tests.

---

### Child per-turn bearer: `sampler_turn.rs` (do not redesign)

**Analog:** `session/acp_session_impl/sampler_turn.rs` `reconstruct_full_config` (lines 231–310)

```rust
let use_bearer_resolver = crate::agent::config::reconstruct_attach_policy_from_facts(
    &model_facts,
    session_token_gate_active,
);
// Codex session OAuth → ensure_fresh_codex_auth(); never xAI token
// xAI → AuthManagerBearerResolver only when attach policy true
```

Phase 7 isolation proofs assert child session chat-state sampling_config + Authorization match child provider; parent model_id unchanged after spawn.

---

### `ReasoningEffort` vocabulary (do not hand-roll)

**Analog:** `crates/codegen/xai-grok-sampling-types/src/types.rs` (lines 765–845)

```rust
pub enum ReasoningEffort { None, Minimal, Low, Medium, High, Xhigh }

impl std::str::FromStr for ReasoningEffort {
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "none" => Ok(Self::None),
            "minimal" => Ok(Self::Minimal),
            "low" => Ok(Self::Low),
            "medium" => Ok(Self::Medium),
            "high" => Ok(Self::High),
            "xhigh" | "max" => Ok(Self::Xhigh),
            _ => Err(format!(
                "invalid reasoning effort: {s:?} (expected one of: none, minimal, low, medium, high, xhigh, max)"
            )),
        }
    }
}
```

Surface accepted tokens from this error (or product subset low/medium/high/xhigh) in Task `invalid_arguments`.

---

### Tests: dual fake tokens + missing-provider

**Analog A:** `crates/codegen/xai-grok-shell/tests/provider_routing.rs`

```rust
const XAI_FAKE: &str = "xai-fake-token";
const CODEX_FAKE: &str = "codex-fake-token";
// dual-token never_cross_slot; mock HTTP Authorization asserts
// Prefer: cargo test -p xai-grok-shell --test provider_routing
```

**Analog B:** `crates/codegen/xai-grok-shell/tests/model_switch_gate.rs`

```rust
const XAI_FAKE: &str = "xai-fake-token-p6";
const CODEX_FAKE: &str = "codex-fake-token-p6";
// BUM_HOME OnceLock hygiene via ensure_sandbox()
// p6_missing_provider_apply_blocks_codex_when_codex_slot_empty
// p6_dual_login_free_switch_xai_to_codex_no_missing_provider
// Prefer: cargo test -p xai-grok-shell --test model_switch_gate p6_
```

**Analog C:** shell unit `subagent/tests/mod.rs` effort precedence (lines 1294–1311):
```rust
fn reasoning_effort_explicit_overrides_role() {
    // explicit high beats role low
}
```

**Phase 7 test naming:** prefix `p7_` for new pure/integration cases; extend Task tool tests for schema + wiring; optional new `--test cross_provider_subagent` only if lib tests cannot assert wire Authorization.

---

## Shared Patterns

### Authentication / credential gate
**Source:** `agent/handlers/model_switch.rs` + `agent/config.rs` + `auth/status.rs`  
**Apply to:** `handle_subagent_request` (authoritative), Task eager preflight (Resources mirror)

- Pure decision: `missing_provider_gate_error(provider, model_id, has_own_credentials, slot_usable)`
- Usable: refreshable OAuth OK; hard-expired without refresh = unusable; empty store = unusable
- xAI live AuthManager can supplement disk; Codex never inferred from xAI manager alone
- BYOK (`has_own_credentials`) skips OAuth-slot gate
- Fail closed — never parent bearer / parent backend fallback
- CLI hint: `bum login --provider {xai|codex}`

### Error handling
**Source:** Task tool `ToolError::invalid_arguments` / `custom`; shell `send_failure` / `send_pre_spawn_failure`  
**Apply to:** invalid effort, unknown model (Tool), missing provider, validation unavailable

| Path | Error style |
|------|-------------|
| Task eager (model/effort/type) | `ToolError::invalid_arguments(msg)` |
| Task missing resource | `ToolError::custom("validation_unavailable", …)` |
| Shell coordinator reject | `send_failure(request, &msg)` → `SubagentResult { success: false, error }` |
| Model switch (reference) | ACP `MODEL_SWITCH_MISSING_PROVIDER` + structured data |

### Validation
**Source:** Task eager chain + `ReasoningEffort::from_str` + `TaskModelValidator`  
**Apply to:** Task input before bg spawn; shell Tool provenance after resolve

1. Depth limit  
2. `validate_type`  
3. Explicit model catalog (`TaskModelValidator`)  
4. **NEW** effort parse (accept product tokens; reject invalid)  
5. **NEW** child-provider usable preflight (model resolved → provider → gate)  
6. Build `SubagentRequest` / spawn  

### Child routing isolation
**Source:** `resolve_model_override_to_config` + `reconstruct_full_config`  
**Apply to:** all explicit-model and resume-pin child spawns

- Catalog `provider` selects credential slot + base_url  
- Dual-key resolve: xAI key XOR Codex snapshot — never both slots from parent  
- Child chat-state seeded at spawn; parent `set_session_model` **not** called  
- Log `key_prefix` only  

### Logging
**Source:** subagent spawn credentials log (handle_request ~774–788)  
**Apply to:** gate rejections and resolve paths  

```rust
tracing::warn!(/* session/subagent ids, provider, model — no secrets */);
// unified_log with key_prefix, base_url, auth_type
```

### Testing conventions
- Fake tokens: `xai-fake-token*` / `codex-fake-token*` only  
- `serial_test` + `BUM_HOME` sandbox for auth.json  
- Phase filter prefixes: `p6_` (switch) → `p7_` (spawn)  
- No live OAuth for phase proofs  

## No Analog Found

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| *(none for core paths)* | — | — | Phase 7 is wiring/gates on existing surfaces |

**Near-greenfield only:** optional dedicated `TaskProviderCredentialGate` resource type (shape clones `TaskModelValidator`); optional `SUBAGENT_SPAWN_MISSING_PROVIDER` error code (semantics clone Phase 6).

## Metadata

**Analog search scope:**  
- `crates/codegen/xai-grok-tools/src/implementations/grok_build/task/`  
- `crates/common/xai-tool-types/src/task.rs`  
- `crates/codegen/xai-grok-shell/src/agent/{subagent,handlers,config}.rs`  
- `crates/codegen/xai-grok-shell/src/auth/status.rs`  
- `crates/codegen/xai-grok-shell/src/session/acp_session_impl/sampler_turn.rs`  
- `crates/codegen/xai-grok-agent/src/builder.rs`  
- `crates/codegen/xai-grok-sampling-types/src/types.rs`  
- `crates/codegen/xai-grok-shell/tests/{provider_routing,model_switch_gate}.rs`  
- `crates/codegen/xai-grok-shell/src/agent/subagent/tests/`

**Files scanned:** ~20 primary sources (targeted greps + section reads; large files not fully loaded)  
**Pattern extraction date:** 2026-07-17  

**Planner note:** Prefer plans that (1) RED `p7_` + Task tests, (2) schema+wire effort, (3) shell authoritative gate + Tool fail-closed effort/unknown-model, (4) Task eager preflight resource, (5) dual-token isolation proofs, (6) AGENT-01 regression green — matching RESEARCH recommended plan shape.
