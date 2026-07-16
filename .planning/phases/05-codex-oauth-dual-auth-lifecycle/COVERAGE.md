# API Coverage — ChatGPT / Codex OAuth (auth.openai.com)

> Full coverage by default. Opt-outs are explicit, reasoned decisions.
> Service: OpenAI Auth / ChatGPT OAuth used by Codex CLI (`https://auth.openai.com`).
> Phase: 05-codex-oauth-dual-auth-lifecycle (AUTH-02..05).
> Product path: **ChatGPT OAuth** into `$BUM_HOME/auth.json` `providers.codex` — not Platform API key billing.

| capability | decision | reason |
|---|---|---|
| authorize (browser PKCE S256) | INTEGRATE | AUTH-02 primary login; ports 1455→1457; redirect `http://localhost:{port}/auth/callback` |
| token (authorization_code exchange) | INTEGRATE | AUTH-02 complete login; form POST `{issuer}/oauth/token` |
| refresh (refresh_token grant) | INTEGRATE | AUTH-05 independent Codex refresh; fixed endpoint, no OIDC discovery |
| device usercode | INTEGRATE | AUTH-02 headless/SSH via `--device-auth`; `{issuer}/api/accounts/deviceauth/usercode` |
| device token poll | INTEGRATE | AUTH-02 complete device flow; `{issuer}/api/accounts/deviceauth/token` |
| claims / account-id parse | INTEGRATE | Persist `chatgpt_account_id` → `GrokAuth.organization_id`; feed `ChatGPT-Account-ID` for multi-workspace |
| revoke (oauth/revoke) | OPT-OUT | Local slot clear only in v1 — matches existing xAI logout; remote revoke optional later |
| obtain_api_key / Platform token-exchange | OPT-OUT | Not product path — ChatGPT OAuth bearer to ChatGPT Codex backend (Phase 4); Platform API key rejected by CONTEXT/STACK |
| OS keyring credential backend | OPT-OUT | v1 locked to file `0600` auth.json under `$BUM_HOME` (Codex keyring parity not required) |
| import stock `~/.codex` auth.json | OPT-OUT | Never v1 — product isolation non-negotiable |
| import stock `~/.grok` credentials | OPT-OUT | Never v1 — product isolation non-negotiable |
| OpenAI Platform API-key primary login | OPT-OUT | Explicitly not product path (AUTH-V2 / deferred); ChatGPT OAuth primary |

```coverage
[
  {"capability": "authorize (browser PKCE S256)", "decision": "INTEGRATE", "reason": ""},
  {"capability": "token (authorization_code exchange)", "decision": "INTEGRATE", "reason": ""},
  {"capability": "refresh (refresh_token grant)", "decision": "INTEGRATE", "reason": ""},
  {"capability": "device usercode", "decision": "INTEGRATE", "reason": ""},
  {"capability": "device token poll", "decision": "INTEGRATE", "reason": ""},
  {"capability": "claims / account-id parse", "decision": "INTEGRATE", "reason": ""},
  {"capability": "revoke (oauth/revoke)", "decision": "OPT-OUT", "reason": "Local slot clear only in v1 — matches xAI logout"},
  {"capability": "obtain_api_key / Platform token-exchange", "decision": "OPT-OUT", "reason": "Not product path — ChatGPT OAuth primary"},
  {"capability": "OS keyring credential backend", "decision": "OPT-OUT", "reason": "v1 file store under BUM_HOME locked"},
  {"capability": "import stock ~/.codex auth.json", "decision": "OPT-OUT", "reason": "Never v1 — product isolation"},
  {"capability": "import stock ~/.grok credentials", "decision": "OPT-OUT", "reason": "Never v1 — product isolation"},
  {"capability": "OpenAI Platform API-key primary login", "decision": "OPT-OUT", "reason": "Deferred AUTH-V2; not product path"}
]
```

## Plan mapping

| capability | Plan |
|---|---|
| authorize + token + device + claims | 05-03 |
| refresh + account-id header inject | 05-05 |
| status/logout use store only (no remote revoke) | 05-02, 05-04 |
| Wave 0 RED + phase gate | 05-01, 05-06 |
