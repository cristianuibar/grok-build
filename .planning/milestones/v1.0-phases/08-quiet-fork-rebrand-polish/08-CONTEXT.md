# Phase 8: Quiet fork & rebrand polish - Context

**Gathered:** 2026-07-17
**Status:** Ready for planning

<domain>
## Phase Boundary

Product presents fully as **bum** and does not phone home or auto-update as stock Grok Build. This phase delivers:

1. **ID-02** — User-facing UI chrome, help text, and product strings present as **bum**, not stock Grok Build
2. **OPS-01** — Stock xAI auto-update channel disabled so bum is not overwritten by official Grok Build updates
3. **OPS-02** — Product telemetry / phone-home to xAI analytics disabled by default for the fork

**In scope:** Product chrome rebrand, clap/help/welcome identity, auto-update hard-off + no-op update path, telemetry/Sentry/Mixpanel defaults off, quiet `/feedback` handling, gate tests for no stock phone-home.

**Out of scope:** Model routing/auth (Phases 2–7 done), public signed x.ai install channel for bum (PROJECT out of scope), internal crate renames (`xai-grok-*` stay), custom agentic workflows, live daily-driver E2E (Phase 9), credential import from stock homes.

**UI hint:** yes — welcome/hero chrome and product-facing strings are part of this phase; not a new design system, a rebrand of existing chrome.

</domain>

<decisions>
## Implementation Decisions

### Product string rebrand (ID-02)
- All product chrome renames to **bum**: welcome/hero, clap about/help, project picker, OAuth browser return text, feedback toasts, subscription/status copy that names the product — not merely a subset of screens
- Keep **model brands** as model brands (`Grok` / GPT catalog names stay); only the *product* shell is `bum`
- Leave internal crate/package names (`xai-grok-*`) unchanged — user-facing only (Phase 1 lock)
- In-tree user-facing help/docs only — no public x.ai signed install channel polish in this phase

### Auto-update kill (OPS-01)
- **Hard-off** stock xAI auto-update: no background check/download from the stock x.ai channel
- Explicit `bum update` / Ctrl+U-style path: **no-op + clear message** that the stock channel is disabled for this fork; point users at local build/install
- Minimum-version / forced update policy from stock: **ignore / disable** — never force-upgrade to stock Grok Build
- Enforce via **code defaults + tests** (`auto_update` default false; startup check skipped; gate tests prove no stock update phone-home)

### Telemetry kill (OPS-02)
- Default product telemetry mode: **Disabled** (not SessionMetrics, not Enabled)
- Mixpanel / product analytics events to xAI: **off** by default
- Sentry / crash phone-home: **off** by default for the quiet fork
- Local tracing / log files under bum home remain fine; no easy stock phone-home opt-in path as the v1 default

### CLI surface, env knobs & residual identity
- Clap CLI name / about: rename from `grok` / “Grok Build TUI” to **`bum`**
- Broader `GROK_*` env family (non-home): **leave internal**; `BUM_HOME` remains the user-facing home knob (Phase 1 already cut home isolation)
- Hero / welcome badge: rebrand to **bum** (drop stock “Grok Build Beta” / xAI marketing chrome)
- Stock `/feedback` path: **rebrand or disable** so product feedback does not phone home to xAI by default (quiet-fork adjacent to OPS-02)

### Claude's Discretion
- Exact inventory of “Grok Build” string sites and batching across crates
- Whether to centralize a `product_name()` / brand constant vs surgical string edits
- How to wire auto-update hard-off in `xai-grok-update` + startup check paths with minimal blast radius
- Telemetry default plumbing (`TelemetryMode::Disabled`, compile-time tokens, config defaults)
- `/feedback` exact UX (local-only note vs command hidden vs clear disabled message)
- Test placement (unit vs integration gate) as long as success criteria are proven

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- Binary already ships as `bum` with `BUM_HOME` / `~/.bum` isolation (Phase 1)
- `xai-grok-update` — auto-update channel, `check_update_status`, background startup check, min-version policy (`cli.auto_update == Some(false)` already respected in places)
- `xai-grok-telemetry` — `TelemetryMode::{Disabled, SessionMetrics, Enabled}`, Mixpanel client, Sentry hooks, external OTEL
- Product chrome strings concentrated in pager welcome/hero (`hero_box.rs`, `views/welcome/mod.rs`), clap (`app/cli.rs`), project picker, shell OAuth return text, feedback toasts

### Established Patterns
- Phase 1: user-facing binary/home rename without internal crate rename
- Config defaults + hermetic tests under temp `BUM_HOME` for isolation proofs
- Privacy constraint already documented in PROJECT.md and AGENTS.md
- Model catalog keeps provider/model brands distinct from product name (Phases 3–4)

### Integration Points
- Startup path: background update check from TUI / pager-bin composition root
- Explicit update command + keyboard fallback (Ctrl+U class paths)
- Telemetry init at process start (shell + pager-bin)
- User-visible strings in clap, welcome hero, project picker, OAuth HTML/message, subscription messaging
- `/feedback` dispatch path in pager notes/feedback

</code_context>

<specifics>
## Specific Ideas

- Avoid half-rename: user must not still “feel” Grok Build when using bum (PROJECT Naming)
- Private/team daily driver first — no stock x.ai install overwrite (PROJECT Out of Scope / Key Decisions)
- Phase 1 already deferred chrome + broader env polish + npm channel here; honor that split
- Model id `grok-build` and Grok model labels are routing/model identity, not product chrome — do not break routing by renaming model IDs

</specifics>

<deferred>
## Deferred Ideas

- Official public distribution / signed x.ai install channel for bum — PROJECT out of scope
- Internal monorepo crate rename (`xai-grok-*` → bum crates) — not v1
- Mass `GROK_*` → `BUM_*` env rename for every knob — leave internal unless a var forces stock home coupling
- Live dual-provider daily-driver E2E validation — Phase 9
- Custom agentic workflows — later milestone

</deferred>
