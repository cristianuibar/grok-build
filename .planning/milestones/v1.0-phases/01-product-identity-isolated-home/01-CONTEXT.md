# Phase 1: Product identity & isolated home - Context

**Gathered:** 2026-07-16
**Status:** Ready for planning

<domain>
## Phase Boundary

Ship the primary CLI as **`bum`** and cut product state over to an isolated home: default `~/.bum` with override `BUM_HOME`. A fresh run must create/use config, auth, sessions, and related product state only under that home — never under `~/.grok` or stock Codex paths. Phase 1 is identity + path isolation only: full UI chrome/string rebrand, quiet-fork telemetry/auto-update kills, multi-provider auth, and model catalog are later phases (esp. Phase 8 for polish).

</domain>

<decisions>
## Implementation Decisions

### Binary shipping
- Rename the shipped `[[bin]]` artifact so the primary command is **`bum`** (not `grok` / `xai-grok-pager` as the user-facing name)
- Keep internal crate paths/packages (e.g. `xai-grok-pager-bin`) as-is this phase — only the user-facing binary name changes
- Phase 1 success = local `cargo build` / `cargo install` (or project-documented build) yields an invocable **`bum`** binary; npm/install-channel/docs polish deferred to Phase 8
- No compatibility alias from `grok` → `bum` in v1

### Home path isolation
- Default product home is **`~/.bum`**
- Primary env override is **`BUM_HOME`**
- Do **not** honor `GROK_HOME` as the product home (avoids silent coupling to stock Grok paths)
- No automatic migration from `~/.grok` / stock Codex stores — clean re-login after switch (PROJECT.md out-of-scope: no import/share)

### Rename depth (phase boundary vs Phase 8)
- Phase 1 must inventory and cut over **all product-state path writers** that use `$GROK_HOME` / `~/.grok` (config, auth, sessions, memory, marketplace cache, logs, leader socket, extracted docs, etc.) so a temp-home run leaves zero product state under `~/.grok`
- UI chrome, help text, and “Grok Build” branding strings → **Phase 8** (except anything that hardcodes a wrong home path or wrong CLI name for invocation)
- Introduce `BUM_HOME` as the home root; leave the broader `GROK_*` knob family for Phase 8 unless a var forces `~/.grok` defaults
- Leave in-repo project layout conventions (e.g. `.grok-plugin`) alone — only product **home** paths change in this phase

### Verification & test strategy
- Prove isolation with integration/unit tests using a temporary home (`BUM_HOME` or sandboxed HOME) and assert product state is written only under that root
- Gate includes **zero writes** under both `~/.grok` and `~/.codex` for product state during a fresh temp-home run
- Prefer unit tests for home-resolution pure logic plus at least one integration-style check that a temp-home run creates expected dirs under bum home only
- Local developer path: document/build so `cargo run -p xai-grok-pager-bin --bin bum` (or equivalent after rename) works

### Claude's Discretion
- Exact inventory of path call sites and the cleanest central home-resolution API (single source of truth for default dir + env)
- Whether to keep a thin internal alias for `GROK_HOME` **only inside tests** temporarily vs full test-support rename in this phase
- How to stage binary rename in Cargo.toml without breaking workspace CI that still references `xai-grok-pager` internally

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- Composition root: `crates/codegen/xai-grok-pager-bin` with `[[bin]] name = "xai-grok-pager"`
- Config layers via `xai-grok-config` under `$GROK_HOME` (default `~/.grok`)
- Auth store: `~/.grok/auth.json` (`GROK_HOME`, `GROK_AUTH_PATH`, `GROK_AUTH`)
- Test support: `EnvGuard` / sandboxed `HOME` + `GROK_HOME` patterns in `xai-grok-test-support` (see codebase TESTING.md)

### Established Patterns
- Env knobs use `GROK_*` prefix (`GROK_HOME`, `GROK_BINARY`, `GROK_LEADER_SOCKET`, …)
- Persistence under `$GROK_HOME`: config.toml, auth.json, sessions, marketplace-cache, mcp_credentials, memory, docs extract
- Root `Cargo.toml` is generated — edit per-crate manifests only

### Integration Points
- Binary name: `xai-grok-pager-bin/Cargo.toml` `[[bin]]`
- Home resolution scattered across config, auth, shell leader socket, marketplace, memory watcher, telemetry/logs — needs inventory + single default cutover
- Phase 8 will own chrome/telemetry/auto-update; Phase 2+ owns multi-slot auth on the isolated home

</code_context>

<specifics>
## Specific Ideas

- Research already recommends: single source of truth for product home default (`~/.bum`) and env override (`BUM_HOME`); no share of `~/.grok`/`~/.codex` in v1
- Temp-home e2e with zero writes under `~/.grok` / `~/.codex` is the isolation proof (PITFALLS / SUMMARY)
- Requirements: **ID-01** (CLI name `bum`), **ID-03** (isolated home)

</specifics>

<deferred>
## Deferred Ideas

- Full UI chrome / help / “Grok Build” string rebrand → Phase 8
- Disable xAI auto-update + product telemetry → Phase 8 (OPS-01/02)
- Multi-slot credentials & xAI OAuth under bum store → Phase 2
- Import/share stock `~/.grok` or Codex credentials → out of scope for v1
- Full `GROK_*` → `BUM_*` env rename family → Phase 8 unless required for home defaults
- Rename crate packages / `.grok-plugin` project layout → later / not Phase 1
- npm install channel / public distribution naming → Phase 8 / out of scope for official x.ai channel

</deferred>
