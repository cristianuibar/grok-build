---
phase: 7
slug: cross-provider-multi-agent-orchestration
status: complete
nyquist_compliant: true
wave_0_complete: true
created: 2026-07-17
updated: 2026-07-17
plans_verified: [01, 02, 03, 04, 05, 06]
reviews_cycle: 3
---

# Phase 7 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution and final gate.
> Prove **AGENT-01..06** (same-provider regression, cross-provider spawn, effort, isolation,
> fail-closed missing provider + background preflight, NL Task schema surface) with
> **fixture tokens only** — no live ChatGPT / xAI OAuth required for CI gates.
>
> **Authority:** shell `handle_subagent_request` is the authoritative spawn missing-provider
> gate (Plan 03), running **before insert_pending and worktree create/rehydrate**. Task eager
> preflight (Plan 04) is **required async** `SubagentBackend::preflight_spawn` (coordinator
> RPC) using the **same live effective-model path** as shell (omit-model inherit via
> ChatStateHandle + role pins) — **not** a sync slug-only Fn resource (C2-H1).
> Dual-direction Authorization isolation via Plan 05 **minimal harness**
> (spawn → one mock sample → capture Authorization/host + parent model → cancel/join) is the
> D-12 bar (both directions mandatory; not resolve-only).
>
> **Green-only protocol (Codex review cycles 1–2):** Plan 01 landed compile-safe **green**
> scaffolds only — no intentional-red under `p7_`, no `TaskToolInput.reasoning_effort`
> references before Plan 02 added the field. Plans 02–05 added green `p7_*` tests with product
> code. Phase gate claims **all required subgroups green** — never “green except expected-red.”
> **AGENT-05 PASS** requires Plan 04 async eager path green (`p7_eager` + shell
> `p7_preflight`/`p7_credential_gate`) — not filter names alone (C2-M5).
>
> **Review convergence (cycle 3 plan patch):** C3-M1 (Plan 04 preflight full
> spawn-resolution inputs / shared resolve helper) and C3-M2 (Plan 06 Task 2
> discover+execute shell preflight + isolation both dirs + `p7_missing` +
> `p7_parent_model` + `p7_tool`) are folded into PLAN.md. `current_actionable=0`
> MEDIUMs; LOWs only. Accepted without full replan — execute Phase 7.
>
> **Plan 06 update:** Filter inventory below uses **greened names from Plans 01–05 SUMMARYs**.
> `wave_0_complete` / `nyquist_compliant` / `plans_verified` / Exists? set after Plan 06
> discover+execute phase gate (see `07-PHASE-GATE.md`).
>
> **Cargo verify hygiene:** one TESTNAME filter per `cargo test`; chain with `&&` only —
> never `||` that masks a failed cargo (including `cargo test A || cargo test B`). Prefer
> `-p xai-grok-tools` / `-p xai-grok-shell` with narrow filters. **Forbidden gate:** unfiltered
> `cargo test -p xai-grok-shell --lib`. Discovery assert ≥1 match before each filtered execute.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Cargo built-in `cargo test` (tools unit + shell unit/lib + `cross_provider_subagent` integration + Plan 05 in-crate seam) |
| **Config file** | none global — per-crate; `rust-toolchain.toml` (1.92.0) |
| **Quick run command** | `cargo test -p xai-grok-shell --lib p7_isolation -- --nocapture` |
| **Full suite command** | See `07-PHASE-GATE.md` (Plan 06) |
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
| Green-only | No intentional-red / `#[ignore]` expected-red under filters the gate discovers as `p7_` |
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
| Plan 05 minimal harness: spawn → one sample → cancel/join | Resolve-only / private `resolve_model_override_to_config` as D-12 proof |
| Mock HTTP Authorization asserts **both directions** (Plan 05) | One-direction Authorization waiver |
| Plan 04 async `preflight_spawn` (live effective model) | Sync slug-only credential Fn as AGENT-05 solution |
| Dual-slot fixture tokens | Real secrets in tree or CI |
| Plan 01 compile-safe green scaffolds | Intentional-red under `p7_` / referencing missing `reasoning_effort` field |

---

## Locked decisions coverage (D-01..D-16)

| ID | Decision | Verified by filters / docs |
|----|----------|----------------------------|
| D-01 | Omit `Task.model` → inherit parent | tools omit-effort/model inherit + Plan 04 `p7_preflight_omit_model_*` / `p7_eager_omit_model_*` |
| D-02 | Explicit model catalog-wide | `p7_tool_unknown_model_*` existing-reject + spawn resolve |
| D-03 | Expose + wire `reasoning_effort` | tools `p7_task_tool_input_schema_includes_reasoning_effort` / `p7_reasoning_effort_*` (Plan 02) |
| D-04 | Invalid effort reject fail-closed | tools `p7_invalid_reasoning_effort_rejected_before_spawn` + shell `p7_invalid_effort_*` |
| D-05 | Spawn-time usable credential check | shell `p7_spawn_missing_provider_*` (pre-pending + pre-worktree) + tools `p7_eager_*` (async preflight_spawn) + shell `p7_preflight_*` |
| D-06 | No parent bearer/backend fallback | `p7_tool` + isolation / missing |
| D-07 | Login CLI hint | `bum login --provider` in spawn/eager/missing messages |
| D-08 | Same-provider no extra friction | `p7_spawn_same_provider_no_extra_friction_when_parent_usable` |
| D-09 | Child SamplingConfig from child model | `p7_isolation_*` route/base_url |
| D-10 | Child bearer only | `p7_isolation_*` Authorization / never_cross_slot |
| D-11 | Parent model unchanged | `p7_parent_model_unchanged_after_cross_provider_spawn` after **real spawn** |
| D-12 | Dual fake-token Authorization + base_url **both dirs** | `p7_isolation_grok_parent_codex_child_route` + `p7_isolation_codex_parent_grok_child_route` via **minimal harness** (not resolve-only) |
| D-13 | NL via Task schema/docs | schema + agent `p7_build_task_description_includes_effort_guidance` |
| D-14 | AGENT-01 hard gate | resume / effort / roles / personas + same-provider spawn + cited lifecycle |
| D-15 | `resume_from` model pin | `resume_model_pinning_overrides_default_resolution` |
| D-16 | Out of scope | Explicit exclusions below |

---

## Phase Requirements → Test Map

Filters below are **greened names** from Plans 01–05 SUMMARYs (Plan 06 refresh). All required
automated proofs Exist ✅. Live dual-login NL E2E remains deferred Phase 9.

| Req ID | Behavior | Plan | Criterion → crate → filter | Expected | Exists? |
|--------|----------|------|----------------------------|----------|---------|
| AGENT-01 | Same-provider effort override | 01/03/06 | shell · `--lib` · `reasoning_effort_explicit_overrides_role` | pass | ✅ |
| AGENT-01 | Resume model pin (D-15) | 03/06 | shell · `--lib` · `resume_model_pinning_overrides_default_resolution` | pass | ✅ |
| AGENT-01 | Roles regression | 06 | shell · `--lib` · `role_default_used_when_no_explicit_override` | pass | ✅ |
| AGENT-01 | Personas regression | 06 | shell · `--lib` · `persona_resolved_from_config` | pass | ✅ |
| AGENT-01 | Same-provider spawn no extra friction | 03/06 | shell · `--lib` / integration · `p7_spawn_same_provider_no_extra_friction_when_parent_usable` | pass | ✅ |
| AGENT-01 | Same-provider lifecycle spawn/completion/resume (C2-L2) | 06 | **Cited existing:** `agent::subagent::tests::rest::upload_lifecycle_spawn_then_completion_preserves_fields` (spawn→completion field routing) **+** `resume_model_pinning_overrides_default_resolution` (resume model pin) **+** `p7_spawn_same_provider_no_extra_friction_when_parent_usable` (real same-provider coordinator spawn) | pass | ✅ cited |
| AGENT-02 | Cross-provider explicit model resolve | 01/03/05 | shell · `--lib` · `p7_isolation_*` production resolve path | pass | ✅ |
| AGENT-02 | Tool unknown model existing-reject regression | 01/03 | shell · `--lib` / integration · `p7_tool_unknown_model_*` | pass | ✅ |
| AGENT-03 | Effort schema on Task | 02 | tools · `--lib` · `p7_task_tool_input_schema_includes_reasoning_effort` | pass | ✅ |
| AGENT-03 | Effort threads to overrides (canonical) | 02 | tools · `--lib` · `p7_reasoning_effort_threads_to_runtime_overrides` | pass | ✅ |
| AGENT-03 | max alias → xhigh canonical | 02 | tools · `--lib` · `p7_reasoning_effort_max_alias_canonicalizes_to_xhigh` | pass | ✅ |
| AGENT-03 | Invalid effort reject at Task | 02 | tools · `--lib` · `p7_invalid_reasoning_effort_rejected_before_spawn` | pass | ✅ |
| AGENT-03 | Tool invalid / unsupported effort fail-closed in shell | 03 | shell · `--lib` · `p7_invalid_effort_*` | pass | ✅ |
| AGENT-03 | Effort medium on child config | 05 | shell · `--lib` · `p7_isolation_reasoning_effort_medium_on_child_config` | pass | ✅ |
| AGENT-04 | Dual-direction child route + **Authorization** isolation | 05 | shell · `--lib` · `p7_isolation_grok_parent_codex_child_route` + `p7_isolation_codex_parent_grok_child_route` | pass | ✅ |
| AGENT-04 | Never cross-slot on child seed | 05 | shell · `--lib` · `p7_isolation_never_cross_slot_on_child_seed` | pass | ✅ |
| AGENT-05 | Shell spawn missing-provider gate **pre-pending + pre-worktree** | 01/03 | shell · integration/lib · `p7_spawn_missing_provider_*` | pass | ✅ |
| AGENT-05 | No worktree/session/**pending** on missing provider | 03 | shell · `p7_spawn_missing_provider_leaves_no_pending_or_active_child` / `creates_neither_worktree_*` | pass | ✅ |
| AGENT-05 | Task eager **async** preflight before bg started (C2-H1/C2-M5) | 04 | tools · `--lib` · `p7_eager_*` | pass | ✅ |
| AGENT-05 | Shell coordinator async preflight (omit-model + role pins) | 04 | shell · `--lib` · `p7_preflight_*` (no separate `p7_credential_gate_*` name; preflight covers gate) | pass | ✅ |
| AGENT-05 | Missing slot no wrong-backend request | 05 | shell · `--lib` · `p7_missing_child_*` | pass | ✅ |
| AGENT-06 | Schema exposes model + reasoning_effort | 02 | tools · `--lib` · `p7_task_tool_input_schema_includes_reasoning_effort` / `p7_reasoning_effort_*` | pass | ✅ |
| AGENT-06 | Automated spawn + effort + isolation path | 02/05 | tools + shell filters above | pass | ✅ |
| AGENT-06 | Live multi-turn dual-login NL E2E | — | **Deferred to Phase 9 (OPS-06)** — not a Phase 7 gate | n/a | n/a deferred |

### Greened filter inventory (Plan 06)

**Tools `xai-grok-tools --lib`:**
- Schema: `p7_task_tool_input_schema_includes_reasoning_effort`
- Effort wire: `p7_reasoning_effort_threads_to_runtime_overrides`, `p7_reasoning_effort_max_alias_canonicalizes_to_xhigh`, `p7_invalid_reasoning_effort_rejected_before_spawn`, `p7_omitted_reasoning_effort_stays_none_on_overrides`, `p7_blank_reasoning_effort_treated_as_omitted`, `p7_task_model_threads_to_runtime_overrides_still_sets_model`
- Eager async preflight (AGENT-05 / C2-M5): `p7_eager_missing_provider_rejects_before_background_started`, `p7_eager_missing_provider_allows_when_preflight_ok`, `p7_eager_omit_model_preflight_forwards_inherit_context`, `p7_eager_preflight_forwards_subagent_type_for_role_pins`, `p7_eager_preflight_forwards_resume_from_when_present`, `p7_eager_preflight_forwards_runtime_overrides_persona_when_present`, `p7_eager_missing_backend_fail_closed`

**Agent `xai-grok-agent --lib`:**
- `p7_build_task_description_includes_effort_guidance`

**Shell `cross_provider_subagent` integration:**
- `p7_wave0_harness_smoke_compiles_and_runs` (infrastructure smoke only — **not** D-12 Authorization proof; C2-L1)
- `p7_spawn_missing_provider_*` (pre-worktree / no-pending)
- `p7_spawn_same_provider_no_extra_friction_when_parent_usable`
- `p7_tool_unknown_model_catalog_reject_shape`
- Pure helpers: `p7_missing_provider_gate_error_*`, `p7_resolve_route_isolates_base_url_key_prefix_both_directions`, `p7_empty_codex_slot_reads_none_with_xai_present`

**Shell lib / in-crate seam (prefer `--lib` — bare package list may be poisoned):**
- Isolation / D-12 (Plan 05 **minimal harness** `p7_isolation_spawn_sample_cancel` — real child-sample Authorization, not resolve-only):
  - `p7_isolation_grok_parent_codex_child_route`
  - `p7_isolation_codex_parent_grok_child_route`
  - `p7_isolation_never_cross_slot_on_child_seed`
  - `p7_isolation_reasoning_effort_medium_on_child_config`
- Parent / missing: `p7_parent_model_unchanged_after_cross_provider_spawn`, `p7_missing_child_codex_no_wrong_backend_request`, `p7_missing_child_xai_no_wrong_backend_request`
- Spawn gate: `p7_spawn_missing_provider_*`, `p7_spawn_same_provider_*`
- Effort: `p7_invalid_effort_*`
- Preflight async (Plan 04 / AGENT-05): `p7_preflight_*` (omit-model, role pin, persona pin, resume pin, live chat state)
- Tool: `p7_tool_unknown_model_rejected_by_existing_task_model_override_error`
- Slot helper: `p7_provider_slot_usable_*`, `p7_missing_provider_spawn_error_message_suggests_login`
- AGENT-01 existing: `reasoning_effort_explicit_overrides_role`, `resume_model_pinning_overrides_default_resolution`, `role_default_used_when_no_explicit_override`, `persona_resolved_from_config`
- Lifecycle (cited C2-L2): `upload_lifecycle_spawn_then_completion_preserves_fields`

### ROADMAP success criteria map

| Criterion (ROADMAP Phase 7) | Proof |
|-----------------------------|--------|
| 1. Same-provider no regression | Plan 06: `reasoning_effort_explicit`, `resume_model_pinning`, `role_default_used`, `persona_resolved`, `p7_spawn_same_provider` + cited lifecycle |
| 2. Spawn child on different provider | Plans 03 + 05: `p7_spawn_missing_provider` (usable path) + `p7_isolation` both dirs |
| 3. Effort on launch | Plans 02 + 03: tools `p7_reasoning_effort` / canonical max + shell `p7_invalid_effort` + isolation effort |
| 4. Child model→provider→creds→backend | Plans 03 + 05: pre-worktree gate + **mock HTTP Authorization** dual-direction via minimal harness |
| 5. NL orchestration + fail-closed login | Plans 02 + 04 + 05: schema/docs + **async** effective-model `p7_eager` / shell `p7_preflight` + `p7_missing` / spawn gate (automated path only) |

### Explicit exclusions (D-16)

- **Phase 8:** full rebrand chrome/help strings, quiet fork polish
- **Phase 9 / OPS-06:** **live multi-turn dual-login NL E2E** (AGENT-06 live matrix) — not required for Phase 7 gate; automated schema + spawn/effort/isolation is the Phase 7 bar
- Workflow engine, cost dashboards, TUI spawn modal (D-16)
- Stock credential import from `~/.codex` / `~/.grok`
- Unfiltered `cargo test -p xai-grok-shell --lib` as a gate command
- Resolve-only `key_prefix` as sole D-12 / AGENT-04 proof (Authorization outbound required both directions)
- Intentional-red / expected-fail under `p7_` as a phase completion path
- Slug-only Task preflight that ignores omit-model inherit or role pins
- Live ChatGPT / xAI OAuth as a required CI gate (fixture tokens only)

---

## Sampling Continuity

| Wave | Plans | Per-task verify | Sampling rule |
|------|-------|-----------------|---------------|
| 1 | 01 | tools `p7_` discovery ≥1 **and all green**; shell `cross_provider_subagent` `p7_` discovery ≥1 **and all green** + smoke | Compile-safe green scaffolds only |
| 2 | 02, 03 (parallel) | tools schema/effort/canonical; shell `p7_spawn_missing_provider` (pre-worktree) + `p7_invalid_effort` + `p7_tool` | ≥2/2 automated per plan; **no `\|\| true`** |
| 3 | 04 | tools `p7_eager` (effective model); shell `p7_preflight` async coordinator | 2/2 automated; omit-model + role pin covered |
| 4 | 05 | minimal harness + `p7_isolation` **both Authorization dirs** + `p7_missing` + `p7_parent_model` real spawn | 2/2 automated; D-12 hard bar both dirs |
| 5 | 06 | **Update** VALIDATION names + PHASE-GATE per-subgroup discover+execute green-only | docs + green filters; AGENT-01 roles/personas + cited lifecycle |

> **Wave alignment:** Plan 01 is **frontmatter `wave: 1`** — green harness / Nyquist scaffolding (historically “Wave 0” in research prose; execution wave number is **1**). Plans 02 and 03 are **wave 2** (parallel). Plan 04 is **wave 3**. Plan 05 is **wave 4**. Plan 06 is **wave 5**.

---

## Phase gate subgroups (required discovery ≥1 each)

**Tools (`xai-grok-tools --lib`):**
- `p7_task_tool_input_schema` (covers `reasoning_effort` after Plan 02)
- `p7_reasoning_effort` (wire + invalid + max→xhigh)
- `p7_eager` (**AGENT-05 hard bar**, C2-M5)

**Agent (`xai-grok-agent --lib`):**
- `p7_` / `p7_build_task_description_includes_effort_guidance`

**Shell integration (`--test cross_provider_subagent`):**
- `p7_wave0_harness_smoke`
- `p7_spawn_missing_provider`
- `p7_tool`
- `p7_spawn_same_provider` (AGENT-01 same-provider spawn)

**Shell lib (`--lib`) — Plan 05 seam + Plan 04 preflight + AGENT-01:**
- `p7_isolation` including **both** `p7_isolation_grok_parent_codex` and `p7_isolation_codex_parent_grok`
- `p7_missing` / `p7_missing_child`
- `p7_parent_model`
- `p7_invalid_effort`
- `p7_preflight` (**AGENT-05 hard bar**, C2-M5; covers credential gate — no separate `p7_credential_gate` filter)
- `p7_tool` (also on lib)
- `reasoning_effort_explicit` (AGENT-01)
- `resume_model_pinning` (AGENT-01 / D-15)
- `role_default_used` (AGENT-01 roles)
- `persona_resolved` (AGENT-01 personas)
- Lifecycle: `upload_lifecycle_spawn_then_completion` (cited C2-L2) + `p7_spawn_same_provider`

**D-12 checklist (Plan 05 + Plan 06):**
- [x] Test seam documented in Plan 05 SUMMARY (`p7_isolation_spawn_sample_cancel` minimal harness)
- [x] `p7_isolation_grok_parent_codex` green with Authorization (child-sample mock HTTP)
- [x] `p7_isolation_codex_parent_grok` green with Authorization (child-sample mock HTTP)
- [x] Neither direction is resolve-only only

**AGENT-05 checklist (Plan 04 async + Plan 06 gate):**
- [x] Plan 04 SUMMARY: production design is async `SubagentBackend::preflight_spawn` (not sync slug-only)
- [x] tools `p7_eager` discovers ≥1 and passes
- [x] shell `p7_preflight` discovers ≥1 and passes (omit-model + role/persona/resume pins)

---

## Status fields (Plan 06)

| Field | At plan time | After Plan 01 | After Plan 06 |
|-------|--------------|---------------|---------------|
| `status` | `draft` | `in_progress` | **`complete`** |
| `wave_0_complete` | `false` | `true` when p7_ discovery ≥1 tools+shell **and all green** | **`true`** |
| `nyquist_compliant` | `false` | partial | **`true`** (all green subgroups + D-12 + AGENT-05 async) |
| `plans_verified` | `[]` | append as SUMMARYs land | **`[01,02,03,04,05,06]`** |
| Exists? column | planned ❌ / existing ✅ | update from SUMMARYs | **all required ✅** or documented deferred |
| `reviews_cycle` | `3` | keep | keep |
