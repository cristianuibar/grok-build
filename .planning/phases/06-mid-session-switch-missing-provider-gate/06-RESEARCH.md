# Phase 6: Mid-session switch & missing-provider gate - Research

**Researched:** 2026-07-17
**Domain:** Mid-session model switch (Rust shell + pager TUI/ACP) with dual-provider auth gate
**Confidence:** HIGH

## Summary

Phase 6 closes the UX gap left by Phases 4–5: routing and dual-slot credentials already work, but `model_switch::apply` is still **ungated on auth**. Users can switch to a Codex/GPT model with an empty `providers.codex` slot and only discover failure mid-turn via 401. MOD-03 (mid-session switch without restart) is largely already implemented by the Action → Effect → ACP `session/set_model` → `SessionCommand::SetSessionModel` path; MOD-06 requires a **switch-time fail-closed gate** with a typed ACP error, QuestionView recovery, light picker badges, and post-login auto-retry of a stashed target model.

The authoritative check must live in **shell** `model_switch::apply` (same path as agent-type incompatibility) so TUI picker, `/model`, settings live switch, and ACP clients cannot diverge. Pager maps a new typed error to a **separate** QuestionView family from `IncompatibleAgent`, reuses `deferred_model_switch` for Login now → AuthComplete auto-retry, and adds a light `needs login` badge without filtering the mixed catalog.

**Primary recommendation:** Add `ModelSwitchMissingProviderError` + early gate in `model_switch::apply` using Phase 5 `store_usable` / `AuthStatusReport` semantics (refreshable = usable); map to pager `SwitchModelError::MissingProvider` + `LocalQuestionKind::MissingProviderLogin`; never optimistic-commit `models.current` for this error class.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

#### Gate timing (when missing credentials block)
- Run the missing-provider check **at switch time** — block `set_model` / picker confirm / `/model` before applying the new model (matches MOD-06: “Selecting a model… blocks the switch”)
- **Usable credentials** = existing provider-slot `has_usable_token` / equivalent APIs: token present and not permanently unusable; **expired-but-refreshable is OK** (Phase 5 independent refresh owns refresh on next sample)
- Mid-session token death after a successful switch: **switch-gate remains primary UX**; mid-turn 401 / re-login recovery stays **secondary**, not the MOD-06 design center
- **Same-provider** model switches (e.g. Sol→Terra with Codex logged in) have **no extra friction** — gate only when the **target model’s provider** lacks usable creds

#### Login recovery UX after a blocked switch
- User-facing block: **clear TUI modal/toast** naming the missing provider (xAI / Codex) and instructing login — never a silent refuse or raw 401 as primary UX
- Offer **“Login now”** that reuses Phase 5 provider login (browser/device) when a minimal in-TUI path exists; **always** show the CLI fallback: `bum login --provider {xai|codex}`
- After successful login from the gate: **auto-retry the pending switch** to the model they selected (reuse/extend existing `deferred_model_switch` pattern)
- Headless / ACP clients: **typed ACP error** with provider id + login hint (machine-readable); no silent 401 as primary

#### Picker presentation (auth-aware list)
- **Keep the full mixed catalog** visible even when a provider is not logged in (Phase 3 locked — no auth-based filtering of GPT rows)
- Add a **light badge/hint** on rows whose provider lacks usable credentials (e.g. “needs login”) — rows stay selectable so the gate can run
- **No optimistic current-model update** until the gate passes; on fail stay on the previous model
- **`/model` slash command** and any other switch entry points use the **same gate policy** as the picker

#### Mid-turn switch & session continuity
- Switching while a turn is running is **allowed**; new model applies to the **next turn only** (Phase 4 next-sample semantics) — do **not** cancel the in-flight turn by default
- **Keep full conversation history** across cross-provider switches in the same session
- On successful cross-provider switch: **light confirmation** (model + provider label) — no modal spam
- Keep existing **incompatible-agent** rejection / new-session offer as a **separate** typed path; missing-provider is its own typed error (do not collapse into one generic failure)

### Claude's Discretion
- Exact modal vs toast chrome, Theme tokens, and copy strings (provider labels “xAI” / “Codex”)
- Module placement for the gate (shell `model_switch` pre-check vs pager-side check before Effect) — prefer single authoritative check so TUI and ACP cannot diverge
- Shape of typed `SwitchModelError` / ACP error code for missing provider
- How deep to wire “Login now” into existing welcome/login surfaces vs opening an external browser from TUI
- Whether auth badge is a suffix, dim row meta, or ACP model meta field
- Test layout (unit pure gate + shell/pager integration) and fixture patterns for dual-slot usable/missing states

### Deferred Ideas (OUT OF SCOPE)
- Cross-provider multi-agent / subagent orchestration → Phase 7
- Full product rebrand of chrome/help strings → Phase 8
- Richer per-provider capability matrix UI / first-class effort controls beyond switch → REQUIREMENTS MOD-V2-01
- Stock credential import from `~/.codex` / `~/.grok` → never v1
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| MOD-03 | User can switch models mid-session anytime; the next turn uses the newly selected model | Existing Action→Effect→`session/set_model`→`SetSessionModel` path already applies sampling for next sample (Phase 4). Confirm success path + scrollback `Switched to {name}`; no process restart; mid-turn allowed without cancel; history preserved on `handle_set_session_model`. |
| MOD-06 | Selecting a model whose provider has no usable credentials blocks the switch and prompts that provider’s login | New shell pre-check in `model_switch::apply` + typed `MODEL_SWITCH_MISSING_PROVIDER`; pager QuestionView Login now / Keep current model; CLI suggestion always present; badge on unusable-provider rows; no silent 401 as primary. |
</phase_requirements>

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Missing-provider policy (block switch) | API / Backend (shell `MvpAgent` / `model_switch::apply`) | — | Single authority for TUI + ACP + headless; cannot diverge |
| Resolve model → provider | API / Backend (catalog `ModelEntry.info.provider`) | — | Phase 3/4 catalog authority (`xai` \| `codex`) |
| Usable credentials query | API / Backend (auth store / status) | Browser/Client (badge display only) | Disk + live slot truth lives under `$BUM_HOME/auth.json` |
| Apply model + rebuild SamplingConfig | API / Backend (session actor) | — | Next-turn route rebuild already implemented |
| ACP typed error surface | API / Backend | Frontend (TUI map) | Machine-readable for all ACP clients |
| QuestionView gate modal | Browser / Client (pager TUI) | — | LocalQuestionKind; pure dispatch |
| Deferred switch + post-login retry | Browser / Client (pager session state) | Shell (re-apply set_model) | Stash target model/effort; re-issue Effect after auth |
| Picker `needs login` badge | Browser / Client (slash/model list) | Shell (optional auth snapshot API) | Full mixed list; light cue only |
| Mid-turn switch (next-turn only) | API / Backend (sampling state) | Browser/Client (no cancel UI) | Session does not abort running turn on model switch |
| Login recovery (provider OAuth) | API / Backend (Phase 5 CLI/auth) | Browser/Client (TUI login detour) | Codex primarily CLI today; TUI Login now depth is discretion |

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Rust edition 2024 / rustc 1.92.0 | pinned `rust-toolchain.toml` | Product language | Existing workspace — no rewrite |
| Tokio 1 | workspace | Async apply / ACP / effects | Existing agent/TUI loop |
| `agent-client-protocol` (ACP) 0.10.4 | workspace | `session/set_model`, structured errors | Switch transport already ACP |
| ratatui 0.29 + crossterm 0.28 | workspace | QuestionView / picker UI | Existing TUI chrome |
| serde / serde_json | workspace | Typed ACP error payloads (`camelCase`) | Mirror `ModelSwitchIncompatibleAgentError` |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| thiserror / anyhow | workspace | Domain vs orchestration errors | Shell libraries vs pager edges |
| insta | pager dev-dep | Snapshot QuestionView/copy if needed | Optional UI snapshots |
| serial_test | shell/pager | Env/auth file isolation in tests | Auth fixture tests |
| cargo test (per-crate) | built-in | Unit + integration | Prefer `-p xai-grok-shell` / `-p xai-grok-pager` |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Shell-authoritative gate | Pager-only pre-check before Effect | Faster UX but ACP/headless bypass — **rejected** by CONTEXT |
| Gate only at sample time | Switch-time block | Violates MOD-06 (401 as primary) — **rejected** |
| Collapse into `IncompatibleAgent` | Separate typed error | Wrong recovery UX (new session vs login) — **rejected** |
| Filter catalog by auth | Badge + full list | Conflicts Phase 3 mixed list — **rejected** |

**Installation:** none — no new crates/packages for this phase.

**Version verification:** stack versions from workspace manifests / `rustc --version` (1.92.0) on research host `[VERIFIED: local toolchain]`.

## Package Legitimacy Audit

> No external packages are installed for this phase (in-tree Rust only).

| Package | Registry | Age | Downloads | Source Repo | Verdict | Disposition |
|---------|----------|-----|-----------|-------------|---------|-------------|
| — | — | — | — | — | N/A | No installs |

**Packages removed due to [SLOP] verdict:** none  
**Packages flagged as suspicious [SUS]:** none

## Architecture Patterns

### System Architecture Diagram

```text
[User] ──/model | picker | settings | ACP session/set_model──►
        │
        ▼
  [Pager dispatch]  Action::SwitchModel / SetDefaultModel
        │  (pure; no auth I/O)
        │  model_switch_pending = true
        │  (settings may optimistically set_current today — MUST NOT for missing-provider)
        ▼
  [Effect::SwitchModel] ──ACP──► MvpAgent::set_session_model
                                      │
                                      ├─ allowed_models gate (existing)
                                      ▼
                              model_switch::apply  ◄── AUTHORITATIVE GATE HERE
                                      │
                      ┌───────────────┼──────────────────────────────┐
                      │               │                              │
                      ▼               ▼                              ▼
            missing provider?   agent-type mismatch?          prepare + SetSessionModel
            MODEL_SWITCH_       MODEL_SWITCH_                 success → ModelChanged
            MISSING_PROVIDER    INCOMPATIBLE_AGENT            broadcast + telemetry
                      │               │
                      ▼               ▼
              [SwitchModelComplete Err]
                      │
        ┌─────────────┴──────────────────┐
        ▼                                ▼
 MissingProvider → QuestionView     IncompatibleAgent → QuestionView
 Login now | Keep current           Yes (new session) | No
        │
        ├─ Keep current: clear intent; stay on prev model
        └─ Login now: deferred_model_switch = (model, effort)
                      → Phase 5 login (provider-scoped)
                      → AuthComplete → re-Effect::SwitchModel
```

### Recommended Project Structure (touch surface)

```text
crates/codegen/xai-grok-shell/src/
├── agent/handlers/model_switch.rs     # early missing-provider gate
├── agent/config.rs                    # MODEL_SWITCH_MISSING_PROVIDER + error type
├── auth/status.rs                     # reuse store_usable / AuthStatusReport
└── (optional) agent/mvp_agent/…       # thin helper: provider_slot_usable(model)

crates/codegen/xai-grok-pager/src/
├── app/actions.rs                     # SwitchModelError::MissingProvider
├── app/effects/mod.rs                 # deserialize typed ACP error
├── app/dispatch/session/lifecycle.rs  # handle_switch_model_complete + open_*_question
├── app/dispatch/settings/setters.rs   # no optimistic current for blocked class
├── app/dispatch/auth.rs               # AuthComplete → apply deferred_model_switch
├── app/agent_view/mod.rs              # LocalQuestionKind → Action
├── views/question_view.rs             # LocalQuestionKind::MissingProviderLogin
└── slash/commands/model.rs            # build_model_items badge
```

### Pattern 1: Typed ACP model-switch errors (mirror IncompatibleAgent)

**What:** Stable `code` string + camelCase JSON in `acp::Error.data`; pager deserializes by code.  
**When to use:** Any switch rejection that needs structured client recovery (not string parse).  
**Example (existing pattern to copy):**

```rust
// Source: crates/codegen/xai-grok-shell/src/agent/config.rs (ModelSwitchIncompatibleAgentError)
pub const MODEL_SWITCH_MISSING_PROVIDER: &str = "MODEL_SWITCH_MISSING_PROVIDER";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ModelSwitchMissingProviderError {
    pub code: String,           // always MODEL_SWITCH_MISSING_PROVIDER
    pub provider: String,       // "xai" | "codex"
    pub model_id: String,
    pub suggestion: String,     // "bum login --provider {id}"
}

impl ModelSwitchMissingProviderError {
    pub fn into_acp_error(self) -> acp::Error { /* message + .data(serde) */ }
    pub fn from_acp_error(err: &acp::Error) -> Option<Self> { /* code match */ }
}
```

### Pattern 2: LocalQuestionKind modal recovery

**What:** `QuestionViewState::with_local_kind` + `with_no_freeform`; submit maps option index → Action.  
**When to use:** Gate recovery needing two explicit actions (not toast-only).  
**Mirror:** `open_agent_type_mismatch_question` in `lifecycle.rs` + `LocalQuestionKind::AgentTypeMismatch` handler in `agent_view/mod.rs`.

### Pattern 3: deferred_model_switch

**What:** `AgentSession.deferred_model_switch: Option<(ModelId, Option<ReasoningEffort>)>` applied when session becomes ready or after reconnect/login.  
**When to use:** Target selected before session id exists, after agent-type new-session, or **post-login retry** (Phase 6).  
**Key:** `take_deferred_model_switch` / `apply_deferred_model_switch` in `lifecycle.rs`; set `model_switch_pending` when re-issuing Effect.

### Pattern 4: TEA purity

**What:** Dispatch stays free of network/FS; auth/badge data must be snapshotted into AppView or queried only in effects.  
**When to use:** Always for pager state mutations.

### Anti-Patterns to Avoid

- **Pager-only gate without shell check:** ACP clients bypass.  
- **Optimistic `models.current` on missing-provider:** UI-SPEC forbids flash of forbidden model (stricter than IncompatibleAgent rollback). Settings path today *does* optimistic `set_current` — must not leave target as current when Err is MissingProvider.  
- **Collapsing error codes:** `IncompatibleAgent` vs `MissingProvider` require different QuestionViews.  
- **Filtering GPT rows when logged out:** Phase 3 violation.  
- **Using `AuthManager::has_usable_token` alone for Codex:** AuthManager is xAI-scoped; Codex usability is store-based (`providers.codex` + `store_usable` / prepare snapshot).  
- **Cancelling in-flight turn on switch:** Not default this phase.  
- **Logging tokens / suggestion with secrets:** Never.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Dual-provider usable check | New ad-hoc token parser | `credential_usable` / `store_usable` / `AuthStatusReport` | Phase 5 already defines refreshable = usable |
| Typed ACP error plumbing | String-match `e.to_string()` | Mirror `ModelSwitchIncompatibleAgentError` | Stable codes + camelCase data |
| Gate modal chrome | New modal system | `QuestionView` + `LocalQuestionKind` | Same family as agent-type mismatch |
| Post-login model re-pick | Force user re-open picker | `deferred_model_switch` | Already used for CLI `-m` / session-created |
| Provider labels / CLI ids | Invent new names | `AuthProvider::label` / `as_str` | `xAI`/`Codex`, `xai`/`codex` |
| Catalog provider on client | Guess from model id string | ACP meta `provider` from `to_acp_model_info` | Already stamped Phase 3/4 |

**Key insight:** Almost all infrastructure exists; Phase 6 is **wiring + one policy check + one new typed error path**, not a new auth system.

## Common Pitfalls

### Pitfall 1: Gate after prepare/apply side effects
**What goes wrong:** Sampling config or `models.current` partially updates, then fails.  
**Why it happens:** Gate placed after `prepare_prepared_sampling_config` or after session command.  
**How to avoid:** Run missing-provider check immediately after `resolve_model_id` (and optional BYOK short-circuit), **before** agent-type rebuild and `SetSessionModel`.  
**Warning signs:** Tests see ModelChanged broadcast on blocked switch.

### Pitfall 2: xAI-only usable API for both providers
**What goes wrong:** Codex models always pass or always fail incorrectly.  
**Why it happens:** `AuthManager::has_usable_token` reflects xAI manager scope.  
**How to avoid:** Resolve target `ModelProvider` → read that slot via `read_provider_auth_store` / `AuthStatusReport` / pure `store_usable`. For xAI, live AuthManager + disk can both count (match status semantics: refreshable OK).  
**Warning signs:** Gate tests only inject xAI tokens.

### Pitfall 3: Settings optimistic current vs MissingProvider
**What goes wrong:** UI shows GPT model name while still on Grok backend after block.  
**Why it happens:** `set_default_model` calls `set_current` before Effect; `handle_switch_model_complete` only rolls back on `IncompatibleAgent` when `prev_model_id` is Some.  
**How to avoid:** On `MissingProvider`, always restore previous model if optimistic, **or** stop optimistically setting current for session switch until success (preferred for this error class).  
**Warning signs:** Prompt chrome shows target name after gate modal opens.

### Pitfall 4: Login now without deferred retry
**What goes wrong:** User logs in, returns to session, still on old model; must re-pick.  
**Why it happens:** `handle_auth_complete` only retries `reauth_stashed_prompt`, not `deferred_model_switch`.  
**How to avoid:** On Login now, set `deferred_model_switch`; on AuthComplete mid-session return, apply deferred switch once credentials usable.  
**Warning signs:** Manual re-`/model` required after gate login.

### Pitfall 5: TUI Login is still xAI-only
**What goes wrong:** “Login now” for Codex starts wrong OAuth or no-ops.  
**Why it happens:** `Action::Login` / `dispatch_login` uses interactive grok.com method only; Phase 5 Codex is primarily `bum login --provider codex`.  
**How to avoid:** Prefer provider-aware login action or CLI-first recovery: always show CLI in option description; for Login now either (a) spawn/document CLI path + poll auth file until usable then retry, or (b) extend TUI authenticate effect for Codex PKCE if Phase 5 left a hook. Do not claim Codex browser TUI if unbuilt.  
**Warning signs:** Gate Login now authenticates xAI when target was Codex.

### Pitfall 6: Badge needs provider + usability without FS in suggest_args
**What goes wrong:** Dispatch purity violated or badges always stale.  
**Why it happens:** `build_model_items` only has `ModelState`; no auth snapshot.  
**How to avoid:** Cache dual-slot usable flags on AppView (refresh on AuthComplete, logout, startup, periodic); pass into suggest context or read from app-level cache. Prefer display suffix ` · needs login` if `ArgItem`/`SuggestionRow` lack `right_label` (today they do not have `right_label` — only modal `PickerRow` does).  
**Warning signs:** `suggest_args` opens auth.json on every keystroke.

### Pitfall 7: BYOK / model-owned credentials
**What goes wrong:** Gate blocks models with `api_key`/`env_key` when OAuth slot empty.  
**Why it happens:** Only checking provider OAuth store.  
**How to avoid:** Treat `ModelEntry::has_own_credentials()` as usable for gate purposes (route already uses own key). Confirm against Phase 4 BYOK tests.  
**Warning signs:** Custom OpenAI base + env_key cannot switch.

### Pitfall 8: Internal `apply` callers (new_session / load_session)
**What goes wrong:** Resume/load of session whose model provider logged out fails hard unexpectedly — or worse, silently continues.  
**Why it happens:** `apply` is shared.  
**How to avoid:** Decide explicitly: user-driven `set_session_model` always gates; bootstrap may log + keep model but sample fails secondary — **recommend gate on user set_model path only** via `set_session_model` wrapper, with pure helper shared, if load_session must not open login UX. Prefer: **gate inside `apply` always** for fail-closed (MOD-06 spirit); load_session error surfaces as known session error already. Document in plan.  
**Warning signs:** Session resume opens TUI gate modal.

### Pitfall 9: `model_switch_pending` blocks prompt drain
**What goes wrong:** While gate modal open, queue stalls if pending left true.  
**Why it happens:** `handle_switch_model_complete` clears pending on complete; if error path forgets, stuck.  
**How to avoid:** Always clear `model_switch_pending` in complete handler (already does); do not set pending for pure client-side modal after Err.  
**Warning signs:** Prompts not sending after dismissed gate.

## Code Examples

### Gate placement (prescriptive)

```rust
// Source: planned insertion in model_switch::apply after resolve_model_id
let model = agent.resolve_model_id(&model_id)?;
let provider = model.info.provider; // ModelProvider::Xai | Codex

if !model.has_own_credentials() && !provider_slot_usable(agent, provider) {
    let provider_id = provider.as_str(); // "xai" | "codex"
    let err = config::ModelSwitchMissingProviderError {
        code: config::MODEL_SWITCH_MISSING_PROVIDER.to_string(),
        provider: provider_id.to_owned(),
        model_id: model_id.0.to_string(),
        suggestion: format!("bum login --provider {provider_id}"),
    };
    // telemetry ModelSwitched success:false error_code MISSING_PROVIDER
    return Err(err.into_acp_error());
}
// then existing agent-type mismatch checks…
```

### Usable semantics (Phase 5 — prefer reuse)

```rust
// Source: crates/codegen/xai-grok-shell/src/auth/status.rs
// usable = hard-unexpired access OR nonblank refresh_token (refreshable)
pub fn credential_usable(auth: &GrokAuth) -> bool { /* … */ }
pub fn store_usable(store: &AuthStore) -> bool { /* any non-WebLogin credential_usable */ }
```

`[VERIFIED: codebase auth/status.rs]` — expired-but-refreshable is usable; hard-expired without refresh is not.

### Pager error map

```rust
// Source: crates/codegen/xai-grok-pager/src/app/effects/mod.rs (extend existing match)
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
```

### QuestionView open (copy contract from 06-UI-SPEC)

```text
Question: Sign in to {ProviderLabel} to use {ModelDisplayName}.
Option 1: Login now — Run login for {ProviderLabel} (CLI: bum login --provider {id})
Option 2: Keep current model — Dismiss and stay on the active model
LocalQuestionKind::MissingProviderLogin { model_id, effort, provider }
```

### Badge on `/model` rows

```rust
// Prefer until ArgItem gains right_label:
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
// needs_login from AppView dual-slot usable cache + meta.provider
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Single xAI auth | Dual-slot `providers.xai` / `providers.codex` | Phase 2/5 | Gate must be per-provider |
| No catalog provider | `provider` on ModelEntry + ACP meta | Phase 3/4 | Client can badge by meta |
| Routing ungated / fake tokens OK | Provider route + prepare sampling | Phase 4 | Switch changes next sample only |
| Full Codex OAuth lifecycle | CLI login/logout/status/refresh | Phase 5 | Usable definitions + CLI recovery exist |
| Switch ungated on auth | **Phase 6 gate** | this phase | Block + login prompt primary UX |

**Deprecated/outdated:**
- Silent mid-turn 401 as primary missing-login UX for model selection  
- Treating all models as xAI credentials for switch validation  

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Login now for Codex may remain CLI-primary if TUI Codex OAuth is not fully wired; gate still succeeds with CLI fallback + deferred retry after external login | Login recovery | Planner over-scopes TUI Codex PKCE in Phase 6 |
| A2 | Putting gate in `apply` is correct for all user-driven switches; bootstrap/load may need explicit decision if gate breaks resume | Pitfall 8 | Resume regressions |
| A3 | `has_own_credentials` should skip OAuth-slot gate | Gate logic | Blocks legitimate BYOK |
| A4 | Display suffix ` · needs login` is acceptable MVP vs full `right_label` plumbing on slash dropdown | Badge | Slightly less polished than UI-SPEC preferred right-label |

**If this table is empty:** N/A — several discretion items remain.

## Open Questions

1. **Login now depth for Codex in TUI**  
   - What we know: Phase 5 shipped CLI Codex OAuth; TUI `Action::Login` is xAI interactive.  
   - What's unclear: Whether Phase 6 must implement full Codex browser detour in TUI or CLI + poll is enough.  
   - Recommendation: Ship CLI-always path + deferred auto-retry; wire TUI provider login only if a thin Effect can call existing shell Codex login without large scope.

2. **Auth usability snapshot for badges**  
   - What we know: Pager cannot hit FS in pure suggest_args.  
   - What's unclear: Existing AppView field for dual-slot usable flags?  
   - Recommendation: Add lightweight `provider_auth_usable: [bool; 2]` (or map) refreshed on auth lifecycle; optional ACP ext-method later.

3. **Bootstrap apply gate**  
   - What we know: `new_session` / `load_session` call `apply`.  
   - What's unclear: Should load of session on missing provider error the load or allow with degraded sample?  
   - Recommendation: Gate user `set_session_model` path hard; for load, fail closed on sample is secondary (not MOD-06 primary) — prefer not opening QuestionView on load.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| rustc / cargo | Build & tests | ✓ | 1.92.0 / 1.92.0 | — |
| node (gsd-tools commit) | Docs commit | ✓ | v22.22.1 | — |
| Live ChatGPT / xAI OAuth | Manual smoke only | optional | — | Fixture tokens + mock store for automated tests |
| Docker | — | N/A | — | Not required |

**Missing dependencies with no fallback:** none for implementation/tests.  
**Missing dependencies with fallback:** live OAuth → fixture `auth.json` + pure unit tests.

Step 2.6: external services not required for automated gate verification (Phase 4/5 pattern).

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | cargo test (crate-local); insta (pager); serial_test (auth env) |
| Config file | per-crate `Cargo.toml` / workspace |
| Quick run command | `cargo test -p xai-grok-shell --lib model_switch -- --nocapture` (after tests named) |
| Full suite command | `cargo test -p xai-grok-shell --test provider_routing --test auth_codex_lifecycle` + `cargo test -p xai-grok-pager switch_model` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| MOD-06 | Missing Codex blocks switch to GPT model | unit/integration shell | `cargo test -p xai-grok-shell missing_provider` | ❌ Wave 0 |
| MOD-06 | Typed ACP error code + suggestion | unit shell | `cargo test -p xai-grok-shell model_switch_missing_provider_error` | ❌ Wave 0 |
| MOD-06 | Same-provider switch with usable creds succeeds | unit shell | (extend provider_routing / model_switch tests) | ⚠️ partial (routing exists) |
| MOD-03 | Successful switch updates next sample route | integration | existing `switch_changes_next_sample_route` in `provider_routing.rs` | ✅ |
| MOD-03 | Mid-session switch success scrollback | unit pager | extend `dispatch/tests/task_result.rs` | ✅ pattern |
| MOD-06 | SwitchModelComplete → QuestionView MissingProvider | unit pager | extend `task_result.rs` | ❌ Wave 0 |
| MOD-06 | Keep current clears deferred; no model apply | unit pager | new lifecycle tests | ❌ Wave 0 |
| MOD-06 | Login now sets deferred_model_switch | unit pager | new lifecycle/auth tests | ❌ Wave 0 |
| MOD-06 | AuthComplete applies deferred switch | unit pager | extend `dispatch/auth` tests | ❌ Wave 0 |
| MOD-06 | No optimistic current on MissingProvider (settings) | unit pager | extend settings tests | ❌ Wave 0 |
| MOD-06 | Badge `needs login` on unusable provider rows | unit pager | `slash/commands/model.rs` tests | ❌ Wave 0 |
| MOD-03/06 | Both providers logged in: Grok↔GPT switch free | integration shell | fixture dual tokens | ❌ Wave 0 |
| MOD-06 | BYOK skips OAuth gate | unit shell | has_own_credentials fixture | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** targeted `cargo test -p <crate> <filter>` for touched module  
- **Per wave merge:** shell model_switch + pager switch_model/task_result suites green  
- **Phase gate:** both crates green for new tests + no regression on `provider_routing` / IncompatibleAgent tests  

### Wave 0 Gaps

- [ ] Shell unit: pure `provider_slot_usable` / gate decision table (xai/codex × empty/usable/refreshable/expired-no-refresh/BYOK)
- [ ] Shell: `ModelSwitchMissingProviderError` serde + `from_acp_error` round-trip
- [ ] Shell integration: `model_switch::apply` returns missing-provider when Codex empty; succeeds with fixture Codex token
- [ ] Pager: `SwitchModelError::MissingProvider` mapping + QuestionView open (no set_current)
- [ ] Pager: deferred retry after simulated AuthComplete
- [ ] Pager: `build_model_items` badge when provider unusable
- [ ] Ensure existing IncompatibleAgent tests still pass (no collapse)

*(Existing infrastructure: `dispatch/tests/task_result.rs` IncompatibleAgent suite is the template; `auth/status.rs` pure tests for usable semantics; `provider_routing.rs` for next-sample route.)*

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | yes | Dual-slot OAuth; gate uses usable-slot check only |
| V3 Session Management | yes | Session model id update only after gate pass |
| V4 Access Control | yes | Provider credentials never cross-slot (Phase 4) |
| V5 Input Validation | yes | Model id via catalog resolve; provider allow-list `xai`\|`codex` |
| V6 Cryptography | no new | No new crypto; do not hand-roll tokens |

### Known Threat Patterns for dual-provider model switch

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Switch to GPT with empty Codex → silent 401 | Information disclosure / DoS UX | Switch-time gate + typed error |
| Cross-slot credential use | Elevation | Existing Phase 4 resolver; gate does not inject tokens |
| Error message leaks tokens | Information disclosure | Sanitize; suggestion is CLI only; never print auth.json |
| Client spoofs provider meta | Tampering | Gate uses in-process catalog `ModelEntry.provider`, not client-supplied provider |
| ACP client skips TUI modal | Bypass | Shell authoritative gate |

## Current Architecture Deep Dive (planner reference)

### Shell model switch

| Piece | Path | Role |
|-------|------|------|
| ACP entry | `mvp_agent/acp_agent.rs` `set_session_model` | `allowed_models` then `model_switch::apply` |
| Apply | `agent/handlers/model_switch.rs` | Resolve model; agent-type gate; prepare sampling; `SessionCommand::SetSessionModel`; broadcast `ModelChanged` |
| Session apply | `session/acp_session_impl/model_switch.rs` | Writes prepared sampling + credentials fields; does **not** rewrite conversation history |
| Errors | `agent/config.rs` | `MODEL_SWITCH_INCOMPATIBLE_AGENT`, `MODEL_SWITCH_REBUILD_FAILED` |
| Prepare auth | `mvp_agent/agent_ops.rs` `prepare_prepared_sampling_config_for_model` | xAI via AuthManager; Codex via `snapshot_codex_session_key_from_auth_store` — **warns** if no key, does not block switch |

**Gap:** no missing-provider return before apply. `[VERIFIED: codebase model_switch.rs]`

### Pager model switch

| Piece | Path | Role |
|-------|------|------|
| Actions | `SwitchModel`, `SetDefaultModel` | User intent |
| Router | `dispatch/router.rs` | Sets `model_switch_pending` or `deferred_model_switch` if no session_id; **no** optimistic set_current on SwitchModel |
| Settings | `dispatch/settings/setters.rs` | **Does** optimistic `set_current` + `prev_model_id` for rollback |
| Effect | `effects/mod.rs` | ACP set_model; maps IncompatibleAgent only |
| Complete | `lifecycle.rs` `handle_switch_model_complete` | Success → set_current + `Switched to {name}`; IncompatibleAgent → rollback + QuestionView; Other → scrollback error |
| Agent type modal | `open_agent_type_mismatch_question` | Pattern for MissingProvider modal |
| Pending flag | `model_switch_pending` | Blocks some prompt queue paths while switch in flight |

### Usable credentials (authoritative definitions)

| API | Scope | Semantics |
|-----|-------|-----------|
| `AuthManager::has_usable_token` | Per AuthManager (xAI product path) | Memory or disk; hard-unexpired (early buffer still usable) |
| `credential_usable` / `store_usable` | Per provider store | Unexpired access **or** refresh_token present |
| `AuthStatusReport` | Both slots | CLI status; paste-safe |
| `snapshot_codex_session_key_from_auth_store` | Codex prepare | Nonblank selected access token snapshot |

**Gate recommendation:** For provider P, usable iff `store_usable(P)` (or live xAI AuthManager equivalent for xAI) **OR** target model `has_own_credentials()`. Do not require non-expired access if refreshable. `[VERIFIED: codebase auth/status.rs + manager.rs]`

### Where to insert the gate (single check)

1. **Preferred:** Start of `model_switch::apply` after `resolve_model_id`, before agent-type / prepare / session command.  
2. **Also gate:** `MvpAgent::set_session_model` is the public user path (already funnels to apply).  
3. **Do not rely on:** pager pre-check alone; prepare-time warn logs; sample-time 401.

### UI wiring summary (UI-SPEC aligned)

| Surface | Behavior |
|---------|----------|
| QuestionView | New `MissingProviderLogin`; separate from `AgentTypeMismatch` |
| Copy | Exact templates in `06-UI-SPEC.md` |
| deferred_model_switch | Stash on Login now; apply on AuthComplete |
| Badge | `needs login` exact; full list remains |
| Success | Existing `Switched to {ModelDisplayName}` |
| ACP | `MODEL_SWITCH_MISSING_PROVIDER` + suggestion field |

### Concrete file touch list

**Shell (authority + typed error)**  
1. `crates/codegen/xai-grok-shell/src/agent/handlers/model_switch.rs` — gate  
2. `crates/codegen/xai-grok-shell/src/agent/config.rs` — const + struct + tests  
3. `crates/codegen/xai-grok-shell/src/auth/status.rs` or thin helper re-export — `provider_slot_usable` if needed  
4. `crates/codegen/xai-grok-shell/tests/` — new or extend model switch / dual auth tests  

**Pager (UX)**  
5. `crates/codegen/xai-grok-pager/src/app/actions.rs` — `SwitchModelError::MissingProvider`; maybe `MissingProviderLoginAnswered` Action  
6. `crates/codegen/xai-grok-pager/src/app/effects/mod.rs` — deserialize  
7. `crates/codegen/xai-grok-pager/src/app/dispatch/session/lifecycle.rs` — complete handler + open modal + answer dispatch  
8. `crates/codegen/xai-grok-pager/src/views/question_view.rs` — `LocalQuestionKind` variant  
9. `crates/codegen/xai-grok-pager/src/app/agent_view/mod.rs` — submit → Action  
10. `crates/codegen/xai-grok-pager/src/app/dispatch/router.rs` — route new Action  
11. `crates/codegen/xai-grok-pager/src/app/dispatch/auth.rs` — AuthComplete deferred switch  
12. `crates/codegen/xai-grok-pager/src/app/dispatch/settings/setters.rs` — optimistic current policy  
13. `crates/codegen/xai-grok-pager/src/slash/commands/model.rs` — badge  
14. Possibly `slash/command.rs` / `slash/mod.rs` if adding `right_label` field  
15. Tests under `dispatch/tests/task_result.rs`, `lifecycle.rs`, `settings.rs`, model command tests  

**Explicit non-touches (Phase 7/8)**  
- Subagent spawn credential inheritance  
- Full chrome rebrand strings  

## Project Constraints (from CLAUDE.md / AGENTS.md)

- Stay on Rust workspace fork harness — no rewrite  
- Dual identity under `~/.bum` only; no stock credential import  
- Per-model routing (not global provider mode)  
- No xAI auto-update / product telemetry as product goals (later phases)  
- Dispatch purity: no I/O in pager dispatch  
- Prefer `cargo test -p <crate>`; full workspace builds are heavy  
- Secrets never logged or shown  

## Sources

### Primary (HIGH confidence)
- Codebase: `model_switch.rs`, `config.rs` error types, `lifecycle.rs`, `effects/mod.rs`, `auth/status.rs`, `auth/manager.rs`, `to_acp_model_info`, `prepare_prepared_sampling_config_for_model`  
- `.planning/phases/06-…/06-CONTEXT.md`, `06-UI-SPEC.md`  
- Phase 4/5 CONTEXT + REQUIREMENTS MOD-03/06  

### Secondary (MEDIUM confidence)
- Phase 5 summaries (TUI login depth still CLI-primary for Codex selective ops)  

### Tertiary (LOW confidence)
- Exact Login now implementation depth for Codex in-TUI OAuth (A1)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new packages; existing harness  
- Architecture: HIGH — end-to-end switch path read in tree  
- Pitfalls: HIGH — optimistic settings path, xAI-only usable, deferred retry gap verified in code  
- Login now TUI depth: MEDIUM — product discretion  

**Research date:** 2026-07-17  
**Valid until:** 2026-08-16 (stable internal APIs; re-check if Phase 5 login surface expands)

## RESEARCH COMPLETE
