# Phase 12: Codex Depth & Attribution Polish - Research

**Researched:** 2026-07-21
**Domain:** Product attribution, Codex Responses transport disclosure, tool compatibility, and embedded documentation verification
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

### Codex Parity Scope
- Defer Codex Responses WebSocket incremental transport. The existing HTTP/SSE path is daily-driver green, while a second transport would add substantial state and reliability risk.
- Validate the existing `codex:apply_patch` implementation and close only documentation or focused test gaps; do not replace it or add a competing patch tool.
- Keep the ChatGPT OAuth and trusted-route `originator` identity as `bum`; never mimic stock Codex CLI identity.
- Document current tool mappings and compatibility gaps without broad tool renames that could break persisted sessions, configuration, or integrations.

### Product Attribution and Chrome
- Describe bum as using ChatGPT/Codex OAuth and compatible model APIs while stating clearly that bum is not the stock Codex CLI.
- Put attribution in the README and user-guide capability documentation, not in repetitive TUI banners.
- Preserve model brands and provider labels such as `GPT-5.6 Sol (Codex)` while keeping all product chrome identified as `bum`.
- Enforce that no user-facing surface claims the executable or product is Grok Build or Codex CLI. Legitimate model/provider names and historical or legal references remain allowed.

### Capability Disclosure and Notices
- Maintain a concise capability matrix covering transport, tools, auth, and known provider differences.
- Label WebSocket incremental transport and broad Codex tool-name parity as deferred, non-blocking enhancements.
- Update third-party notices only if substantial copied or derived code or constants land; retain the existing Codex `apply_patch` attribution.
- Close the phase with automated identity and gap-document checks plus focused tests. Do not repeat the Phase 9 live dual-login gate unless implementation changes runtime or wire behavior.

### the agent's Discretion
- Exact documentation file placement and matrix formatting, provided it is discoverable from the primary README or shipped user guide.
- Exact focused regression-test placement and wording of the non-stock-Codex disclaimer.
- Whether small code changes are needed to enforce identity invariants discovered by the final sweep.

### Deferred Ideas (OUT OF SCOPE)
- Codex Responses WebSocket incremental transport.
- Broad Codex tool-name parity or rename campaign.
- Any additional live dual-login gate unless this phase changes runtime or wire behavior.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| ID-02 (deepening) | Product UI chrome, help text, and user-facing strings present as bum, not stock Grok Build. | The embedded guide is a runtime surface and still contains stock product/executable claims; the plan needs a scoped identity sweep plus allowlisted automated checks. [VERIFIED: codebase grep] |
| OPS-04 (disclosure) | GPT-5.6 remains a productive coding path with tools, edits, and shell as supported. | Existing HTTP/SSE and `codex:apply_patch` focused suites are green; documentation must distinguish daily-driver compatibility from stock-CLI feature parity. [VERIFIED: cargo test and codebase grep] |
| D-10 (Phase 9 gap policy) | Provider capability gaps remain explicit rather than silently waived. | A maintained matrix should cover auth, transport, continuity, tool naming/wire shape, and known non-blocking differences. [VERIFIED: `.planning/phases/09-daily-driver-end-to-end-validation/09-VALIDATION.md`] |
</phase_requirements>

## Project Constraints (from AGENTS.md)

- Stay in the Rust 2024 workspace on pinned Rust 1.92.0, Tokio, and the existing TUI/agent crates; this is fork evolution, not a rewrite. [VERIFIED: `AGENTS.md`, `rust-toolchain.toml`]
- Preserve ChatGPT OAuth as the primary Codex identity path, xAI OAuth for Grok, per-model routing, and isolated `~/.bum` / `BUM_HOME` storage. [VERIFIED: `AGENTS.md`]
- Do not re-enable xAI auto-update or product telemetry phone-home. [VERIFIED: `AGENTS.md`]
- Keep product and CLI naming as `bum`; legitimate provider/model brands and internal `xai-grok-*` crate names are separate concerns. [VERIFIED: `AGENTS.md`, Phase 12 CONTEXT]
- Treat the generated root `Cargo.toml` as read-only; edit per-crate manifests only if a dependency change is ever required. This phase requires no dependency changes. [VERIFIED: `AGENTS.md`, codebase audit]
- Keep TUI dispatch pure; filesystem/network work belongs in effects or shell. No TUI runtime change is recommended for this phase. [VERIFIED: `AGENTS.md`]
- Use `thiserror` for domain errors, `anyhow` at application boundaries, structured `tracing`, no secret logging, `dunce::canonicalize` instead of raw canonicalize, and `// SAFETY:` for every unsafe block. These conventions matter only if the final sweep finds a small code invariant to enforce. [VERIFIED: `AGENTS.md`]
- Prefer per-crate `cargo test` / `cargo clippy`; the full workspace is intentionally heavy. Run `cargo fmt --all -- --check` for this docs/test phase to avoid unrelated formatter spill. [VERIFIED: `AGENTS.md`, `.planning/STATE.md`]
- Preserve project-local `.grok/` compatibility layouts where the runtime intentionally reads them; user-global product state remains under `BUM_HOME` / `~/.bum`. [VERIFIED: Phase 01 summaries and codebase grep]
- Tool ports must retain crate-local and root third-party notices. [VERIFIED: `AGENTS.md`, notice files]

## Summary

Phase 12 should be planned as a documentation-contract and regression-gate phase. bum already has the runtime behavior that the locked scope wants to preserve: ChatGPT OAuth identifies the product with `originator=bum`; trusted Responses requests add `originator: bum` and stable session/thread/request UUID headers only on the trusted Codex OAuth route; the sampler sends Responses through HTTP `POST` with `Accept: text/event-stream`; and the current Codex patch implementation has a broad, green focused suite. [VERIFIED: codebase grep; focused cargo tests]

The missing work is product truthfulness and durable disclosure. The root README says bum is a Grok Build fork but does not yet say explicitly that it is not the stock OpenAI Codex CLI, and it has no provider capability matrix. More importantly, the in-app user guide is compiled with `include_str!`, exposed through `/docs`, and extracted under the product home at startup, yet its guide index and 21 of the 22 embedded numbered documents still contain stock Grok Build product claims, stock `grok` command examples, or stock-home wording. [VERIFIED: `README.md`, `xai-grok-pager/src/docs.rs`, codebase grep] This is not a cosmetic repository-only artifact; it is shipped runtime content. [VERIFIED: `extract_user_guide_docs` and `USER_GUIDE`]

**Primary recommendation:** add one canonical capability matrix to the already-embedded Authentication guide, link it from the root README and guide index, perform a scoped identity correction across every embedded guide/reference document, and enforce the result with phase-prefixed Rust tests plus a non-vacuous static phase gate. Preserve provider/model/legal lineage terms through an explicit allowlist; do not touch transport, wire behavior, or persisted tool identifiers. [VERIFIED: codebase audit; locked CONTEXT]

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Non-stock-Codex disclaimer and capability matrix | Browser / Client documentation | Frontend/TUI embedded docs | The README is the repository entry point; `xai-grok-pager::docs` embeds and serves the shipped guide. [VERIFIED: codebase grep] |
| Product identity invariant | Frontend/TUI and CLI | API / Backend trusted headers | User-facing words belong to bum, while route-scoped request attribution must remain `originator: bum`. [VERIFIED: codebase grep] |
| Codex Responses transport | API / Backend sampler | Shell route reconstruction | The sampler owns HTTP/SSE; the shell decides whether the route is trusted and may attach Codex metadata. [VERIFIED: `xai-grok-sampler/src/client.rs`, `sampler_turn.rs`] |
| Tool compatibility disclosure | Agent/tool registry | API / Responses serialization | Tool presets and identifiers are owned by `xai-grok-agent`/`xai-grok-tools`; Responses converts exposed tool definitions to function tools. [VERIFIED: codebase grep] |
| Third-party attribution | Repository/legal docs | Tool implementation modules | Both root and crate-local notices already name the OpenAI-derived modules. [VERIFIED: notice files] |
| Regression gate | Rust per-crate tests | Static shell checks in phase validation | Rust tests can inspect embedded content and tool presets; static checks validate README/notices and reject vacuous documentation. [VERIFIED: existing Phase 8 gate pattern] |

## Standard Stack

### Core

| Library / Facility | Version | Purpose | Why Standard |
|--------------------|---------|---------|--------------|
| Rust / Cargo | 1.92.0 | Compile-time embedded-doc tests and focused tool/identity tests | Pinned project toolchain; no new runtime or test framework is needed. [VERIFIED: `cargo --version`, `rustc --version`, `rust-toolchain.toml`] |
| Built-in `cargo test` | Rust 1.92.0 | Phase-prefixed unit and integration regressions | Existing project convention and test infrastructure. [VERIFIED: `AGENTS.md`, codebase tests] |
| `include_str!` + `USER_GUIDE` | in-tree | Make capability disclosure available in the in-app `/docs` surface and extracted product-home docs | This is the existing single source of truth for shipped guide content. [VERIFIED: `xai-grok-pager/src/docs.rs`] |
| ripgrep (`rg`) | 15.2.0 | Non-vacuous residual/allowlist checks across docs and notices | Existing phase gates use `rg`; it is installed in the environment. [VERIFIED: environment probe, Phase 8 gate] |

### Supporting

| Library / Facility | Version | Purpose | When to Use |
|--------------------|---------|---------|-------------|
| Existing `xai-grok-tools` tests | workspace 0.1.220-alpha.4 | Validate parser, add/delete/update/move/multi-file patch behavior and error paths | Re-run unchanged as the apply-patch baseline; add only a narrow preset/schema contract if needed. [VERIFIED: cargo output] |
| Existing shell trusted-route tests | workspace 0.1.220-alpha.4 | Prove `originator=bum`, stable UUID metadata, and non-leakage | Reuse; do not duplicate the full Phase 10 matrix. [VERIFIED: focused cargo test and codebase grep] |
| Markdown | in-tree | README and user-guide disclosure | Keep tables concise and human-readable; no documentation generator is required. [VERIFIED: repository layout] |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Existing HTTP/SSE | Responses WebSocket incremental transport | Rejected by locked scope. Current upstream requires capability gating, beta headers, prewarm, continuity state, reconnect/retry, and sticky HTTP fallback, so it is not a documentation-sized change. [VERIFIED: official openai/codex source at local commit `2895d82`; Phase 12 CONTEXT] |
| Current bum/Codex tool identifiers | Rename tools to stock Codex names | Rejected by locked scope because identifiers may be persisted or integrated, and name parity would still not guarantee payload-shape parity. [VERIFIED: Phase 12 CONTEXT; registry code] |
| Existing `codex:apply_patch` | New freeform/competing patch tool | Rejected by locked scope; the current tool is registered and behavior-tested. [VERIFIED: registry and cargo test] |
| Embedded guide matrix | Repetitive TUI banners | Rejected by locked scope; discoverable README/guide disclosure avoids noisy product chrome. [VERIFIED: Phase 12 CONTEXT] |

**Installation:** none. No external package or crate should be added. [VERIFIED: scope and codebase audit]

**Version verification:** `cargo 1.92.0`, `rustc 1.92.0`, and `rg 15.2.0` were available on 2026-07-21. [VERIFIED: environment probe]

## Architecture Patterns

### System Architecture Diagram

```text
Repository visitor ──> README disclaimer + capability link
                                  │
                                  v
                    Embedded user-guide capability matrix
                                  │
                 include_str! ────┼────> /docs picker
                                  └────> $BUM_HOME/docs/user-guide on startup

Model selected ──> shell provider/trust resolution ──> sampler HTTP POST /responses
                                                        │
                                                        v
                                                  SSE event decoder
                                                        │
                                                        v
                                                 bum tool dispatcher

Decision branch:
  trusted ChatGPT OAuth route ──> originator/session/thread headers owned by bum
  BYOK/custom/xAI route        ──> reserved Codex identity headers stripped

Deferred branch:
  Responses WebSocket ──X (document only; no implementation in Phase 12)
```

The diagram reflects current ownership and the locked deferred branch. [VERIFIED: codebase grep; Phase 12 CONTEXT]

### Recommended Project Structure

```text
README.md                                                # primary disclaimer + matrix link
crates/codegen/xai-grok-pager/
├── docs/user-guide/README.md                            # guide discovery/index wording
├── docs/user-guide/02-authentication.md                 # canonical provider capability matrix
├── docs/user-guide/{01..22}-*.md                        # embedded identity/executable sweep
├── docs/hooks-and-plugins.md                            # embedded reference identity sweep
├── docs/custom-hooks.md                                 # embedded product-home correction
└── src/docs.rs                                          # phase-prefixed embedded-doc checks
crates/codegen/xai-grok-agent/src/config.rs              # optional narrow toolset contract test
crates/codegen/xai-grok-tools/THIRD_PARTY_NOTICES.md     # verify, normally no change
THIRD-PARTY-NOTICES                                      # verify, normally no change
.planning/phases/12-codex-depth-attribution-polish/
├── 12-VALIDATION.md                                     # requirement-to-check map
└── 12-PHASE-GATE.md                                     # reproducible final static/test gate
```

The Authentication guide is recommended as the single shipped matrix location because it is already embedded and covers provider credentials; the root README and guide index can link to its anchor without adding a new document-registration seam. [VERIFIED: `USER_GUIDE`; recommendation based on locked discretion]

### Pattern 1: One Canonical Capability Matrix

**What:** Keep one table with columns for xAI/Grok, ChatGPT/Codex in bum, and stock Codex CLI comparison notes. Link to it rather than copying it. [VERIFIED: locked CONTEXT; recommendation]

**When to use:** Every claim about transport, auth, tools, or provider-specific gaps should resolve to this table. [VERIFIED: D-10 policy]

**Required rows:**

| Area | bum truth to document |
|------|-----------------------|
| Product | bum is an independent Grok Build fork; it is not the stock OpenAI Codex CLI. [VERIFIED: README lineage] [CITED: https://help.openai.com/en/articles/11096431] |
| Auth | xAI OAuth and ChatGPT/Codex OAuth use separate slots under the bum product home. [VERIFIED: requirements and auth code] |
| Transport | ChatGPT Codex uses Responses over HTTP POST with SSE streaming; Responses WebSocket incremental transport is deferred and non-blocking. [VERIFIED: sampler code; locked CONTEXT] |
| Continuity | Current HTTP path resends input with `store: false` and removes `previous_response_id` on trusted Codex; do not imply WS incremental prefix reuse. [VERIFIED: sampler tests] |
| Identity headers | Trusted Codex OAuth sends bum-owned originator/session metadata; untrusted/BYOK/custom/xAI routes do not inherit it. [VERIFIED: shell tests] |
| Standard bum tools | A normal bum session keeps bum's harness/toolset across compatible model switches; selecting a provider does not promise stock CLI tool names. [VERIFIED: harness compatibility code] |
| Codex preset | The optional Codex agent preset includes bum's bash plus Codex-derived read, apply_patch, list, and grep ports. [VERIFIED: `codex_toolset()`] |
| Patch shape | bum's `codex:apply_patch` is a function tool with JSON `{ patch }`; stock Codex may expose freeform apply_patch or intercept shell invocation. Compatibility is behavioral, not exact wire-name parity. [VERIFIED: bum tool implementation; official openai/codex source] |
| Live gate | Phase 10 already proved the daily-driver HTTP path; no new live dual-login gate is required unless Phase 12 changes runtime or wire behavior. [VERIFIED: Phase 10 verification; locked CONTEXT] |

### Pattern 2: Allowlisted Identity Sweep

**What:** Reject product/executable claims, not every provider or lineage word. [VERIFIED: locked CONTEXT]

**When to use:** README, every `USER_GUIDE` entry, both embedded reference docs, CLI help/chrome, and phase-gate grep. [VERIFIED: embedded-doc inventory]

Use semantic forbidden patterns such as stock install commands, `grok --version`, `grok update`, headings/descriptions that say Grok Build is the running product, and language that says bum is Codex CLI. Keep explicit allowlist categories for `Grok Build (xAI)` model labels, `grok-build` model IDs, historical fork lineage, provider names, legal notice text, internal crate/type names, `grok.com` service hosts, and project-local `.grok/` compatibility directories. [VERIFIED: locked CONTEXT; Phase 1/8 decisions]

### Pattern 3: Non-vacuous Documentation Contract Test

**What:** Tests must prove both required positive content and forbidden negative content. [VERIFIED: Phase 8 gate pattern]

**Example:**

```rust
// Source: existing xai-grok-pager/src/docs.rs test style
#[test]
fn p12_capability_disclosure_is_embedded_and_complete() {
    let auth = get_howto_doc("Authentication").expect("embedded auth guide");
    for required in [
        "not the stock OpenAI Codex CLI",
        "HTTP/SSE",
        "WebSocket",
        "deferred",
        "originator",
        "apply_patch",
    ] {
        assert!(auth.contains(required), "missing capability marker: {required}");
    }
}
```

The exact wording may differ, but the test should assert stable semantic markers rather than an entire prose paragraph. [VERIFIED: recommendation]

### Anti-Patterns to Avoid

- **Generic `s/Grok Build/bum/g`:** This would corrupt the legitimate xAI model brand, historical lineage, test fixtures, and legal attribution. Use an explicit surface inventory and allowlist. [VERIFIED: codebase grep; locked CONTEXT]
- **Provider label equals product identity:** `GPT-5.6 Sol (Codex)` is a provider/model label, not permission to call the executable Codex CLI. [VERIFIED: model catalog; locked CONTEXT]
- **Tool name equals tool parity:** Matching `apply_patch` text does not imply the same Responses wire kind or input schema. [VERIFIED: bum and official source comparison]
- **README-only fix:** The guide is embedded and extracted at runtime; leaving it stale preserves user-facing stock identity claims. [VERIFIED: `docs.rs`]
- **Broad runtime rename:** Do not rename tool namespaces, crate names, project-local compatibility directories, or env families as part of an attribution doc phase. [VERIFIED: locked CONTEXT; AGENTS.md]
- **Notice churn without derived code:** Documentation and tests alone do not require a new third-party notice entry. [VERIFIED: locked CONTEXT]

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Responses WebSocket parity | A partial socket client around the existing SSE decoder | Defer; retain current HTTP/SSE path | Official behavior includes capability gating, beta headers, prewarm, incremental continuity, retry, and sticky fallback. [VERIFIED: official openai/codex source] |
| Patch application | A second parser/executor or freeform adapter | Existing `codex::ApplyPatchTool` and its focused suite | Parser, fuzzy matching, filesystem operations, notifications, and output variants already exist. [VERIFIED: codebase and cargo test] |
| Product docs delivery | A new docs server or separate generated site | Existing `include_str!`, `/docs`, and startup extraction | These are already the shipped user-guide seams. [VERIFIED: `docs.rs`] |
| Identity scanning | A new dependency or AST framework | Small Rust content tests plus scoped `rg` gate | Markdown/product identity invariants are simple strings with an explicit allowlist. [VERIFIED: existing phase-gate pattern] |
| Legal notice generation | A new notice generator | Existing root and crate-local notice files, verified statically | Both already record OpenAI/Codex ancestry and license text. [VERIFIED: notice files] |

**Key insight:** the hard part is defining the boundary between product identity and legitimate provider/lineage references; a broad rename or a new transport would create more risk than the phase closes. [VERIFIED: inventory and locked decisions]

## Runtime State Inventory

| Category | Items Found | Action Required |
|----------|-------------|-----------------|
| Stored data | Registered guide files are extracted to `$BUM_HOME/docs/user-guide` and overwritten from embedded content on startup. [VERIFIED: `extract_user_guide_docs`] | Code/docs edit only; no record migration. Re-launching a rebuilt binary refreshes registered docs. [VERIFIED: extraction behavior] |
| Live service config | None. The capability/disclaimer and identity sweep do not change provider dashboards, OAuth app configuration, or remote service state. [VERIFIED: phase scope and codebase inventory] | None. Do not alter OAuth client identity or endpoints. [VERIFIED: locked CONTEXT] |
| OS-registered state | None. The executable remains `bum`; no service, task, plist, or unit rename is in scope. [VERIFIED: phase scope] | None. [VERIFIED: phase scope] |
| Secrets/env vars | No secret key or environment variable rename is required. `CODEX_ORIGINATOR` already equals `bum`; `BUM_HOME` remains the product root; internal compatibility env names are not a phase target. [VERIFIED: codebase and AGENTS.md] | None; retain existing names and secret-redaction behavior. [VERIFIED: locked scope] |
| Build artifacts | Markdown is embedded into the Rust binary by `include_str!`; an already-built binary and its previously extracted docs stay stale until rebuild/relaunch. [VERIFIED: `docs.rs`] | Rebuild the pager/binary and run extraction tests; no Cargo clean is required. [VERIFIED: Rust include behavior and existing tests] |

After repository files are updated, the only durable old wording expected outside source is extracted guide content from an older binary; the next launch of the rebuilt product overwrites registered guide files. [VERIFIED: extraction code]

## Common Pitfalls

### Pitfall 1: Treating the Whole Repository as Product Chrome

**What goes wrong:** broad greps flag model IDs, fixtures, legal notices, internal crates, and historical fork statements, leading to destructive renames. [VERIFIED: codebase inventory]

**Why it happens:** the repository retains upstream lineage and internal names by design. [VERIFIED: README, AGENTS.md]

**How to avoid:** enumerate shipped surfaces and maintain a documented allowlist. [VERIFIED: Phase 8 precedent]

**Warning signs:** a plan proposes changing `xai-grok-*`, `grok-build` model IDs, `ToolNamespace::Codex`, or notice copyright text. [VERIFIED: locked constraints]

### Pitfall 2: Missing the Embedded Guide

**What goes wrong:** README becomes honest while `/docs` and extracted user files still teach stock installation, `grok` commands, or Grok Build product identity. [VERIFIED: current guide grep]

**Why it happens:** markdown looks like repository documentation but is compiled and extracted by the pager. [VERIFIED: `docs.rs`]

**How to avoid:** test every `USER_GUIDE` and `REFERENCE_DOCS` content entry, and update the guide index alongside changed pages. [VERIFIED: recommendation]

**Warning signs:** only `README.md` appears in a plan's files list. [VERIFIED: inventory]

### Pitfall 3: Overclaiming Stock Codex Tool Parity

**What goes wrong:** users expect `exec_command`, `write_stdin`, or freeform `apply_patch` solely because the model/provider is Codex. [VERIFIED: official/bum source comparison]

**Why it happens:** model brand, agent preset, tool name, namespace, and wire payload are separate axes. [VERIFIED: tool registry and harness code]

**How to avoid:** document standard bum-session tools separately from the optional Codex preset and state the JSON `{ patch }` shape. [VERIFIED: codebase]

**Warning signs:** capability text says "same tools as Codex CLI" or "full Codex parity." [VERIFIED: locked CONTEXT]

### Pitfall 4: Turning Deferred WS into a Flag Stub

**What goes wrong:** a dormant config flag or partial WebSocket client creates an unsupported branch with no fallback/continuity proof. [VERIFIED: official upstream complexity]

**Why it happens:** the same event vocabulary makes WS look like a transport-only swap. [VERIFIED: official source]

**How to avoid:** document it as deferred and add a docs contract check; make no production transport changes. [VERIFIED: locked CONTEXT]

**Warning signs:** new `tokio-tungstenite` use in sampler, `OpenAI-Beta` headers, `previous_response_id` state, or provider `supports_websockets` flags. [VERIFIED: upstream/source comparison]

### Pitfall 5: Notice Updates Without a Trigger

**What goes wrong:** legal text drifts or duplicates merely because existing code was revalidated. [VERIFIED: existing dual notice structure]

**Why it happens:** attribution polish is confused with adding derived code. [VERIFIED: phase scope]

**How to avoid:** statically verify current OpenAI/Codex apply-patch entries; edit notices only if new derived code/constants land. [VERIFIED: locked CONTEXT]

**Warning signs:** notice files change in a docs-only plan without any new port. [VERIFIED: locked CONTEXT]

## Code Examples

Verified patterns from current project sources:

### Preserve bum-Owned Trusted Route Identity

```rust
// Source: crates/codegen/xai-grok-shell/src/session/acp_session_impl/sampler_turn.rs
if codex_session_oauth {
    extra_headers.insert("originator".to_string(), "bum".to_string());
    extra_headers.insert("session-id".to_string(), session_id.clone());
    extra_headers.insert("thread-id".to_string(), session_id.clone());
    extra_headers.insert("x-client-request-id".to_string(), session_id);
}
```

This is already covered by trusted-route and non-leakage tests; Phase 12 should preserve it, not refactor it. [VERIFIED: focused cargo test]

### HTTP/SSE Responses Choke Point

```rust
// Source: crates/codegen/xai-grok-sampler/src/client.rs
let http_request = grok_headers
    .apply(self.post(self.endpoint("responses"))?)
    .header(ACCEPT, HeaderValue::from_static("text/event-stream"))
    .json(&request_body);
```

This is the supported transport the matrix should name. [VERIFIED: codebase]

### Current bum Apply-Patch Input Shape

```rust
// Source: crates/codegen/xai-grok-tools/src/implementations/codex/apply_patch/tool.rs
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct ApplyPatchInput {
    pub patch: String,
}
```

The tool is registered as `Codex:apply_patch`, reports model-facing id `apply_patch`, and is a `ToolKind::Edit` function tool. [VERIFIED: tool and registry code]

### Focused Validation Commands

```bash
cargo test -p xai-grok-tools --lib apply_patch -- --nocapture
cargo test -p xai-grok-pager --lib docs::tests -- --nocapture
cargo test -p xai-grok-shell --lib trusted_codex_reconstruct_enables_profile_and_metadata -- --nocapture
cargo fmt --all -- --check
```

The first three commands passed during this research: 41 apply-patch-related tests, 15 docs-related tests, and one trusted Codex reconstruction test. [VERIFIED: cargo test output, 2026-07-21]

## State of the Art

| Old / Tempting Approach | Current / Required Approach | When Changed / Observed | Impact |
|-------------------------|-----------------------------|-------------------------|--------|
| Call the product Grok Build because the harness originated there | Product chrome is bum; Grok Build may appear as lineage or an xAI model brand | Phase 8 and Phase 12 locked decisions | Requires an allowlisted shipped-doc sweep, not a repository-wide rename. [VERIFIED: Phase 8/12 artifacts] |
| Imply Codex model access means stock Codex CLI | Say bum uses ChatGPT/Codex OAuth and compatible APIs but is not the stock CLI | Phase 12 locked decision | Avoids product impersonation while preserving provider labels. [CITED: https://help.openai.com/en/articles/11096431] |
| Responses HTTP plus `previous_response_id` continuity | Trusted bum HTTP uses `store: false`, full input, SSE, and removes `previous_response_id`; upstream incremental continuity is a WS concern | Phase 10 implementation / upstream 2026-07-18 source | Matrix must not conflate the transports. [VERIFIED: sampler tests and official source] |
| Treat WebSocket as a simple streaming alternative | Official Codex WS v2 includes beta header, prewarm, state reuse, retries, and sticky HTTP fallback | Official source commit `2895d82`, 2026-07-18 | Confirms deferral is technically sound. [VERIFIED: official local source] |
| Stock Codex patch as only a dedicated freeform tool | Current upstream can use freeform and shell interception; bum keeps a JSON function port | Official source / bum current tree | Document behavioral compatibility and shape differences. [VERIFIED: source comparison] |

**Deprecated/outdated:**

- README's note that product paths in docs may still say `~/.grok` is no longer an acceptable steady-state disclaimer for shipped docs; Phase 1 established `BUM_HOME` / `~/.bum` as product-home truth. [VERIFIED: README and Phase 1 verification]
- The apply-patch module comment saying all I/O is handled "in a later milestone" is stale because `tool.rs` already performs filesystem operations. Correcting that comment is a safe optional documentation cleanup, not a functional rewrite. [VERIFIED: module and tool code]

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| — | None. All implementation claims were verified against the current tree, focused tests, planning artifacts, or official OpenAI/Codex sources. | — | — |

## Open Questions

1. **How broad should the embedded-guide wording sweep be?**
   - What we know: all 22 registered numbered docs plus two reference docs are shipped through `include_str!`; 21 numbered docs contain at least one stock product/executable/home pattern. [VERIFIED: codebase grep]
   - What's unclear: some pages also document intentionally preserved project-local `.grok/` compatibility and internal `GROK_*` knobs, which must not be blindly renamed. [VERIFIED: Phase 1 decisions and codebase]
   - Recommendation: sweep every embedded page for product/executable claims, but classify each `.grok`/`GROK_*` occurrence as user-global product state, project-local compatibility, or internal compatibility before editing. [VERIFIED: recommendation]

2. **Should Phase 12 add a new toolset regression?**
   - What we know: 41 apply-patch-related tests pass and the Codex preset currently contains `Codex:apply_patch`; there is no phase-prefixed assertion that locks the preset/name boundary against broad parity renames. [VERIFIED: cargo test and codebase grep]
   - What's unclear: whether the planner considers static source verification enough for the "focused test gaps" decision. [VERIFIED: locked discretion]
   - Recommendation: add one pure `xai-grok-agent` config test that asserts the Codex preset retains `Codex:apply_patch` while the default bum preset retains its existing edit tool; do not add new filesystem behavior tests unless an actual defect is found. [VERIFIED: recommendation]

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|-------------|-----------|---------|----------|
| Rust compiler | Docs/tool/identity unit tests | ✓ | 1.92.0 | None required. [VERIFIED: environment probe] |
| Cargo | Focused test execution | ✓ | 1.92.0 | None required. [VERIFIED: environment probe] |
| ripgrep | Static identity/notice gates | ✓ | 15.2.0 | Rust content assertions for embedded docs. [VERIFIED: environment probe] |
| protoc | Existing workspace builds that trigger protobuf crates | ✓ | system binary and repo launcher | Repo `bin/protoc`. [VERIFIED: environment probe] |
| Official Codex reference tree | Source comparison | ✓ | commit `2895d82b5e449407712439ba4f89954f3fa0c7e3`, 2026-07-18 | Official GitHub source URLs. [VERIFIED: local git] |

**Missing dependencies with no fallback:** none. [VERIFIED: environment audit]

**Missing dependencies with fallback:** none. [VERIFIED: environment audit]

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in `cargo test` 1.92.0 plus scoped `rg` assertions. [VERIFIED: environment and project conventions] |
| Config file | `rust-toolchain.toml`; no separate global test config. [VERIFIED: repository] |
| Quick run command | `cargo test -p xai-grok-pager --lib p12_ -- --nocapture` [VERIFIED: recommended phase prefix pattern] |
| Full suite command | Discover and execute `p12_` filters in pager/agent plus existing apply-patch and trusted-originator filters, then run static README/guide/notice gates and `cargo fmt --all -- --check`. [VERIFIED: Phase 8 gate pattern; recommendation] |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| ID-02 | Embedded docs identify the executable/product as bum, with explicit allowlists for brands/lineage/legal text | unit + static | `cargo test -p xai-grok-pager --lib p12_embedded_docs_use_bum_product_identity -- --nocapture` | ❌ Wave 0 |
| D-10 | Shipped capability matrix states non-stock disclaimer, HTTP/SSE support, deferred WS, tool-shape gap, and bum originator | unit | `cargo test -p xai-grok-pager --lib p12_capability_disclosure_is_embedded_and_complete -- --nocapture` | ❌ Wave 0 |
| OPS-04 | Existing apply-patch behavior remains green | unit | `cargo test -p xai-grok-tools --lib apply_patch -- --nocapture` | ✅ 41 green |
| D-10 | Codex preset retains current apply-patch boundary without renaming the default toolset | unit | `cargo test -p xai-grok-agent --lib p12_codex_toolset_identity -- --nocapture` | ❌ Wave 0 |
| Product honesty | Trusted route identifies as bum and upstream identity does not leak | unit + integration | `cargo test -p xai-grok-shell --lib trusted_codex_reconstruct_enables_profile_and_metadata -- --nocapture` and `cargo test -p xai-grok-shell --test model_switch_gate trusted_codex_wire_headers_are_sent_and_stable -- --nocapture` | ✅ existing |
| Notices | Root and crate-local notices retain OpenAI/Codex apply-patch ancestry; no new notice required absent new derived code | static | scoped `rg` positive checks plus `git diff --` review | ✅ notice content; ❌ Phase 12 gate |
| Deferred scope | No sampler WebSocket implementation or stock identity header appears in phase diff | static | phase-diff `rg` / file allowlist in `12-PHASE-GATE.md` | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** run the changed crate's single `p12_` filter plus `cargo fmt --all -- --check`. [VERIFIED: project convention]
- **Per wave merge:** run all discovered `p12_` filters, docs tests, and the existing apply-patch/originator filters. [VERIFIED: recommendation]
- **Phase gate:** full focused suite green; README/embedded-doc positive and negative checks non-vacuous; notice attribution present; no live provider credentials or network required. [VERIFIED: locked CONTEXT]

### Wave 0 Gaps

- [ ] `xai-grok-pager/src/docs.rs::p12_embedded_docs_use_bum_product_identity` — scans every embedded guide/reference doc using a documented allowlist. [VERIFIED: identified gap]
- [ ] `xai-grok-pager/src/docs.rs::p12_capability_disclosure_is_embedded_and_complete` — asserts stable matrix/disclaimer markers. [VERIFIED: identified gap]
- [ ] `xai-grok-agent/src/config.rs::p12_codex_toolset_identity` — locks `Codex:apply_patch` and default bum edit-tool separation. [VERIFIED: identified gap]
- [ ] `12-VALIDATION.md` and `12-PHASE-GATE.md` — non-vacuous discovery, static README/notice checks, allowlist, and no-live-UAT rule. [VERIFIED: identified gap]
- [ ] No framework install is required. [VERIFIED: environment and existing tests]

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | yes, regression-only | Existing ChatGPT OAuth PKCE/device flow and independent provider credential slots; Phase 12 must not change them. [VERIFIED: auth code and requirements] |
| V3 Session Management | yes, regression-only | Trusted Codex session/thread/request UUIDs remain stable and route-scoped; existing tests cover this. [VERIFIED: shell code/tests] |
| V4 Access Control | yes | Reserved Codex identity headers are stripped before trusted reconstruction; BYOK/custom/xAI routes cannot inherit them. [VERIFIED: shell code/tests] |
| V5 Input Validation | yes | Existing apply-patch parser plus tool permission/sandbox layers remain authoritative; documentation must not claim stronger containment than tests prove. [VERIFIED: tool code and project architecture] |
| V6 Cryptography | yes, unchanged | Existing OAuth2/PKCE and project crypto libraries; never hand-roll auth crypto in this phase. [VERIFIED: auth code and AGENTS.md] |

### Known Threat Patterns for Rust CLI + Provider Routing

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Product impersonation through docs or headers | Spoofing | Explicit non-stock disclaimer, bum-owned originator, negative identity checks, and allowlisted provider brands. [VERIFIED: locked CONTEXT] |
| Reserved Codex identity metadata leaking to custom/BYOK/xAI routes | Spoofing / Information Disclosure | Strip reserved headers before trusted, exact-route reinsertion; preserve existing tests. [VERIFIED: shell code/tests] |
| Stale shipped docs directing users to stock install/update or credential homes | Tampering / Information Disclosure | Correct every embedded surface; use `BUM_HOME` for global state and retain only verified project-local compatibility paths. [VERIFIED: guide inventory and Phase 1] |
| Patch path or mutation claims exceeding enforced sandbox/permission behavior | Tampering | Keep current permission/sandbox gates, re-run focused patch tests, and document only observed behavior. Do not certify full stock Codex sandbox parity. [VERIFIED: codebase audit] |
| Credential exposure in docs/tests | Information Disclosure | Never place live tokens in fixtures or phase gates; no live dual-login gate in this phase. [VERIFIED: locked CONTEXT and project security rules] |

## Sources

### Primary (HIGH confidence)

- Current bum source tree: `xai-grok-sampler/src/client.rs`, `xai-grok-shell/src/session/acp_session_impl/sampler_turn.rs`, `xai-grok-tools/src/implementations/codex/apply_patch/`, `xai-grok-agent/src/config.rs`, and `xai-grok-pager/src/docs.rs` — transport, identity, tools, and embedded docs. [VERIFIED: codebase grep]
- Current planning artifacts: Phase 12 CONTEXT, Phase 10 verification, Phase 8 gate, Phase 1 isolation summaries — locked scope and established verification patterns. [VERIFIED: local files]
- Root and crate-local third-party notices — existing OpenAI/Codex attribution. [VERIFIED: local files]
- Official openai/codex local source tree at commit `2895d82b5e449407712439ba4f89954f3fa0c7e3` (2026-07-18), especially `core/src/client.rs`, `codex-api/src/endpoint/responses_websocket.rs`, `core/tests/suite/agent_websocket.rs`, `core/tests/suite/websocket_fallback.rs`, and tool handlers/spec tests. [VERIFIED: official local git source]

### Secondary (MEDIUM confidence)

- [OpenAI Help Center: OpenAI Codex CLI – Getting Started](https://help.openai.com/en/articles/11096431) — confirms that “OpenAI Codex CLI” names OpenAI's own CLI product. [CITED: https://help.openai.com/en/articles/11096431]
- [Official openai/codex apply-patch instructions](https://github.com/openai/codex/blob/main/codex-rs/core/prompt_with_apply_patch_instructions.md) — current upstream patch invocation and format. [CITED: https://github.com/openai/codex/blob/main/codex-rs/core/prompt_with_apply_patch_instructions.md]

### Tertiary (LOW confidence)

- None used for recommendations. [VERIFIED: research log]

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new dependencies; exact installed versions and existing project facilities were verified. [VERIFIED: environment/codebase]
- Architecture: HIGH — ownership follows current shell/sampler/tool/docs implementation. [VERIFIED: codebase]
- Pitfalls: HIGH — each is demonstrated by current residuals, locked scope, or official upstream complexity. [VERIFIED: codebase/planning/upstream]

**Research date:** 2026-07-21
**Valid until:** 2026-08-20 for bum internals; re-check official Codex WS/tool details against upstream before any future implementation because that surface is fast-moving. [VERIFIED: source recency; recommendation]
