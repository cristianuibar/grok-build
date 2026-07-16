---
phase: 02-multi-slot-credentials-xai-oauth
verified: 2026-07-16T07:59:44Z
status: human_needed
score: 2/3 must-haves verified
behavior_unverified: 1
overrides_applied: 0
mvp_note: "ROADMAP marks mode: mvp but phase goal is not user-story format (user-story.validate=false). Verified against ROADMAP success criteria + plan must_haves; User Flow Coverage derived from SC outcome clauses."
behavior_unverified_items:
  - truth: "User can complete xAI OAuth (browser and/or device-code) with credentials stored only under the bum auth store"
    test: "Run `bum login --oauth` and/or `bum login --device-auth` against real xAI IdP with a temp BUM_HOME (or real ~/.bum smoke); optionally confirm mock unit tests when shell --lib harness is repaired"
    expected: "Login succeeds; $BUM_HOME/auth.json (or product-home/auth.json) is nested {version:1, providers:{xai:…}}; no write to ~/.grok/auth.json or foreign GROK_AUTH_PATH"
    why_human: "Live IdP OAuth cannot be exercised without network/browser/device-code UX. In-tree mock browser + device-code tests assert nested multi-slot shape but cannot run via cargo test --lib until pre-existing shell test harness is fixed (WorkspaceOps::for_test et al.)"
human_verification:
  - test: "Live xAI OAuth smoke under isolated product home"
    expected: "After `BUM_HOME=<tmpdir> bum login --oauth` (or --device-auth), credentials land only under that home’s auth.json as nested providers.xai; agent turn can call xAI with that credential"
    why_human: "Requires real IdP / browser or device-code interaction; optional per phase CONTEXT but remaining SC1 runtime proof"
  - test: "(Optional) Re-run in-tree auth unit suites after harness repair"
    expected: "cargo test -p xai-grok-shell --lib auth::storage::, auth::manager_tests multi-slot filters, auth::credential_provider nested_*, auth::device_code mock, auth::oidc login mock all pass"
    why_human: "lib-test binary does not compile today (Phase 1 deferred harness debt); source tests are present and substantive"
---

# Phase 2: Multi-slot credentials & xAI OAuth — Verification Report

**Phase Goal:** Auth storage is provider-scoped so dual OAuth is safe; xAI login still works end-to-end under bum  
**Requirement:** AUTH-01  
**Verified:** 2026-07-16T07:59:44Z  
**Status:** human_needed  
**Re-verification:** No — initial verification  

**MVP mode note:** ROADMAP lists `mode: mvp` but the phase goal is not in User Story form (`As a …, I want to …, so that ….`). Verification proceeded against the three ROADMAP success criteria (the contract). User Flow Coverage below maps those outcomes to evidence.

## Goal Achievement

### User Flow Coverage

User story (derived from goal + AUTH-01 / success criteria):  
«As a bum user, I want multi-slot auth storage and working xAI OAuth under the bum auth store, so that dual OAuth is safe and xAI login still works end-to-end.»

| Step | Expected | Evidence | Status |
|------|----------|----------|--------|
| Open login | `bum login` defaults to xAI; `--oauth` / `--device-auth` available; no required `--provider` | `cli.rs` `Command::Login` + `bum_login_defaults_to_xai_without_provider_argument`; pager-bin wires `run_cli_login` | ✓ code |
| Complete xAI OAuth | Browser and/or device-code login persist credentials under product home only | Production flows call `AuthManager::update` → `write_auth_json` (nested `AuthDocument`); mock IdP tests assert `providers.xai` + codex survival; `resolve_auth_path` rejects foreign `GROK_AUTH_PATH` | ⚠️ wired; live OAuth / mock suite not run this session |
| Multi-slot store | Nested `providers.xai` + reserved `providers.codex` without clobber | `AuthDocument` + `apply_xai_slot`; storage/manager tests (source); integration `auth_multi_slot` codex survival | ✓ |
| Agent turn after login | Nested store → AuthManager token → Bearer on xAI path | `auth_multi_slot` green; `ShellAuthCredentialProvider` + `nested_auth_seeds_sampling_api_key_and_sampler_bearer` (source); agent config seeds `SamplerConfig.api_key` | ✓ load proven; wire test unrunnable via --lib |
| Outcome | Dual-OAuth-safe storage; xAI login E2E under bum | SC2+SC3 verified in code/integration; SC1 needs live smoke or runnable mock suite | ⚠️ |

### Observable Truths (ROADMAP success criteria)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | User can complete xAI OAuth (browser and/or device-code) with credentials stored only under the bum auth store | ⚠️ PRESENT_BEHAVIOR_UNVERIFIED | **Wired:** `run_cli_login` ← pager-bin Login; device-code docs `$BUM_HOME/auth.json`; OIDC login + device-code mock tests assert nested `version`/`providers.xai` and seeded `providers.codex` survival (`device_code.rs` ~863–887, `oidc/login.rs` ~673–688). **Isolation:** `resolve_auth_path` accepts only exact product-home `auth.json` (`manager.rs` 276–292); tests `foreign_grok_auth_path_cannot_receive_manager_writes`, `in_home_auth_symlink_cannot_escape_product_home` (source). **Not run this session:** live IdP OAuth; mock login unit tests blocked by `--lib` harness. |
| 2 | Auth store is structured for multiple provider slots (xAI first; second slot reserved without overwriting xAI) | ✓ VERIFIED | `PROVIDER_XAI`/`PROVIDER_CODEX`/`AUTH_DOCUMENT_VERSION=1`/`AuthDocument` (`model.rs`); read migration legacy→xAI; `apply_xai_slot` only inserts xAI and preserves siblings (`storage.rs`); dual mutate APIs; prune deletes file only when all slots empty; `try_devbox_recovery` → `replace_xai_after_devbox_mint` clears only xAI. Integration: `auth_multi_slot` loads xAI from nested doc and leaves codex key intact (**passed**). |
| 3 | After xAI login, user can start an agent turn that authenticates successfully against the xAI path | ✓ VERIFIED | Public path: nested auth.json → `AuthManager::new`/`auth()`/`get_valid_token` (`auth_multi_slot` **ok**). Bearer seam: `ShellAuthCredentialProvider` applies `Authorization: Bearer …` from multi-slot-backed manager (source tests); agent/sampler still seed `SamplerConfig.api_key` from AuthManager token (`credential_provider.rs` nested_* tests; `agent/config.rs` sampling). Multi-slot is transparent via `read_auth_json` xAI adapter. |

**Score:** 2/3 truths verified (1 present, behavior-unverified)

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `crates/codegen/xai-grok-shell/src/auth/model.rs` | AuthDocument + provider constants | ✓ VERIFIED | Exists, substantive, used by storage/manager |
| `crates/codegen/xai-grok-shell/src/auth/storage.rs` | Multi-slot read/mutate/prune + API key | ✓ VERIFIED | Dual APIs, atomic write, StorageFull fallback, fixture helper |
| `crates/codegen/xai-grok-shell/src/auth/manager.rs` | Guard-held writers + path isolation + recovery | ✓ VERIFIED | `write_auth_json_with_lock`, `write_scope_removal`, `resolve_auth_path`, `try_devbox_recovery` |
| `crates/codegen/xai-grok-shell/src/auth/manager/enrichment.rs` | Locked vs acquiring write | ✓ VERIFIED | with-lock vs acquiring on timeout |
| `crates/codegen/xai-grok-shell/src/auth/credential_provider.rs` | Multi-slot → Bearer | ✓ VERIFIED | Nested manager tests + sampling Bearer test present |
| `crates/codegen/xai-grok-shell/src/auth/manager_tests.rs` | Multi-slot manager/path tests | ✓ VERIFIED | Nested load/update, path isolation, codex-preserving logout/recovery |
| `crates/codegen/xai-grok-shell/src/auth/device_code.rs` | Mock multi-slot assert + bum path docs | ✓ VERIFIED | Nested assert + `$BUM_HOME/auth.json` docs |
| `crates/codegen/xai-grok-shell/src/auth/oidc/login.rs` | Browser mock multi-slot assert | ✓ VERIFIED | Nested assert + codex survival |
| `crates/codegen/xai-grok-pager/src/app/cli.rs` | Login flags without provider | ✓ VERIFIED | oauth/device_auth only; unit test present |
| `crates/codegen/xai-grok-shell/tests/auth_multi_slot.rs` | Public integration proof | ✓ VERIFIED | **Runs green** |
| `crates/codegen/xai-grok-shell/src/util/grok_auth_credentials.rs` | AuthManager → Bearer helper | ✓ VERIFIED | Live `get_valid_token` path |

gsd-tools `verify.artifacts`: all plan artifacts **passed** (13/13 across 02-01…02-04).

### Key Link Verification

| From | To | Via | Status | Details |
|------|-----|-----|--------|---------|
| `mutate_auth_document` | `AuthDocument.providers` | acquire lock → still_live → RMW → write | ✓ WIRED | `storage.rs` 281–311 |
| `mutate_auth_document_with_lock` | `AuthDocument.providers` | still_live only; no re-acquire | ✓ WIRED | same |
| Manager lock-holding writers | `write_auth_json_with_lock` / prune with-lock | startup cleanup, scope removal, update/save, enrichment | ✓ WIRED | `manager.rs` 352, 528, 932; `enrichment.rs` 198–200 |
| Unlocked writers | acquiring mutate / write | store_api_key, enrichment timeout, write_auth_json | ✓ WIRED | `storage.rs` store_api_key; enrichment `None` branch |
| `read_auth_json` | AuthManager / API key | xAI slot adapter only | ✓ WIRED | returns `providers.xai` map |
| pager-bin `Command::Login` | `run_cli_login` | oauth/device_auth flags | ✓ WIRED | `main.rs` ~1739–1751 |
| Nested auth.json | sampling Bearer | AuthManager → api_key / ShellAuthCredentialProvider | ✓ WIRED | integration load + production sampler/credential paths |
| GROK_AUTH_PATH | product-home auth.json | exact path equality (rejects foreign + non-exact in-home symlink) | ✓ WIRED | Stricter than plan’s optional dunce parent-containment; isolation intent met |

Note: plan `key_links.from` used symbolic names (not file paths), so automated `verify.key-links` reported false negatives; manual wiring verification above.

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|--------------------|--------|
| AuthManager | in-memory `GrokAuth` / token key | `read_auth_json` → xAI slot from nested or legacy disk | Yes (disk scopes / OAuth update) | ✓ FLOWING |
| ShellAuthCredentialProvider | Bearer token | AuthManager current/snapshot | Yes when credential present | ✓ FLOWING |
| SamplerConfig | `api_key` | AuthManager `get_valid_token` / resolve credentials | Yes on agent turn seed path | ✓ FLOWING |
| auth.json on disk | `providers` map | `write_auth_document` after xAI slot mutate | Yes nested JSON | ✓ FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| Nested multi-slot load via public AuthManager | `cargo test -p xai-grok-shell --test auth_multi_slot` | 1 passed | ✓ PASS |
| Shell lib compiles | `cargo check -p xai-grok-shell --lib` | Finished ok | ✓ PASS |
| bum binary compiles | `cargo check -p xai-grok-pager-bin --bin bum` | Finished ok | ✓ PASS |
| Shell lib unit suite (auth::) | `cargo test -p xai-grok-shell --lib …` | compile fail: `WorkspaceOps::for_test` et al. (pre-existing) | ? SKIP (known constraint) |
| Live xAI OAuth | interactive login | not executed | ? SKIP → human |

### Probe Execution

| Probe | Command | Result | Status |
|-------|---------|--------|--------|
| — | — | No phase-declared `scripts/*/tests/probe-*.sh` | SKIP |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| AUTH-01 | 02-01…02-04 | xAI OAuth login with credentials only under bum auth store | ⚠️ PARTIAL → human_needed | Multi-slot store + isolation + agent credential path verified; live OAuth not executed; mock OAuth unit tests present but unrunnable via --lib |

No orphaned Phase 2 requirements beyond AUTH-01.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| — | — | No TBD/FIXME/XXX debt markers in phase-touched production logic | — | oidc login.rs “code=XXX” is docstring example only |
| shell --lib harness | various | Pre-existing compile breaks (`WorkspaceOps::for_test`, `MemoryStorage::with_paths`, `EnvVarGuard`, …) | ℹ️ Info | Blocks in-tree unit suite; not introduced by Phase 2; Phase 1 deferred |
| Plan 02-04 “phase gate green: auth:: suite” | — | Full filter suite cannot run | ⚠️ Warning | Pragmatic gate (check + auth_multi_slot) used instead — consistent with known harness debt |

### Implementation notes (not gaps)

1. **GROK_AUTH_PATH:** Plan text allowed exact product path **or** dunce-canonical parent containment. Implementation uses **exact equality only** — more restrictive, still rejects foreign paths and in-home symlink overrides that are not the exact product auth path. Intent preserved.
2. **Codex slot:** Reserved schema/constants; no Codex OAuth (correctly out of scope → Phase 5).
3. **AuthDocument** is `pub(crate)`; public `read_auth_json` still returns xAI `AuthStore` only.

### Human Verification Required

#### 1. Live xAI OAuth smoke under isolated product home

**Test:** With a clean temp product home, run `BUM_HOME=/tmp/bum-auth-smoke bum login --oauth` (and/or `--device-auth`). Complete IdP flow. Inspect `$BUM_HOME/auth.json`. Optionally start an agent turn against xAI.

**Expected:** Nested document `version: 1`, `providers.xai` holds OIDC/API scopes; no write under `~/.grok` or a foreign `GROK_AUTH_PATH`; agent turn authenticates.

**Why human:** Real IdP/browser/device-code UX and network.

#### 2. (Optional) Unit suite after harness repair

**Test:** Fix shell/pager lib-test harness (Phase 1 debt), then run auth multi-slot, path isolation, mock login, and credential_provider nested Bearer tests.

**Expected:** All listed tests pass.

**Why human / follow-up:** Not phase-goal-blocking given integration + cargo check + code review; strengthens regression safety.

### Gaps Summary

No **BLOCKER** gaps against the phase goal in the codebase: multi-slot store, non-clobber writes, product-home isolation, and xAI agent credential path are implemented and partially proven by a green integration test plus successful compile checks.

Residual status is **human_needed** solely because SC1’s full OAuth completion is a runtime behavior not exercised live in this verification, and the bulk of mock/unit behavioral tests cannot execute until the pre-existing shell `--lib` harness is repaired.

---

_Verified: 2026-07-16T07:59:44Z_  
_Verifier: Claude (gsd-verifier)_
