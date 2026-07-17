# Phase 7: Cross-provider multi-agent orchestration - Research

**Researched:** 2026-07-17
**Domain:** Multi-agent / subagent spawn with dual-provider model routing, reasoning effort, and fail-closed credential gates (Rust shell + Task tool)
**Confidence:** HIGH

## Summary

Phase 7 makes parent/child multi-agent work **across** xAI and Codex providers without regressing same-provider subagent behavior. Most of the hard plumbing already exists: `SubagentRuntimeOverrides` already carries `model` and `reasoning_effort`; explicit model overrides already go through provider-aware `resolve_model_override_to_config` (Phase 4 dual-key credentials + base_url); child turns already rebuild sampling via `reconstruct_full_config` (xAI AuthManager bearer vs Codex `ensure_fresh`); Phase 6 exported a pure `missing_provider_gate_error` + usable-slot helpers. What is **not** done is the product contract for AGENT-01..06: expose effort on the Task schema, wire `Task.reasoning_effort` instead of hardcoding `None`, **reject invalid effort**, and **gate spawn on child-provider usable credentials** before background launch succeeds — never fall back to the parent's bearer/backend.

The highest-risk gap is UX timing: Task defaults to `run_in_background: true` and returns a "started" notice before the coordinator finishes pre-spawn checks. A shell-only gate that fails after fire-and-forget will not satisfy AGENT-05 for the default path unless Task eagerly preflights credentials (or awaits pre-spawn outcome) the same way it already eagerly validates `subagent_type` and `Task.model`.

**Primary recommendation:** Reuse Phase 4 `resolve_model_override_to_config` + Phase 6 `missing_provider_gate_error` / `provider_slot_usable` for a single authoritative spawn gate in `handle_subagent_request` after effective model resolution; mirror an eager Task-layer preflight (like `validate_type`) so background spawns fail closed with a typed tool error + `bum login --provider {xai|codex}`; wire `reasoning_effort` end-to-end through `TaskToolInput` → overrides → child `SamplerConfig`, rejecting invalid tokens with the existing `ReasoningEffort` parse vocabulary.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

#### Spawn contract (model + effort)
- **Omit `Task.model` → inherit parent model/provider** (existing same-provider default). Explicit catalog slug required only when user/parent wants a different model (including cross-provider).
- **Validate explicit model via existing `TaskModelValidator` / live catalog** — unknown slug rejects before spawn; valid GPT entries (e.g. `gpt-5.6-sol`) are allowed even when parent is xAI (and vice versa). Do not restrict child model to parent’s provider.
- **Expose `reasoning_effort` (or equivalent) on the model-facing Task tool schema** and wire it through `SubagentRuntimeOverrides.reasoning_effort` (field already exists; Task currently hardcodes `None`). Omit effort → inherit child’s model default / parent effort behavior already used by the harness.
- **Effort vocabulary:** reuse existing product tokens (`low` / `medium` / `high` / `xhigh` and any catalog-supported aliases). Invalid effort rejects with a clear tool error listing accepted values — no silent clamp.

#### Cross-provider credential gate
- **Check usable credentials for the child’s resolved provider at spawn time** (before background launch), analogous to Phase 6 switch-time gate: token present and not permanently unusable; expired-but-refreshable is OK (refresh on first child sample).
- **Fail closed — never fall back to parent bearer or parent backend** when child provider is missing/unusable. Do not rewrite parent credentials into the child slot.
- **Error surface:** typed tool/ACP error naming the missing provider (xAI / Codex) plus explicit CLI hint `bum login --provider {xai|codex}`. No silent 401 as primary UX for spawn.
- **Same-provider spawns** (child model same provider as parent, parent already usable): **no extra gate friction**. Gate only when target child model’s provider lacks usable creds (including cross-provider and same-provider-if-somehow-missing).

#### Child routing isolation
- **Child session builds SamplingConfig from the child’s model → catalog `provider` → base_url + credential slot + api_backend** using Phase 4 pure resolver — not parent’s fixed route.
- **Child turns resolve bearer from the child’s provider slot only** (live AuthManager / SharedApiKeyProvider-style per request). Parent and child may run concurrent turns with different providers without clobbering each other.
- **Parent session model/provider is unchanged by spawn** — spawning a Codex child does not switch the parent’s current model.
- **Prove with automated tests + dual fake tokens:** assert child request Authorization + base URL match child provider; parent continues on its own slot; missing child slot fails spawn with login-shaped error (no wrong-backend request).

#### NL orchestration & phase boundary
- **Enable NL orchestration via Task tool contract + schema/docs** so the parent model can call `task` with `model` + `reasoning_effort` when the user asks (e.g. “start a Codex Sol medium-effort subagent to research X”). Prefer light schema/description improvements over a new workflow engine.
- **Same-provider regression is a hard gate (AGENT-01):** spawn/resume/roles/personas/parent↔child routing keep working when parent and child share a provider; cross-provider is additive.
- **`resume_from` keeps existing pin:** resume inherits source child’s model (ignore model override on resume — already soft-ignored). Document that cross-provider resume stays on the prior child model.
- **Out of scope this phase:** custom agentic workflows, multi-account cost dashboards, Phase 8 rebrand polish, Phase 9 full daily-driver matrix (except automated proofs needed for AGENT-01..06).

### Claude's Discretion
- Exact module placement for spawn-time provider gate (Task tool pre-check vs shell `handle_subagent_request` — prefer single authoritative check so tool and harness paths cannot diverge)
- How deep to thread effort into child SamplingConfig rebuild vs session-level override
- Test layout (tools unit vs shell integration) and fixture patterns for dual-slot spawn
- Whether to extend goal-orchestrator / harness-internal spawns with effort in this phase or only model-facing Task path
- Copy strings for missing-provider spawn errors (align with Phase 6 provider labels)

### Deferred Ideas (OUT OF SCOPE)
- Custom agentic workflow engine / workflow product surface — later milestone (WF-V2-01)
- Cross-provider multi-account / cost attribution dashboards — AGENT-V2-01
- Richer per-provider capability matrix UI and first-class TUI effort settings beyond spawn args — MOD-V2-01
- Full quiet rebrand of strings/chrome — Phase 8
- Daily-driver end-to-end validation matrix — Phase 9
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| AGENT-01 | Same-provider subagent spawn/resume/roles/personas still work | Existing coordinator + `handle_subagent_request` + resume pin + roles/personas path; regression tests must stay green; omit `model` inherits parent via `resolve_subagent_sampling_config` |
| AGENT-02 | Spawn with explicit model on different provider than parent | `Task.model` + `TaskModelValidator` already allow any catalog slug; `resolve_model_override_to_config` already provider-routes; prove both Grok→Codex and Codex→Grok |
| AGENT-03 | Launch with reasoning effort; child runs with that effort | Expose schema field; stop hardcoding `None`; apply into `effective_sampling_config.reasoning_effort` (path exists; invalid must reject not warn) |
| AGENT-04 | Child turns use child model→provider→creds→backend | Phase 4 resolve + child chat-state credentials + `reconstruct_full_config` attach policy; dual fake-token wire tests |
| AGENT-05 | Missing child provider fails closed with login prompt | Reuse Phase 6 usable semantics + spawn-time gate; surface before background "started"; never parent fallback |
| AGENT-06 | NL orchestration (Grok parent → Codex Sol medium effort) | Schema/descriptions for `model` + `reasoning_effort`; same spawn path as AGENT-02/03; automated proof, not live LLM |
</phase_requirements>

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Task schema (`model`, `reasoning_effort`) | API / Backend (`xai-tool-types` + Task tool) | — | Model-facing JSON schema is tool boundary |
| Catalog model validation | API / Backend (`TaskModelValidator` / shell models) | — | Live catalog; already injected into Resources |
| Spawn-time credential gate | API / Backend (shell `handle_subagent_request` authoritative) | Task eager preflight | Single authority for tool + harness; Task must not return "started" before fail-closed |
| Child SamplingConfig build | API / Backend (`resolve_model_override_to_config` / Phase 4) | — | Model → provider → base_url / slot / backend |
| Child per-turn bearer | API / Backend (session `reconstruct_full_config`) | Auth dual-slot store | xAI AuthManager vs Codex ensure_fresh; never cross-slot |
| Parent model stability | API / Backend (spawn must not call set_session_model) | — | Parent chat state untouched by child spawn |
| Resume model pin | API / Backend (`handle_subagent_request` + Task soft-ignore) | — | Source model wins; ignore override |
| NL orchestration enablement | API / Backend (schema + `build_task_description`) | Parent model (uses tool) | Light docs/schema — not a workflow engine |
| Same-provider regression | API / Backend (existing subagent suite) | — | Hard gate AGENT-01 |
| TUI spawn gate modal | Browser / Client (pager) | — | Optional; tool error is primary for model-spawned Task; Phase 6 QuestionView is for model *switch*, not required for tool spawn |

## Standard Stack

### Core (reuse — do not replace)

| Library / component | Version / location | Purpose | Why standard |
|---------------------|-------------------|---------|--------------|
| Rust workspace | edition 2024, toolchain 1.92.0 | Product harness | Project constraint — fork evolution |
| `TaskTool` | `xai-grok-tools` `task/mod.rs` | Model-facing spawn | Existing multi-agent entry |
| `SubagentRequest` / `SubagentRuntimeOverrides` | `task/types.rs` | Spawn protocol | Already has model + reasoning_effort fields |
| `ChannelBackend` / `SubagentBackend` | `task/backend.rs` | Transport to coordinator | In-process mpsc + oneshot |
| `handle_subagent_request` | `xai-grok-shell` `agent/subagent/handle_request.rs` | Authoritative spawn orchestration | Worktree, model, effort, session spawn |
| `resolve_model_override_to_config` | `agent/subagent/mod.rs` | Explicit model → SamplerConfig | Phase 4 provider-aware dual-key resolve |
| `resolve_provider_route` / `resolve_credentials_for_provider` / `sampling_config_for_model` | `agent/config.rs` | Route + credentials | Phase 4 pure resolver stack |
| `missing_provider_gate_error` / `MODEL_SWITCH_MISSING_PROVIDER` | `agent/config.rs` | Fail-closed missing creds policy | Phase 6 pure gate — reuse semantics |
| `credential_usable` / `store_usable` / `provider_slot_usable` | `auth/status.rs` | Usable slot semantics | Refreshable OK; hard-expired no refresh = false |
| `ReasoningEffort` | `xai-grok-sampling-types` | Effort parse / wire | `low|medium|high|xhigh` + `max` alias |
| Tokio + `cargo test` | workspace | Async + tests | Existing harness |
| Dual fake tokens | test pattern | Auth isolation proofs | `xai-fake-token` / `codex-fake-token` (Phase 4/5/6) |

### Supporting

| Component | Purpose | When to use |
|-----------|---------|-------------|
| `TaskModelValidator` resource | Reject unknown Task.model before spawn | Already for explicit model |
| `build_task_description` | NL-facing tool description | AGENT-06 schema/docs |
| `ModelSwitchMissingProviderError` | Copy/shape reference for login hint | Align spawn error messaging (new code ok if spawn-specific) |
| `provider_routing` integration tests | Wire Authorization + base_url | Template for child isolation tests |
| `model_switch_gate` tests | Usable-slot + dual-login patterns | Template for spawn gate tests |
| Existing subagent unit tests | Resume, roles, pre-spawn failure | AGENT-01 regression |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Reuse Phase 6 pure gate | Hand-roll spawn-only usable check | Worse — drift from switch semantics |
| Authoritative shell gate + Task preflight | Shell-only gate | Background default returns "started" before shell runs — fails AGENT-05 UX |
| Task schema effort field | Config-only / role effort | Blocks NL orchestration (AGENT-06) |
| New multi-agent API | Extend Task + existing coordinator | Locked: prefer existing Task plumbing |

**Installation:** none — no new crates/npm packages required. [VERIFIED: codebase]

**Version verification:** Pin remains workspace Rust 1.92.0 / edition 2024; no registry package adds for this phase. [VERIFIED: rust-toolchain.toml / codebase]

## Package Legitimacy Audit

> No new external packages are recommended for this phase. Work is confined to the existing Rust workspace crates (`xai-tool-types`, `xai-grok-tools`, `xai-grok-shell`, optionally `xai-grok-agent` description builder).

| Package | Registry | Age | Downloads | Source Repo | Verdict | Disposition |
|---------|----------|-----|-----------|-------------|---------|-------------|
| *(none)* | — | — | — | — | — | No installs |

**Packages removed due to [SLOP] verdict:** none  
**Packages flagged as suspicious [SUS]:** none

## Architecture Patterns

### System Architecture Diagram

```text
User / Parent model
        │
        ▼
  Task tool (xai-grok-tools)
   ├─ depth / type validate (eager)
   ├─ TaskModelValidator (eager, explicit model)
   ├─ [NEW] effort parse reject invalid
   ├─ [NEW] eager child-provider usable preflight (bg-safe)
   └─ SubagentRequest { model?, reasoning_effort?, ... }
        │
        ▼  SubagentEvent::Spawn (ChannelBackend)
  MvpAgent subagent coordinator
        │
        ▼
  handle_subagent_request (AUTHORITATIVE)
   ├─ resolve roles/personas/toolset
   ├─ resolve effective model:
   │    explicit → resolve_model_override_to_config (provider-aware)
   │    omit → resolve_subagent_sampling_config (pin / def / inherit parent)
   ├─ [NEW] missing_provider_gate on child's provider (BYOK skip)
   ├─ apply reasoning_effort → SamplerConfig (reject invalid if Tool path)
   ├─ seed child Credentials + SamplingConfig
   └─ spawn_session_on_thread (parent model UNCHANGED)
        │
        ▼
  Child SessionActor turns
   └─ reconstruct_full_config
        ├─ provider from child's model catalog facts
        ├─ xAI → AuthManager bearer (only if provider Xai)
        └─ Codex → ensure_fresh_codex_auth (never xAI token)
```

### Recommended Project Structure (touch points)

```text
crates/common/xai-tool-types/src/task.rs          # TaskToolInput.reasoning_effort + schema docs
crates/codegen/xai-grok-tools/.../task/mod.rs     # wire effort; eager gate hooks; tests
crates/codegen/xai-grok-tools/.../task/types.rs   # already has overrides (minimal change)
crates/codegen/xai-grok-agent/src/builder.rs      # optional NL description for model+effort
crates/codegen/xai-grok-shell/src/agent/subagent/
  handle_request.rs                               # authoritative gate + effort fail-closed
  mod.rs                                          # resolve_model_override_to_config (reuse)
crates/codegen/xai-grok-shell/src/agent/config.rs # missing_provider_gate_error (reuse)
crates/codegen/xai-grok-shell/src/auth/status.rs  # provider_slot_usable (reuse)
crates/codegen/xai-grok-shell/tests/
  provider_routing.rs                             # dual fake token patterns
  model_switch_gate.rs                            # usable-slot patterns
  [new] cross_provider_subagent.rs (optional integration harness)
```

### Pattern 1: Explicit model override → provider-isolated SamplingConfig

**What:** Catalog entry provider selects credential slot and base URL; never copies parent bearer into the other slot.  
**When to use:** Any explicit `runtime_overrides.model` (Task or harness).  
**Example:**

```rust
// Source: crates/codegen/xai-grok-shell/src/agent/subagent/mod.rs
// resolve_model_override_to_config (~1006)
let (xai_key, codex_key) = match entry.info.provider {
    ModelProvider::Xai => (session_key, None),
    ModelProvider::Codex => (None, codex_session_owned.as_deref()),
};
let mut credentials =
    resolve_credentials_for_provider(&entry, &endpoints, xai_key, codex_key);
let config = sampling_config_for_model(&entry, credentials, /* ... */);
```

[VERIFIED: codebase]

### Pattern 2: Phase 6 pure missing-provider gate (reuse for spawn)

**What:** Fail closed when OAuth slot unusable and model is not BYOK.  
**When to use:** After child model is resolved, before `spawn_session_on_thread`.  
**Example:**

```rust
// Source: agent/config.rs missing_provider_gate_error + handlers/model_switch.rs
if let Some(err) = missing_provider_gate_error(
    target_provider,
    model_id,
    model.has_own_credentials(),
    provider_oauth_slot_usable(/* dual-slot disk + xAI live */),
) {
    // spawn: send_failure / tool error with err.suggestion ("bum login --provider …")
    return;
}
```

[VERIFIED: codebase]

### Pattern 3: Task eager validation before background fire-and-forget

**What:** `validate_type` and `TaskModelValidator` already run *before* background spawn returns "started". Credential gate and invalid effort must follow the same pattern.  
**When to use:** Default `run_in_background: true`.  
**Example:** Existing model validation block in `task/mod.rs` (~275–285) before building `SubagentRequest`. [VERIFIED: codebase]

### Pattern 4: Resume pins source model

**What:** `resume_from` clears/ignores model override; source model re-applied after resolve.  
**When to use:** Always on resume (cross-provider resume stays on prior child model). [VERIFIED: codebase `handle_request.rs` ~217–224, ~451–470; Task soft-ignore ~170–182]

### Anti-Patterns to Avoid

- **Silent effort clamp / warn-and-ignore on Task path:** Today parse failures only `warn!` and continue (`handle_request.rs` ~479–485). Locked decision: invalid effort must **reject**. [VERIFIED: codebase]
- **Parent-model fallback after explicit Tool model:** Unknown resolved model currently falls back to parent (`handle_request.rs` ~435–449). For Tool provenance this can become wrong-provider inherit — fail closed instead after validation. [VERIFIED: codebase]
- **Attaching xAI `AuthManagerBearerResolver` to Codex children:** `reconstruct_attach_policy_from_facts` already blocks this — do not bypass. [VERIFIED: codebase]
- **Using parent `ctx.auth` as universal session key for Codex:** Explicit override path already dual-keys; do not "simplify" to parent-only. [VERIFIED: codebase]
- **New multi-agent product API / workflow engine:** Deferred (WF-V2). Extend Task only.
- **Gate only in shell after background return:** Breaks AGENT-05 default path.
- **Logging full tokens:** Existing `key_prefix` only — keep secrets out of logs.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Provider → base_url / slot | Custom URL ifs in Task | `resolve_provider_route` / `resolve_credentials_for_provider` | Host trust + dual-slot already proven |
| Usable credentials | Ad-hoc expiry checks | `credential_usable` / `provider_slot_usable` | Refreshable OAuth semantics match Phase 6 |
| Effort parsing | Custom string match | `ReasoningEffort::from_str` / `parse_canonical_effort_token` | Aliases (`max`→xhigh) already defined |
| Subagent transport | New RPC | `SubagentBackend` / coordinator | Depth, cancel, resume already integrated |
| Dual auth store format | New auth.json shape | Existing multi-slot under `$BUM_HOME` | Phase 5 complete |
| NL orchestration engine | Workflow product | Task schema + description | Locked; Phase 9 for live E2E |

**Key insight:** Phase 7 is mostly **wiring and gates**, not greenfield multi-agent. The regression risk is higher than the greenfield risk — protect same-provider paths while adding fail-closed cross-provider edges.

## Common Pitfalls

### Pitfall 1: Background spawn reports success before gate runs

**What goes wrong:** Parent model thinks subagent started; failure only appears later (poll / log).  
**Why it happens:** Task `run_in_background` default true; fire-and-forget drops result.  
**How to avoid:** Eager preflight (type + model + effort + provider usable) before returning started message; still keep shell authoritative gate for harness paths.  
**Warning signs:** Tests only cover blocking `spawn().await` and miss default background.

### Pitfall 2: Empty Codex key still spawns with parent xAI identity

**What goes wrong:** Child request hits wrong host or 401 with confusing auth.  
**Why it happens:** `resolve_credentials_for_provider` can construct empty key configs; unknown model falls back to parent config.  
**How to avoid:** Gate with `missing_provider_gate_error` after resolve; for Tool explicit model, do not fall back to parent on unknown.  
**Warning signs:** Child `base_url` equals parent cli-chat-proxy when model is `gpt-5.6-sol`.

### Pitfall 3: Invalid effort silently ignored

**What goes wrong:** User/parent asks for medium effort; child runs model default.  
**Why it happens:** Current parse path warns and continues.  
**How to avoid:** On Task/Tool provenance, `Err` → `send_failure` / tool `invalid_arguments` listing accepted tokens.  
**Warning signs:** Only `tracing::warn` in spawn path for effort.

### Pitfall 4: Effort applied but model does not support reasoning effort

**What goes wrong:** Override dropped when `model_supports_reasoning_effort` is false.  
**Why it happens:** Existing guard skips apply.  
**How to avoid:** Discretion — either clear tool error ("model does not support reasoning_effort") or document soft-ignore for unsupported models only (invalid tokens still hard-fail). Prefer clear error when caller *explicitly* set effort.  
**Warning signs:** AGENT-03 tests pass on Grok models that lack RE support but fail on Sol.

### Pitfall 5: Concurrent parent/child turns clobber credentials

**What goes wrong:** Parent switch or refresh overwrites child route.  
**Why it happens:** Shared process AuthManager + shared chat state mistakes.  
**How to avoid:** Child has own session chat state seeded at spawn; reconstruct uses *child* model facts; Codex ensure_fresh is slot-scoped; do not call parent `set_session_model` on spawn.  
**Warning signs:** Parent model id changes after spawn; child Authorization shows parent token after parent switch.

### Pitfall 6: Resume + model override confusion

**What goes wrong:** Parent tries to "resume as other provider".  
**Why it happens:** Models emit both `resume_from` and `model`.  
**How to avoid:** Keep soft-ignore + pin; document in Task schema.  
**Warning signs:** Cross-provider resume tests expect model override to win.

### Pitfall 7: Restricting child model to parent provider in validator

**What goes wrong:** AGENT-02 blocked by overzealous filter.  
**Why it happens:** Tempting "safety" check.  
**How to avoid:** `TaskModelValidator` stays catalog-visible only (no provider equality). Gate is credential-based, not provider-match-based.  
**Warning signs:** Error text "must match parent provider".

## Code Examples

### Current Task hardcode (must change)

```rust
// Source: crates/codegen/xai-grok-tools/.../task/mod.rs ~305-308
runtime_overrides: SubagentRuntimeOverrides {
    model,
    model_override_provenance: ModelOverrideProvenance::Tool,
    reasoning_effort: None, // ← AGENT-03: wire from TaskToolInput
    // ...
},
```

[VERIFIED: codebase]

### TaskToolInput lacks effort field (must add)

```rust
// Source: crates/common/xai-tool-types/src/task.rs ~96-104
/// Optional model slug for this subagent.
pub model: Option<String>,
// no reasoning_effort field today
```

[VERIFIED: codebase]

### Effort apply path already present (strengthen reject)

```rust
// Source: handle_request.rs ~472-486
if let Some(raw) = effective_runtime.reasoning_effort.as_deref()
    && ctx.models_manager.model_supports_reasoning_effort(effective_model_id.0.as_ref())
{
    match raw.parse::<ReasoningEffort>() {
        Ok(eff) => effective_sampling_config.reasoning_effort = Some(eff),
        Err(err) => { /* today: warn+ignore — Phase 7: fail closed for Tool */ }
    }
}
```

[VERIFIED: codebase]

### ReasoningEffort vocabulary

```text
FromStr accepts: none, minimal, low, medium, high, xhigh, max(=xhigh)
CONTEXT product tokens: low / medium / high / xhigh (+ catalog aliases)
```

[VERIFIED: codebase `xai-grok-sampling-types`]

### Child credential seed at spawn

```rust
// Source: handle_request.rs ~761-773
let credentials = xai_chat_state::Credentials {
    api_key: effective_sampling_config.api_key.clone(),
    auth_type: inherited_auth_type,
    // ...
};
// passed into spawn_session_on_thread — parent session model not mutated
```

[VERIFIED: codebase]

### Dual fake-token proof pattern (Phase 4)

Reuse `crates/codegen/xai-grok-shell/tests/provider_routing.rs` style: mock HTTP, assert `Authorization` and host differ per provider; never cross-slot. Extend with subagent resolve + gate cases rather than inventing a new mock stack. [VERIFIED: codebase]

## State of the Art (in this fork)

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Single-provider Grok subagents | Dual-slot auth + provider-tagged catalog | Phases 2–5 | Child can target Codex models in catalog |
| Switch without auth gate | Phase 6 missing-provider gate | Phase 6 | Switch fail-closed; spawn still open |
| Task.model optional | Catalog validator; no provider filter | Pre-existing | Cross-provider model arg already allowed |
| RuntimeOverrides.reasoning_effort | Field exists; Task hardcodes None | Pre-existing | AGENT-03 is wiring, not design |
| Child model resolve | Provider-aware for explicit override | Phase 4 path in subagent | Main isolation already correct if gate + no parent fallback |

**Deprecated/outdated for this phase:**
- Treating mid-turn 401 as primary spawn UX (Phase 6/7: fail closed earlier)
- Silent ignore of invalid Task effort

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Goal/harness-internal spawns need not expose effort in this phase (Task path only) — matches discretion | Discretion | Product may want goal roles to pass effort later |
| A2 | Spawn missing-provider error can be tool `invalid_arguments` / `custom` with Phase 6-aligned message; full ACP typed code optional if tool path only | Error surface | Headless ACP spawn clients may want structured code like `MODEL_SWITCH_MISSING_PROVIDER` |
| A3 | No TUI QuestionView required for Task spawn failures (tool error + login CLI hint sufficient); switch modal remains Phase 6 | Architecture | Users spawning only via UI chrome (if any) might need modal — none identified beyond Task |

**If empty table:** N/A — three discretion-facing assumptions logged.

## Open Questions

1. **Typed error code for spawn missing provider**
   - What we know: Phase 6 uses `MODEL_SWITCH_MISSING_PROVIDER` for model switch ACP.
   - What's unclear: Whether spawn should reuse that code, introduce `SUBAGENT_SPAWN_MISSING_PROVIDER`, or tool-string-only.
   - Recommendation: Reuse pure gate + login suggestion string; prefer a distinct spawn code if ACP clients need to branch; tool path can embed the same suggestion text either way.

2. **Explicit effort on models that do not support reasoning effort**
   - What we know: apply is gated on `model_supports_reasoning_effort`.
   - What's unclear: Hard fail vs soft-ignore for supported-token-on-unsupported-model.
   - Recommendation: Hard fail with clear message when Task sets effort explicitly; soft-ignore only when effort comes from role default and model lacks support.

3. **How "eager" Task credential preflight gets auth truth without shell coupling**
   - What we know: Task already uses Resources (`TaskModelValidator`, backend).
   - What's unclear: New resource (`TaskProviderCredentialGate`) vs backend RPC preflight vs temporary await of spawn pre-ack.
   - Recommendation: Pure gate function in shell/auth re-exported; inject thin `Fn(model_slug) -> Option<String>` resource parallel to `TaskModelValidator` so tools stay free of auth.json I/O details.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust / cargo | Build + tests | ✓ | 1.92.0 | — |
| protoc | Codegen crates check | ✓ | 3.21.12 | repo `bin/protoc` |
| Node (gsd-tools) | Planning commits | ✓ | v22.22.1 | — |
| Live ChatGPT/xAI OAuth | Product E2E | N/A for phase proofs | — | Dual fake tokens + mock HTTP only |
| graphify | Optional graph context | disabled | — | Codebase grep |

**Missing dependencies with no fallback:** none for automated Phase 7 proofs.

**Missing dependencies with fallback:** live OAuth (use fakes).

## Validation Architecture

> `workflow.nyquist_validation` is enabled in `.planning/config.json`.

### Test Framework

| Property | Value |
|----------|-------|
| Framework | `cargo test` (crate-local); `serial_test` for env/auth isolation |
| Config file | none workspace-wide — per-crate / `--test` binaries |
| Quick run command | `cargo test -p xai-grok-tools --lib task::` and `cargo test -p xai-grok-shell --lib p7_` (or subagent filter) |
| Full suite command | `cargo test -p xai-grok-tools --lib` + `cargo test -p xai-grok-shell --lib` + targeted `--test provider_routing` / new harness |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|--------------|
| AGENT-01 | Same-provider spawn/resume/roles | unit/integration | existing subagent tests + `p7_same_provider_*` | ✅ partial / ❌ Wave 0 for p7_ prefix |
| AGENT-02 | Cross-provider explicit model resolve | unit | `resolve_model_override_to_config` both directions | ❌ Wave 0 |
| AGENT-03 | Effort wired + invalid reject | unit | Task input → overrides; parse reject | ❌ Wave 0 |
| AGENT-04 | Child Authorization/base_url isolation | integration mock HTTP | dual fake tokens child vs parent | ❌ Wave 0 |
| AGENT-05 | Missing slot fails before spawn / no wrong backend | unit + integration | empty codex slot + Grok parent | ❌ Wave 0 |
| AGENT-06 | Schema exposes model+effort; description guidance | unit | TaskToolInput schema / build_task_description | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** focused `cargo test -p <crate> --lib <filter> -q`
- **Per wave merge:** shell + tools lib filters for `p7_` / subagent + provider_routing smoke
- **Phase gate:** All AGENT-01..06 mapped tests green; same-provider suite non-regressed

### Wave 0 Gaps

- [ ] `p7_` pure unit tests for spawn missing-provider decision (reuse Phase 6 helpers)
- [ ] Task tool tests: `reasoning_effort` parse accept/reject; wiring into `SubagentRuntimeOverrides`
- [ ] Shell tests: cross-provider resolve isolation (base_url + key_prefix) both directions
- [ ] Shell/integration: dual fake tokens — concurrent parent/child Authorization
- [ ] Shell/integration: missing child slot → `send_failure` / tool error contains `bum login --provider`
- [ ] Schema/description test: model + reasoning_effort visible to model
- [ ] Optional dedicated `--test cross_provider_subagent` if lib tests cannot reach wire asserts cleanly

*(Existing infrastructure: `provider_routing.rs`, `model_switch_gate.rs`, `task/mod.rs` unit tests, `agent/subagent/tests` — extend rather than replace.)*

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | yes | Dual-slot OAuth under `$BUM_HOME/auth.json`; spawn gate usable check |
| V3 Session Management | yes | Independent refresh; child ensure_fresh / AuthManager per provider |
| V4 Access Control | yes | Never attach wrong provider bearer; BYOK skip only when `has_own_credentials` |
| V5 Input Validation | yes | Catalog model slug + `ReasoningEffort` parse; cwd/type already validated |
| V6 Cryptography | no new | Existing token storage; do not hand-roll crypto |

### Known Threat Patterns for multi-provider subagents

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Cross-provider token leakage (parent bearer on child backend) | Information Disclosure | Provider-scoped resolve + reconstruct attach policy |
| Spawn with empty child slot → wrong backend probe | Spoofing / Tampering | Spawn-time missing_provider gate fail closed |
| Secret logging (full API keys) | Information Disclosure | `key_prefix` only; no token bodies in tool errors |
| Model slug injection / unknown model fallback to parent | Elevation / Spoofing | TaskModelValidator + no parent fallback on Tool override |
| Effort string injection | Tampering | Enum parse allowlist |

## Sources

### Primary (HIGH confidence)

- [VERIFIED: codebase] `crates/codegen/xai-grok-tools/src/implementations/grok_build/task/{mod,types,backend}.rs` — Task spawn, hardcodes `reasoning_effort: None`, background path
- [VERIFIED: codebase] `crates/common/xai-tool-types/src/task.rs` — `TaskToolInput` schema (model only)
- [VERIFIED: codebase] `crates/codegen/xai-grok-shell/src/agent/subagent/handle_request.rs` — spawn orchestration, effort apply, resume pin
- [VERIFIED: codebase] `crates/codegen/xai-grok-shell/src/agent/subagent/mod.rs` — `resolve_model_override_to_config`, inherit path
- [VERIFIED: codebase] `crates/codegen/xai-grok-shell/src/agent/config.rs` — Phase 4 routing + Phase 6 `missing_provider_gate_error`
- [VERIFIED: codebase] `crates/codegen/xai-grok-shell/src/auth/status.rs` — usable slot semantics
- [VERIFIED: codebase] `crates/codegen/xai-grok-shell/src/session/acp_session_impl/sampler_turn.rs` — child/parent turn bearer attach
- [VERIFIED: codebase] `crates/codegen/xai-grok-shell/tests/provider_routing.rs` + `model_switch_gate.rs` — dual fake + gate patterns
- [VERIFIED: codebase] `.planning/phases/07-.../07-CONTEXT.md`, `REQUIREMENTS.md`, `ROADMAP.md`

### Secondary (MEDIUM confidence)

- Phase 6 RESEARCH patterns for gate placement and error copy alignment (same repo planning artifact)

### Tertiary (LOW confidence)

- None material; phase is in-tree Rust.

## Metadata

**Confidence breakdown:**
- Standard stack: **HIGH** — reuse of existing crates/paths verified in source
- Architecture: **HIGH** — spawn + resolve + reconstruct paths traced end-to-end
- Pitfalls: **HIGH** — background fire-and-forget + silent effort ignore + parent fallback confirmed in code

**Research date:** 2026-07-17  
**Valid until:** 2026-08-16 (stable domain; re-check if Task/subagent modules heavily refactored)

## Recommended Plan Shape (for planner)

Fine granularity (project mode), ~5–6 plans:

1. **Wave 0 harness** — RED `p7_` tests: effort schema/wiring contracts, missing-provider spawn gate pure decisions, dual-token isolation stubs, same-provider regression anchors.
2. **Task contract** — Add `reasoning_effort` to `TaskToolInput` + schemars; wire into `SubagentRuntimeOverrides`; reject invalid; update description builder lightly (AGENT-03/06 surface).
3. **Authoritative spawn gate + effort fail-closed** — `handle_subagent_request` after model resolve: `missing_provider_gate_error`; reject invalid effort for Tool provenance; no parent fallback for Tool explicit model; resume pin unchanged (AGENT-02/04/05 core).
4. **Eager Task preflight** — Resource or backend preflight so background default fails closed with login hint before "started" (AGENT-05).
5. **Isolation proofs** — Dual fake tokens: child Authorization/base_url both directions; parent model unchanged; concurrent parent/child smoke (AGENT-04).
6. **Phase gate / AGENT-01 regression** — Same-provider spawn/resume/roles suite green; map AGENT-01..06 to green tests; no live OAuth.

**Primary files to change:**
- `crates/common/xai-tool-types/src/task.rs`
- `crates/codegen/xai-grok-tools/src/implementations/grok_build/task/mod.rs`
- `crates/codegen/xai-grok-shell/src/agent/subagent/handle_request.rs`
- `crates/codegen/xai-grok-shell/src/agent/subagent/mod.rs` (only if extract helpers / fail-closed fallback)
- Optional: `xai-grok-agent/src/builder.rs` description
- Tests under tools + shell (new `p7_` filters / optional integration test file)

**Blockers:** none for planning/execution with fake tokens. Live dual-login E2E remains Phase 9 (OPS-06).
