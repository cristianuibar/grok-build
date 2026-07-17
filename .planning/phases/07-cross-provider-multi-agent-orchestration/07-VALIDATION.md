---
phase: 7
slug: cross-provider-multi-agent-orchestration
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-07-17
updated: 2026-07-17
plans_verified: []
---

# Phase 7 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution and final gate.
> Prove **AGENT-01..06** (same-provider regression, cross-provider spawn, effort, isolation,
> fail-closed missing provider + background preflight, NL Task schema surface) with
> **fixture tokens only** — no live ChatGPT / xAI OAuth required for CI gates.
>
> **Authority:** shell `handle_subagent_request` is the authoritative spawn missing-provider
> gate (Plan 03). Task eager `TaskProviderCredentialGate` (Plan 04) closes the background
> "started" UX hole. Dual-direction Authorization isolation (Plan 05) is the D-12 bar.
>
> **Scaffold note:** This file is created at plan time (Nyquist Dimension 8e) from RESEARCH
> Validation Architecture + planned `p7_` filters. Plan 06 **updates** filter names after
> greening (does not first-create). Mark `wave_0_complete` / `nyquist_compliant` and refresh
> Exists? column when Plan 01+ contracts land and Plan 06 re-verifies.
>
> **Cargo verify hygiene:** one TESTNAME filter per `cargo test`; chain with `&&` only —
> never `||` that masks a failed cargo. Prefer `-p xai-grok-tools` / `-p xai-grok-shell`
> with narrow filters. **Forbidden gate:** unfiltered `cargo test -p xai-grok-shell --lib`.
> Discovery assert ≥1 match before each filtered execute group.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Cargo built-in `cargo test` (tools unit + shell unit/lib + `cross_provider_subagent` integration) |
| **Config file** | none global — per-crate; `rust-toolchain.toml` (1.92.0) |
| **Quick run command** | `cargo test -p xai-grok-shell --test cross_provider_subagent p7_ -- --nocapture` |
| **Full suite command** | See Phase gate aggregate / `07-PHASE-GATE.md` (Plan 06) |
| **Estimated runtime** | ~60–180 seconds after first compile (tools + shell subagent) |

### Cargo verify hygiene (locked)

| Rule | Detail |
|------|--------|
| One TESTNAME filter | Cargo accepts **one** positional TESTNAME filter per invocation |
| Multi-test coverage | Chain single-filter invocations with `&&` |
| Exit status | Prefer **no pipe** on cargo execute path |
| Chains | Use `&&` only — never `;` or trailing `\|\| true` that masks failures |
| Discovery assert | **Every** required subgroup: `discover()` → `test "$n" -ge 1` then execute |
| Unique prefixes | Phase 7 proofs use `p7_` (plus retained AGENT-01 names: `reasoning_effort_explicit`, `resume_model_pinning`, role/persona) |
| Forbidden gates | Unfiltered `cargo test -p xai-grok-shell --lib`; aggregate-only `grep -c p7_` as sole gate |
| Fixtures only | `xai-fake-token*` / `codex-fake-token*` + tempfile auth; no live OAuth secrets |

### Discovery assert helper (canonical — shared with PHASE-GATE)

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

> **Per-subgroup rule:** every required filter below must pass its own `discover`.
> Aggregate `grep -c p7_` per crate is **not** a substitute for a missing subgroup.

### Harness policy

| Allowed | Forbidden |
|---------|-----------|
| `cargo test -p xai-grok-tools --lib p7_<filter>` | Live ChatGPT / xAI login as required gate |
| `cargo test -p xai-grok-shell --test cross_provider_subagent p7_<filter>` | Treating resolve-only key_prefix as sole D-12 proof |
| `cargo test -p xai-grok-shell --lib p7_<filter>` / named AGENT-01 filters | Unfiltered shell `--lib` as phase gate |
| Mock HTTP Authorization asserts (Plan 05) | Parent bearer fallback on missing child slot |
| Dual-slot fixture tokens | Real secrets in tree or CI |

---

## Locked decisions coverage (D-01..D-16)

| ID | Decision | Verified by filters / docs |
|----|----------|----------------------------|
| D-01 | Omit `Task.model` → inherit parent | tools omit-effort/model inherit + shell inherit path |
| D-02 | Explicit model catalog-wide | `p7_tool_unknown_model` + spawn resolve |
| D-03 | Expose + wire `reasoning_effort` | tools `p7_task_tool_input_schema` / `p7_reasoning_effort` |
| D-04 | Invalid effort reject fail-closed | tools `p7_invalid_reasoning_effort` + shell `p7_invalid_effort` |
| D-05 | Spawn-time usable credential check | shell `p7_spawn_missing_provider` + tools `p7_eager` |
| D-06 | No parent bearer/backend fallback | `p7_tool_unknown_model` + isolation / missing |
| D-07 | Login CLI hint | `bum login --provider` in spawn/eager/missing messages |
| D-08 | Same-provider no extra friction | `p7_spawn_same_provider` + `p7_same_provider` |
| D-09 | Child SamplingConfig from child model | `p7_isolation` route/base_url |
| D-10 | Child bearer only | `p7_isolation` Authorization / never_cross_slot |
| D-11 | Parent model unchanged | `p7_parent_model` |
| D-12 | Dual fake-token Authorization + base_url both dirs | `p7_isolation` **mock HTTP Authorization required** (not resolve-only) |
| D-13 | NL via Task schema/docs | schema + agent description filters |
| D-14 | AGENT-01 hard gate | resume / effort / roles / personas + `p7_same_provider` |
| D-15 | `resume_from` model pin | `resume_model_pinning` |
| D-16 | Out of scope | Explicit exclusions below |

---

## Phase Requirements → Test Map

Filters below are **planned** names from Plans 01–06. Plan 01 lands RED discovery;
Plans 02–05 green product filters; Plan 06 refreshes Exists? and greened names from SUMMARYs.

| Req ID | Behavior | Plan | Criterion → crate → filter | Expected | Exists? |
|--------|----------|------|----------------------------|----------|---------|
| AGENT-01 | Same-provider effort override | 01/03/06 | shell · `--lib` · `reasoning_effort_explicit` | pass | ✅ existing |
| AGENT-01 | Resume model pin | 03/06 | shell · `--lib` · `resume_model_pinning` | pass | ✅ existing |
| AGENT-01 | Roles / personas regression | 06 | shell · `--lib` · `role_default_used` + `persona_resolved` (+ siblings) | pass | ✅ existing |
| AGENT-01 | Same-provider spawn no extra friction | 01/03/06 | shell · `cross_provider_subagent` · `p7_same_provider` (and/or `p7_spawn_same_provider`) | pass | ❌ Plan 01 |
| AGENT-02 | Cross-provider explicit model resolve | 01/03/05 | shell · `cross_provider_subagent` · `p7_isolation` / resolve path | pass | ❌ Plan 01/05 |
| AGENT-02 | Tool unknown model fail-closed (no parent inherit) | 01/03 | shell · `cross_provider_subagent` · `p7_tool_unknown_model` | pass | ❌ Plan 01/03 |
| AGENT-03 | Effort schema on Task | 01/02 | tools · `--lib` · `p7_task_tool_input_schema` | pass | ❌ Plan 01/02 |
| AGENT-03 | Effort threads to runtime overrides | 01/02 | tools · `--lib` · `p7_reasoning_effort` | pass | ❌ Plan 01/02 |
| AGENT-03 | Invalid effort reject at Task | 01/02 | tools · `--lib` · `p7_invalid_reasoning_effort` | pass | ❌ Plan 01/02 |
| AGENT-03 | Tool invalid effort fail-closed in shell | 01/03 | shell · `--lib` · `p7_invalid_effort` | pass | ❌ Plan 01/03 |
| AGENT-03 | Effort medium on child config | 05 | shell · `cross_provider_subagent` · `p7_isolation_reasoning_effort` (under `p7_isolation`) | pass | ❌ Plan 05 |
| AGENT-04 | Dual-direction child route + **Authorization** isolation | 05 | shell · `cross_provider_subagent` · `p7_isolation` (mock HTTP header required) | pass | ❌ Plan 05 |
| AGENT-04 | Never cross-slot on child seed | 05 | shell · `cross_provider_subagent` · `p7_isolation` | pass | ❌ Plan 05 |
| AGENT-05 | Shell spawn missing-provider gate | 01/03 | shell · `cross_provider_subagent` · `p7_spawn_missing_provider` | pass | ❌ Plan 01/03 |
| AGENT-05 | Task eager preflight before bg started | 04 | tools · `--lib` · `p7_eager` | pass | ❌ Plan 04 |
| AGENT-05 | Shell injects TaskProviderCredentialGate | 04 | shell · `--lib` or harness · `p7_credential_gate` | pass | ❌ Plan 04 |
| AGENT-05 | Missing slot no wrong-backend request | 05 | shell · `cross_provider_subagent` · `p7_missing` | pass | ❌ Plan 05 |
| AGENT-06 | Schema exposes model + reasoning_effort | 02 | tools · `--lib` · `p7_task_tool_input_schema` / `p7_reasoning_effort` | pass | ❌ Plan 02 |
| AGENT-06 | Automated spawn + effort + isolation path | 02/05 | tools + shell filters above | pass | ❌ Plans 02–05 |
| AGENT-06 | Live multi-turn dual-login NL E2E | — | **Deferred to Phase 9 (OPS-06)** — not a Phase 7 gate | n/a | n/a deferred |

### Planned filter inventory (representative names)

**Tools `xai-grok-tools --lib`:**
- `p7_task_tool_input_schema_includes_reasoning_effort`
- `p7_reasoning_effort_threads_to_runtime_overrides`
- `p7_invalid_reasoning_effort_rejected_before_spawn`
- `p7_omitted_reasoning_effort_stays_none_on_overrides`
- `p7_eager_missing_provider_rejects_before_background_started`
- `p7_eager_missing_provider_allows_when_gate_none`
- `p7_eager_gate_unavailable_resource_policy`

**Agent `xai-grok-agent --lib` (optional Plan 02):**
- effort guidance / `build_task_description` filter (exact name from Plan 02 SUMMARY)

**Shell `cross_provider_subagent`:**
- `p7_wave0_harness_smoke_compiles_and_runs` (smoke; name may stay after wave 1)
- `p7_spawn_missing_provider_gate_blocks_codex_child_when_codex_slot_empty`
- `p7_spawn_missing_provider_allows_when_slot_usable`
- `p7_spawn_same_provider_no_extra_friction_when_parent_usable`
- `p7_tool_unknown_model_does_not_inherit_parent_provider`
- `p7_isolation_grok_parent_codex_child_route` (+ Authorization)
- `p7_isolation_codex_parent_grok_child_route` (+ Authorization)
- `p7_isolation_never_cross_slot_on_child_seed`
- `p7_isolation_reasoning_effort_medium_on_child_config`
- `p7_parent_model_unchanged_after_cross_provider_spawn`
- `p7_missing_child_codex_no_wrong_backend_request`
- `p7_missing_child_xai_no_wrong_backend_request`
- `p7_same_provider_*` (AGENT-01 anchor if distinct from spawn_same_provider)

**Shell unit / lib:**
- `p7_invalid_effort_tool_provenance_fails_closed`
- `p7_credential_gate_*` / injection unit (Plan 04)
- Existing: `reasoning_effort_explicit_overrides_role`, `resume_model_pinning_overrides_default_resolution`
- Existing roles/personas: `role_default_used_when_no_explicit_override`, `persona_resolved_from_config` (and related under subagent tests)

### ROADMAP success criteria map

| Criterion (ROADMAP Phase 7) | Proof |
|-----------------------------|--------|
| 1. Same-provider no regression | Plan 01 anchors + Plan 06: `reasoning_effort_explicit`, `resume_model_pinning`, `role_default_used` / `persona_resolved`, `p7_same_provider` / `p7_spawn_same_provider` |
| 2. Spawn child on different provider | Plans 03 + 05: `p7_spawn_missing_provider` (usable path) + `p7_isolation` |
| 3. Effort on launch | Plans 02 + 03: tools `p7_reasoning_effort` / `p7_invalid_reasoning_effort` + shell `p7_invalid_effort` + isolation effort |
| 4. Child model→provider→creds→backend | Plans 03 + 05: resolve + **mock HTTP Authorization** dual-direction (`p7_isolation`) |
| 5. NL orchestration + fail-closed login | Plans 02 + 04 + 05: schema/docs + `p7_eager` + `p7_missing` / spawn gate (automated path only) |

### Explicit exclusions

- **Phase 8:** full rebrand chrome/help strings, quiet fork polish
- **Phase 9 / OPS-06:** **live multi-turn dual-login NL E2E** (AGENT-06 live matrix) — not required for Phase 7 gate; automated schema + spawn/effort/isolation is the Phase 7 bar
- Workflow engine, cost dashboards, TUI spawn modal (D-16)
- Stock credential import from `~/.codex` / `~/.grok`
- Unfiltered `cargo test -p xai-grok-shell --lib` as a gate command
- Resolve-only `key_prefix` as sole D-12 / AGENT-04 proof (Authorization outbound required)

---

## Sampling Continuity

| Wave | Plans | Per-task verify | Sampling rule |
|------|-------|-----------------|---------------|
| 1 | 01 | tools `p7_` discovery ≥1; shell `cross_provider_subagent` `p7_` discovery ≥1 + smoke | RED contracts discoverable; smoke green |
| 2 | 02, 03 (parallel) | tools schema/effort; shell `p7_spawn_missing_provider` + `p7_invalid_effort` + `p7_tool` | ≥2/2 automated per plan; **no `\|\| true`** |
| 3 | 04 | tools `p7_eager`; shell `p7_credential_gate` injection | 2/2 automated; named injection filter required |
| 4 | 05 | `p7_isolation` (Authorization) + `p7_missing` + `p7_parent_model` | 2/2 automated; D-12 mock HTTP bar |
| 5 | 06 | **Update** VALIDATION names + PHASE-GATE per-subgroup discover+execute | docs + green filters; AGENT-01 roles/personas included |

> **Wave alignment:** Plan 01 is **frontmatter `wave: 1`** — RED harness / Nyquist contract scaffolding (historically called “Wave 0” in research prose; execution wave number is **1**). Plans 02 and 03 are **wave 2** (parallel; no shared `files_modified` conflict). Plan 04 is **wave 3**. Plan 05 is **wave 4**. Plan 06 is **wave 5**.

---

## Phase gate subgroups (required discovery ≥1 each)

**Tools (`xai-grok-tools --lib`):**
- `p7_task_tool_input_schema` (or schema filter covering reasoning_effort)
- `p7_reasoning_effort` (wire + invalid may share prefix)
- `p7_invalid_reasoning_effort` (if split from effort prefix)
- `p7_eager`

**Shell integration (`--test cross_provider_subagent`):**
- `p7_wave0_harness_smoke` (or retained smoke name)
- `p7_spawn_missing_provider`
- `p7_tool` (unknown model; also covered under Plan 03)
- `p7_isolation`
- `p7_missing`
- `p7_parent_model`
- `p7_same_provider` and/or `p7_spawn_same_provider`

**Shell lib (`--lib`):**
- `p7_invalid_effort` (Tool provenance fail-closed)
- `p7_credential_gate` (Plan 04 injection)
- `reasoning_effort_explicit` (AGENT-01)
- `resume_model_pinning` (AGENT-01 / D-15)
- `role_default_used` (AGENT-01 roles)
- `persona_resolved` (AGENT-01 personas)

**Optional agent crate:** description/effort guidance filter from Plan 02 SUMMARY.

---

## Status fields (Plan 06 updates)

| Field | At plan time | After Plan 01 | After Plan 06 |
|-------|--------------|---------------|---------------|
| `status` | `draft` | `in_progress` | `complete` |
| `wave_0_complete` | `false` | `true` when p7_ discovery ≥1 tools+shell | `true` |
| `nyquist_compliant` | `false` | partial | `true` when all subgroups green |
| `plans_verified` | `[]` | append as SUMMARYs land | `[01,02,03,04,05,06]` |
| Exists? column | planned ❌ / existing ✅ | update from SUMMARYs | all required ✅ or documented deferred |
