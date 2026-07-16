# Phase 5: Codex OAuth & dual auth lifecycle - Context

**Gathered:** 2026-07-16
**Status:** Ready for planning
**Mode:** Smart discuss (recommended answers locked — interactive gate declined; yolo defaults)

<domain>
## Phase Boundary

Deliver first-class **ChatGPT/Codex OAuth** for bum and a **dual-provider auth lifecycle**: login, logout, status, and independent token refresh for **xAI** and **Codex**, with credentials only under the bum multi-slot store (`providers.xai` / `providers.codex`).

In scope (AUTH-02..05):
- Codex PKCE browser login + device-code where applicable
- Selective per-provider logout without clearing the other slot
- Per-provider auth status (logged in / usable)
- Independent access-token refresh per provider (no cross-wipe)

Out of scope (later phases):
- Mid-session model-switch gate / polished missing-provider prompt UX → **Phase 6**
- Cross-provider subagent spawn credentials → **Phase 7**
- Full rebrand of login chrome / help strings → **Phase 8** (except fixes required so paths/CLI names are not wrong)
- Import or share of stock `~/.codex` / `~/.grok` credentials → **never v1**
- OpenAI Platform API-key as primary Codex path → **not product path** (Phase 4 locked)

</domain>

<decisions>
## Implementation Decisions

### Login entry surface
- Extend CLI so users can target a provider: **`bum login --provider codex|xai`** (stable ids match Phase 2 slots). Bare **`bum login` remains xAI-only** (Phase 2 locked) — do not break that default.
- Codex login defaults to **browser PKCE** (Codex-style loopback); support **`--device-auth` / `--device-code`** for headless/SSH, mirroring xAI flags and openai/codex device flow.
- Ship **CLI login as the hard requirement** for AUTH-02; wire a **minimal TUI/in-app path** if the existing login surface can select provider without Phase 6 gate work — do not block the phase on rich dual-login chrome.
- Re-login / force re-auth **overwrites only that provider’s slot**; never clears or rewrites the other provider’s credentials.

### Logout & status (AUTH-03 / AUTH-04)
- Selective logout: **`bum logout --provider codex|xai`** clears only that slot. Offer **`bum logout --all`** to clear both when explicit. Bare **`bum logout` must not silently wipe both** — either require `--provider` / `--all` or print clear usage (fail closed).
- Status: first-class **`bum auth status`** (or equivalent subcommand) listing **both** providers with usable/logged-in state.
- Status fields: provider id, logged-in yes/no, usability (token present + not known-invalid), optional email/account/plan labels when claims exist — **never print access/refresh tokens or raw secrets**.
- TUI logout/status may stay light this phase if CLI covers AUTH-03/04; do not regress existing xAI logout paths into dual-wipe.

### Independent refresh & dual lifecycle (AUTH-05)
- Each provider has an **independent refresh chain** (own tokens, expiry, invalidation). Refreshing Codex must not touch xAI and vice versa.
- On `invalid_grant` / permanent auth failure for one provider, **wipe or mark invalid only that provider’s slot** — never the whole `auth.json` dual map.
- Prefer extending the existing multi-slot **AuthManager / storage** so dual-session health is one store with per-slot operations (not a second process or shared “active key” overwrite).
- **No import** from `~/.codex` or stock Grok stores (Phase 1/2 non-negotiable). Credentials live only under `$BUM_HOME/auth.json`.

### OAuth contract & verification
- Implement Codex OAuth **in-tree**, mirroring openai/codex `codex-rs/login` (PKCE S256, public client_id, authorize/token endpoints). Prefer research defaults: loopback **`127.0.0.1:1455`** with **1457** fallback; device-code APIs as documented in `.planning/research/STACK.md`.
- Persist under **`providers.codex`**: access token, refresh token, expiry, and ChatGPT account id / claims needed for `ChatGPT-Account-ID` on sampling (Phase 4 routing already expects Codex slot).
- **Primary product path = ChatGPT OAuth**, not Platform API key. Do not market or default to OpenAI API-key billing for GPT models.
- Prove AUTH-02..05 with **automated tests** (fixture/mock IdP or injected token shapes, multi-slot isolation, selective logout, independent refresh/wipe). Live ChatGPT browser login is optional manual smoke only.

### Claude's Discretion
- Exact clap shape (`--provider` vs positional), whether `bum auth login|logout|status` nests under an `auth` parent, and TUI depth beyond minimal wiring
- Module placement for Codex PKCE/device-code (new `auth/codex/` vs extend existing oidc/device modules)
- AuthManager internal threading for dual refresh (single manager with slot param vs thin dual handles)
- Wire details: originator tag, exact scope list, JWT claim parsing, header injection seam reuse from Phase 4
- Test layout (unit vs integration binary) and mock HTTP vs pure storage assertions

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- Multi-slot store already in place: `providers.xai` / `providers.codex` constants (`PROVIDER_XAI`, `PROVIDER_CODEX`), atomic write + `auth.json.lock` (`shell/src/auth/storage.rs`, `model.rs`)
- xAI browser OIDC + device-code: `auth/oidc/`, `auth/device_code.rs`, `run_cli_login` / `run_cli_logout`
- CLI: `Command::Login { oauth, device_auth, .. }` — bare `bum login` defaults to xAI (Phase 2 tests)
- Phase 4 routing: model → provider → base_url + credential slot (Codex path ready for real tokens)
- Research contract: `.planning/research/STACK.md` Codex OAuth section (PKCE, device-code, store shape)

### Established Patterns
- Provider ids are short wire strings: `"xai"` | `"codex"`
- Slot isolation on write is already tested (concurrent xAI/Codex store updates)
- Credentials resolve per request via AuthManager / SharedApiKeyProvider-style seams
- Fail closed preferred over silent wrong-credential (Phase 4/6)

### Integration Points
- `xai-grok-pager-bin` main matches `Command::Login` / logout → `xai_grok_shell::auth::*`
- Pager TUI dispatch: `dispatch_login` / `dispatch_logout` (today xAI-centric)
- Sampler already accepts Codex base URL + bearer once credentials exist
- Do not write product auth under `~/.grok` or `~/.codex`

</code_context>

<specifics>
## Specific Ideas

- Mirror official Codex CLI login behavior closely enough that a ChatGPT Plus/Pro user gets the same subscription-backed coding path, not Platform API billing.
- Status output should be greppable and safe to paste into support (no secrets).
- Prefer extending existing login flag surface (`--oauth`, `--device-auth`) with `--provider` rather than inventing a second top-level command family unless clap nesting is cleaner.

</specifics>

<deferred>
## Deferred Ideas

- Polished mid-session “login to Codex” gate when selecting a GPT model → Phase 6 (MOD-06)
- Cross-provider subagent auth inheritance / fail-closed child login → Phase 7
- Full login chrome rebrand (strings still saying Grok where harmless) → Phase 8
- Optional OpenAI Platform API-key secondary path → AUTH-V2 / out of v1 primary
- Import stock `~/.codex` auth.json → never v1

</deferred>
