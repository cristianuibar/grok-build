# Feature Research

**Domain:** Multi-provider OAuth coding-agent CLI (Grok Build fork → **bum**)
**Researched:** 2026-07-16
**Confidence:** HIGH

## Feature Landscape

Survey scope: Claude Code, Codex CLI, OpenCode, Continue, Aider (OpenRouter multi-model), and this brownfield Grok Build harness. Mapped strictly against **bum v1** goals: dual OAuth (xAI + ChatGPT/Codex), GPT-5.6 in picker, per-model routing, full rebrand to `bum` / `~/.bum`, quiet fork — **not** arbitrary multi-cloud or custom workflows.

### Table Stakes (Users Expect These)

Features users assume exist for a daily-driver multi-provider coding CLI. Missing = product feels incomplete or untrustworthy.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| **Product CLI as `bum`** | Users launch `claude` / `codex` / `grok` by short name; half-renamed binary still feels like stock Grok | MEDIUM | Binary, install scripts, help, UI chrome, env defaults; avoid residual `grok` presentation |
| **Isolated home `~/.bum`** | Every mature CLI owns its config root (`~/.claude`, Codex home, `~/.local/share/opencode`, `~/.grok`) | MEDIUM | Config, auth, sessions, MCP creds; re-login after switch; no share with stock CLIs in v1 |
| **xAI OAuth login (preserved)** | Grok models require working subscription/device auth already in-tree | LOW–MEDIUM | Keep browser + device-code paths; store under bum auth, not `~/.grok` |
| **ChatGPT / Codex OAuth login** | Codex CLI primary path is “Sign in with ChatGPT”; OpenCode exposes ChatGPT Plus/Pro OAuth similarly | HIGH | First-class provider: PKCE/browser and headless/device-code fallback; TUI + CLI entrypoints |
| **API-key fallback (secondary)** | Claude Code, Codex, OpenCode all offer key as alternate; CI/headless needs it | MEDIUM | Not primary identity story; env + stored key per provider; must not replace OAuth UX |
| **Login / logout / status per provider** | Multi-provider users need `auth status`, selective logout, re-auth without wiping the other provider | MEDIUM | Pattern: Claude `auth login|logout|status`; OpenCode `/connect` + `auth.json`; bum needs **per-provider** commands/UI |
| **Mixed model picker (both ecosystems)** | Daily-driver value is one list: Grok + GPT-5.6 family; switch anytime | MEDIUM | Extend `default_models.json` + runtime list; show provider label; not a global “mode” that filters the whole session |
| **GPT-5.6 family in selector** | Codex CLI surface markets current GPT-5.x coding models (e.g. gpt-5.6-*) once ChatGPT-signed-in | MEDIUM | Exact IDs must match Codex/OpenAI availability at implement time; gate on Codex auth |
| **Provider-aware request routing** | Selecting GPT must hit OpenAI/Codex with Codex tokens; Grok must hit xAI/cli-chat-proxy with xAI tokens | HIGH | Core architectural table stake for multi-provider; wrong route = silent wrong bill/backend |
| **Missing-provider gate** | Selecting a model without credentials must **block + prompt login**, not 401 mid-stream | MEDIUM | OpenCode/Claude fail closed on missing auth; multi-provider needs **which** provider to login |
| **Token refresh without session death** | OAuth tokens expire; coding sessions last hours | MEDIUM | Independent refresh per provider; avoid single AuthManager wipe races (see codebase CONCERNS) |
| **Full-screen TUI coding loop** | Expected of Claude Code / Codex / Grok Build class tools | LOW (exists) | Baseline: pager + agent turn loop already present |
| **Tool surface for real work** | bash, edit, search, web, MCP, permissions | LOW (exists) | Daily-driver bar = these work with **both** backends; document provider gaps if any |
| **Sessions resume / history** | Codex `resume`, Claude sessions, Grok shell storage | LOW–MEDIUM | Paths under `~/.bum`; continuity across model switches |
| **Clear auth errors (401 vs rate vs policy)** | Multi-provider debugging; 403≠auth wipe | MEDIUM | Map errors to “re-login xAI” vs “re-login Codex” vs “retry later”; protect auth store |
| **Quiet local fork (no stock phone-home)** | Private daily driver must not auto-update from x.ai or send product telemetry as stock client | MEDIUM | Disable auto-update channel + product Mixpanel/telemetry defaults; optional local logs OK |
| **Permissions / sandbox posture retained** | Table stakes for any agent that runs shell | LOW (exists) | Do not regress permission stack while adding providers |

### Differentiators (Competitive Advantage)

Features that set **bum** apart from “just use codex + grok” or pure multi-key routers (OpenRouter/Aider). Align with Core Value: *one CLI, both OAuth ecosystems, switch models freely*.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| **True dual-subscription OAuth in one harness** | Most official CLIs are single-vendor; OpenCode multi-provider is often API-key heavy; few ship first-class **xAI OAuth + ChatGPT OAuth** together | HIGH | Primary differentiator vs running two CLIs; subscription-backed GPT + Grok without OpenRouter middleman |
| **Per-model (not session-mode) provider binding** | Switch Grok → GPT mid-task without “enter Codex mode”; best model per turn | MEDIUM | PROJECT.md decision: mixed picker, not global filter |
| **Provider login gate at selection time** | Fail closed with actionable “login to Codex” before the turn starts — better than mid-tool-call surprise | MEDIUM | Table stake for multi-provider UX; still rare as polished product behavior |
| **Mature Grok Build agent stack under custom brand** | Inherit TUI, tools, MCP, subagents, hooks, skills — not a thin chat wrapper | LOW–MEDIUM | Differentiation is *product surface + dual auth*, not rewriting agent runtime |
| **Isolated identity from stock `grok`/`codex`** | Team can treat bum as internal product; no credential coupling to vendor homes | MEDIUM | Explicit non-goal: import stock stores in v1 |
| **Honest dual-path daily driver** | One place for Buff Up Media work: Grok when best, GPT-5.6 when best | MEDIUM | Success metric is “default coding CLI,” not feature checklist theater |

**Not differentiators for v1 (defer):** custom agentic workflows, 75+ providers, cost dashboards, cloud remote agents, multi-account aliases, marketplace ecosystems beyond what the fork already has.

### Anti-Features (Commonly Requested, Often Problematic)

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| **Arbitrary third-party providers (Anthropic, Gemini, OpenRouter, 75+)** | OpenCode/Continue/Aider set expectation of “any model” | Dilutes v1; multiplies auth/API surface; blocks ship of dual OAuth | Architecture that *could* add providers later; ship only xAI + Codex/OpenAI in v1 |
| **Import / share `~/.grok` or Codex credential stores** | Avoid re-login | Coupling, format drift, security surprise, product identity leak | Isolated `~/.bum` only; re-login both providers once |
| **Custom agentic workflows / workflow engine** | Later vision; “make it ours” | Scope explosion; v1 needs daily-driver multi-provider first | Explicit later milestone on same harness |
| **Global provider “mode” that locks session** | Simpler mental model | Fights “switch anytime”; forces restarts; weaker than per-model routing | Per-model provider binding + mixed picker |
| **OpenRouter (or similar) as primary identity** | One key, many models | Not real Codex/xAI subscription OAuth; different billing/ToS; not PROJECT.md goal | Direct OAuth to xAI + ChatGPT; optional key only as fallback |
| **Multiple OAuth accounts per provider with aliases** | Work/personal | Complex store, picker UX, token refresh matrix | Single account per provider in v1 |
| **Enterprise IdP SSO customization beyond fork’s xAI path** | Large orgs | Out of scope; high support cost | Keep existing xAI OIDC hooks; no new enterprise surface |
| **Credential-store merge / dual-home read** | Convenience after rename | Race conditions, half-migrated state | Clean cutover to `~/.bum` |
| **Stock xAI auto-update / product telemetry as stock client** | “Stay current” / “usage metrics” | Phones home as Grok Build; wrong for private fork | Disable; document manual update path |
| **Rewrite agent runtime / replace with new framework** | Clean multi-provider design | Rewrites kill brownfield leverage | Evolve Grok Build routing/auth only |
| **God-file monorepo cleanup as v1 success criteria** | Code health | Background debt; not user-facing daily driver | Track separately; don’t block multi-provider ship |
| **Public signed distribution as x.ai install channel** | Easy install | Not authorized; not v1 audience | Local/team install; internal packaging |
| **Cloud remote agent / multi-device session sync** | Codex cloud parity | Huge product surface | Local-first v1 |
| **Full cost/usage analytics dashboard** | Multi-provider spend anxiety | Needs per-provider metering APIs; polish sink | Surface provider errors + existing status; defer dashboards |
| **Pretend GPT via xAI proxy** | Avoid second OAuth | Breaks “real GPT-5.6” requirement | Route GPT → OpenAI/Codex with Codex credentials |

## Feature Dependencies

```
Product rename (`bum` binary + chrome)
    └──requires──> Isolated home (`~/.bum` paths)

Isolated home
    └──requires──> Credential store relocation (auth, MCP, sessions)

xAI OAuth (preserve)
    └──requires──> Isolated home + rename of auth paths/env

Codex / ChatGPT OAuth
    └──requires──> Provider abstraction in auth store
    └──requires──> Login UX (CLI + TUI) + device-code fallback
    └──enhances──> GPT-5.6 models in selector

GPT-5.6 in selector
    └──requires──> Model catalog entries with provider metadata
    └──requires──> Codex OAuth (or key) usable for those models

Provider-aware routing
    └──requires──> Model → provider mapping
    └──requires──> Per-provider credential resolution at sample time
    └──requires──> Separate HTTP clients / base URLs (xAI proxy vs OpenAI/Codex)

Missing-provider gate
    └──requires──> Provider-aware routing (know which provider is needed)
    └──requires──> Login UX invokable mid-session
    └──enhances──> Mixed model picker (safe selection)

Mixed model picker / switch anytime
    └──requires──> Model catalog with both families
    └──requires──> Provider-aware routing
    └──enhances──> Daily-driver bar

Token refresh per provider
    └──requires──> Dual credential store
    └──enhances──> Long sessions after dual login

Quiet fork (no auto-update / telemetry)
    └──requires──> Config/feature flags on update + telemetry crates
    └──conflicts──> Stock “always auto-update from x.ai” behavior

Daily-driver bar (tools + sessions both providers)
    └──requires──> Dual auth + routing + gate + rename
    └──requires──> Baseline tool/session stack (exists)
    └──conflicts──> Scope expansion into workflows / N providers

API-key fallback
    └──enhances──> Headless/CI; does not replace OAuth table stakes

Custom agentic workflows
    └──conflicts──> v1 scope (defer)

Import stock credential homes
    └──conflicts──> Isolated identity decision
```

### Dependency Notes

- **Rename requires isolated home:** User-visible `bum` without `~/.bum` still collides with `~/.grok` and confuses identity.
- **GPT-5.6 requires Codex auth path:** Listing models without a working OpenAI/Codex credential path is a trap; gate availability or login on select.
- **Routing requires model→provider metadata:** Catalog must carry provider id (and backend hints), not only display names.
- **Missing-provider gate requires invokable login:** Gate is useless if OAuth only works at cold start.
- **Daily-driver requires both paths working end-to-end:** Auth-only without tools/sessions on GPT path fails the success bar.
- **Workflows conflict with v1:** Explicit PROJECT.md out-of-scope; do not pull into multi-provider phases.

## MVP Definition

### Launch With (v1)

Minimum to validate Core Value: *one CLI, log into both, switch Grok ↔ GPT-5.6 in a real coding session*.

- [ ] **Product rename to `bum`** — binary, chrome, docs strings; not stock Grok presentation
- [ ] **`~/.bum` isolated home** — config, auth, sessions isolated from `~/.grok` / Codex
- [ ] **xAI OAuth preserved** under bum store (browser + device-code)
- [ ] **Codex/ChatGPT OAuth** first-class (browser + headless-friendly path)
- [ ] **Per-provider login / logout / status**
- [ ] **Mixed model picker** with Grok + GPT-5.6 family; switch anytime
- [ ] **Provider-aware routing** (Grok→xAI path; GPT→OpenAI/Codex path)
- [ ] **Missing-provider gate** → block + prompt correct OAuth
- [ ] **Quiet fork** — disable xAI auto-update + product telemetry phone-home
- [ ] **Daily-driver bar** — tools, permissions, sessions usable with both providers for real work
- [ ] **API-key fallback (secondary)** for headless where OAuth is painful

### Add After Validation (v1.x)

- [ ] **Richer per-provider status UI** — plan/tier hints, token expiry countdown, last-error by provider (trigger: dual-login confusion in daily use)
- [ ] **Model catalog freshness** — document/update GPT-5.6 IDs as OpenAI renames them; optional remote catalog later
- [ ] **Reasoning-effort / provider-specific sampling knobs in picker** — if GPT path needs Codex-style effort controls
- [ ] **Provider capability matrix** — which tools/features degrade on OpenAI vs xAI (trigger: first real gap found)
- [ ] **Hardened multi-process auth locking** — if concurrent `bum` instances race dual stores
- [ ] **Selective logout without full wipe** — polish if status UX shows need

### Future Consideration (v2+)

- [ ] **Custom agentic workflows** on the same harness (project vision; out of v1)
- [ ] **Additional providers** (Anthropic, etc.) using the multi-provider abstraction
- [ ] **Optional credential import wizards** from stock CLIs (only if isolation pain is high)
- [ ] **Multi-account aliases per provider**
- [ ] **Usage/cost aggregation across providers**
- [ ] **Public distribution packaging** (if product graduates beyond internal daily driver)
- [ ] **Cloud/remote agent surfaces**

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| Isolated `~/.bum` + path cutover | HIGH | MEDIUM | P1 |
| Product rename (`bum` binary/chrome) | HIGH | MEDIUM | P1 |
| Preserve xAI OAuth under bum | HIGH | LOW–MEDIUM | P1 |
| Codex/ChatGPT OAuth | HIGH | HIGH | P1 |
| Provider-aware routing | HIGH | HIGH | P1 |
| Mixed model picker + GPT-5.6 entries | HIGH | MEDIUM | P1 |
| Missing-provider gate | HIGH | MEDIUM | P1 |
| Per-provider login/logout/status | HIGH | MEDIUM | P1 |
| Quiet fork (no update/telemetry) | HIGH | MEDIUM | P1 |
| Daily-driver tools/sessions both providers | HIGH | MEDIUM | P1 |
| Token refresh dual-store | HIGH | MEDIUM | P1 |
| API-key fallback secondary | MEDIUM | MEDIUM | P2 |
| Provider-labeled errors / status polish | MEDIUM | LOW–MEDIUM | P2 |
| Reasoning-effort / GPT sampling knobs | MEDIUM | MEDIUM | P2 |
| Capability matrix docs | MEDIUM | LOW | P2 |
| Extra providers (Anthropic, etc.) | MEDIUM | HIGH | P3 |
| Custom agentic workflows | HIGH (later) | HIGH | P3 |
| Credential import from stock homes | LOW–MEDIUM | MEDIUM | P3 |
| Multi-account aliases | LOW | HIGH | P3 |
| Cost dashboards / cloud remote | LOW for v1 | HIGH | P3 |

**Priority key:**
- **P1:** Must have for v1 launch / daily-driver success
- **P2:** Should have once dual path works; polish and resilience
- **P3:** Nice / later milestones — do not pull into v1 phases

## Competitor Feature Analysis

| Feature | Claude Code | Codex CLI | OpenCode | Aider / Continue | **bum (v1 approach)** |
|---------|-------------|-----------|----------|------------------|------------------------|
| Primary auth | Anthropic OAuth / console / cloud IdPs | ChatGPT OAuth (+ API key) | `/connect`: OAuth (ChatGPT, Copilot, xAI SuperGrok, …) + keys; 75+ providers | Mostly API keys; Aider OpenRouter OAuth onboarding | **Dual first-class OAuth: xAI + ChatGPT/Codex only** |
| Model switch mid-session | Yes (`/model`, shortcuts) | Yes (`/model`, effort) | Yes (`/models`) | Yes (flags/config; Continue roles) | **Yes — mixed picker, anytime** |
| Multi-provider in one session | Effectively single-vendor | Single-vendor OpenAI | Core pitch | Yes via keys/router | **Two providers; subscription OAuth, not OpenRouter-first** |
| Missing auth UX | Re-login prompts | Sign-in on first run / failure | Connect before use | Config/env errors | **Gate at model select with provider-specific login** |
| Config home | Claude-specific | Codex-specific | `~/.local/share/opencode` etc. | `~/.aider*`, `~/.continue` | **`~/.bum` isolated** |
| Tooling depth | Strong (bash, MCP, hooks, subagents) | Strong (skills, plugins, MCP, sandbox) | Strong multi-provider agent | Edit-centric (Aider) / IDE (Continue) | **Keep Grok Build stack** |
| Auto-update / telemetry | Product-managed | Product-managed | OSS self-update norms | N/A / light | **Disable stock xAI update + product telemetry** |
| Workflows / multi-agent product | Agent teams evolving | Skills, cloud, subagents | Multi-agent configs | Limited | **Defer custom workflows to later milestone** |

## Sources

- Project requirements: `.planning/PROJECT.md` (v1 scope, decisions, out-of-scope)
- Codebase: `.planning/codebase/INTEGRATIONS.md`, `CONCERNS.md`; `crates/codegen/xai-grok-models/default_models.json` (Grok-centric catalog today)
- Codex CLI product surface: [Codex CLI docs](https://learn.chatgpt.com/docs/codex/cli) — ChatGPT sign-in, `/model`, permissions, MCP, resume
- OpenCode multi-provider: [opencode.ai](https://opencode.ai/), [Providers docs](https://opencode.ai/docs/providers/) — `/connect`, `auth.json`, ChatGPT OAuth, GitHub Copilot device flow, xAI SuperGrok OAuth + device-code, model picker
- Claude Code: CLI auth login/logout/status, model switch, MCP — community/official CLI references
- Continue: multi-provider YAML models/roles — [Continue model providers](https://docs.continue.dev/customize/model-providers/overview)
- Aider / OpenRouter: multi-model via router + keys — [Aider OpenRouter](https://aider.chat/docs/llms/openrouter.html)
- Community patterns: dual-OAuth and per-provider credential stores in multi-agent tooling discussions (e.g. OpenCode ecosystem plugins, Codex OAuth compatibility projects)

**Confidence notes:** Competitor UX patterns (OAuth, `/model`, isolated homes, multi-provider connect) are well-documented (HIGH). Exact GPT-5.6 model ID strings and Codex backend endpoint details for this fork need confirmation during implementation (MEDIUM until wired against live Codex/OpenAI). Internal Buff Up “daily-driver bar” is product judgment, not market survey (MEDIUM).

---
*Feature research for: multi-provider OAuth coding-agent CLI (bum)*
*Researched: 2026-07-16*
