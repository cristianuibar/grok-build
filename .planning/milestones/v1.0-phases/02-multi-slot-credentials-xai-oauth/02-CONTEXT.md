# Phase 2: Multi-slot credentials & xAI OAuth - Context

**Gathered:** 2026-07-16
**Status:** Ready for planning

<domain>
## Phase Boundary

Auth storage becomes **provider-scoped** so dual OAuth is safe later, while **xAI login still works end-to-end under bum** (AUTH-01). Credentials live only under the bum product home (`~/.bum` / `BUM_HOME`). This phase implements multi-slot store structure with **xAI as the first live provider** and a **reserved Codex slot** that is not filled yet.

**In scope:**
- Multi-provider auth store schema (`xai` + reserved `codex`)
- Preserve/fix xAI OAuth (browser + device-code) and API-key fallback under the xAI slot
- Writes never clobber sibling provider slots; store only under bum home
- Prove agent turns can authenticate on the xAI path after login (automated tests)

**Out of scope (later phases):**
- Codex/ChatGPT OAuth login and dual lifecycle (AUTH-02–05 → Phase 5)
- Model catalog / GPT entries (Phase 3), provider-aware routing (Phase 4)
- Mid-session switch & missing-provider gate (Phase 6)
- Full UI chrome/string rebrand (Phase 8)
- Import/share of `~/.grok` or stock Codex credential stores (explicitly out of v1)

</domain>

<decisions>
## Implementation Decisions

### Multi-slot store schema
- **Single `auth.json`** with a top-level **provider map** (`providers.xai`, reserved `providers.codex`) — one file, existing `auth.json.lock`, atomic write path
- Stable short provider ids: **`xai`** and **`codex`** (ChatGPT OAuth will live under `codex` in Phase 5)
- Nest today’s xAI scopes (`issuer` keys, `xai::api_key`) under **`providers.xai`**
- **Read migration:** if only the legacy flat scope map is present under bum home, treat it as the xAI slot (no import from `~/.grok`)
- Codex slot is **schema + empty/absent OK** this phase — write paths must not clobber other slots; no Codex login

### xAI OAuth surface (AUTH-01)
- **Browser OAuth + device-code** must both work, equivalent to stock Grok Build (CLI + existing TUI paths)
- **`bum login` defaults to xAI** — no required provider argument yet; keep `--oauth` / `--device-auth` (and aliases)
- Credentials stored **only** under bum product home (`$BUM_HOME/auth.json` / `~/.bum/auth.json`) — never `~/.grok` or Codex paths
- Keep **xAI API-key fallback** working under `providers.xai` (OAuth remains primary)

### Isolation, safety & AuthManager scope
- Reuse existing **advisory lock** (`auth.json.lock`) and **crash-safe atomic write** (with in-place fallback) for the multi-slot store
- **Slot isolation:** updating one provider must never wipe or overwrite the other provider’s credentials
- **Runtime default remains xAI** this phase — multi-slot is under the store; full dual AuthManager/routing is later (Phases 4–5)
- **No import** from stock `~/.grok` or Codex credential stores (PROJECT / Phase 1 locked)

### Verification & phase boundary
- Prove success with **automated tests**: multi-slot structure, xAI write/read without clobber, AuthManager supplies xAI credential for a request/auth path (mock/fixture IdP where needed). Live browser OAuth optional manual smoke only
- Multi-provider **logout / status** → Phase 5 (AUTH-03/04); xAI-only paths may keep working if already present
- **No Codex OAuth or ChatGPT tokens** in this phase — reservation only
- User-facing rebrand of login chrome/strings → **Phase 8**, except fixes required so paths/CLI name are not wrong for bum

### Claude's Discretion
- Exact JSON schema versioning / serde shape for `providers` map (field names, optional envelope version)
- Whether legacy flat map is rewritten on first successful write (eager migrate) vs read-only migration until next write
- How deep to thread provider-aware types into `AuthManager` vs storage-only adapter for this phase
- Test layout (unit vs integration) and any minimal fixture IdP for OAuth write path

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `crates/codegen/xai-grok-shell/src/auth/` — full OIDC/OAuth2 login, device-code, refresh, AuthManager, storage lock/write
- `crates/codegen/xai-grok-auth` — `HttpAuth` / `AuthCredentialProvider` traits and middleware
- `AuthStore` / `GrokAuth` / scope keys (`API_KEY_SCOPE`, issuer URLs) in `auth/model.rs`
- `auth/storage.rs` — `auth.json` read/write, corrupt backup, `auth.json.lock` RAII
- CLI: `bum login` via pager-bin → `run_cli_login` with `--oauth` / `--device-auth`
- Phase 1: product home is `~/.bum` / `BUM_HOME` only (no `GROK_HOME` as product home)

### Established Patterns
- Credentials as JSON map of scope → `GrokAuth` (legacy flat); needs provider nesting without losing refresh/lock semantics
- Proactive token refresh with sleep-gate; lock revalidation before spend-refresh / write
- API key as first-class scope `xai::api_key` alongside OIDC entries
- Tests use temp product home via `BUM_HOME` (Phase 1 test-support)

### Integration Points
- Shell AuthManager feeds sampler / HTTP middleware for agent turns
- Pager auth dispatch / login UI (TUI) for interactive login
- Config/env: auth path under product home; `XAI_API_KEY` env fallback still relevant
- Phase 5 will fill `providers.codex`; Phase 4 will route by model → provider → slot

</code_context>

<specifics>
## Specific Ideas

- Prefer evolving the existing single-file store over a multi-file credential layout
- “Reserved” Codex slot means architecture readiness, not a half-implemented second OAuth
- Agent-turn proof should be automated so CI catches slot clobber and xAI auth regressions

</specifics>

<deferred>
## Deferred Ideas

- Codex/ChatGPT OAuth + dual logout/status/refresh independence → Phase 5
- Provider-aware request routing → Phase 4
- Mixed model catalog / GPT-5.6 entries → Phase 3
- Missing-provider gate on model switch → Phase 6
- Full product string rebrand for auth chrome → Phase 8
- Import from `~/.grok` / stock Codex stores → out of v1 (AUTH-V2-02)
- Multiple OAuth accounts per provider → AUTH-V2-03

</deferred>
