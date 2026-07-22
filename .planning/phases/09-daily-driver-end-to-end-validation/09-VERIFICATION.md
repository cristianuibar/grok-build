---
status: passed
phase: 09-daily-driver-end-to-end-validation
verified: 2026-07-20
mode: hybrid (automated residual + live operator UAT)
operator: Cristian
nyquist_compliant: true
---

# Phase 9 Verification — Daily-driver end-to-end validation

## Goal-backward check

Phase goal: bum works as a real daily driver — live sessions on both providers, mid-session switch, cross-provider subagents.

| Requirement | Evidence | Verdict |
|-------------|----------|---------|
| OPS-03 xAI productive session | `09-UAT.md` OPS-03, operator live 2026-07-18 | PASS |
| OPS-04 Codex productive session | `09-UAT.md` OPS-04 (promoted from `10-VALIDATION.md` live evidence, 2026-07-18) | PASS |
| OPS-05 mid-session switch, same process | `09-UAT.md` OPS-05 (Grok → gpt-5.6-luna, 2026-07-18) | PASS |
| OPS-06 cross-provider spawn, both directions | `09-UAT.md` OPS-06 Direction A Run 2 + Direction B, operator live 2026-07-20; result_returned=yes both ways; parent model unchanged both ways (verified via /model) | PASS |

All four rows carry **live** operator evidence — no fixture/mock substitution (D-16 honored).

## In-phase fixes required to reach OPS-06 PASS (2026-07-20)

Live UAT surfaced four product blockers on the Codex path, all root-caused and fixed in-phase (details + file references in `09-UAT.md` Direction A Run 2):

1. Unknown Responses SSE frame types (new backend `response.metadata`) killed streams — now skipped tolerantly.
2. Subagent children hard-coded `ResponsesWireProfile::Disabled` — now derived from the child's resolved model + auth.
3. TrustedCodex wire profile lacked official-Codex parity: now strips `temperature`/`top_p`/`max_output_tokens`, adds `strict` on function tools and `prompt_cache_key`.
4. Backend now sends terminal `response.completed` with empty `output` after streaming tool-call items — final response reconstructed from accumulated `output_item.done` items (this was the `empty_response (no_visible_content)` retry storm affecting **all** tool-using Codex turns).

Regression coverage: `xai-grok-sampler` lib tests (165, incl. new unknown-frame + trusted-profile assertions), `cross_provider_subagent.rs` 12/12, new wire-profile unit tests.

## Deferred (non-blocking)

- Codex-parent Task allow-list hides session-auth-only xAI slugs (`grok-build`); operator-approved fallback `grok-4.5` used in Direction B. Fix: per-provider `visible_for_auth`. → Phase 11/12 backlog.
- Requested-vs-effective child slug surfacing in Task UX.
- Subagent path lacks per-turn Codex bearer refresh (uses spawn-time snapshot); headers otherwise at parity. Follow-up if long-running children hit token expiry.
- Residual `Usage: grok` / `~/.grok` chrome in `--help` — accepted cosmetic; rebrand sweep is the next task.

## Verdict

Phase 9 **passed**. OPS-03..06 all live-PASS with operator sign-off (`09-UAT.md` §Sign-off).
