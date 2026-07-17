# Phase 9: Daily-driver end-to-end validation - Pattern Map

**Mapped:** 2026-07-17
**Files analyzed:** 9
**Analogs found:** 9 / 9

> **Validation-first phase.** Primary deliverables are planning/evidence artifacts
> (VALIDATION, PHASE-GATE, UAT checklist, VERIFICATION) plus thin fixture-only
> `p9_` smoke and **re-invocation** of prior phase gates. Product code only when
> live UAT finds OPS-03..06 blockers. Prefer **reusing Phase 7/8 gate structure**
> over inventing a new verification product (CONTEXT locked).

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `.planning/.../09-VALIDATION.md` | config / validation map | batch (req → proof map) | `08-VALIDATION.md` (+ `07-VALIDATION.md` hybrid rows) | exact |
| `.planning/.../09-PHASE-GATE.md` | config / runbook | batch (discover+execute) | `08-PHASE-GATE.md` (+ `07-PHASE-GATE.md` prior-gate re-run) | exact |
| `.planning/.../09-UAT.md` (or `09-UAT-CHECKLIST.md`) | config / human checklist | request-response (live session) | Phase 3 “Manual / Human Verifications” **elevated to required** + Phase 2 “Human Verification Required” live OAuth | role-match (required, not advisory) |
| `.planning/.../09-VERIFICATION.md` | config / evidence report | batch | `07-VERIFICATION.md` / `08-VERIFICATION.md` (truth tables + deferred live → now fill OPS rows) | exact |
| Thin `p9_*` unit/integration smoke (shell/pager/tools as needed) | test | request-response / batch | `p7_*` in `cross_provider_subagent.rs` + `p8_*` green-only scaffolds | role-match |
| Prior-gate re-run slices (p6/p7/p8 critical subgroups) | test (invoke, not invent) | batch | `06-PHASE-GATE.md`, `07-PHASE-GATE.md`, `08-PHASE-GATE.md` copy-paste sequences | exact |
| Dual-fixture residual smoke (fixture tokens only) | test | request-response | `shell/tests/cross_provider_subagent.rs` + `model_switch_gate.rs` dual-slot fixtures | exact |
| Live dual-login run notes / redacted log excerpts | config / evidence | file-I/O (human) | `02-VERIFICATION.md` Human Verification Required (live OAuth smoke) | role-match |
| Optional small product fix (blocker only) | service / component | request-response | Same file as Phase 5–7 fail-closed / login-hint patterns | partial (only if UAT finds blocker) |

## Pattern Assignments

### `09-VALIDATION.md` (validation map, batch)

**Analog (primary):** `.planning/phases/08-quiet-fork-rebrand-polish/08-VALIDATION.md`  
**Analog (hybrid / dual-provider):** `.planning/phases/07-cross-provider-multi-agent-orchestration/07-VALIDATION.md`  
**Human rows analog:** `.planning/phases/03-model-catalog-gpt-5-6-entries/03-VALIDATION.md` “Manual / Human Verifications” — **but Phase 9 must flip “optional advisory” → required for OPS-03..06**

**Frontmatter pattern** (copy shape from 08; adapt reqs):
```yaml
---
phase: 9
slug: daily-driver-end-to-end-validation
status: draft   # → green after gate + signed UAT
nyquist_compliant: false  # → true after wave 0 / map complete
wave_0_complete: false
created: 2026-07-17
updated: 2026-07-17
plans_verified: []
gate: 09-PHASE-GATE.md
---
```

**Authority blurb pattern** (from 08/07 — adapt for hybrid bar):
```markdown
> Per-phase validation contract for feedback sampling during execution and final gate.
> Prove **OPS-03..OPS-06** with a **hybrid bar**:
> - **Automated:** prior phase critical gates (`p6_` / `p7_` / `p8_`) + thin `p9_` smoke —
>   **fixture tokens only**; no live OAuth in CI.
> - **Human UAT:** signed dual-login checklist under real xAI + Codex OAuth (`~/.bum` or
>   isolated `BUM_HOME`) — **required for gate GREEN**, not advisory.
>
> **Green-only protocol:** no intentional-red under `p9_`. Phase gate claims **all required
> automated subgroups green AND signed human checklist** — never automated-only or
> human-only.
```

**Test Infrastructure + Cargo verify hygiene** — copy tables from `08-VALIDATION.md` lines 43–66 / `07-VALIDATION.md` lines 53–75:

| Rule | Detail (locked) |
|------|-----------------|
| One TESTNAME filter | One positional filter per `cargo test` |
| Chains | `&&` only — never `;` or `\|\| true` |
| Discovery assert | Every required subgroup: `discover()` → `n >= 1` then execute |
| Unique prefixes | New thin proofs use `p9_`; re-run prior filters keep `p6_`/`p7_`/`p8_` |
| Green-only | No intentional-red / `#[ignore]` expected-red under gate-discovered filters |
| Forbidden | Unfiltered full-workspace or bare `cargo test -p xai-grok-shell --lib`; aggregate-only `grep -c p9_` |
| Fixtures only (automated) | `xai-fake-token*` / `codex-fake-token*`; **live OAuth never required for automated half** |
| Secrets | Never commit tokens, `auth.json`, or full raw transcripts with secrets |

**Discovery helper** (canonical — identical to 07/08):
```bash
set -euo pipefail
discover() {
  local pkg="$1" filter="$2"; shift 2
  local extra=("$@")
  local n
  n=$(cargo test -p "$pkg" "${extra[@]}" -- --list 2>/dev/null | grep -c "$filter" || true)
  test "$n" -ge 1
  cargo test -p "$pkg" "${extra[@]}" "$filter" -- --nocapture
}
```

**Requirements → Test Map pattern** (extend 07/08 table; add dual columns for auto vs human):

| Req ID | Behavior | Automated proof | Live UAT proof | Exists? |
|--------|----------|-----------------|----------------|---------|
| OPS-03 | Productive xAI tool turn after login | Prior routing/auth + thin `p9_` residual | UAT: login xAI → read + edit/shell on real backend | ⬜ |
| OPS-04 | Productive Codex/GPT-5.6 tool turn after login | Prior dual-slot + routing | UAT: login Codex → read + edit/shell (supported tools) | ⬜ |
| OPS-05 | Mid-session switch both ways without restart | Re-run `p6_dual_login*` / switch gate | UAT: xAI turn → `/model` GPT → Codex turn (optional reverse) | ⬜ |
| OPS-06 | Cross-provider spawn both dirs | Re-run `p7_isolation_*` both dirs + spawn | UAT: Grok→Codex child **and** Codex→Grok child (NL or Task + effort) | ⬜ |
| ALL | Quiet fork / chrome / isolation residual | Re-run critical `p8_` + `home_isolation` | Optional spot-check chrome during UAT | ⬜ |

**Human table pattern** — invert Phase 3 advisory (03-VALIDATION.md lines 164–177):

```markdown
## Manual / Human Verifications (REQUIRED for gate GREEN)

| Behavior | Req | Why Manual | Required? | Instructions |
|----------|-----|------------|-----------|--------------|
| Real xAI coding turn (read + edit/shell) | OPS-03 | Live backend + real OAuth | **Required** | See 09-UAT.md §OPS-03 |
| Real Codex coding turn | OPS-04 | Live backend + real OAuth | **Required** | See 09-UAT.md §OPS-04 |
| Same-session provider switch | OPS-05 | Real multi-turn session | **Required** | See 09-UAT.md §OPS-05 |
| Cross-provider Task both dirs | OPS-06 | Live dual-login spawn | **Required** | See 09-UAT.md §OPS-06 |

**Do not** mark live OPS rows green on fixture-only evidence.
**Do not** treat environment skip as phase pass (unlike Phase 3 advisory UAT).
If network/account fails → block human path (fix or re-auth); do not waive.
```

**Explicit exclusions** (from CONTEXT deferred + 07/08 pattern):
- Public signed x.ai install channel
- Crate rename / mass `GROK_*` rename
- Full model-catalog matrix / multi-hour soak
- Live OAuth secrets in CI
- Intentional-red under `p9_`
- Product feature work unless UAT finds OPS blockers

**Anti-patterns:**
- Copying Phase 3 “optional advisory UAT” language for OPS rows
- Claiming hybrid GREEN with only automated half
- Full workspace test as gate
- Requiring live network for automated `p9_` smoke

---

### `09-PHASE-GATE.md` (runbook, batch)

**Analog (primary):** `.planning/phases/08-quiet-fork-rebrand-polish/08-PHASE-GATE.md`  
**Analog (prior-gate composition):** `.planning/phases/07-cross-provider-multi-agent-orchestration/07-PHASE-GATE.md`  
**Analog (switch half):** `.planning/phases/06-mid-session-switch-missing-provider-gate/06-PHASE-GATE.md`

**Frontmatter pattern:**
```yaml
---
phase: 9
slug: daily-driver-end-to-end-validation
status: draft
requirements: [OPS-03, OPS-04, OPS-05, OPS-06]
fixture_only: true          # automated half
green_only: true
human_uat_required: true    # NEW vs 07/08
---
```

**Header bullets** (compose 07+08 + hybrid):
```markdown
- **Automated half:** fixtures only — re-run critical prior subgroups + thin `p9_` smoke.
- **Human half:** signed 09-UAT checklist — real dual OAuth; not CI; not fixture-green.
- **Green-only** — no intentional-red under `p9_`.
- **Per-subgroup discovery ≥1** before each execute.
- **Both halves required** for gate GREEN (like Phase 6 “both halves” shell+pager).
- **Forbidden:** unfiltered shell `--lib`; live multi-turn as *automated* gate;
  marking OPS live rows from fixtures; committing secrets.
```

**Copy-paste sequence structure** (from 08-PHASE-GATE.md lines 76–176):

```bash
set -euo pipefail

discover() {
  local pkg="$1" filter="$2"; shift 2
  local extra=("$@")
  local n
  n=$(cargo test -p "$pkg" "${extra[@]}" -- --list 2>/dev/null | grep -c "$filter" || true)
  echo "discover $pkg $filter (${extra[*]:-}): n=$n"
  test "$n" -ge 1
  cargo test -p "$pkg" "${extra[@]}" "$filter" -- --nocapture
}

# ========== Phase 9 thin smoke (fixture residual) ==========
# discover xai-grok-shell p9_ --lib   # or --test … once scaffolds land
# discover … p9_… as planned

# ========== Prior critical: Phase 6 switch / dual-login ==========
discover xai-grok-shell p6_dual_login --test model_switch_gate
discover xai-grok-shell p6_missing_provider --test model_switch_gate
# (subset only — do not paste entire 06 gate unless planner expands)

# ========== Prior critical: Phase 7 cross-provider ==========
discover xai-grok-tools p7_eager --lib
discover xai-grok-shell p7_isolation --lib
discover xai-grok-shell p7_isolation_grok_parent_codex --lib
discover xai-grok-shell p7_isolation_codex_parent_grok --lib
discover xai-grok-shell p7_spawn_missing_provider --lib
# integration dual-doc optional:
# discover xai-grok-shell p7_spawn_missing_provider --test cross_provider_subagent

# ========== Prior critical: Phase 8 quiet fork residual ==========
discover xai-grok-shell p8_telemetry --lib
discover xai-grok-pager-bin p8_no_auto_update --bin bum
discover xai-grok-pager-bin p8_sentry --bin bum
# home isolation
n=$(cargo test -p xai-grok-pager-bin --test home_isolation -- --list 2>/dev/null | grep -c 'hermetic\|home' || true)
test "$n" -ge 1
cargo test -p xai-grok-pager-bin --test home_isolation -- --nocapture

# ========== Human gate (docs assert, not cargo) ==========
# Require 09-UAT.md signed rows for OPS-03..06 all pass (who/when/models)
# Fail if UAT missing or any OPS live row is fixture-only

echo "PHASE 9 AUTOMATED HALF: GREEN — human UAT still required for full gate"
```

**Checklist pattern** (08-PHASE-GATE.md lines 180–200 + human rows):

| # | Check | Status |
|---|-------|--------|
| 1…N | Automated discover+execute subgroups | ⬜ |
| N+1 | `home_isolation` green | ⬜ |
| N+2 | Dual-direction `p7_isolation_*` names present + green | ⬜ |
| N+3 | 09-UAT.md OPS-03 signed pass | ⬜ **human** |
| N+4 | 09-UAT.md OPS-04 signed pass | ⬜ **human** |
| N+5 | 09-UAT.md OPS-05 signed pass | ⬜ **human** |
| N+6 | 09-UAT.md OPS-06 both dirs signed pass | ⬜ **human** |
| N+7 | No secrets committed; redacted notes only | ⬜ |

**Target selection notes** (from 06/07/08 lessons):
| Case | Why |
|------|-----|
| Prefer `--lib` for shell unit when bare list is poisoned | Phase 6 lesson |
| Integration spawn dual-doc on `--test cross_provider_subagent` | Phase 7 |
| Pager-bin composition tests on `--bin bum` | Phase 8 |
| Isolation `home_isolation` on `--test home_isolation` | Phase 1/8 |

**Anti-patterns:**
- Gate GREEN with checklist “deferred” or “environment skip”
- Running only `p9_` and skipping prior critical subgroups
- Piping cargo to mask failures
- Putting live OAuth tokens in the shell script

---

### `09-UAT.md` / UAT checklist (human, request-response)

**Closest analogs:**
1. **Required live OAuth smoke:** `.planning/phases/02-multi-slot-credentials-xai-oauth/02-VERIFICATION.md` “Human Verification Required” (live xAI OAuth under isolated home)
2. **Checklist layout (elevate):** `.planning/phases/03-model-catalog-gpt-5-6-entries/03-VALIDATION.md` Manual table + `03-03-PLAN.md` Task 4 structure — **but gate=required, not optional**
3. **Session matrix content:** `09-CONTEXT.md` OPS-03..06 minimums
4. **Dual-login automated fixture shape (for contrast):** `crates/codegen/xai-grok-shell/tests/model_switch_gate.rs` `p6_dual_login_*` — fixtures prove switch *logic*; UAT proves *live* product path

**Structure to copy (from 02-VERIFICATION Human section, adapted):**

```markdown
# Phase 9 — Live Dual-Login UAT Checklist

**Operator:** Cristian (default per CONTEXT)
**Environment:** real dual OAuth under `~/.bum` **or** isolated temp `BUM_HOME`
**Secrets policy:** never paste tokens / full auth.json; redacted notes only
**Gate rule:** all OPS-03..06 rows must PASS with live evidence — fixture green does not count

## Preflight
- [ ] Built `bum` binary available (`cargo build -p xai-grok-pager-bin` or installed)
- [ ] Network + xAI account + ChatGPT/Codex account usable
- [ ] Product home isolated if desired: `export BUM_HOME=/tmp/bum-uat-$$`
- [ ] `bum login` (xAI) and `bum login --provider codex` both succeed
- [ ] `bum auth status` (or equivalent) shows both slots usable — **no token dump in notes**

## OPS-03 — xAI productive session
| Step | Action | Expected | Pass? | Notes (model id, when) |
|------|--------|----------|-------|------------------------|
| 1 | Start TUI (prefer daily path) on current xAI model | Session starts as bum chrome | ⬜ | |
| 2 | Ask agent to **read** a real file in workspace | Tool succeeds on real backend | ⬜ | |
| 3 | **Edit** or **shell** one productive change | Succeeds (not fixture mock) | ⬜ | |

## OPS-04 — Codex / GPT-5.6 productive session
| Step | Action | Expected | Pass? | Notes |
|------|--------|----------|-------|-------|
| 1 | Switch or start on GPT-5.6 catalog entry (e.g. Sol) | Model/provider Codex path | ⬜ | |
| 2 | Read + edit/shell as supported | Daily-driver tools work; document capability gaps honestly | ⬜ | |

## OPS-05 — Mid-session switch (same CLI process)
| Step | Action | Expected | Pass? | Notes |
|------|--------|----------|-------|-------|
| 1 | Productive xAI turn | Completes | ⬜ | |
| 2 | `/model` → GPT-5.6 without restarting CLI | Switch accepted; no false missing-provider if both logged in | ⬜ | |
| 3 | Productive Codex turn | Completes | ⬜ | |
| 4 | (Optional) reverse switch | Works | ⬜ | |

## OPS-06 — Cross-provider spawn both directions
| Step | Action | Expected | Pass? | Notes |
|------|--------|----------|-------|-------|
| 1 | Parent Grok → child Codex (NL or Task + model + effort) | Child completes; results return; parent model unchanged | ⬜ | |
| 2 | Parent Codex → child Grok | Same | ⬜ | |

## Sign-off
| Field | Value |
|-------|-------|
| Operator | |
| Date (UTC) | |
| Binary version / commit | |
| Models under test | xAI: … / Codex: … |
| Capability gaps documented | |
| Secrets committed? | **No** |
```

**Phase 2 live OAuth instruction shape** (02-VERIFICATION.md) — reuse for preflight wording:
```text
With a clean temp product home, run:
  BUM_HOME=/tmp/bum-uat-smoke bum login
  BUM_HOME=/tmp/bum-uat-smoke bum login --provider codex
Inspect $BUM_HOME/auth.json structure only (version + providers keys) — do not commit file.
```

**Prefer TUI daily path** when practical (CONTEXT discretion); headless acceptable for a given OPS row if TUI blocked, but prefer TUI for OPS-03..05.

**Single-session multi-OPS** allowed (CONTEXT): one dual-login session may cover switch + both providers + spawn when natural — still fill each OPS row.

**Anti-patterns:**
- Phase 3 “skip with environment note = not a fail” for OPS rows
- Marking pass without model IDs / who / when
- Committing transcripts with Authorization headers or tokens
- Running live UAT *as* CI job with secrets
- Waiving OPS-06 one direction

---

### `09-VERIFICATION.md` (evidence report, batch)

**Analog:** `.planning/phases/07-cross-provider-multi-agent-orchestration/07-VERIFICATION.md`  
**Analog:** `.planning/phases/08-quiet-fork-rebrand-polish/08-VERIFICATION.md`

**Frontmatter / score pattern** (07):
```yaml
---
phase: 09-daily-driver-end-to-end-validation
status: pending
score: 0/4 must-haves verified   # OPS-03..06
behavior_unverified: 0
overrides_applied: 0
---
```

**Observable Truths table** — fill live + automated evidence (07 pattern lines 27–36):

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | OPS-03 real xAI coding session | ⬜ | UAT notes + optional auto residual |
| 2 | OPS-04 real Codex coding session | ⬜ | UAT notes; capability gaps if any |
| 3 | OPS-05 same-session switch productive | ⬜ | UAT + re-run `p6_dual_login` |
| 4 | OPS-06 both spawn directions live | ⬜ | UAT both dirs; auto `p7_isolation` residual |

**Deferred items section** — Phase 7 deferred live NL E2E here; after Phase 9, that deferred row should be **cleared** (addressed), not re-deferred.

**Behavioral Spot-Checks** — re-run discover+execute (not SUMMARY claims only) like 07/08.

**Human Verification Required** — for Phase 9 this is **mandatory**, opposite of 08-VERIFICATION “None required for phase gate.”

**Anti-patterns:**
- Leaving OPS rows “deferred Phase 9” after this phase
- Score green with only automated proofs
- Probe scripts invented when cargo filters + UAT suffice (07/08 SKIP probes)

---

### Thin `p9_*` smoke tests (test, batch / request-response)

**Analog (fixture dual-slot + naming):**  
`crates/codegen/xai-grok-shell/tests/cross_provider_subagent.rs` (lines 1–113, 130–180)  
**Analog (green-only thin proofs):** Phase 8 `p8_*` unit tests (e.g. `p8_telemetry_*` in shell `agent/config.rs`)  
**Analog (dual-login switch residual):**  
`crates/codegen/xai-grok-shell/tests/model_switch_gate.rs` — `p6_dual_login_*`  
**Analog (home isolation):**  
`crates/codegen/xai-grok-pager-bin/tests/home_isolation.rs`

**Naming convention** (locked across phases):
```text
p{phase}_{topic}_{behavior_phrase}
```
Examples to invent for Phase 9 (planner discretion):
- `p9_discovery_smoke_*` — harness compiles / dual-slot residual
- `p9_fixture_dual_token_auth_document_*` — both providers readable under temp BUM_HOME
- Prefer **thin residual** over re-implementing full `p7_isolation` Authorization harness

**Fixture constants pattern** (`cross_provider_subagent.rs` lines 47–51):
```rust
const XAI_FAKE: &str = "xai-fake-token-p9";
const CODEX_FAKE: &str = "codex-fake-token-p9";
const CODEX_MODEL: &str = "gpt-5.6-sol";
const XAI_MODEL: &str = "grok-build";
```

**BUM_HOME OnceLock hygiene** (same file lines 25–28, 75–90) — **must copy**:
```rust
/// Process-wide product home for this integration binary (OnceLock-safe).
fn ensure_sandbox() -> &'static Path {
    static SANDBOX: OnceLock<TempDir> = OnceLock::new();
    let dir = SANDBOX.get_or_init(|| {
        let d = TempDir::new().expect("tempdir");
        // SAFETY: this test binary owns BUM_HOME; set once before any grok_home().
        unsafe {
            std::env::set_var("BUM_HOME", d.path());
            std::env::set_var("GROK_TELEMETRY_ENABLED", "false");
            // …
        }
        d
    });
    dir.path()
}
// Do not flip BUM_HOME to a second path mid-process.
```

**auth.json dual-slot write pattern** (lines 92–113):
```rust
fn write_auth_document(home: &Path, xai: Option<GrokAuth>, codex: Option<GrokAuth>) {
    // version: 1, providers.xai / providers.codex AuthStore maps
}
```

**Green assertion shape** (login-hint / fail-closed residual — lines 131–142):
```rust
#[test]
fn p7_missing_provider_gate_error_suggests_bum_login_for_empty_codex() {
    let err = missing_provider_gate_error(/* empty slot */)
        .expect("empty Codex slot must fail closed");
    assert_eq!(err.suggestion, "bum login --provider codex");
}
// p9 thin smoke: reassert residual or discovery-only compile smoke — do not invent live HTTP.
```

**home_isolation spawn pattern** (`home_isolation.rs` lines 81–100):
```rust
#[test]
fn hermetic_temp_home_writes_only_under_bum_home() {
    let tmp = TempDir::new().expect("tempdir");
    // env_clear + BUM_HOME + trap GROK/CODEX homes
    // spawn CARGO_BIN_EXE_bum "version"
    // assert writes only under BUM_HOME
}
```

**Where to put tests:**
| Kind | Location |
|------|----------|
| Shell dual-slot residual | Prefer extend `tests/cross_provider_subagent.rs` or small `tests/p9_daily_driver_smoke.rs` |
| Pager-bin isolation re-use | Existing `--test home_isolation` — re-run, don't fork unless needed |
| Unit pure residual | `#[cfg(test)]` next to production like `p8_*` |

**Anti-patterns:**
- Intentional-red `p9_` scaffolds
- Live network in `p9_` tests
- Duplicating entire Phase 5–7 suites under new names
- Multi-`BUM_HOME` mutation in one process
- Unfiltered `--lib` “to be safe”

---

### Prior-gate re-run (invoke existing, batch)

**Do not rewrite** full 06/07/08 gates into new product code.  
**Compose subsets** into `09-PHASE-GATE.md` (exact analog = those runbooks).

**Minimum re-invocation candidates** (CONTEXT: “which prior gates” is discretion — these are strongest OPS-linked):

| Prior | Subgroup / filter | Maps to OPS |
|-------|-------------------|-------------|
| 6 | `p6_dual_login`, `p6_missing_provider` (`--test model_switch_gate`) | OPS-05 auto residual |
| 7 | `p7_isolation` both dirs, `p7_eager`, `p7_spawn_missing_provider`, `p7_parent_model` | OPS-06 auto residual |
| 8 | `p8_telemetry`, `p8_no_auto_update`, `p8_sentry`, residual greps optional | Quiet fork still holds during daily driver |
| 1/8 | `home_isolation` | Identity / no stock home writes |

Copy discover+execute verbatim from source PHASE-GATE files; adjust only which subgroups are **required** for Phase 9.

---

### Live dual-login scripts (optional helper, request-response)

**No dedicated live OAuth automation script exists as a product gate** (by design across phases). Closest patterns:

| Need | Analog | Notes |
|------|--------|-------|
| Isolated home + binary spawn | `home_isolation.rs` Command + env | For **fixture/hermetic** only |
| Headless agent with mock inference | `xai-grok-test-support` `run_headless` + `MockInferenceServer` | CI-safe; **not** live OPS proof |
| Built binary e2e | `shell/tests/test_built_binary_e2e.rs` | Often `#[ignore]`; not live dual-login |
| Human login steps | `02-VERIFICATION.md` Human Verification Required | Preferred for real OAuth |

**If planner adds a helper script** (e.g. `.planning/phases/09-…/scripts/uat-preflight.sh`):
- Export `BUM_HOME`, print checklist steps, **never** store tokens
- Do not auto-mark UAT pass
- Do not curl live model APIs with committed secrets

**Anti-patterns:**
- Encoding ChatGPT/xAI refresh tokens in repo scripts
- Treating mock `run_headless` success as OPS-03/04 pass
- CI workflow that expects interactive browser OAuth

---

### Product code only for UAT blockers (partial / on-demand)

**When UAT finds a blocker**, fix with existing phase patterns — do not start greenfield:

| Symptom | Copy from |
|---------|-----------|
| Missing-provider message / login hint | Phase 6/7 `missing_provider_gate_error`, `bum login --provider …` |
| Wrong bearer / cross-slot | Phase 7 `p7_isolation_*` Authorization isolation |
| Mid-session switch friction | Phase 6 `model_switch::apply` + pager QuestionView |
| Product chrome still says grok | Phase 8 residual CLI / UI-SPEC |
| Telemetry/update phone-home | Phase 8 hard-off defaults |

**Anti-patterns:** full catalog matrix work, crate renames, install channel, “while we're here” features.

## Shared Patterns

### Hybrid gate (Phase 9 unique composition)

**Sources:** Phase 6 “both halves required” (shell + pager) + Phase 7 “live deferred” + Phase 8 green-only automated  
**Apply to:** `09-VALIDATION.md`, `09-PHASE-GATE.md`, `09-VERIFICATION.md`

```text
Gate GREEN ⇔ (all required automated subgroups discover≥1 AND execute green)
           ∧ (09-UAT signed PASS for OPS-03 ∧ OPS-04 ∧ OPS-05 ∧ OPS-06)
           ∧ (no secrets committed)
```

### Green-only + discover+execute

**Source:** `07-PHASE-GATE.md` / `08-PHASE-GATE.md` `discover()` helper  
**Apply to:** All automated Phase 9 proof commands

### Fixture dual-slot auth

**Source:** `cross_provider_subagent.rs` + `model_switch_gate.rs`  
**Apply to:** Any `p9_` automated residual  
**Tokens:** `xai-fake-token*` / `codex-fake-token*` only in CI

### Fail closed / never cross-send

**Source:** Phase 5–7 auth + spawn gates  
**Apply to:** Live UAT expectations and any blocker fixes  
Messages: `bum login --provider {xai|codex}`

### Secrets / evidence hygiene

**Source:** CONTEXT + 07/08 fixture-only language  
**Apply to:** UAT notes, VERIFICATION, any log excerpts  
Never commit: tokens, auth.json, full Authorization headers

### Per-crate targeted cargo

**Source:** TESTING.md + all phase gates  
**Apply to:** Phase 9 automated half — no full workspace test

## No Analog Found

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| — | — | — | **None.** Hybrid bar composes existing 06–08 gates + elevates Phase 2/3 human verification to required OPS proof. No greenfield verification product. |

> Closest “gap” is that prior phases treated live dual-login as **deferred or advisory**; Phase 9 is the first phase where **live dual OAuth is gate-blocking**. Pattern is elevation of existing human-verification prose, not invention.

## Anti-Patterns Summary (planner must avoid)

| Anti-pattern | Why |
|--------------|-----|
| Phase 3 optional UAT waiver for OPS rows | CONTEXT: live OAuth required for green OPS rows |
| Fixture-only green for OPS-03..06 live claims | Hybrid bar; Phase 7 deferred live here for a reason |
| Intentional-red `p9_` | Green-only protocol (07/08) |
| Unfiltered `cargo test -p xai-grok-shell --lib` | Poisoned / too heavy (06/07 lesson) |
| Aggregate `grep -c p9_` as sole gate | Vacuous discovery |
| Full model catalog matrix / soak | CONTEXT deferred |
| Committing secrets or raw dual-login transcripts | Privacy / security |
| Public install channel / crate rename as Phase 9 work | PROJECT outs |
| Replacing prior gates with only new thin smoke | Regression bar needs re-run of p6/p7/p8 criticals |
| Marking one OPS-06 direction sufficient | CONTEXT: both live directions |

## Metadata

**Analog search scope:**
- `.planning/phases/06-*`, `07-*`, `08-*`, `03-*`, `02-*` (VALIDATION, PHASE-GATE, VERIFICATION, PATTERNS)
- `.planning/codebase/TESTING.md`, `STRUCTURE.md`
- `crates/codegen/xai-grok-shell/tests/{cross_provider_subagent,model_switch_gate}.rs`
- `crates/codegen/xai-grok-pager-bin/tests/home_isolation.rs`
- `rg` for `p7_`, `p8_`, `UAT`, `home_isolation`, dual-provider fixtures

**Files scanned:** ~40 planning + ~15 code/test surfaces  
**Pattern extraction date:** 2026-07-17  
**Phase intent alignment:** Hybrid automated regression + live dual-login UAT for OPS-03..OPS-06
