---
phase: 07-cross-provider-multi-agent-orchestration
reviewed: 2026-07-17T09:35:48Z
depth: standard
files_reviewed: 11
files_reviewed_list:
  - crates/common/xai-tool-types/src/task.rs
  - crates/codegen/xai-grok-tools/src/implementations/grok_build/task/mod.rs
  - crates/codegen/xai-grok-tools/src/implementations/grok_build/task/types.rs
  - crates/codegen/xai-grok-tools/src/implementations/grok_build/task/backend.rs
  - crates/codegen/xai-grok-agent/src/builder.rs
  - crates/codegen/xai-grok-shell/src/agent/config.rs
  - crates/codegen/xai-grok-shell/src/agent/handlers/model_switch.rs
  - crates/codegen/xai-grok-shell/src/agent/subagent/handle_request.rs
  - crates/codegen/xai-grok-shell/src/agent/subagent/mod.rs
  - crates/codegen/xai-grok-shell/src/agent/mvp_agent/subagent_coordinator.rs
  - crates/codegen/xai-grok-shell/src/test_support/lsp_runtime.rs
findings:
  critical: 1
  warning: 3
  info: 2
  total: 6
status: findings
---

# Phase 7: Code Review Report

**Reviewed:** 2026-07-17T09:35:48Z  
**Depth:** standard  
**Files Reviewed:** 11  
**Status:** findings  

## Summary

Adversarial review of Phase 7 product changes (plans 01–06): Task `reasoning_effort` schema/wire, async `preflight_spawn`, missing-provider spawn gate, dual-slot credential resolve, and isolation seams.

**What looks solid**

- Task vocabulary parse is fail-closed before spawn (`parse_task_reasoning_effort` + allowlist; blank/`null`/`undefined` omitted; `max` → `xhigh`).
- Async Task preflight is real coordinator RPC (not a sync slug-only Fn); `Unavailable` / `Denied` block the background “started” notice.
- Spawn gate runs **before** `insert_pending` / worktree on the shell path; pure `oauth_provider_slot_usable` is shared with model switch; Codex is never inferred from live xAI alone.
- `resolve_model_override_to_config` isolates slots (`Xai → (xai_key, None)`, `Codex → (None, codex_key)`); no parent bearer rewrite into the child provider slot on the explicit-model path.
- Plan 05 isolation harness exercises wire `Authorization` both directions with dual fake tokens.

**Key concerns**

1. **Effort fail-closed provenance is wrong on the live Task path** — role/persona/definition effort is treated as Tool effort because Task always stamps `ModelOverrideProvenance::Tool`. That contradicts the Plan 03 “role/harness soft-skip” contract and can abort otherwise-valid spawns.
2. Effort **support** is checked only after pending/worktree (and after bg “started”), so fail-closed is incomplete for background Task.
3. Subagent spawn still logs **API-key prefixes at info** — conflicts with Phase 7 “secrets not logged” bar.
4. Residual parent-model fallback after the credential gate remains a latent cross-provider footgun if catalog identity ever diverges.

---

## Critical Issues

### CR-01: Role/persona/definition effort hard-fails under Task provenance

**File:** `crates/codegen/xai-grok-shell/src/agent/subagent/handle_request.rs:691-700`  
**Also:** `crates/codegen/xai-grok-tools/src/implementations/grok_build/task/mod.rs:359-361`  
**Severity:** BLOCKER (incorrect behavior vs D-04 / Plan 03 soft-skip contract)

**Issue:**  
`apply_subagent_reasoning_effort` chooses hard-fail vs soft-skip from `request.runtime_overrides.model_override_provenance`. Every model-facing Task call sets:

```rust
model_override_provenance: ModelOverrideProvenance::Tool,
```

even when `reasoning_effort` is **omitted**. Effective effort is then filled from role / persona / `AgentDefinition.effort`:

```rust
// resolve_effective_overrides merges role/persona effort
// then definition.effort fills remaining None
if effective_runtime.reasoning_effort.is_none() {
    effective_runtime.reasoning_effort = definition.effort.map(|e| <&str>::from(e).to_string());
}
// ...
apply_subagent_reasoning_effort(
    raw,
    ...,
    request.runtime_overrides.model_override_provenance, // always Tool for Task
    ...
)
```

Unit tests document the intended split (`p7_invalid_effort_unsupported_model_tool_fails_closed` vs `p7_invalid_effort_harness_soft_skips_unsupported` as “role default path”), but production **never** passes `Harness` for role-sourced effort on Task spawns. Result: a Task spawn onto a model that does not support reasoning effort will **abort** if any role/persona/definition supplies a default effort — even when the model did not request `reasoning_effort`.

**Fix:** Derive effort provenance from whether Task (or harness) **explicitly** set effort, not from model provenance:

```rust
// When applying effective effort:
let effort_provenance = if request.runtime_overrides.reasoning_effort.is_some() {
    // Explicit Task/harness override — keep Tool vs Harness semantics.
    request.runtime_overrides.model_override_provenance
} else {
    // Role / persona / AgentDefinition defaults — soft-skip if unsupported.
    ModelOverrideProvenance::Harness
};
match apply_subagent_reasoning_effort(
    raw,
    effective_model_id.0.as_ref(),
    supports,
    effort_provenance,
    &mut effective_sampling_config,
) { /* ... */ }
```

Add an integration-style unit test: Task provenance + omitted effort + role `reasoning_effort=high` + `supports=false` → soft-skip (spawn continues), whereas explicit Task `reasoning_effort=high` + `supports=false` → hard-fail.

---

## Warnings

### WR-01: Effort support fail-closed runs after pending/worktree (and after bg “started”)

**File:** `crates/codegen/xai-grok-shell/src/agent/subagent/handle_request.rs:435-459` (gate + `insert_pending`) vs `:691-707` (effort apply)  
**Also:** `crates/codegen/xai-grok-tools/src/implementations/grok_build/task/mod.rs:372-465` (await preflight, then fire-and-forget + return started)

**Issue:**  
Missing-provider credentials are gated before side effects (correct). Explicit Tool effort on an **unsupported** model is only detected later — after `insert_pending`, worktree creation, and (on the Task background path) after the tool has already returned the “started” notice. Preflight covers credentials/effective model only, not effort×model support.

**Fix:**  
Either:

1. Extend `preflight_subagent_spawn` / `SubagentPreflightInput` to validate explicit Tool effort against `models_manager.model_supports_reasoning_effort(effective_model)` and return `Denied` before Task returns started; or  
2. Move effort support application to the same pre-pending block as the credential gate (still before `insert_pending`).

Prefer (1) so Task bg and shell spawn stay aligned.

### WR-02: Partial bearer tokens logged at `unified_log::info` on every subagent spawn

**File:** `crates/codegen/xai-grok-shell/src/agent/subagent/handle_request.rs:996-1011`  
**Also:** `crates/codegen/xai-grok-shell/src/agent/subagent/mod.rs:865-897`, `:1086-1097`

**Issue:**  
`key_prefix` takes the first **8 characters** of `api_key` and emits them as `key_prefix` / `parent_key_prefix` on **info**-level `subagent spawn credentials` (and debug resolution logs). Phase 7 acceptance explicitly requires secrets not logged. Eight chars of OAuth/JWT-style tokens are still secret material (tests even assert JWT-looking prefixes like `eyJ0eXAi`). Info-level unified logs are more likely to be retained/shipped than debug traces.

**Fix:**

```rust
// Prefer boolean / length-only diagnostics — never raw key material.
"has_child_key": effective_sampling_config.api_key.as_ref().is_some_and(|k| !k.is_empty()),
"has_parent_key": ctx.sampling_config.api_key.as_ref().is_some_and(|k| !k.is_empty()),
"keys_match": effective_sampling_config.api_key == ctx.sampling_config.api_key,
// If a fingerprint is required: blake3/sha256 truncated hex — not key[:8]
```

Drop `key_prefix` from info paths entirely; if debug still needs disambiguation, use a non-reversible hash.

### WR-03: Post-gate “unknown model → parent sampling config” fallback can still re-home child onto parent credentials

**File:** `crates/codegen/xai-grok-shell/src/agent/subagent/handle_request.rs:654-668`

**Issue:**  
After the missing-provider gate passes for `preflight_model_id`, a second resolve may still replace the child’s `SamplerConfig` with the **parent’s** config (api_key + base_url + model) when `effective_sampling_config.model` is not found in `available_models`. For Tool-validated catalog slugs this path should be rare (identity usually matches `info.model`), but it is still a fail-**open** to parent bearer after a child-provider gate — the exact anti-pattern Phase 7 forbids (“no parent bearer fallback”).

**Fix:**  
For Tool provenance (and preferably always after a cross-provider pin), fail closed instead of inheriting parent:

```rust
if model_unknown {
    if request.runtime_overrides.model_override_provenance == ModelOverrideProvenance::Tool
        || effective_runtime.model.is_some()
    {
        let msg = format!(
            "Resolved subagent model '{model_str}' is not in the available catalog; \
             refusing parent credential fallback."
        );
        send_failure(request, &msg);
        return;
    }
    // Same-provider inherit-only legacy path, if still required:
    let (parent_config, parent_mid) = read_parent_sampling_config(&ctx).await;
    effective_sampling_config = parent_config;
    effective_model_id = parent_mid;
}
```

At minimum, re-run `missing_provider_spawn_gate_message` on the **final** `effective_model_id` before `spawn_session_on_thread`.

---

## Info

### IN-01: Preflight returns `Ok` when agent definition is missing

**File:** `crates/codegen/xai-grok-shell/src/agent/subagent/handle_request.rs:161-172`

**Issue:** Unknown `subagent_type` yields `SubagentPreflightOutcome::Ok` (“spawn path will reject”). Task normally validates type first, so production impact is low; any caller that only preflights could still get a green light.

**Fix:** Return `Denied { message: format!("Unknown subagent type: …") }` (or `Unavailable` if preferred) so preflight is a strict superset of pre-spawn rejects where cheap.

### IN-02: Effort level not checked against model’s offered menu

**File:** `crates/codegen/xai-grok-shell/src/agent/subagent/handle_request.rs:191-233`, `:691-694`

**Issue:** Only `model_supports_reasoning_effort` (boolean) is consulted. A model that supports effort but only offers `{low, medium}` will still accept Task `xhigh` and stamp it onto `SamplerConfig`. Parent session switch path has richer `model_offers_reasoning_effort` logic.

**Fix:** For Tool provenance, reject when the parsed `ReasoningEffort` is not in `models_manager.model_reasoning_efforts(model_id)` (when the menu is non-empty).

---

## Areas Reviewed Without Defect

| Area | Verdict |
|------|---------|
| Task `reasoning_effort` schema + sanitize/canonicalize | Correct allowlist; invalid rejected before spawn |
| Async `SubagentBackend::preflight_spawn` | ChannelBackend timeout/fail-closed; coordinator Preflight arm side-effect free |
| Missing-provider gate placement | Before pending/worktree; shared pure helpers; login-shaped message |
| Dual-slot resolve (explicit model) | No cross-slot bearer; Codex never from xAI live alone |
| Plan 05 Authorization isolation tests | Both directions assert fixture tokens; parent model stability covered |
| NL Task description effort guidance | Present after model guidance |

---

## Counts

| Severity | Count |
|----------|------:|
| Critical (P0 / BLOCKER) | 1 |
| Warning (P1) | 3 |
| Info | 2 |
| **Total** | **6** |

**Ship recommendation:** Fix **CR-01** before treating Phase 7 effort fail-closed as complete; address **WR-01**/**WR-02** in the same pass if possible (bg false-started + secret logging).

---

_Reviewed: 2026-07-17T09:35:48Z_  
_Reviewer: Claude (gsd-code-reviewer)_  
_Depth: standard_  
