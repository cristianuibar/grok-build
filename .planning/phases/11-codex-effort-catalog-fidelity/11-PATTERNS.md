# Phase 11: Codex effort & catalog fidelity - Pattern Map

**Mapped:** 2026-07-21
**Files analyzed:** 6 (1 new/modified logic site is a helper function to add; rest are modify-in-place)
**Analogs found:** 6 / 6 (all sites are existing code — this phase modifies established seams, no greenfield files)

All work in this phase edits existing files at a known choke point rather than creating new files. "Analog" below means the existing sibling pattern at the same seam to copy shape from (e.g. the trusted-profile mutation pattern, the system-notice pattern).

## File Classification

| File to modify | Role | Data Flow | Closest analog (same file, existing pattern) | Match Quality |
|---|---|---|---|---|
| `crates/codegen/xai-grok-sampling-types/src/types.rs` | utility (enum + parse) | transform | itself — `FromStr` impl (max→xhigh alias) at lines 829-845 | exact |
| `crates/codegen/xai-grok-sampling-types/src/conversation.rs` | transform (wire builder) | request-response | itself — `From<&ConversationRequest> for rs::CreateResponse`, `reasoning:` field lines 2159-2162 | exact |
| `crates/codegen/xai-grok-sampler/src/client.rs` | service (Codex trusted-profile mutator + its test) | transform | itself — `apply_trusted_codex_response_profile` (1233-1257) + `trusted_codex_responses_profile_on_off_serializes_exactly` test (2475-2507) | exact |
| `crates/codegen/xai-grok-shell/src/agent/handlers/model_switch.rs` | event-driven handler (mid-session model switch) | event-driven | itself — `effort_override` gate (162-181) | exact |
| `crates/codegen/xai-grok-shell/src/agent/models.rs` | service (ModelsManager catalog accessors) | CRUD (read-only lookup) | itself — `model_supports_reasoning_effort` (451-458) / `model_reasoning_efforts` (476-483) | exact |
| `crates/codegen/xai-grok-pager/src/app/dispatch/session/lifecycle.rs` | controller (dispatch effect handler) | event-driven | itself — model-switch scrollback notice (1211-1223) | exact |
| `crates/codegen/xai-grok-shell/tests/model_switch_gate.rs` | test | request-response | itself — `p6_*` naming convention + `trusted_codex_wire_headers_are_sent_and_stable` (1033) | exact |
| `crates/codegen/xai-grok-models/default_models.json` | config (catalog data) | CRUD (static data, read-only) | itself — GPT-5.6 sol/terra/luna `reasoning_efforts` entries (lines 20-125) | exact — no change needed, just reference shape |

## Pattern Assignments

### 1. Clamp rule helper — new function, `types.rs` or shared crate (Claude's Discretion per CONTEXT: pick single choke point)

**Analog:** `crates/codegen/xai-grok-sampling-types/src/types.rs` — existing `FromStr for ReasoningEffort` (lines 829-845) shows the project convention for effort transforms: pure function, no I/O, `ReasoningEffort` in/out.

**Official Codex clamp algorithm to port** (verified exact, `codex/codex-rs/core/src/session/turn_context.rs:243-263`):
```rust
let reasoning_effort = if let Some(current_reasoning_effort) = self.reasoning_effort.clone() {
    if supported_reasoning_levels.contains(&current_reasoning_effort) {
        Some(current_reasoning_effort)
    } else {
        supported_reasoning_levels
            .get(supported_reasoning_levels.len().saturating_sub(1) / 2)
            .cloned()
            .or_else(|| model_info.default_reasoning_level.clone())
    }
} else {
    supported_reasoning_levels
        .get(supported_reasoning_levels.len().saturating_sub(1) / 2)
        .cloned()
        .or_else(|| model_info.default_reasoning_level.clone())
};
```
Port this shape: keep-if-supported, else `list[(len-1)/2]` (middle, lower-biased on even counts), else catalog default. Input data source for `supported_reasoning_levels` in bum is `ModelsManager::model_reasoning_efforts(model_id) -> Vec<ReasoningEffortOption>` (models.rs:476-483) — already exists, read-only, no new plumbing needed to get the list.

**Existing alias to keep, do not touch:**
```rust
// types.rs:839
"xhigh" | "max" => Ok(Self::Xhigh), // max is a CLI/UX alias of xhigh
```

---

### 2. `reasoning.effort` / `reasoning.summary` wire emission — `conversation.rs` (modify in place)

**Analog:** itself, current unconditional emission at lines 2159-2162:
```rust
reasoning: Some(rs::Reasoning {
    effort: req.reasoning_effort.map(|e| e.to_responses_api()),
    summary: Some(rs::ReasoningSummary::Concise),
}),
```
**Target shape (per CONTEXT + RESEARCH A2):** `effort` only when post-clamp value is in the model's supported list (else `None`/omit — struct field already has `Option`, existing `skip_serializing_if` semantics on `rs::Reasoning` do the omission for free once the value is `None`). This is a value-computation change at the call site, not a serde/struct change — same file, same field, same `Option` plumbing already present.

---

### 3. Codex trusted-profile summary override — `client.rs` (modify in place)

**Analog:** itself, current unconditional summary clear at lines 1242-1244:
```rust
if let Some(reasoning) = request.reasoning.as_mut() {
    reasoning.summary = None;
}
```
This already implements "trusted Codex path always omits summary" — CONTEXT's "catalog default `none` → omit" requirement is **already satisfied on the trusted path** by this existing line; no change needed here beyond verifying it still holds once effort clamp lands. Confirmed by test:
```rust
// client.rs:2475
assert!(trusted["reasoning"].get("summary").is_none());
// client.rs:2505 (generic/non-trusted path, contrast)
assert_eq!(generic["reasoning"]["summary"], "concise");
```
If the generic (non-trusted) Responses path also needs `none`→omit alignment for GPT-5.6, that's a second small conditional near this block, following the same `if let Some(reasoning) = request.reasoning.as_mut()` shape.

---

### 4. Mid-session switch: apply clamp instead of silent-drop — `model_switch.rs` (modify in place)

**Analog:** itself, current effort-override gate (lines 162-181):
```rust
let mut prepared =
    agent.prepare_prepared_sampling_config_for_model(&model, handle.origin_client.clone());
if let Some(eff) = effort_override {
    if agent
        .models_manager
        .model_supports_reasoning_effort(model_id.0.as_ref())
    {
        tracing::info!(
            session_id = % session_id.0, effort = % eff,
            "set_session_model: applying reasoning_effort override from meta"
        );
        prepared.sampler_config.reasoning_effort = Some(eff);
    } else {
        tracing::warn!(
            session_id = % session_id.0, model_id = % model_id.0, effort = % eff,
            "set_session_model: ignoring reasoning_effort override — model does not support it"
        );
    }
}
let applied_effort = prepared.sampler_config.reasoning_effort;
```
CONTEXT decision: clamp belongs at **request-build time**, not here (this handler is switch-time UI plumbing, sticky preference must survive round-trips back to Grok). Use this site only to confirm `applied_effort`/`resolved_effort` values continue flowing to the scrollback notice (pattern 5) unchanged — the boolean-gate `if/else` + `tracing::info!/warn!` pair is the project's logging convention to replicate wherever the new clamp path needs parallel logging.

---

### 5. TUI one-line clamp notice — `lifecycle.rs` (modify in place / extend)

**Analog:** itself, existing model-switch scrollback notice (lines 1211-1223):
```rust
if let Some(agent) = app.agents.get_mut(&agent_id) {
    if !unchanged {
        let msg = if let Some(eff) = resolved_effort {
            format!("Switched to {display_name} ({eff} effort)")
        } else {
            format!("Switched to {display_name}")
        };
        agent.scrollback.push_block(RenderBlock::system(msg));
        effects.push(Effect::PersistPreferredModel {
            model_id: model_id.clone(),
            reasoning_effort: resolved_effort,
        });
    }
    effects.extend(maybe_drain_queue(agent));
}
```
Locked copy (per UI-SPEC referenced in RESEARCH B7): `reasoning effort clamped to {level} ({model} supports {list})`. Emit via the same `RenderBlock::system(msg)` + `agent.scrollback.push_block(...)` call, gated by "clamp changed the effective value" (mirrors the existing `if !unchanged` gate shape) — silent when no clamp occurred. Second existing call site for the same mechanism, useful as a second reference: `push_system_to_any_agent` in `dispatch/status.rs:213-218` (not read in full — same `RenderBlock::system` shape per RESEARCH).

---

### 6. Tests — `model_switch_gate.rs` and sampler client.rs suite (extend)

**Analog — naming/shape convention:** `crates/codegen/xai-grok-shell/tests/model_switch_gate.rs`, existing `p6_*` prefixed tests (lines 91-149) and `trusted_codex_wire_headers_are_sent_and_stable` (line 1033, `async fn`). New soft-clamp/sticky-preference tests should follow the same file, prefixed for this phase (e.g. `p11_` or continue `p6_` numbering per existing suite convention — confirm current max prefix in file before naming).

**Analog — wire-shape assertion style:** `crates/codegen/xai-grok-sampler/src/client.rs`, `trusted_codex_responses_profile_on_off_serializes_exactly` (2475-2507) — pattern is: build request via helper (`serialized_response_stream_request`), assert on raw `serde_json::Value` field presence/absence with `.get(...).is_none()` for omitted fields and `assert_eq!` for present ones. Reuse this exact helper/assertion idiom for the new `reasoning.effort` supported-only-emission tests.

---

## Shared Patterns

### System notice / non-blocking TUI message
**Source:** `crates/codegen/xai-grok-pager/src/app/dispatch/session/lifecycle.rs:1211-1218` (also `dispatch/status.rs:213-218`)
**Apply to:** the new clamp notice (pattern 5 above) — the only mechanism this codebase uses for one-line non-blocking scrollback messages.
```rust
agent.scrollback.push_block(RenderBlock::system(msg));
```

### Catalog-driven capability gate (`if agent.models_manager.model_X(...)  { apply } else { warn+ignore }`)
**Source:** `crates/codegen/xai-grok-shell/src/agent/handlers/model_switch.rs:164-180`
**Apply to:** any new gate that decides whether to apply an effort value based on catalog support — same `models_manager.model_supports_reasoning_effort` / `model_reasoning_efforts` accessors (`crates/codegen/xai-grok-shell/src/agent/models.rs:451-483`), no new catalog plumbing needed.

### Wire test assertion idiom (serde_json::Value field presence/absence)
**Source:** `crates/codegen/xai-grok-sampler/src/client.rs:2465-2507`
**Apply to:** all new `reasoning.effort`/`reasoning.summary` emission tests — `assert!(value["reasoning"].get("summary").is_none())` for omitted, `assert_eq!(value["reasoning"]["effort"], "low")` for present.

## No Analog Found

None — every touch point in this phase is a modification to an existing, well-established seam. No new files or new architectural roles are introduced.

## Metadata

**Analog search scope:** `crates/codegen/xai-grok-sampling-types/src`, `crates/codegen/xai-grok-sampler/src`, `crates/codegen/xai-grok-shell/src/agent`, `crates/codegen/xai-grok-pager/src/app/dispatch/session`, `crates/codegen/xai-grok-shell/tests`, `crates/codegen/xai-grok-models`
**Files scanned/read:** 8 (all listed above; line ranges verified exact against RESEARCH.md citations, no drift found)
**Pattern extraction date:** 2026-07-21
**Note:** RESEARCH.md §B4-B8 already did the primary excerpt extraction with verified line numbers; this file cross-verifies each cited range by direct Read and adds the two additional catalog accessors (`model_supports_reasoning_effort`, `model_reasoning_efforts` — `models.rs:451-483`) that the clamp helper will read from, plus the test-suite naming convention (`p6_*`) not otherwise spelled out in RESEARCH.
