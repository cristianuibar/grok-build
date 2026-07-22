# bum

**Expansion:** **BUM** = **Build Using Multiagents**

## What This Is

**bum** (*Build Using Multiagents*) is a full-product fork of Grok Build (this repo): a terminal AI coding agent TUI/CLI launched as `bum`. **v1.0 shipped** as a multi-provider daily driver — xAI OAuth for Grok models, ChatGPT/Codex OAuth for GPT-5.6 models, per-model routing, cross-provider subagents, quiet fork (no phone-home), and isolated `~/.bum` identity. Later milestones layer custom agentic workflows on the same harness.

## Core Value

You can run one CLI (`bum`), log into both xAI and Codex, and freely switch between Grok and GPT-5.6 models in a real coding session without leaving the tool.

## Current Milestone: v1.1 Upstream Grok Build parity

**Goal:** Bring `bum` up to parity with the latest official `xai-org/grok-build` revision through a controlled, fully researched integration while preserving bum's product contracts.

**Target features:**
- Pin a reproducible upstream baseline and inventory all divergence despite the repositories currently having no shared Git ancestry.
- Integrate every applicable upstream feature, fix, test, dependency, documentation, and architecture change.
- Preserve `bum`, `~/.bum`, dual xAI + Codex OAuth, mixed per-model routing, cross-provider subagents, and quiet/no-phone-home behavior when adapting overlaps.
- Record every intentionally excluded, superseded, or modified upstream change with rationale and source-revision traceability.
- Establish repeatable future upstream synchronization and validate CLI/TUI, dual-provider, privacy, and regression contracts.

## Current State

**Shipped:** v1.0 Multi-provider daily driver (2026-07-22)  
**Archive:** `.planning/milestones/v1.0-ROADMAP.md`, `v1.0-REQUIREMENTS.md`, `v1.0-MILESTONE-AUDIT.md`, `v1.0-phases/`  
**Live identity:** `bum version` prints bum product label (ID-02 closed in Phase 12.1).

## Requirements

### Validated

Baseline fork + v1.0 multi-provider daily driver:

- ✓ Full-screen TUI coding agent (pager) with action → dispatch → effect loop — existing
- ✓ Interactive, headless, and ACP agent runtimes — existing
- ✓ Tool surface (bash, edit, search, web, MCP, subagents, workspace/VCS) — existing
- ✓ Isolated `~/.bum` product home + `bum` binary (ID-01, ID-03) — v1.0
- ✓ Multi-slot auth store + xAI OAuth under bum (AUTH-01) — v1.0
- ✓ Codex / ChatGPT OAuth dual lifecycle (AUTH-02..05) — v1.0
- ✓ Mixed Grok + GPT-5.6 catalog with explicit provider binding (MOD-01..02) — v1.0
- ✓ Provider-aware routing + mid-session switch + missing-provider gate (MOD-03..06) — v1.0
- ✓ Cross-provider multi-agent orchestration (AGENT-01..06) — v1.0
- ✓ Quiet fork: no stock auto-update / product telemetry phone-home (OPS-01..02) — v1.0
- ✓ Daily-driver live bar OPS-03..06 + Codex wire/effort/attribution polish — v1.0
- ✓ User-facing product chrome including `bum version` (ID-02) — v1.0 / Phase 12.1

### Active

- [ ] **Upstream Grok Build parity** — research and integrate all applicable official changes through a pinned upstream revision while preserving bum-specific contracts (v1.1)
- [ ] **Repeatable upstream synchronization** — retain source-revision traceability, conflict policy, exclusions, and verification gates for future updates (v1.1)
- [ ] **Custom agentic workflows** — workflow engine / multi-agent pipelines on the same harness (deferred until after the parity milestone)

### Out of Scope

- Sharing or importing stock `~/.grok` / Codex credential stores — isolated `~/.bum` only (v1 constraint retained)
- Official public distribution / signed x.ai install channel for bum — local/team daily driver first
- Replacing the agent runtime with a new framework — stay on this Grok Build fork’s runtime
- Supporting arbitrary third-party providers beyond xAI + Codex/OpenAI until a later milestone decides
- Enterprise IdP SSO customization beyond what the fork already supports for xAI
- Full monorepo cleanup of god-files / workspace-types dual system — background debt
- Complete internal crate rename away from `xai-grok-*` — residual rebrand tech debt, not user-facing

## Context

- **Codebase:** Brownfield Grok Build fork. Composition root `xai-grok-pager-bin` ships binary **`bum`**. Mapped under `.planning/codebase/`.
- **Auth:** Dual multi-slot store under `~/.bum` — xAI OAuth + ChatGPT/Codex OAuth (PKCE/device), selective logout, independent refresh.
- **Models:** Mixed catalog — Grok (xAI) + GPT-5.6 Sol/Terra/Luna (Codex) with provider-aware sampling and mid-session switch.
- **Why this exists:** Single harness that is *our* product surface — not stock `grok` or `codex` — both ecosystems under OAuth, then custom agentic workflows without two CLIs.
- **Audience:** Cristian / Buff Up Media internal daily driver.
- **Current vision:** Restore parity with the public official Grok Build history first, without regressing bum's identity, providers, orchestration, or privacy contracts; custom agentic workflows follow afterward.
- **Upstream topology:** GitHub records this repository as a fork of `xai-org/grok-build`, but current `main` and `upstream/main` have no shared Git ancestor. Integration therefore requires content/history archaeology rather than a blind merge.

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
| Product/CLI name `bum` with full rebrand | Clear fork identity; launch like `grok`/`codex`/`claude` | Phase 1 binary/`BUM_HOME`; chrome polish → Phase 8 |
| Isolated `~/.bum` home + auth store | Full rebrand; no accidental coupling to stock grok/codex logins | Phase 1–2 — multi-slot `auth.json` under bum home |
| Codex auth = ChatGPT OAuth (Codex CLI style) | Matches “real” Codex login and subscription-backed models | Phase 5 — PKCE + device, dual lifecycle |
| GPT traffic → OpenAI/Codex API with Codex credentials | Real GPT-5.6, not pretending via xAI proxy | Phase 4 — route resolver + dual-key sampling |
| Grok traffic → existing xAI OAuth + proxy path | Preserve working Grok Build path | Phase 2 auth + Phase 4 routing |
| Mixed model picker; switch anytime | One session, best model per task | Phase 3 catalog; mid-session gate → Phase 6 |
| Missing provider → block + prompt login | Fail closed and fixable, not mid-request surprise | Phase 6 — validated |
| Disable xAI auto-update + telemetry | Private daily-driver fork must not phone home as stock client | Phase 8 — validated |
| Custom agentic workflows deferred | Keep v1 shippable: identity + models + cross-provider subagents + rebrand | — Deferred until after v1.1 parity |
| v1.1 targets full applicable upstream parity | Avoid silently missing upstream fixes/features while retaining explicit exclusions for incompatible stock-product behavior | Approved milestone scope |
| Preserve bum contracts over conflicting upstream defaults | Product identity, isolated storage, dual providers, cross-provider agents, and privacy are non-negotiable | Approved milestone invariant |
| Treat upstream as content lineage without a merge base | Current Git histories have no shared ancestor; blind merge/rebase is unsafe | Research must reconstruct baseline and integration method |
| Cross-provider subagents in v1 | Parent model/provider must not limit child; NL + tool spawn with model + effort | ✓ Phase 7 + Phase 9 live both-direction PASS |
| v1 success = feature-complete daily driver | Not a prototype — usable as default coding CLI | ✓ Phase 9 OPS-03..06 + Phase 12.1 ID-02 close |
| Version path product token local const + pure formatter | Match Phase 8 product-name pattern; unit + hermetic + static gates | ✓ Phase 12.1 |

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
*Last updated: 2026-07-22 for v1.1 Upstream Grok Build parity milestone*
