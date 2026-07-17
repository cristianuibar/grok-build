# Phase 8: Quiet fork & rebrand polish - Research

**Researched:** 2026-07-17
**Domain:** Brownfield Rust TUI/CLI product rebrand + privacy (auto-update kill, telemetry/feedback phone-home off)
**Confidence:** HIGH

## Summary

Phase 8 is a **quiet-fork polish** on the existing Rust workspace — not a stack change and not a crate rename. Phase 1 already shipped the `bum` binary and `~/.bum` / `BUM_HOME` isolation; this phase finishes product identity and privacy so the user never feels “stock Grok Build” and the process does not download stock binaries or ship analytics to xAI by default.

Three independent seams already exist and should be reused, not reinvented:

1. **Product chrome strings** — concentrated in pager welcome/hero, clap (`app/cli.rs`), project picker, billing/subscription copy, OAuth return HTML, pager-minimal welcome, pager-bin version banner. Model catalog labels (`Grok Build (xAI)` for id `grok-build`) must stay. [VERIFIED: codebase]
2. **Auto-update** — chokepoints in `xai-grok-pager-bin` (`should_check_for_updates`, `run_update_command`, `enforce_minimum_version_or_exit`) and `xai-grok-update` (`run_update_if_available`, `check_update_background`, `ensure_latest_on_disk`, `print_update_status`). Stock channel is `https://x.ai/cli`. Today `auto_update: None` **defaults ON** and first-run **persists** `true`. [VERIFIED: codebase]
3. **Telemetry / feedback** — `TelemetryMode` already defaults to `Disabled` in resolve path; Sentry is gated by `is_error_reporting_disabled_sync()` (inherits telemetry-off). **Feedback defaults ON** (`resolve_feedback().default(true)`), and the **pager `/feedback` path does not check the gate** before emitting `Effect::SendFeedback` → ACP `x.ai/feedback`. [VERIFIED: codebase]

**Primary recommendation:** Centralize a small `product_name()` / brand constant (or equivalent single-source strings module) for chrome; hard-off auto-update at `should_check_for_updates` + invert `unwrap_or(true)`/`settings default true` + no-op CLI/min-version; short-circuit feedback at pager dispatch and flip feedback default to false; keep Mixpanel/Sentry inert under Disabled without baking new phone-home paths; prove with unit + hermetic gate tests (Phase 1 `home_isolation` style).

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

#### Product string rebrand (ID-02)
- All product chrome renames to **bum**: welcome/hero, clap about/help, project picker, OAuth browser return text, feedback toasts, subscription/status copy that names the product — not merely a subset of screens
- Keep **model brands** as model brands (`Grok` / GPT catalog names stay); only the *product* shell is `bum`
- Leave internal crate/package names (`xai-grok-*`) unchanged — user-facing only (Phase 1 lock)
- In-tree user-facing help/docs only — no public x.ai signed install channel polish in this phase

#### Auto-update kill (OPS-01)
- **Hard-off** stock xAI auto-update: no background check/download from the stock x.ai channel
- Explicit `bum update` / Ctrl+U-style path: **no-op + clear message** that the stock channel is disabled for this fork; point users at local build/install
- Minimum-version / forced update policy from stock: **ignore / disable** — never force-upgrade to stock Grok Build
- Enforce via **code defaults + tests** (`auto_update` default false; startup check skipped; gate tests prove no stock update phone-home)

#### Telemetry kill (OPS-02)
- Default product telemetry mode: **Disabled** (not SessionMetrics, not Enabled)
- Mixpanel / product analytics events to xAI: **off** by default
- Sentry / crash phone-home: **off** by default for the quiet fork
- Local tracing / log files under bum home remain fine; no easy stock phone-home opt-in path as the v1 default

#### CLI surface, env knobs & residual identity
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

### Deferred Ideas (OUT OF SCOPE)
- Official public distribution / signed x.ai install channel for bum — PROJECT out of scope
- Internal monorepo crate rename (`xai-grok-*` → bum crates) — not v1
- Mass `GROK_*` → `BUM_*` env rename for every knob — leave internal unless a var forces stock home coupling
- Live dual-provider daily-driver E2E validation — Phase 9
- Custom agentic workflows — later milestone
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| ID-02 | Product UI chrome, help text, and user-facing strings present as **bum**, not stock Grok Build | File inventory of chrome sites; keep model catalog `Grok Build (xAI)`; UI-SPEC locked copy; clap `name`/`about`; hero/project picker/OAuth/billing |
| OPS-01 | Stock xAI auto-update channel disabled so bum is not overwritten by official Grok Build updates | `should_check_for_updates`, `run_update_if_available` None→true trap, CLI `run_update_command`, `enforce_minimum_version_or_exit`, leader hourly checker, settings default |
| OPS-02 | Product telemetry / phone-home to xAI analytics disabled by default for the fork | `TelemetryMode::Disabled` default path; Sentry `disabled` flag; Mixpanel only when Enabled; feedback default true trap + pager ungated SendFeedback |
</phase_requirements>

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Clap CLI name/about/help | Composition root + pager CLI (`xai-grok-pager` `app/cli.rs`) | pager-bin banners | User-facing CLI identity |
| Welcome hero / version badge / subtitle | Browser/Client (TUI) `views/welcome/*` | pager-minimal | First-screen product chrome |
| Project picker copy | TUI QuestionView | — | Product-named onboarding |
| OAuth browser return text | API/Backend shell `auth/oidc/login.rs` | — | Browser HTML/message after login |
| Billing / free-usage product strings | TUI dispatch (`billing.rs`, `app_view`) | shell subscription paths | Product-named upsell only (keep SuperGrok commercial) |
| `/feedback` quiet path | TUI dispatch (`notes.rs`) + effects | shell `extensions/feedback.rs` | Pager currently bypasses feedback gate |
| Auto-update startup / background | Composition root (`pager-bin`) | `xai-grok-update` | Central gate `should_check_for_updates` |
| Explicit `bum update` | pager-bin `run_update_command` | update crate printers | No-op + locked message |
| Min-version force upgrade | `xai-grok-update` `minimum_version.rs` | pager-bin call sites | Must never install stock binary |
| Leader hourly update | shell `run_auto_update_checker` | pager-bin `LeaderAutoUpdateConfig` | Skip spawn / check_fn always false |
| Product telemetry (Mixpanel/events) | `xai-grok-telemetry` + shell `resolve_telemetry_mode` | remote settings | Default Disabled already; harden against remote re-enable if required |
| Sentry crash phone-home | pager-bin `sentry::init` + `is_error_reporting_disabled_sync` | telemetry crate | Off when telemetry/error reporting disabled |
| Local logs under `~/.bum` | config paths (Phase 1) | tracing | Allowed; not telemetry |

## Standard Stack

> No new third-party packages. Brownfield string/config/behavior on existing crates. [VERIFIED: codebase]

### Core

| Library / surface | Version / pin | Purpose | Why standard |
|-------------------|---------------|---------|--------------|
| Rust toolchain | **1.92.0** (`rust-toolchain.toml`) | Build product | Workspace pin [VERIFIED: environment] |
| `xai-grok-pager` | path crate | TUI chrome, clap, feedback, billing | Owns user-visible product strings [VERIFIED: codebase] |
| `xai-grok-pager-bin` | path crate | Startup update checks, Sentry init, version banner | Composition root gates [VERIFIED: codebase] |
| `xai-grok-update` | path crate | Stock channel fetch/install, status print, min-version | OPS-01 authority [VERIFIED: codebase] |
| `xai-grok-telemetry` | path crate | TelemetryMode, Mixpanel client, Sentry | OPS-02 authority [VERIFIED: codebase] |
| `xai-grok-shell` | path crate | OAuth return text, resolve_telemetry/feedback, leader auto-update | Auth + config resolve [VERIFIED: codebase] |
| clap 4 | workspace | CLI name/about | Existing CLI surface [VERIFIED: STACK] |
| ratatui 0.29 | workspace | Welcome/hero render | Existing TUI [VERIFIED: STACK] |

### Supporting

| Library / surface | Purpose | When to use |
|-------------------|---------|-------------|
| `xai-grok-pager-minimal` | Minimal welcome product name | Rebrand `"Grok Build"` → `"bum"` |
| `xai-grok-test-support` | Env guards, hermetic cmd | Gate tests mirroring Phase 1 |
| `serial_test` + `tempfile` | Env-mutating unit tests | Telemetry/feedback default tests |
| `mockito` / `wiremock` | HTTP fixtures | Only if proving no network to x.ai/cli (prefer pure gate unit tests) |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Hard-off in code defaults | Rely only on `GROK_DISABLE_AUTOUPDATER=1` / config | Env-only fails release default path — **rejected by OPS-01** |
| Delete update crate code | No-op wrappers at chokepoints | Keep crate for local version display / future own channel; less blast radius |
| Mass `Grok Build` sed across monorepo | Surgical product-chrome inventory | Model labels + docs/comments would break routing identity |
| Hide `/feedback` command | Disabled message (UI-SPEC) | Discoverable + clear is locked UX |

**Installation:** none (no new crates).

**Version verification:** toolchain 1.92.0 present; no registry packages added. [VERIFIED: environment]

## Package Legitimacy Audit

> Phase installs **no** external packages.

| Package | Registry | Age | Downloads | Source Repo | Verdict | Disposition |
|---------|----------|-----|-----------|-------------|---------|-------------|
| — | — | — | — | — | — | N/A — no new deps |

**Packages removed due to [SLOP] verdict:** none  
**Packages flagged as suspicious [SUS]:** none

## Project Constraints (from CLAUDE.md / AGENTS.md)

Actionable directives that constrain this phase (from project AGENTS.md / PROJECT):

- Stay on this **Rust workspace** (edition 2024, Tokio, existing TUI/agent crates) — no rewrite
- **Privacy:** No xAI auto-update; no product telemetry phone-home
- **Naming:** Product and CLI are `bum`; avoid half-rename that still presents as Grok Build
- **Storage:** `~/.bum` isolation already done (Phase 1); leave internal `GROK_*` non-home knobs
- **Internal crates** stay `xai-grok-*`
- Prefer **per-crate** `cargo test -p <crate>` / targeted filters (full workspace is heavy)
- Use `dunce::canonicalize` (clippy ban on raw canonicalize)
- GSD workflow: research → plan → execute; commit docs via gsd-tools when `commit_docs: true`

## Current State Inventory

### A. User-facing product chrome (ID-02) — must rebrand [VERIFIED: codebase]

| Site | Current stock string | Locked target (UI-SPEC) |
|------|----------------------|-------------------------|
| `xai-grok-pager/src/app/cli.rs` | `name = "grok"`, `about = "Grok Build TUI"` | `bum` / `bum TUI` |
| `xai-grok-pager/src/app/mod.rs` | `"Grok Build TUI"` (error/context string) | `bum TUI` |
| `xai-grok-pager/src/views/welcome/hero_box.rs` | `HERO_SUBTITLE` advertises `/feedback` | `Thanks for using bum.` |
| `xai-grok-pager/src/views/welcome/mod.rs` | badge `Grok Build  `, `Grok Build Beta  `, ZDR, workspace trust L1 | `bum` (+ drop Beta marketing); ZDR/trust product rename |
| `xai-grok-pager/src/project_picker/mod.rs` | `Run Grok Build in a project directory?…` | Locked bum question |
| `xai-grok-pager/src/app/dispatch/notes.rs` | thank-you + `Effect::SendFeedback` | Disabled path; no team thank-you |
| `xai-grok-pager/src/app/dispatch/billing.rs` | free usage / purchase / get most of **Grok Build** | product → bum; keep SuperGrok URL |
| `xai-grok-pager/src/app/app_view.rs` | `Subscribe to use Grok Build` | `Subscribe to use bum` |
| `xai-grok-shell/src/auth/oidc/login.rs` | `return to Grok Build` | `return to bum` |
| `xai-grok-pager-bin/src/main.rs` | `Grok Build (pager) - v{}` | `bum (pager) - v{}` |
| `xai-grok-pager-minimal/src/welcome.rs` | `"Grok Build"` | `"bum"` |
| `xai-grok-update/src/auto_update.rs` | `Grok Build - v…`, new version available | bum / disabled path messages |

### B. Do **not** rebrand (model / internal) [VERIFIED: codebase + CONTEXT]

| Kind | Examples |
|------|----------|
| Model catalog display | `default_models.json` `"name": "Grok Build (xAI)"`, picker rows, ACP `ModelInfo` tests |
| Model id slug | `grok-build` |
| Crate/docs comments | Cargo.toml descriptions, rustdoc “for Grok Build” |
| Agent system prompts / toolset descriptions | e.g. “Grok Build agent for software engineering…” — **discretion**: prefer rebrand only if models introduce themselves as product; not required for ID-02 chrome; avoid breaking prompt snapshots without tests |
| `GROK_*` env names | Leave internal |
| SuperGrok commercial | Keep as provider SKU |

### C. Auto-update call graph (OPS-01) [VERIFIED: codebase]

```
pager-bin startup (release builds)
  ├─ enforce_minimum_version_or_exit(update_config)  // may download stock if floor set
  ├─ should_check_for_updates(no_auto_update)
  │     false if: debug_assertions | --no-auto-update | GROK_DISABLE_AUTOUPDATER set
  │     true otherwise  ← RELEASE DEFAULT ON
  ├─ auto_update::run_update_if_available(NonBlocking|Blocking)
  │     skips if auto_update == Some(false)
  │     None → unwrap_or(true) AND may persist auto_update=true  ← FIRST-RUN OPT-IN TRAP
  │     fetches https://x.ai/cli (CLI_BASE_URL_PRIMARY)
  ├─ check_update_background → may spawn detached `update` child
  └─ LeaderAutoUpdateConfig { check_fn: ensure_latest_on_disk loop hourly }

Explicit CLI: Command::Update → run_update_command
  ├─ --check → check_update_status + print_update_status (Grok Build strings)
  └─ else → run_update (install from stock channel)
```

**Critical traps:**

1. `run_update_if_available`: `cli.auto_update.unwrap_or(true)` + first-run write of `Some(true)`. [VERIFIED: `auto_update.rs` ~486–494]
2. Settings registry: `auto_update` Bool default **true**; description promises automatic install. [VERIFIED: `settings/defs.rs`]
3. Min-version: only treats `Some(false)` as opt-out; otherwise installs stock. [VERIFIED: `minimum_version.rs` ~205–210]
4. Debug builds already skip via `cfg!(debug_assertions)` — **release/dist builds do not**.

### D. Telemetry / Sentry / feedback (OPS-02) [VERIFIED: codebase]

| Path | Default today | Quiet-fork target |
|------|---------------|-------------------|
| `resolve_telemetry_mode` | `TelemetryMode::Disabled` when unset | Keep Disabled; consider ignoring remote enable for fork hard privacy |
| Mixpanel / product events | Client not active when Disabled | Remain off |
| `TelemetryConfig::default` | `mixpanel_enabled` true if compile-time token present | Mode gate still blocks emit; ensure no accidental Enabled |
| Sentry `init(disabled: is_error_reporting_disabled_sync())` | Disabled inherits telemetry-off → **off by default** | Keep off; hard-disable if DSN baked and someone enables reporting |
| `resolve_feedback` | **default true** | **default false** + locked UX |
| Pager `dispatch_enter_feedback_mode` / `dispatch_send_feedback` | **No gate** — always can send | Short-circuit before Effect |
| Shell `handle_feedback` | Checks `is_feedback_enabled()` | Align message with UI-SPEC (no “set GROK_FEEDBACK_ENABLED=true” easy opt-in as primary copy) |

Local tracing / files under bum home: allowed (Phase 1 paths).

## Architecture Patterns

### System Architecture Diagram

```text
User CLI / TUI
    │
    ▼
┌─────────────────────────────────────────────────────────┐
│ xai-grok-pager-bin (composition root)                   │
│  • clap parse (name=bum)                                │
│  • sentry::init(disabled=?)                             │
│  • should_check_for_updates ──► SKIP (hard-off)         │
│  • enforce_minimum_version ──► NO-OP (ignore floor)     │
│  • Command::Update ──► print disabled message, exit 0   │
└───────────────┬───────────────────────────┬─────────────┘
                │                           │
                ▼                           ▼
┌───────────────────────────┐   ┌───────────────────────────┐
│ xai-grok-pager (TUI)      │   │ xai-grok-shell            │
│  welcome/hero "bum"       │   │  OAuth "return to bum"    │
│  project picker bum copy  │   │  resolve_telemetry_mode   │
│  /feedback → local msg    │   │  resolve_feedback → false │
│  NO Effect::SendFeedback  │   │  leader update checker off│
│  billing product → bum    │   │                           │
└───────────────────────────┘   └─────────────┬─────────────┘
                                              │
                ┌─────────────────────────────┼─────────────────┐
                ▼                             ▼                 ▼
     xai-grok-update                 xai-grok-telemetry    local logs
     (no x.ai/cli fetch)             (mode Disabled;       under ~/.bum
     local version string only        Mixpanel/Sentry off)
```

### Recommended Project Structure

No new crates. Optional small brand module:

```
crates/codegen/xai-grok-pager/src/
  brand.rs            # optional: product_name(), HERO_SUBTITLE, etc.
  views/welcome/…
  app/cli.rs
  app/dispatch/notes.rs
  project_picker/mod.rs

crates/codegen/xai-grok-update/src/
  auto_update.rs      # defaults + print + early returns
  minimum_version.rs  # hard no-op / Allow always for fork

crates/codegen/xai-grok-pager-bin/src/
  main.rs             # should_check_for_updates → false; update command no-op

crates/codegen/xai-grok-shell/src/
  auth/oidc/login.rs
  agent/config.rs     # feedback default false; telemetry remains Disabled
```

### Pattern 1: Central product brand constant
**What:** One `pub const PRODUCT_NAME: &str = "bum"` (or `product_name()`) used by chrome formatters.  
**When to use:** Multiple crates need the same user-facing name (pager, update printers, shell OAuth). Prefer thin constant in a leaf already depended on, or duplicate the three-letter string surgically if dependency edges hurt — both OK per CONTEXT discretion.  
**Recommendation:** Surgical edits first (UI-SPEC inventory is small); extract constant only if ≥2 crates share format helpers.

### Pattern 2: Hard-off at chokepoint, not scatter
**What:** Flip `should_check_for_updates` to always `false` (or `false` unless explicit future fork channel); invert update defaults to `unwrap_or(false)`; make `run_update_command` early-return with locked copy; make `enforce_minimum_version_or_exit` return immediately.  
**When to use:** Always for OPS-01 — do not rely on users setting env vars.

### Pattern 3: Feedback disabled at entry
**What:** `dispatch_enter_feedback_mode` / `dispatch_send_feedback` push system block with UI-SPEC copy and return `vec![]` (no Effect). Optionally flip `resolve_feedback` default to `false` so shell ACP path also refuses.  
**When to use:** OPS-02 adjacent + UI-SPEC locked UX.

### Anti-Patterns to Avoid
- **Mass sed of `Grok Build`:** breaks model catalog, tests, routing labels
- **Leaving `unwrap_or(true)` for auto_update:** re-enables stock channel on fresh installs
- **Only setting env in tests:** release binary still phones home
- **Rebranding SuperGrok / grok.com billing hosts:** commercial provider surface
- **Easy opt-in copy** (“set GROK_FEEDBACK_ENABLED=true”) as the primary disabled message — contradicts quiet-fork intent

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Auto-update policy | New update subsystem | Existing `should_check_for_updates` + config `cli.auto_update` | Already multi-callsite; one gate |
| Telemetry off | Custom analytics shim | Existing `TelemetryMode::Disabled` + client mode checks | Client already no-ops |
| Sentry off | Remove sentry crate | `sentry::Config { disabled: true }` / error-reporting resolve | Guard already exists |
| Feedback disable | Delete slash command | Short-circuit dispatch + `resolve_feedback` default false | UI-SPEC: discoverable disabled |
| Product name | Macro rewrite of monorepo | Surgical chrome inventory + optional constant | Minimal blast radius |

**Key insight:** The dangerous behavior is **default-on network** (auto-update None→true, feedback default true, release `should_check_for_updates` true), not missing libraries.

## Runtime State Inventory

> Not a rename/migration phase of stored keys; product home already `~/.bum`. Explicit answers for planner:

| Category | Items Found | Action Required |
|----------|-------------|------------------|
| Stored data | Existing `~/.bum/config.toml` may have `cli.auto_update = true` or absent (treated as true today); `version.json` update cache | Code: treat default false; optional: treat None as false so old true must be explicit; no DB migration |
| Live service config | None for product brand (no n8n/Datadog) | None |
| OS-registered state | None tied to “Grok Build” product string for this fork | None |
| Secrets/env vars | `GROK_*` telemetry/update knobs remain names; `SENTRY_DSN` may be compile-time | Code: disabled by default; do not rename env family (CONTEXT) |
| Build artifacts | `bum` binary already; release-dist may bake Mixpanel/Sentry tokens via `option_env!` | Ensure mode/disabled gates hold even if tokens baked |

**Nothing found requiring data migration of credentials/sessions for rebrand.**

## Common Pitfalls

### Pitfall 1: Half-rename (product still “Grok Build”)
**What goes wrong:** Hero says bum but clap/`bum update`/billing still say Grok Build.  
**Why:** Scattered string sites; tests assert model labels and get mixed with product greps.  
**How to avoid:** Execute UI-SPEC inventory as checklist; grep gate for product chrome files excluding model fixtures.  
**Warning signs:** `bum --help` still “Grok Build TUI”.

### Pitfall 2: Auto-update None defaults true
**What goes wrong:** Fresh `config.toml` without `auto_update` still downloads from x.ai.  
**Why:** `unwrap_or(true)` + first-run persistence.  
**How to avoid:** Invert to `unwrap_or(false)`; stop persisting true on first run; set settings default false; gate `should_check_for_updates` hard false.  
**Warning signs:** Network to `x.ai/cli` on release start; settings show Auto-update On.

### Pitfall 3: Min-version still force-installs
**What goes wrong:** Managed/remote `cli.minimum_version` triggers stock install even with background checks off.  
**Why:** Separate code path `enforce_minimum_version_or_exit`.  
**How to avoid:** No-op enforcer for fork (always Allow) or treat as AutoUpdateDisabled without exit.  
**Warning signs:** Startup “Updating to …” against x.ai.

### Pitfall 4: Feedback still phones home from TUI
**What goes wrong:** Shell gate is true/false but pager never consults it.  
**Why:** `dispatch_send_feedback` always emits Effect.  
**How to avoid:** Short-circuit at pager entry; also default resolve_feedback false.  
**Warning signs:** ACP method `x.ai/feedback` in traffic after `/feedback`.

### Pitfall 5: Renaming model “Grok Build” rows
**What goes wrong:** Catalog/tests/routing identity break.  
**Why:** Same English phrase for model product vs CLI product.  
**How to avoid:** Explicit allowlist of chrome files; never edit `default_models.json` name for this phase.  
**Warning signs:** Model picker loses “Grok Build (xAI)”.

### Pitfall 6: Breaking settings e2e defaults
**What goes wrong:** Many tests assert `auto_update` default true.  
**Why:** `settings/registry.rs` and `settings_e2e.rs` hardcode default true.  
**How to avoid:** Update registry assert + e2e in same plan as default flip.  
**Warning signs:** `settings_e2e` failures on `auto_update` default.

### Pitfall 7: Remote settings re-enable telemetry
**What goes wrong:** Prefetch remote `telemetry_enabled: true` turns analytics on.  
**Why:** `resolve_telemetry_mode` honors remote after config.  
**How to avoid:** For quiet fork, either ignore remote telemetry flags or pin Disabled above remote (discretion; CONTEXT: no easy phone-home). Prefer hard pin Disabled for product events unless user/env explicit — document decision in plan.  
**Warning signs:** Mixpanel traffic after remote settings load.

## Code Examples

### Product clap identity
```rust
// Target: crates/codegen/xai-grok-pager/src/app/cli.rs
#[derive(Debug, Clone, Parser)]
#[command(
    name = "bum",
    version = version_with_channel(),
    about = "bum TUI",
    // …
)]
pub struct PagerArgs { /* … */ }
```
[VERIFIED: current file uses `name = "grok"`, `about = "Grok Build TUI"`]

### Auto-update hard-off chokepoint
```rust
// Target: crates/codegen/xai-grok-pager-bin/src/main.rs
fn should_check_for_updates(_no_auto_update_flag: bool) -> bool {
    // Quiet fork: never probe stock x.ai auto-update channel.
    false
}
```
[ASSUMED: recommended shape; current implementation returns true in release when flags unset]

### Feedback disabled at dispatch
```rust
// Target: crates/codegen/xai-grok-pager/src/app/dispatch/notes.rs
pub(super) fn dispatch_enter_feedback_mode(app: &mut AppView) -> Vec<Effect> {
    with_active_agent(app, |agent| {
        agent.scrollback.push_block(RenderBlock::system(
            "Feedback is disabled in bum (no phone-home).".to_string(),
        ));
        // stay Normal — do not enter Feedback mode
    });
    vec![]
}
```
[ASSUMED: recommended; current enters Feedback mode]

### Telemetry default (already correct)
```rust
// crates/codegen/xai-grok-shell/src/agent/config.rs — resolve_telemetry_mode
Resolved::new(TelemetryMode::Disabled, ConfigSource::Default)
```
[VERIFIED: codebase]

## State of the Art

| Old Approach (stock Grok Build) | Current Approach (bum target) | When Changed | Impact |
|---------------------------------|-------------------------------|--------------|--------|
| CLI name `grok` / Grok Build TUI | `bum` / bum TUI | Phase 8 | User identity |
| auto_update default true | default false + hard skip | Phase 8 | No overwrite by stock |
| Feedback default on + HTTP to x.ai | Disabled + local message | Phase 8 | No feedback phone-home |
| TelemetryMode default Disabled | Keep Disabled (harden) | Already default; Phase 8 verify | Quiet analytics |
| Binary `bum` + `~/.bum` | Shipped Phase 1 | 2026-07-16 | Path isolation done |

**Deprecated/outdated for this fork:**
- Stock install one-liners (`curl https://x.ai/cli/install.sh`) as user guidance in update messages — replace with local build/install language (UI-SPEC)
- “Grok Build Beta” marketing badge

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Ignoring remote-settings telemetry re-enable is desired for quiet fork | Pitfall 7 / OPS-02 | If teams rely on remote enable, they lose that path — confirm if any remote telemetry needed |
| A2 | Agent system prompts saying “Grok Build agent” can stay for v1 chrome scope | Inventory B | Model may still introduce as Grok Build in chat — optional follow-up |
| A3 | Exit code 0 for disabled `bum update` is preferred | UI-SPEC / OPS-01 | Scripts expecting non-zero on “failed update” may differ — UI-SPEC prefers 0 + message |
| A4 | No compile-time SENTRY_DSN/Mixpanel token in local developer builds | Telemetry | Dist builds with baked tokens still safe only if mode stays Disabled |

**If empty table was expected:** remaining claims are codebase-verified; A1–A4 need planner awareness only.

## Open Questions

1. **Remote telemetry / feedback feature flags**
   - What we know: resolve path can re-enable from remote settings after default Disabled/false.
   - What's unclear: whether any managed deployment needs remote toggle.
   - Recommendation: hard-prefer local Disabled/false for product events and feedback; document env override only for developers (`GROK_TELEMETRY_ENABLED`) without advertising in UX.

2. **Agent/system prompt product naming**
   - What we know: templates and agent descriptions still say Grok Build.
   - What's unclear: whether ID-02 “user-facing” includes model self-introduction.
   - Recommendation: defer to discretion / Phase 9 if chrome greps clean; optional small prompt rebrand plan if user wants model to say bum.

3. **In-tree user-guide markdown under pager/docs**
   - What we know: CONTEXT allows in-tree help/docs polish; many `docs/user-guide/*.md` say Grok Build.
   - What's unclear: depth for v1.
   - Recommendation: rebrand user-guide product name if extracted to `~/.bum/docs` (pager-bin extracts docs); skip CHANGELOG archaeology.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| rustc / cargo | build & test | ✓ | 1.92.0 | — |
| Existing crates only | all features | ✓ | workspace | — |
| Network to x.ai | **must not be required** for phase success | N/A | — | Hard-off gates; tests must not need live x.ai |

**Missing dependencies with no fallback:** none  
**Step 2.6:** No new external tools; network absence is a success property for OPS-01/02.

## Validation Architecture

> `workflow.nyquist_validation` is enabled in `.planning/config.json`.

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust `cargo test` (crate-local) + existing integration tests |
| Config file | per-crate; no jest/pytest |
| Quick run command | `cargo test -p xai-grok-pager --lib p8_ -- --nocapture` (and sibling crate filters) |
| Full suite command | Targeted: pager + update + shell config + pager-bin isolation; not full workspace |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| ID-02 | clap name/about contain bum not Grok Build product | unit | `cargo test -p xai-grok-pager --lib p8_cli_brand` | ❌ Wave 0 |
| ID-02 | hero subtitle / badge constants | unit | `cargo test -p xai-grok-pager --lib p8_welcome` | ❌ Wave 0 |
| ID-02 | model catalog still “Grok Build (xAI)” | unit/regression | existing model catalog tests | ✅ |
| OPS-01 | `should_check_for_updates` always false (or equivalent pure gate) | unit | `cargo test -p xai-grok-pager-bin p8_no_auto_update` | ❌ Wave 0 |
| OPS-01 | auto_update effective default false; no first-run true persist | unit | `cargo test -p xai-grok-update p8_auto_update_default` | ❌ Wave 0 |
| OPS-01 | min-version enforcer does not install | unit | `cargo test -p xai-grok-update p8_min_version` | ❌ Wave 0 |
| OPS-01 | `run_update_command` / status path no stock phone-home (mock or pure early return) | unit/integration | `cargo test -p xai-grok-pager-bin p8_update_cmd` | ❌ Wave 0 |
| OPS-02 | resolve_telemetry_mode default Disabled | unit | existing + `p8_telemetry` if needed | ✅ partial |
| OPS-02 | resolve_feedback default false | unit | update `resolve_feedback_defaults_to_true` → false | ✅ exists (must flip) |
| OPS-02 | pager feedback dispatch emits no SendFeedback | unit | `cargo test -p xai-grok-pager --lib p8_feedback` | ❌ Wave 0 |
| ALL | hermetic no stock update when env traps set | integration | extend `pager-bin/tests/home_isolation.rs` pattern | ✅ pattern |

### Sampling Rate
- **Per task commit:** filtered `p8_` tests in touched crate
- **Per wave merge:** all `p8_` + affected settings/update unit tests
- **Phase gate:** `p8_` green + model catalog regression green + home_isolation still green

### Wave 0 Gaps
- [ ] `p8_` unit tests for clap/welcome/feedback dispatch (pager)
- [ ] `p8_` unit tests for auto-update default + min-version no-op (update)
- [ ] Gate test: update path does not call fetch when hard-off (mock installer / early return)
- [ ] Flip existing `resolve_feedback_defaults_to_true_when_unset` expectation
- [ ] Update settings registry default assertion for `auto_update`
- [ ] PTY e2e strings that assert welcome banner `"Grok Build"` → `"bum"` (`minimal_*` tests)

### Recommended plan split (for planner)

| Wave | Focus | Crates | Req |
|------|-------|--------|-----|
| 0 | RED `p8_` harness + flip known default assertions | pager, update, shell config, pager-bin | all |
| 1 | ID-02 product chrome (clap, hero, picker, OAuth, billing, banners, minimal) | pager, shell, pager-bin, pager-minimal | ID-02 |
| 2 | OPS-01 hard-off (gates, defaults, CLI no-op, min-version, leader, settings copy) | pager-bin, update, pager settings, shell leader wiring | OPS-01 |
| 3 | OPS-02 feedback short-circuit + feedback default false + telemetry/Sentry verify + remote hardening if chosen | pager, shell, telemetry | OPS-02 |
| 4 | Phase gate: p8_ discovery, model-label regression, home_isolation, string inventory checklist | multi | all |

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no (not this phase) | — |
| V3 Session Management | no | — |
| V4 Access Control | no | — |
| V5 Input Validation | yes (light) | Feedback text unused on quiet path; clap still validates CLI |
| V6 Cryptography | no new crypto | — |
| Privacy / data exfiltration | **yes** | Disable auto-update downloads, Mixpanel, Sentry, feedback POST by default |

### Known Threat Patterns for quiet fork

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Stock binary overwrite via auto-update | Tampering | Hard-off channel + min-version ignore |
| Analytics / crash phone-home | Information disclosure | TelemetryMode::Disabled; Sentry disabled; no baked opt-in UX |
| Feedback POST to x.ai | Information disclosure | Pager short-circuit + feedback default false |
| Accidental model-id rename | Denial of service / breakage | Do not edit catalog model brands |
| Secrets in Sentry/Mixpanel if re-enabled | Information disclosure | Existing `xai-grok-secrets` scrubber; keep off by default |

## Sources

### Primary (HIGH confidence)
- Codebase: `xai-grok-update/src/auto_update.rs`, `minimum_version.rs`, `version.rs` (`CLI_BASE_URL_PRIMARY`)
- Codebase: `xai-grok-pager-bin/src/main.rs` (`should_check_for_updates`, Sentry init, version banner, update command)
- Codebase: `xai-grok-telemetry/src/config.rs`, `sentry.rs`, `client.rs`
- Codebase: `xai-grok-shell/src/agent/config.rs` (`resolve_telemetry_mode`, `resolve_feedback`, error reporting sync)
- Codebase: pager welcome/cli/project_picker/notes/billing; shell OIDC login
- Codebase: `xai-grok-pager-bin/tests/home_isolation.rs` Phase 1 isolation pattern
- Phase artifacts: `08-CONTEXT.md`, `08-UI-SPEC.md`, `REQUIREMENTS.md`, `ROADMAP.md`, `01-RESEARCH.md`

### Secondary (MEDIUM confidence)
- None required beyond codebase for this brownfield privacy/rebrand phase

### Tertiary (LOW confidence)
- A1–A4 in Assumptions Log (policy preference edges)

## Metadata

**Confidence breakdown:**
- Standard stack: **HIGH** — no new deps; crates and call sites read in-tree
- Architecture: **HIGH** — clear chokepoints for update/telemetry/feedback/chrome
- Pitfalls: **HIGH** — defaults-on traps verified in source; settings e2e coupling known

**Research date:** 2026-07-17  
**Valid until:** 2026-08-16 (stable brownfield; re-verify if update/telemetry modules refactor)
