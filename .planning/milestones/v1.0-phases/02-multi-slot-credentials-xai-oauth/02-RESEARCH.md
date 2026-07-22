# Phase 2: Multi-slot credentials & xAI OAuth - Research

**Researched:** 2026-07-16
**Domain:** Brownfield multi-provider auth store + xAI OAuth under isolated `~/.bum`
**Confidence:** HIGH

## Summary

Phase 2 is a **schema + write-path evolution** of the existing shell auth stack, not a new OAuth implementation. xAI browser OAuth, device-code, refresh, lock, and `AuthManager` already work end-to-end; Phase 1 already points product state (including `auth.json`) at `$BUM_HOME` / `~/.bum`. The missing piece for dual OAuth safety is **provider-scoped storage**: nest today’s flat scope → `GrokAuth` map under `providers.xai`, reserve `providers.codex` (empty/absent), and ensure every write is a **read-merge-write** that cannot clobber sibling providers. [VERIFIED: codebase]

Runtime for this phase stays **xAI-default**: a single `AuthManager` continues to resolve the xAI issuer scope / API key and feed sampler HTTP via `ShellAuthCredentialProvider` / `GrokAuthCredentials`. Codex login, dual logout/status, and provider routing are out of scope (Phases 4–5). Success is proven with **automated unit tests** (multi-slot isolation + xAI credential path), not live browser OAuth in CI. [VERIFIED: codebase + CONTEXT]

**Primary recommendation:** Implement multi-slot **in `auth/storage.rs` first** (document type + legacy read migration + merge-safe writes), keep `AuthManager` mostly storage-agnostic via “xAI slot as `AuthStore`” adapters, leave OAuth flows/device-code/OIDC refresh untouched, and extend existing `manager_tests` / storage tests with clobber-isolation + credential-supply fixtures under temp `BUM_HOME`.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
#### Multi-slot store schema
- **Single `auth.json`** with a top-level **provider map** (`providers.xai`, reserved `providers.codex`) — one file, existing `auth.json.lock`, atomic write path
- Stable short provider ids: **`xai`** and **`codex`** (ChatGPT OAuth will live under `codex` in Phase 5)
- Nest today’s xAI scopes (`issuer` keys, `xai::api_key`) under **`providers.xai`**
- **Read migration:** if only the legacy flat scope map is present under bum home, treat it as the xAI slot (no import from `~/.grok`)
- Codex slot is **schema + empty/absent OK** this phase — write paths must not clobber other slots; no Codex login

#### xAI OAuth surface (AUTH-01)
- **Browser OAuth + device-code** must both work, equivalent to stock Grok Build (CLI + existing TUI paths)
- **`bum login` defaults to xAI** — no required provider argument yet; keep `--oauth` / `--device-auth` (and aliases)
- Credentials stored **only** under bum product home (`$BUM_HOME/auth.json` / `~/.bum/auth.json`) — never `~/.grok` or Codex paths
- Keep **xAI API-key fallback** working under `providers.xai` (OAuth remains primary)

#### Isolation, safety & AuthManager scope
- Reuse existing **advisory lock** (`auth.json.lock`) and **crash-safe atomic write** (with in-place fallback) for the multi-slot store
- **Slot isolation:** updating one provider must never wipe or overwrite the other provider’s credentials
- **Runtime default remains xAI** this phase — multi-slot is under the store; full dual AuthManager/routing is later (Phases 4–5)
- **No import** from stock `~/.grok` or Codex credential stores (PROJECT / Phase 1 locked)

#### Verification & phase boundary
- Prove success with **automated tests**: multi-slot structure, xAI write/read without clobber, AuthManager supplies xAI credential for a request/auth path (mock/fixture IdP where needed). Live browser OAuth optional manual smoke only
- Multi-provider **logout / status** → Phase 5 (AUTH-03/04); xAI-only paths may keep working if already present
- **No Codex OAuth or ChatGPT tokens** in this phase — reservation only
- User-facing rebrand of login chrome/strings → **Phase 8**, except fixes required so paths/CLI name are not wrong for bum

### Claude's Discretion
- Exact JSON schema versioning / serde shape for `providers` map (field names, optional envelope version)
- Whether legacy flat map is rewritten on first successful write (eager migrate) vs read-only migration until next write
- How deep to thread provider-aware types into `AuthManager` vs storage-only adapter for this phase
- Test layout (unit vs integration) and any minimal fixture IdP for OAuth write path

### Deferred Ideas (OUT OF SCOPE)
- Codex/ChatGPT OAuth + dual logout/status/refresh independence → Phase 5
- Provider-aware request routing → Phase 4
- Mixed model catalog / GPT-5.6 entries → Phase 3
- Missing-provider gate on model switch → Phase 6
- Full product string rebrand for auth chrome → Phase 8
- Import from `~/.grok` / stock Codex stores → out of v1 (AUTH-V2-02)
- Multiple OAuth accounts per provider → AUTH-V2-03
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| AUTH-01 | User can log in to xAI via OAuth (browser and/or device-code), equivalent to original Grok Build, with credentials stored only under the bum auth store | Existing `run_cli_login` / `run_auth_flow` / device-code + OIDC under `xai-grok-shell/src/auth/`; store under `$BUM_HOME/auth.json` via `AuthManager::new` + multi-slot `providers.xai`; automated tests for store + credential path |
</phase_requirements>

## Project Constraints (from AGENTS.md / PROJECT.md)

| Directive | Implication for Phase 2 |
|-----------|-------------------------|
| Stay on Rust workspace (edition 2024, Tokio, existing harness) | No new agent framework; evolve shell auth only [VERIFIED: AGENTS.md] |
| Identity: xAI OAuth preserved; ChatGPT OAuth later | This phase is xAI-only live auth [VERIFIED: PROJECT.md] |
| Storage: `~/.bum` isolation; no credential sharing with stock CLIs | No `~/.grok` / `~/.codex` import or dual-read [VERIFIED: CONTEXT + Phase 1] |
| Prefer unit tests with `BUM_HOME` temp sandboxes | Follow Phase 1 test-support patterns [VERIFIED: STATE.md decisions] |
| Root `Cargo.toml` is generated — edit per-crate manifests | No new workspace dep unless regenerating [VERIFIED: AGENTS.md] |
| Prefer per-crate `cargo test -p xai-grok-shell` | Full workspace tests are heavy [VERIFIED: TESTING.md] |

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Multi-slot `auth.json` schema + migration | Shell auth storage (`auth/storage.rs`, `auth/model.rs`) | AuthManager writers | File format ownership; all writers must go through merge-safe API |
| xAI OAuth browser (loopback PKCE) | Shell auth OIDC (`auth/oidc/`, `auth/flow.rs`) | Pager TUI login effects | Existing flow; no provider argument this phase |
| xAI device-code | Shell auth (`auth/device_code.rs`) | CLI `bum login --device-auth`, pager device UI | RFC 8628 against `auth.x.ai`; already complete |
| Credential refresh / lock | AuthManager + `auth/manager/lock.rs` | OIDC refresher | File lock + refresh_chain already multi-process safe for one scope map |
| Credential → HTTP (agent turn) | Shell `GrokAuthCredentials` / `ShellAuthCredentialProvider` | Sampler / file-utils / OTEL | `AuthManager::get_valid_token` / `current_or_expired` → Bearer |
| Auth path resolution | Product home SoT (`xai-grok-config::grok_home`) | `GROK_AUTH_PATH` / `GROK_AUTH` overrides | Default `home/auth.json`; home is `BUM_HOME` only after Phase 1 |
| Login CLI entry | `xai-grok-pager-bin` → `Command::Login` | `run_cli_login` | Defaults to xAI; flags `--oauth` / `--device-auth` |
| Reserved Codex slot | Storage schema only | — | Empty/absent; no login, no AuthManager dual runtime |

## Standard Stack

> **No new third-party packages.** Reuse existing auth stack. [VERIFIED: codebase]

### Core

| Library / surface | Version / pin | Purpose | Why standard |
|-------------------|---------------|---------|--------------|
| Rust toolchain | **1.92.0** (`rust-toolchain.toml`) | Build | Workspace pin [VERIFIED: codebase] |
| `xai-grok-shell` auth module | path crate | OAuth, store, AuthManager | Full production auth surface [VERIFIED: codebase] |
| `xai-grok-auth` | path crate | `HttpAuth` / `AuthCredentialProvider` DI seam | Sampler/storage consumers [VERIFIED: codebase] |
| `xai-grok-config` | path crate | `grok_home()` / `BUM_HOME` SoT | Auth file parent directory [VERIFIED: codebase] |
| `oauth2` | **5** (shell Cargo.toml) | OAuth2/PKCE types | Already used by shell OIDC [VERIFIED: codebase] |
| `serde` / `serde_json` | workspace | `auth.json` (de)serialize | Existing pretty write + fsync [VERIFIED: codebase] |
| `axum` | workspace | Loopback OAuth callback server | Existing browser flow [VERIFIED: codebase] |
| Tokio | workspace | Async login / refresh | Existing runtime [VERIFIED: codebase] |

### Supporting

| Library / surface | Purpose | When to use |
|-------------------|---------|-------------|
| `tempfile` + `serial_test` | Temp home / env isolation | Multi-slot unit tests (Phase 1 pattern) |
| `xai-grok-test-support` | `EnvGuard`, `BUM_HOME` sandboxes | Process-level tests if needed |
| `wiremock` / axum test servers | Fixture IdP | Only if testing HTTP OAuth write path beyond AuthManager fixtures |
| `chrono` | `expires_at` / TTL | Existing `GrokAuth` fields |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Nested `providers.{xai,codex}` (LOCKED) | Flat multi-scope map with `openai::chatgpt` key (earlier STACK research) | Flat scopes cannot isolate logout/refresh of whole providers cleanly; **CONTEXT supersedes STACK.md** for layout |
| Single `auth.json` | Per-provider files (`xai.json`, `codex.json`) | Extra lock/race surface; CONTEXT locks one file |
| Dual AuthManager now | Storage-only multi-slot + single xAI manager | Dual runtime is Phase 4–5; over-scoping risks regressions |
| Import `~/.grok/auth.json` | Clean re-login under bum | Forbidden by PROJECT/v1 isolation |
| New OAuth crate | Existing in-tree OIDC + device-code | Already production-hardened |

**Installation:** none (no new crates).

**Version verification:** `oauth2 = "5"` in `xai-grok-shell/Cargo.toml`; no registry installs required. [VERIFIED: codebase]

## Package Legitimacy Audit

> Phase installs **no** external packages.

| Package | Registry | Age | Downloads | Source Repo | Verdict | Disposition |
|---------|----------|-----|-----------|-------------|---------|-------------|
| — | — | — | — | — | — | N/A — no new installs |

**Packages removed due to [SLOP] verdict:** none  
**Packages flagged as suspicious [SUS]:** none

## Architecture Patterns

### System Architecture Diagram

```text
┌─────────────┐   bum login [--oauth|--device-auth]
│  pager-bin  │──────────────────────────────────────┐
│  Command::  │                                      │
│  Login      │                                      ▼
└──────┬──────┘                            ┌──────────────────┐
       │ TUI Action::Login                 │ run_cli_login /  │
       ▼                                   │ run_auth_flow    │
┌─────────────┐                            │ (flow.rs)        │
│  Pager TUI  │── ACP / local ────────────►│                  │
└─────────────┘                            └────────┬─────────┘
                                                    │
                     ┌──────────────────────────────┼──────────────────────────┐
                     │                              ▼                          │
                     │                   ┌─────────────────────┐               │
                     │                   │ Browser OIDC/PKCE   │               │
                     │                   │ (oidc/)  OR         │               │
                     │                   │ Device-code         │               │
                     │                   │ (device_code.rs)    │               │
                     │                   │ → auth.x.ai         │               │
                     │                   └──────────┬──────────┘               │
                     │                              │ GrokAuth                 │
                     │                              ▼                          │
                     │                   ┌─────────────────────┐               │
                     │                   │ AuthManager         │               │
                     │                   │ scope = xAI issuer  │               │
                     │                   │   ::client_id       │               │
                     │                   │ update / refresh_   │               │
                     │                   │ chain / get_valid_  │               │
                     │                   │ token               │               │
                     │                   └──────────┬──────────┘               │
                     │                              │                          │
                     │                              ▼                          │
                     │                   ┌─────────────────────┐               │
                     │                   │ storage.rs          │               │
                     │                   │ AuthDocument        │               │
                     │                   │  providers.xai{}    │◄── merge      │
                     │                   │  providers.codex{}  │    never wipe │
                     │                   │ auth.json.lock      │               │
                     │                   │ atomic write 0o600  │               │
                     │                   └──────────┬──────────┘               │
                     │                              │                          │
                     │                              ▼                          │
                     │                   $BUM_HOME/auth.json                   │
                     │                   (~/.bum/auth.json)                    │
                     │                                                         │
                     │   Agent turn path:                                      │
                     │   AuthManager ──► ShellAuthCredentialProvider           │
                     │              ──► GrokAuthCredentials.apply()            │
                     │              ──► Authorization: Bearer + X-XAI-Token-Auth│
                     │              ──► cli-chat-proxy / sampler                │
                     └─────────────────────────────────────────────────────────┘
```

### Recommended Project Structure (touch points only)

```text
crates/codegen/xai-grok-shell/src/auth/
├── model.rs              # AuthStore, GrokAuth, provider ids, AuthDocument types
├── storage.rs            # Multi-slot read/write, migration, lock, API key helpers
├── manager.rs            # Keep single-scope xAI runtime; call storage adapters
├── manager/lock.rs       # Reuse auth.json.lock unchanged
├── flow.rs               # run_cli_login / run_auth_flow (minimal/no change)
├── device_code.rs        # Device login (persist via AuthManager only)
├── config.rs             # auth_scope(), XAI issuer (unchanged)
├── credential_provider.rs# HTTP credential seam (unchanged if manager works)
└── manager_tests.rs      # Extend: multi-slot isolation + credential path

crates/codegen/xai-grok-auth/src/   # Traits only — no multi-slot types required this phase
crates/codegen/xai-grok-pager/src/app/cli.rs  # Login flags (no --provider yet)
crates/codegen/xai-grok-pager-bin/src/main.rs # Command::Login → run_cli_login
```

### Pattern 1: Storage-only multi-slot adapter (recommended discretion)

**What:** Introduce an on-disk `AuthDocument` with `providers`, while keeping in-memory/runtime `AuthStore = BTreeMap<String, GrokAuth>` as the **xAI provider slot**. `read_auth_json` returns the xAI slot (after migration). `write_auth_json` **reloads the full document**, replaces only `providers.xai`, preserves `providers.codex`, writes nested form.  
**When to use:** Phase 2 (CONTEXT: runtime default remains xAI; dual AuthManager later).  
**Why:** Minimizes blast radius across ~100 manager tests and callers (`managed_config`, relay, app reloader) that already treat `AuthStore` as “all scopes for the active product auth”. [VERIFIED: codebase call sites]

```rust
// Source: recommended pattern for this phase (implements CONTEXT decisions)
// Provider ids — stable wire keys
pub const PROVIDER_XAI: &str = "xai";
pub const PROVIDER_CODEX: &str = "codex";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuthDocument {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<u32>,
    #[serde(default)]
    pub providers: BTreeMap<String, AuthStore>,
}

// Read path: nested | legacy flat → AuthDocument, then expose providers.xai as AuthStore
// Write path: load AuthDocument → set providers[xai] = store → write nested JSON
```

### Pattern 2: Existing AuthManager update (scope merge within one map)

**What:** Today `AuthManager::update` already read-merge-writes a single scope into a flat map. [VERIFIED: codebase `manager.rs` ~794–848]  
**When to use:** Unchanged control flow after storage adapter lands — insert still targets `self.scope` inside the xAI slot.  
**Critical change:** Storage layer must not serialize *only* the xAI map as the whole file without re-merging sibling providers.

### Pattern 3: Auth path resolution (Phase 1 complete)

**What:** `AuthManager::new(grok_home, config)` uses:
1. `GROK_AUTH` inline JSON (`GrokAuth`) — highest priority, read-only in memory  
2. Else `GROK_AUTH_PATH` or `{grok_home}/auth.json`  
`grok_home()` resolves `$BUM_HOME` or `~/.bum` (never `GROK_HOME`). [VERIFIED: codebase `manager.rs` 262–298, `paths.rs`]

### Pattern 4: Login CLI defaults to xAI

```text
bum login                  → run_cli_login(oauth=false, device_auth=false)
                           → resolves device vs loopback from flags/env/config/remote
bum login --oauth          → force browser/loopback
bum login --device-auth    → force device-code (alias --device-code)
```

No `--provider` flag this phase. [VERIFIED: codebase `cli.rs`, `flow.rs` `run_cli_login`]

### Anti-Patterns to Avoid

- **Whole-file replace with only xAI scopes:** After nesting, `serde_json::to_writer(auth_store)` that ignores other providers wipes Codex once Phase 5 fills it — and fails isolation tests even with a seeded empty/fixture codex slot.
- **Importing `~/.grok/auth.json`:** Violates isolation; CONTEXT forbids.
- **Threading full multi-provider AuthManager now:** Out of scope; increases regression risk on refresh/sleep-gate code (~2k LOC).
- **Deleting `auth.json` when xAI scopes empty but codex has data:** `write_scope_removal` currently deletes file if map empty — must become “delete only if all providers empty”. [VERIFIED: codebase `manager.rs` 482–493]
- **Treating corrupt recovery empty map as license to wipe siblings:** Recovery already backs up corrupt file; multi-slot writes after recovery must still only write intended provider content into a fresh document.
- **Hand-rolling OAuth:** Use existing oidc/device_code paths only.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| OAuth2 / PKCE / device-code | New OAuth client | Existing `auth/oidc/*`, `device_code.rs`, `oauth2` crate | Production edge cases (lock, sleep, family revocation) already handled |
| Atomic credential write | Custom fsync scheme | `write_auth_json` atomic + StorageFull in-place fallback | Disk-full + 0o600 + corrupt backup already tested |
| Cross-process refresh races | Ad-hoc mutex only | `auth.json.lock` + `refresh_chain` + `still_live()` | Sleep/suspend lock break is subtle [VERIFIED: codebase] |
| HTTP credential attachment | Manual header string building in sampler | `GrokAuthCredentials` / `ShellAuthCredentialProvider` | Bearer + `X-XAI-Token-Auth` contract |
| Multi-slot merge | Ad-hoc write in each call site | Central storage API | Many writers: manager, store_api_key, remove_scope, enrichment |

**Key insight:** Phase 2 risk is **data integrity under multi-provider writes**, not OAuth protocol novelty. Concentrate tests and code review on storage merge semantics.

## Runtime State Inventory

> Schema migration of on-disk credentials under product home (not a binary rename).

| Category | Items Found | Action Required |
|----------|-------------|-----------------|
| Stored data | `$BUM_HOME/auth.json` — legacy flat `BTreeMap` scope→`GrokAuth`; after Phase 2 nested `providers` | **Data migration (read):** treat flat map as `providers.xai`. **Write:** emit nested form (eager migrate recommended). No migration of `~/.grok/auth.json` |
| Live service config | None for multi-slot (local file only). OAuth IdP is external `auth.x.ai` — no config change | None |
| OS-registered state | None (no systemd/launchd auth registrations) | None — verified: auth is file + lock file only |
| Secrets/env vars | `BUM_HOME`, `GROK_AUTH`, `GROK_AUTH_PATH`, `XAI_API_KEY`, `GROK_OAUTH2_*`, `GROK_LOCAL_AUTH` | Code continues to honor existing env names this phase (Phase 8 renames); ensure writes never target `GROK_HOME` |
| Build artifacts | None for auth schema | None |

**Nothing found that requires importing or rewriting stock `~/.grok` / `~/.codex` stores.**

## Common Pitfalls

### Pitfall 1: Clobbering sibling provider on xAI write
**What goes wrong:** Codex (or fixture) credentials disappear after xAI login/refresh/API-key store.  
**Why it happens:** Call sites pass a full-file `AuthStore` and overwrite the document.  
**How to avoid:** Single merge-safe write API; unit test “seed codex fixture → update xAI → codex intact”.  
**Warning signs:** Tests that only round-trip xAI keys; `write_auth_json` still `to_writer(auth_store)` without document load.

### Pitfall 2: File delete on “empty xAI” logs out Codex
**What goes wrong:** `remove_scope` / logout deletes entire `auth.json` when xAI map is empty.  
**Why it happens:** Current empty-file deletion assumes one flat map. [VERIFIED: codebase]  
**How to avoid:** Delete file only when **all** providers have no scopes; else write document with remaining providers.

### Pitfall 3: Legacy flat migration only on read, tests write flat forever
**What goes wrong:** On-disk format never stabilizes; Phase 5 assumes nested.  
**Why it happens:** Read-only migration without eager rewrite.  
**How to avoid (discretion):** Eager rewrite nested form on first successful write after load (recommended). Still accept flat on read under bum home only.

### Pitfall 4: Breaking ~100 AuthManager tests by changing `AuthStore` wire type naively
**What goes wrong:** Mass test failures because `read_auth_json` returns a document or nested map.  
**Why it happens:** `AuthStore` type alias used everywhere as scope map.  
**How to avoid:** Keep `AuthStore` as scope map; add `AuthDocument` for on-disk; adapters at storage boundary; update tests that assert raw file JSON shape.

### Pitfall 5: Tests write raw flat JSON bypassing storage API
**What goes wrong:** Relay/helpers use `serde_json::to_string_pretty(&map)` + `fs::write` (e.g. relay tests). [VERIFIED: codebase `agent/relay.rs` test helper]  
**How to avoid:** Prefer `write_auth_json` helper; migration must still accept flat for hermetic fixtures.

### Pitfall 6: Assuming `update()` holds file lock
**What goes wrong:** Concurrent multi-slot writers race.  
**Why it happens:** File lock is mandatory on **refresh** IdP path; login flow takes lock in expired path; plain `update()` does not always hold lock. [VERIFIED: codebase]  
**How to avoid:** For multi-slot, at minimum merge-read before write is atomic at process level via rename; document that Phase 5 dual writers need same lock discipline as refresh. Do not weaken refresh locking.

### Pitfall 7: API key helpers write outside `providers.xai`
**What goes wrong:** `store_api_key` / `read_api_key` / `clear_api_key` target wrong nest.  
**How to avoid:** Route all three through storage adapter; keep scope key `xai::api_key` **inside** xAI provider. [VERIFIED: codebase `API_KEY_SCOPE`]

### Pitfall 8: Path still documents `~/.grok`
**What goes wrong:** Comments/user errors say `~/.grok/auth.json` or `grok login` after Phase 1.  
**How to avoid:** Fix path-wrong messages that would send users to stock home; full chrome rebrand still Phase 8. Device-code module still has a stale `~/.grok/auth.json` doc comment. [VERIFIED: codebase `device_code.rs` L207]

### Pitfall 9: Proving AUTH-01 only with live browser OAuth
**What goes wrong:** CI cannot gate the phase.  
**How to avoid:** Fixture credentials + `AuthManager::update` / `get_valid_token` / `ShellAuthCredentialProvider::snapshot` / `GrokAuthCredentials::apply` assertions; optional manual smoke for real IdP.

## Code Examples

### Key file map (planner checklist)

| File | Role in Phase 2 |
|------|-----------------|
| `auth/model.rs` | `AuthStore`, `GrokAuth`, `API_KEY_SCOPE`, `lookup_auth`; add provider constants + document types |
| `auth/storage.rs` | **Primary change surface** — read/write/migrate/lock/API key |
| `auth/manager.rs` | `new`, `update`, `save_without_enrichment`, `remove_scope`, `read_disk_auth`; mostly keep logic, rely on storage |
| `auth/manager/lock.rs` | Reuse; lock path still sibling of `auth.json` |
| `auth/flow.rs` | `run_cli_login`, `run_auth_flow`, `perform_logout` — expect minimal changes |
| `auth/device_code.rs` | Unchanged logic; persists via `AuthManager` |
| `auth/config.rs` | `auth_scope()` = `{issuer}::{client_id}`; default xAI issuer `https://auth.x.ai` |
| `auth/credential_provider.rs` | Credential → HTTP; verify with tests after store change |
| `util/grok_auth_credentials.rs` | Bearer apply for agent turns |
| `pager/.../cli.rs` + `pager-bin/main.rs` | Login entrypoints |
| Callers of `read_auth_json` | `managed_config.rs`, `agent/app.rs`, `agent/relay.rs` — continue if xAI-slot adapter preserved |

### Recommended on-disk shape (discretion)

```json
{
  "version": 1,
  "providers": {
    "xai": {
      "https://auth.x.ai::b1a00492-073a-47ea-816f-4c329264a828": {
        "key": "<access_token>",
        "auth_mode": "oidc",
        "create_time": "2026-07-16T00:00:00Z",
        "user_id": "...",
        "refresh_token": "...",
        "expires_at": "...",
        "oidc_issuer": "https://auth.x.ai",
        "oidc_client_id": "b1a00492-073a-47ea-816f-4c329264a828"
      },
      "xai::api_key": {
        "key": "<api_key>",
        "auth_mode": "api_key",
        "create_time": "...",
        "user_id": ""
      }
    }
  }
}
```

Notes:
- Default client_id is compile-time obfuscated in `GrokComConfig::default` (`b1a00492-…`). [VERIFIED: codebase]
- `providers.codex` may be **absent** this phase (reserved by schema + code paths that know the id).
- Optional `"version": 1` aids future migrations — recommended under Claude's Discretion.

### Legacy flat (read migration input)

```json
{
  "https://auth.x.ai::b1a00492-073a-47ea-816f-4c329264a828": { "...": "GrokAuth" },
  "xai::api_key": { "...": "GrokAuth" }
}
```

Detection: JSON object **without** a `providers` object key (or with only scope-shaped keys) → wrap as `providers.xai`. Do **not** read `~/.grok`. [VERIFIED: CONTEXT]

### AuthManager construction + path (existing)

```rust
// Source: crates/codegen/xai-grok-shell/src/auth/manager.rs (AuthManager::new)
// Priority: GROK_AUTH (inline GrokAuth) > GROK_AUTH_PATH > {grok_home}/auth.json
// grok_home from xai_grok_config::grok_home() → BUM_HOME | ~/.bum
let path = std::env::var("GROK_AUTH_PATH")
    .map(PathBuf::from)
    .unwrap_or_else(|_| grok_home.join("auth.json"));
```

### Credential path for agent turn (existing)

```rust
// Source: util/grok_auth_credentials.rs + auth/credential_provider.rs
// Live: AuthManager::get_valid_token() / current_or_expired()
// Wire: Authorization: Bearer <token>
//       X-XAI-Token-Auth: xai-grok-cli   (user/OAuth, not deployment key)
```

### CLI login (existing)

```rust
// Source: pager-bin main.rs + auth/flow.rs
xai_grok_shell::auth::run_cli_login(&config, oauth, device_auth, devbox).await?;
// device branch: AuthManager::new(&grok_home, config.grok_com_config) + run_auth_flow_interactive
// loopback branch: ensure_authenticated_with_override(..., reauth=true, ...)
```

### Merge-safe write sketch (implement)

```rust
// Source: recommended — storage layer only
fn write_xai_auth_store(path: &Path, xai_store: &AuthStore) -> std::io::Result<()> {
    let mut doc = read_auth_document_or_empty_recovering(path)?;
    // Preserve providers.codex and any future keys
    doc.providers.insert(PROVIDER_XAI.to_owned(), xai_store.clone());
    doc.version = Some(1);
    write_auth_document_atomic(path, &doc)
}
```

## State of the Art

| Old Approach | Current Approach (Phase 2 target) | When Changed | Impact |
|--------------|-----------------------------------|--------------|--------|
| Flat `auth.json` scope map (single product) | Nested `providers.{xai,codex}` single file | Phase 2 | Dual OAuth-safe storage |
| Home `~/.grok` / `GROK_HOME` | `~/.bum` / `BUM_HOME` only | Phase 1 | Auth already under bum home |
| Single-provider AuthManager only | Still single-provider **runtime**; multi-slot **store** | Phase 2 vs later 4–5 | Clear phase boundary |
| Earlier research: flat multi-scope keys | CONTEXT: nested provider map | Discuss-phase 2026-07-16 | Planner must not use flat multi-scope as primary |

**Deprecated/outdated:**
- `.planning/research/STACK.md` multi-provider layout using flat `openai::chatgpt` scope as the *primary* bum store shape — superseded for v1 by CONTEXT nested `providers` map (Codex payload may still map into `GrokAuth` fields inside `providers.codex` in Phase 5).
- Doc comments saying credentials persist to `~/.grok/auth.json` (stale after Phase 1).

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Optional `"version": 1` field is desirable | Discretion / schema | Low — omit if team prefers minimal JSON |
| A2 | Eager migrate-on-write is better than read-only migration | Pitfalls / discretion | Medium — if avoided, on-disk stays mixed longer |
| A3 | Storage-only adapter is sufficient for AUTH-01 without dual AuthManager types | Architecture | Medium — if later phases need earlier provider typing, some rework; still correct for Phase 2 |
| A4 | No live IdP needed in CI if AuthManager + HTTP header path tested with fixtures | Validation | Low — matches CONTEXT automated-tests mandate |

**If this table is empty:** N/A — assumptions listed for discuss/planner confirmation on discretion items only. Locked CONTEXT decisions are not assumptions.

## Open Questions

1. **Eager vs lazy migrate on disk**  
   - What we know: CONTEXT allows either.  
   - What's unclear: User preference for immediate rewrite after first login.  
   - Recommendation: **Eager migrate on successful write** so Phase 5 always sees nested form in real homes.

2. **Whether `read_auth_json` public API stays “xAI scopes only” forever**  
   - What we know: Several non-auth modules call it.  
   - What's unclear: Phase 5 may need multi-provider readers.  
   - Recommendation: Keep xAI-adapter now; add `read_auth_document` for multi-provider later without breaking callers.

3. **Codex placeholder: absent key vs empty object**  
   - What we know: CONTEXT says empty/absent OK.  
   - Recommendation: **Absent by default**; do not write empty `codex: {}` on every xAI write (noise + larger git-like diffs for secrets dir). Code must still preserve key if present.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust / cargo | Build & tests | ✓ | 1.92.0 | — |
| protoc | Workspace builds (unrelated to auth logic) | ✓ | system | repo `bin/protoc` |
| Live `auth.x.ai` | Manual OAuth smoke only | N/A in CI | — | Fixture tokens + unit tests |
| Browser | Manual browser OAuth smoke | N/A | — | Device-code manual / unit tests only |
| Network fixture IdP | Optional deeper OAuth tests | Not required | — | Skip; use AuthManager fixtures |

**Missing dependencies with no fallback:** none for automated Phase 2 scope.  
**Missing dependencies with fallback:** live IdP → fixtures.

## Validation Architecture

> `workflow.nyquist_validation` is enabled in `.planning/config.json`.

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Cargo built-in + `tokio::test` (workspace) |
| Config file | Per-crate; no jest/vitest |
| Quick run command | `cargo test -p xai-grok-shell --lib auth::` |
| Full suite command | `cargo test -p xai-grok-shell --lib` |

Prefer lib unit tests under `auth/storage.rs` and `auth/manager_tests.rs`; avoid full-workspace and ignored binary e2e for the AUTH-01 gate.

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| AUTH-01 | Legacy flat `auth.json` under temp home reads as xAI slot | unit | `cargo test -p xai-grok-shell --lib auth::storage::` (new) | ❌ Wave 0 |
| AUTH-01 | Write xAI credential emits nested `providers.xai` | unit | same | ❌ Wave 0 |
| AUTH-01 | Seed `providers.codex` fixture → xAI update does not clobber codex | unit | same | ❌ Wave 0 |
| AUTH-01 | `store_api_key` / `read_api_key` / `clear_api_key` only touch xAI slot | unit | same | ❌ Wave 0 |
| AUTH-01 | `AuthManager::update` + `get_valid_token` / `auth()` returns xAI key from multi-slot file | unit | `cargo test -p xai-grok-shell --lib auth::manager::` | ⚠️ existing manager tests assume flat; **extend** |
| AUTH-01 | Credential provider / `GrokAuthCredentials` applies Bearer from manager after multi-slot load | unit | `cargo test -p xai-grok-shell --lib` filter credential/auth | ⚠️ existing; re-verify after storage change |
| AUTH-01 | Empty xAI after logout does not delete file if codex scopes present | unit | new storage/manager test | ❌ Wave 0 |
| AUTH-01 | Path stays under temp `BUM_HOME` / passed `grok_home` (never `~/.grok`) | unit | construct `AuthManager::new(temp, …)` | ✅ pattern exists |
| AUTH-01 | Browser + device-code code paths still compile and unit-covered (no live IdP) | unit | existing `flow` / `device_code` tests | ✅ extend only if breakage |

### Sampling Rate

- **Per task commit:** `cargo test -p xai-grok-shell --lib auth::`
- **Per wave merge:** `cargo test -p xai-grok-shell --lib`
- **Phase gate:** Auth lib tests green + multi-slot isolation tests green before `/gsd:verify-work`

### Wave 0 Gaps

- [ ] `auth/storage.rs` tests: migrate flat → nested; merge-safe write; codex clobber resistance; API key under xAI
- [ ] `auth/manager_tests.rs` (or sibling): AuthManager load/update against nested document; logout/delete semantics with sibling provider
- [ ] Optional helper: `write_fixture_auth_document(path, xai, codex)` for tests (prefer over raw `fs::write`)
- [ ] Audit test helpers that bypass storage (`agent/relay.rs` test `write_test_auth_to_disk`) — still accept flat via migration **or** switch to storage API

*(Existing ~97 manager tests + storage write-fallback tests remain; they must keep passing after adapter lands.)*

## Security Domain

> `security_enforcement` enabled (ASVS L1).

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | yes | Existing OIDC/OAuth2 + device-code against `auth.x.ai`; API key fallback secondary |
| V3 Session Management | yes | Refresh tokens in `auth.json`; proactive refresh; permanent-failure handling; no auto-clear on transient errors |
| V4 Access Control | partial | Team pin / `force_login_team_uuid`; managed config eligibility — leave as-is |
| V5 Input Validation | yes | Serde-typed `GrokAuth` / document; device `user_code` charset validation; corrupt file backup |
| V6 Cryptography | yes | Do not hand-roll tokens/JWT crypto; use existing OIDC + `oauth2` crate; file mode **0o600** |

### Known Threat Patterns for this stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Refresh token double-spend → family revocation | Denial of service / Tampering | `auth.json.lock` + `still_live()` before IdP refresh [VERIFIED: codebase] |
| Sibling provider credential wipe | Tampering / Elevation of privilege (account mix-up) | Merge-safe multi-slot writes; isolation tests |
| Auth file world-readable | Information disclosure | `open_secure_file` / 0o600 [VERIFIED: codebase] |
| Logging tokens | Information disclosure | `token_suffix` only; Debug redacts [VERIFIED: codebase] |
| Import stock credential stores | Information disclosure / coupling | Forbidden — bum home only |
| Corrupt JSON silent overwrite | Tampering / availability | Backup to `auth.json.corrupt.<ts>` then recover [VERIFIED: codebase] |
| Inline `GROK_AUTH` injection | Spoofing | Existing env path; treat as trusted local operator config |

## Sources

### Primary (HIGH confidence)
- Codebase: `crates/codegen/xai-grok-shell/src/auth/{model,storage,manager,flow,device_code,config,credential_provider}.rs`
- Codebase: `crates/codegen/xai-grok-auth/src/{lib,auth_provider}.rs`
- Codebase: `crates/codegen/xai-grok-config/src/paths.rs` (`BUM_HOME` / `grok_home`)
- Codebase: `crates/codegen/xai-grok-pager/src/app/cli.rs`, `xai-grok-pager-bin/src/main.rs` Login
- `.planning/phases/02-…/02-CONTEXT.md` (locked decisions)
- `.planning/REQUIREMENTS.md` AUTH-01; `.planning/ROADMAP.md` Phase 2
- Phase 1 research/context for home isolation patterns

### Secondary (MEDIUM confidence)
- `.planning/research/STACK.md` — OAuth/Codex background; **layout superseded by CONTEXT** for multi-slot shape
- `.planning/codebase/INTEGRATIONS.md` — auth endpoints and env inventory

### Tertiary (LOW confidence)
- None critical; no unverified external package recommendations

## Discretion Recommendations (for planner)

| Decision | Recommendation | Rationale |
|----------|----------------|-----------|
| Schema version field | Include `"version": 1` optional | Cheap forward compat |
| Migrate timing | Eager rewrite nested on first successful write | Stable on-disk for Phase 5 |
| AuthManager depth | Storage-only adapter; single xAI manager | CONTEXT runtime default xAI; less risk |
| Test layout | Unit tests in storage + manager_tests; no live IdP | CONTEXT + CI |
| Empty codex key | Absent by default; preserve if present | Slot isolation without noise |

## Metadata

**Confidence breakdown:**
- Standard stack: **HIGH** — no new packages; existing crates verified in tree
- Architecture: **HIGH** — auth module structure and call sites verified by read/grep
- Pitfalls: **HIGH** — empty-file delete, flat write, and bypass helpers confirmed in source
- Schema field names: **MEDIUM** — exact serde shape is Claude's Discretion (recommendation above)

**Research date:** 2026-07-16  
**Valid until:** 2026-08-16 (stable brownfield; re-check if auth module heavily refactored)
