# Phase 3: Model catalog & GPT-5.6 entries - Pattern Map

**Mapped:** 2026-07-16
**Files analyzed:** 9 (7 modify / touch, 2 inherit-only)
**Analogs found:** 9 / 9

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `crates/codegen/xai-grok-models/default_models.json` | config | batch | same file (extend shape) | exact |
| `crates/codegen/xai-grok-shell/src/agent/config.rs` (`ModelProvider` + fields) | model | transform | `ApiBackend` + `AuthMode` serde enums; existing `ModelInfo` field chain | exact |
| `crates/codegen/xai-grok-shell/src/agent/config.rs` (`resolve_model_list` merge) | service | transform | same fn (prefetch replace + enterprise skip) | exact |
| `crates/codegen/xai-grok-shell/src/agent/config.rs` (`to_acp_model_info`) | service | transform | same fn (`meta.agentType` insert) | exact |
| `crates/codegen/xai-grok-shell/src/remote/client.rs` (`parse_remote_model_value`) | service | request-response | same fn (struct-literal field defaults) | exact |
| `crates/codegen/xai-grok-pager/src/models.rs` | controller (CLI) | request-response | same file (print loop) | exact |
| `crates/codegen/xai-grok-shell/src/agent/config.rs` + `models.rs` tests | test | batch | existing `resolve_model_list_*` / `make_entry_config*` | exact |
| `crates/codegen/xai-grok-pager/src/slash/commands/model.rs` | component | request-response | same (`build_model_items` uses `info.name`) | inherit |
| `crates/codegen/xai-grok-pager/src/settings/registry.rs` | component | request-response | `DynamicEnumSource::ActiveModelCatalog` | inherit |

**Reference-only (do not invent new ids):** `crates/codegen/xai-grok-shell/src/auth/model.rs` — `PROVIDER_XAI` / `PROVIDER_CODEX`.

**Likely no code change:** `crates/codegen/xai-grok-models/src/lib.rs` (only parses `model` + default id; stays valid after JSON expand). `acp/model_state.rs` inherits ACP names.

## Pattern Assignments

### `crates/codegen/xai-grok-models/default_models.json` (config, batch)

**Analog:** same file — current single Grok entry

**Core pattern** (lines 1-18):
```json
{
  "default": "grok-build",
  "web_search": "grok-4.20-multi-agent",
  "image_description": "grok-build",
  "session_summary": "grok-build",
  "models": [
    {
      "model": "grok-build",
      "name": "Grok Build",
      "description": "Best for advanced coding tasks",
      "context_window": 500000,
      "temperature": 0.7,
      "top_p": 0.95,
      "api_backend": "responses",
      "supported_in_api": false
    }
  ]
}
```

**Copy / extend:**
- Keep top-level `"default": "grok-build"` (locked).
- Update Grok `name` → `"Grok Build (xAI)"`; add `"provider": "xai"`.
- Append three GPT rows with UI-SPEC names/descriptions, `provider: "codex"`, `context_window: 272000`, `api_backend: "responses"`, `supported_in_api: true`.
- **Do not** set `agent_type: "codex"` on GPT rows this phase (strict harness — see RESEARCH).
- Order in array = picker order: Grok first, then Sol → Terra → Luna.

**Consumer of JSON fields:** `DefaultModelJson` + `default_models()` in shell `config.rs` (not the models crate’s minimal `DefaultModelEntry`, which only reads `model` for id asserts).

---

### `ModelProvider` enum + schema fields on catalog types (model, transform)

**Primary analog (serde enum):** `crates/codegen/xai-grok-sampling-types/src/types.rs` `ApiBackend`

```rust
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApiBackend {
    #[default]
    ChatCompletions,
    Responses,
    Messages,
}
```

**Wire-id constants to match (auth slots):** `crates/codegen/xai-grok-shell/src/auth/model.rs` lines 16-20:

```rust
pub const PROVIDER_XAI: &str = "xai";
pub const PROVIDER_CODEX: &str = "codex";
```

**Enum helpers analog:** `crates/codegen/xai-grok-shell/src/upload/manifest.rs` lines 15-29 (`as_str` on rename_all snake_case enum).

**Recommended shape (from RESEARCH, align with ApiBackend style):**
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ModelProvider {
    #[default]
    Xai,
    Codex,
}

impl ModelProvider {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Xai => "xai",     // == PROVIDER_XAI
            Self::Codex => "codex", // == PROVIDER_CODEX
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

**Where to add `provider: ModelProvider` (full type chain):**

| Type | Location in `config.rs` | Serde default |
|------|-------------------------|---------------|
| `DefaultModelJson` | ~3318 | `#[serde(default)]` → `Xai` |
| `ModelEntryConfig` | ~3413 | `#[serde(default)]` |
| `ConfigModelOverride` | ~3548 | `Option<ModelProvider>` + apply if `Some` |
| `ModelInfo` | ~3693 | `#[serde(default)]` |

**Field mapping pattern** — extend existing `default_models()` constructor (lines 3374-3407): map `m.provider` into `ModelEntryConfig { … provider: m.provider, … }`.

**`ModelInfo::from_config` / `fallback` pattern** (lines 3764-3831) — every struct literal must gain `provider`:
- `fallback`: `provider: ModelProvider::default()` (xai)
- `from_config`: `provider: entry.provider`

**`ConfigModelOverride::apply` pattern** (lines 3589-3688) — optional overlay:
```rust
if let Some(v) = self.provider {
    entry.info.provider = v;
}
```

**Compile fan-out pattern:** test helpers use full struct literals (no `..Default`):
- `test_model_entry` — `config.rs` ~5350-5393
- `make_entry_config` / `make_entry_config_with_id` — `models.rs` ~3349-3391

Add `provider: ModelProvider::default()` (or explicit) to each in the same wave.

---

### `resolve_model_list` Codex union-append (service, transform)

**Analog:** same function — `crates/codegen/xai-grok-shell/src/agent/config.rs` lines 3107-3226

**Existing layers (copy structure, insert new layer):**
1. Skip built-in defaults when `cfg.endpoints.has_custom_endpoint()` (lines 3112-3122)
2. Load defaults via `default_model_entries`
3. If `prefetched`: inherit cw/agent_type/api_backend from donor, then **`resolved = prefetched`** (full replace) (lines 3123-3154)
4. Apply `config_models` overrides
5. Slug-match inherit + global agent_type + extra_headers + scalars + derive RE

**Enterprise gate analog** (lines 3112-3118 + `has_custom_endpoint` at 273-275):
```rust
pub fn has_custom_endpoint(&self) -> bool {
    self.models_base_url.is_some() || self.models_list_url.is_some()
}
// …
if cfg.endpoints.has_custom_endpoint() {
    tracing::info!(/* … */, "custom models endpoint active, skipping built-in defaults");
} else {
    let defaults = default_model_entries(&cfg.endpoints);
    resolved.extend(defaults);
}
```

**Insert after prefetch replace, before `config_models` loop** (cycle 2: **remove-then-append** only Codex rows — not replace-in-place):
```rust
// After: resolved = prefetched;
// Before: for (key, model_override) in &cfg.config_models
if !cfg.endpoints.has_custom_endpoint() {
    let codex_defaults: Vec<_> = default_model_entries(&cfg.endpoints)
        .into_iter()
        .filter(|(_, entry)| entry.info.provider == ModelProvider::Codex)
        .collect();
    for (key, _) in &codex_defaults {
        resolved.shift_remove(key);
    }
    for (key, entry) in codex_defaults {
        resolved.insert(key, entry);
    }
}
// Result: remote/xAI keys first, then Sol→Terra→Luna in JSON order even if
// prefetched listed terra first or rebound a gpt-5.6-* id to xai.
```

**Tests that encode current prune behavior** (must update, not delete enterprise intent) — `config.rs` ~10922-10979:

| Test | Current expectation | Phase 3 expectation |
|------|---------------------|---------------------|
| `resolve_model_list_prefetch_replaces_bundled_entirely` | no `grok-build` after remote-only map | still no grok-build; **must** contain gpt-5.6-* |
| `resolve_model_list_keeps_prefetch_only_entries_and_prunes_defaults` | no grok-build | same for xAI defaults; Codex defaults present |
| `resolve_model_list_empty_prefetch_yields_empty_base` | empty | RESEARCH Q1: prefer inject Codex when `!custom_endpoint`; update assert |
| `resolve_model_list_prefetch_visibility_matches_auth_and_server_list` | sess=1, api=empty | counts rise when GPT (`supported_in_api: true`) always-visible |

---

### `to_acp_model_info` meta.provider (service, transform)

**Analog:** same fn — `config.rs` lines 4788-4837

**Core meta insert pattern** (copy `agentType` style):
```rust
map.insert(
    "agentType".to_string(),
    serde_json::Value::String(info.agent_type.clone()),
);
// ADD:
map.insert(
    "provider".to_string(),
    serde_json::Value::String(info.provider.as_str().to_owned()),
);
```

**Display name:** already `info.name.clone().unwrap_or_else(|| info.model.clone())` — provider-visible label is the **JSON `name` suffix**, not a separate ACP field for UI.

---

### `parse_remote_model_value` (service, request-response)

**Analog:** `crates/codegen/xai-grok-shell/src/remote/client.rs` lines 791-943

**Pattern:** large `Some(ModelEntryConfig { … })` with per-field defaults. Remote models are xAI-sourced — default `provider` to `Xai` when absent (do not read a remote `provider` as `codex` unless product later wants it; RESEARCH recommends remote → xai).

Add to struct literal end (before closing):
```rust
provider: crate::agent::config::ModelProvider::Xai, // or Default::default()
```

Existing defaults to mirror style (e.g. `supported_in_api` unwrap_or true, `hidden` unwrap_or false) at lines 865-874.

---

### `crates/codegen/xai-grok-pager/src/models.rs` (CLI, request-response)

**Analog:** same file lines 10-40

**Current print loop** (lines 27-36):
```rust
println!("Default model: {}", state.current_model_id.0);
println!();
println!("Available models:");
for m in state.available_models {
    if m.model_id == state.current_model_id {
        println!("  * {} (default)", m.model_id.0);
    } else {
        println!("  - {}", m.model_id.0);
    }
}
```

**UI-SPEC target** (provider in name, id first for scripting):
```rust
// Prefer ACP ModelInfo.name; fall back to id if empty.
//   * {id} ({name})
//   - {id} ({name})
// Keep "Default model: {id}" header unchanged.
// Note: existing marks current with "(default)" — UI-SPEC uses `*` / `-` with name;
// prefer UI-SPEC: `  * {id} ({name})` / `  - {id} ({name})` without extra "(default)" suffix
// if name already carries provider (avoid double noise). Align with UI-SPEC copy table.
```

Auth preamble lines 11-19 stay; **do not** filter models by Codex login.

---

### Inherit-only: slash `/model` (`build_model_items`)

**Analog:** `crates/codegen/xai-grok-pager/src/slash/commands/model.rs` lines 153-182

```rust
let display = if is_current {
    format!("{} (current)", info.name)
} else {
    info.name.clone()
};
// …
items.push(ArgItem {
    display,
    match_text: info.name.clone(),
    insert_text,
    description: info.description.clone().unwrap_or_default(),
});
```

**No structural change** if catalog `name` is `GPT-5.6 Sol (Codex)` etc. Current model becomes `Grok Build (xAI) (current)` per UI-SPEC.

---

### Inherit-only: settings DynamicEnum

**Analog:** `crates/codegen/xai-grok-pager/src/settings/registry.rs` — `DynamicEnumSource::ActiveModelCatalog` builds choices from live catalog names. No new badge column; inherits name suffixes.

---

### Tests (test, batch)

**Analogs:**
1. Prefetch / prune suite — `config.rs` ~10922+
2. Helpers — `test_model_entry`, `make_entry_config*`
3. `available_models` — `models.rs` 1777-1787 (`visible_for_auth` only; **not** Codex credential gate)

```rust
pub fn available_models(
    catalog: &IndexMap<String, ModelEntry>,
    is_session_auth: bool,
) -> IndexMap<acp::ModelId, acp::ModelInfo> {
    let visible: IndexMap<String, ModelEntry> = catalog
        .iter()
        .filter(|(_, e)| e.info.visible_for_auth(is_session_auth))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    config::to_acp_model_info(&visible)
}
```

**New tests to colocate (names from RESEARCH Validation Architecture):**
- `catalog_includes_gpt56` — Sol/Terra/Luna present; `provider == Codex`; names match UI-SPEC
- `mixed_catalog_order` — no-prefetch order: grok-build then sol → terra → luna
- `codex_defaults_survive_prefetch` — xAI-only prefetched map still has gpt-5.6-*
- `provider_default_xai` — missing provider parses as xai
- `available_models` with `is_session_auth` true/false — GPT visible both (supported_in_api true); never gate on Codex auth slot
- Default remains `grok-build` / `default_model()`

**Run filters (prefer pure resolve path if full `--lib` is flaky):**
```bash
cargo test -p xai-grok-shell --lib resolve_model_list -- --nocapture
cargo test -p xai-grok-models --lib
```

## Shared Patterns

### Provider wire ids (auth ↔ catalog)

**Source:** `crates/codegen/xai-grok-shell/src/auth/model.rs` lines 16-20  
**Apply to:** `ModelProvider::as_str`, JSON `provider` values, optional ACP meta  
```rust
pub const PROVIDER_XAI: &str = "xai";
pub const PROVIDER_CODEX: &str = "codex";
```
Do **not** invent `openai` / `chatgpt` as catalog provider ids.

### Serde default for new enum fields

**Source:** `ApiBackend` + existing `#[serde(default)]` / `#[serde(default = "default_true")]` on `ModelInfo`  
**Apply to:** all catalog types receiving `provider`  
Missing JSON/TOML/remote → `ModelProvider::Xai` (backward compatible). Ship defaults always set explicitly.

### Catalog visibility vs auth

**Source:** `ModelInfo::visible_for_auth` — `config.rs` ~3853-3862  
**Apply to:** GPT rows use `supported_in_api: true`; **never** filter by Codex OAuth presence this phase  
```rust
pub fn visible_for_auth(&self, is_session_auth: bool) -> bool {
    !self.hidden && (is_session_auth || self.supported_in_api)
}
```
`is_session_auth` means **xAI OAuth session vs API key**, not multi-provider login.

### Name-as-label (single SoT for all UI surfaces)

**Source:** UI-SPEC + `to_acp_model_info` + `build_model_items`  
**Apply to:** display only via catalog `name` suffix ` (xAI)` / ` (Codex)`  
No new picker badge column (`views/picker.rs` reuse).

### agent_type ≠ provider

**Source:** `ModelInfo.agent_type` docs + RESEARCH Pattern 4  
**Apply to:** GPT catalog rows leave `agent_type` at stock default (`default_agent_type()` / grok-build-plan). Setting `codex` harness early hits `MODEL_SWITCH_INCOMPATIBLE_AGENT` / rebuild without Phase 4 routing.

### IndexMap order

**Source:** `resolve_model_list` returns `IndexMap`; insertion order drives flat list  
**Apply to:** JSON array order + append Codex after prefetched map (Grok remote first, then Sol→Terra→Luna).

### Enterprise custom endpoint

**Source:** `has_custom_endpoint` skip of built-ins  
**Apply to:** Codex re-insert must also be gated on `!has_custom_endpoint()` so enterprise-only catalogs stay clean.

## No Analog Found

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| — | — | — | Full chain is brownfield; no greenfield files. Closest work is field + merge + CLI print on existing files. |

If planner introduces a tiny pure format helper for CLI (`format_model_list_line(id, name)`) for unit testing without spawning ACP: no prior helper — extract from `pager/src/models.rs` print loop and test inline.

## Metadata

**Analog search scope:**
- `crates/codegen/xai-grok-models/`
- `crates/codegen/xai-grok-shell/src/agent/{config,models}.rs`
- `crates/codegen/xai-grok-shell/src/remote/client.rs`
- `crates/codegen/xai-grok-shell/src/auth/model.rs`
- `crates/codegen/xai-grok-pager/src/{models.rs,slash/commands/model.rs,settings/registry.rs}`
- `crates/codegen/xai-grok-sampling-types/src/types.rs` (`ApiBackend`)

**Files scanned:** ~15 primary touchpoints + helpers  
**Pattern extraction date:** 2026-07-16

## Planner action cheat-sheet

1. **Schema + JSON:** `ModelProvider`; fields on `DefaultModelJson` / `ModelEntryConfig` / `ModelInfo` / `ConfigModelOverride`; `default_models()` + `from_config` + `fallback` + remote parse + test helpers; extend `default_models.json`.
2. **Merge:** Codex re-insert after prefetch; gate custom endpoint; update prune tests + empty-prefetch expectation.
3. **Projection + CLI:** `meta.provider` in `to_acp_model_info`; CLI `id (name)`; slash/settings inherit.
4. **Tests:** mixed catalog, provider binding, prefetch survival, default still grok-build, visibility without Codex login.
)
