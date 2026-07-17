# NeevCloud — Testing & Verification

Companion to `01-protocol-reference.md` (verified wire ground truth), `05-security-and-privacy.md`, and `07-wire-fidelity.md` (owner of the fidelity spec, constants module, and the sampler-clobber analysis — §4c/§4h/§8 here test what 07 specifies). (Docs `03-auth-slot.md` / `04-routing.md` / `05-implementation-plan.md` referenced by earlier drafts do **not** exist in this tree.)

**Path shorthand used below** (bare filenames are ambiguous — there is no `xai-grok-shell/src/config.rs`):
`config.rs`, `models.rs`, `cli_models.rs` → `crates/codegen/xai-grok-shell/src/agent/{config,models}.rs` and `src/cli_models.rs`;
`model.rs`, `status.rs`, `storage.rs`, `meta.rs`, `flow.rs`, `token_type.rs`, `device_code.rs`, `codex/*` → `crates/codegen/xai-grok-shell/src/auth/`;
`lifecycle.rs`, `actions.rs`, `app_view.rs`, `cli.rs` → `crates/codegen/xai-grok-pager/src/app/`.

The thesis, same as Phase 5's: **the OAuth is not the hard part; slot isolation is.** Every test below is aimed at one of four failure classes that this port can actually hit:

| Class | Why it bites | Test that catches it |
|---|---|---|
| Any unexpected 403 | Presents as 403, reads as auth bug; the copied poll loop treats it as *pending* and spins 15 min | `neev_poll_surfaces_403_immediately` (unit) |
| Silent xAI fallthrough | `matches!(provider, Codex)` guards compile fine, wrong for a 3rd variant | `neev_login_leaves_xai_and_codex_slots_intact` (integration) |
| `verification_uri_complete` is a PATH | Copy-pasting `codex_device_verify_url` prints a broken link | `neev_verify_url_concatenates_path_onto_base` (unit) |
| `/api/config` 404 → `Err` instead of `None` | A config-less org fails closed forever | `config_404_is_none_not_error` (unit) |
| Identity-B UA clobbered by the sampler | Nothing fails — every response is 200 either way (§8a); the fidelity decision is silently un-implemented | `extra_headers_user_agent_survives` (unit, §4h) + the capture-diff (§8b) |

---

## 1. Mock-server stack — match what's already here

**Do not add wiremock or mockito to `xai-grok-shell`.** Both are workspace deps (`Cargo.toml:271` wiremock 0.6, `Cargo.toml:172` mockito 1), but neither is a dev-dep of `xai-grok-shell` — its `[dev-dependencies]` block (`crates/codegen/xai-grok-shell/Cargo.toml:183-211`) includes `criterion`, `filetime`, `tempfile`, `serial_test`, `xai-grok-test-support`, `xai-test-utils`, `tokio` + `test-util` (plus `ring`/`tar`/`flate2`/`rsa`/`semver` and several workspace crates), but no HTTP mock crate. `axum` is a **regular** dep (`xai-grok-shell/Cargo.toml:80`), and the auth tests hand-roll axum routers against `TcpListener::bind("127.0.0.1:0")`.

Two existing harnesses to copy verbatim:

- **Inline unit tests** → `spawn_token_server(responses: Vec<(u16, serde_json::Value)>) -> (String, JoinHandle)` at `crates/codegen/xai-grok-shell/src/auth/device_code.rs:789`. Serves responses in order, repeating the last. `run_poll` (`device_code.rs:834`) wraps it. Note the comment at `device_code.rs:830-833`: **real time, not `start_paused`** — the shared client's 30s connect_timeout fires under auto-advance.
- **Integration tests** → the `MockState { polls: Arc<AtomicUsize> }` + `Router::new().route(...).with_state(st)` pattern in `codex_device_pending_slowdown_exchange_denied` (`crates/codegen/xai-grok-shell/tests/auth_codex_lifecycle.rs:287-365`). Asserts poll count AND byte-equality of `auth.json` before/after.

Adding wiremock to this crate would be a new dev-dep for what 40 lines of axum already do, in a file next to five existing examples.

---

## 2. The curl ladder — manual smoke test

Six rungs, each provable independently. The `-A 'neev/latest/0.0.2/cli'` below matches the identity the port adopts (`07 §2A`); it is **not a requirement** of the server — see rung 0 for the measured matrix. Rung 0 is worth running once so every later 403 is unambiguous.

### Rung 0 — measure what Cloudflare actually bans (do this first)

Measured live against `/api/orgs`, authenticated:

```bash
BASE=https://code.neevcloud.com
curl -sS -o /dev/null -w '%{http_code}\n' "$BASE/api/orgs"                            # 200 — curl's default UA
curl -sS -o /dev/null -w '%{http_code}\n' -A 'neev/latest/0.0.2/cli' "$BASE/api/orgs" # 200
curl -sS -o /dev/null -w '%{http_code}\n' -A ''                      "$BASE/api/orgs" # 200 — empty UA
curl -sS -o /dev/null -w '%{http_code}\n' -A 'Python-urllib/3.13'    "$BASE/api/orgs" # 403 — error_code 1010, browser_signature_banned
```

Cloudflare bans a **short list of known-bot UA signatures** (`Python-urllib/*`). It does **not** block unknown or absent UAs. The same urllib client given any other UA (even `curl/8.0`) returns 200. The inference gateway (`/zen/go/v1`) is not UA-gated at all. A Rust `reqwest` client never sends `Python-urllib`, so this is not a risk for the port.

**403 vs 401 is still the whole signal.** 403 = Cloudflare, you never reached the app. 401 = you reached the app and it wants a token.

### Rung 1 — device code

```bash
curl -sS -A 'neev/latest/0.0.2/cli' -H 'Content-Type: application/json' \
  -d '{"client_id":"neev-cli","client":"neev CLI 0.0.2 on linux"}' \
  "$BASE/auth/device/code" | jq
```
Returns `{device_code, user_code, verification_uri_complete, expires_in, interval}`. **`verification_uri_complete` is a path** — the URL to open is `"$BASE$verification_uri_complete"`.

### Rung 2 — poll

```bash
curl -sS -A 'neev/latest/0.0.2/cli' -H 'Content-Type: application/json' \
  -d '{"grant_type":"urn:ietf:params:oauth:grant-type:device_code","device_code":"'"$DEVICE_CODE"'","client_id":"neev-cli"}' \
  "$BASE/auth/device/token" | jq
```
Pending → RFC 8628 error body (`authorization_pending` / `slow_down`). Success → `{access_token, refresh_token, expires_in}`.

### Rungs 3-6 — authenticated (see §3 for the token-handling rules)

```bash
curl -sS -A 'neev/latest/0.0.2/cli' -K /dev/fd/3 "$BASE/api/orgs" 3<<<"header = \"Authorization: Bearer $TOK\""     # → [{"id":"wrk_01…","name":"Default"}]
curl -sS -A 'neev/latest/0.0.2/cli' -K /dev/fd/3 -H "x-org-id: $ORG" "$BASE/api/user"   3<<<"header = \"Authorization: Bearer $TOK\""
curl -sS -A 'neev/latest/0.0.2/cli' -K /dev/fd/3 -H "x-org-id: $ORG" "$BASE/api/config" 3<<<"header = \"Authorization: Bearer $TOK\""
```

`/api/config` → `.config.provider.opencode.options.{baseURL,apiKey}`. **404 here is normal** = no org config → `None`, not an error.

Rung 6 — the gateway, the only rung that costs tokens:

```bash
curl -sS -A 'neev/latest/0.0.2/cli' -K /dev/fd/3 -H 'Content-Type: application/json' \
  -d '{"model":"minimax-m2","max_tokens":20,"messages":[{"role":"user","content":"say ok"}]}' \
  "https://code.neevcloud.com/zen/go/v1/chat/completions" \
  3<<<"header = \"Authorization: Bearer $GATEWAY_KEY\"" | jq '.model, .usage'
```
Expect `"MiniMax-M2"` (note the casing flip vs the request's `minimax-m2`) and a `usage` block with `completion_tokens_details.reasoning_tokens`.

---

## 3. Exact reproduction commands used to verify the protocol

### 3a. Read the token out of neev's SQLite store

neev stores it **0644 world-readable** — that's gotcha #5, the thing bum must not copy.

```bash
ls -l ~/.local/share/opencode/auth.db          # note the 0644 mode; this is the weakness
sqlite3 -readonly ~/.local/share/opencode/auth.db '.tables'
sqlite3 -readonly ~/.local/share/opencode/auth.db 'select key, substr(value,1,12)||"…" from auth;'
```
Read the value into a shell var without echoing it:
```bash
TOK=$(sqlite3 -readonly ~/.local/share/opencode/auth.db "select json_extract(value,'\$.access')  from auth where key like '%neev%';")
```
> `~/.local/share/opencode/auth.db` and the `auth` table/column layout are **NOT VERIFIED** in this doc's own session record — the SHA256 identity of access_token/refresh_token/apiKey is. Confirm the path with `.tables` before trusting the query; `~/.neev/` is the other candidate.

The SHA256 identity check (gotcha #2 — all three are the same 67-char `sk-…`):
```bash
printf %s "$TOK" | sha256sum
printf %s "$GATEWAY_KEY" | sha256sum   # from /api/config options.apiKey — identical
```

### 3b. The curl config-file pattern — keeps the token out of argv/ps

`curl -H "Authorization: Bearer $TOK"` puts the token in `/proc/*/cmdline`, visible to `ps` for every user on the box, and in shell history. Use `-K` with a config file on a fd (never a temp file on disk):

```bash
curl -sS -A 'neev/latest/0.0.2/cli' -K /dev/fd/3 "$BASE/api/user" 3<<<"header = \"Authorization: Bearer $TOK\""
```
`ps aux | grep curl` then shows only `-K /dev/fd/3`. Equivalent with a heredoc for multi-header calls:
```bash
curl -sS -A 'neev/latest/0.0.2/cli' -K - "$BASE/api/config" <<EOF
header = "Authorization: Bearer $TOK"
header = "x-org-id: $ORG"
EOF
```

Post-verification hygiene: `unset TOK GATEWAY_KEY; history -d $((HISTCMD-1))`. Never paste a real value into an issue, a plan doc, or a test fixture — fixtures use `sk-fake-…`.

---

## 4. Unit tests (inline `#[cfg(test)] mod tests`)

Live in `src/auth/neev/{device,refresh,console}.rs`. Behavior-phrase names, no `test_` prefix (`.planning/codebase/TESTING.md:40-46`).

### 4a. The URL quirk — the cheapest test in the set

```rust
#[test]
fn verify_url_concatenates_path_onto_base() {
    // Server returns a PATH, not an absolute URL. Codex's codex_device_verify_url
    // (auth/codex/device.rs:40) builds from a fixed suffix — a structural copy
    // prints a broken link.
    assert_eq!(
        super::neev_device_verify_url("https://code.neevcloud.com/", "/auth/device?code=ABCD"),
        "https://code.neevcloud.com/auth/device?code=ABCD",
    );
}
```
Pair it with the guard: concatenate **first**, then run the result through `validate_verification_uri` (`auth/device_code.rs:521`) — a bare path fails its `url::Url::parse` + https check. Do not delete the guard to make it pass.

### 4b. Poll loop — mirror `device_code.rs:789-900`

Four cases, one `run_poll` helper each:

```rust
async fn run_poll(responses: Vec<(u16, serde_json::Value)>) -> (Result<GrokAuth, NeevLoginError>, tempfile::TempDir) {
    let (base, server) = spawn_neev_server(responses).await;  // routes /auth/device/code + /auth/device/token
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("auth.json");
    seed_dual_fixture(&path);  // xai + codex slots, so isolation is provable
    let result = super::run_neev_device_poll_only_with_base(&path, &base, NEEV_CLIENT_ID).await;
    server.abort();
    (result, dir)
}

#[tokio::test] async fn poll_succeeds_on_first_poll() { … }
#[tokio::test] async fn poll_succeeds_after_authorization_pending() { … }
#[tokio::test] async fn poll_backs_off_on_slow_down() { … }        // assert interval += 5s
#[tokio::test] async fn access_denied_writes_nothing() { … }        // assert auth.json bytes unchanged
#[tokio::test] async fn expired_token_names_the_retry_command() { … }
```

Two constraints inherited from the existing harnesses:
- **Real time, not `start_paused`** — `device_code.rs:830-833` documents why: the shared client's 30s connect_timeout fires under auto-advance. Serve `interval: 1` from the mock.
- **Sleep-first ordering** (`codex/device.rs:201`, rationale at `device_code.rs:231-232`). Assert it: with `interval: 1` and a success-on-first-response mock, elapsed ≥ 1s.

Deadline expiry is untestable in real time (floored at `MIN_DEVICE_CODE_EXPIRY_FALLBACK_SECS = 10 * 60`, `device_code.rs:22`) — `device_code.rs:830-833` says so explicitly. Don't try.

### 4c. Wire fidelity — the headers must be present and exactly-valued

The port impersonates the neev CLI on the wire (adopted decision; spec and constants: **`07-wire-fidelity.md`**). Nothing below is *required* by the server — see §8a for the measured 4-result table — so these are **should**-tests, not must-tests. They exist because fidelity is unverifiable by response code: a broken header set still returns 200, so a silent regression is invisible without an assertion.

Three layers, cheapest first:

1. `fidelity.rs`'s own unit tests — owned by `07 §3`/`§6` (constants match the captured strings; `gateway_headers` deliberately carries no UA; id shape + uniqueness). Don't restate them here.
2. **Per-path header assertions** (below) — that the request builders actually attach what `fidelity.rs` produces.
3. **The sampler-survival test** (§4h) — the one regression that silently breaks identity B.

Console path (identity A), against the axum mock from §1 — capture the request headers in the handler instead of asserting on a `RequestBuilder` (reqwest exposes no read-back before `send`):

```rust
#[tokio::test]
async fn console_calls_send_the_neev_identity() {
    let seen = Arc::new(Mutex::new(HeaderMap::new()));
    let (base, server) = spawn_capturing_console(seen.clone(), json!({"id": "wrk_01"})).await;
    let _ = super::neev_get_orgs(&base, "sk-fake-abcdefghij0123456789").await.unwrap();
    server.abort();

    let h = seen.lock().unwrap();
    assert_eq!(h[USER_AGENT], "neev/latest/0.0.2/cli");   // 07 §2A — exact captured string
    assert_eq!(h[ACCEPT], "*/*");
    assert_eq!(h[AUTHORIZATION], "Bearer sk-fake-abcdefghij0123456789");
    // /api/config and /api/mcp/list additionally carry x-org-id; /auth/* must not.
    assert!(!h.contains_key("x-org-id"));
}

#[tokio::test]
async fn api_config_sends_accept_json() {
    // 07 §3: console_headers(accept_json = true) — /api/config is the acceptJson caller.
    assert_eq!(headers_for_config_call()[ACCEPT], "application/json");
}
```

Exact-valued, not `contains`: a `starts_with("neev/")` assertion passes on a stale `neev/latest/0.0.1/cli`, which is exactly the drift 07 §7 warns about. If one of these fails after a version bump, **re-derive the string from a fresh capture** (§8b) rather than editing the expectation to match the code.

The env overrides (`BUM_NEEV_CONSOLE_UA`, `BUM_NEEV_GATEWAY_UA`, `BUM_NEEV_PROJECT`, `07 §3`) are process env — any test touching them needs `#[serial]` and must restore, same rule as §5's `grok_home()` note. One override test is enough; it proves the escape hatch exists so a stale pin can be fixed without a rebuild:

```rust
#[test]
#[serial]
fn console_ua_env_override_wins() {
    unsafe { std::env::set_var(ENV_CONSOLE_UA, "neev/latest/9.9.9/cli") };
    assert_eq!(console_user_agent(), "neev/latest/9.9.9/cli");
    unsafe { std::env::remove_var(ENV_CONSOLE_UA) };
}
```

(General note, unchanged: `set_client_name` (`xai-grok-http/src/lib.rs:164`) is process-global and `expect`s on second call, so it is never the tool for a per-provider UA. The console path uses a per-request `.header(USER_AGENT, …)` override — `07 §3`.)

### 4d. Expiry

```rust
#[test]
fn expires_at_comes_from_expires_in_not_a_jwt() {
    // Neev's token is an opaque 67-char sk-… — jsonwebtoken insecure_decode
    // yields nothing and codex/claims.rs:16 CONSERVATIVE_EXPIRES_FALLBACK (5 min)
    // would re-refresh a 1-year token every 5 minutes. auth/neev/claims.rs must not exist.
    let auth = super::grok_auth_from_neev_tokens(&resp_with(31_536_000), "wrk_01", "a@b.c");
    let ttl = auth.expires_at.unwrap() - chrono::Utc::now();
    assert!(ttl > chrono::Duration::days(300), "got {ttl}");
}

#[test]
fn refresh_token_is_stored_even_though_it_equals_the_access_token() {
    // token_type.rs:28-29 — AuthMode::Oidc WITHOUT a refresh_token silently
    // degrades to the unrefreshable LegacySession and status reports usable:false.
    let auth = super::grok_auth_from_neev_tokens(&resp, "wrk_01", "a@b.c");
    assert_eq!(auth.auth_mode, AuthMode::Oidc);
    assert!(auth.refresh_token.is_some());
    assert!(crate::auth::credential_usable(&auth));  // status.rs:121
}
```

### 4e. `/api/config` parsing

```rust
#[test] fn config_404_is_none_not_error() { … }               // Ok(None); precedent device_code.rs:166
#[test] fn config_parses_baseurl_and_apikey_under_opencode() {
    // The provider key is literally "opencode" (fork heritage) despite the
    // "NeevCode Zen" display name. Do not key on the display name.
}
#[test] fn config_whitelist_filters_models() { … }
#[test] fn zero_context_limit_falls_back_not_unwraps() {
    // context_window is NonZeroU64; config.rs:3479-3481 is the fallback idiom
    // (`m.context_window.unwrap_or_else(|| NonZeroU64::new(200_000)...)`).
    assert_eq!(super::context_window_or_default(0).get(), 200_000);
}
#[test] fn model_slug_is_the_lowercase_request_id() {
    // Request "minimax-m2", response "MiniMax-M2" (protocol §7). Never round-trip-compare.
}
```

### 4f. 403 must error, not spin — the real trap in the copied code

The important half is structural, not cosmetic: `codex/device.rs:249` does `if status == 403 || status == 404 { continue; }` — 403 is treated as **pending**, so *any* unexpected 403 never reaches the error arm at line 254; it silently spins until the 15-minute timeout (`codex/device.rs:198`). Do **not** carry that arm over to Neev. Assert the error surfaces on the *first* 403.

```rust
#[tokio::test]
async fn neev_poll_surfaces_403_immediately() {
    let (result, _) = run_device_code(vec![(403, json!({"error_code": 1010}))]).await;
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("blocked"), "403 must error on the first response, not `continue` \
        as pending (codex/device.rs:249): {msg}");
}
```
Defensive, ~5 lines: a named `BlockedClient` variant for 403 + `error_code: 1010`. Not because the port will hit it — it won't, the ban is a `Python-urllib` signature list — but because a bare 403 reads as an auth failure and Cloudflare rules are the vendor's to change.

### 4g. Never leak the token

`codex/device.rs:252` — `// Do not include body (may leak tokens)`. Sharper for Neev: the token **is** the inference key **is** the account session.

```rust
#[tokio::test]
async fn errors_never_echo_the_response_body() {
    let (result, _) = run_poll(vec![(500, json!({"leak": "sk-fake-abcdefghij0123456789"}))]).await;
    assert!(!result.unwrap_err().to_string().contains("sk-fake-"));
}
```
Redaction is already free: `xai-grok-secrets` `API_KEY_PREFIX_REGEX` is `\b(?:sk[-_]|xai-)[A-Za-z0-9_-]{20,}` (`sanitizer.rs:11`) — Neev's 67-char `sk-…` matches. Add no pattern. `format_auth_status` is already paste-safe and has its own guard test, `format_never_emits_token_material` (`auth/status.rs:305`) — extend it to the third row rather than writing a new one.

### 4h. The gateway UA must survive the sampler — the silent identity-B regression

`SamplingClient::new` applies `extra_headers` at `client.rs:444-450` (`headers.insert(header_name, header_value)` at `:449`), then unconditionally `headers.insert(USER_AGENT, …)` at `client.rs:500` (comment at `:490`: "Always set User-Agent"). `HeaderMap::insert` **replaces** — so identity B's UA cannot be delivered through `extra_headers` as the code stands. `07 §4` analyses the two exits and recommends the ~2-line conditional insert at `:500`; this is the test that pins it.

Lives in `xai-grok-sampler`'s inline tests, next to the existing `sampling_client_always_has_user_agent` (`crates/codegen/xai-grok-sampler/src/client.rs:2307-2312`), which asserts `default_headers` **has** a UA — it stays green under the conditional insert, and it is the reason the fidelity path must always supply one.

```rust
// Regression: extra_headers["user-agent"] must win over the composed default.
// Without this, the NeevCloud gateway silently sees `grok-shell/<ver> (os; arch)`
// instead of identity B and nothing fails — every response is still 200 (06 §8a).
#[test]
fn extra_headers_user_agent_survives() {
    const NEEV_UA: &str = "opencode/0.0.2 ai-sdk/provider-utils/4.0.23 runtime/bun/1.3.13";
    let mut cfg = minimal_config();                      // client.rs:2088
    cfg.extra_headers
        .insert("user-agent".to_string(), NEEV_UA.to_string());
    let client = SamplingClient::new(cfg).expect("build");
    assert_eq!(client.default_headers[USER_AGENT], NEEV_UA);
}

#[test]
fn default_user_agent_still_applies_without_an_override() {
    // Scopes the change: every existing caller (extra_headers empty by default,
    // client.rs:2098) keeps the grok-shell UA.
    let client = SamplingClient::new(minimal_config()).expect("build");
    assert!(client.default_headers[USER_AGENT].to_str().unwrap().starts_with("grok-shell/"));
}
```

`default_headers` is a private field (`client.rs:281`) — fine, these are inline `#[cfg(test)] mod tests` in the same file, like `sampling_client_always_has_user_agent` already is. On the bum side, one test that the `x-opencode-*` set reaches `SamplerConfig.extra_headers` with fresh ids:

```rust
#[test]
fn neev_sampler_config_carries_the_opencode_headers() {
    let cfg = super::sampling_config_for_neev(&model, "sk-fake-…");
    assert_eq!(cfg.extra_headers["x-opencode-client"], "cli");
    assert_eq!(cfg.extra_headers["x-opencode-project"], "global");
    assert!(cfg.extra_headers["x-opencode-session"].starts_with("ses_"));
    assert!(cfg.extra_headers["x-opencode-request"].starts_with("msg_"));
    assert_eq!(cfg.extra_headers["user-agent"], NEEV_GATEWAY_UA);   // dead without §4h's fix
    // Two configs from the same session share x-opencode-session, differ on -request.
}
```

If Option 1 (`origin_client`) is taken instead of the conditional insert, `extra_headers_user_agent_survives` cannot be written — the assertion becomes `default_headers[USER_AGENT] == "opencode/0.0.2 grok-shell/<ver> (linux; x86_64)"`, i.e. the composite, and the fidelity claim is downgraded to "approximate". That test-writability is itself the argument for Option 2 (`07 §4`).

---

## 5. Integration strategy — zero live account

New binary `crates/codegen/xai-grok-shell/tests/auth_neev_lifecycle.rs`, mirroring `auth_codex_lifecycle.rs`. Its module doc must restate the two hygiene rules verbatim from `auth_codex_lifecycle.rs:1-38`:

1. **`grok_home()` is a `OnceLock`** (`xai-grok-config/src/paths.rs:64`, static at `:7`) — you cannot re-point `BUM_HOME` between tests in one process. Use path-taking `*_at` APIs and `tempfile` only. `GROK_AUTH_PATH` is not an escape hatch: `resolve_auth_path` (`auth/manager.rs:276-292`) rejects anything that isn't exactly `{grok_home}/auth.json`.
2. **Name isolation** — integration test fn names must not collide with `--lib` names in the same crate (`auth_codex_lifecycle.rs:24-33`).

Everything runs against a `tempfile::tempdir()` + an axum mock console + fake `sk-fake-…` tokens.

### The one test that earns its keep

```rust
/// Slot isolation, version trap, permissions, and prune — one test.
#[tokio::test]
#[serial]
async fn neev_login_leaves_xai_and_codex_slots_intact() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("auth.json");
    write_triple_auth_fixture(&path);            // providers.{xai,codex,neev}
    let xai_before   = slot_bytes(&path, "xai");
    let codex_before = slot_bytes(&path, "codex");

    let (base, server) = spawn_neev_console(happy_path()).await;
    neev::run_neev_device_login_with_base(&path, &base, NEEV_CLIENT_ID).await.unwrap();
    server.abort();

    assert_eq!(slot_bytes(&path, "xai"),   xai_before,   "neev login must not touch providers.xai");
    assert_eq!(slot_bytes(&path, "codex"), codex_before, "neev login must not touch providers.codex");

    let doc: serde_json::Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(doc["version"].as_u64(), Some(1), "AUTH_DOCUMENT_VERSION must stay 1 (model.rs:73) — \
        bumping to 2 makes every older bum fail-closed on the WHOLE file (storage.rs:125-129)");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(std::fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0o600,
            "never copy neev's 0644 sqlite");   // same contract as storage.rs:934, :1024
    }
}
```

The mirror of `device_code.rs:883-889` (which asserts the codex slot survives an xAI device login) run in the other direction. Plus:

| Test | Asserts |
|---|---|
| `neev_device_denied_writes_nothing` | Byte-equality of `auth.json` after `access_denied` — persist only after success (`codex/device.rs:180`) |
| `logout_neev_leaves_other_slots` | `clear_provider_slot(path, Neev)` → file still present, xai/codex intact |
| `logout_all_clears_three_slots_and_deletes_file` | `AuthProvider::all()` becomes `[Self; 3]`; `ProviderStoreMutation::FileDeleted` (variant at `storage.rs:26`, returned at `:627`) |
| `auth_status_lists_three_providers` | `format_auth_status` (`status.rs:234`) emits a NeevCloud row — it iterates, so this catches a missed element in the `providers` arrays at `status.rs:77` and `:93` |
| `bare_logout_still_fails_closed` | `run_cli_logout_at_path` (`flow.rs:1228`) bare → `Err` + usage, zero mutation (`flow.rs:1234-1238`) |
| `neev_meta_slot_defaults_for_old_clients` | `ProviderAuthMetaSlots` deserializes without the `neev` field (`#[serde(default)]`, regression test at `meta.rs:175`) |

### The silent-fallthrough tests — the ones the compiler won't write for you

`AuthProvider::all()` (`model.rs:61`, today `[Self; 2]`), `AuthStatusReport.providers` (`status.rs:38`, today `[ProviderAuthStatus; 2]`), and `ProviderAuthMetaSlots::from_report`'s exhaustive match (`meta.rs:43-50`) all fail loudly when widened to 3. These do **not**:

```rust
#[test] fn parse_provider_wire_id_knows_neev() {           // pager actions.rs:43 — _ => None
    assert!(parse_provider_wire_id("neev").is_some());
}
#[test] fn usable_for_wire_knows_neev() {                   // pager app_view.rs:572 — _ => None
    assert!(snapshot.usable_for_wire("neev").is_some());
}
#[test] fn neev_login_is_cli_primary_like_codex() {         // lifecycle.rs:1433 — `let is_codex = provider == "codex"`
    // "Login now" on a neev-gated model must NOT start an xAI OAuth flow.
}
#[test] fn provider_label_for_neev_is_not_xai() {
    // lifecycle.rs:302-305 is a `let provider_label = match provider { "codex" => …,
    // _ => ModelProvider::Xai.display_label() }` — an inline binding, NOT a fn.
    // Extract it to a testable fn first; a neev model currently labels as "xAI".
}
```
Before writing these, grep: `rg -n '"codex"' crates/codegen/xai-grok-pager/src crates/codegen/xai-grok-shell/src` and `rg -n 'matches!\(.*(AuthProvider|ModelProvider)::' crates/codegen`. Every hit is either a legit Codex-only branch or a bug. The lazy fix is converting each `matches!` into an exhaustive `match` *before* adding the variant — then the compiler writes the checklist and half these tests become unnecessary.

### Routing / catalog

Extend the existing files, don't add new ones:
- `tests/provider_routing.rs` — mirror `resolve_provider_route_codex_default` (`:203`) and `codex_model_routes_to_codex_backend_with_codex_token` (`:453`). Add `is_first_party_neev_url` cases: **host AND `/zen/go/v1` path prefix** — `is_first_party_codex_url`'s doc (`config.rs:4499`) says host-only is insufficient, and a hostile `/api/config` naming a foreign host is a real threat here because the server picks the host that receives the account secret.
- `tests/model_catalog.rs` — the case the injection seam exists for: neev rows present in **both** `resolve_model_list(&cfg, None)` **and** `resolve_model_list(&cfg, Some(prefetched))` (the prefetch replaces the map wholesale at `config.rs:3178`). Plus `assert!(entry.api_key.is_none())` on every neev row — `ModelEntry.api_key` is `Serialize` and `ModelsCacheManager::persist` (`models.rs:1305`) writes it to `models_cache.json`, which is not the 0600 auth store.
- `tests/model_switch_gate.rs` — a `missing_provider_gate_error(Neev, …)` row; the suggestion string auto-becomes `bum login --provider neev` via `config.rs:5557`.

### Run commands

Per-crate, filtered. Never an unfiltered workspace run (`.planning/codebase/TESTING.md:23-31`).

```bash
cargo fmt --all
cargo clippy -p xai-grok-shell
cargo test -p xai-grok-shell --test auth_neev_lifecycle -- --list
cargo test -p xai-grok-shell --test auth_neev_lifecycle neev_login_leaves_xai_and_codex_slots_intact
cargo test -p xai-grok-shell --lib auth::neev
cargo test -p xai-grok-shell --test provider_routing neev
cargo test -p xai-grok-pager --lib provider
cargo test -p xai-grok-shell --lib auth::neev::fidelity
cargo test -p xai-grok-sampler --lib user_agent      # §4h — the clobber regression
```

---

## 6. What a mock cannot replace

Everything above passes against a mock that is, by construction, whatever the implementer believed. Two things are only knowable live: the gateway leg actually samples and meters (§7), and the header set bum actually puts on the wire (§8).

### 6a. What the harness sends by default, and why it isn't enough

`shared_client()` (`xai-grok-http/src/lib.rs:282`) builds with `.user_agent(process_user_agent_string())` (`:289`), rendered by `UserAgent::render` (`:126`) as `grok-shell/<VERSION> (<os>; <arch>)`. `<VERSION>` is `xai_grok_version::VERSION` — `CARGO_PKG_VERSION` (**`0.2.0-dev`** in this tree), overridable by the `GROK_VERSION` build env.

That string *works* — rung 0 measured a console UA on the gateway returning 200, and even an empty UA passes. It is rejected for fidelity, not for function: the port presents as the neev CLI (`07 §1`), so both paths override the client default per request. The default is the fallback the §4h test proves still applies to xAI and Codex traffic.

### 6b. Live-checking a UA is a null test

```bash
curl -sS -o /dev/null -w '%{http_code}\n' -A 'grok-shell/0.2.0-dev (linux; x86_64)' https://code.neevcloud.com/api/orgs
curl -sS -o /dev/null -w '%{http_code}\n' -A 'neev/latest/0.0.2/cli'                https://code.neevcloud.com/api/orgs
```

Both 200/401. **No live request can tell you whether fidelity is correct** — that's §8's whole point, and why the assertions in §4c/§4h carry the load instead.

---

## 7. Testing against the real service — safely

Free tier: 10,000,000 token limit, `tokenUsed` observed at 67, `plan.endDate` ~1 month out. **Metering is live and immediate** — a single 20-token test call showed up in `/api/user` `plan.tokenUsed` within seconds. There is no sandbox and no dry-run.

Rules:

1. **Live tests are `#[ignore]`d and env-gated.** The tree's convention for network-touching tests is `#[ignore]` + `cargo test -p xai-grok-shell -- --ignored` (`.planning/codebase/TESTING.md:23-31`). Gate additionally on an explicit opt-in so `--ignored` on CI can never burn quota:
   ```rust
   #[tokio::test]
   #[ignore = "live NeevCloud account; costs metered tokens"]
   async fn live_gateway_smoke() {
       let Ok(_) = std::env::var("BUM_NEEV_LIVE_TEST") else { return };
       // max_tokens: 8. One call. Assert HTTP 200 + usage.total_tokens > 0.
   }
   ```
2. **`max_tokens: 8`, one message, no tools, no streaming.** The verified 20-token call cost 67 total (47 prompt). Budget ~100 tokens per live run. 10M ≈ 100k such runs — quota is not the risk; **noise in `tokenUsed` is**, because it's the only usage signal you have when debugging real overuse.
3. **Never in CI.** Live tests are a local pre-ship gate, run by a human, on the human's own account. CI has no NeevCloud credential and must not get one — the token is the whole account (gotcha #2).
4. **Read-only rungs are free.** `/api/orgs`, `/api/user`, `/api/config`, and both device endpoints cost nothing. Rungs 0-5 of the ladder can be run freely and often; only rung 6 meters.
5. **Skip `/api/usage/report`** (protocol §8). Fire-and-forget in neev, no bum analog, and reporting fake usage against a live account pollutes the meter. Out of scope until billing matters.
6. **Verify metering after a live run:**
   ```bash
   curl -sS -A 'neev/latest/0.0.2/cli' -K /dev/fd/3 -H "x-org-id: $ORG" "$BASE/api/user" \
     3<<<"header = \"Authorization: Bearer $TOK\"" | jq '.plan | {tokenUsed, tokenLimit}'
   ```
   If `tokenUsed` doesn't move, your call didn't reach the gateway — check the response status.
7. **A revoked token looks fresh for a year.** `expires_at` is ~365 days out, so `is_expired_with_buffer` (`model.rs:444`) never fires and `credential_usable` (`status.rs:121`) stays true. Revocation only surfaces as a gateway 401. Don't build revocation detection into the store; the sampler's 401 path drives re-login. Don't write a test that asserts a revoked token reports unusable — it won't.

---

## 8. Fidelity verification — capture bum, diff against neev

### 8a. Why this is a "should", not a "must"

Measured live against `https://code.neevcloud.com/zen/go/v1/chat/completions`, authenticated:

| Request | Result |
|---|---|
| bare — no UA, no `x-opencode-*` at all | **200** |
| full neev fidelity (identity B UA + all four `x-opencode-*`) | **200** |
| console UA on the gateway (mismatched identity) | **200** |
| malformed `x-opencode-session: NOT_A_VALID_ID` | **200** |

Nothing is validated or required today. Fidelity is **insurance** against NeevCloud later gating, metering, or rate-limiting on client identity — not a fix (`07 §1`). Two consequences for testing:

- **Fidelity can never be the cause of a bug.** A failing neev call is auth, routing, or the slot — look there first. Never "fix" a 4xx by tweaking a header string.
- **Every fidelity check is an assertion, never an expectation of failure.** Nothing breaks when fidelity breaks; that's the reason to assert it in code (§4c, §4h) rather than trust a smoke test.

Console API measured separately: no UA required either (curl default → 200, empty UA → 200); only the known-bot signature `Python-urllib/*` is 403'd (rung 0). reqwest never sends that. The UA is sent for **fidelity**, not to satisfy Cloudflare.

### 8b. The capture-diff ritual

The only proof of fidelity is comparing bum's own outbound bytes against neev's. **`07 §8` owns the procedure** — proxying bum (`HTTPS_PROXY` + `SSL_CERT_FILE`, since reqwest needs the CA where Bun doesn't), the `hdr.py` extraction script, and the four-row table of differences that are *expected* (`x-opencode-session`/`-request` values, absent `b3`/`traceparent`, `accept-encoding`, `content-length`). Don't re-derive it; run it.

Testing-side rules on top:

1. **Manual pre-release gate, never CI.** It needs a proxy, a live login, and a human reading the expected-difference table. Same class as §7's live gateway smoke.
2. **Recapture neev's side when the version moves.** `~/bum/neev/artifacts/` holds the captured flows; `~/bum/neev/artifacts/scripts/mitm.sh` recaptures against a current `@neevcode/neev` install (`netcap.sh`, `netcap2.sh` alongside it). A diff against a stale reference proves fidelity to last year's neev — the exact failure mode `07 §7` describes, where the insurance becomes the outage.
3. **The diff is authoritative over the constants, not the reverse.** If `console_ua_matches_captured_neev_string` (`07 §3`) fails, re-derive from a fresh capture and bump the const; don't edit the expected value to match whatever the code now emits.
4. **Never diff a real secret.** `07 §8`'s script redacts `authorization` to `Bearer sk-…`. Keep that; the diff output tends to end up pasted into issues.
5. **Read the gateway `user-agent` line first.** A `… grok-shell/<ver> (linux; x86_64)` there means the sampler clobber (§4h) is unfixed and identity B is not being sent — the single most likely fidelity defect, and the only one this ritual catches that the unit tests can't when `extra_headers` plumbing is wired but `client.rs:500` was never touched.

## 9. Troubleshooting

| Symptom | Cause | Fix |
|---|---|---|
| HTTP **403**, body `error_code: 1010`, `browser_signature_banned` | Cloudflare bot-signature ban. **Not** an auth failure. Unreachable from reqwest — it only fires on known-bot UAs like `Python-urllib/*` (rung 0). If you see it, the vendor changed a rule. | Surface it as a named `BlockedClient` error, don't retry. Check the UA you actually sent against rung 0's matrix. |
| Device login hangs ~15 min, then `device auth timed out` | You copied `codex/device.rs:249` (`403 \|\| 404 => continue`), which treats **any** 403 as `authorization_pending` | Don't copy that arm. Error on the first 403. This is the real trap — it's independent of any UA question. |
| Error reads `device auth failed with HTTP 403` | 403 fell into the generic arm (`codex/device.rs:254`) | Add an explicit 403 arm (`BlockedClient` on `error_code: 1010`) so it doesn't read as an auth failure. |
| HTTP **401** on `/api/config` or `/api/mcp/list` | Missing `x-org-id` | `GET /api/orgs` → `[0].id` (`wrk_01…`). Persist to `GrokAuth.organization_id` at login, or you re-hit `/api/orgs` every cold start. |
| HTTP 401 on `/api/orgs` | Bad/absent bearer — you reached the app. | Re-login. |
| HTTP **404** on `/api/config` | No org config. **Normal.** | `Ok(None)`, never `Err`. Precedent: `device_code.rs:166` maps 404 → typed `NotEnabled`. Map to `EnsureFreshNeevResult::Unusable`, **not** `Unavailable` — `Unavailable` means transient and retries forever (`codex/ensure_fresh.rs:44`). |
| Verification link is broken / 404s in the browser | `verification_uri_complete` is a **path**; you built an absolute URL like `codex_device_verify_url` (`codex/device.rs:40`) | `format!("{base}{path}")`, then `validate_verification_uri` (`device_code.rs:521`). |
| `validate_verification_uri` rejects the server's response | You validated the bare path before concatenating — `url::Url::parse` fails, https check fails | Concatenate first, validate second. Don't delete the guard. |
| `bum auth status` shows neev `usable: false` right after a successful login | `AuthMode::Oidc` **without** `refresh_token` degrades to `LegacySession` (`token_type.rs:28-29`) | Store the same string in `key` **and** `refresh_token`. They're byte-identical; store both anyway. |
| bum re-refreshes the token every ~5 minutes | You derived `expires_at` from JWT claims; the opaque `sk-…` yields nothing and `CONSERVATIVE_EXPIRES_FALLBACK` = 5 min (`codex/claims.rs:16`) | Take `expires_in` from the token response. `auth/neev/claims.rs` must not exist. |
| Neev models vanish after first launch, no error | The prefetch **replaces** the catalog (`config.rs:3178`); only `provider == Codex` rows are re-appended (`config.rs:3189`) | Add neev to that filter, and append **after** the whole `if let Some(prefetched)` block or `resolve_model_list(cfg, None)` callers (e.g. `cli_models.rs:40`) never see them. |
| Model switch says "no usable credentials", nothing logged | `ModelProvider::as_str()` ≠ `PROVIDER_NEEV`. The two enums are linked by convention only. | One wire id, `"neev"`, byte-identical in `cli.rs`, `PROVIDER_NEEV`, `ModelProvider::as_str`, `usable_for_wire`, the `is_cli_primary` check. |
| `--provider neev-cloud` instead of `neev` | clap `ValueEnum` kebab-cases variants; `AuthProviderArg` has no `#[value(name=…)]` overrides (`pager/src/app/cli.rs:11-17`) | Spell the variant `Neev`, or pin `#[value(name = "neev")]`. |
| Every neev sample ships an empty bearer | `session_oauth_allowed` (field `config.rs:4435`) is derived per-provider from the **final** `base_url` (`config.rs:4475-4483`) and gates the session key at `:4833`; `is_first_party_neev_url` rejected the runtime `options.baseURL`, so the new arm resolved `false` | The gateway `/zen/go/v1` is on the **same host** as the console (unlike Codex) — allowlist host + `/zen/go/v1` prefix. |
| Neev requests authenticate with the **Codex** token | `prepare_sampling_credentials(model, endpoints, xai_session_key, codex_session_key)` takes positional `Option<&str>` keys (`config.rs:4579`) and delegates to `resolve_credentials_for_provider`; a caller passes the third key in the wrong slot | Update the signature + its two production callers (`models.rs:959`, `agent_ops.rs:1115`). `subagent/mod.rs` does **not** call it — it goes through `sampling_config_for_model` (`:1036`) and imports `resolve_credentials_for_provider` (`:1011`), so widen that fn too. Test asserts the neev route gets the neev key and nothing else. |
| First model switch after login sees an empty slot | The snapshot cache keys on (path, mtime, len, epoch) (`config.rs:4718-4726`) — same-second writes collide | Call `invalidate_neev_session_key_snapshot()` after login, mirroring `invalidate_codex_session_key_snapshot` (`config.rs:4756`) and its `CODEX_SNAPSHOT_EPOCH` counter (`:4760`). |
| Neev logout wiped the xAI slot | A `matches!(provider, Xai)` guard fell through (`flow.rs:1077`/`:1085`, `extensions/auth.rs:163`) | Exhaustive `match`, not `matches!`. This is why the isolation test in §5 exists. |
| `auth.json` disappeared after a neev write | `persist_document_or_prune` deletes the file when every slot is empty (`storage.rs:622-628`) | Reject `ProviderStoreMutation::FileDeleted` after a persist as an internal error — Codex guards it explicitly at `browser.rs:205-212`. |
| Older bum binary lost its xAI + Codex logins | You bumped `AUTH_DOCUMENT_VERSION` | Don't. `version > 1` is a hard `ErrorKind::Unsupported` on the **whole file** (`storage.rs:125-129`). A new providers key is additive; old binaries return `Ok(None)` for a slot they don't know. |
| Tests pass individually, fail together | `grok_home()` is a `OnceLock` (`paths.rs:65`) — `BUM_HOME` set once per process | Path-taking `*_at` APIs + `tempfile`. `#[serial]` on anything env-touching. |
| `dev.` console silently wins over prod | `select_provider_access_token` prefers the lexicographically-smaller scope among equal ranks (`model.rs:419-423`) | Pin one scope constant, or key the scope off the resolved console URL and select deliberately. |
| `tokenUsed` didn't move after a live gateway call | The call never reached the gateway | Check the response status — a real 200 meters within seconds. |
| Capture-diff shows `user-agent: … grok-shell/<ver> (linux; x86_64)` on `/zen/go/v1` | `client.rs:500` still inserts unconditionally after `extra_headers` (`:444-450`) — identity B's UA was dropped | Apply `07 §4` Option 2 (conditional insert) and pin it with `extra_headers_user_agent_survives` (§4h). `origin_client` does **not** fix this — it yields the composite `opencode/0.0.2 grok-shell/…`. |
| A neev call 4xx's and the header set looks off | Coincidence. Nothing on either host validates UA or `x-opencode-*` (§8a: bare, mismatched, and malformed all → 200) | Fidelity is never the cause. Debug auth (`x-org-id`, bearer slot), routing, or the model id first. |
| `console_ua_matches_captured_neev_string` fails after a dep bump | Someone edited a `fidelity.rs` const | Re-derive from a fresh capture (`~/bum/neev/artifacts/scripts/mitm.sh`, §8b) — the capture is authoritative, not the code. |
| Fidelity tests fail only under `cargo test` (pass alone) | A `#[serial]`-less test left `BUM_NEEV_*_UA` set | `#[serial]` + restore on every env-override test (§4c), same rule as `grok_home()`'s in §5. |
