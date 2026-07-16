---
phase: 6
slug: mid-session-switch-missing-provider-gate
status: draft
nyquist_compliant: true
wave_0_complete: false
created: 2026-07-17
updated: 2026-07-17
---

# Phase 6 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Prove **MOD-03** (mid-session free switch; next turn uses new model) and **MOD-06**
> (switch-time missing-provider gate + login prompt; no silent mid-turn 401 as primary UX)
> with **fixture tokens only** — no live ChatGPT / xAI OAuth required for CI gates.
>
> **Authority:** shell `model_switch::apply` is the single missing-provider gate (Plan 01).
> Pager QuestionView + deferred login (Plans 02–03) and picker badge (Plan 04) are UX halves.
> Free dual-provider switch proofs live in Plan 05 (`provider_routing`).
>
> **Cargo verify hygiene:** one TESTNAME filter per `cargo test`; never bare `| tail` without
> `set -o pipefail`; chain with `&&` only. Prefer `-p xai-grok-shell` / `-p xai-grok-pager`
> with narrow filters. **Forbidden gate:** unfiltered `cargo test -p xai-grok-shell --lib`.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Cargo built-in `cargo test` (shell integration + pager unit/dispatch tests) |
| **Config file** | none global — per-crate; `rust-toolchain.toml` (1.92.0) |
| **Quick run command** | `cargo test -p xai-grok-shell --test provider_routing missing_provider -- --nocapture` |
| **Full suite command** | See Phase gate aggregate below |
| **Estimated runtime** | ~60–180 seconds after first compile (shell routing + pager dispatch) |

### Cargo verify hygiene (locked)

| Rule | Detail |
|------|--------|
| One TESTNAME filter | Cargo accepts **one** positional TESTNAME filter per invocation |
| Multi-test coverage | Chain single-filter invocations with `&&` |
| Exit status | Prefer **no pipe** on cargo |
| Chains | Use `&&` only — never `;` that masks failures |
| Forbidden gates | Unfiltered `cargo test -p xai-grok-shell --lib` |
| Fixtures only | Fake tokens / tempfile `auth.json`; no live OAuth secrets |

### Harness policy

| Allowed | Forbidden |
|---------|-----------|
| `cargo test -p xai-grok-shell --test provider_routing <filter>` | Unfiltered shell `--lib` as phase gate |
| `cargo test -p xai-grok-shell <narrow unit filter>` | Live ChatGPT / xAI login as required gate |
| `cargo test -p xai-grok-pager <dispatch/slash filter>` | Phase 7 subagent orchestration tests as Phase 6 proof |
| Explicit auth file paths | Stock `~/.codex` / `~/.grok` credential import |
| Dual-slot fixture tokens | Collapsing MissingProvider into IncompatibleAgent |

---

## Phase Requirements → Test Map

| Req ID | Behavior | Plan | Automated Command | File Exists? |
|--------|----------|------|-------------------|--------------|
| MOD-06 | Missing Codex blocks switch to GPT model | 01 | `cargo test -p xai-grok-shell --test provider_routing missing_provider -- --nocapture` | ❌ until Plan 01 |
| MOD-06 | Typed ACP error code + suggestion round-trip | 01 | `cargo test -p xai-grok-shell model_switch_missing_provider_error -- --nocapture` | ❌ until Plan 01 |
| MOD-06 | Refreshable token is usable (not blocked) | 01 | `cargo test -p xai-grok-shell --test provider_routing missing_provider -- --nocapture` | ❌ until Plan 01 |
| MOD-06 | BYOK `has_own_credentials` skips OAuth-slot gate | 01 / 05 | `cargo test -p xai-grok-shell --test provider_routing byok -- --nocapture` | ❌ until Plan 01/05 |
| MOD-06 | SwitchModelComplete → QuestionView MissingProviderLogin | 02 | `cargo test -p xai-grok-pager missing_provider -- --nocapture` | ❌ until Plan 02 |
| MOD-06 | No optimistic `models.current` on MissingProvider | 02 | `cargo test -p xai-grok-pager missing_provider -- --nocapture` | ❌ until Plan 02 |
| MOD-06 | Keep current dismisses; stays on previous model | 02 | `cargo test -p xai-grok-pager keep_current -- --nocapture` | ❌ until Plan 02 |
| MOD-06 | Login now stashes `deferred_model_switch` | 03 | `cargo test -p xai-grok-pager login_now -- --nocapture` | ❌ until Plan 03 |
| MOD-06 | AuthComplete applies deferred switch retry | 03 | `cargo test -p xai-grok-pager auth_complete -- --nocapture` | ❌ until Plan 03 |
| MOD-06 | Full mixed catalog + `needs login` badge | 04 | `cargo test -p xai-grok-pager needs_login -- --nocapture` | ❌ until Plan 04 |
| MOD-06 | Badge usable semantics (refreshable = no badge) | 04 | `cargo test -p xai-grok-pager build_model_items -- --nocapture` | ❌ until Plan 04 |
| MOD-03 | Successful switch updates next sample route | 05 / Phase 4 | `cargo test -p xai-grok-shell --test provider_routing switch_changes_next_sample_route -- --nocapture` | ✅ |
| MOD-03 | Dual-login free Grok↔GPT switch | 05 | `cargo test -p xai-grok-shell --test provider_routing dual_login -- --nocapture` | ❌ until Plan 05 |
| MOD-03 | Same-provider switch no missing-provider friction | 05 | `cargo test -p xai-grok-shell --test provider_routing same_provider -- --nocapture` | ❌ until Plan 05 |
| MOD-03 / D-06 | History preserved + next-turn-only switch | 05 | `cargo test -p xai-grok-shell --test provider_routing history -- --nocapture` | ❌ until Plan 05 |
| MOD-06 / D-07 | IncompatibleAgent path not collapsed | 02 / 06 | `cargo test -p xai-grok-pager incompatible_agent -- --nocapture` | ✅ pattern |

### ROADMAP success criteria map

| Criterion | Proof |
|-----------|--------|
| 1. Switch mid-session anytime; next turn uses new model | Plan 05 free switch + `switch_changes_next_sample_route`; Plan 03 success scrollback reuse |
| 2. Missing credentials block switch + login prompt | Plans 01–03 (shell gate + QuestionView + deferred login) |
| 3. Both providers logged in → free Grok↔GPT in one session | Plan 05 `dual_login` / `free_switch` |

### Explicit exclusions

- Phase 7: cross-provider subagent spawn credentials
- Phase 8: full rebrand chrome/help strings
- Live OAuth browser smoke (optional manual only)
- Stock credential import from `~/.codex` / `~/.grok`

---

## Sampling Continuity

| Wave | Plans | Per-task verify | Sampling rule |
|------|-------|-----------------|---------------|
| 1 | 01 | shell unit + `provider_routing missing_provider` | 2/2 automated |
| 2 | 02, 05 | pager missing_provider / keep_current; shell dual_login | ≥2/3 per 3-task window |
| 3 | 03, 04 | login_now / auth_complete; needs_login / build_model_items | 4/4 automated |
| 4 | 06 | VALIDATION present + aggregate cargo gate | docs + green filters |

---

## Wave 0 gaps (closed by plan RED tasks, not a separate plan)

- [ ] Shell unit: pure provider_slot_usable decision table + ModelSwitchMissingProviderError serde (Plan 01 T1)
- [ ] Shell integration: apply returns missing-provider when Codex empty (Plan 01 T2)
- [ ] Pager: SwitchModelError::MissingProvider + QuestionView (Plan 02)
- [ ] Pager: deferred retry after simulated AuthComplete (Plan 03)
- [ ] Pager: build_model_items badge (Plan 04)
- [ ] Shell: dual free switch + history/BYOK (Plan 05)
- [ ] IncompatibleAgent regression still green (Plan 02 / 06)

---

## Phase gate aggregate (Plan 06)

```bash
cargo test -p xai-grok-shell --test provider_routing switch_changes_next_sample_route -- --nocapture && \
cargo test -p xai-grok-shell missing_provider -- --nocapture && \
cargo test -p xai-grok-shell --test provider_routing dual_login -- --nocapture && \
cargo test -p xai-grok-pager missing_provider -- --nocapture && \
cargo test -p xai-grok-pager keep_current -- --nocapture && \
cargo test -p xai-grok-pager login_now -- --nocapture && \
cargo test -p xai-grok-pager auth_complete -- --nocapture && \
cargo test -p xai-grok-pager needs_login -- --nocapture && \
cargo test -p xai-grok-pager incompatible_agent -- --nocapture
```

> Plan 06 Task 1 updates this file with **actual** test names from 06-0N-SUMMARY.md after execution.
> Plan 06 Task 2 records pass timestamp in `06-PHASE-GATE.md`.

---

## Security notes (gate-related)

| Threat | Mitigation proven by |
|--------|----------------------|
| Silent mid-turn 401 as primary UX | Switch-time shell gate (missing_provider filters) |
| Client-spoofed provider on error | Provider from catalog `ModelEntry.info.provider` only |
| Token leak in error/modal | Suggestion is CLI only; structured fields; sanitize Other |
| Cross-slot credential use | Phase 4 route + dual free-switch asserts target slot |
