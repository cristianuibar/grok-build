# 07 — Wire fidelity: presenting as the neev CLI

> **This file owns the fidelity policy.** [01-protocol-reference.md § 2](01-protocol-reference.md) and
> [04-rust-implementation.md § 3](04-rust-implementation.md) establish the *measured* fact that nothing on the wire
> is required or validated — that fact stands, unchanged, and is restated here. This file adds the *policy* those
> sections defer to: fidelity is an adopted design decision (§1), so bum pins the headers anyway. Requirement and
> policy are different questions; both answers are correct simultaneously. Those sections now wire the headers via
> this file's constants module (§3) rather than defining strings of their own.

## 1. The decision, and the honest caveat

**Decision (adopted, not open):** bum's NeevCloud calls impersonate the neev CLI on the wire — same `User-Agent`,
same custom headers, same `client_id`. NeevCloud sees traffic identical to that from their own OpenCode fork.

**Rationale:** forward-compatibility insurance. If NeevCloud ever gates, meters, or rate-limits on client identity,
a byte-identical client keeps working with no code change.

**Caveat, measured, not inferred:** none of it is required today. Live tests against the real gateway
(`https://code.neevcloud.com/zen/go/v1/chat/completions`, authenticated):

| Request | Result |
|---|---|
| bare request, no fidelity headers at all | **200** |
| full neev fidelity (UA + all `x-opencode-*`) | **200** |
| console UA (`neev/latest/0.0.2/cli`) on the gateway — mismatched identity | **200** |
| malformed `x-opencode-session: NOT_A_VALID_ID` | **200** |

Console API, measured separately: curl default UA → 200, empty UA → 200. Only the known-bot signature
`Python-urllib/*` is 403'd (`error_code: 1010, browser_signature_banned`) by the Cloudflare edge. `reqwest` never
emits that, so reqwest is never at risk.

Two consequences, both load-bearing:

1. **Fidelity is insurance, not a fix.** It buys nothing today. It is bought against a hypothetical future gate.
2. **Fidelity can never be the cause of a bug.** Nothing validates these headers, so no call fails *because of*
   them. If a NeevCloud call misbehaves, look at auth, the URL join, the model id, or the body — never here.
   The one exception is §7's failure mode: a *stale pinned* value under a future minimum-version gate. That is
   the insurance itself failing, not fidelity causing an unrelated bug.

## 2. The two client identities

neev is not one HTTP client. It is two, with different identities, and copying one onto both endpoints is not
fidelity — it is a third identity that matches nothing. Captured by TLS-intercepting a real neev run
(mitmproxy, decrypted; see §7 for re-derivation).

### A. Console / catalog client — the effect `HttpClient`

Used for `/auth/*`, `/api/*`, `models.dev`.

| Header | Exact value | Produced in neev by | Where bum sets it |
|---|---|---|---|
| `User-Agent` | `neev/latest/0.0.2/cli` | UA template (below) | `.header(USER_AGENT, …)` per request in `auth/neev/http.rs` |
| `Accept` | `*/*`, or `application/json` when `acceptJson` is piped (e.g. `/api/config`) | effect `HttpClient` default | `.header(ACCEPT, …)` |
| `Accept-Encoding` | `gzip, deflate, br, zstd` | Bun's fetch | **do not set** — reqwest manages this; see note below |
| `Connection` | `keep-alive` | Bun's fetch | **do not set** — hop-by-hop, connection-pool managed |
| `Authorization` | `Bearer <token>` | per-call | already in `neev_get` (`04 §3`) |
| `x-org-id` | `<wrk_…>` | per-call, `/api/config` + `/api/mcp/list` only | already in `neev_get_org` (`04 §3`) |
| `b3`, `traceparent` | OTel/W3C trace context | neev's tracing | skip — see §5 |

**UA template decomposition.** From the neev binary:

```js
`neev/${y0}/${v0}/${_0.OPENCODE_CLIENT}`   ->  neev/latest/0.0.2/cli
        channel  version    client-type
```

- `channel = "latest"` — release channel, not a version
- `version = "0.0.2"` — the npm package version of `@neevcode/neev`
- `client = "cli"` — client type (same vocabulary as `x-opencode-client` on identity B)

**`Accept-Encoding` / `Connection` are deliberately not pinned.** reqwest sets `accept-encoding` from its enabled
decompression features and manages `connection` itself; forcing either byte-for-byte means either fighting the
client or advertising an encoding bum cannot decode. The encodings differ (`br`/`zstd` presence depends on
reqwest features) — this is a knowingly accepted fidelity gap on two headers no server identifies clients by.

### B. Inference gateway client — the Vercel AI SDK

Used for `/zen/go/v1/chat/completions`.

| Header | Exact value | Produced in neev by | Where bum sets it |
|---|---|---|---|
| `User-Agent` | `opencode/0.0.2 ai-sdk/provider-utils/4.0.23 runtime/bun/1.3.13` | AI SDK `provider-utils` | **not `extra_headers`** — see §4 |
| `Content-Type` | `application/json` | AI SDK | already set by `SamplingClient` (`client.rs:19` imports `CONTENT_TYPE`) |
| `Authorization` | `Bearer <apiKey>` | provider config | `SamplerConfig` auth (`AuthScheme::Bearer`, the default) |
| `x-opencode-client` | `cli` | provider `headers` | `extra_headers` — rides fine |
| `x-opencode-project` | `global` | provider `headers` | `extra_headers` — rides fine |
| `x-opencode-session` | `ses_0911d2d8dffecwBOwGkRcjLNkj` | per-session id | `extra_headers`, freshly generated (§6) |
| `x-opencode-request` | `msg_f6ee2d407001y4XYhvarbxAd2j` | per-message id | `extra_headers`, freshly generated (§6) |
| `Accept` | `*/*` | AI SDK | already set by `SamplingClient` |
| `Accept-Encoding` | `gzip, deflate, br, zstd` | Bun's fetch | **do not set** (as above) |
| `Connection` | `keep-alive` | Bun's fetch | **do not set** (as above) |

**Provenance — read this before treating identity B as gospel.** Identity B was captured against
`opencode.ai/zen/v1` (the logged-out free tier), because the intercepted container was not logged into NeevCloud.
It is the same AI-SDK code path that serves the NeevCloud gateway — provider key `opencode`, only `baseURL` and
`apiKey` differ — so the same headers apply to `code.neevcloud.com/zen/go/v1`. **That is a strong inference, not
a direct capture.** The header *values* above are real captured bytes. Re-derive against a logged-in NeevCloud run
(§7) if you ever want this promoted to direct capture.

The `ses_…` / `msg_…` values above are real captured ids shown for **shape only**. Never replay them (§6).

### C. Not neev's identity — do not copy

| Host | UA | Why it's here |
|---|---|---|
| `github.com` / release-assets | `opencode/0.0.2` | ripgrep download — bum doesn't do this |
| `registry.npmjs.org` | npm's own UA | not neev's client at all |

Irrelevant to the port. Listed so a future reader of the same capture doesn't mistake them for identity A or B.

## 3. Rust: one constants module, `auth/neev/fidelity.rs`

Every fidelity string lives here and nowhere else. One file, greppable, bumpable, env-overridable.
`crates/codegen/xai-grok-shell/src/auth/neev/fidelity.rs`:

```rust
//! Every wire-fidelity string for the NeevCloud port. ONE place, on purpose.
//!
//! These make bum's requests look like the neev CLI's (docs/neevcloud/07-wire-fidelity.md).
//! Nothing on the wire requires them today — they are forward-compat insurance
//! against NeevCloud gating on client identity later.
//!
//! THEY GO STALE. Re-derive with ~/bum/neev/artifacts/scripts/mitm.sh against the
//! current @neevcode/neev, then bump the consts below. Every one is also overridable
//! at runtime via env, so a mismatch is fixable without a rebuild.

use std::borrow::Cow;

use reqwest::header::{ACCEPT, HeaderMap, HeaderName, HeaderValue, USER_AGENT};

// --- Identity A: console / catalog client -----------------------------------
/// Release channel segment of the console UA. Captured: "latest".
pub const NEEV_CHANNEL: &str = "latest";
/// @neevcode/neev package version. Captured: "0.0.2". STALE-PRONE.
pub const NEEV_VERSION: &str = "0.0.2";
/// Client-type segment, shared with `x-opencode-client`. Captured: "cli".
pub const NEEV_CLIENT_TYPE: &str = "cli";

// --- Identity B: inference gateway (Vercel AI SDK) ---------------------------
/// Full gateway UA. Captured verbatim against opencode.ai/zen/v1 (same AI-SDK
/// path as code.neevcloud.com/zen/go/v1 — strong inference, not direct capture).
/// Embeds three independently-moving versions. VERY STALE-PRONE.
pub const NEEV_GATEWAY_UA: &str =
    "opencode/0.0.2 ai-sdk/provider-utils/4.0.23 runtime/bun/1.3.13";
/// `x-opencode-project`. Captured: "global".
pub const NEEV_PROJECT: &str = "global";

// --- Env overrides -----------------------------------------------------------
/// Overrides the whole console UA (not a segment).
pub const ENV_CONSOLE_UA: &str = "BUM_NEEV_CONSOLE_UA";
/// Overrides the whole gateway UA.
pub const ENV_GATEWAY_UA: &str = "BUM_NEEV_GATEWAY_UA";
/// Overrides `x-opencode-project`.
pub const ENV_PROJECT: &str = "BUM_NEEV_PROJECT";

fn env_or<'a>(key: &str, fallback: impl Into<Cow<'a, str>>) -> Cow<'a, str> {
    match std::env::var(key) {
        Ok(v) if !v.is_empty() => Cow::Owned(v),
        _ => fallback.into(),
    }
}

/// `neev/latest/0.0.2/cli` — identity A.
pub fn console_user_agent() -> Cow<'static, str> {
    env_or(
        ENV_CONSOLE_UA,
        Cow::Owned(format!(
            "neev/{NEEV_CHANNEL}/{NEEV_VERSION}/{NEEV_CLIENT_TYPE}"
        )),
    )
}

/// `opencode/0.0.2 ai-sdk/... runtime/bun/...` — identity B.
pub fn gateway_user_agent() -> Cow<'static, str> {
    env_or(ENV_GATEWAY_UA, NEEV_GATEWAY_UA)
}

/// Identity A headers. `accept_json` mirrors neev's `acceptJson` pipe
/// (`application/json` on /api/config, `*/*` elsewhere).
///
/// Accept-Encoding and Connection are deliberately NOT set: reqwest owns both,
/// and advertising an encoding we can't decode is worse than a fidelity gap on
/// two headers nobody identifies clients by. See 07 §2.
pub fn console_headers(accept_json: bool) -> HeaderMap {
    let mut h = HeaderMap::new();
    if let Ok(v) = HeaderValue::from_str(&console_user_agent()) {
        h.insert(USER_AGENT, v);
    }
    h.insert(
        ACCEPT,
        HeaderValue::from_static(if accept_json { "application/json" } else { "*/*" }),
    );
    h
}

/// Identity B headers, minus User-Agent — the sampler clobbers a UA passed
/// through `extra_headers` (07 §4, `xai-grok-sampler/src/client.rs:499-501`).
/// The `x-opencode-*` headers ride `extra_headers` fine.
///
/// `session_id` / `request_id` must be freshly generated (07 §6), never replayed.
pub fn gateway_headers(session_id: &str, request_id: &str) -> Vec<(HeaderName, String)> {
    vec![
        (
            HeaderName::from_static("x-opencode-client"),
            NEEV_CLIENT_TYPE.to_string(),
        ),
        (
            HeaderName::from_static("x-opencode-project"),
            env_or(ENV_PROJECT, NEEV_PROJECT).into_owned(),
        ),
        (
            HeaderName::from_static("x-opencode-session"),
            session_id.to_string(),
        ),
        (
            HeaderName::from_static("x-opencode-request"),
            request_id.to_string(),
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn console_ua_matches_captured_neev_string() {
        // The exact string captured from @neevcode/neev@0.0.2. If this fails,
        // someone bumped a const — re-derive, don't just fix the test.
        assert_eq!(console_user_agent(), "neev/latest/0.0.2/cli");
    }

    #[test]
    fn gateway_headers_carry_no_user_agent() {
        // A UA here would be silently dropped by the sampler (07 §4) — the
        // absence is load-bearing, not an oversight.
        let hs = gateway_headers("ses_x", "msg_x");
        assert!(!hs.iter().any(|(k, _)| k == USER_AGENT));
        assert_eq!(hs.len(), 4);
    }
}
```

`SamplerConfig.extra_headers` is an `IndexMap<String, String>` (`client.rs:2098` constructs it as
`IndexMap::new()`), so the wiring is:

```rust
for (name, value) in fidelity::gateway_headers(&session_id, &request_id) {
    cfg.extra_headers.insert(name.to_string(), value);
}
```

`console_headers` folds into the `04 §3` builders:

```rust
pub(super) fn neev_get(url: &str, bearer: &str) -> RequestBuilder {
    crate::http::shared_client()
        .get(url)
        .headers(super::fidelity::console_headers(false))
        .bearer_auth(bearer)
        .timeout(NEEV_TIMEOUT)
}
```

The per-request `.headers(…)` overrides the client-level `.user_agent(process_user_agent_string())` set at
`crates/codegen/xai-grok-http/src/lib.rs:289` for these calls only. That is the whole reason it must be
per-request: `set_client_name` (`lib.rs:164`) is process-global and
`.expect("set_client_name called more than once")`s on a second call — using it would rewrite xAI's and Codex's
UA too, and panic if anything already called it.

## 4. The sampler clobber problem

`SamplingClient::new` builds its default header map in this order
(`crates/codegen/xai-grok-sampler/src/client.rs`):

1. `:443-449` — apply `config.extra_headers` verbatim, `headers.insert(header_name, header_value)`.
   The comment at `:440-442` calls it "the single injection point for … any other URL- or
   environment-specific headers the session decides to set."
2. `:489-501` — **unconditionally**:

```rust
// Always set User-Agent: per-session origin if available, else fallback.
{
    let ua_string = match config.origin_client.as_ref() {
        Some(origin) => user_agent_string_for(origin),
        None => user_agent_string_for(&OriginClientInfo {
            product: AGENT_PRODUCT.to_string(),
            version: Some(agent_version()),
        }),
    };
    if let Ok(v) = HeaderValue::from_str(&ua_string) {
        headers.insert(USER_AGENT, v);   // client.rs:500 — insert REPLACES
    }
}
```

`HeaderMap::insert` replaces. **A `User-Agent` set via `extra_headers` is silently clobbered on the gateway
path.** No error, no warning — the header just isn't the one you set. Only the UA is affected; the four
`x-opencode-*` headers are not touched by anything downstream and ride `extra_headers` intact.

Two ways out.

### Option 1 — `origin_client` (supported seam, cannot be byte-exact)

`SamplerConfig.origin_client: Option<OriginClientInfo>` (`crates/codegen/xai-grok-sampler/src/config.rs:201-205`,
`{ product: String, version: Option<String> }`) feeds `user_agent_string_for` (`client.rs:361`), which
**composes** — it does not pass through:

- `client.rs:365-370` — if `product == AGENT_PRODUCT` (`"grok-shell"`, `:40`) and version matches:
  `grok-shell/<ver> (linux; x86_64)`
- `client.rs:376-386` — otherwise: `{origin.product}/{origin.version} grok-shell/{agent_version} ({os}; {arch})`

So the closest reachable string is:

```
opencode/0.0.2 grok-shell/<ver> (linux; x86_64)
```

vs. the target `opencode/0.0.2 ai-sdk/provider-utils/4.0.23 runtime/bun/1.3.13`. Not byte-exact, and it leaks
`grok-shell` — which is arguably the *more honest* wire but is precisely what the decision in §1 rejects. There
is no combination of `product`/`version` that suppresses the `grok-shell/… (os; arch)` suffix: it is a
`format!` literal, not a template. **`origin_client` cannot achieve identity B.**

### Option 2 — conditional insert at `client.rs:500` (~2 lines, byte-exact)

Let `extra_headers` win when it explicitly set a UA:

```rust
        if let Ok(v) = HeaderValue::from_str(&ua_string)
            && !headers.contains_key(USER_AGENT)
        {
            headers.insert(USER_AGENT, v);
        }
```

Behavior change is scoped exactly to callers that put a `user-agent` in `extra_headers`. Today that is nobody —
`extra_headers` is empty by default (`client.rs:2098`) and the existing test `new_applies_extra_headers`
(`client.rs:2262-2266`) sets other headers. `default_headers.contains_key(USER_AGENT)` (asserted at
`client.rs:2311`) stays true either way, since the fidelity path supplies one. Fixing the comment at `:489`
("Always set" → "Set unless extra_headers already did") is part of the diff.

### Recommendation

**Option 2.** It is the only route to the decision in §1, it is two lines plus a comment, and it converts a silent
footgun into documented precedence — a caller that explicitly sets a `User-Agent` through the header seam should
get the one it set. Option 1's composite UA fails the goal *and* announces `grok-shell` to NeevCloud, which is the
worst of both: not honest-by-design, not fidelity. Add one test asserting `extra_headers["user-agent"]` survives.

Fork note: this is a change to vendored xAI code and will conflict on upstream merges around `client.rs:489-501`.
It is two lines and the conflict is trivially resolvable — worth flagging in the merge checklist, not worth
avoiding.

## 5. Trace headers — `b3` and `traceparent`

Identity A also sends:

```
b3: <trace>-<span>-1-<parent>          W3C-predecessor B3 (Zipkin) trace context
traceparent: 00-<trace>-<span>-01      W3C Trace Context
```

Both are **distributed-trace context**, not client identity: random per-request ids describing neev's own span
tree. A server cannot use them to distinguish neev from an impostor — any client can emit well-formed ones, and
their values carry no client-identifying bytes. Fidelity value: ~zero. Their *presence* is weakly
identifying (an fingerprinting server could note "no `b3`"), which is the only argument for emitting them.

**What bum already has (checked, not guessed):**

- `xai_file_utils::trace_context::inject_trace_context_into_request(builder)`
  (`crates/codegen/xai-file-utils/src/trace_context.rs:22-34`, `pub`, module exported at
  `xai-file-utils/src/lib.rs:25`) injects W3C `traceparent`/`tracestate` into a `reqwest::RequestBuilder` from the
  current span. `xai-grok-shell` already depends on `xai-file-utils` (`Cargo.toml:149`). One call site would do it.
- The global propagator is W3C-only: `global::set_text_map_propagator(TraceContextPropagator::new())` at
  `crates/codegen/xai-grok-telemetry/src/otel_layer/mod.rs:87`. It is installed only when `build_otel_layer` runs,
  so outside a traced session `inject_trace_context` emits **nothing** — the header silently disappears.
- **`b3` has no producer.** No B3 propagator crate is in `Cargo.lock` (`opentelemetry-propagator-b3` absent).
  Emitting `b3` means a new workspace dependency, or hand-rolling the format.

**Recommendation: skip both.** Zero fidelity value, and each has a real cost: `traceparent` appears only when
telemetry happens to be on (so bum's traffic would be *inconsistent* — worse than consistently absent), and `b3`
costs a dependency for a header no one reads. If a future NeevCloud gate ever inspects trace context — it won't;
gates check identity, not traces — revisit then. Recorded here so the omission reads as a decision, not an
oversight.

## 6. Session and request id shapes

```
x-opencode-session: ses_0911d2d8dffecwBOwGkRcjLNkj    "ses_" + 26 chars
x-opencode-request: msg_f6ee2d407001y4XYhvarbxAd2j    "msg_" + 26 chars
```

26 chars, mixed-case alphanumeric — ULID-ish (a 26-char Crockford-base32 ULID is uppercase-only; these are mixed
case, so they are a ULID-*like* scheme, not canonical ULID). Exact alphabet **NOT VERIFIED** — only these two
samples were captured, and the gateway accepts `NOT_A_VALID_ID` anyway (§1), so the shape is cosmetic.

**Generate fresh, never replay.** One `ses_` per bum session, one `msg_` per request. Replaying captured ids means
every bum install worldwide reports the same session to NeevCloud telemetry — which is both useless as fidelity
(one "session" making millions of calls is *more* anomalous than a fresh id) and a correlation surface.

**Existing generators in the workspace (grepped):**

- **ULID: NOT FOUND.** No `ulid` crate in `Cargo.lock`, none in `crates/*/Cargo.toml`.
- `uuid` is a workspace dep (`Cargo.toml:263`, `v4`/`v5`; `xai-grok-sampler/Cargo.toml:26` already enables `v4`).
  Wrong shape — a UUID is 36 chars with dashes, or 32 hex undashed. Neither is 26.
- `xai_grok_telemetry::id::agent_instance_id` (`crates/codegen/xai-grok-telemetry/src/id.rs:25-30`) is a
  per-process `Uuid::new_v4`. Right *lifetime* for a session id, wrong shape, and it's the telemetry agent
  identity — do not overload it.
- `rand` is a workspace dep (`Cargo.toml:199`, `rand = "0.9"`).

Nothing to reuse. Twenty lines in `fidelity.rs`:

```rust
/// ULID-ish id: `{prefix}_` + 26 chars. Shape-compatible with neev's
/// ses_/msg_ ids (07 §6). Timestamp prefix + random suffix, like a ULID, so
/// ids sort roughly by creation — the gateway validates none of it (07 §1).
///
// ponytail: hand-rolled, not the `ulid` crate — 20 lines vs a new workspace
// dep for a cosmetic header. Swap to `ulid` if anything else ever needs one.
fn ulid_ish(prefix: &str) -> String {
    use rand::Rng;
    const ALPHABET: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

    let millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    // 12 hex chars of millis (good past year 10889), then 14 random.
    let mut s = format!("{prefix}_{:012x}", millis & 0xffff_ffff_ffff);
    let mut rng = rand::rng();
    for _ in 0..14 {
        s.push(ALPHABET[rng.random_range(0..ALPHABET.len())] as char);
    }
    s
}

/// One per bum session. Cache it in the session, do not call per request.
pub fn new_session_id() -> String {
    ulid_ish("ses")
}

/// One per outbound gateway request.
pub fn new_request_id() -> String {
    ulid_ish("msg")
}

#[cfg(test)]
mod id_tests {
    use super::*;

    #[test]
    fn ids_match_captured_shape_and_are_unique() {
        let a = new_session_id();
        assert!(a.starts_with("ses_"));
        assert_eq!(a.len(), 4 + 26, "captured shape is prefix + 26 chars: {a}");
        assert!(a[4..].chars().all(|c| c.is_ascii_alphanumeric()));
        assert_ne!(a, new_session_id(), "ids must never repeat");
        assert!(new_request_id().starts_with("msg_"));
    }
}
```

`rand = "0.9"` must be added to `xai-grok-shell/Cargo.toml` if not already present — check before assuming.

## 7. Maintenance: the insurance can become the outage

Every string in §3 is a snapshot of someone else's release:

| String | Moves when |
|---|---|
| `0.0.2` (both identities) | every `@neevcode/neev` release |
| `4.0.23` | every AI SDK `provider-utils` bump inside neev |
| `bun/1.3.13` | every Bun bump in neev's build |
| `latest` | never, probably — it's a channel |

They **will** go stale. And here is the failure mode that matters:

> If NeevCloud ever enforces a **minimum client version**, the hard-pinned stale UA is the exact thing that
> breaks. A bare request (no UA claim at all) might sail through where `opencode/0.0.2` gets rejected as
> too old. **The insurance policy becomes the outage.**

That is the honest shape of this trade: fidelity protects against identity gating and exposes you to version
gating. It is still the right call (§1) — version gates are rarer than identity gates, and the mitigations below
make the failure a 10-second fix instead of a release.

**Mitigations, all already in §3:**

1. **One place.** Every string in `auth/neev/fidelity.rs`. `grep -rn '0\.0\.2' crates/codegen/xai-grok-shell/src/auth/neev/`
   must return only that file. If it ever returns two, fix it before shipping.
2. **Env-overridable.** `BUM_NEEV_CONSOLE_UA`, `BUM_NEEV_GATEWAY_UA`, `BUM_NEEV_PROJECT`. A user hitting a version
   gate sets one env var and keeps working; no rebuild, no release. This is the actual insurance on the insurance.
3. **Re-derivable.** Documented below.
4. **Best-effort, not a contract.** No test may assert bum's live traffic *matches production neev* — that test
   fails when the vendor ships, and a red CI over someone else's release is how a doc-only fidelity policy turns
   into a maintenance tax. Assert against the *captured constants* (§3's tests), which change only when a human
   re-derives.

**How to re-derive:**

```bash
# ~/bum/neev/artifacts/scripts/mitm.sh — installs mitmproxy + @neevcode/neev,
# routes Bun through the proxy (HTTPS_PROXY + NODE_EXTRA_CA_CERTS), dumps flows.
bash ~/bum/neev/artifacts/scripts/mitm.sh          # -> /out/flows.mitm
mitmdump -nr /out/flows.mitm -s dump_headers.py    # or: mitmproxy -r /out/flows.mitm
```

Pin the current version in the script (`npm install -g @neevcode/neev@<current>`), log in to NeevCloud inside the
container so identity B is captured against `code.neevcloud.com/zen/go/v1` directly — that also promotes §2's
strong inference to a direct capture — then read the request headers and bump the consts. Companion scripts:
`netcap.sh`, `netcap2.sh` in the same directory.

## 8. Verification: prove fidelity, don't assume it

Fidelity is unverifiable by response code — everything returns 200 (§1). The only proof is comparing bum's own
outbound bytes against neev's captured set.

**Capture bum through the same proxy.** reqwest honors `HTTPS_PROXY` and, unlike Bun, needs the CA in the system
store or `SSL_CERT_FILE`:

```bash
mitmdump -p 8080 -w /tmp/bum.mitm &
HTTPS_PROXY=http://127.0.0.1:8080 \
SSL_CERT_FILE=$HOME/.mitmproxy/mitmproxy-ca-cert.pem \
  bum --provider neev -p 'hi'
```

**Diff the header sets.** Same extraction for both flow files, then `diff`:

```bash
cat > /tmp/hdr.py <<'PY'
# mitmdump -nr <flows> -s /tmp/hdr.py  -> "HOST PATH\n  header: value" per request
def request(flow):
    print(f"{flow.request.host} {flow.request.path}")
    for k, v in sorted(flow.request.headers.items()):
        if k.lower() == "authorization":
            v = "Bearer sk-…"          # never diff a real secret
        print(f"  {k.lower()}: {v}")
PY

mitmdump -nr ~/bum/neev/artifacts/flows.mitm -s /tmp/hdr.py > /tmp/neev.hdr
mitmdump -nr /tmp/bum.mitm              -s /tmp/hdr.py > /tmp/bum.hdr
diff /tmp/neev.hdr /tmp/bum.hdr
```

**Expected diff — these lines SHOULD differ, and the diff is only clean if they are the only ones:**

| Difference | Why it's expected |
|---|---|
| `x-opencode-session` / `x-opencode-request` values | freshly generated per run (§6) — compare the *shape*, never the value |
| `b3`, `traceparent` present in neev, absent in bum | deliberate (§5) |
| `accept-encoding` value | reqwest's feature set vs Bun's (§2) |
| `content-length` | different bodies |

**Anything else — especially a `user-agent` mismatch on `/zen/go/v1` — is the §4 clobber, or a fidelity bug.**
A `user-agent` reading `… grok-shell/<ver> (linux; x86_64)` on the gateway means Option 2 was not applied and
the sampler overwrote the fidelity UA at `client.rs:500`.

The one automated check worth having is `fidelity.rs`'s own unit tests (§3, §6) — they assert the constants match
the captured strings and that `gateway_headers` carries no UA. The capture-diff above is a **manual pre-release
ritual**, not CI: it needs a proxy, a live login, and a human reading four expected differences.

## 9. Operational consideration

bum presents as NeevCloud's first-party CLI and uses their OAuth `client_id` (`neev-cli` — unavoidable; there is
no self-registration path), so NeevCloud's telemetry will attribute bum's usage to neev; note it against vendor
ToS and support attribution.
