# Authentication

bum supports provider-scoped interactive browser login, enterprise single sign-on (SSO), API keys, and headless CI/CD runners.

## Provider capability contract

bum uses ChatGPT/Codex OAuth and compatible model APIs, but it is not the stock OpenAI Codex CLI. Provider and model names such as ChatGPT/Codex, GPT-5.6, xAI, and Grok identify the services and models used by bum; they do not change the product or executable identity.

| Area | xAI/Grok in bum | ChatGPT/Codex in bum | Compatibility boundary |
|------|-----------------|-----------------------|------------------------|
| Product and auth | Uses the xAI OAuth slot. | Uses a separate ChatGPT/Codex OAuth slot. | Both slots live only in the bum product home (`BUM_HOME`, default `~/.bum`); one provider's credentials do not replace the other's. |
| Transport | Uses bum's provider-routed HTTP path. | Responses are sent with HTTP POST and streamed with SSE. | Responses WebSocket incremental transport is deferred and non-blocking; the supported daily-driver path remains HTTP/SSE. |
| Conversation continuity | bum preserves compatible session history when models are switched. | The supported HTTP path uses `store: false` and resends full input rather than relying on stored response IDs. | This does not claim WebSocket prefix reuse or stock Codex continuity behavior. |
| Request identity metadata | xAI routes do not receive reserved Codex identity metadata. | A trusted Codex OAuth route receives bum-owned `originator` and session metadata. | BYOK and custom endpoints do not inherit that metadata; bum never impersonates the stock Codex CLI. |
| Standard bum tools | A normal session keeps the bum tool harness across compatible model switches. | Selecting a Codex model does not silently replace the harness with stock Codex CLI tool names. | Tool availability remains controlled by the active bum agent configuration and permissions. |
| Optional Codex preset | Not applicable. | Includes bum's bash tool and Codex-derived read, `apply_patch`, list, and grep ports. | This is an optional bum preset, not the stock Codex CLI. |
| Patch shape | Uses the edit tools configured for the active bum preset. | `codex:apply_patch` provides behavioral patch compatibility through a JSON `{ patch }` field. | It is not a promise of exact stock wire format, tool naming, or stronger path containment than bum's existing permission and sandbox checks prove. |
| Deferred parity | No change to the supported xAI path. | Broad stock Codex tool-name parity is deferred and non-blocking. | Deferred parity does not block productive shell, read, search, or edit workflows. |
| Live validation | The existing dual-provider daily-driver flow remains the baseline. | Phase 10 already proved the live HTTP/SSE Codex daily-driver path. | This documentation-only phase does not repeat the live dual-login gate because it changes no runtime or wire behavior. |

---

## Browser Login (Default)

On first launch, bum opens your browser to authenticate with grok.com:

```bash
bum
```

bum stores provider-scoped credentials in `~/.bum/auth.json` and reuses them across sessions. bum refreshes access tokens automatically in the background. When a token can't be refreshed, bum prompts you to sign in again. Credentials without a server-provided expiry fall back to a 30-day lifetime.

### Re-authenticate

To switch accounts or resolve an authentication problem, select the provider
slot explicitly:

```bash
bum login --provider xai
bum login --provider codex
bum login --provider codex --device-auth
```

Bare `bum login` is retained as an xAI-only compatibility shortcut. Provider
selection (`--provider xai|codex`) is separate from transport selection. By
default, login opens the selected provider's browser flow. Use a transport flag
when that provider supports the alternate flow:

| Flag | Description |
|------|-------------|
| `--oauth` | Use the browser OAuth flow. For xAI this signs in at `auth.x.ai`; the flag is optional because browser OAuth is the default. |
| `--device-auth` (alias `--device-code`) | Sign in with the device-code flow for headless or remote environments. |

Sign out of one provider slot, or explicitly clear both:

```bash
bum logout --provider xai
bum logout --provider codex
bum logout --all
```

Bare `bum logout` is rejected and does not mutate credentials. A provider-scoped
logout preserves the other provider's slot.

---

## API Key

For CI/CD, automation, or environments without browser access, use an API key from [console.x.ai](https://console.x.ai):

```bash
export XAI_API_KEY="xai-..."
bum
```

bum uses the API key as a fallback when no xAI session token is active. If you have already signed in interactively, the stored session token takes precedence. To fall back to the API key, sign out of the xAI provider slot or remove that slot from `~/.bum/auth.json` with bum's auth commands.

---

## OIDC (Customer SSO)

Authenticate developers through your own Identity Provider (IdP) -- such as Okta, Azure AD, or Auth0 -- instead of grok.com.

### 1. Register a public client in your IdP

- Grant type: Authorization Code with PKCE (Proof Key for Code Exchange)
- Redirect URI: `http://127.0.0.1/callback` -- a loopback address. bum binds a random port at sign-in time, and most IdPs treat the loopback redirect as port-agnostic per [RFC 8252](https://tools.ietf.org/html/rfc8252).
- No client secret. PKCE replaces it.

### 2. Configure the CLI

Via config file:

```toml
# ~/.bum/config.toml
[grok_com_config.oidc]
issuer = "https://acme.okta.com"
client_id = "0oa1b2c3d4e5f6g7h8i9"
```

Or via environment variables:

```bash
export GROK_OIDC_ISSUER="https://acme.okta.com"
export GROK_OIDC_CLIENT_ID="0oa1b2c3d4e5f6g7h8i9"
```

You can also override the API endpoint to point at your own proxy:

```bash
export GROK_CLI_CHAT_PROXY_BASE_URL="https://grok-proxy.acme.com/v1"
```

### 3. Run `bum`

The CLI discovers endpoints via `{issuer}/.well-known/openid-configuration`, opens the IdP login page, and stores tokens in `~/.bum/auth.json`. Tokens auto-refresh silently via the stored `refresh_token`.

### Optional fields

| Field | Default | Notes |
|-------|---------|-------|
| `scopes` | `["openid", "profile", "email", "offline_access", "api:access"]` | `offline_access` enables silent token refresh |
| `audience` | None | Required by some IdPs (e.g., Auth0) |

---

## External Auth Provider

When browser-based login isn't possible -- for example, on sandboxed VMs, CI runners, or air-gapped networks -- delegate authentication to an external binary or script.

### How It Works

```
+--------------+     sh -c     +------------------------+
|     bum      |-------------->|  your auth binary      |
|              |               |                        |
|  reads       |<-- stdout ----|  prints token          |
|  auth.json   |               |                        |
|              |   (stderr)    |  prints status/URLs    |--> surfaced to user
+--------------+               +------------------------+
```

1. bum runs your command via `sh -c "<command>"`
2. Your binary runs whatever auth flow it needs (SSO, device code, certificate exchange)
3. **stderr** carries human-readable output, such as login URLs and status messages. bum reads stderr and surfaces it to the user; in the TUI, it turns the first `https://` URL into a clickable sign-in link.
4. **stdout** is captured by bum and saved as the access token
5. Exit 0 = success; exit non-zero = bum falls back to interactive login

### The stdout / stderr Contract

| Stream | What to print | Who sees it |
|--------|---------------|-------------|
| **stdout** | The token -- nothing else | bum (parsed and stored in auth.json) |
| **stderr** | Login URLs, status messages, errors | The user (bum reads stderr and shows the sign-in URL as a clickable link in the TUI) |

**Do not print anything to stdout except the token.** No progress messages, no debug output. bum reads stdout, trims surrounding whitespace, and parses the result as a token.

### stdout Token Format

**Bare string** -- just the raw token:

```
eyJhbGciOiJSUzI1NiIs...
```

**JSON** -- with optional refresh token and expiry:

```json
{"access_token": "eyJhbGciOi...", "refresh_token": "ref-tok", "expires_in": 3600}
```

Use JSON if your tokens expire and you want bum to automatically re-run the binary before expiry.

### Configuration

Via config file:

```toml
# ~/.bum/config.toml
[auth]
auth_provider_command = "/usr/local/bin/my-auth-provider"
auth_provider_label = "Acme Corp"   # optional -- customizes the TUI login button
auth_token_ttl = 3600               # optional -- token lifetime in seconds
```

Or via environment variables:

```bash
export GROK_AUTH_PROVIDER_COMMAND="/usr/local/bin/my-auth-provider"
export GROK_AUTH_PROVIDER_LABEL="Acme Corp"
export GROK_AUTH_TOKEN_TTL=3600
```

### Token Refresh

When bum needs to refresh an expired token, it re-runs your binary with the internal compatibility variable `GROK_AUTH_EXPIRED=1` set in the environment. Your binary can use this to take a faster silent-refresh path:

```bash
#!/bin/sh
if [ "$GROK_AUTH_EXPIRED" = "1" ]; then
    echo "Refreshing token..." >&2
    TOKEN=$(my-company-auth --refresh --silent)
else
    echo "Authenticating via Acme Corp SSO..." >&2
    TOKEN=$(my-company-auth --login --interactive)
fi

if [ -z "$TOKEN" ]; then
    echo "Authentication failed" >&2
    exit 1
fi

echo "{\"access_token\": \"$TOKEN\", \"expires_in\": 3600}"
```

### Environment Variables

| Variable | Description |
|----------|-------------|
| `GROK_AUTH_PROVIDER_COMMAND` | Path to your auth binary |
| `GROK_AUTH_PROVIDER_LABEL` | Display name on the TUI login screen (e.g., "Acme Corp") |
| `GROK_AUTH_TOKEN_TTL` | Token lifetime in seconds (for bare-string tokens without `expires_in`) |
| `GROK_AUTH_EXPIRED` | Set to `1` by bum when re-running the binary for token refresh |
| `GROK_AUTH_EARLY_INVALIDATION_SECS` | Seconds before expiry to proactively refresh (default: 300) |

---

## Device Code Flow

For headless environments (SSH sessions, Docker containers, remote VMs) where no browser is available locally:

```bash
bum login --device-auth    # or: bum login --device-code
```

This prints a URL and code to the terminal. Open the URL on any device, enter the code, and complete authentication. bum polls until the login is confirmed.

You can also implement the device-code flow through an [External Auth Provider](#external-auth-provider) for full control.

---

## Automatic Credential Refresh

bum automatically refreshes expired credentials:

- **Before expiry:** If your auth provider returned `expires_in` (JSON output) or you set `auth_token_ttl`, bum re-runs the auth binary ~5 minutes before expiry.
- **On auth error:** If the server returns 401 Unauthorized, bum refreshes the credentials and retries the request.
- **OIDC:** If a `refresh_token` is available, bum silently refreshes via your IdP without re-opening the browser.

Tune the refresh buffer:

```bash
# Refresh 5 minutes before expiry (default)
export GROK_AUTH_EARLY_INVALIDATION_SECS=300

# Disable the proactive buffer: refresh at expiry or on a 401 (set to 0)
export GROK_AUTH_EARLY_INVALIDATION_SECS=0
```

---

## Hot Reload

bum picks up changes to `~/.bum/auth.json` automatically. If you update credentials externally (for example, with a script that writes new tokens), bum uses the new credentials on the next API call without a restart.

---

## Auth Precedence

bum resolves credentials for each request in this order, highest to lowest:

1. **Per-model `api_key` or `env_key`** -- set under `[model.<name>]` in `config.toml`. Wins whenever present.
2. **Active provider token** -- obtained through browser, OIDC/OAuth2, or external-provider login and stored in `~/.bum/auth.json`.
3. **`XAI_API_KEY`** -- fallback when no session token is active.

When more than one xAI login flow is configured, bum populates that provider slot from the first available source, highest to lowest:

1. **External auth provider** (`auth_provider_command`)
2. **Enterprise OIDC** -- when OIDC is configured, through `[grok_com_config.oidc]` in `config.toml` or the `GROK_OIDC_ISSUER` and `GROK_OIDC_CLIENT_ID` environment variables
3. **SpaceXAI OAuth2 browser login** -- the default

During a session, the active method handles all mid-session refreshes.

---

## Troubleshooting

### Debug logging

Set `RUST_LOG` to control the verbosity of the file log and headless stderr output. (The TUI's on-screen tracing pane uses a fixed filter and ignores `RUST_LOG`.) In the TUI, file logging defaults to `DEBUG`; in headless mode (`-p`), `RUST_LOG` defaults to `off` so only the answer is printed — set `RUST_LOG=error` (or broader) to see logs on stderr.

In the TUI, set `GROK_LOG_FILE` to an absolute path to write logs to that file:

```bash
GROK_LOG_FILE=/tmp/bum.log RUST_LOG=debug bum
tail -f /tmp/bum.log
```

`GROK_LOG_FILE` is treated as a literal file path. A relative value such as `1` writes a file named `1` in the current directory.

In headless mode, logs go to stderr. Redirect them to a file:

```bash
RUST_LOG=debug bum -p "hello" 2> /tmp/bum.log
```

### Common log messages

| Log message | What it means |
|-------------|---------------|
| `auth: running external auth provider` | bum is running your binary |
| `auth: external auth provider returned fresh token` | bum parsed and stored the token |
| `auth: external auth provider failed` | Binary exited non-zero or stdout was empty |
| `auth: external auth provider timed out (likely needs interactive auth), killing` | Binary did not exit before the timeout and was killed |
| `auth: failed to start external auth provider` | Command could not be spawned (binary not found) |

### Common fixes

- **"Authentication failed"** -- Run `bum logout` to clear the selected provider's cached credentials, then `bum login` to sign in again.
- **Token expires too quickly** -- Set `auth_token_ttl` or return `expires_in` in your auth provider's JSON output.
- **OIDC redirect fails** -- Ensure your IdP allows loopback redirect URIs (`http://127.0.0.1/callback`).
- **External auth provider not found** -- Check that the `auth_provider_command` path is correct and the binary is executable.
