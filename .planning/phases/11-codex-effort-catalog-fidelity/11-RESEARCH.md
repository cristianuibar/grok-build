# Phase 11: Codex effort & catalog fidelity - Research

**Researched:** 2026-07-21 by Grok 4.5 (high effort, read-only, 33 turns; session 019f82d9-9bd4-7862-b3a1-be6353052e93). Citations spot-verified by orchestrator (turn_context.rs clamp, conversation.rs Reasoning, client.rs trusted summary-clear — all exact).


## A1. Official Codex: unsupported effort handling

**Soft-clamp lives in model switch (`TurnContext::with_model`), not request serialization.**

When the active effort is not in the target model’s `supported_reasoning_levels`, Codex **keeps the turn going** and picks the **middle entry** of that list (else the model default). It does **not** hard-fail.

```243:263:codex/codex-rs/core/src/session/turn_context.rs
        let supported_reasoning_levels = model_info
            .supported_reasoning_levels
            .iter()
            .map(|preset| preset.effort.clone())
            .collect::<Vec<_>>();
        let reasoning_effort = if let Some(current_reasoning_effort) = self.reasoning_effort.clone()
        {
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

Separate wire remap (not a supported-list clamp): **`Ultra` → `Max`** before the payload is built.

```175:179:codex/codex-rs/core/src/client.rs
fn reasoning_effort_for_request(effort: ReasoningEffortConfig) -> ReasoningEffortConfig {
    match effort {
        ReasoningEffortConfig::Ultra => ReasoningEffortConfig::Max,
        effort => effort,
    }
}
```

`build_reasoning` then applies that map and falls back to catalog default when effort is unset:

```805:821:codex/codex-rs/core/src/client.rs
    fn build_reasoning(
        model_info: &ModelInfo,
        effort: Option<ReasoningEffortConfig>,
        summary: ReasoningSummaryConfig,
    ) -> Reasoning {
        Reasoning {
            effort: effort
                .or_else(|| model_info.default_reasoning_level.clone())
                .map(reasoning_effort_for_request),
            summary: (model_info.supports_reasoning_summary_parameter
                && summary != ReasoningSummaryConfig::None)
                .then_some(summary),
            // ...
        }
    }
```

**Note vs Phase 11 CONTEXT:** `11-CONTEXT.md` says “nearest supported, prefer downgrade.” Official code uses **middle-of-list**, not nearest-neighbor downgrade. Plan should pick which contract to implement.

---

## A2. Official Codex: when `reasoning.effort` / `reasoning.summary` are sent

Serialization types (`skip_serializing_if = Option::is_none`):

```147:154:codex/codex-rs/codex-api/src/common.rs
pub struct Reasoning {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort: Option<ReasoningEffortConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<ReasoningSummaryConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<ReasoningContext>,
}
```

**`reasoning.effort`**
- Included when `build_reasoning` produces `Some(effort)` (explicit turn effort, else `model_info.default_reasoning_level`).
- Omitted when both are `None` (`Option::is_none` skip).
- Always mapped through `reasoning_effort_for_request` (Ultra→Max).

**`reasoning.summary`**
- Included only when `supports_reasoning_summary_parameter && summary != None`.
- **`None` → field omitted** (not sent as `"none"`).

```813:815:codex/codex-rs/core/src/client.rs
            summary: (model_info.supports_reasoning_summary_parameter
                && summary != ReasoningSummaryConfig::None)
                .then_some(summary),
```

Request always attaches a `reasoning` object (possibly with fields skipped):

```865:898:codex/codex-rs/core/src/client.rs
        let reasoning = Self::build_reasoning(model_info, effort, summary);
        // ...
            reasoning: Some(reasoning),
```

`ReasoningSummary` enum includes `None` as the “disable summaries” value:

```47:54:codex/codex-rs/protocol/src/config_types.rs
pub enum ReasoningSummary {
    #[default]
    Auto,
    Concise,
    Detailed,
    /// Option to disable reasoning summaries.
    None,
}
```

---

## A3. Official effort ladder / catalog / max↔xhigh

**Enum + wire strings** (`protocol/src/openai_models.rs`):

```40:68:codex/codex-rs/protocol/src/openai_models.rs
pub enum ReasoningEffort {
    None,
    Minimal,
    Low,
    #[default]
    Medium,
    High,
    XHigh,
    Max,
    Ultra,
    Custom(String),
}
// as_str: none, minimal, low, medium, high, xhigh, max, ultra
```

**GPT-5.6 catalog** (`models-manager/models.json`):

| Model | default_reasoning_level | supported_reasoning_levels | default_reasoning_summary |
|-------|-------------------------|----------------------------|---------------------------|
| gpt-5.6-sol | `low` | low, medium, high, xhigh, **max**, **ultra** | `none` |
| gpt-5.6-terra | `medium` | same as Sol | `none` |
| gpt-5.6-luna | `medium` | low…max (**no ultra**) | `none` |

Evidence (Sol):

```30:58:codex/codex-rs/models-manager/models.json
      "default_reasoning_summary": "none",
      "default_reasoning_level": "low",
      "supported_reasoning_levels": [
        { "effort": "low", ... },
        { "effort": "medium", ... },
        { "effort": "high", ... },
        { "effort": "xhigh", ... },
        { "effort": "max", ... },
        { "effort": "ultra", ... }
      ],
```

**max / xhigh / ultra mapping**
- On the wire they are **distinct** strings (`xhigh`, `max`, `ultra`) — `as_str` above.
- **UI ultra → wire max** via `reasoning_effort_for_request` (A1).
- bum’s parse maps CLI `max` → internal `Xhigh` only (`types.rs:839`); that is bum-side, not official wire identity.

---

## B4. bum effort flow end-to-end + choke point

| Stage | Path | Behavior |
|-------|------|----------|
| TUI `/model` + effort phase | `xai-grok-pager/src/slash/commands/model.rs` | Chained menu; `resolve_effort_for_model` → `Action::SwitchModel { effort: Some(...) }` or Error |
| Dashboard staging | `pager/.../dispatch/dashboard.rs:1473–1521` | `stage_dashboard_model` → `pending_model.effort` → `deferred_model_switch` |
| Headless `--effort` | `pager/src/headless.rs:765–815` | `resolve_effort_for_model`; Unsupported → warn+ignore; else meta on `SetSessionModel` |
| Shell apply | `shell/.../handlers/model_switch.rs:43,162–181` | meta effort only applied if `model_supports_reasoning_effort`; else warn-ignore |
| Prepared config | `shell/.../config.rs:5293` | `reasoning_effort: info.reasoning_effort` (catalog default) |
| Chat-state request | `xai-chat-state/.../request_builder.rs:145` | `reasoning_effort: sampling_config.reasoning_effort` |
| Wire conversion | `sampling-types/.../conversation.rs:2159–2161` | builds `rs::Reasoning { effort, summary }` |
| Trusted Codex profile | `sampler/src/client.rs:1233–1244` | forces `reasoning.summary = None` |

**Sampler config field:**

```88:89:grok-build/crates/codegen/xai-grok-sampling-types/src/types.rs
    pub reasoning_effort: Option<ReasoningEffort>,
```

(Also mirrored on `xai-grok-sampler` `SamplingConfig` at `sampler/src/config.rs:94`.)

**Wire builder (every Responses request):**

```2159:2162:grok-build/crates/codegen/xai-grok-sampling-types/src/conversation.rs
            reasoning: Some(rs::Reasoning {
                effort: req.reasoning_effort.map(|e| e.to_responses_api()),
                summary: Some(rs::ReasoningSummary::Concise),
            }),
```

**Trusted Codex post-pass:**

```1242:1244:grok-build/crates/codegen/xai-grok-sampler/src/client.rs
        if let Some(reasoning) = request.reasoning.as_mut() {
            reasoning.summary = None;
        }
```

**Single choke point for clamp (planning recommendation):**  
The place every Codex-bound Responses body finalizes effort is  
`From<&ConversationRequest> for rs::CreateResponse` (`conversation.rs:2159`) **plus** the trusted profile pass (`client.rs:1233`) for Codex-only summary.  

Catalog-aware soft-clamp needs model-supported levels; that data is available earlier (shell prepare / turn / model catalog) but **not** inside the bare `From` impl today. Plan should either:
1. clamp when building `ConversationRequest` / `SamplingConfig` on the Codex path, with sticky preference preserved separately, or  
2. pass supported levels into the Responses conversion / trusted profile.

CONTEXT already requires **per-request clamp at build time**, not switch-time UI handlers (`.planning/phases/11-codex-effort-catalog-fidelity/11-CONTEXT.md:16–20`).

---

## B5. Today: Grok effort → Codex model that does not support that value

**Not a soft-clamp. Behavior depends on path:**

1. **`/model` / `/effort` with explicit token**  
   Menu-gated at TUI: unoffered levels → **`CommandResult::Error`** (hard fail at command layer, no turn).  
   `model_state.rs:248–271`, `effort.rs:84–90`, `model.rs:95–99`.

2. **Model switch without effort meta** (`effort: None`)  
   Shell **rebuilds** sampling config from **catalog default**, not previous session effort:

```162:181:grok-build/crates/codegen/xai-grok-shell/src/agent/handlers/model_switch.rs
    let mut prepared =
        agent.prepare_prepared_sampling_config_for_model(&model, handle.origin_client.clone());
    if let Some(eff) = effort_override {
        if agent.models_manager.model_supports_reasoning_effort(model_id.0.as_ref()) {
            prepared.sampler_config.reasoning_effort = Some(eff);
        } else {
            // ignore override — model does not support it
        }
    }
```

   `sampling_config_for_model` sets `reasoning_effort: info.reasoning_effort` (`config.rs:5293`).  
   So mid-session switch **resets to target default** (Sol→low, Terra/Luna→medium). No clamp notice; previous Grok effort is **not sticky**.

3. **Effort override when model has `supports_reasoning_effort=false`**  
   **Silent drop** (warn log only) — lines 174–178 above. Headless same: `headless.rs:768–774`.

4. **Override when model supports effort but value not in menu**  
   Shell **does not** re-check the menu — it stamps the override if the bool flag is true. TUI normally prevents this; shell would **send it on the wire** with no clamp.

5. **Wire layer**  
   No soft-clamp; whatever is on `ConversationRequest.reasoning_effort` is serialized (`conversation.rs:2160`).  
   `effort: None` → field omitted (`async-openai` `skip_serializing_if` on `Reasoning.effort`).

**Summary:** Today is **reset-to-default / silent-ignore / TUI hard-error**, not official Codex middle-of-list soft-clamp, and not sticky preference + per-request clamp.

---

## B6. `reasoning.summary` emission + catalog defaults

**Emission today**
1. Always set to `Some(Concise)` in Responses conversion (`conversation.rs:2161`).
2. **Trusted Codex profile forces `summary = None`** → omitted on wire (`client.rs:1242–1244`).
3. Non-trusted / generic Responses keeps `"summary": "concise"` (test at `client.rs:2505`).

```2475:2505:grok-build/crates/codegen/xai-grok-sampler/src/client.rs
        assert!(trusted["reasoning"].get("summary").is_none());
        // ...
        assert_eq!(generic["reasoning"]["summary"], "concise");
```

**Catalog (`default_models.json`)**  
GPT-5.6 Sol/Terra/Luna advertise **effort menus only** — **no** `default_reasoning_summary` / summary field:

```28:52:grok-build/crates/codegen/xai-grok-models/default_models.json
      "supports_reasoning_effort": true,
      "reasoning_effort": "low",
      "reasoning_efforts": [
        { "value": "low", ..., "default": true },
        { "value": "medium", ... },
        { "value": "high", ... },
        { "value": "xhigh", ... }
      ]
```

(Sol default effort `low`; Terra/Luna `medium` at lines 64, 99.)

Official GPT-5.6 catalog uses `default_reasoning_summary: "none"` → omit. bum approximates that only via **trusted profile hard-clear**, not catalog-driven summary defaults.

---

## B7. TUI one-line system notices (reuse for clamp)

**Mechanism:** `RenderBlock::system(...)` pushed onto agent scrollback.

Concrete model-switch success notice:

```1211:1218:grok-build/crates/codegen/xai-grok-pager/src/app/dispatch/session/lifecycle.rs
            if let Some(agent) = app.agents.get_mut(&agent_id) {
                if !unchanged {
                    let msg = if let Some(eff) = resolved_effort {
                        format!("Switched to {display_name} ({eff} effort)")
                    } else {
                        format!("Switched to {display_name}")
                    };
                    agent.scrollback.push_block(RenderBlock::system(msg));
```

CLI effort error uses the same channel:

```104:108:grok-build/crates/codegen/xai-grok-pager/src/app/dispatch/session/lifecycle.rs
    if let Some(err) = outcome.effort_error {
        let msg = format!("--effort/--reasoning-effort: {}", err.message());
        // ...
        agent.scrollback.push_block(RenderBlock::system(msg));
```

Also `push_system_to_any_agent` in `dispatch/status.rs:213–218`.

UI-SPEC locked copy:  
`reasoning effort clamped to {level} ({model} supports {list})`  
(`.planning/phases/11-codex-effort-catalog-fidelity/11-UI-SPEC.md:75`).

---

## B8. Existing test seams

### Effort parsing / menus
| Path | What |
|------|------|
| `crates/codegen/xai-grok-sampling-types/src/types.rs` | `ReasoningEffort` parse (`max`→`Xhigh`), meta helpers |
| `crates/codegen/xai-grok-pager/src/acp/model_state.rs` | `resolve_effort_for_model`, menu options |
| `crates/codegen/xai-grok-pager/src/slash/commands/model.rs` | chained effort phase, reject unoffered |
| `crates/codegen/xai-grok-pager/src/slash/commands/effort.rs` | `/effort` unit tests |
| `crates/codegen/xai-grok-shell/tests/model_catalog.rs` | `gpt56_supports_reasoning_effort_menus` |
| `crates/codegen/xai-grok-shell/tests/test_built_binary_e2e.rs` | `headless_reasoning_efforts_payload_parses_and_legacy_effort_rides_wire` |

### Codex request wire shape (Phase 10 focused suite)
| Path | What |
|------|------|
| `crates/codegen/xai-grok-sampler/src/client.rs` | `trusted_codex_responses_profile_on_off_serializes_exactly` (summary omit, store, tools) |
| `crates/codegen/xai-grok-sampling-types/src/conversation.rs` | Responses conversion + `strip_input_item_ids_for_store_false` |
| `crates/codegen/xai-grok-shell/tests/model_switch_gate.rs` | `trusted_codex_wire_headers_are_sent_and_stable` |
| `crates/codegen/xai-grok-shell/src/session/acp_session_tests/codex_reconstruct_refresh_tests.rs` | trusted reconstruct / profile |
| `.planning/phases/10-codex-responses-wire-parity/10-VALIDATION.md` | consolidated **26/26** suite inventory |

### Model-switch behavior
| Path | What |
|------|------|
| `crates/codegen/xai-grok-shell/tests/model_switch_gate.rs` | `p6_` ACP apply, dual-provider, history, mid-turn |
| `crates/codegen/xai-grok-pager/src/app/dispatch/tests/task_result.rs` | `SwitchModelComplete` + scrollback message |
| `crates/codegen/xai-grok-pager/src/app/dispatch/tests/router.rs` | SwitchModel effect/pending |
| `crates/codegen/xai-grok-pager/src/app/dispatch/tests/session/lifecycle.rs` | deferred switch / no-session |
| `crates/codegen/xai-grok-pager/tests/pty_e2e/zero_turn_model_switch_no_modal.rs` | zero-turn UX |
| `crates/codegen/xai-grok-pager/tests/pty_e2e/agent_type_mismatch_modal_on_model_switch.rs` | mismatch modal |

---

## Planning implications

1. **Choke point location**  
   Finalize clamp where every Codex Responses body is built: `conversation.rs` `From` conversion and/or `apply_trusted_codex_response_profile`, with catalog-supported levels available (shell turn / prepare). Prefer **request-build**, not switch-time UI only, so sticky preference works (`11-CONTEXT.md:19–20`).

2. **Clamp rule (must encode explicitly)**  
   Official Codex = **keep if supported, else middle of `supported_reasoning_levels`, else default** (`turn_context.rs:249–257`). CONTEXT wants **nearest + prefer downgrade**. Plan must pick one and test it; do not claim “matches official” while implementing nearest-downgrade unless documented as intentional product delta.

3. **Emission rules**  
   - `reasoning.effort`: send only post-clamp when level ∈ model’s advertised list; omit if unsupported/empty (`CODEX-RESPONSES-WIRE.md` §5).  
   - `reasoning.summary`: catalog-default `none` → **omit** (official). Today trusted path always clears summary; generic still forces `concise`. Align Codex path with catalog (may need catalog field or hardcode GPT-5.6 → omit).  
   - Keep bum parse alias `max`→`Xhigh`; do not invent ultra UI this phase.

4. **Notice mechanism**  
   Reuse `RenderBlock::system` (same as “Switched to …”). UI-SPEC copy:  
   `reasoning effort clamped to {level} ({model} supports {list})`.  
   Emit only when clamp **changes** effective wire effort; silence when already supported.

5. **Test seams**  
   Extend Phase 10 wire suite (`client.rs` trusted serialization + conversation conversion) and model-switch/effort unit tests (`model_state`, `model_switch_gate`, switch complete scrollback). Add: soft-clamp unit matrix, sticky preference across Grok↔Codex, summary omit under trusted/catalog, no hard-fail on unsupported carry-over.

---

Context received: p11-research