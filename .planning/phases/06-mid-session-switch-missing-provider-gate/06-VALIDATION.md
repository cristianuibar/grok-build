---
phase: 6
slug: mid-session-switch-missing-provider-gate
status: complete
nyquist_compliant: true
wave_0_complete: true
created: 2026-07-17
updated: 2026-07-17
review_cycle: 2
plans_verified: [01, 02, 03, 04, 05]
---

# Phase 6 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution and final gate.
> Prove **MOD-03** (mid-session free switch; next turn uses new model) and **MOD-06**
> (switch-time missing-provider gate + login prompt; no silent mid-turn 401 as primary UX)
> with **fixture tokens only** — no live ChatGPT / xAI OAuth required for CI gates.
>
> **Authority:** shell `model_switch::apply` is the single missing-provider gate (Plan 01).
> Pager QuestionView + deferred login (Plans 02–03) and picker badge (Plan 04) are UX halves.
> Free dual-provider switch proofs live in Plan 05 (`model_switch_gate` session harness).
>
> **MOD-06 full UX requires both halves:** shell typed error alone is partial; ACP error without
> QuestionView is not the product UX. Gate must run shell **and** pager `p6_` subgroups.
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
| **Full suite command** | See Phase gate aggregate below / `06-PHASE-GATE.md` |
| **Estimated runtime** | ~60–180 seconds after first compile (shell gate + pager dispatch) |

### Cargo verify hygiene (locked)

| Rule | Detail |
|------|--------|
| One TESTNAME filter | Cargo accepts **one** positional TESTNAME filter per invocation |
| Multi-test coverage | Chain single-filter invocations with `&&` |
| Exit status | Prefer **no pipe** on cargo |
| Chains | Use `&&` only — never `;` that masks failures |
| Discovery assert | **Every** required subgroup: `discover()` → `test "$n" -ge 1` then execute |
| Unique prefixes | All Phase 6 proofs use `p6_` (avoid bare `auth_complete`, `needs_login`, `deferred_model_switch`) |
| Forbidden gates | Unfiltered `cargo test -p xai-grok-shell --lib`; aggregate-only `grep -c p6_` |
| Fixtures only | Fake tokens / tempfile `auth.json`; no live OAuth secrets |

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

> **Per-subgroup rule (cycle 2):** every required filter below must pass its own `discover`.
> Aggregate `grep -c p6_` per crate is **not** a substitute for a missing subgroup.

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

## Locked decisions coverage (D-01..D-08)

| ID | Decision | Verified by filters / docs |
|----|----------|----------------------------|
| D-01 | Shell authority: gate in `model_switch::apply` | shell `p6_missing_provider` apply gate |
| D-02 | Usable = store_usable / refreshable; dual slots | usable helpers + badge semantics (`p6_provider_slot_usable`, `p6_auth_meta`, `p6_provider_auth`, `p6_needs_login`) |
| D-03 | Login now / deferred / external CLI / focus refresh | `p6_login_now` + `p6_deferred` + `p6_auth_` + `p6_external_cli` + `p6_focus_gained` / `p6_refresh_generation` |
| D-04 | Full catalog + needs-login badge + lifecycle refresh | `p6_needs_login` + catalog count + `p6_provider_auth` lifecycle |
| D-05 | Transactional default + no optimistic current | `p6_missing_provider` + `p6_transactional_default` |
| D-06 | Dual free switch + history + mid-turn non-cancel | `p6_dual_login` + `p6_history` + `p6_mid_turn` |
| D-07 | MissingProvider ≠ IncompatibleAgent | separate codes + `incompatible_agent` green |
| D-08 | UI-SPEC copy on QuestionView / badge | UI-SPEC copy assertions in pager `p6_missing_provider` / `p6_needs_login` |

---

## Phase Requirements → Test Map

Filters below match greened names from `06-01`…`06-05` SUMMARYs (2026-07-16/17).

| Req ID | Behavior | Plan | Criterion → crate → p6_ filter | Expected | Exists? |
|--------|----------|------|--------------------------------|----------|---------|
| MOD-06 | Missing Codex blocks switch to GPT (apply path) | 01 | shell · `model_switch_gate` · `p6_missing_provider` | pass | ✅ |
| MOD-06 | Typed ACP error code + suggestion round-trip | 01 | shell · unit/lib · `p6_model_switch_missing_provider` | pass | ✅ |
| MOD-06 | Apply blocked → no side effects | 01 | shell · `model_switch_gate` · `p6_missing_provider_apply_no_side_effects` (under `p6_missing_provider`) | pass | ✅ |
| MOD-06 | Refreshable token is usable (not blocked) | 01 | shell · `model_switch_gate` · `p6_missing_provider_apply_allows_codex_when_refreshable` | pass | ✅ |
| MOD-06 | BYOK `has_own_credentials` skips OAuth-slot gate | 01 / 05 | shell · `model_switch_gate` · `p6_byok` | pass | ✅ |
| MOD-06 | SwitchModelComplete → QuestionView MissingProviderLogin | 02 | pager · lib · `p6_missing_provider` | pass | ✅ |
| MOD-06 | Transactional default — no optimistic current/persist | 02 | pager · lib · `p6_transactional_default` | pass | ✅ |
| MOD-06 | Keep current dismisses; stays on previous model | 02 | pager · lib · `p6_keep_current` | pass | ✅ |
| MOD-06 | Login now stashes provider-aware deferred | 03 | pager · lib · `p6_login_now` | pass | ✅ |
| MOD-06 | AuthComplete applies deferred when required provider usable | 03 | pager · lib · `p6_auth_` | pass | ✅ |
| MOD-06 | External CLI status refresh applies deferred | 03 | pager · lib · `p6_external_cli` | pass | ✅ |
| MOD-06 | Full mixed catalog + `needs login` badge | 04 | pager · lib · `p6_needs_login` | pass | ✅ |
| MOD-06 | AuthMeta dual-slot usable + AppView cache | 04 | pager · lib · `p6_provider_auth` (+ shell `p6_auth_meta`) | pass | ✅ |
| MOD-06 | Badge cache refresh on startup/login/logout/focus | 04 | pager · lib · `p6_provider_auth` + `p6_refresh` | pass | ✅ |
| MOD-06 | BYOK suppresses needs-login badge | 04 | pager · lib · `p6_needs_login` (incl. `p6_needs_login_byok_*`) | pass | ✅ |
| MOD-06 | Settings DynamicEnum same needs-login badge | 04 | pager · lib · `p6_needs_login` (settings DynamicEnum cases) | pass | ✅ |
| MOD-06 | External CLI FocusGained / poll emission + generation cancel | 03 | pager · lib · `p6_focus_gained` + `p6_refresh_generation` | pass | ✅ |
| MOD-06 | Deferred carries persist_default | 02 / 03 | pager · lib · `p6_deferred` + `p6_transactional_default` | pass | ✅ |
| MOD-03 | Successful switch updates next sample route (pure) | 05 / Phase 4 | shell · `provider_routing` · `switch_changes_next_sample_route` (no p6_ required) | pass | ✅ |
| MOD-03 | Dual-login free Grok↔GPT switch (apply) | 05 | shell · `model_switch_gate` · `p6_dual_login` | pass | ✅ |
| MOD-03 | Same-provider switch no missing-provider friction | 05 | shell · `model_switch_gate` · `p6_same_provider` | pass | ✅ |
| MOD-03 / D-06 | History preserved + mid-turn non-cancel | 05 | shell · `model_switch_gate` · `p6_history` + `p6_mid_turn` | pass | ✅ |
| MOD-06 / D-07 | IncompatibleAgent path not collapsed | 02 / 06 | pager · lib · `incompatible_agent` (non-p6_ intentional) | pass | ✅ |

### Landed filter inventory (representative names)

**Shell `model_switch_gate`:**
- `p6_missing_provider_apply_blocks_codex_when_codex_slot_empty`
- `p6_missing_provider_apply_no_side_effects`
- `p6_missing_provider_apply_allows_codex_when_refreshable_token_present`
- `p6_dual_login_free_switch_xai_to_codex_no_missing_provider`
- `p6_dual_login_free_switch_codex_to_xai_no_missing_provider`
- `p6_dual_login_next_sample_uses_target_provider`
- `p6_same_provider_codex_switch_with_usable_creds`
- `p6_byok_model_skips_oauth_slot_gate` / `p6_byok_own_credentials_skips_oauth_missing_provider`
- `p6_history_preserved_across_successful_switch`
- `p6_mid_turn_switch_does_not_cancel_inflight`

**Shell unit / lib:**
- `p6_model_switch_missing_provider_error_round_trips_acp_data`
- `p6_model_switch_missing_provider_error_message_includes_cli_suggestion`
- `p6_provider_slot_usable_*`, `p6_auth_meta_*`, `p6_missing_provider_gate_error_decision_table`

**Pager:**
- `p6_switch_model_complete_missing_provider_opens_question_view` (+ siblings under `p6_missing_provider`)
- `p6_transactional_default_model_*`
- `p6_keep_current_*`
- `p6_login_now_*`, `p6_deferred_*`, `p6_auth_complete_*`
- `p6_external_cli_status_refresh_applies_deferred`
- `p6_focus_gained_while_awaiting_emits_refresh`
- `p6_refresh_generation_stale_ignored`
- `p6_needs_login_*`, `p6_provider_auth_*`, `p6_refresh_provider_auth_*`
- regression: `switch_model_incompatible_agent_shows_question_modal`, `incompatible_agent_*`

### ROADMAP success criteria map

| Criterion (ROADMAP Phase 6) | Proof |
|-----------------------------|--------|
| 1. Switch mid-session anytime; next turn uses new model | Plan 05 `p6_dual_login` + `p6_dual_login_next_sample_uses_target_provider`; pure `switch_changes_next_sample_route`; Plan 03 deferred apply on success |
| 2. Missing credentials block switch + login prompt (no silent mid-turn 401 primary UX) | Plans 01–03: shell `p6_missing_provider` + pager QuestionView + deferred/external CLI (`p6_login_now` / `p6_auth_` / `p6_external_cli`) |
| 3. Both providers logged in → free Grok↔GPT in one continuous session | Plan 05 `p6_dual_login` (+ `p6_same_provider`, `p6_history`, `p6_mid_turn`) |

### Explicit exclusions

- **Phase 7:** cross-provider subagent spawn credentials / parent-child orchestration
- **Phase 8:** full rebrand chrome/help strings, quiet fork polish
- **Live OAuth browser smoke** — optional manual only; not required for phase gate
- Stock credential import from `~/.codex` / `~/.grok`
- Unfiltered `cargo test -p xai-grok-shell --lib` as a gate command

---

## Sampling Continuity

| Wave | Plans | Per-task verify | Sampling rule |
|------|-------|-----------------|---------------|
| 1 | 01 | shell unit + `model_switch_gate p6_missing_provider` | 2/2 automated |
| 2 | 02, 05 | pager `p6_missing_provider` / `p6_transactional_default`; shell `p6_dual_login` / `p6_history` / `p6_mid_turn` | ≥2/2 per plan |
| 3 | 04 | `p6_provider_auth` / `p6_refresh` / `p6_needs_login` (slash + settings) | 3/3 automated |
| 4 | 03 | `p6_login_now` / `p6_auth_` / `p6_external_cli` / `p6_focus_gained` / `p6_refresh_generation` | 2/2 automated |
| 5 | 06 | VALIDATION present + **per-subgroup** cargo gate with discovery asserts | docs + green filters |

> **Wave alignment (review cycle 2):** Plan 02 and 05 are **wave 2** (parallel). Plan 04 is **wave 3** (depends on 02 — sole owner of RefreshProviderAuthStatus after SwitchModel Effect lands). Plan 03 is **wave 4** (depends on 02 + 04). Plan 06 is **wave 5**.

---

## Wave 0 gaps (closed — Plans 01–05 greened)

- [x] Shell unit: pure provider_slot_usable + ModelSwitchMissingProviderError serde (Plan 01)
- [x] Shell integration: apply returns missing-provider + no side effects (Plan 01 — `model_switch_gate`)
- [x] Pager: SwitchModelError::MissingProvider + transactional default + QuestionView (Plan 02)
- [x] Pager: AuthMeta dual-slot + lifecycle refresh (startup/login/logout/focus) + badge + settings DynamicEnum + BYOK (Plan 04)
- [x] Shell: dual free switch + named history/mid-turn session proofs (Plan 05)
- [x] Pager: deferred (persist_default) + AuthComplete + external CLI poll/FocusGained + generation cancel (Plan 03)
- [x] IncompatibleAgent regression still green (Plan 02 / 06 gate)

---

## Required subgroups (each must discover ≥1)

### Shell

| Subgroup filter | Target | Notes |
|-----------------|--------|-------|
| `p6_missing_provider` | `--test model_switch_gate` | D-01 apply gate |
| `p6_dual_login` | `--test model_switch_gate` | MOD-03 free dual switch |
| `p6_same_provider` | `--test model_switch_gate` | present (Sol→Terra Codex) |
| `p6_byok` | `--test model_switch_gate` | OAuth-slot skip |
| `p6_history` | `--test model_switch_gate` | D-06 history preserve |
| `p6_mid_turn` | `--test model_switch_gate` | D-06 mid-turn non-cancel |
| `p6_model_switch_missing_provider` | `--lib` (not bare package list) | typed error ACP |
| `switch_changes_next_sample_route` | `--test provider_routing` | pure routing; no p6_ required |

### Pager (`--lib` target for every row)

| Subgroup filter | Notes |
|-----------------|-------|
| `p6_missing_provider` | QuestionView MissingProvider |
| `p6_transactional_default` | D-05 transactional default |
| `p6_keep_current` | dismiss path |
| `p6_login_now` | D-03 Login now |
| `p6_deferred` | persist_default carry |
| `p6_auth_` | AuthComplete apply (note trailing `_` — unique prefix) |
| `p6_external_cli` | external CLI refresh apply |
| `p6_focus_gained` | FocusGained while awaiting |
| `p6_refresh_generation` | stale generation cancel |
| `p6_needs_login` | D-04 badge / full catalog |
| `p6_provider_auth` | dual-slot cache + lifecycle |
| `p6_refresh` | refresh effect / status (lifecycle siblings) |
| `incompatible_agent` | D-07 non-collapse (pre-existing; discovery ≥1) |

---

## Phase gate aggregate (Plan 06)

Canonical runnable sequence — also recorded with pass timestamp in `06-PHASE-GATE.md`:

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

# --- Shell ---
discover xai-grok-shell p6_missing_provider --test model_switch_gate
discover xai-grok-shell p6_dual_login --test model_switch_gate
discover xai-grok-shell p6_same_provider --test model_switch_gate
discover xai-grok-shell p6_byok --test model_switch_gate
discover xai-grok-shell p6_history --test model_switch_gate
discover xai-grok-shell p6_mid_turn --test model_switch_gate
# --lib: bare package --list fails on unrelated signed_managed_config* compile
discover xai-grok-shell p6_model_switch_missing_provider --lib
cargo test -p xai-grok-shell --test provider_routing switch_changes_next_sample_route -- --nocapture

# --- Pager (--lib: bare package --list fails on pre-existing settings_e2e syntax error) ---
discover xai-grok-pager p6_missing_provider --lib
discover xai-grok-pager p6_transactional_default --lib
discover xai-grok-pager p6_keep_current --lib
discover xai-grok-pager p6_login_now --lib
discover xai-grok-pager p6_deferred --lib
discover xai-grok-pager p6_auth_ --lib
discover xai-grok-pager p6_external_cli --lib
discover xai-grok-pager p6_focus_gained --lib
discover xai-grok-pager p6_refresh_generation --lib
discover xai-grok-pager p6_needs_login --lib
discover xai-grok-pager p6_provider_auth --lib
discover xai-grok-pager p6_refresh --lib
discover xai-grok-pager incompatible_agent --lib
```

> Do **not** use bare filters that already match unrelated tests as the only Phase 6 proof
> (`auth_complete`, bare `needs_login` without `p6_`, `deferred_model_switch`).
>
> **Pass record:** see `06-PHASE-GATE.md` (`gate_passed: 2026-07-17T00:17:46Z`, all subgroups green).

---

## Security notes (gate-related)

| Threat | Mitigation proven by |
|--------|----------------------|
| Silent mid-turn 401 as primary UX | Switch-time shell gate (`p6_missing_provider` filters) |
| Client-spoofed provider on error | Provider from catalog `ModelEntry.info.provider` only; pager parses xai\|codex only |
| Token leak in error/modal | Suggestion is CLI only; structured fields; sanitize Other |
| Optimistic default persist | `p6_transactional_default` |
| Cross-slot credential use | Phase 4 route + `p6_dual_login` asserts target slot |
| Empty filter false green | Discovery assert ≥1 **per** required subgroup (not aggregate p6_) |
| Live secrets in CI | Fixture-only; PHASE-GATE forbids live token env requirements (T-06-17) |
| Skipping shell or pager half | Both halves required in aggregate (T-06-18) |
