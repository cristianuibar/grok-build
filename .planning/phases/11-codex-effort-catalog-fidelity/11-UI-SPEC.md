---
phase: 11
slug: codex-effort-catalog-fidelity
status: draft
shadcn_initialized: false
preset: none
created: 2026-07-21
---

# Phase 11 — UI Design Contract

> Visual and interaction contract for the effort soft-clamp surface. Terminal TUI phase — web
> design-system rows are declared not-applicable; the binding contracts are Copywriting,
> Interaction states, and reuse of existing pager theme tokens.

---

## Design System

| Property | Value |
|----------|-------|
| Tool | none (Rust TUI — existing `xai-grok-pager` render/theme stack) |
| Preset | not applicable |
| Component library | existing pager widgets only (transcript system line, `/model` chained effort menu) |
| Icon library | none — plain text glyphs already used by pager status lines |
| Font | terminal-inherited (no font control) |

---

## Spacing Scale

Terminal cell grid — px scale not applicable. Declared usage:

| Token | Value | Usage |
|-------|-------|-------|
| line | 1 row | Clamp notice occupies exactly one transcript/system line |
| indent | existing | Notice uses the same left indent as existing system/status messages |

Exceptions: none — no new spacing primitives; reuse existing pager layout.

---

## Typography

Terminal-inherited. One role used:

| Role | Size | Weight | Line Height |
|------|------|--------|-------------|
| System notice | terminal cell | existing system-message style (dim/informational) | 1 |

No new text styles — the clamp notice MUST reuse the existing informational/system message
style already rendered by the pager (same style family as the headless
"model does not support reasoning effort; ignoring" class of messages).

---

## Color

| Role | Value | Usage |
|------|-------|-------|
| Dominant (60%) | existing theme background | unchanged |
| Secondary (30%) | existing theme surfaces | unchanged |
| Accent (10%) | existing theme accent | unchanged — notice does NOT use accent |
| Destructive | existing theme error | NOT used — clamp is informational, never an error style |

Accent reserved for: existing pager accents only. The clamp notice renders in the existing
dim/informational system style — explicitly not error-red (soft-clamp is not a failure).

---

## Copywriting Contract

| Element | Copy |
|---------|------|
| Clamp notice (effort adjusted) | `reasoning effort clamped to {level} ({model} supports {list})` — one line, lowercase level names, no trailing period |
| No-clamp case | no output — silence when the active effort is already supported |
| Unsupported-model case (empty effort list) | existing behavior preserved: effort omitted from wire; existing "model does not support reasoning effort" messaging unchanged where already present |
| Effort picker rows | unchanged — existing `/model` chained effort phase rows (low / medium / high / xhigh) |
| Error state | none introduced — clamp path must never produce an error or hard-fail the turn |

---

## UI Considerations

Applicable state considerations resolved: 13 covered, 2 backstop, 0 unresolved

| Category | Element(s) | Status | Resolution / Reason |
|----------|------------|--------|---------------------|
| empty | effort-clamp-notice | ✅ covered | No-clamp turns render nothing — notice appears only when clamp actually changed the effective level |
| loading | effort-clamp-notice | ✅ covered | No loading state exists — notice is emitted synchronously at request-build from embedded catalog data |
| error | effort-clamp-notice | ✅ covered | Clamp path has no error state by design — informational style only; turn proceeds with clamped level |
| populated | effort-clamp-notice | ✅ covered | Happy path is the locked copy: one line, clamped level + supported list (Copywriting Contract row 1) |
| partial | effort-clamp-notice | ✅ covered | Partial data impossible — level and supported list come from the same catalog entry atomically |
| overflow | effort-clamp-notice | 🧪 backstop | `{ statement: "clamp notice renders as one line with the full supported list at max ladder length", verification: backstop }` |
| zero-one-many | effort-clamp-notice | ✅ covered | At most one notice per clamp event; each subsequent clamping switch emits its own single line (no stacking/dedup) |
| long-text | effort-clamp-notice | 🧪 backstop | Level names bounded to known ladder; same single-line backstop test as overflow |
| empty | effort-picker-menu | ✅ covered | Model with empty supported-effort list: picker phase soft-skips (existing behavior); wire omits `reasoning.effort` |
| loading | effort-picker-menu | ✅ covered | Catalog is embedded — menu builds synchronously; no async load state |
| error | effort-picker-menu | ✅ covered | Unknown effort input handled by existing parse error path (unchanged this phase) |
| populated | effort-picker-menu | ✅ covered | One row per supported level (existing `/model` chained phase — unchanged) |
| partial | effort-picker-menu | ✅ covered | Supported list read atomically from one catalog entry — no partial menu possible |
| overflow | effort-picker-menu | ✅ covered | Ladder bounded to 4 known levels; existing menu widget handles this size today |
| zero-one-many | effort-picker-menu | ✅ covered | Zero levels → soft-skip phase; one+ → one row each (existing behavior) |

<!-- Status vocabulary (locked by probe-core projectTruths):
     ✅ covered   → a plain truth string lifted into must_haves.truths
     🧪 backstop  → a flat scalar { statement, verification: backstop }; at verify time, no explicit
                    evidence → insufficient_spec → human_needed (never a silent pass, #1154)
     ⚠ unresolved → an explicit planner assumption (surfaced, never silently dropped)
     Rows are REPLACED (not appended) on a probe re-run — idempotent. -->

---

## Registry Safety

| Registry | Blocks Used | Safety Gate |
|----------|-------------|-------------|
| shadcn official | none | not required |
| third-party | none | not required |

No component registries — pure Rust TUI reusing in-tree widgets.

---

## Checker Sign-Off

- [x] Dimension 1 Copywriting: PASS — exact clamp copy locked; silence-on-no-clamp locked
- [x] Dimension 2 Visuals: PASS — one-line system notice, existing indent, no new widgets
- [x] Dimension 3 Color: PASS — informational style only; destructive explicitly excluded
- [x] Dimension 4 Typography: PASS — terminal-inherited, existing system style reuse
- [x] Dimension 5 Spacing: PASS — single transcript line, no new spacing primitives
- [x] Dimension 6 Registry Safety: PASS — no registries used

**Approval:** approved 2026-07-21 (orchestrator-inline check — TUI-scoped contract; independent adversarial review of the embedding plan follows via Codex convergence)
