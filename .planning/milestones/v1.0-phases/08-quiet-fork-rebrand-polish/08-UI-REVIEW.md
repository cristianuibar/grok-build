# Phase 8 — UI Review

**Audited:** 2026-07-17
**Baseline:** `08-UI-SPEC.md` (rebrand/copy locks — not greenfield design)
**Screenshots:** not captured (terminal TUI/CLI product; no web dev server on :3000/:5173/:8080)
**Registry audit:** skipped — `components.json` absent; UI-SPEC shadcn gate N/A

---

## Pillar Scores

| Pillar | Score | Key Finding |
|--------|-------|-------------|
| 1. Copywriting | 4/4 | All UI-SPEC locked product strings match; model brand “Grok Build (xAI)” retained |
| 2. Visuals | 4/4 | Hero geometry + badge modes preserved; Beta marketing chrome removed |
| 3. Color | 4/4 | No palette redesign; product badge uses body/bold, not accent_model |
| 4. Typography | 4/4 | String-only rebrand; existing 2-weight / role map unchanged |
| 5. Spacing | 4/4 | `HERO_BOX_MIN_WIDTH` 90 / `V_PAD` 1 / `H_INSET` 2 / `LOGO_H_PAD` 3 intact |
| 6. Experience Design | 3/4 | Quiet feedback/update paths correct; residual `Effect::SendFeedback` network arm still present |

**Overall: 23/24**

---

## Top 3 Priority Fixes

1. **Residual `Effect::SendFeedback` network path** — A misconfig or future call site that emits the effect still POSTs feedback and can surface `Couldn't send feedback: …` instead of the locked disabled copy — In `effects/mod.rs` short-circuit `Effect::SendFeedback` (no network) and map `TaskResult::FeedbackFailed` to `Feedback is disabled in bum (no phone-home).` so residual paths honor OPS-02 / UI-SPEC residual-error mapping.

2. **Stale free-usage e2e expectation** — CI/e2e still asserts stock product chrome (`reached your free Grok Build usage limit`) while production `FREE_USAGE_USER_MESSAGE` correctly says `free bum usage limit` — Update `test_built_binary_e2e.rs` so tests enforce the UI-SPEC product token and cannot regress half-rename.

3. **Dev playground self-identity still “Grok Build”** — `mouse_events_playground` / `scrollback_selection_playground` intro lines still read as stock product — Low user impact (non-shipped bins) but half-rename hygiene; swap product self-ID to `bum` if playgrounds stay in-tree.

---

## Detailed Findings

### Pillar 1: Copywriting (4/4)

**Locked inventory — verified present**

| UI-SPEC lock | Implementation | Status |
|--------------|----------------|--------|
| Clap `name` / `about` | `cli.rs`: `name = "bum"`, `about = "bum TUI"` | ✅ |
| Banner | `pager-bin`: `bum (pager) - v{version}` | ✅ |
| Version status prefix | `auto_update.rs`: `bum - v{version}` | ✅ |
| `HERO_SUBTITLE` | `Thanks for using bum.` (no `/feedback` advertise) | ✅ |
| `PRODUCT_BADGE_LABEL` | `"bum  "` (trailing spacer; no Beta) | ✅ |
| ZDR blocked | `bum is not yet available for this account.` | ✅ |
| Trust L1 | `bum may run or modify contents in this directory,` | ✅ |
| Project picker | Locked two-paragraph bum question | ✅ |
| OAuth return | `You can close this window and return to bum.` | ✅ |
| Feedback disabled | `Feedback is disabled in bum (no phone-home).` + optional `~/.bum` logs hint | ✅ |
| Update disabled | Primary + optional x.ai-channel secondary lines exact | ✅ |
| Settings auto_update desc | `Stock update channel (permanently disabled in bum; cannot re-enable).` | ✅ (stronger than preferred short form; still locked direction) |
| Subscribe / credits / upsell product | `Subscribe to use bum`, `Purchase credits to keep using bum`, `Get the most out of bum…` | ✅ |
| Free-usage product | `free bum usage limit` + SuperGrok + external `grok.com` host | ✅ |
| pager-minimal product | `MINIMAL_WELCOME_PRODUCT_NAME = "bum"` | ✅ |

**Model brand exclusion (correct non-change):** catalog / dynamic enum still expose `Grok Build (xAI)` for id `grok-build` — matches Product vs model brand table.

**Minor residuals (not score-breaking):**

- **WARNING:** Playground bins still use `"I'm Grok Build, an interactive CLI agent…"` (`xai-grok-pager/src/bin/*_playground.rs`). Outside UI-SPEC executor inventory; not default `bum` CLI chrome.
- **WARNING:** `xai-grok-shell/tests/test_built_binary_e2e.rs:389` still expects `free Grok Build usage limit` — test drift vs production copy.
- Crate `//!` docs and internal comments still say “Grok Build” (out of user-facing chrome scope per CONTEXT / UI-SPEC).

No generic “Submit / Click Here / OK” CTA regressions in phase surfaces. Stock team thank-you (`Thanks for the feedback! The Grok Build team is on it.`) is eliminated from the dispatch path (asserted in `p8_feedback` / notes tests).

### Pillar 2: Visuals (4/4)

- Focal product chrome remains welcome hero + version badge; rebrand shortens product span (`bum` vs `Grok Build`) without rebalancing logo columns (UI-SPEC overflow rule).
- VersionBadge modes preserved: Full / HeroFooter / HeroInline; stock **Beta** marketing span omitted (`// Stock " Beta" marketing chrome omitted`).
- No new icon-only controls, modals, or layout chrome introduced this phase.
- Hero subtitle still secondary/gray role via existing `subtitle_style` path; hidden when info slot filled (`subtitle_rows`).

**No BLOCKER / WARNING on visuals.** Geometry-only inheritance holds.

### Pillar 3: Color (4/4)

- GrokNight / theme tokens **unchanged** (phase contract: no palette redesign).
- Product badge Full/HeroInline: `theme.text_primary` + `Modifier::BOLD` — **not** `accent_model` / `fuzzy_accent` (matches “product name is Body, not accent”).
- Version/channel: `theme.gray` secondary.
- Disabled feedback/update messaging uses system blocks / informational toasts — not `accent_error` alarm red (matches UI-SPEC destructive reservation).
- Free-usage / SuperGrok commercial upsell paths unchanged in accent usage; model names continue prior `accent_model` path (out of rebrand scope).
- Minimal welcome product span uses `theme.accent_user` (pre-existing minimal-card role). Not a new brand color; Full hero correctly uses body text. Acceptable under “no color redesign.”

**No hardcoded hex rebrand palette.** No “bum theme.”

### Pillar 4: Typography (4/4)

- Still terminal monospaced; roles = Theme + Modifier only.
- Product badge keeps stock bold on product span; version/channel regular gray.
- No new display type, no extra weights, no font-size inflation for shorter product name.
- CLI/update lines plain stdout — body role.

**Contract met:** rebrand is string substitution, not type system expansion.

### Pillar 5: Spacing (4/4)

Chrome geometry audit (`hero_box.rs`):

| Token | Spec | Code | Status |
|-------|------|------|--------|
| `HERO_BOX_MIN_WIDTH` | 90 | `90` | ✅ |
| `V_PAD` | 1 | `1` | ✅ |
| `H_INSET` | 2 | `2` | ✅ |
| `LOGO_H_PAD` | 3 | `3` | ✅ |

- Project picker: QuestionView frame only — text swap.
- Update CLI: plain lines, no new modal padding.
- Feedback: existing system-block channel — no new spacing scale.

**No arbitrary rem/px; no hero restyle.**

### Pillar 6: Experience Design (3/4)

**Covered well (UI-SPEC state table):**

| State | Evidence | Status |
|-------|----------|--------|
| Feedback empty collector blocked | `dispatch_enter_feedback_mode` forces `PromptInputMode::Normal`, empty effects | ✅ |
| Feedback disabled copy | System blocks with locked primary + optional logs hint | ✅ |
| No `Effect::SendFeedback` on Action path | `dispatch_send_feedback` returns `vec![]`; notes tests assert | ✅ |
| Update explicit no-op | `stock_auto_update_disabled_lines()` + print path; no stock helper calls | ✅ |
| Startup update silent skip | `should_check_for_updates` hard `false` | ✅ |
| auto_update default off | registry `Bool { default: false }`; refuse `SetAutoUpdate(true)` toast | ✅ |
| Min-version force install | `enforce_minimum_version_or_exit` no-op | ✅ |
| Telemetry default quiet | Not user-visible when Disabled (prior plan proofs) | ✅ |

**Findings:**

- **WARNING:** `Effect::SendFeedback` handler in `app/effects/mod.rs` still builds `ClientFeedbackInput` and can emit network work. Default Action dispatch never emits it, but UI-SPEC residual rule requires misconfig paths map to **disabled** copy, not `Couldn't send feedback: …` (`task_result.rs` FeedbackFailed). Dead-path risk remains until the effect arm is neutralized.

- **WARNING:** Slash `/feedback` with text still routes `Action::SendFeedback` → short-circuit (OK), but `PromptInputMode::Feedback` still exists as a mode that *would* submit feedback text if re-entered via another path. Enter path correctly refuses collect mode; keep mode unreachable or map all entries to disabled.

No destructive product actions in phase scope. Loading update spinner correctly absent on quiet default.

---

## Files Audited

| File | Role in audit |
|------|----------------|
| `.planning/phases/08-quiet-fork-rebrand-polish/08-UI-SPEC.md` | Design contract / locked copy |
| `.planning/phases/08-quiet-fork-rebrand-polish/08-VERIFICATION.md` | Goal-backward proof map |
| `.planning/phases/08-quiet-fork-rebrand-polish/08-03-SUMMARY.md` | Shell/bin residual rebrand summary |
| `crates/codegen/xai-grok-pager/src/app/cli.rs` | Clap name/about |
| `crates/codegen/xai-grok-pager/src/views/welcome/hero_box.rs` | Subtitle + geometry constants |
| `crates/codegen/xai-grok-pager/src/views/welcome/mod.rs` | Badge / ZDR / trust product strings |
| `crates/codegen/xai-grok-pager/src/project_picker/mod.rs` | Project picker question |
| `crates/codegen/xai-grok-pager/src/app/dispatch/notes.rs` | Feedback quiet path |
| `crates/codegen/xai-grok-pager/src/app/dispatch/billing.rs` | Free-usage + credits product copy |
| `crates/codegen/xai-grok-pager/src/app/app_view.rs` | Subscribe product string |
| `crates/codegen/xai-grok-pager/src/settings/defs.rs` | auto_update description |
| `crates/codegen/xai-grok-pager/src/settings/registry.rs` | auto_update default false |
| `crates/codegen/xai-grok-pager/src/app/dispatch/settings/setters.rs` | Refuse enable auto_update |
| `crates/codegen/xai-grok-pager/src/app/effects/mod.rs` | Residual SendFeedback effect |
| `crates/codegen/xai-grok-pager/src/app/dispatch/task_result.rs` | FeedbackFailed copy |
| `crates/codegen/xai-grok-shell/src/auth/oidc/login.rs` | OAuth return HTML |
| `crates/codegen/xai-grok-pager-minimal/src/welcome.rs` | Minimal product name |
| `crates/codegen/xai-grok-pager-bin/src/main.rs` | Banner + update disabled lines |
| `crates/codegen/xai-grok-update/src/auto_update.rs` | `bum - v` status prefix |
| Grep inventory: `Grok Build` across pager/shell/update production surfaces | Half-rename residual scan |

---

## Recommendation Summary

| Severity | Count | Notes |
|----------|-------|-------|
| BLOCKER | 0 | No user-task-breaking half-rename or quiet-path failure on default flows |
| WARNING | 3 | Residual feedback effect arm; stale free-usage e2e assert; playground self-ID |
| Priority fixes | 3 | Listed above |
| Minor polish | 1 | Settings description already exceeds preferred short form (acceptable) |

**Ship readiness (UI contract):** Pass for ID-02 / OPS-01 / OPS-02 chrome and messaging. Address residual `Effect::SendFeedback` before treating OPS-02 as defense-in-depth complete.
