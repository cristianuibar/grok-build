# Requirements: bum

**Defined:** 2026-07-16
**Core Value:** One CLI (`bum`) can log into both xAI and Codex and freely switch between Grok and GPT-5.6 models in a real coding session — including cross-provider subagent orchestration.

## v1 Requirements

Requirements for initial release. Each maps to roadmap phases.

### Identity & product surface

- [x] **ID-01**: User can launch the product as the `bum` CLI binary (not `grok` / `xai-grok-pager` as the primary command name)
- [x] **ID-02**: Product UI chrome, help text, and user-facing strings present as **bum**, not stock Grok Build
- [x] **ID-03**: Config, auth, sessions, and related state live under an isolated `~/.bum` home (or `BUM_HOME`), not `~/.grok` or stock Codex paths

### Authentication

- [x] **AUTH-01**: User can log in to xAI via OAuth (browser and/or device-code), equivalent to original Grok Build, with credentials stored only under the bum auth store
- [x] **AUTH-02**: User can log in to ChatGPT/Codex via ChatGPT OAuth (Codex CLI-style PKCE browser flow, plus device-code where applicable), credentials stored only under the bum auth store
- [x] **AUTH-03**: User can log out of one provider without clearing the other provider’s credentials
- [x] **AUTH-04**: User can inspect per-provider auth status (which providers are logged in / usable)
- [x] **AUTH-05**: Access tokens for each provider refresh independently without wiping or invalidating the other provider’s session

### Models & routing

- [x] **MOD-01**: Model selector includes current GPT-5.6 family options usable under ChatGPT/Codex OAuth (e.g. Sol / Terra / Luna or live Codex catalog IDs at implement time), labeled by provider
- [x] **MOD-02**: Model selector includes Grok/xAI models alongside GPT models in one mixed list
- [x] **MOD-03**: User can switch models mid-session anytime; the next turn uses the newly selected model
- [x] **MOD-04**: Requests for Grok/xAI models use the xAI / cli-chat-proxy path with xAI credentials
- [x] **MOD-05**: Requests for GPT/Codex models use the OpenAI/Codex (ChatGPT backend) path with ChatGPT OAuth credentials — not Platform API key semantics and not the xAI proxy
- [x] **MOD-06**: Selecting a model whose provider has no usable credentials blocks the switch and prompts that provider’s login (fail closed; no silent mid-turn 401 as the primary UX)

### Multi-agent / subagent orchestration

- [x] **AGENT-01**: Existing Grok Build multi-agent / subagent spawn, resume, roles, personas, and parent↔child routing continue to work in bum after the fork/rebrand (no regression of in-tree subagent behavior when parent and child share a provider)
- [x] **AGENT-02**: User (or parent agent via tool) can spawn a subagent with an explicit **model** that may belong to a **different provider** than the parent session (e.g. parent on Grok 4.5 / grok-build; child on `gpt-5.6-sol`)
- [x] **AGENT-03**: User (or parent agent) can specify **reasoning effort** (or equivalent Codex effort control) when launching a subagent (e.g. “medium effort”), and the child runs with that effort
- [x] **AGENT-04**: Cross-provider subagent turns use the **child’s** model → provider → credentials → backend routing (not the parent’s bearer or base URL)
- [x] **AGENT-05**: Spawning a cross-provider subagent when the child provider is not logged in fails closed with a clear prompt/error to log in that provider (does not fall back to parent credentials or wrong backend)
- [x] **AGENT-06**: Natural-language orchestration works end-to-end: e.g. with main model Grok, asking to “start a Codex Sol medium-effort subagent to research X” results in a child subagent on the Sol model at medium effort performing that task and returning results to the parent

### Quiet fork & daily driver

- [x] **OPS-01**: Stock xAI auto-update channel is disabled so bum is not overwritten by official Grok Build updates
- [x] **OPS-02**: Product telemetry / phone-home to xAI analytics is disabled by default for the fork
- [x] **OPS-03**: User can complete a real coding session on an xAI model after xAI login (tools, edit, shell as in stock daily use)
- [x] **OPS-04**: User can complete a real coding session on a GPT-5.6 model after Codex login (tools, edit, shell as supported on that path)
- [x] **OPS-05**: User can switch between Grok and GPT-5.6 in one session and continue productive work without restarting the CLI
- [x] **OPS-06**: Parent-on-Grok + child-on-Codex (and parent-on-Codex + child-on-Grok) subagent research/coding tasks complete successfully when both providers are logged in

## v2 Requirements

Deferred to future release. Tracked but not in current roadmap.

### Auth & providers

- **AUTH-V2-01**: Optional API-key fallback per provider for CI/headless (secondary to OAuth)
- **AUTH-V2-02**: Import or read stock `~/.grok` / Codex credential stores
- **AUTH-V2-03**: Multiple OAuth accounts per provider with aliases
- **PROV-V2-01**: Additional providers beyond xAI + Codex/OpenAI (Anthropic, Gemini, OpenRouter, etc.)

### Product

- **WF-V2-01**: Custom agentic workflows integrated into the harness
- **OPS-V2-01**: Team install packaging / non-local distribution
- **MOD-V2-01**: Richer per-provider capability matrix UI and reasoning-effort controls as first-class TUI settings beyond subagent spawn args
- **AGENT-V2-01**: Cross-provider multi-account / cost attribution dashboards for subagent fleets

## Out of Scope

Explicitly excluded. Documented to prevent scope creep.

| Feature | Reason |
|---------|--------|
| Custom agentic workflow engine | Later milestone; v1 is multi-provider identity + routing + subagents |
| Sharing stock grok/codex credential homes | Isolated `~/.bum` identity for full rebrand |
| Arbitrary N-provider marketplace in v1 | Only xAI + Codex/OpenAI ship; architecture should not block later |
| Pretending GPT via xAI proxy | Real GPT-5.6 requires Codex/ChatGPT path |
| Global session “provider mode” that filters models | Conflicts with mixed picker + cross-provider subagents |
| Rewrite agent runtime / new framework | Stay on Grok Build fork harness |
| God-file monorepo cleanup as v1 gate | Background debt; not user-facing success criteria |
| Stock xAI signed public install channel for bum | Private/team daily driver first |
| Enterprise IdP beyond existing xAI OIDC hooks | Not a v1 goal |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| ID-01 | Phase 1 | Complete |
| ID-02 | Phase 8 | Complete |
| ID-03 | Phase 1 | Complete |
| AUTH-01 | Phase 2 | Complete |
| AUTH-02 | Phase 5 | Complete |
| AUTH-03 | Phase 5 | Complete |
| AUTH-04 | Phase 5 | Complete |
| AUTH-05 | Phase 5 | Complete |
| MOD-01 | Phase 3 | Complete |
| MOD-02 | Phase 3 | Complete |
| MOD-03 | Phase 6 | Complete |
| MOD-04 | Phase 4 | Complete |
| MOD-05 | Phase 4 | Complete |
| MOD-06 | Phase 6 | Complete |
| AGENT-01 | Phase 7 | Complete |
| AGENT-02 | Phase 7 | Complete |
| AGENT-03 | Phase 7 | Complete |
| AGENT-04 | Phase 7 | Complete |
| AGENT-05 | Phase 7 | Complete |
| AGENT-06 | Phase 7 | Complete |
| OPS-01 | Phase 8 | Complete |
| OPS-02 | Phase 8 | Complete |
| OPS-03 | Phase 9 | Complete |
| OPS-04 | Phase 9 | Complete |
| OPS-05 | Phase 9 | Complete |
| OPS-06 | Phase 9 | Complete |

**Coverage:**

- v1 requirements: 26 total
- Mapped to phases: 26
- Unmapped: 0

---
*Requirements defined: 2026-07-16*
*Last updated: 2026-07-16 after roadmap creation (traceability filled)*
