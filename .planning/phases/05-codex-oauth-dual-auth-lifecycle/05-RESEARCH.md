# Phase 5: Codex OAuth & dual auth lifecycle - Research

**Researched:** 2026-07-16
**Domain:** Dual-provider OAuth lifecycle (ChatGPT/Codex PKCE + device-code; multi-slot login/logout/status/refresh) in brownfield Rust TUI/CLI
**Confidence:** HIGH (in-tree multi-slot + xAI auth deeply scouting; Codex OAuth contract verified from openai/codex `codex-rs/login` source + official Codex auth docs)

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Extend CLI: **`bum login --provider codex|xai`**. Bare **`bum login` remains xAI-only** (Phase 2 lock).
- Codex login defaults to **browser PKCE** (Codex-style loopback); support **`--device-auth` / `--device-code`** for headless/SSH.
- Ship **CLI login as the hard requirement** for AUTH-02; wire a **minimal TUI/in-app path** only if trivial — do not block on rich dual-login chrome.
- Re-login / force re-auth **overwrites only that provider’s slot**.
- Selective logout: **`bum logout --provider codex|xai`**. **`bum logout --all`** clears both when explicit. Bare **`bum logout` must not silently wipe both** (fail closed with usage).
- Status: first-class **`bum auth status`** listing both providers; **never print tokens/secrets**.
- Each provider has an **independent refresh chain**. On `invalid_grant` / permanent auth failure, **wipe or mark invalid only that provider’s slot**.
- Prefer extending multi-slot **AuthManager / storage** (one store, per-slot ops).
- **No import** from `~/.codex` or stock Grok stores. Credentials only under `$BUM_HOME/auth.json`.
- Implement Codex OAuth **in-tree**, mirroring openai/codex `codex-rs/login` (PKCE S256, public client_id, authorize/token endpoints). Loopback **`127.0.0.1:1455`** with **1457** fallback; device-code APIs as in STACK.md.
- Persist under **`providers.codex`**: access token, refresh token, expiry, and ChatGPT account id / claims for `ChatGPT-Account-ID`.
- **Primary product path = ChatGPT OAuth**, not Platform API key.
- Prove AUTH-02..05 with **automated tests** (mock/fixture IdP). Live browser login optional manual smoke only.

### Claude's Discretion
- Exact clap shape (`--provider` vs positional), whether `bum auth login|logout|status` nests under an `auth` parent, and TUI depth beyond minimal wiring
- Module placement for Codex PKCE/device-code (new `auth/codex/` vs extend existing oidc/device modules)
- AuthManager internal threading for dual refresh (single manager with slot param vs thin dual handles)
- Wire details: originator tag, exact scope list, JWT claim parsing, header injection seam reuse from Phase 4
- Test layout (unit vs integration binary) and mock HTTP vs pure storage assertions

### Deferred Ideas (OUT OF SCOPE)
- Polished mid-session “login to Codex” gate when selecting a GPT model → Phase 6 (MOD-06)
- Cross-provider subagent auth inheritance / fail-closed child login → Phase 7
- Full login chrome rebrand (strings still saying Grok where harmless) → Phase 8
- Optional OpenAI Platform API-key secondary path → AUTH-V2 / out of v1 primary
- Import stock `~/.codex` auth.json → never v1
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| AUTH-02 | ChatGPT/Codex OAuth (PKCE browser + device-code); credentials only under bum store | Codex OAuth contract (issuer, client_id, ports, scopes); new `auth/codex/` flow writing `providers.codex` via multi-slot RMW; CLI `--provider codex` |
| AUTH-03 | Log out of one provider without clearing the other | Generic provider-slot clear; `perform_logout` / `run_cli_logout` dual-safe; bare logout fail-closed; `--all` explicit |
| AUTH-04 | Per-provider auth status (logged in / usable) | Read-only document inspect; paste-safe greppable format from UI-SPEC; no secrets |
| AUTH-05 | Independent access-token refresh per provider without wiping the other | Per-slot refresh + permanent-fail isolation; do not reuse xAI-only `AuthManager::clear` / full-file wipe paths for Codex failures |
</phase_requirements>

## Summary

Phase 5 fills the **reserved `providers.codex` slot** with a real ChatGPT OAuth lifecycle and makes login/logout/status/refresh **provider-scoped end-to-end**. Phases 1–4 already delivered product home (`~/.bum`), multi-slot `AuthDocument` + lock-scoped RMW that **preserves sibling providers**, provider-aware routing (`credential_slot = xai|codex`), and prepare-time Codex token **snapshot**. What is missing is almost entirely on the **write/lifecycle side**: there is no Codex PKCE/device login, no generic “clear provider X” API, `AuthManager` still owns a **single xAI OIDC scope**, CLI login/logout are xAI-only, bare logout would not fail-closed for dual wipe, and there is no `auth status` surface.

Codex OAuth should be implemented **in-tree** (mirror `openai/codex` `codex-rs/login`, not crates.io `codex-oauth`). Verified contract: issuer `https://auth.openai.com`, public `client_id = app_EMoamEEZ73f0CkXaXp7hrann`, bind `127.0.0.1:1455` (fallback **1457**), authorize redirect `http://localhost:{port}/auth/callback`, PKCE S256, scopes including `offline_access`, device flow under `{issuer}/api/accounts/deviceauth/*`. Persist as `GrokAuth` under `providers.codex` with `organization_id` (or equivalent) carrying `chatgpt_account_id` for Phase 4 sampling.

**Primary recommendation:** Add `auth/codex/` (PKCE + device) + **generic provider-slot storage ops**, extend CLI (`--provider`, fail-closed logout, `auth status`), then a **Codex-scoped refresher** that mutates only `providers.codex`. Reuse existing axum loopback, sha2/base64 PKCE helpers, and lock/atomic write — do not invent a second auth file or import `~/.codex`.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| CLI login/logout/status parse | CLI (binary + clap) | Shell auth | `Command::*` in pager-bin/cli; handlers in `shell::auth` |
| Browser PKCE loopback server | API / local process | — | axum/tiny_http-style loopback on 1455/1457; not TUI |
| Device-code poll | API / local process | CLI stderr UX | HTTP to auth.openai.com; copy per UI-SPEC |
| Credential persistence | Database / Storage (`auth.json`) | Shell AuthManager | Multi-slot document under `$BUM_HOME` |
| Independent refresh | API / Backend (shell runtime) | Storage | Per-slot IdP exchange + RMW; never cross-wipe |
| Per-request Codex bearer | API / Backend (sampler prepare) | Storage | Phase 4 route + slot; Phase 5 live refresh invalidates snapshot |
| `ChatGPT-Account-ID` header | API / Backend (sampling config) | — | Required for multi-workspace ChatGPT accounts |
| TUI `/logout` dual-safety | Browser / Client (TUI) | Shell | Fail-closed messaging; no silent dual-wipe |
| Missing-provider switch gate | Deferred Phase 6 | — | Out of scope |

## Standard Stack

### Core (brownfield — no new product deps)

| Library | Version (lockfile) | Purpose | Why Standard |
|---------|-------------------|---------|--------------|
| Rust | 1.92.0 / edition 2024 | Entire product | Pinned workspace |
| Tokio | 1 (full) | Async login/poll/refresh | Existing |
| clap | 4 | `--provider`, `auth status` | Existing CLI surface |
| reqwest | 0.12.24 (rustls) | Token POST, device poll | Existing HTTP |
| axum | 0.8.6 | Loopback OAuth callback | Prefer over Codex `tiny_http` (already in tree for xAI OIDC) |
| sha2 | 0.10.9 | PKCE S256 | Already used by xAI `oidc::protocol::generate_pkce` |
| base64 | 0.22.1 | PKCE verifier/challenge | Same |
| urlencoding | 2.1.3 | Form / query encoding | Already in shell crate |
| serde_json / chrono | workspace | auth.json + expiry | Existing `GrokAuth` |
| oauth2 | 5.0.0 | Optional helpers | Present; raw reqwest form POST is fine for Codex parity |

### Codex OAuth constants (implement as consts)

| Constant | Value | Source |
|----------|-------|--------|
| Issuer | `https://auth.openai.com` | openai/codex `DEFAULT_ISSUER` [VERIFIED: github raw openai/codex server.rs] |
| Public client_id | `app_EMoamEEZ73f0CkXaXp7hrann` | openai/codex `CLIENT_ID` [VERIFIED: github raw openai/codex manager.rs] |
| Preferred port | `1455` | openai/codex `DEFAULT_PORT` [VERIFIED] |
| Fallback port | `1457` | openai/codex `FALLBACK_PORT` [VERIFIED] |
| Redirect path | `/auth/callback` on **localhost** (not 127.0.0.1 in the URI string) | `http://localhost:{port}/auth/callback` [VERIFIED: server.rs `redirect_uri`] |
| Bind address | `127.0.0.1:{port}` | openai/codex `bind_server` [VERIFIED] |
| Scopes | `openid profile email offline_access api.connectors.read api.connectors.invoke` | build_authorize_url [VERIFIED] |
| Extra authorize flags | `id_token_add_organizations=true`, `codex_cli_simplified_flow=true`, `originator=<bum>` | [VERIFIED] + CONTEXT discretion for originator |
| Token endpoint | `{issuer}/oauth/token` | [VERIFIED] |
| Device usercode | `{issuer}/api/accounts/deviceauth/usercode` | [VERIFIED: device_code_auth.rs] |
| Device token poll | `{issuer}/api/accounts/deviceauth/token` | [VERIFIED] |
| Device verify URL | `{issuer}/codex/device` | [VERIFIED] + UI-SPEC |
| Device code exchange redirect | `{issuer}/deviceauth/callback` | [VERIFIED] |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| wiremock | 0.6.5 | Fake auth.openai.com | Integration tests for exchange/refresh |
| tempfile + serial_test | workspace | Isolated `$BUM_HOME` tests | Already pattern in `auth_multi_slot.rs` |
| jsonwebtoken (dangerous decode) | existing via jwt.rs | Parse `exp` / claims without verify | Extend for `chatgpt_account_id` / email from id_token |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| In-tree PKCE mirror | crates.io `codex-oauth` | **Rejected** — incomplete, low adoption (STACK) |
| Vendor full `codex-login` crate | path/git dep | Monorepo-coupled; pulls tiny_http + codex config; heavier than needed |
| Second AuthManager process | dual managers | Overkill; one document + per-slot ops is correct |
| Platform API key primary | ChatGPT OAuth | **Rejected** by product lock |

**Installation:** none required for v1 — all crates already workspace deps.

**Version verification:** lockfile versions above from `Cargo.lock` on 2026-07-16 [VERIFIED: Cargo.lock].

## Package Legitimacy Audit

No **new** external packages. Existing deps used by this phase:

| Package | Registry | Age | Downloads | Source Repo | Verdict | Disposition |
|---------|----------|-----|-----------|-------------|---------|-------------|
| oauth2 | crates | since 2014 | ~812k/wk | github.com/ramosbugs/oauth2-rs | OK | Already in tree |
| sha2 | crates | since 2016 | ~15M/wk | RustCrypto/hashes | OK | Already in tree |
| base64 | crates | since 2015 | ~20M/wk | marshallpierce/rust-base64 | OK | Already in tree |
| urlencoding | crates | since 2016 | ~3.6M/wk | kornelski/rust_urlencoding | OK | Already in tree |
| axum | crates | since 2021 | ~7.5M/wk | tokio-rs/axum | OK | Already in tree |
| reqwest | crates | since 2016 | ~11M/wk | seanmonstar/reqwest | OK | Already in tree |

**Packages removed due to [SLOP] verdict:** none  
**Packages flagged as suspicious [SUS]:** none  
**Do not add:** `codex-oauth` (crates.io) — rejected by STACK + product decision.

## Architecture Patterns

### System Architecture Diagram

```text
┌─────────────┐   clap    ┌──────────────────────┐
│ bum CLI     │──────────▶│ run_cli_login/logout  │
│ auth status │           │ + provider arg        │
└─────────────┘           └──────────┬───────────┘
                                     │
              ┌──────────────────────┼──────────────────────┐
              ▼                      ▼                      ▼
     ┌────────────────┐    ┌─────────────────┐    ┌──────────────────┐
     │ xAI OIDC/device│    │ Codex PKCE      │    │ Status inspector │
     │ auth/oidc/*    │    │ auth/codex/*    │    │ read AuthDocument│
     │ device_code.rs │    │ (NEW)           │    │ no secrets       │
     └───────┬────────┘    └────────┬────────┘    └────────┬─────────┘
             │                      │                      │
             │  GrokAuth write      │  GrokAuth write      │ read-only
             ▼                      ▼                      ▼
     ┌────────────────────────────────────────────────────────────┐
     │ auth.json  (single file, auth.json.lock RMW)               │
     │  providers.xai  → scope map (issuer / xai::api_key)        │
     │  providers.codex → scope map (openai::chatgpt or issuer)   │
     └────────────────────────────┬───────────────────────────────┘
                                  │
              ┌───────────────────┼───────────────────┐
              ▼                   ▼                   ▼
     ┌────────────────┐  ┌────────────────┐  ┌────────────────────┐
     │ AuthManager    │  │ Codex refresher│  │ Phase 4 routing    │
     │ (xAI scope)    │  │ (slot=codex)   │  │ snapshot/live key  │
     │ independent RT │  │ independent RT │  │ + ChatGPT-Account  │
     └────────────────┘  └────────────────┘  └────────────────────┘
```

### Recommended Project Structure

```text
crates/codegen/xai-grok-shell/src/auth/
├── model.rs              # PROVIDER_*, GrokAuth (reuse; map chatgpt_account_id → organization_id)
├── storage.rs            # ADD: mutate_provider_store / clear_provider_slot (generic)
├── manager.rs            # Keep xAI; do NOT clear whole file on Codex fail
├── flow.rs               # run_cli_login/logout → dispatch by provider; auth status
├── oidc/                 # xAI only (keep)
├── device_code.rs        # xAI device (keep)
├── codex/                # NEW (recommended module placement)
│   ├── mod.rs            # public login entry, constants
│   ├── pkce.rs           # or re-export/share generate_pkce from oidc::protocol
│   ├── browser.rs        # axum loopback 1455/1457, authorize URL, exchange
│   ├── device.rs         # deviceauth usercode/token poll
│   ├── claims.rs         # id_token → email, chatgpt_account_id, plan labels
│   └── refresh.rs        # fixed-endpoint refresh (no OIDC discovery required)
├── refresh/
│   ├── oidc_refresher.rs # xAI
│   └── codex_refresher.rs # NEW or dual-dispatch TokenRefresher
└── ...

crates/codegen/xai-grok-pager/src/app/cli.rs   # Login { provider, ... }, Logout { provider, all }, Auth { Status }
crates/codegen/xai-grok-pager-bin/src/main.rs  # match arms
crates/codegen/xai-grok-pager/src/slash/commands/logout.rs  # dual-safe copy
crates/codegen/xai-grok-shell/tests/
├── auth_multi_slot.rs    # extend isolation proofs
└── auth_codex_lifecycle.rs  # NEW integration: login mock, logout selective, status, refresh wipe isolation
```

### Pattern 1: Provider-slot RMW (extend storage)

**What:** Full-document mutate that inserts/replaces **one** provider key and preserves siblings (already partially implemented for xAI via `apply_xai_slot` / `mutate_xai_store_or_prune`).

**When to use:** Any login persist, refresh persist, selective logout.

**Example shape:**

```rust
// Target API (planner should name these in tasks)
pub(crate) fn mutate_provider_store_or_prune(
    auth_file: &Path,
    provider: &str, // PROVIDER_XAI | PROVIDER_CODEX
    f: impl FnOnce(&mut AuthStore),
) -> std::io::Result<ProviderStoreMutation>;
```

Today `write_auth_json` only calls `apply_xai_slot`. Codex login **must not** call xAI-only writers. [VERIFIED: storage.rs]

### Pattern 2: In-tree Codex browser PKCE (mirror upstream, axum transport)

**What:** Bind 1455→1457, generate PKCE S256 + state, open authorize URL, exchange code with form body matching Codex.

**Critical parity details (from openai/codex server.rs):**
- Redirect URI host string is **`localhost`**, not `127.0.0.1`, even though the server binds `127.0.0.1` [VERIFIED]
- Path is **`/auth/callback`** (xAI OIDC uses `/callback` on a dynamic port — do not reuse xAI path/port blindly) [VERIFIED: oidc/login.rs vs codex server.rs]
- On port busy: attempt cancel previous login server then fallback 1457 [VERIFIED bind_server policy]
- Persist access + refresh + account_id; set `last_refresh` equivalent (Codex field) — map into `GrokAuth` (`create_time`/`expires_at` + optional extension or derive from JWT `exp`)

**Do not** implement Codex’s optional **token-exchange → Platform API key** (`obtain_api_key`) as the product path — ChatGPT OAuth bearer goes to `chatgpt.com/backend-api/codex` (Phase 4). Optional exchange is out of v1 primary. [ASSUMED product interpretation of CONTEXT lock; aligned with STACK]

### Pattern 3: Independent permanent-failure isolation

**What:** xAI `AuthManager` today records permanent failure and may clear **its scope** via `remove_scope` / `clear` against the **xAI slot only** (via `mutate_xai_store_or_prune`). Codex permanent fail must use **codex slot clear only**.

**When to use:** `invalid_grant`, refresh_token_reused/expired/invalidated taxonomy from Codex.

**Existing safety:** Manager tests assert invalid_grant does **not** auto-delete entire auth.json [VERIFIED: manager_tests.rs comments]. Extend that guarantee cross-provider: wiping Codex never empties xAI scopes.

### Pattern 4: Status as pure document inspect

**What:** Read `AuthDocument`, for each of `xai`/`codex` compute `logged_in` / `usable` from presence of non-blank key + not sticky permanent-invalid; surface email/plan when present.

**Format locked by 05-UI-SPEC.md** (greppable key:value).

### Anti-Patterns to Avoid

- **Calling `AuthManager::clear()` for bare logout** after dual login — today clears the manager’s current (xAI) scope; bare CLI must fail closed per CONTEXT, not clear “whatever is current.”
- **Writing Codex via `write_auth_json`** — that replaces only xAI but is the wrong semantic entry; add explicit provider API.
- **Reusing xAI OIDC discovery** for auth.openai.com — Codex uses **fixed paths**, not `.well-known` discovery [VERIFIED openai/codex].
- **Redirect host mismatch** (`127.0.0.1` in authorize URL vs allow-listed `localhost`) — causes silent token exchange / callback failures [VERIFIED redirect construction uses localhost].
- **Importing or reading `~/.codex/auth.json`** — never v1.
- **Sending ChatGPT OAuth bearer to `api.openai.com` as Platform key** — wrong billing/surface (PITFALLS + STACK).
- **Logging tokens** — only suffixes/lengths; status paste-safe.
- **TUI dual chrome** — non-blocking; Phase 6 owns gate UX.
- **Platform API key marketing** in login copy — UI-SPEC forbids.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Multi-slot atomic write | New file format / dual files | `mutate_auth_document*` + lock | Already handles sibling preserve, corrupt recovery, version fail-closed |
| PKCE S256 | Custom crypto | Existing `oidc::protocol::generate_pkce` (export/share) or identical sha2/base64 | Same algorithm Codex uses |
| Loopback HTTP | New server crate | axum (xAI path) | In tree; avoid tiny_http dep |
| JWT exp parse | Hand parsers | `auth/jwt.rs` + claim peek helpers | Existing insecure_decode pattern |
| OAuth form POST | New client stack | reqwest `.form` / body | Same as oidc exchange_code |
| Mock IdP | Live ChatGPT in CI | wiremock / axum fixture server | Phase 2/4 test style |
| Credential encryption | OS keyring | File `0600` auth.json under BUM_HOME | Codex supports keyring; v1 file store locked |

**Key insight:** Complexity is **isolation and CLI contracts**, not inventing OAuth. The dangerous bugs are cross-slot wipe, wrong redirect URI, and refresh that rewrites the whole document.

## Runtime State Inventory

> Not a rename phase — included briefly because logout/login mutate durable runtime state.

| Category | Items Found | Action Required |
|----------|-------------|-----------------|
| Stored data | `$BUM_HOME/auth.json` multi-slot (`providers.xai` / `providers.codex`) | Code: fill/clear `codex` slot; migrate not required beyond existing nested format v1 |
| Live service config | None for Codex (no n8n/etc.) | None |
| OS-registered state | Loopback ports 1455/1457 during login only | Transient; document conflict with stock `codex login` |
| Secrets/env vars | `BUM_HOME`, `GROK_AUTH_PATH` (rejected if outside product store), `XAI_API_KEY` (xAI only) | Do not invent Codex env primary; optional later |
| Build artifacts | None specific | None |

**Nothing found in category:** Live service config / build artifacts — verified by codebase scout (auth is file-based under product home).

## Common Pitfalls

### Pitfall 1: Silent dual-wipe on logout / permanent fail
**What goes wrong:** Bare `bum logout` or one provider’s `invalid_grant` clears entire `auth.json`.  
**Why:** Single-provider mental model; `perform_logout`/`clear` history.  
**How to avoid:** Fail-closed bare logout; slot-scoped clear APIs; tests with both slots populated.  
**Warning signs:** After Codex logout, xAI model samples 401.

### Pitfall 2: Redirect URI / port mismatch vs allow-list
**What goes wrong:** Authorize succeeds in browser but callback never completes or token exchange fails.  
**Why:** Using `127.0.0.1` in redirect string, wrong path (`/callback` vs `/auth/callback`), or arbitrary port.  
**How to avoid:** Match Codex exactly: bind 127.0.0.1, redirect `http://localhost:{port}/auth/callback`, ports 1455→1457.  
**Warning signs:** “State mismatch”, connection refused on 1455, token endpoint 400.

### Pitfall 3: Refresh without disk persist → token family revocation
**What goes wrong:** Refresh succeeds in memory but disk keeps old RT; next refresh hits reuse → permanent fail.  
**Why:** Community multi-agent agents skip persist (STACK/PITFALLS).  
**How to avoid:** Always RMW-write new access+refresh under lock after success; adopt sibling disk token on invalid_grant before wipe (xAI pattern in `OidcRefresher::retry_with_fresh_disk_token`).  
**Warning signs:** Forced re-login every few hours despite “success” refresh logs.

### Pitfall 4: AuthManager remains xAI-only while Codex uses stale snapshot
**What goes wrong:** Phase 4 `snapshot_codex_session_key_from_auth_store` caches mtime; refresh updates tokens but long-lived process keeps old bearer until prepare re-reads; or never refreshes proactively.  
**Why:** Phase 4 intentionally deferred live Codex AuthManager.  
**How to avoid:** Phase 5: invalidate cache on login/logout/refresh; prefer live resolver or short-lived re-read; proactive Codex refresh policy (JWT exp ≤ now+5m; optional last_refresh ~8d from Codex).  
**Warning signs:** Mid-session Codex 401 while status says usable.

### Pitfall 5: Missing `ChatGPT-Account-ID`
**What goes wrong:** Multi-workspace accounts fail routing on ChatGPT backend.  
**Why:** Bearer alone insufficient (STACK).  
**How to avoid:** Parse `chatgpt_account_id` from id_token; store in `GrokAuth.organization_id` (STACK mapping); inject header in Codex sampling path.  
**Warning signs:** Works for single-workspace users only.

### Pitfall 6: Port conflict with stock Codex CLI
**What goes wrong:** Both tools fight over 1455.  
**Why:** Shared public client allow-list.  
**How to avoid:** Fallback 1457 + UI-SPEC port bind error copy; optional cancel-peer behavior (Codex does cancel request to prior server).  
**Warning signs:** Bind fail; error message should suggest `--device-auth`.

### Pitfall 7: Device-code disabled in workspace
**What goes wrong:** Device login fails even when browser would work.  
**Why:** ChatGPT security settings must enable device code (official docs).  
**How to avoid:** Clear error; document; browser remains default.  
**Warning signs:** Device endpoint 403/disabled.

### Pitfall 8: Logging secrets in “status” or exchange errors
**What goes wrong:** Tokens paste into support tickets.  
**Why:** Debug dumps of auth.json or error bodies with tokens.  
**How to avoid:** UI-SPEC secrets policy; redacted diagnostics like `AuthStoreReadError`.  

## Code Examples

### Codex authorize URL shape (mirror)

```text
// Source: openai/codex codex-rs/login/src/server.rs build_authorize_url [VERIFIED]
https://auth.openai.com/oauth/authorize
  ?response_type=code
  &client_id=app_EMoamEEZ73f0CkXaXp7hrann
  &redirect_uri=http://localhost:1455/auth/callback
  &scope=openid%20profile%20email%20offline_access%20api.connectors.read%20api.connectors.invoke
  &code_challenge=<S256>
  &code_challenge_method=S256
  &id_token_add_organizations=true
  &codex_cli_simplified_flow=true
  &state=<state>
  &originator=bum
```

### Token exchange form (browser)

```text
// Source: openai/codex exchange_code_for_tokens [VERIFIED]
POST https://auth.openai.com/oauth/token
Content-Type: application/x-www-form-urlencoded

grant_type=authorization_code
&code=...
&redirect_uri=http://localhost:1455/auth/callback
&client_id=app_EMoamEEZ73f0CkXaXp7hrann
&code_verifier=...
```

### Persist mapping into multi-slot store

```json
{
  "version": 1,
  "providers": {
    "xai": { "https://auth.x.ai::<client>": { "/* existing */": "" } },
    "codex": {
      "https://auth.openai.com::app_EMoamEEZ73f0CkXaXp7hrann": {
        "key": "<access_token>",
        "auth_mode": "oidc",
        "create_time": "...",
        "user_id": "<chatgpt_user_id or sub>",
        "email": "user@example.com",
        "refresh_token": "...",
        "expires_at": "...",
        "oidc_issuer": "https://auth.openai.com",
        "oidc_client_id": "app_EMoamEEZ73f0CkXaXp7hrann",
        "organization_id": "<chatgpt_account_id>"
      }
    }
  }
}
```

Scope key: prefer stable `issuer::client_id` or fixed `openai::chatgpt` — **pick one** and use it consistently for `select_provider_access_token` (Phase 4 already selects best Oidc>ApiKey). [ASSUMED: recommend `openai::chatgpt` for fixture stability OR issuer-style for parity with xAI; planner should lock one.]

### Existing multi-slot read already used by Phase 4

```rust
// crates/codegen/xai-grok-shell/src/agent/config.rs
// snapshot_codex_session_key_from_auth_store → read_provider_auth_store(path, PROVIDER_CODEX)
// → select_provider_access_token(&store).map(|a| a.key)
```

Phase 5 must keep this working and invalidate its mtime cache on mutations.

### CLI surface (recommended clap)

```rust
// Discretion: --provider on Login/Logout; Auth subcommand for status
Login {
  provider: Option<ProviderCli>, // None => xai
  oauth: bool,
  device_auth: bool,
  ...
}
Logout {
  provider: Option<ProviderCli>,
  all: bool, // conflicts with provider
}
Auth {
  Status,
}
```

Bare `Logout` without provider/all → usage + non-zero exit (UI-SPEC).

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Single flat auth.json | Nested `version` + `providers` map | Phase 2 (this fork) | Dual OAuth possible without clobber |
| xAI-only AuthManager | Still xAI-scoped runtime | Phase 2–4 | Phase 5 must add Codex lifecycle without breaking xAI |
| Codex token snapshot at prepare | Need live/refresh-aware Codex path | Phase 4 deferred | Phase 5 AUTH-05 |
| Platform API as “OpenAI auth” | ChatGPT OAuth primary | Product lock | Correct billing + models |
| Chat Completions for Codex | Responses API | Codex CLI | Already Phase 4 routing |

**Deprecated/outdated:**
- crates.io `codex-oauth` as production dependency
- Treating ChatGPT OAuth token as Platform API key
- Silent full-store logout

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Prefer new module `auth/codex/` over stuffing Codex into `oidc/` | Architecture | Slight rework of file layout only |
| A2 | Store `chatgpt_account_id` in `GrokAuth.organization_id` | Persist mapping | Header injection wrong field; need dedicated field if overloaded |
| A3 | Skip Codex `obtain_api_key` token-exchange (Platform key) | Pattern 2 | If some GPT features require exchanged API key, samples fail until added |
| A4 | Scope key `openai::chatgpt` vs issuer::client_id — either works if consistent | Persist | Fixture/tests flakiness if mixed |
| A5 | Proactive Codex refresh mirrors Codex 5m JWT / ~8d last_refresh | AUTH-05 | May refresh more/less often than stock CLI |
| A6 | `originator=bum` accepted by IdP (Codex sets its own originator) | OAuth URL | Cosmetic analytics only if ignored; unlikely hard fail |
| A7 | HTTP SSE remains sufficient for ChatGPT backend (Phase 4 open item) | Out of AUTH scope | Inference issues are Phase 9/4 residual, not AUTH-02 |

## Open Questions

1. **Codex refresh ownership model**
   - What we know: xAI `AuthManager` is large (sleep gate, proactive loop, lock revalidation).
   - What's unclear: whether to parameterize it by provider or add a thinner `CodexAuthHandle`.
   - Recommendation: **thin Codex handle** + shared storage RMW for Phase 5 speed; share `TokenRefresher` trait. Avoid rewriting xAI manager.

2. **`last_refresh` field**
   - Codex persists `last_refresh`; `GrokAuth` has no dedicated field.
   - Recommendation: derive proactive refresh from JWT `exp` first; add optional serde field only if 8-day policy is required.

3. **TUI `/logout`**
   - Today: clears via shell without provider choice; description implies full logout.
   - Recommendation: fail-closed toast if would dual-wipe; CLI is source of truth this phase (UI-SPEC).

4. **Live ChatGPT Account header in prepare path**
   - Phase 4 stamps bearer; `sampling_config_for_model` does not yet inject `ChatGPT-Account-ID`.
   - Recommendation: include in Phase 5 when persisting real account_id so dual auth is actually usable (still not Phase 6 gate).

5. **Revoke on logout**
   - Codex has `oauth/revoke`; bum xAI logout is local clear.
   - Recommendation: **local clear only** for v1 (matches current xAI; simpler tests). Optional revoke later.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust/cargo | Build/tests | ✓ | 1.92.0 | — |
| Network to auth.openai.com | Live OAuth smoke | optional | — | Mock IdP in CI |
| Free TCP 1455/1457 | Browser login | env-dependent | — | 1457 fallback; device-auth |
| Browser | Default PKCE | env-dependent | — | Print URL + paste; device-auth |
| stock `codex` CLI | Manual golden compare | optional | — | Not required for CI |

**Missing dependencies with no fallback:** none for automated AUTH-02..05 proofs.

**Missing dependencies with fallback:** live browser / ports → device-auth + mocks.

## Validation Architecture

> `workflow.nyquist_validation` is enabled in `.planning/config.json`.

### Test Framework

| Property | Value |
|----------|-------|
| Framework | cargo test (crate-local) + existing shell integration tests |
| Config file | none special — `crates/codegen/xai-grok-shell` |
| Quick run command | `cargo test -p xai-grok-shell --test auth_multi_slot --test auth_codex_lifecycle --lib auth:: -- --nocapture` (adjust names) |
| Full suite command | `cargo test -p xai-grok-shell --test auth_multi_slot --test auth_codex_lifecycle --test provider_routing` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| AUTH-02 | Mock browser/device exchange writes only `providers.codex` under temp BUM_HOME | integration | `cargo test -p xai-grok-shell --test auth_codex_lifecycle codex_login_persists_slot` | ❌ Wave 0 |
| AUTH-02 | PKCE authorize URL contains client_id, S256, localhost:1455/auth/callback | unit | `cargo test -p xai-grok-shell codex_authorize_url_` | ❌ Wave 0 |
| AUTH-03 | Logout codex leaves xai token; logout xai leaves codex | integration | `cargo test -p xai-grok-shell selective_logout_isolates` | ❌ Wave 0 |
| AUTH-03 | Bare logout prints usage + non-zero (or Err) without mutating | unit/cli | clap + handler test | ❌ Wave 0 |
| AUTH-03 | `--all` clears both when both present | integration | `... logout_all` | ❌ Wave 0 |
| AUTH-04 | Status greppable; both providers; no token substrings | unit | `auth_status_format_` | ❌ Wave 0 |
| AUTH-05 | Refresh codex mock updates only codex slot | integration | `codex_refresh_isolates` | ❌ Wave 0 |
| AUTH-05 | invalid_grant marks/wipes codex only | integration | `codex_invalid_grant_no_xai_wipe` | ❌ Wave 0 |
| AUTH-01 reg | xAI login still works / multi-slot sibling preserve | existing | `cargo test -p xai-grok-shell --test auth_multi_slot` | ✅ |
| MOD-04/05 reg | Routing still selects slots | existing | `cargo test -p xai-grok-shell --test provider_routing` | ✅ |

### Sampling Rate

- **Per task commit:** focused unit tests for touched module
- **Per wave merge:** auth_multi_slot + auth_codex_lifecycle + provider_routing
- **Phase gate:** full map green; optional manual `bum login --provider codex` smoke

### Wave 0 Gaps

- [ ] `crates/codegen/xai-grok-shell/tests/auth_codex_lifecycle.rs` — AUTH-02..05 integration
- [ ] Mock IdP helpers (wiremock routes for `/oauth/token`, deviceauth) under test module or `auth/codex/test_support`
- [ ] Clap parse tests in `cli.rs` for `--provider`, bare logout fail-closed, `auth status`
- [ ] Export or duplicate PKCE helper visibility for unit tests

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | yes | OAuth2/PKCE public client; device-code secondary |
| V3 Session Management | yes | Refresh tokens; independent invalidation; local logout |
| V4 Access Control | partial | Provider slot isolation; no cross-credential sampling |
| V5 Input Validation | yes | Validate OAuth `state`; reject missing code; sanitize error bodies |
| V6 Cryptography | yes | PKCE S256 via sha2; no hand-rolled crypto; TLS via rustls/reqwest |

### Known Threat Patterns

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| OAuth CSRF on loopback | Spoofing | Cryptographic `state` compare (Codex + xAI pattern) |
| Token leakage in logs/status | Information disclosure | Never print tokens; redacted diagnostics; UI-SPEC |
| Refresh token theft / reuse | Elevation | Lock around spend-refresh; sibling adopt; slot-scoped wipe |
| Open redirect / wrong callback | Tampering | Fixed allow-listed redirect URI only |
| Cross-provider wipe | Denial of service / Integrity | Provider-scoped mutate only |
| Credential file theft | Information disclosure | `0600` auth.json under isolated home; no stock import |
| Confused deputy (xAI bearer on Codex host) | Elevation | Phase 4 route + never cross_slot tests |

## Project Constraints (from AGENTS.md)

- Stay on Rust workspace (edition 2024, Tokio) — fork evolution not rewrite
- ChatGPT OAuth primary for Codex (not API-key-only)
- Per-model routing already done Phase 4 — Phase 5 supplies credentials
- `~/.bum` isolation; no credential sharing with stock CLIs
- Product/CLI name `bum` on new/rewritten strings (Phase 8 for full chrome)
- Prefer per-crate tests; `cargo test -p xai-grok-shell`
- Root `Cargo.toml` generated — edit per-crate manifests only
- Use `dunce::canonicalize` not std canonicalize
- `thiserror` / `anyhow` error style as existing auth modules

## Sources

### Primary (HIGH confidence)

- In-tree: `crates/codegen/xai-grok-shell/src/auth/{mod,model,storage,manager,flow,device_code,oidc/*}.rs`
- In-tree: `crates/codegen/xai-grok-shell/src/agent/config.rs` (`snapshot_codex_session_key_from_auth_store`, `resolve_credentials_for_provider`)
- In-tree: `crates/codegen/xai-grok-pager/src/app/cli.rs` Login/Logout; `pager-bin` Command match
- In-tree: `tests/auth_multi_slot.rs`, `tests/provider_routing.rs`
- openai/codex `codex-rs/login/src/server.rs` — PKCE, ports, redirect, exchange [VERIFIED: raw GitHub 2026-07-16]
- openai/codex `codex-rs/login/src/auth/manager.rs` — `CLIENT_ID` [VERIFIED]
- openai/codex `codex-rs/login/src/device_code_auth.rs` — device API paths [VERIFIED]
- `.planning/research/STACK.md` — integration contract (pre-verified session research)
- Official Codex auth docs: https://learn.chatgpt.com/docs/auth — ChatGPT vs API key, device-code, `codex login --device-auth`, credential cache [CITED]
- Phase CONTEXT/UI-SPEC/REQUIREMENTS for locks and copy

### Secondary (MEDIUM confidence)

- Community confirmation of port 1455 loopback failures (OpenAI forum / GitHub issues) [CITED: web search]
- Public client_id reuse discussion (community) — legal/ToS durability MEDIUM (STATE.md)

### Tertiary (LOW confidence)

- Exact ToS durability of third-party public client_id reuse long-term
- Whether ChatGPT Responses requires WebSocket for full parity (deferred transport; not AUTH lifecycle)

## Recommended Plan Wave Structure

| Wave | Focus | Delivers | Req IDs |
|------|-------|----------|---------|
| **0** | Test harness + RED fixtures | `auth_codex_lifecycle` skeleton; clap parse tests; mock IdP stubs | — |
| **1** | Storage + status primitives | `mutate_provider_store` / `clear_provider_slot`; status model + format (no CLI yet OK) | AUTH-03 foundation, AUTH-04 core |
| **2** | Codex login (browser PKCE + device) | `auth/codex/*`; CLI `login --provider codex`; persist claims | AUTH-02 |
| **3** | Logout CLI dual lifecycle | `--provider` / `--all` / bare fail-closed; TUI `/logout` dual-safe copy | AUTH-03 |
| **4** | Status CLI + wire-up | `bum auth status`; path error copy | AUTH-04 |
| **5** | Independent Codex refresh + cache invalidate | refresher; invalid_grant isolation; invalidate Phase 4 snapshot cache; optional ChatGPT-Account-ID inject | AUTH-05 (+ usability) |
| **6** | Phase gate | Full test map; optional manual smoke notes | AUTH-02..05 |

Parallelism: Wave 1 can start immediately; Wave 2 depends on 1 writers; Wave 3–4 can parallel after 1; Wave 5 after 2.

## Metadata

**Confidence breakdown:**
- Standard stack: **HIGH** — brownfield versions from Cargo.lock; Codex constants from upstream source
- Architecture: **HIGH** — storage/manager/CLI integration points named from live code
- Pitfalls: **HIGH** — match PITFALLS.md + upstream OAuth footguns + in-tree wipe history

**Research date:** 2026-07-16  
**Valid until:** ~2026-08-16 (OAuth client_id/ports stable; re-check openai/codex login if upstream changes allow-list)

### Key module inventory (planner name these)

| Symbol / path | Role today | Phase 5 action |
|---------------|------------|----------------|
| `PROVIDER_XAI` / `PROVIDER_CODEX` | Slot ids | Keep |
| `AuthDocument` / `read_auth_document` | Multi-slot parse | Keep |
| `mutate_auth_document*` | Full RMW | Build provider helpers on top |
| `write_auth_json` / `mutate_xai_store_or_prune` | **xAI-only** writers | Do not use for Codex |
| `read_provider_auth_store` | Per-slot read | Status + refresh + Phase 4 |
| `select_provider_access_token` | Best token in slot | Codex login must write selectable Oidc entry |
| `AuthManager` | xAI scope runtime | Leave for xAI; don't dual-wipe |
| `run_cli_login` / `run_cli_logout` / `perform_logout` | xAI CLI | Provider dispatch + fail-closed |
| `report_signed_in` | Copy | Provider-labeled success (UI-SPEC) |
| `oidc::protocol::generate_pkce` | S256 | Share with Codex |
| `snapshot_codex_session_key_from_auth_store` | mtime cache | Invalidate on mutates; live path |
| `Command::Login` / `Logout` | clap | Add provider/all/auth status |
| `/logout` slash | TUI | Dual-safe messaging |
