---
phase: 6
slug: mid-session-switch-missing-provider-gate
status: draft
nyquist_compliant: true
wave_0_complete: false
created: 2026-07-17
updated: 2026-07-17
review_cycle: 2
---

# Phase 6 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Prove **MOD-03** (mid-session free switch; next turn uses new model) and **MOD-06**
> (switch-time missing-provider gate + login prompt; no silent mid-turn 401 as primary UX)
> with **fixture tokens only** — no live ChatGPT / xAI OAuth required for CI gates.
>
> **Authority:** shell `model_switch::apply` is the single missing-provider gate (Plan 01).
> Pager QuestionView + deferred login (Plans 02–03) and picker badge (Plan 04) are UX halves.
> Free dual-provider switch proofs live in Plan 05 (`model_switch_gate` session harness).
>
> **Review cycle 1:** unique `p6_` test prefixes + discovery assert ≥1 match before each
> filtered execute group (prevents cargo false-green on empty filters).
>
> **Review cycle 2:** Plan 04 owns badge-cache lifecycle refresh (startup/login/logout/focus);
> Plan 04 wave 3 after Plan 02 (effects ownership); deferred carries `persist_default`;
> Plan 05 history/mid-turn use named observables only; Plan 06 gate uses **per-subgroup**
> discovery (not aggregate `p6_` only); Plan 04 pager verify has no `||` masking.
>
> **Cargo verify hygiene:** one TESTNAME filter per `cargo test`; never bare `| tail` without
> `set -o pipefail`; chain with `&&` only — never `||` that masks a failed cargo. Prefer
> `-p xai-grok-shell` / `-p xai-grok-pager` with narrow filters. **Forbidden gate:** unfiltered
> `cargo test -p xai-grok-shell --lib`.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Cargo built-in `cargo test` (shell integration + pager unit/dispatch tests) |
| **Config file** | none global — per-crate; `rust-toolchain.toml` (1.92.0) |
| **Quick run command** | `cargo test -p xai-grok-shell --test model_switch_gate p6_missing_provider -- --nocapture` |
| **Full suite command** | See Phase gate aggregate below |
| **Estimated runtime** | ~60–180 seconds after first compile (shell gate + pager dispatch) |

### Cargo verify hygiene (locked)

| Rule | Detail |
|------|--------|
| One TESTNAME filter | Cargo accepts **one** positional TESTNAME filter per invocation |
| Multi-test coverage | Chain single-filter invocations with `&&` |
| Exit status | Prefer **no pipe** on cargo |
| Chains | Use `&&` only — never `;` that masks failures |
| Discovery assert | Before each group: `n=$(cargo test … -- --list \| grep -c 'p6_…' \|\| true); test "$n" -ge 1` |
| Unique prefixes | All new Phase 6 tests use `p6_` prefix (avoid `auth_complete`, `needs_login`, `deferred_model_switch` alone) |
| Forbidden gates | Unfiltered `cargo test -p xai-grok-shell --lib` |
| Fixtures only | Fake tokens / tempfile `auth.json`; no live OAuth secrets |

### Harness policy

| Allowed | Forbidden |
|---------|-----------|
| `cargo test -p xai-grok-shell --test model_switch_gate <p6_ filter>` | Unfiltered shell `--lib` as phase gate |
| `cargo test -p xai-grok-shell --test provider_routing switch_changes_next_sample_route` | Treating pure routing as history/mid-turn proof |
| `cargo test -p xai-grok-pager p6_<filter>` | Live ChatGPT / xAI login as required gate |
| Explicit auth file paths | Stock `~/.codex` / `~/.grok` credential import |
| Dual-slot fixture tokens | Collapsing MissingProvider into IncompatibleAgent |
| Session/MvpAgent apply path | provider_routing-only for MOD-06 gate proofs |

---

## Phase Requirements → Test Map

| Req ID | Behavior | Plan | Automated Command | File Exists? |
|--------|----------|------|-------------------|--------------|
| MOD-06 | Missing Codex blocks switch to GPT (apply path) | 01 | `cargo test -p xai-grok-shell --test model_switch_gate p6_missing_provider -- --nocapture` | ❌ until Plan 01 |
| MOD-06 | Typed ACP error code + suggestion round-trip | 01 | `cargo test -p xai-grok-shell p6_model_switch_missing_provider_error -- --nocapture` | ❌ until Plan 01 |
| MOD-06 | Apply blocked → no side effects | 01 | `cargo test -p xai-grok-shell --test model_switch_gate p6_missing_provider_apply_no_side_effects -- --nocapture` | ❌ until Plan 01 |
| MOD-06 | Refreshable token is usable (not blocked) | 01 | `cargo test -p xai-grok-shell --test model_switch_gate p6_missing_provider -- --nocapture` | ❌ until Plan 01 |
| MOD-06 | BYOK `has_own_credentials` skips OAuth-slot gate | 01 / 05 | `cargo test -p xai-grok-shell --test model_switch_gate p6_byok -- --nocapture` | ❌ until Plan 01/05 |
| MOD-06 | SwitchModelComplete → QuestionView MissingProviderLogin | 02 | `cargo test -p xai-grok-pager p6_missing_provider -- --nocapture` | ❌ until Plan 02 |
| MOD-06 | Transactional default — no optimistic current/persist | 02 | `cargo test -p xai-grok-pager p6_transactional_default -- --nocapture` | ❌ until Plan 02 |
| MOD-06 | Keep current dismisses; stays on previous model | 02 | `cargo test -p xai-grok-pager p6_keep_current -- --nocapture` | ❌ until Plan 02 |
| MOD-06 | Login now stashes provider-aware deferred | 03 | `cargo test -p xai-grok-pager p6_login_now -- --nocapture` | ❌ until Plan 03 |
| MOD-06 | AuthComplete applies deferred when required provider usable | 03 | `cargo test -p xai-grok-pager p6_auth_ -- --nocapture` | ❌ until Plan 03 |
| MOD-06 | External CLI status refresh applies deferred | 03 | `cargo test -p xai-grok-pager p6_external_cli -- --nocapture` | ❌ until Plan 03 |
| MOD-06 | Full mixed catalog + `needs login` badge | 04 | `cargo test -p xai-grok-pager p6_needs_login -- --nocapture` | ❌ until Plan 04 |
| MOD-06 | AuthMeta dual-slot usable + AppView cache | 04 | `cargo test -p xai-grok-pager p6_provider_auth -- --nocapture` | ❌ until Plan 04 |
| MOD-06 | Badge cache refresh on startup/login/logout/focus | 04 | `cargo test -p xai-grok-pager p6_provider_auth -- --nocapture` + `p6_refresh` | ❌ until Plan 04 |
| MOD-06 | BYOK suppresses needs-login badge | 04 | `cargo test -p xai-grok-pager p6_needs_login_byok -- --nocapture` | ❌ until Plan 04 |
| MOD-06 | Settings DynamicEnum same needs-login badge | 04 | `cargo test -p xai-grok-pager p6_needs_login_settings -- --nocapture` | ❌ until Plan 04 |
| MOD-06 | External CLI FocusGained / poll emission + generation cancel | 03 | `cargo test -p xai-grok-pager p6_focus_gained -- --nocapture` + `p6_refresh_generation` | ❌ until Plan 03 |
| MOD-06 | Deferred carries persist_default | 02 / 03 | `cargo test -p xai-grok-pager p6_deferred -- --nocapture` + `p6_transactional_default` | ❌ until Plan 02/03 |
| MOD-03 | Successful switch updates next sample route (pure) | 05 / Phase 4 | `cargo test -p xai-grok-shell --test provider_routing switch_changes_next_sample_route -- --nocapture` | ✅ |
| MOD-03 | Dual-login free Grok↔GPT switch (apply) | 05 | `cargo test -p xai-grok-shell --test model_switch_gate p6_dual_login -- --nocapture` | ❌ until Plan 05 |
| MOD-03 | Same-provider switch no missing-provider friction | 05 | `cargo test -p xai-grok-shell --test model_switch_gate p6_same_provider -- --nocapture` | ❌ until Plan 05 |
| MOD-03 / D-06 | History preserved (chat history snapshot) + mid-turn non-cancel (held MockInferenceServer + no session/cancel) | 05 | `cargo test -p xai-grok-shell --test model_switch_gate p6_history -- --nocapture` + `p6_mid_turn` | ❌ until Plan 05 |
| MOD-06 / D-07 | IncompatibleAgent path not collapsed | 02 / 06 | `cargo test -p xai-grok-pager incompatible_agent -- --nocapture` | ✅ pattern |

### ROADMAP success criteria map

| Criterion | Proof |
|-----------|--------|
| 1. Switch mid-session anytime; next turn uses new model | Plan 05 `p6_dual_login` + next-sample assert; pure `switch_changes_next_sample_route`; Plan 03 success scrollback reuse |
| 2. Missing credentials block switch + login prompt | Plans 01–03 (shell gate + transactional QuestionView + deferred + external CLI) |
| 3. Both providers logged in → free Grok↔GPT in one session | Plan 05 `p6_dual_login` on apply harness |

### Explicit exclusions

- Phase 7: cross-provider subagent spawn credentials
- Phase 8: full rebrand chrome/help strings
- Live OAuth browser smoke (optional manual only)
- Stock credential import from `~/.codex` / `~/.grok`

---

## Sampling Continuity

| Wave | Plans | Per-task verify | Sampling rule |
|------|-------|-----------------|---------------|
| 1 | 01 | shell unit + `model_switch_gate p6_missing_provider` | 2/2 automated |
| 2 | 02, 05 | pager p6_missing_provider / p6_transactional_default; shell p6_dual_login / p6_history / p6_mid_turn | ≥2/2 per plan |
| 3 | 04 | p6_provider_auth / p6_refresh / p6_needs_login (slash + settings) | 3/3 automated |
| 4 | 03 | p6_login_now / p6_auth_ / p6_external_cli / p6_focus_gained / p6_refresh_generation | 2/2 automated |
| 5 | 06 | VALIDATION present + **per-subgroup** cargo gate with discovery asserts | docs + green filters |

> **Wave alignment (review cycle 2):** Plan 02 and 05 are wave 2 (parallel). Plan 04 is **wave 3** (depends on 02 — sole owner of RefreshProviderAuthStatus after SwitchModel Effect lands). Plan 03 is **wave 4** (depends on 02 + 04). Plan 06 is **wave 5**.

---

## Wave 0 gaps (closed by plan RED tasks, not a separate plan)

- [ ] Shell unit: pure provider_slot_usable + ModelSwitchMissingProviderError serde (Plan 01 T1)
- [ ] Shell integration: apply returns missing-provider + no side effects (Plan 01 T2 — `model_switch_gate`)
- [ ] Pager: SwitchModelError::MissingProvider + transactional default + QuestionView (Plan 02)
- [ ] Pager: AuthMeta dual-slot + lifecycle refresh (startup/login/logout/focus) + badge + settings DynamicEnum + BYOK (Plan 04)
- [ ] Shell: dual free switch + named history/mid-turn session proofs (Plan 05)
- [ ] Pager: deferred (persist_default) + AuthComplete + external CLI poll/FocusGained + generation cancel (Plan 03)
- [ ] IncompatibleAgent regression still green (Plan 02 / 06)

---

## Phase gate aggregate (Plan 06)

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

discover xai-grok-shell p6_missing_provider --test model_switch_gate
discover xai-grok-shell p6_dual_login --test model_switch_gate
discover xai-grok-shell p6_byok --test model_switch_gate
discover xai-grok-shell p6_history --test model_switch_gate
discover xai-grok-shell p6_mid_turn --test model_switch_gate
cargo test -p xai-grok-shell --test provider_routing switch_changes_next_sample_route -- --nocapture
discover xai-grok-shell p6_model_switch_missing_provider

discover xai-grok-pager p6_missing_provider
discover xai-grok-pager p6_transactional_default
discover xai-grok-pager p6_keep_current
discover xai-grok-pager p6_login_now
discover xai-grok-pager p6_deferred
discover xai-grok-pager p6_auth_
discover xai-grok-pager p6_external_cli
discover xai-grok-pager p6_focus_gained
discover xai-grok-pager p6_refresh_generation
discover xai-grok-pager p6_needs_login
discover xai-grok-pager p6_provider_auth
discover xai-grok-pager p6_refresh
cargo test -p xai-grok-pager incompatible_agent -- --nocapture
```

> **Per-subgroup rule (cycle 2):** every `discover` line above is required independently. Aggregate
> `grep -c p6_` per crate is **not** a substitute for missing subgroups.
>
> Plan 06 Task 1 updates this file with **actual** test names from 06-0N-SUMMARY.md after execution.
> Plan 06 Task 2 records pass timestamp in `06-PHASE-GATE.md`.

---

## Security notes (gate-related)

| Threat | Mitigation proven by |
|--------|----------------------|
| Silent mid-turn 401 as primary UX | Switch-time shell gate (p6_missing_provider filters) |
| Client-spoofed provider on error | Provider from catalog `ModelEntry.info.provider` only; pager parses xai\|codex only |
| Token leak in error/modal | Suggestion is CLI only; structured fields; sanitize Other |
| Optimistic default persist | p6_transactional_default |
| Cross-slot credential use | Phase 4 route + p6_dual_login asserts target slot |
| Empty filter false green | Discovery assert ≥1 per p6_ group |
