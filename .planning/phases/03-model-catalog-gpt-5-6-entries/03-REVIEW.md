---
phase: 03-model-catalog-gpt-5-6-entries
reviewed: 2026-07-16T14:30:00Z
depth: standard
files_reviewed: 12
files_reviewed_list:
  - crates/codegen/xai-grok-models/default_models.json
  - crates/codegen/xai-grok-models/src/lib.rs
  - crates/codegen/xai-grok-shell/src/agent/config.rs
  - crates/codegen/xai-grok-shell/src/agent/config_model_override_parse.rs
  - crates/codegen/xai-grok-shell/src/agent/models.rs
  - crates/codegen/xai-grok-shell/src/remote/client.rs
  - crates/codegen/xai-grok-shell/src/agent/mvp_agent/tests.rs
  - crates/codegen/xai-grok-shell/src/agent/subagent/tests/mod.rs
  - crates/codegen/xai-grok-shell/tests/model_catalog.rs
  - crates/codegen/xai-grok-pager/src/models.rs
  - crates/codegen/xai-grok-pager/tests/format_cli_model_row.rs
  - crates/codegen/xai-grok-pager/tests/dynamic_enum_model_names.rs
findings:
  critical: 0
  warning: 5
  info: 3
  total: 8
status: issues_found
fix_iteration_1: |
  WR-02 fixed (83675bb), WR-04 fixed (3aa9b2b). See 03-REVIEW-FIX.md.
  Deferred: WR-01 (Phase 4 routing), WR-03 (optional test), WR-05 (config rebind intentional).
---

# Phase 3: Code Review Report

**Reviewed:** 2026-07-16T14:30:00Z
**Depth:** standard
**Files Reviewed:** 12
**Status:** issues_found

> **Fix iteration 1 (2026-07-16):** WR-02 and WR-04 addressed — see `03-REVIEW-FIX.md`. WR-01 / WR-03 / WR-05 deferred per fix scope.

## Summary

Reviewed the Phase 3 model-catalog implementation: `ModelProvider` type chain, mixed `default_models.json` (Grok + GPT-5.6 Sol/Terra/Luna), prefetch remove-then-append of Codex defaults, ACP `meta.provider`, CLI `format_cli_model_row`, and integration tests.

Schema threading, remote provider force-to-Xai, remove-then-append collision authority, and surface projections (ACP / CLI / settings inherit) are coherent with the phase plans. No critical security holes (spoofing remote `provider` into Codex is blocked at parse). However, Codex rows still ship with **xAI inference endpoints**, empty-prefetch interaction can silently default the session onto a GPT id, and several regressions/tests leave those gaps unguarded.

## Narrative Findings (AI reviewer)

## Warnings

### WR-01: Codex catalog rows inherit xAI `base_url` / `api_base_url`

**File:** `crates/codegen/xai-grok-shell/src/agent/config.rs:3431-3435`
**Severity:** WARNING
**Issue:** Every bundled default — including `provider: codex` GPT-5.6 rows — is constructed with:

```rust
base_url: endpoints.resolve_inference_base_url(),
api_base_url: Some(endpoints.xai_api_base_url.clone()),
```

Selecting `gpt-5.6-sol|terra|luna` therefore routes credentials and HTTP to the xAI proxy / `api.x.ai` with a Codex model id. Phase 4 is supposed to fix routing, but Phase 3 makes these models **user-selectable** (`supported_in_api: true`, no missing-provider gate). API-key and session users can switch to GPT and get hard failures or wrong-backend traffic until Phase 4/6 land.

`default_models_dual_endpoint_routing` (same file ~5542) further asserts *all* defaults with `api_base_url` must hit `xai_api_base_url`, which currently green-washes the wrong endpoint for Codex rows and will fight Phase 4.

**Fix:** Until Phase 4 routing exists, either:
1. Leave Codex rows list-visible but mark them non-selectable (`user_selectable = false` or `hidden` until provider routing is ready), **or**
2. Set Codex `base_url` / `api_base_url` to empty / placeholder values that cannot silently hit xAI, and refuse sampling with a clear error when provider ≠ routed backend, **or**
3. Narrow the dual-endpoint test to `provider == Xai` only and document Codex endpoints as intentionally unset.

```rust
// Example direction for (2)/(3): do not stamp xAI api_base_url onto Codex defaults
let (base_url, api_base_url) = match m.provider {
    ModelProvider::Xai => (
        endpoints.resolve_inference_base_url(),
        Some(endpoints.xai_api_base_url.clone()),
    ),
    ModelProvider::Codex => {
        // Phase 4 fills real Codex endpoints; leave unset so sampling fails closed.
        (String::new(), None)
    }
};
```

### WR-02: Empty prefetch + Codex re-append can make GPT the effective default

**File:** `crates/codegen/xai-grok-shell/src/agent/config.rs:3160-3174`  
**File:** `crates/codegen/xai-grok-shell/src/agent/models.rs:1694-1714`  
**Severity:** WARNING
**Issue:** When `prefetched = Some({})` and `!has_custom_endpoint`, bundled xAI defaults stay pruned and only Codex GPT rows are re-appended (Q1 / `empty_prefetch_still_gets_codex_defaults`). Preferred default `grok-build` is then absent. `resolve_default_model` falls through to `first_or_fallback()`, which picks the first visible entry — **`gpt-5.6-sol`**. Combined with WR-01, a rare empty-prefetch / empty-cache path can auto-start on a Codex-labeled model that still points at xAI.

D-13 is only asserted as `xai_grok_models::default_model() == "grok-build"`, not as effective session default after empty-prefetch merge.

**Fix:** After Codex re-append, if preferred/default id is missing, prefer a bundled xAI fallback (or keep `grok-build` via re-insert of the default id) before first-GPT:

```rust
// After re-append, if catalog lacks default_model(), re-insert just that xAI default
// (or teach first_or_fallback to prefer provider==Xai / default_model() over GPT).
let default_id = crate::models::default_model();
if !resolved.contains_key(default_id) {
    if let Some(entry) = default_model_entries(&cfg.endpoints).shift_remove(default_id) {
        // insert at front so first_or_fallback stays on grok-build
        let mut ordered = IndexMap::new();
        ordered.insert(default_id.to_owned(), entry);
        ordered.extend(resolved);
        resolved = ordered;
    }
}
```

Add an integration test: empty prefetch + `resolve_default_model` must not land on `gpt-5.6-*` when `default_model()` is `grok-build`.

### WR-03: Remote provider spoof mitigation is untested

**File:** `crates/codegen/xai-grok-shell/src/remote/client.rs:835`
**Severity:** WARNING
**Issue:** `parse_remote_model_value` hardcodes `provider: ModelProvider::Xai` (good — blocks remote spoofing). There is no unit/integration test that a remote JSON field `"provider": "codex"` is ignored. A future edit that starts deserializing remote `provider` would silently re-introduce T-03-01 without failing Phase 3 gates.

**Fix:** Add a focused test next to other `parse_remote_model_value_*` tests:

```rust
#[test]
fn parse_remote_model_value_ignores_remote_provider_field() {
    let value = serde_json::json!({
        "model": "gpt-5.6-sol",
        "contextWindow": 272000,
        "provider": "codex",
        "name": "Hijacked"
    });
    let parsed = parse_remote_model_value(&value, "https://default.url").unwrap();
    assert_eq!(parsed.provider, crate::agent::config::ModelProvider::Xai);
}
```

### WR-04: Invalid-provider override test can false-green

**File:** `crates/codegen/xai-grok-shell/tests/model_catalog.rs:268-275`
**Severity:** WARNING
**Issue:** `override_invalid_provider_warns_keeps_model` treats *any* `InvalidValue` warning as success:

```rust
|| matches!(
    w.kind,
    ModelOverrideWarningKind::InvalidValue
)
```

A warning on a different field (or any InvalidValue) satisfies the assert even if the `provider` field produced no warning. That weakens the invalid-provider contract.

**Fix:** Require a warning whose `field` is `Some("provider")` (and ideally `kind == InvalidValue`):

```rust
let provider_warn = cfg.model_override_warnings.iter().any(|w| {
    w.field.as_deref() == Some("provider")
        && matches!(w.kind, ModelOverrideWarningKind::InvalidValue)
});
assert!(provider_warn, "expected InvalidValue on field provider; warnings={:?}", ...);
```

### WR-05: Config override can rebind bundled Codex provider after authority re-append (footgun)

**File:** `crates/codegen/xai-grok-shell/src/agent/config.rs:3177-3199` + `3675-3677`
**Severity:** WARNING
**Issue:** Layer order is: prefetch replace → Codex remove-then-append → **config.toml overrides**. Any `[model.gpt-5.6-sol] provider = "xai"` (managed policy, stale template, user experiment) rebinds the “authoritative” Codex id back to xAI after the security-sensitive re-append. Phase 3 tests prove remote cannot rebind, but not that config cannot. For multi-provider routing later, managed/user overrides can silently undo catalog binding for the three reserved ids.

**Fix (pick one policy and test it):**
- Document that config **may** rebind (power user / BYOK), and add a test so the policy is explicit; or
- For the three bundled Codex keys, ignore override `provider` (or only allow `Some(Codex)`), preserving D-11 authority:

```rust
// in ConfigModelOverride::apply, when key is a reserved codex default:
if is_bundled_codex_id(key) {
    entry.info.provider = ModelProvider::Codex;
} else if let Some(v) = self.provider {
    entry.info.provider = v;
}
```

## Info

### IN-01: `ModelProvider::display_label` unused in production

**File:** `crates/codegen/xai-grok-shell/src/agent/config.rs:3360-3367`
**Issue:** `display_label` is only exercised by `model_catalog` tests. UI labels come from hard-coded name suffixes in JSON, so the helper can drift from real picker text.
**Fix:** Use `display_label()` when building display names, or delete until a UI surface needs it.

### IN-02: Prefetch path parses bundled defaults twice

**File:** `crates/codegen/xai-grok-shell/src/agent/config.rs:3119` and `3161`
**Issue:** `default_model_entries` (JSON parse + map build) runs once for initial defaults and again for Codex re-append on every prefetch resolve.
**Fix:** Reuse the first `defaults` map (or extract Codex keys once) when `prefetched` is `Some`.

### IN-03: Settings DynamicEnum test uses fixture names, not live catalog

**File:** `crates/codegen/xai-grok-pager/tests/dynamic_enum_model_names.rs:19-28`
**Issue:** Test injects hand-built `(xAI)`/`(Codex)` strings into `PagerLocalSnapshot`. Proves inherit formatting, not that production catalog names reach the settings path. Acceptable given shell/pager `--lib` debt, but weaker than end-to-end.
**Fix:** Optional later: integration that builds snapshot from `to_acp_model_info(resolve_model_list(...))` names.

## What looks solid

- `ModelProvider` serde (`xai`/`codex`), default-missing → Xai, invalid override prune path
- GPT agent_type remains stock (not `codex` harness) per D-05
- Remove-then-append order Sol→Terra→Luna and collision authority vs remote rebind
- Enterprise `has_custom_endpoint` skips inject
- ACP `meta.provider` always set from in-process `ModelInfo.provider`
- CLI `format_cli_model_row` star/dash + empty-name fallback matches UI-SPEC
- Remote parse forces `provider = Xai` (implementation correct; needs WR-03 test)

---

_Reviewed: 2026-07-16T14:30:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
