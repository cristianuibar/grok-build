# Phase 3 — UI Review

**Audited:** 2026-07-16  
**Baseline:** `03-UI-SPEC.md` (approved design contract; surface: terminal-tui / ratatui)  
**Screenshots:** not captured (no web dev server; product is CLI/TUI — code-only audit)  
**Registry audit:** skipped (`shadcn_initialized: false`; TUI stack)  
**Stance:** adversarial — catalog/picker/CLI surfaces scored against UI-SPEC, not softened for “it works”

---

## Pillar Scores

| Pillar | Score | Key Finding |
|--------|-------|-------------|
| 1. Copywriting | 3/4 | Locked catalog names/descriptions match UI-SPEC; settings description injects literal backticks not in contract |
| 2. Visuals | 3/4 | Provider-in-name hierarchy works; end-ellipsis `truncate_str` can clip `(Codex)` / `(xAI)` on narrow rows |
| 3. Color | 2/4 | No per-provider colors (good); prompt model name uses dim `text_secondary` blend, not reserved `accent_model` |
| 4. Typography | 4/4 | Monospaced roles: body / label / secondary; bold only on selected row — matches 2-weight contract |
| 5. Spacing | 4/4 | Picker geometry locked: fold/prefix 2, trailing pad 1, right-label gap 2, indent step 2 |
| 6. Experience Design | 3/4 | Empty catalog / unknown pick / GPT-without-auth covered; interactive `/model` UAT never run |

**Overall: 19/24**

---

## Top 3 Priority Fixes

1. **Prompt info-line model name ignores `theme.accent_model`** — UI-SPEC reserves teal accent for the current model chrome and lists it as surface (4); users cannot visually anchor the active model with the declared accent. — In `prompt_widget::render_info_line`, style `info.model_name` with `Style::default().fg(theme.accent_model).bg(bg)` (keep unfocused fade if desired) instead of `chrome_caption_style` only.

2. **Settings “Default model” description has literal backticks** — Users see `Pick \`(no override)\` to clear.` with visible `` ` `` characters (`Span::raw`), not the contract copy. — Change `settings/defs.rs` description to `Pick (no override) to clear.` exactly as UI-SPEC, or render markdown if intentional mono styling is desired.

3. **Long-label truncation drops the provider suffix first** — On narrow terminals, `truncate_str(row.label, …)` / slash label truncate is end-ellipsis, so `(Codex)` / `(xAI)` is the first thing to disappear — weakening MOD-02 visibility. — Prefer truncating the family segment before the final ` (Provider)` suffix (UI-SPEC long-text preference); add a unit test for a 20-col budget retaining `(Codex)`.

---

## Detailed Findings

### Pillar 1: Copywriting (3/4)

**What matches**

| Contract | Implementation | Evidence |
|----------|----------------|----------|
| `Grok Build (xAI)` + desc | Exact | `default_models.json` L8–10 |
| `GPT-5.6 Sol/Terra/Luna (Codex)` + descs | Exact | `default_models.json` L19–39 |
| Provider human labels `xAI` / `Codex` | Exact | name suffixes; not raw `xai`/`codex` in display |
| Format `{Family} ({Provider})` | Exact | space before `(` |
| Slash description `Switch the active model` | Exact | `slash/commands/model.rs:26` |
| Slash usage `/model <name> [effort]` | Exact | `model.rs:40` |
| Error empty args `Usage: /model <name> [effort]` | Exact | `model.rs:70` |
| Error unknown `Unknown model: {input}` | Exact on slash | `model.rs:103` |
| Current slash suffix `{name} (current)` | Exact | `build_model_items` L160–164 → e.g. `Grok Build (xAI) (current)` |
| Settings label `Default model` | Exact | `settings/defs.rs:762` |
| CLI header `Available models:` | Exact | `models.rs:42` |
| CLI default `Default model: {id}` | Exact | `models.rs:40` |
| CLI rows `  *\|- {id} ({name})` | Exact | `format_cli_model_row` + tests |
| No `(default)` text on current CLI row | Exact | tests assert absence |

**Findings**

- **WARNING — settings description drift** (`settings/defs.rs:763`):  
  Code: `Pick \`(no override)\` to clear.`  
  UI-SPEC: `Pick (no override) to clear.`  
  Descriptions render via `Span::raw` / plain wrap — backticks are literal glyphs, not markdown.

- **WARNING — settings unknown-model quotes** (`settings_modal.rs` ~826):  
  `Unknown model: "{buffer}"` adds quotes; UI-SPEC template is `Unknown model: {input}` (slash path is correct without quotes). Minor inconsistency across surfaces.

- **OK — empty-state strings N/A for slash:** `suggest_args` returns `None` when `models.is_empty()` (`model.rs:56–58`); matches UI-SPEC authority. Reserved “No models available” copy is unused (acceptable).

- **OK — CLI auth preamble retained** without filtering GPT rows (`models.rs:24–32` + Phase 02 visibility proofs).

### Pillar 2: Visuals (3/4)

**What matches**

- Single flat mixed list — no provider tabs/filter chrome (catalog + inherit-only slash/settings).
- Provider identity is **name suffix**, not a separate badge column (reuse picker/slash).
- Focal hierarchy: primary label = model name with provider; secondary = catalog description on `/model` rows (`build_model_items` sets `description` from ACP).
- Settings DynamicEnum is **names-only** (empty model-row descriptions) per UI-SPEC (`dynamic_enum_choices` L121–126; integration test).
- Selected row: bold + `bg_visual` / hover `bg_hover` (picker + slash dropdown patterns).
- CLI uses star vs dash markers only — no competing default-text chrome.

**Findings**

- **WARNING — provider suffix lost under end truncation:**  
  `picker.rs:812` and `slash_dropdown.rs:306` use `truncate_str` on the full display string. UI-SPEC long-text row prefers keeping ` (xAI)` / ` (Codex)` visible. Phase 3 accepted reuse-only but left no test note documenting the tradeoff.

- **WARNING — CLI nested parentheses readability:**  
  Spec-compliant shape `  * grok-build (Grok Build (xAI))` nests provider parens inside CLI parens. Machine-friendly; human scan is slightly noisier. Not a contract break; polish candidate later.

- **INFO — no interactive visual proof:** Plan 03 Task 4 optional `/model` + Settings UAT was skipped (no TUI session). Hierarchy and glyph chrome inferred from code + unit tests only.

### Pillar 3: Color (2/4)

**What matches**

- **No per-provider accent colors** this phase — provider is plain text in the name (UI-SPEC non-goal met).
- Fuzzy match highlights on `/model` arg dropdown use `theme.fuzzy_accent` (`slash_dropdown.rs:285–291`) — accent reservation #2 met.
- Catalog rows do not use `accent_error` / destructive styling.

**Findings**

- **WARNING (contract miss) — prompt model name is not `accent_model`:**  
  UI-SPEC Color table + Design System surface (4): *“prompt status model name via `accent_model`”* and accent reserved for *“Current model name in the prompt status / info line (`accent_model`)”*.  
  Actual (`prompt_widget/mod.rs:3325–3351`):

  ```rust
  let model_style = Self::chrome_caption_style(bg, theme, focused);
  // chrome_caption_style → blend toward theme.text_secondary
  left_spans.push(Span::styled(info.model_name, model_style));
  ```

  Repo-wide, `theme.accent_model` is **not** applied to the prompt model name (only plugin CTA + list-dir tool glyph). After Phase 3 expands names to include provider suffixes, the active model line still lacks the declared teal (GrokNight) accent.

- **OK — no hard-coded provider hex** in catalog surfaces; themes keep role tokens.

### Pillar 4: Typography (4/4)

**What matches**

| Role | Contract | Implementation |
|------|----------|----------------|
| Body (primary row) | regular / **bold when selected** | picker selected → `Modifier::BOLD` + `text_primary`; slash same |
| Label (provider suffix, CLI bullets) | regular; CLI `* `/`- ` dim not required | suffix is part of name string; CLI plain stdout |
| Secondary (description) | gray / secondary | slash_dropdown `theme.gray` for description |

- Max **2** weights on catalog rows: regular + bold selected — no italic on model names.
- No Display-size titles introduced for this phase.
- Terminal monospaced host font; no app font stack.

**Findings**

- None that break the typography contract. Score **4**.

### Pillar 5: Spacing (4/4)

**What matches (cell geometry vs UI-SPEC)**

| Element | Contract | Code |
|---------|----------|------|
| Fold/prefix width | 2 cells | `picker.rs:788` `fold_width: u16 = 2` |
| Trailing pad | 1 cell | `picker.rs:801` `trailing_pad = 1` |
| Right-label gap | 2 when non-empty | `picker.rs:809` `gap = 2` |
| Indent step | 2 cells | `picker.rs:778` `"  ".repeat(row.indent)` |
| Description indent | md-ish | `DESC_INDENT = 4` (`picker.rs:690`) |

- No invented rem/px CSS spacing; Phase 3 did not introduce wide gutters (`xl`/`2xl`/`3xl`).
- List row height remains 1 cell + optional description wrap.

**Findings**

- None for Phase 3 catalog work. Score **4**.

### Pillar 6: Experience Design (3/4)

**State coverage vs UI Considerations table**

| State | Contract | Evidence | Status |
|-------|----------|----------|--------|
| empty catalog → slash | `suggest_args` → `None` | `model.rs:56–58`; unit test `model_suggest_args_empty_models_returns_none` | ✅ |
| empty settings enum | sentinel only | `dynamic_enum_choices_empty_catalog_is_sentinel_only` | ✅ |
| fuzzy no-matches | empty filtered list, no crash | existing slash fuzzy matcher + dropdown | ✅ |
| loading / remote | no spinner; GPT must not vanish | remove-then-append Codex after prefetch (`config.rs`); integration tests | ✅ |
| error unknown pick | `Unknown model: …` | slash + settings | ✅ |
| populated order | Grok first, Sol→Terra→Luna | `mixed_catalog_order` test + JSON order | ✅ |
| GPT without Codex login | always listed | `gpt_visible_*` tests; no auth filter in CLI | ✅ |
| partial name | fall back to id | `to_acp_model_info_name_falls_back_to_model_id`; CLI empty name → id | ✅ |
| ACP machine meta | `meta.provider` always | `to_acp_model_info` L4906–4911; tests | ✅ |
| default model | `grok-build` | `default_models.json` + tests | ✅ |

**Findings**

- **WARNING — interactive surface UAT skipped:** Plan 03 Task 4 (advisory) never exercised live `/model` dropdown or Settings → Default model. Automated projection tests cover strings/order/meta, not keyboard focus chrome or overflow at real terminal widths.

- **WARNING — CLI empty list silent:** If `available_models` is empty, CLI prints headers and zero rows with no “No models available” body. UI-SPEC reserves that copy for panels that render empty lists; slash correctly returns no dropdown. Low severity (defaults always ship rows).

- **OK — no destructive confirmation** this phase (none required).

- **OK — no missing-provider gate** (correctly deferred Phase 6).

---

## Registry Safety

Not applicable — `shadcn_initialized: false`, no third-party UI registries, ratatui TUI stack.  
`Registry audit: 0 third-party blocks checked, no flags`

---

## Files Audited

| File | Role in audit |
|------|----------------|
| `.planning/phases/03-model-catalog-gpt-5-6-entries/03-UI-SPEC.md` | Design contract baseline |
| `.planning/phases/03-model-catalog-gpt-5-6-entries/03-CONTEXT.md` | Scope / decisions |
| `.planning/phases/03-model-catalog-gpt-5-6-entries/03-0{1,2,3}-SUMMARY.md` | What was built |
| `crates/codegen/xai-grok-models/default_models.json` | Catalog SoT names/descriptions/providers |
| `crates/codegen/xai-grok-shell/src/agent/config.rs` | `to_acp_model_info`, provider, resolve list |
| `crates/codegen/xai-grok-shell/tests/model_catalog.rs` | Order, suffixes, meta.provider, visibility |
| `crates/codegen/xai-grok-pager/src/models.rs` | CLI list + `format_cli_model_row` |
| `crates/codegen/xai-grok-pager/tests/format_cli_model_row.rs` | CLI shape proofs |
| `crates/codegen/xai-grok-pager/src/slash/commands/model.rs` | `/model` copy, items, empty/error |
| `crates/codegen/xai-grok-pager/src/settings/defs.rs` | Default model label/description |
| `crates/codegen/xai-grok-pager/src/settings/registry.rs` | DynamicEnum choices |
| `crates/codegen/xai-grok-pager/tests/dynamic_enum_model_names.rs` | Settings name-suffix proofs |
| `crates/codegen/xai-grok-pager/src/views/picker.rs` | Spacing geometry, truncate, selection styles |
| `crates/codegen/xai-grok-pager/src/views/slash_dropdown.rs` | Fuzzy accent, bold selected, desc gray |
| `crates/codegen/xai-grok-pager/src/views/prompt_widget/mod.rs` | Prompt model name color (accent miss) |
| `crates/codegen/xai-grok-pager/src/acp/model_state.rs` | Current model name projection |
| `crates/codegen/xai-grok-pager/src/views/settings_modal.rs` | Unknown model error format |

---

## Scoring notes (anti-softening)

- Color is **2**, not 3: the UI-SPEC’s primary accent reservation for this phase’s named surface (prompt model chrome) is unimplemented. Passing “no bad provider colors” alone is not a Color pass.
- Copywriting is **3**, not 4: locked catalog content is excellent, but shipped settings description diverges from the contract string with visible backticks.
- Experience is **3**, not 4: automated gates are strong; the only interactive visual checkpoint was skipped, and truncation/overflow of provider labels remains unproven at real widths.
- Typography and Spacing score **4** because Phase 3 correctly reused existing chrome without inventing weights or gutters outside the cell table.

---

## Minor recommendations (beyond Top 3)

4. Align settings unknown-model copy with slash (`Unknown model: {input}` without extra quotes).  
5. Optional: document CLI nested-parens as intentional scripting format in user-facing help.  
6. When interactive environment exists, run Plan 03 Task 4 checklist (`/model` list + Settings → Default model) and attach notes to this review.  
7. If prompt chrome intentionally dimmed model names post-redesign, **update UI-SPEC** Color/accent reservation rather than leaving the contract stale.
