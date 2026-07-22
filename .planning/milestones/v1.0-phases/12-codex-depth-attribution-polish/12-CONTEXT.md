# Phase 12: Codex Depth & Attribution Polish - Context

**Gathered:** 2026-07-21
**Status:** Ready for planning

<domain>
## Phase Boundary

Close v1 with an explicit Codex parity scope, honest bum-versus-provider attribution, and maintained capability-gap documentation. Validate and document existing compatible surfaces; do not introduce a risky new transport or broad tool rename campaign.

</domain>

<decisions>
## Implementation Decisions

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

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `README.md` already links the root and tool-specific third-party notices.
- `crates/codegen/xai-grok-tools/src/implementations/codex/apply_patch/` already provides the Codex-derived patch surface.
- `crates/codegen/xai-grok-tools/THIRD_PARTY_NOTICES.md` and the root `THIRD-PARTY-NOTICES` already record Codex tool ancestry.
- `CODEX_ORIGINATOR` is already defined as `bum`, with OAuth and routing tests covering the value.

### Established Patterns
- Prior phases treat official Codex source as an interoperability reference, never as product identity.
- Provider-sensitive behavior is fail-closed and route-scoped; xAI, BYOK, and custom endpoints must remain unaffected.
- User-facing product chrome is bum, while legitimate provider and model brands remain visible.

### Integration Points
- Primary product and attribution documentation begins at `README.md` and the pager user guide.
- Tool compatibility and attribution connect through the tool registry, Codex implementation module, and crate-local notices.
- Identity checks connect to auth originator constants, sampler trusted-route headers, CLI help, and TUI/user-guide strings.

</code_context>

<specifics>
## Specific Ideas

- Prefer a compact, honest provider capability matrix over claims of full stock-CLI parity.
- Explicitly call out the existing HTTP/SSE transport as supported and Codex Responses WebSocket incremental transport as deferred.
- Keep attribution informative and discoverable without turning every screen into a disclaimer.

</specifics>

<deferred>
## Deferred Ideas

- Codex Responses WebSocket incremental transport.
- Broad Codex tool-name parity or rename campaign.
- Any additional live dual-login gate unless this phase changes runtime or wire behavior.

</deferred>
