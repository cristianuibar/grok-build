# Phase 6: Mid-session switch & missing-provider gate - Context

**Gathered:** 2026-07-17
**Status:** Ready for planning
**Mode:** Smart discuss (recommended answers locked — all areas accepted)

<domain>
## Phase Boundary

Deliver **mid-session model switching** and a **fail-closed missing-provider gate** so users freely move between Grok and GPT-5.6 models in one continuous session, and selecting a model whose provider has no usable credentials blocks the switch with a clear login prompt (not a silent mid-turn 401 as primary UX).

In scope (MOD-03, MOD-06):
- Switch models anytime mid-session; next turn uses the newly selected model without restarting the CLI
- Block switch when target provider lacks usable credentials; prompt that provider’s login
- With both providers logged in, free movement between Grok/xAI and GPT/Codex models in one session
- TUI picker + `/model` + ACP `session/set_model` all honor the same gate policy
- Light picker auth cues and post-login auto-retry of the pending switch

Out of scope (later phases / already done):
- Provider-aware routing seam (base URL + credential slot) → **Phase 4 done** (next-sample rebuild)
- Codex OAuth login/logout/status/refresh lifecycle → **Phase 5 done**
- Cross-provider subagent spawn credentials → **Phase 7**
- Full rebrand chrome / help strings → **Phase 8**
- Import or share of stock `~/.codex` / `~/.grok` credentials → **never v1**

</domain>

<decisions>
## Implementation Decisions

### Gate timing (when missing credentials block)
- Run the missing-provider check **at switch time** — block `set_model` / picker confirm / `/model` before applying the new model (matches MOD-06: “Selecting a model… blocks the switch”)
- **Usable credentials** = existing provider-slot `has_usable_token` / equivalent APIs: token present and not permanently unusable; **expired-but-refreshable is OK** (Phase 5 independent refresh owns refresh on next sample)
- Mid-session token death after a successful switch: **switch-gate remains primary UX**; mid-turn 401 / re-login recovery stays **secondary**, not the MOD-06 design center
- **Same-provider** model switches (e.g. Sol→Terra with Codex logged in) have **no extra friction** — gate only when the **target model’s provider** lacks usable creds

### Login recovery UX after a blocked switch
- User-facing block: **clear TUI modal/toast** naming the missing provider (xAI / Codex) and instructing login — never a silent refuse or raw 401 as primary UX
- Offer **“Login now”** that reuses Phase 5 provider login (browser/device) when a minimal in-TUI path exists; **always** show the CLI fallback: `bum login --provider {xai|codex}`
- After successful login from the gate: **auto-retry the pending switch** to the model they selected (reuse/extend existing `deferred_model_switch` pattern)
- Headless / ACP clients: **typed ACP error** with provider id + login hint (machine-readable); no silent 401 as primary

### Picker presentation (auth-aware list)
- **Keep the full mixed catalog** visible even when a provider is not logged in (Phase 3 locked — no auth-based filtering of GPT rows)
- Add a **light badge/hint** on rows whose provider lacks usable credentials (e.g. “needs login”) — rows stay selectable so the gate can run
- **No optimistic current-model update** until the gate passes; on fail stay on the previous model
- **`/model` slash command** and any other switch entry points use the **same gate policy** as the picker

### Mid-turn switch & session continuity
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

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `crates/codegen/xai-grok-shell/src/agent/handlers/model_switch.rs` — `model_switch::apply` (ungated auth path; agent-type compatibility already enforced)
- Pager: `Action::SwitchModel`, `Effect::SwitchModel`, `TaskResult::SwitchModelComplete`, typed `SwitchModelError` (`IncompatibleAgent` | `Other`)
- Pager session state: `model_switch_pending`, `deferred_model_switch` for post-reconnect / deferred apply
- Phase 4 routing: pure resolver model → provider → base_url + credential slot; next sample rebuilds SamplingConfig
- Phase 5 dual auth: `providers.xai` / `providers.codex`, `has_usable_token`, selective login/logout/status, independent refresh
- Catalog: explicit `provider: "xai" | "codex"` on model entries (Phase 3)

### Established Patterns
- Action → dispatch → Effect → TaskResult TEA loop in pager (dispatch must stay pure)
- ACP `session/set_model` via `MvpAgent` → `model_switch::apply`
- Multi-client fan-out of `ModelChanged` after successful switch
- Fail-closed CLI auth patterns from Phase 5 (no silent dual wipe)

### Integration Points
- Gate should run **before** optimistic UI commit and before shell applies model (or shell returns typed error and pager rolls back)
- Login recovery should call into Phase 5 `bum login --provider` / existing OAuth entry points
- Badge data can hang off catalog/provider usability query used by status
- Automated tests: missing Codex blocks switch to GPT; with both logged in, switch Grok↔GPT changes next sample route (reuse Phase 4 fake-token style for positive path)

</code_context>

<specifics>
## Specific Ideas

- MOD-06 wording is switch-time block + provider-specific login prompt — not “try the request and surface 401”
- Reuse `deferred_model_switch` so post-login does not force the user to re-pick the model
- Keep Phase 3 mixed list always complete; gate is the auth boundary, not the catalog
- Cross-provider subagent spawn (parent Grok → child Sol) remains Phase 7 — this phase is **session model** switch only

</specifics>

<deferred>
## Deferred Ideas

- Cross-provider multi-agent / subagent orchestration → Phase 7
- Full product rebrand of chrome/help strings → Phase 8
- Richer per-provider capability matrix UI / first-class effort controls beyond switch → REQUIREMENTS MOD-V2-01
- Stock credential import from `~/.codex` / `~/.grok` → never v1

</deferred>
