# bum

## What This Is

**bum** is a full-product fork of Grok Build (this repo): a terminal AI coding agent TUI/CLI that you launch as `bum` instead of `grok`. v1 keeps the Grok Build agent/TUI harness and makes it a multi-provider daily driver — xAI OAuth for Grok models, ChatGPT/Codex OAuth for GPT-5.6 models, with per-model routing and an isolated `~/.bum` identity. Later milestones will layer custom agentic workflows on the same harness.

## Core Value

You can run one CLI (`bum`), log into both xAI and Codex, and freely switch between Grok and GPT-5.6 models in a real coding session without leaving the tool.

## Requirements

### Validated

Capabilities already present in this Grok Build fork (baseline the product builds on):

- ✓ Full-screen TUI coding agent (pager) with action → dispatch → effect loop — existing
- ✓ Interactive, headless, and ACP agent runtimes (`xai-grok-shell` / `xai-grok-pager`) — existing
- ✓ xAI OAuth2 / device-code auth against `auth.x.ai` (and local-dev issuer) — existing
- ✓ Model selector driven by embedded `default_models.json` + runtime model list — existing
- ✓ Tool surface (bash, edit, search, web, MCP, subagents, workspace/VCS) — existing
- ✓ Session, config, hooks, and skills infrastructure under `~/.grok`-style home — existing (paths rebrand to `~/.bum` in Active)

### Active

- [ ] **Product rename to bum** — Ship the primary binary as `bum` (not `grok` / `xai-grok-pager`); full rebrand of CLI name, UI chrome, install/docs strings, and default home to `~/.bum` so the product is clearly bum, not stock Grok Build
- [ ] **Isolated home & credential store** — Config, auth, sessions under `~/.bum` (not shared with `~/.grok` or stock Codex paths); re-login required after switch
- [ ] **Preserve xAI OAuth** — Keep working browser/device-code login to xAI equivalent to original Grok Build; store credentials in the bum auth store
- [ ] **Codex / ChatGPT OAuth login** — Add ChatGPT OAuth (Codex CLI-style) as a first-class provider alongside xAI; support login UX in TUI and CLI
- [ ] **Multi-provider session** — Model picker lists both xAI (Grok) and OpenAI/Codex (GPT-5.6) models; switching models anytime routes requests to the correct backend with that provider’s credentials
- [ ] **GPT-5.6 models in selector** — Ship current GPT-5.6 family options usable for coding once Codex OAuth is present (exact IDs confirmed against Codex/OpenAI availability during research/implementation)
- [ ] **Provider-aware request routing** — Grok models → xAI / cli-chat-proxy path with xAI tokens; GPT models → OpenAI/Codex API with ChatGPT OAuth tokens
- [ ] **Missing-provider gate** — Selecting a model for a provider without valid credentials blocks and prompts that provider’s OAuth login (no silent failure mid-turn)
- [ ] **Quiet local fork** — Disable xAI auto-update channel and product telemetry so bum does not phone home to x.ai as a stock client
- [ ] **Daily-driver bar** — After v1, bum is usable as the default coding agent for real work: auth, model switch, tools, and sessions work for both providers

### Out of Scope

- Custom agentic workflows / workflow engine — deferred to a later milestone; v1 is multi-provider identity + routing + rebrand only
- Sharing or importing stock `~/.grok` / Codex credential stores — v1 is isolated `~/.bum` only
- Official public distribution / signed x.ai install channel for bum — local/team daily driver first
- Replacing the agent runtime with a new framework — stay on this Grok Build fork’s runtime
- Supporting arbitrary third-party providers beyond xAI + Codex/OpenAI in v1 — multi-provider architecture should not block more later, but only these two ship in v1
- Enterprise IdP SSO customization beyond what the fork already supports for xAI — not a v1 goal
- Full monorepo cleanup of god-files / workspace-types dual system — tracked as background debt, not v1 success criteria

## Context

- **Codebase:** Brownfield Grok Build (SpaceXAI terminal agent). Composition root `xai-grok-pager-bin` → binary historically `xai-grok-pager` / shipped as `grok`. Mapped under `.planning/codebase/` (ARCHITECTURE, STACK, CONCERNS, etc., 2026-07-16).
- **Auth today:** xAI OAuth2 device/browser flow (`auth.x.ai`), device-code grant, credential store under Grok home; API key fallback exists. No Codex/ChatGPT OAuth in-tree yet.
- **Models today:** Embedded `default_models.json` is Grok-centric (e.g. `grok-build`); sampling goes through cli-chat-proxy / xAI paths.
- **Why this exists:** Want a single harness (`bum`) that is *our* product surface — not stock `grok` or `codex` — able to use both ecosystems’ models under OAuth, then later host custom agentic workflows without bolting them onto two separate CLIs.
- **Audience:** Cristian / Buff Up Media internal daily driver (CLI name and project name **bum**).
- **Later vision:** Custom agentic workflows integrated into the same harness (explicitly not v1).

## Constraints

- **Tech stack**: Stay on this Rust workspace (edition 2024, Tokio, existing TUI/agent crates) — fork evolution, not a rewrite
- **Identity**: ChatGPT OAuth for Codex (not API-key-only as the primary path); xAI OAuth preserved for Grok
- **Routing**: Per-model provider selection — mixed picker, not a global “mode” that filters the whole session
- **Storage**: `~/.bum` isolation; no credential sharing with stock CLIs in v1
- **Privacy**: No xAI auto-update; no product telemetry phone-home
- **Compatibility**: Tool/session behavior should remain usable as a daily driver for both providers (provider gaps documented if OpenAI/Codex cannot support a Grok-only tool feature)
- **Naming**: Product and CLI are `bum`; avoid half-rename that still presents as Grok Build to the user

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Product/CLI name `bum` with full rebrand | Clear fork identity; launch like `grok`/`codex`/`claude` | — Pending |
| Isolated `~/.bum` home + auth store | Full rebrand; no accidental coupling to stock grok/codex logins | — Pending |
| Codex auth = ChatGPT OAuth (Codex CLI style) | Matches “real” Codex login and subscription-backed models | — Pending |
| GPT traffic → OpenAI/Codex API with Codex credentials | Real GPT-5.6, not pretending via xAI proxy | — Pending |
| Grok traffic → existing xAI OAuth + proxy path | Preserve working Grok Build path | — Pending |
| Mixed model picker; switch anytime | One session, best model per task | — Pending |
| Missing provider → block + prompt login | Fail closed and fixable, not mid-request surprise | — Pending |
| Disable xAI auto-update + telemetry | Private daily-driver fork must not phone home as stock client | — Pending |
| Custom agentic workflows deferred | Keep v1 shippable: identity + models + rebrand | — Pending |
| v1 success = feature-complete daily driver | Not a prototype — usable as default coding CLI | — Pending |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd:transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd:complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state
5. Promote next milestone (e.g. custom agentic workflows) into Active when ready

---
*Last updated: 2026-07-16 after initialization*
