# Phase 9: Daily-driver end-to-end validation - Research

**Researched:** 2026-07-17
**Domain:** Hybrid validation (fixture regression gates + live dual-login human UAT) for OPS-03..OPS-06 on brownfield Rust CLI/TUI
**Confidence:** HIGH

## Summary

Phase 9 is **validation-first**, not a feature build. Phases 1–8 already shipped identity, dual-slot auth, catalog, routing, Codex OAuth, mid-session switch, cross-provider subagents, and quiet-fork polish. What remains is proving **bum is a real daily driver** under live xAI + ChatGPT/Codex OAuth for OPS-03..OPS-06, while CI stays fixture-only. Phase 7 explicitly deferred **live multi-turn dual-login NL E2E** here; Phase 8 deferred the same for daily-driver E2E. [VERIFIED: codebase + planning artifacts]

The standard approach is already established in-repo: **discover+execute green-only phase gates** (`06/07/08-PHASE-GATE.md`) plus a **human checklist elevated from advisory → required**. Do not invent a new verification product, live-OAuth CI job, or full-catalog matrix. Re-invoke critical prior filters (`p6_dual_login`, `p7_isolation` both dirs, `p7_eager`/`p7_spawn_missing_provider`, quiet-fork `p8_*` + `home_isolation`), add thin **fixture-only** `p9_` residual smoke, prepare a structured **09-UAT** checklist for Cristian, and fill **09-VERIFICATION** with both automated results and signed live evidence. Product code lands only when live UAT finds blockers that violate OPS-03..06. [VERIFIED: 09-CONTEXT.md, 07/08-PHASE-GATE.md, 09-PATTERNS.md]

**Primary recommendation:** Plan waves as **harness/docs → prior-gate subset + thin p9_ → UAT runbook → live dual-login execute (human) → hybrid phase gate + VERIFICATION**. Gate GREEN ⇔ automated discover+execute green **∧** signed UAT PASS for all four OPS rows under real OAuth — never fixture-only for live OPS rows.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

#### Validation methodology
- **Hybrid proof:** green automated regression (prior `p*_` / phase gates + thin `p9_` smoke with fixtures) **plus** live dual-login human UAT for OPS-03..OPS-06 — matches Phase 7 deferral of live multi-turn dual-login NL E2E to Phase 9; CI stays fixture-only
- **Phase gate requires both** automated green **and** signed human checklist covering all four OPS requirements — not automated-only or human-only
- **Fix blocking bugs in-phase** — small fail-closed / routing / chrome / tool-path fixes that block the daily-driver bar; larger feature work → deferred ideas / later milestone
- **Live UAT environment:** real dual OAuth under `~/.bum` (or isolated temp `BUM_HOME`) on the developer machine; no CI live OAuth secrets

#### Session matrix
- **OPS-03 / OPS-04 minimum:** one productive tool turn per provider after login — at least read + one of edit/shell succeeds on a real backend (not fixture-only)
- **OPS-05:** same session — xAI turn → switch to GPT-5.6 → Codex turn (optional reverse) without restarting the CLI
- **OPS-06:** both live directions — Grok parent → Codex child **and** Codex parent → Grok child (NL or Task spawn with explicit model + effort)
- **Models under test:** one current xAI daily model + one GPT-5.6 catalog entry (e.g. Sol) — not a full catalog sweep

#### Evidence & pass criteria
- **Live evidence:** structured UAT checklist in phase dir + VERIFICATION.md rows for OPS-03..06 with pass/fail and short notes (who, when, model IDs)
- **Provider capability gaps:** document honestly if Codex/OpenAI cannot support a Grok-only tool feature; remaining supported tools must still clear the daily-driver bar
- **Automated scope:** re-run critical prior-phase gates / residual greps + add thin `p9_` discovery smoke where useful; no intentional-red; no live network required for automated gate
- **Secrets:** never commit tokens, auth.json, or full raw transcripts with secrets; short redacted notes / log excerpts only

#### Scope boundary & fix policy
- **Validation-first** — harness, checklists, VALIDATION/PHASE-GATE, evidence docs; product code only for blockers found during UAT
- **Out of scope (unchanged PROJECT outs):** public x.ai install channel, crate rename (`xai-grok-*`), mass `GROK_*` rename, custom workflows, full catalog matrix, enterprise IdP
- **Human UAT:** Cristian runs the live dual-login checklist during execute; agent prepares scripts/checklists and lands automated green first
- **Live OAuth required for green OPS rows** — if network/account fails, block the human path (fix or re-auth); do not mark live OPS rows green on fixtures alone

### Claude's Discretion
- Exact checklist layout, p9_ filter names, and which prior gates are re-invoked in PHASE-GATE
- Whether live UAT is TUI, headless, or both for a given OPS row (prefer TUI daily path when practical)
- How deep automated p9_ smoke goes beyond discovery (fixture dual-token residual only)
- Copy for any new login/switch error strings if UAT finds friction (align Phase 5–7 labels)
- Whether a single UAT session can cover multiple OPS rows when natural (e.g. one dual-login session hits switch + both providers + spawn)

### Deferred Ideas (OUT OF SCOPE)
- Public signed x.ai install channel for bum
- Internal crate renames (`xai-grok-*`) / mass `GROK_*` env rename
- Custom agentic workflow engine (later milestone)
- Full model-catalog matrix / multi-account cost dashboards
- Enterprise IdP beyond existing xAI OIDC hooks
- Long multi-hour soak sessions (beyond one productive tool turn + switch + spawn matrix)
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| OPS-03 | Real coding session on xAI after xAI login (tools, edit, shell) | Live UAT TUI/headless on `grok-build` (or current xAI daily); automated residual via auth/routing/home isolation; not fixture-green alone |
| OPS-04 | Real coding session on GPT-5.6 after Codex login | Live UAT on `gpt-5.6-sol` (or current GPT-5.6 catalog entry); document tool capability gaps honestly |
| OPS-05 | Switch Grok ↔ GPT-5.6 mid-session without restart | Re-run `p6_dual_login*` + switch route tests; live same-process `/model` matrix in UAT |
| OPS-06 | Parent Grok→Codex child **and** reverse when both logged in | Re-run `p7_isolation_*` both dirs + spawn/preflight; live Task/NL spawn both directions with effort |
</phase_requirements>

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Automated regression gate (p6/p7/p8/p9) | Local dev / CI (Cargo) | — | Fixture-only discover+execute; no live secrets |
| Live OAuth login (xAI + Codex) | API / Backend (shell auth) + Browser (IdP) | CLI composition root | Real tokens only under `$BUM_HOME/auth.json` |
| Productive coding turn (read/edit/shell) | API / Backend (agent + tools) | TUI (pager) | Daily-driver bar is real backend + tools |
| Mid-session model switch | TUI + shell session | Catalog / routing | Prefer TUI `/model` path for OPS-05 |
| Cross-provider Task spawn | API / Backend (shell subagent + Task tool) | Parent model NL | Both directions; effort on spawn |
| Evidence / phase gate docs | Planning artifacts | Human operator | Hybrid GREEN needs signed checklist |
| Blocker product fixes | Same tier as broken seam | — | Fail-closed / chrome / routing only |

## Standard Stack

### Core (reuse — do not replace)

| Component | Version / location | Purpose | Why standard |
|-----------|-------------------|---------|--------------|
| Rust workspace | edition 2024, toolchain **1.92.0** | Product under test | Project constraint — fork evolution [VERIFIED: rust-toolchain.toml] |
| Cargo `cargo test` | built-in | Automated gate | Existing Nyquist pattern Phases 1–8 [VERIFIED: TESTING.md] |
| `bum` binary | `xai-grok-pager-bin` `[[bin]] name = "bum"` | Daily-driver CLI | Phase 1 ship name [VERIFIED: Cargo.toml] |
| Dual-slot auth | `$BUM_HOME/auth.json` `providers.xai` / `providers.codex` | Live OAuth store | Phases 2, 5 [VERIFIED: shell auth] |
| Catalog models | `default_models.json`: `grok-build`, `gpt-5.6-sol` / terra / luna | UAT model IDs | Phase 3 [VERIFIED: default_models.json] |
| Provider routing | shell `resolve_provider_route` / `provider_routing` tests | Route residual | Phase 4 [VERIFIED: provider_routing.rs] |
| Switch gate | `--test model_switch_gate` `p6_*` | OPS-05 auto residual | Phase 6 [VERIFIED: 06-PHASE-GATE.md] |
| Cross-provider spawn | `p7_*` lib + `--test cross_provider_subagent` | OPS-06 auto residual | Phase 7 [VERIFIED: 07-PHASE-GATE.md] |
| Quiet fork residual | `p8_*` + residual greps + `home_isolation` | Daily-driver hygiene | Phase 8 [VERIFIED: 08-PHASE-GATE.md] |
| Headless single-turn | pager `headless::run_single_turn` / `-p` / `-m` | Optional live path when TUI blocked | Prefer TUI for OPS-03..05 [VERIFIED: headless.rs] |
| Mock inference | `xai-grok-test-support` | CI fixture E2E only | **Not** live OPS proof [VERIFIED: TESTING.md] |

### Supporting

| Component | Purpose | When to use |
|-----------|---------|-------------|
| `bum login` / `bum login --provider codex` | Live OAuth entry | UAT preflight |
| `bum auth status` | Dual usable without secrets | UAT preflight + evidence note |
| `discover()` bash helper | Per-subgroup ≥1 then execute | PHASE-GATE (copy from 07/08) |
| Optional `scripts/uat-preflight.sh` in phase dir | Print steps, set `BUM_HOME` | Human helper only — never stores tokens |
| `serial_test` + `TempDir` + `OnceLock` BUM_HOME | Hermetic fixture tests | Any new `p9_` integration |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Hybrid gate (locked) | Automated-only green | Fails CONTEXT: live OAuth required for OPS rows |
| Hybrid gate (locked) | Human-only UAT | Loses regression guard on p6/p7/p8 seams |
| Re-run **subset** of prior gates | Full re-run of every 06/07/08 filter | Subset is faster and OPS-linked; full optional if time |
| TUI live path | Headless-only live matrix | Headless OK per row if TUI blocked; prefer TUI for daily-driver bar |
| Thin `p9_` residual | New full dual-login mock multi-turn | Overbuild; p6/p7 already cover fixture depth |
| Live OAuth in CI | Fixture-only CI | Locked: no CI live secrets |

**Installation:** none — no new crates or npm packages for this phase.

**Version verification:** toolchain `rustc 1.92.0` available on research host; binary `target/debug/bum` present (`bum 0.1.220-alpha.4`). [VERIFIED: local env]

## Package Legitimacy Audit

> Phase installs **no** external packages. Validation uses existing workspace deps only.

| Package | Registry | Age | Downloads | Source Repo | Verdict | Disposition |
|---------|----------|-----|-----------|-------------|---------|-------------|
| — | — | — | — | — | n/a | No new packages |

**Packages removed due to [SLOP] verdict:** none  
**Packages flagged as suspicious [SUS]:** none

## Architecture Patterns

### System Architecture Diagram

```text
                    ┌─────────────────────────────────────┐
                    │  Phase 9 Hybrid Gate                │
                    │  GREEN ⇔ auto ∧ signed UAT          │
                    └──────────────┬──────────────────────┘
                                   │
              ┌────────────────────┴────────────────────┐
              ▼                                         ▼
   ┌──────────────────────┐               ┌──────────────────────────┐
   │ Automated half (CI)  │               │ Human half (dev machine) │
   │ fixtures only        │               │ real dual OAuth          │
   └──────────┬───────────┘               └────────────┬─────────────┘
              │                                        │
   discover+execute:                        preflight:
   p6_dual_login / missing                  BUM_HOME=~/.bum|temp
   p7_isolation both dirs                   bum login [xai]
   p7_eager / spawn_missing                 bum login --provider codex
   p8_telemetry / no_update / sentry        bum auth status (no secrets)
   home_isolation                           │
   thin p9_ residual smoke                  ▼
              │                   ┌─────────────────────┐
              │                   │ TUI session (prefer) │
              │                   │ or headless -p -m    │
              │                   └──────────┬──────────┘
              │                              │
              │              OPS-03: xAI read+edit/shell
              │              OPS-04: GPT-5.6 read+edit/shell
              │              OPS-05: /model switch same process
              │              OPS-06: Task spawn both dirs + effort
              │                              │
              ▼                              ▼
   09-PHASE-GATE results          09-UAT signed rows
              │                              │
              └──────────────┬───────────────┘
                             ▼
                    09-VERIFICATION.md
                    (OPS-03..06 pass/fail + model IDs)
```

### Recommended Project Structure (phase artifacts)

```text
.planning/phases/09-daily-driver-end-to-end-validation/
├── 09-CONTEXT.md          # locked decisions (exists)
├── 09-PATTERNS.md         # pattern map (exists)
├── 09-RESEARCH.md         # this file
├── 09-VALIDATION.md       # req → auto + human map (create)
├── 09-PHASE-GATE.md       # discover+execute + human sign-off (create)
├── 09-UAT.md              # live dual-login checklist (create)
├── 09-VERIFICATION.md     # evidence after execute (create)
└── scripts/               # optional
    └── uat-preflight.sh   # BUM_HOME + checklist printer; no secrets
```

### Pattern 1: Discover + execute green-only gate
**What:** Per-subgroup `cargo test -- --list | grep -c filter ≥ 1` then execute; `&&` chains only.  
**When to use:** All automated Phase 9 subgroups and re-invoked prior filters.  
**Example:**

```bash
# Source: 07-PHASE-GATE.md / 08-PHASE-GATE.md (canonical)
discover() {
  local pkg="$1" filter="$2"; shift 2
  local extra=("$@")
  local n
  n=$(cargo test -p "$pkg" "${extra[@]}" -- --list 2>/dev/null | grep -c "$filter" || true)
  echo "discover $pkg $filter (${extra[*]:-}): n=$n"
  test "$n" -ge 1
  cargo test -p "$pkg" "${extra[@]}" "$filter" -- --nocapture
}
```

### Pattern 2: Hybrid both-halves gate
**What:** Automated green **and** signed human checklist — same spirit as Phase 6 shell+pager both halves, but human half is live OAuth.  
**When to use:** Final phase GREEN decision only.  
**Gate formula:**

```text
Gate GREEN ⇔ (all required automated subgroups discover≥1 AND execute green)
           ∧ (09-UAT signed PASS for OPS-03 ∧ OPS-04 ∧ OPS-05 ∧ OPS-06)
           ∧ (no secrets committed)
```

### Pattern 3: Thin p9_ fixture residual (not live)
**What:** Small green tests proving dual-slot fixture residual / discovery under temp `BUM_HOME`; never live network.  
**When to use:** Wave 0 / early plan only if useful beyond re-running p6/p7.  
**Fixture constants:**

```rust
// Source: cross_provider_subagent.rs / model_switch_gate.rs pattern
const XAI_FAKE: &str = "xai-fake-token-p9";
const CODEX_FAKE: &str = "codex-fake-token-p9";
const CODEX_MODEL: &str = "gpt-5.6-sol";
const XAI_MODEL: &str = "grok-build";
```

### Pattern 4: Live UAT session matrix (prefer single dual-login session)
**What:** One dual-login TUI session can cover OPS-03..06 when natural; still fill each OPS row.  
**When to use:** Human execute wave.  
**Models (recommended defaults):** xAI `grok-build` + Codex `gpt-5.6-sol`. [VERIFIED: default_models.json]

### Anti-Patterns to Avoid
- **Fixture-green as live OPS pass:** violates locked “Live OAuth required for green OPS rows”
- **Phase 3-style “optional advisory UAT”** for OPS rows — environment skip is **not** pass
- **Intentional-red under `p9_`**
- **Unfiltered** `cargo test -p xai-grok-shell --lib` or full workspace as sole gate
- **CI live OAuth** secrets / browser automation as required gate
- **Committing** `auth.json`, tokens, or raw transcripts with secrets
- **Re-implementing** full p6/p7 Authorization harness under new names instead of re-running
- **Full catalog matrix** or multi-hour soak (deferred)
- **Treating** `MockInferenceServer` / `run_headless` mock success as OPS-03/04

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Phase gate runner | New custom test framework | `discover()` + Cargo filters from 07/08 | Proven; Nyquist-compatible |
| Dual-login fixture auth | New auth store format | Existing dual-slot write helpers in shell tests | Already green for p6/p7 |
| Live OAuth automation in CI | Playwright/IdP bots + secret store | Human UAT checklist | Locked: no CI live secrets |
| Cross-provider isolation proof | New minimal harness | Re-run `p7_isolation_*` both dirs | Plan 05 already Authorization-grade |
| Mid-session switch proof | New switch simulator | Re-run `p6_dual_login*` + live `/model` | Phase 6 covered fixture; live is UAT |
| Home isolation | New process fuzzer | `--test home_isolation` | Phase 1/8 hermetic proof |
| Evidence report | Ad-hoc chat notes only | `09-UAT.md` + `09-VERIFICATION.md` | Traceability for verify-work |

**Key insight:** Phase 9 value is **composition and evidence**, not new infrastructure. The failure mode is claiming daily-driver done from fixture suites alone.

## Common Pitfalls

### Pitfall 1: Marking OPS live rows green without live OAuth
**What goes wrong:** VERIFICATION says OPS-03..06 PASS from p6/p7 only.  
**Why it happens:** Prior phases taught “fixtures = gate”; Phase 9 flips that for human half.  
**How to avoid:** Explicit dual columns (Automated | Live) in VALIDATION; refuse score green without UAT sign-off fields (who, when, model IDs).  
**Warning signs:** Empty operator/date; model IDs blank; “same as automated” notes.

### Pitfall 2: Auth expiry / refresh mid-UAT
**What goes wrong:** OPS-04 or spawn fails with 401 after long session; misread as product bug.  
**Why it happens:** Live tokens expire; refresh paths differ per provider.  
**How to avoid:** Preflight `bum auth status` usable both; re-auth on hard fail; note refresh vs product bug in UAT notes.  
**Warning signs:** One slot `usable: no` mid-session; only one provider fails after idle.

### Pitfall 3: Wrong model ID / catalog drift
**What goes wrong:** UAT uses a slug not in catalog; switch/spawn rejects.  
**Why it happens:** Live Codex catalog may union-append; embedded defaults are `gpt-5.6-sol|terra|luna`.  
**How to avoid:** Prefer embedded IDs in checklist; record actual `/model` list IDs used.  
**Warning signs:** “unknown model” tool errors; empty Codex list when not logged in (prefetch).

### Pitfall 4: One spawn direction only for OPS-06
**What goes wrong:** Grok→Codex works; reverse never run; still marked pass.  
**Why it happens:** NL prompt bias toward Grok parent.  
**How to avoid:** Checklist forces both rows; VERIFICATION fails if either direction blank.  
**Warning signs:** Only one OPS-06 row filled.

### Pitfall 5: Stale binary / residual chrome confuses UAT
**What goes wrong:** Help usage still shows `grok` examples; operator thinks rebrand failed.  
**Why it happens:** Clap arg help strings still contain `grok "…"` examples; binary help may show `Usage: grok` depending on argv/bin_name path even when about is `bum TUI`. [VERIFIED: local `target/debug/bum --help` on research host; clap `name = "bum"` + residual grok examples in cli.rs]  
**How to avoid:** UAT preflight notes version/commit; residual chrome blockers are in-phase fix candidates (ID-02 residual), not scope expansion to full crate rename.  
**Warning signs:** Usage/examples still say `grok` while chrome hero says bum.

### Pitfall 6: Cargo list poisoned by bare package
**What goes wrong:** `cargo test -p xai-grok-shell -- --list` empty/false because unrelated integration targets fail compile.  
**Why it happens:** Phase 6 lesson — bare package lists all integration tests.  
**How to avoid:** Prefer `--lib` or explicit `--test <name>` as in 06/07/08 gates.  
**Warning signs:** discover n=0 despite tests existing under `--lib`.

### Pitfall 7: Tool capability gap misread as OPS-04 fail
**What goes wrong:** Codex path lacks a Grok-only tool; entire OPS-04 marked fail.  
**Why it happens:** PROJECT allows documented provider gaps if daily tools (read/edit/shell) still work.  
**How to avoid:** Document gaps in UAT; require read + edit/shell bar; don't fail solely on Grok-only tools.  
**Warning signs:** Fail notes only mention computer-use / Grok-specific tools.

### Pitfall 8: Secrets in commits
**What goes wrong:** auth.json or bearer-bearing logs land in git.  
**Why it happens:** Operator pastes “evidence.”  
**How to avoid:** Redacted notes only; checklist forbids secret paste; pre-push scan.  
**Warning signs:** `auth.json` in git status; long JWT-like strings in VERIFICATION.

## Code Examples

### Dual-login free switch residual (automated)

```rust
// Source: crates/codegen/xai-grok-shell/tests/model_switch_gate.rs
// Filters: p6_dual_login_free_switch_xai_to_codex_no_missing_provider
//          p6_dual_login_free_switch_codex_to_xai_no_missing_provider
//          p6_dual_login_next_sample_uses_target_provider
// Run: cargo test -p xai-grok-shell --test model_switch_gate p6_dual_login -- --nocapture
```

### Cross-provider isolation residual (automated)

```bash
# Source: 07-PHASE-GATE.md
discover xai-grok-shell p7_isolation --lib
discover xai-grok-shell p7_isolation_grok_parent_codex --lib
discover xai-grok-shell p7_isolation_codex_parent_grok --lib
discover xai-grok-tools p7_eager --lib
discover xai-grok-shell p7_spawn_missing_provider --lib
```

### Live UAT CLI preflight

```bash
# Source: product CLI (Phase 5) + CONTEXT live environment
export BUM_HOME="${BUM_HOME:-$HOME/.bum}"   # or mktemp -d isolation
cargo build -p xai-grok-pager-bin
./target/debug/bum login                    # xAI default
./target/debug/bum login --provider codex
./target/debug/bum auth status              # both usable; no token dump
# Prefer TUI: ./target/debug/bum
# Optional headless single turn:
# ./target/debug/bum -p "read README.md and summarize" -m grok-build --always-approve
```

### Home isolation residual

```bash
# Source: 08-PHASE-GATE.md
cargo test -p xai-grok-pager-bin --test home_isolation -- --nocapture
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Phase 3–5 optional advisory live smoke | Phase 9 **required** live dual-login UAT | Phase 9 | OPS rows cannot green without human sign-off |
| Phase 7 fixture-only AGENT-06 | Live OPS-06 matrix both dirs | Phase 9 | Closes deferred live multi-turn dual-login NL E2E |
| Automated-only phase gates (7–8) | Hybrid auto + human | Phase 9 | Both halves for GREEN |
| Intentional-red Wave 0 (early phases) | Green-only scaffolds (7–8 lesson) | Phase 7+ | Keep green-only for any `p9_` |

**Deprecated/outdated:**
- Treating Phase 3 “environment skip = not fail” for OPS UAT
- Re-deferring live dual-login to a later phase after Phase 9 starts

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Headless single-turn with `-p`/`-m` is acceptable fallback for a given OPS row when TUI is blocked (CONTEXT allows discretion; prefer TUI) | UAT methodology | Planner forces headless-only and misses TUI chrome bugs — mitigate by preferring TUI in checklist |
| A2 | Daily-driver tool bar for OPS-04 is satisfied by **read + edit or shell** even if some Grok-only tools fail on Codex (PROJECT compatibility language) | Pitfalls / OPS-04 | User expects every tool — confirm gaps documented |
| A3 | Recommended models `grok-build` + `gpt-5.6-sol` remain valid daily choices at execute time | Session matrix | Catalog renames — UAT records actual IDs used |
| A4 | Residual clap help still saying `Usage: grok` / `grok "…"` examples is an **in-phase fix candidate** if UAT treats it as daily-driver chrome friction (not mass GROK_* rename) | Pitfall 5 | May be argv0/display only; verify after rebuild before large string sweep |

**If empty table:** N/A — table has assumptions for discuss/execute confirmation.

## Open Questions

1. **How wide is the prior-gate re-run subset?**
   - What we know: CONTEXT leaves which gates to re-invoke to discretion; OPS maps strongly to p6 switch, p7 isolation/spawn, p8 quiet residual, home_isolation.
   - What's unclear: Whether to include full p6 pager dispatch set or full p8 residual greps.
   - **Recommendation (default):** Minimum OPS-linked subset below in Validation Architecture + optional “broad residual” optional block; do not paste entire 06/07/08 gates unless execute has time budget.

2. **How many thin `p9_` tests?**
   - What we know: Must be green-only fixture; must not require live OAuth.
   - What's unclear: Whether discovery-only smoke is enough vs dual-token residual assert.
   - **Recommendation:** 1–3 thin tests max (e.g. dual-slot auth document readable under temp BUM_HOME; login-hint residual still says `bum login --provider`; optional discovery harness). Prefer re-run over invent.

3. **TUI vs headless split for OPS rows?**
   - What we know: Prefer TUI daily path; headless exists with `-m` / `-p` / `--always-approve`.
   - What's unclear: Whether OPS-06 NL is reliable enough vs explicit Task tool.
   - **Recommendation:** OPS-03..05 prefer TUI; OPS-06 accept NL **or** explicit Task spawn with model+effort; record which path used.

4. **Where does live UAT store evidence?**
   - What we know: Checklist in phase dir + VERIFICATION rows; no secrets.
   - What's unclear: Whether to attach redacted log files under phase dir.
   - **Recommendation:** Fill tables in `09-UAT.md` + summary in `09-VERIFICATION.md` only; optional short redacted excerpts inline; no raw logs in git.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain | Automated tests + binary build | ✓ | 1.92.0 | — |
| Cargo | Gates | ✓ | present | — |
| `protoc` | Workspace builds that need proto | ✓ | system | repo `bin/protoc` |
| `target/debug/bum` | Live UAT / isolation tests | ✓ | 0.1.220-alpha.4 | `cargo build -p xai-grok-pager-bin` |
| Network (xAI IdP + API) | Live OPS-03/05/06 | operator-dependent | — | Block human path; re-auth — **no fixture waive** |
| Network (ChatGPT/Codex OAuth + API) | Live OPS-04/05/06 | operator-dependent | — | Block human path; re-auth |
| `~/.bum/auth.json` dual usable | Live dual-login | ✗ at research time (missing) | — | Run `bum login` + `bum login --provider codex` during UAT |
| Browser for OAuth | Login preflight | operator-dependent | — | `--device-auth` where applicable |
| Graphify graph | Cross-doc queries | ✗ | — | Continue without graph (status: absent) |

**Missing dependencies with no fallback:**
- Live dual OAuth accounts + network for human half (blocks OPS green rows only; automated half can still green)

**Missing dependencies with fallback:**
- Pre-existing `~/.bum/auth.json` — create via login during UAT
- Graph — not required for this phase

**Step 2.6 note:** External package managers (npm/pip) not required. No new services to install.

## Validation Architecture

> `workflow.nyquist_validation` is **enabled** in `.planning/config.json` (`nyquist_validation: true`). This section is mandatory.

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Cargo built-in `cargo test` (+ bash `discover()` for gates) |
| Config file | none global — per-crate; `rust-toolchain.toml` 1.92.0 |
| Quick run command | `cargo test -p xai-grok-shell --test model_switch_gate p6_dual_login -- --nocapture` |
| Full suite command | See recommended `09-PHASE-GATE.md` aggregate (prior subset + thin `p9_` + residual) |
| Estimated runtime | ~2–8 min automated after compile; live UAT human-time ~30–90 min |

### Cargo verify hygiene (locked — copy prior phases)

| Rule | Detail |
|------|--------|
| One TESTNAME filter | One positional filter per `cargo test` |
| Multi-test coverage | Chain with `&&` only |
| Discovery assert | Every required subgroup: `discover` → `n >= 1` then execute |
| Unique prefixes | New thin proofs `p9_`; re-runs keep `p6_`/`p7_`/`p8_` names |
| Green-only | No intentional-red / expected-fail under gate filters |
| Forbidden | Unfiltered full workspace; bare `cargo test -p xai-grok-shell --lib` as sole gate; aggregate-only `grep -c p9_` |
| Automated network | Fixture tokens only (`xai-fake-token*` / `codex-fake-token*`) |
| Secrets | Never commit tokens / auth.json / secret transcripts |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|--------------|
| OPS-03 | Productive xAI tool turn after login | **live UAT required** + auto residual | UAT §OPS-03; residual: routing/auth/home | ❌ Wave 0 UAT doc; auto residual ✅ prior |
| OPS-04 | Productive GPT-5.6 tool turn after Codex login | **live UAT required** + auto residual | UAT §OPS-04; residual: dual-slot + `provider_routing` codex | ❌ Wave 0 UAT; auto ✅ prior |
| OPS-05 | Same-session switch productive | **live UAT** + auto | `discover xai-grok-shell p6_dual_login --test model_switch_gate`; `switch_changes_next_sample_route` on `provider_routing` | auto ✅; live ❌ Wave 0 |
| OPS-06 | Cross-provider spawn both dirs live | **live UAT** + auto | `p7_isolation` both dirs + `p7_eager` + `p7_spawn_missing_provider` | auto ✅; live ❌ Wave 0 (was deferred) |
| ALL | Quiet fork still holds | auto residual | `p8_telemetry`, `p8_no_auto_update`, `p8_sentry`, `home_isolation` | ✅ prior |
| ALL | Thin phase discovery smoke | unit/integration | `p9_*` (to add, green-only, fixture) | ❌ Wave 0 |

### Recommended automated re-run inventory (discretion default)

| Priority | Package | Target | Filter | Why |
|----------|---------|--------|--------|-----|
| P0 | `xai-grok-shell` | `--test model_switch_gate` | `p6_dual_login` | OPS-05 residual |
| P0 | `xai-grok-shell` | `--test model_switch_gate` | `p6_missing_provider` | Fail-closed residual |
| P0 | `xai-grok-shell` | `--test provider_routing` | `switch_changes_next_sample_route` | Route after switch |
| P0 | `xai-grok-shell` | `--lib` | `p7_isolation` | OPS-06 Authorization residual |
| P0 | `xai-grok-shell` | `--lib` | `p7_isolation_grok_parent_codex` | D-12 dir 1 |
| P0 | `xai-grok-shell` | `--lib` | `p7_isolation_codex_parent_grok` | D-12 dir 2 |
| P0 | `xai-grok-tools` | `--lib` | `p7_eager` | Spawn preflight residual |
| P0 | `xai-grok-shell` | `--lib` | `p7_spawn_missing_provider` | Fail-closed spawn |
| P0 | `xai-grok-shell` | `--lib` | `p7_parent_model` | Parent model stability |
| P1 | `xai-grok-shell` | `--test cross_provider_subagent` | `p7_spawn_same_provider` | AGENT-01 residual |
| P1 | `xai-grok-shell` | `--lib` | `p8_telemetry` | Quiet fork residual |
| P1 | `xai-grok-pager-bin` | `--bin bum` | `p8_no_auto_update` | No stock overwrite |
| P1 | `xai-grok-pager-bin` | `--bin bum` | `p8_sentry` | No phone-home |
| P1 | `xai-grok-pager-bin` | `--test home_isolation` | `hermetic` / full | Identity residual |
| P2 | `xai-grok-shell` / new | `--lib` or `--test …` | `p9_*` | Thin discovery residual |

### Manual / Human Verifications (REQUIRED for gate GREEN)

| Behavior | Req | Why Manual | Required? | Instructions |
|----------|-----|------------|-----------|--------------|
| Real xAI coding turn (read + edit/shell) | OPS-03 | Live backend + OAuth | **Required** | `09-UAT.md` §OPS-03 |
| Real Codex/GPT-5.6 coding turn | OPS-04 | Live backend + OAuth | **Required** | `09-UAT.md` §OPS-04 |
| Same-session provider switch | OPS-05 | Real multi-turn session | **Required** | `09-UAT.md` §OPS-05 |
| Cross-provider Task both dirs | OPS-06 | Live dual-login spawn | **Required** | `09-UAT.md` §OPS-06 |

**Do not** mark live OPS rows green on fixture-only evidence.  
**Do not** treat environment skip as phase pass.  
If network/account fails → block human path (fix or re-auth).

### Sampling Rate

- **Per task commit:** focused filter for touched crate / docs-only skip with static check
- **Per wave merge:** all automated subgroups planned for that wave green
- **Phase gate:** full automated aggregate **and** signed UAT for OPS-03..06
- **Human verify mode:** `end-of-phase` (config) — but Phase 9 elevates human checks into the gate itself

### Wave 0 Gaps

- [ ] `09-VALIDATION.md` — dual auto/human map; nyquist frontmatter
- [ ] `09-PHASE-GATE.md` — discover+execute aggregate + human sign-off section
- [ ] `09-UAT.md` — required live dual-login checklist (not advisory)
- [ ] Thin `p9_*` green smoke (0–3 tests) — fixture dual residual / discovery
- [ ] Optional `scripts/uat-preflight.sh` — non-secret helper
- [ ] After execute: `09-VERIFICATION.md` with OPS rows + model IDs + operator sign-off

*(Prior phase automated filters exist — Wave 0 is documentation + thin residual + hybrid gate wiring, not a full new harness.)*

## Security Domain

> `security_enforcement` enabled (ASVS level 1). Phase is validation-heavy; auth dual-slot and secret hygiene remain critical.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | yes | Live dual OAuth under `$BUM_HOME/auth.json`; never CI secrets |
| V3 Session Management | yes | Independent per-provider refresh; re-auth on hard expiry during UAT |
| V4 Access Control | yes | Never cross-send bearers (p7 isolation residual); missing provider fail-closed |
| V5 Input Validation | yes | Catalog model IDs for switch/spawn; effort enum on Task |
| V6 Cryptography | no new | Existing token storage; do not hand-roll |

### Known Threat Patterns for daily-driver dual-login

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Token leakage in VERIFICATION / commits | Information Disclosure | Redacted notes only; no auth.json in git |
| Cross-provider bearer on wrong backend | Information Disclosure / Spoofing | Re-run p7 isolation; live spawn both dirs |
| Fixture tokens mistaken for live proof | Spoofing (process integrity) | Hybrid gate both halves |
| Logging Authorization headers in UAT evidence | Information Disclosure | Short redacted excerpts; no raw HTTP dumps |
| Stock auto-update overwriting bum mid-UAT | Tampering | p8 no_auto_update residual |

## Sources

### Primary (HIGH confidence)

- [VERIFIED: planning] `.planning/phases/09-…/09-CONTEXT.md` — locked hybrid methodology
- [VERIFIED: planning] `.planning/REQUIREMENTS.md` OPS-03..06; `.planning/ROADMAP.md` Phase 9 success criteria
- [VERIFIED: planning] `07-PHASE-GATE.md`, `07-VALIDATION.md`, `07-VERIFICATION.md` — green-only discover+execute + deferred live → Phase 9
- [VERIFIED: planning] `08-PHASE-GATE.md`, `08-VALIDATION.md` — residual greps + quiet-fork filters
- [VERIFIED: planning] `06-PHASE-GATE.md` — `p6_dual_login` / model_switch_gate
- [VERIFIED: planning] `09-PATTERNS.md` — hybrid composition + UAT checklist skeleton
- [VERIFIED: codebase] `crates/codegen/xai-grok-models/default_models.json` — `grok-build`, `gpt-5.6-sol`
- [VERIFIED: codebase] `crates/codegen/xai-grok-shell/tests/{model_switch_gate,provider_routing,cross_provider_subagent}.rs`
- [VERIFIED: codebase] `crates/codegen/xai-grok-pager-bin/tests/home_isolation.rs`
- [VERIFIED: codebase] CLI: `bum login --provider`, `bum auth status`; clap `name = "bum"`
- [VERIFIED: local env] rustc 1.92.0; `target/debug/bum`; `~/.bum` exists without auth.json
- [VERIFIED: config] `.planning/config.json` `workflow.nyquist_validation: true`, `security_enforcement: true`

### Secondary (MEDIUM confidence)

- Phase 2 VERIFICATION human OAuth smoke shape (required elevation for Phase 9)
- Phase 3 advisory UAT table inverted to required (pattern contrast)

### Tertiary (LOW confidence)

- Exact live Codex tool capability gaps under GPT-5.6 path — must be documented during UAT (A2)

## Project Constraints (from CLAUDE.md / AGENTS.md)

| Directive | Implication for Phase 9 |
|-----------|-------------------------|
| Stay on Rust workspace fork | No rewrite; validation of existing harness only |
| Dual OAuth (xAI + Codex ChatGPT) | Live UAT must exercise both slots |
| Per-model routing, not global mode | Switch mid-session via model picker |
| `~/.bum` isolation | Prefer `BUM_HOME` temp or real `~/.bum`; never stock homes |
| No xAI auto-update / no telemetry phone-home | Re-run p8 residual during daily-driver gate |
| Provider gaps documented | OPS-04 honest gap notes allowed if daily tools work |
| Product presents as bum | Residual chrome (help examples) fixable in-phase if blocker |
| Prefer per-crate cargo tests | PHASE-GATE uses filtered `-p` commands |
| GSD workflow for edits | Plans via GSD; research already in-flow |

## Metadata

**Confidence breakdown:**
- Standard stack: **HIGH** — pure reuse of verified crates/gates; no new deps
- Architecture: **HIGH** — hybrid gate formula and artifact layout fixed by CONTEXT + prior phases
- Pitfalls: **HIGH** — derived from prior phase lessons + live OAuth risks; chrome residual verified locally

**Research date:** 2026-07-17  
**Valid until:** 2026-08-16 (re-check if catalog model IDs or auth CLI change)

## Recommended Plan Shape (for planner)

Fine granularity (project mode), **~4–6 plans**:

| Wave | Plan focus | Deliverables |
|------|------------|--------------|
| 1 | Harness + maps | `09-VALIDATION.md` (dual auto/human map), thin green `p9_*` smoke (0–3), frontmatter nyquist flags |
| 2 | Phase gate automated half | `09-PHASE-GATE.md` discover+execute prior P0/P1 subset + p9_; run automated green |
| 3 | UAT runbook | `09-UAT.md` checklist (required), optional `scripts/uat-preflight.sh` |
| 4 | Live dual-login execute | Human: dual login → OPS-03..06 matrix (single session OK); fill checklist; fix blockers only |
| 5 | Hybrid close | Re-run automated gate; complete `09-VERIFICATION.md`; sign-off; clear Phase 7 “deferred live” |

**Primary files to create:**
- `.planning/phases/09-daily-driver-end-to-end-validation/09-VALIDATION.md`
- `.planning/phases/09-daily-driver-end-to-end-validation/09-PHASE-GATE.md`
- `.planning/phases/09-daily-driver-end-to-end-validation/09-UAT.md`
- `.planning/phases/09-daily-driver-end-to-end-validation/09-VERIFICATION.md` (after execute)
- Optional: thin tests under `xai-grok-shell/tests/` or lib `p9_*`
- Optional: phase-local `scripts/uat-preflight.sh`

**Primary files to change only on UAT blockers:** same seams as Phases 5–8 (auth gate copy, routing, residual chrome, tool path) — not greenfield.

**Blockers for planning:** none.  
**Blockers for phase GREEN:** live dual OAuth + human operator time (Cristian); automated half unblocked now.
