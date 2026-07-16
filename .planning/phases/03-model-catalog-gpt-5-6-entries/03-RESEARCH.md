# Phase 3: Model catalog & GPT-5.6 entries - Research

**Researched:** 2026-07-16
**Domain:** Rust model catalog (embedded JSON + shell resolve/merge + TUI/CLI picker surfaces)
**Confidence:** HIGH

## Summary

Phase 3 is a **brownfield catalog schema + data + visibility** change on the existing Grok Build model stack — not a new framework. The single source of truth for shipped defaults is `crates/codegen/xai-grok-models/default_models.json`, parsed by `xai-grok-shell` `agent/config.rs` into `ModelEntryConfig` → `ModelInfo` / `ModelEntry`, then filtered by `agent/models.rs` (`resolve_model_catalog` / `available_models`) and projected to the TUI/CLI via ACP `ModelInfo` (`to_acp_model_info`). [VERIFIED: codebase]

Today the embedded catalog is **one Grok entry** (`grok-build`, `supported_in_api: false`, name `"Grok Build"`). There is **no** `provider` field anywhere on catalog types; routing/auth later must not infer provider from `agent_type` (that field is the **agent harness** template — e.g. `grok-build` / `codex` strict harness — not the Phase 2 auth slot). Phase 2 already locked wire ids **`xai`** / **`codex`** (`PROVIDER_XAI` / `PROVIDER_CODEX` in `auth/model.rs`). [VERIFIED: codebase]

**Critical planning risk:** `resolve_model_list` currently **replaces** bundled defaults entirely when a prefetched remote catalog is present (`resolved = prefetched`). Existing unit tests encode “prefetch prunes defaults.” On the normal logged-in path (xAI remote `/v1/models`), **GPT rows added only to `default_models.json` will disappear** unless merge policy is extended to **re-insert non-xAI (Codex) bundled defaults** when not on a custom models endpoint. Enterprise `has_custom_endpoint()` must continue to skip built-in defaults. [VERIFIED: codebase]

Wire IDs **`gpt-5.6-sol`**, **`gpt-5.6-terra`**, **`gpt-5.6-luna`** match live Codex catalog slugs (local Codex `models_cache.json`, context window 272000, priority Sol→Terra→Luna) and third-party API ID writeups. UI-SPEC display names/suffixes are locked. [VERIFIED: local Codex models_cache + web sources]

**Primary recommendation:** Extend `default_models.json` + full type chain with explicit `provider: "xai"|"codex"` (default missing → `xai`); ship Sol/Terra/Luna with `provider: codex`, UI-SPEC names, `context_window: 272000`, `supported_in_api: true`; keep `agent_type` at stock default (do **not** set `codex` harness this phase); **change `resolve_model_list` to union-append bundled Codex defaults after prefetch** (not full prune); surface labels via `name` suffix + optional ACP meta `provider`; update CLI list to show `id (name)`; prove with unit tests on resolve/available paths (no routing, no Codex OAuth).

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Ship **Sol, Terra, and Luna** as the GPT-5.6 family set (matches daily-driver / codex-cli usage)
- Prefer stable IDs **`gpt-5.6-sol`**, **`gpt-5.6-terra`**, **`gpt-5.6-luna`**; research may refine wire IDs if Codex/OpenAI differs — keep display names stable
- **Extend** embedded `default_models.json` (and any remote/default merge path already used) so GPT entries ship in the same catalog source of truth as Grok — not a separate hardcoded picker list
- GPT entries need: id/slug, display `name`, short `description`, **`provider: codex`**, reasonable `context_window`, and enough fields to appear in the selector; full Codex backend routing deferred to Phase 4
- Add an **explicit `provider` field** on each catalog entry (`"xai"` | `"codex"`) — do **not** treat `agent_type` alone as the provider binding for routing later
- Canonical ids match Phase 2 auth slots: **`xai`** and **`codex`**
- All current Grok entries get **`provider: "xai"`** explicitly in the catalog
- Missing `provider` defaults to **`xai`** for backward compatibility with existing configs/tests, with a clear parse path; new ship defaults always set the field explicitly
- **Single flat mixed list** — no session-global provider mode that filters the whole session (PROJECT non-negotiable)
- Show provider as a **label/badge or name suffix** (e.g. `GPT-5.6 Sol (Codex)` / provider meta) so users see who serves the model
- Default ordering: **Grok/xAI entries first**, then GPT-5.6 family (Sol → Terra → Luna unless research says otherwise)
- **Show GPT entries even if Codex is not logged in** this phase — missing-provider gate is Phase 6; catalog completeness is the Phase 3 goal
- Keep **`grok-build` (xAI) as the default model** for the milestone default path
- Phase 3 is **catalog + provider binding + selector visibility only** — do not implement turn routing to Codex backends here
- Prove with **automated tests**: mixed list contains Grok + GPT entries; each entry has correct provider; no auth-based filtering of GPT rows; default remains Grok
- Do not require custom BYOK GPT entries for v1 success criteria this phase

### Claude's Discretion
- Exact serde field placement (`ModelInfo` vs `ModelEntry` vs config TOML mirror), ACP meta wire shape for provider label, and whether display is suffix vs badge column
- Whether GPT entries use placeholder/stub `base_url` values safe for list-only vs empty with defaults
- How deep to thread provider into remote model merge without breaking existing remote catalog overrides
- Snapshot/test harness style for picker vs shell list APIs
- Wire-id refinements during research if Codex catalog differs from `gpt-5.6-*` (preserve Sol/Terra/Luna family + display names)

### Deferred Ideas (OUT OF SCOPE)
- Provider-aware request routing (base URL + credentials per provider) → Phase 4
- Codex/ChatGPT OAuth lifecycle → Phase 5
- Missing-provider block + login prompt on model switch → Phase 6
- Cross-provider subagent spawn using catalog models → Phase 7
- Custom BYOK / user-defined GPT models as first-class v1 requirement
- Richer capability matrix UI and reasoning-effort first-class TUI settings beyond catalog fields (MOD-V2-01)
- Additional providers beyond xAI + Codex/OpenAI (PROV-V2-01)
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| MOD-01 | Model selector includes current GPT-5.6 family options usable under ChatGPT/Codex OAuth (Sol / Terra / Luna or live Codex catalog IDs), labeled by provider | Wire IDs confirmed `gpt-5.6-{sol,terra,luna}`; UI-SPEC names `GPT-5.6 Sol (Codex)` etc.; ship in `default_models.json` + union-merge after remote prefetch; `provider: codex` + name suffix |
| MOD-02 | Model selector includes Grok/xAI models alongside GPT models in one mixed list | Same flat catalog/list surfaces (`/model`, settings DynamicEnum, `bum models`); no provider-mode filter; Grok first then Sol→Terra→Luna; no Codex-login gate |
</phase_requirements>

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Default catalog SoT (JSON + ids) | Library (`xai-grok-models`) | Shell parse | Embedded `DEFAULT_MODELS_JSON`; shell owns rich field parse |
| Provider binding schema | API / Backend (shell agent config) | — | `ModelInfo.provider` is the routing-ready field for Phase 4 |
| Catalog merge (defaults ∪ remote ∪ config.toml) | API / Backend (`resolve_model_list`) | — | Must preserve Codex defaults when xAI remote replaces Grok list |
| Visibility (hidden / supported_in_api / allowed) | API / Backend (`resolve_model_catalog` / `available_models`) | — | Auth visibility is **xAI session vs API-key**, not Codex login |
| ACP projection | API / Backend (`to_acp_model_info`) | Browser/TUI consumer | Name carries label; optional meta `provider` |
| `/model` slash dropdown | TUI (pager) | Shell catalog | `build_model_items` uses `info.name` / description |
| Settings default-model enum | TUI (pager settings) | Shell catalog | DynamicEnum from `available_models` names |
| `bum models` CLI list | CLI (pager `models.rs`) | Shell ACP list_models | Must print id + provider-bearing name (UI-SPEC) |
| Phase 2 auth slots `xai`/`codex` | Auth storage | Catalog (ids only) | Catalog `provider` values must match; no login this phase |

## Standard Stack

### Core

| Library / component | Version / location | Purpose | Why Standard |
|---------------------|-------------------|---------|--------------|
| Rust workspace | edition 2024, toolchain 1.92.0 | Entire product | PROJECT non-negotiable — fork evolution |
| `xai-grok-models` | crate path `crates/codegen/xai-grok-models` | Embedded `default_models.json` + default id helpers | Existing SoT for default model id |
| `xai-grok-shell` agent config/models | `agent/config.rs`, `agent/models.rs` | Parse, merge, visibility, ACP projection | Existing catalog pipeline |
| `indexmap::IndexMap` | workspace dep | Ordered catalog map | Preserves insertion order for flat list |
| `serde` / `serde_json` | workspace | JSON/TOML model fields | Existing config pattern |
| `agent-client-protocol` | 0.10.4 (workspace) | ACP `ModelInfo` for TUI | Existing picker wire |
| ratatui picker / slash | `xai-grok-pager` | Model selector UI | UI-SPEC surfaces |

### Supporting

| Library / component | Version | Purpose | When to Use |
|---------------------|---------|---------|-------------|
| `serial_test` + `xai-grok-test-support` | workspace | Env-isolated unit tests | Auth/catalog tests that touch env |
| Codex `models_cache.json` (local reference only) | host `~/.codex` | Confirm wire ids / context / effort menus | Research + optional field defaults — **do not** import at runtime |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Explicit `provider` field | Infer from `agent_type == "codex"` | **Rejected by CONTEXT** — harness ≠ auth provider |
| Separate hardcoded picker list for GPT | Extend `default_models.json` | **Rejected** — dual SoT; remote merge wouldn't know about GPT |
| Full replace merge (current) | Union-append Codex defaults after prefetch | Full replace drops GPT on normal login path |
| `agent_type: "codex"` on GPT rows now | Keep default stock harness | Strict harness mismatches block/rebuild switch before routing works |
| Fetch GPT list from OpenAI at runtime | Static embedded entries | Needs Codex OAuth (Phase 5) and is out of scope |

**Installation:** none — no new crates or npm packages. Edit existing Rust types + JSON.

**Version verification:** no new packages. Existing stack verified via workspace manifests and `rustc 1.92.0` on host. [VERIFIED: environment]

## Package Legitimacy Audit

> Phase installs **no** external packages.

| Package | Registry | Age | Downloads | Source Repo | Verdict | Disposition |
|---------|----------|-----|-----------|-------------|---------|-------------|
| — | — | — | — | — | — | N/A — no installs |

**Packages removed due to [SLOP] verdict:** none  
**Packages flagged as suspicious [SUS]:** none

## Architecture Patterns

### System Architecture Diagram

```text
default_models.json (embedded)
        │
        ▼
 default_models() / DefaultModelJson  ──► ModelEntryConfig ──► ModelEntry/ModelInfo
        │                                      │                      │
        │                              config.toml [model.*]          │
        │                                      │                      │
        ▼                                      ▼                      │
 resolve_model_list:                                              provider field
   [if !custom_endpoint] inject bundled defaults                  (xai|codex)
   [if prefetched] replace base with remote map                         │
   [NEW] re-append bundled defaults where                              │
         provider=codex (keys missing)                                  │
   apply config_models overrides                                        │
        │                                                               │
        ▼                                                               │
 resolve_model_catalog (disabled/allowed/hidden globs)                  │
        │                                                               │
        ▼                                                               │
 available_models(is_session_auth)  ◄── xAI OAuth vs API-key only       │
        │                         (NOT Codex credential presence)        │
        ▼                                                               │
 to_acp_model_info ── name (suffix) + meta.agentType + meta.provider ───┘
        │
        ├─► ModelsManager.available() ──► ACP session modelState
        │         │
        │         ├─► pager /model build_model_items (display=name)
        │         ├─► settings DynamicEnum default_model
        │         └─► bum models CLI (id + name)
        │
        └─► Phase 4 (later): provider → credential slot + base URL
```

### Recommended Project Structure (touch points only)

```text
crates/codegen/xai-grok-models/
  default_models.json          # SoT: grok-build + Sol/Terra/Luna + provider
  src/lib.rs                   # still only default *id* helpers (ok)

crates/codegen/xai-grok-shell/src/
  auth/model.rs                # PROVIDER_XAI / PROVIDER_CODEX (reuse ids)
  agent/config.rs              # provider on types + parse + to_acp + resolve_model_list
  agent/models.rs              # catalog tests; available_models unchanged logic
  remote/client.rs             # parse_remote_model_value: set provider=xai default
  cli_models.rs                # data only; display owned by pager

crates/codegen/xai-grok-pager/src/
  models.rs                    # CLI print id (name) per UI-SPEC
  slash/commands/model.rs      # inherits name suffix if catalog correct
  acp/model_state.rs           # no structural change if name carries label
  settings/registry.rs         # inherits available_models names
```

### Pattern 1: Explicit provider on catalog metadata (not credentials)

**What:** Add `provider` to `DefaultModelJson`, `ModelEntryConfig`, `ModelInfo`, and `ConfigModelOverride` with serde default `xai`.  
**When to use:** Always for ship defaults; missing = `xai` for old TOML/JSON/remote.  
**Example:**

```rust
// Recommended shape (planner discretion on enum vs newtype)
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
            Self::Xai => "xai",      // match auth::model::PROVIDER_XAI
            Self::Codex => "codex",  // match auth::model::PROVIDER_CODEX
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

[VERIFIED: Phase 2 constants `PROVIDER_XAI`/`PROVIDER_CODEX`; UI-SPEC labels]

### Pattern 2: Union-append Codex bundled defaults after remote replace

**What:** Keep enterprise full-replace for xAI defaults; after `resolved = prefetched` (and when `!has_custom_endpoint()`), insert each bundled default with `provider == Codex` if key absent.  
**When to use:** Required for MOD-01/02 under normal remote-prefetch login path.  
**Why not full union of all defaults:** Existing tests and enterprise intent prune Grok bundled entries when remote supplies its list (`resolve_model_list_prefetch_replaces_bundled_entirely`). Only multi-provider (Codex) rows need survival. [VERIFIED: codebase tests]

### Pattern 3: Provider label via name suffix (UI-SPEC locked)

**What:** Set `name` in JSON to `Grok Build (xAI)` / `GPT-5.6 Sol (Codex)` etc. All picker surfaces already prefer `info.name`.  
**When to use:** Phase 3 display — no new badge column in `views/picker.rs`.  
**ACP:** Optional `meta["provider"] = "codex"` for machine consumers; UI must not depend on meta alone. [VERIFIED: UI-SPEC + `build_model_items`]

### Pattern 4: agent_type stays stock for GPT this phase

**What:** Leave GPT `agent_type` at default (`grok-build-plan` via `default_agent_type()`).  
**When to use:** Phase 3 only.  
**Why:** `harnesses_are_compatible` treats `codex` as **strict** — stock↔codex is incompatible and triggers zero-turn rebuild / mid-turn reject. Routing still hits xAI until Phase 4; setting strict harness early worsens select UX without benefit. Phase 4/7 can set `agent_type: "codex"` when backend path exists. [VERIFIED: `mvp_agent` harness helpers + tests]

### Anti-Patterns to Avoid

- **Using `agent_type` as provider binding** — breaks Phase 4 and confuses harness rebuild.
- **Hardcoding GPT only in pager slash** — bypasses CLI/settings/ACP and remote merge.
- **Filtering GPT by Codex auth presence** — Phase 6; CONTEXT forbids for Phase 3.
- **Relying only on JSON edit without merge change** — remote prefetch strips GPT.
- **Changing `default` away from `grok-build`** — locked default.
- **Implementing Codex base_url routing “just to make select work”** — Phase 4 scope.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Ordered model map | Custom Vec + HashMap | Existing `IndexMap` catalog | Order + key lookup already wired |
| Picker UI chrome | New modal/badge column | Existing picker + name suffix | UI-SPEC: reuse `views/picker.rs` |
| Provider id strings | New vocabulary | `xai` / `codex` (Phase 2) | Auth slots already use these |
| GPT model discovery | Runtime OpenAI list client | Embedded JSON entries | No Codex OAuth yet |
| Auth-gated catalog | Custom login check in list | Defer to Phase 6 gate | Catalog completeness this phase |
| Display formatting | Per-surface hardcodes | Catalog `name` field | One SoT for all surfaces |

**Key insight:** The hard part is not listing three strings — it is **keeping Codex rows alive across the intentional remote-prefetch prune** while not regressing enterprise custom endpoints.

## Common Pitfalls

### Pitfall 1: Remote prefetch wipes GPT defaults

**What goes wrong:** Logged-in users see only xAI remote models; Sol/Terra/Luna never appear.  
**Why it happens:** `resolve_model_list` does `resolved = prefetched` after loading defaults.  
**How to avoid:** Union-append bundled `provider=codex` entries when `!has_custom_endpoint()`.  
**Warning signs:** Unit tests pass with `prefetched: None` but fail with non-empty prefetched map; manual login session missing GPT.  
**Tests that will need updates:** `resolve_model_list_keeps_prefetch_only_entries_and_prunes_defaults`, `resolve_model_list_prefetch_visibility_matches_auth_and_server_list` (counts change when GPT always-visible), any assert that catalog length equals only prefetched keys. [VERIFIED: codebase]

### Pitfall 2: Confusing `supported_in_api` with Codex login

**What goes wrong:** Plan adds “hide GPT without Codex OAuth” or mis-sets `supported_in_api: false` so API-key users never see GPT.  
**Why it happens:** `visible_for_auth(is_session_auth)` is about **xAI session OAuth vs API key**, not multi-provider.  
**How to avoid:** GPT rows `supported_in_api: true`; never gate on `providers.codex` this phase.  
**Warning signs:** GPT missing when using `XAI_API_KEY` only.

### Pitfall 3: Setting `agent_type: "codex"` too early

**What goes wrong:** Selecting GPT from a stock session hits `MODEL_SWITCH_INCOMPATIBLE_AGENT` or forces harness rebuild with no working Codex route.  
**Why it happens:** Strict harness rules in `harnesses_are_compatible`.  
**How to avoid:** Keep default agent_type on GPT until Phase 4/7.  
**Warning signs:** `/model` selection errors mentioning agent type.

### Pitfall 4: Struct-literal compile fallout

**What goes wrong:** Adding `provider` to `ModelInfo` / `ModelEntryConfig` breaks many test helpers and `parse_remote_model_value` struct literals.  
**Why it happens:** No `..Default` on those structs.  
**How to avoid:** `#[serde(default)]` + `Default` on `ModelProvider`; update `test_model_entry`, `make_entry_config*`, `ModelInfo::fallback` / `from_config`, remote parser, and `default_models()` mapping in the same plan wave.  
**Warning signs:** Large compile error fan-out in shell crate.

### Pitfall 5: CLI still prints id-only

**What goes wrong:** TUI shows `(Codex)` names but `bum models` does not.  
**Why it happens:** `pager/src/models.rs` prints only `m.model_id.0`.  
**How to avoid:** Print `* {id} ({name})` / `- {id} ({name})` per UI-SPEC (ACP `ModelInfo` already has `name`). [VERIFIED: UI-SPEC + `models.rs`]

### Pitfall 6: Empty / custom endpoint edge cases

**What goes wrong:** GPT leaks into enterprise-only catalogs, or empty prefetch edge cases become ambiguous.  
**How to avoid:** Gate Codex re-insert on `!endpoints.has_custom_endpoint()`; keep enterprise skip of built-in defaults. Document empty `Some(IndexMap::new())` test expectation update if Codex rows are re-inserted.

### Pitfall 7: Order instability after union-append

**What goes wrong:** GPT appear mid-list.  
**How to avoid:** Append Codex defaults **after** prefetched map (Grok remote first, then Sol→Terra→Luna JSON order). With no prefetch, JSON order is Grok then Sol→Terra→Luna.

## Code Examples

### Target `default_models.json` shape (illustrative)

```json
{
  "default": "grok-build",
  "web_search": "grok-4.20-multi-agent",
  "image_description": "grok-build",
  "session_summary": "grok-build",
  "models": [
    {
      "model": "grok-build",
      "name": "Grok Build (xAI)",
      "description": "Best for advanced coding tasks",
      "provider": "xai",
      "context_window": 500000,
      "temperature": 0.7,
      "top_p": 0.95,
      "api_backend": "responses",
      "supported_in_api": false
    },
    {
      "model": "gpt-5.6-sol",
      "name": "GPT-5.6 Sol (Codex)",
      "description": "GPT-5.6 Sol — Codex / ChatGPT",
      "provider": "codex",
      "context_window": 272000,
      "api_backend": "responses",
      "supported_in_api": true
    },
    {
      "model": "gpt-5.6-terra",
      "name": "GPT-5.6 Terra (Codex)",
      "description": "GPT-5.6 Terra — Codex / ChatGPT",
      "provider": "codex",
      "context_window": 272000,
      "api_backend": "responses",
      "supported_in_api": true
    },
    {
      "model": "gpt-5.6-luna",
      "name": "GPT-5.6 Luna (Codex)",
      "description": "GPT-5.6 Luna — Codex / ChatGPT",
      "provider": "codex",
      "context_window": 272000,
      "api_backend": "responses",
      "supported_in_api": true
    }
  ]
}
```

Display/description strings: **UI-SPEC locked**. Wire IDs and `context_window: 272000`: **Codex models_cache**. [VERIFIED: UI-SPEC + local Codex cache]

### Merge sketch (resolve_model_list)

```rust
// After applying prefetched (existing replace) and BEFORE config_models loop:
if !cfg.endpoints.has_custom_endpoint() {
    for (key, entry) in default_model_entries(&cfg.endpoints) {
        if entry.info.provider == ModelProvider::Codex && !resolved.contains_key(&key) {
            resolved.insert(key, entry);
        }
    }
}
```

[ASSUMED: exact helper method name; behavior verified as required by merge analysis]

### ACP meta (optional)

```rust
map.insert(
    "provider".to_string(),
    serde_json::Value::String(info.provider.as_str().to_owned()),
);
// Keep existing agentType meta — distinct meaning
```

### CLI list (UI-SPEC)

```rust
// Current: println!("  * {} (default)", m.model_id.0);
// Target:
//   * {id} ({name})
//   - {id} ({name})
// Prefer ACP ModelInfo.name; fall back to id if empty.
```

## State of the Art

| Old Approach | Current Approach (this phase) | When Changed | Impact |
|--------------|-------------------------------|--------------|--------|
| Single Grok default catalog | Mixed Grok + GPT-5.6 with provider tags | Phase 3 | Enables multi-provider picker |
| Infer backend from session only | Per-entry `provider` for later routing | Phase 3 schema; Phase 4 consume | No silent wrong-slot routing later |
| Prefetch replaces all defaults | Prefetch replaces xAI list; Codex defaults re-appended | Phase 3 merge fix | GPT visible while logged into xAI |
| GPT via Platform API key mental model | Codex/ChatGPT OAuth path (later) | Phase 4–5 | Catalog still lists GPT before OAuth |

**Deprecated/outdated:**
- Treating catalog as “xAI models only once remote fetch succeeds”
- Using `agent_type` as the multi-provider discriminator

## GPT-5.6 Wire ID Findings

| Catalog id / model slug | Display (UI-SPEC) | Codex priority | context_window | Notes |
|-------------------------|-------------------|----------------|----------------|-------|
| `gpt-5.6-sol` | GPT-5.6 Sol (Codex) | 1 | 272000 | Flagship; bare `gpt-5.6` alias often → Sol |
| `gpt-5.6-terra` | GPT-5.6 Terra (Codex) | 2 | 272000 | Balanced / everyday |
| `gpt-5.6-luna` | GPT-5.6 Luna (Codex) | 3 | 272000 | Fast / affordable |

**Recommendation:** Keep CONTEXT ids as-is — they match Codex cache slugs. Do not use bare `gpt-5.6` as a catalog key. Optional later: reasoning_efforts menus from Codex (low/medium/high/xhigh/max[/ultra]) — **discretion**; not required for MOD-01/02 list visibility. [VERIFIED: local Codex models_cache; web corroboration for family naming]

**base_url for GPT this phase:** Accept whatever `default_models()` assigns via `endpoints.resolve_inference_base_url()` (xAI proxy defaults). Phase 4 will select Codex endpoints by `provider`. Do not invent a production Codex base URL here without Phase 4 research. [VERIFIED: `default_models()` mapping]

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Union-append only `provider=codex` defaults (not all defaults) is the right merge compromise | Architecture | If product wants all bundled Grok defaults to survive remote too, merge policy differs |
| A2 | Leaving GPT `agent_type` at stock default is preferred for Phase 3 | Patterns | If planners set `codex` harness now, switch UX regressions appear early |
| A3 | Reasoning-effort menus on GPT rows are optional for Phase 3 | GPT Wire IDs | Subagent effort (AGENT-03) may want them sooner; can add without routing |
| A4 | `web_search` default id `grok-4.20-multi-agent` may be absent from catalog; leave as-is | Code Examples | Pre-existing; out of scope unless tests assert presence |

**If empty:** N/A — assumptions listed above.

## Open Questions

### Q1 — Empty prefetched map Codex inject — **RESOLVED**

**Decision:** Empty `Some(IndexMap::new())` **still receives** bundled Codex defaults when `!has_custom_endpoint()`.

- Rationale: Fetch failures use `None` (full defaults path). An empty `Some` is a degenerate remote list; re-appending Codex-only bundles matches D-03/D-09 mixed-catalog intent without resurrecting pruned xAI defaults.
- Plan impact: Plan 02 encodes `empty_prefetch_still_gets_codex_defaults` in `tests/model_catalog.rs`; do not keep empty-base as the phase gate.

### Q2 — Rewrite remote xAI names with `(xAI)` — **RESOLVED**

**Decision:** **Do not** rewrite remote-prefetched Grok display names this phase.

- Rationale: UI-SPEC locks embedded default JSON names; remote overrides matching keys keep server-provided labels. GPT bundled names + explicit `provider` field carry Phase 3 labeling; optional polish later.
- Plan impact: Plan 01 sets remote parse `provider = Xai` only; Plan 03 ACP/CLI pass through catalog `name` without suffix synthesis on remote rows.

### Q3 — Shell/pager `--lib` harness health — **RESOLVED**

**Decision:** Wave 0 and all Phase 3 gates use **integration tests on public APIs** — not repair of shell/pager `--lib` suites.

- What we know: `cargo test -p xai-grok-shell --lib` fails to compile (~cross-crate `cfg(test)` leakage, missing test-only re-exports); pager `--lib` similarly unhealthy at scale. Do **not** schedule fixing ~32 shell / ~169 pager lib-test errors in Phase 3.
- Plan impact:
  - Shell: `crates/codegen/xai-grok-shell/tests/model_catalog.rs` via `cargo test -p xai-grok-shell --test model_catalog …` against `resolve_model_list`, `available_models`, `to_acp_model_info`, `Config`, `ModelEntry`, `ModelProvider`.
  - Pager CLI: make `format_cli_model_row` **pub**; `crates/codegen/xai-grok-pager/tests/format_cli_model_row.rs` via `cargo test -p xai-grok-pager --test format_cli_model_row …`.
  - Models crate: `cargo test -p xai-grok-models --lib` remains OK.
- See `03-VALIDATION.md` for full phase gate and sampling rates.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| rustc / cargo | Build + tests | ✓ | 1.92.0 | — |
| Workspace crates (shell, models, pager) | Implementation | ✓ | in-tree | — |
| protoc | Unrelated full workspace builds | ✓ (repo bin) | — | Not needed for catalog-only |
| Network / OpenAI API | Live GPT turns | N/A | — | Not required Phase 3 |
| Codex OAuth | Live GPT auth | N/A | — | Phase 5 |

**Missing dependencies with no fallback:** none for this phase.

**Missing dependencies with fallback:** none.

## Validation Architecture

> `workflow.nyquist_validation` is **true** in `.planning/config.json`.
> Canonical per-phase contract: `03-VALIDATION.md` (authoritative after plan revision).

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Built-in `cargo test` — **integration preferred** for shell/pager (Q3 RESOLVED) |
| Config file | none (Cargo defaults) |
| Quick run command | `cargo test -p xai-grok-shell --test model_catalog -- --nocapture` |
| Full suite command | `cargo test -p xai-grok-models --lib && cargo test -p xai-grok-shell --test model_catalog -- --nocapture && cargo test -p xai-grok-pager --test format_cli_model_row -- --nocapture` |
| Forbidden gates | `cargo test -p xai-grok-shell --lib …` and `cargo test -p xai-grok-pager --lib …` for Phase 3 |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| MOD-01 | Catalog contains Sol/Terra/Luna with provider=codex and Codex-labeled names | integration | `cargo test -p xai-grok-shell --test model_catalog catalog_includes_gpt56 -- --nocapture` | ❌ Wave 0 |
| MOD-01 | Names match UI-SPEC / provider suffix | integration | same + inherit name-suffix assert in model_catalog | ❌ Wave 0 |
| MOD-02 | Mixed list: Grok + GPT together; order Grok then Sol→Terra→Luna when no prefetch | integration | `cargo test -p xai-grok-shell --test model_catalog mixed_catalog_order -- --nocapture` | ❌ Wave 0 |
| MOD-02 | Prefetched xAI-only map still includes GPT rows | integration | `cargo test -p xai-grok-shell --test model_catalog codex_defaults_survive_prefetch -- --nocapture` | ❌ Wave 0 |
| MOD-02 | Custom models endpoint does **not** inject GPT | integration | `… custom_endpoint_skips_codex_inject` in model_catalog | ❌ Wave 0 |
| MOD-02 | Empty `Some({})` still injects Codex when `!has_custom_endpoint` (Q1) | integration | `… empty_prefetch_still_gets_codex_defaults` | ❌ Wave 0 |
| Success #3 | Every entry has provider; missing defaults to xai | integration | `cargo test -p xai-grok-shell --test model_catalog provider_default_xai -- --nocapture` | ❌ Wave 0 |
| Success #3 | `to_acp_model_info` exposes name + meta.provider | integration | `cargo test -p xai-grok-shell --test model_catalog to_acp_model_info -- --nocapture` | ❌ Wave 0 |
| MOD-01/02 | GPT visible for API-key auth (`supported_in_api`) and OAuth; not filtered by Codex credentials | integration | `cargo test -p xai-grok-shell --test model_catalog gpt_visible -- --nocapture` | ❌ Wave 0 |
| Default | `default` remains `grok-build` / `default_model()` | unit + integration | `cargo test -p xai-grok-models --lib` + model_catalog assert | ✅ partial |
| CLI | `bum models` format id + name | integration | `cargo test -p xai-grok-pager --test format_cli_model_row -- --nocapture` | ❌ Wave 0 (Plan 03) |

### Sampling Rate

- **Per task commit:** targeted filter on the integration binary touched  
- **Per wave merge:** full suite command above  
- **Phase gate:** all Phase 3 requirement tests green before `/gsd:verify-work` (see 03-VALIDATION.md)

### Wave 0 Gaps

- [ ] `crates/codegen/xai-grok-shell/tests/model_catalog.rs` — mixed catalog + provider binding + prefetch survival + ACP meta (public APIs only)
- [ ] Prefetch contracts that replace empty-base expectation with Q1 Codex inject
- [ ] Optional: lightweight `xai-grok-models` test that JSON `default` ∈ models ids after GPT add (existing LazyLock assert already)
- [ ] `crates/codegen/xai-grok-pager/tests/format_cli_model_row.rs` + `pub format_cli_model_row` (Plan 03)
- [x] ~~Confirm shell `--lib` health~~ — **RESOLVED Q3:** do not use `--lib`; use integration tests only

*(Do not plan repair of entire shell/pager lib-test suites in this phase.)*

## Security Domain

> `security_enforcement` enabled (ASVS level 1).

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no (Phase 3) | Catalog only; auth slots exist but unused for gating |
| V3 Session Management | no | — |
| V4 Access Control | partial | Do not introduce authz bypass; listing is intentional |
| V5 Input Validation | yes | Serde parse of `provider` from TOML/JSON; unknown values → error or default `xai` (prefer **reject unknown** in user TOML, **default** for remote missing) |
| V6 Cryptography | no | No new crypto |

### Known Threat Patterns for catalog changes

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Malicious config.toml sets provider=xai on GPT id to steal routes later | Elevation / Spoofing | Phase 4 must trust binding carefully; Phase 3 document that provider is user-configurable like base_url (BYOK already allows arbitrary endpoints) |
| Catalog injection via remote models adding provider=codex | Spoofing | Remote parser defaults provider to `xai`; only bundled JSON sets `codex` unless user TOML overrides |
| Accidentally shipping secrets in model entries | Info disclosure | Do not put API keys in `default_models.json` (existing pattern) |

## Project Constraints (from AGENTS.md / PROJECT)

- Stay on Rust workspace fork (no rewrite)
- Mixed picker, per-model routing later — **no global provider mode**
- `~/.bum` isolation; Phase 2 multi-slot `xai`/`codex`
- Product CLI `bum`
- Phase 3 must not implement Codex OAuth, routing, or missing-provider gate

## Sources

### Primary (HIGH confidence)

- Codebase: `crates/codegen/xai-grok-models/default_models.json`, `src/lib.rs`
- Codebase: `crates/codegen/xai-grok-shell/src/agent/config.rs` (`DefaultModelJson`, `ModelEntryConfig`, `ModelInfo`, `resolve_model_list`, `to_acp_model_info`, prefetch-replace tests)
- Codebase: `crates/codegen/xai-grok-shell/src/agent/models.rs` (`available_models`, `resolve_model_catalog`, prefetch fetch path)
- Codebase: `crates/codegen/xai-grok-shell/src/auth/model.rs` (`PROVIDER_XAI`, `PROVIDER_CODEX`)
- Codebase: `crates/codegen/xai-grok-shell/src/agent/mvp_agent/mod.rs` (`harnesses_are_compatible`)
- Codebase: `crates/codegen/xai-grok-pager/src/{models.rs,slash/commands/model.rs,acp/model_state.rs,settings/registry.rs}`
- Phase artifacts: `03-CONTEXT.md`, `03-UI-SPEC.md`, `REQUIREMENTS.md` MOD-01/02, `02-CONTEXT.md`
- Local Codex: `~/.codex/models_cache.json` slugs `gpt-5.6-sol|terra|luna`, `context_window=272000`

### Secondary (MEDIUM confidence)

- OpenAI GPT-5.6 family announcements (Sol/Terra/Luna tiers; Codex + API availability)
- Third-party API ID pages confirming `gpt-5.6-terra` etc. (corroborates local Codex cache)

### Tertiary (LOW confidence)

- Optional reasoning-effort menus for GPT rows (from Codex cache) — nice-to-have, not required for MOD-01/02

## Metadata

**Confidence breakdown:**
- Standard stack: **HIGH** — in-tree, no new deps
- Architecture: **HIGH** — merge replace + types fully traced in code
- Pitfalls: **HIGH** — backed by existing unit tests that will break if merge ignored
- Wire IDs: **HIGH** — Codex local cache + consistent web IDs
- Runtime agent_type recommendation: **MEDIUM** — solid code evidence; product preference could still set codex harness early

**Research date:** 2026-07-16  
**Valid until:** ~30 days (catalog schema stable; GPT wire ids may gain aliases — recheck Codex cache at implement if >30d)

## Planner Task Hints (non-binding)

Suggested plan waves:

1. **Schema + JSON:** `ModelProvider`, fields on JSON/config/info/override, remote parse default, `default_models.json` entries + grok name/provider, fix all constructors/fallback/from_config.
2. **Merge policy:** Codex default re-insert after prefetch; update prune tests; enterprise still clean.
3. **Projection + CLI:** `to_acp_model_info` meta; `pager/src/models.rs` name display; confirm slash/settings inherit names.
4. **Tests / Wave 0:** MOD-01/02 automated proofs; default remains grok-build; visibility without Codex login.
)
