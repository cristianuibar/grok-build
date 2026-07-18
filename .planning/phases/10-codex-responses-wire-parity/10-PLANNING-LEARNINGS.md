---
phase: 10
slug: codex-responses-wire-parity
status: pre-execution
captured: 2026-07-18
scope: phase planning, review orchestration, and Grok delegation defaults
formal_extraction: deferred until execution produces SUMMARY artifacts
---

# Phase 10 Planning Learnings: Codex Responses Wire Parity

This is a pre-execution record. The formal `10-LEARNINGS.md` extraction remains deferred because
Phase 10 has no execution summaries yet.

## Decisions

### Keep the Codex Responses profile semantic and trust-gated

The sampler receives a disabled-by-default typed profile, while the shell alone decides whether a
first-party ChatGPT/Codex OAuth route may enable it. This prevents the profile or Codex-only headers
from leaking to xAI, BYOK, or custom endpoints.

**Rationale:** Provider isolation is a product and privacy constraint, not a sampler heuristic.
**Source:** `10-CONTEXT.md`; `10-01-PLAN.md`; `10-04-PLAN.md`.

### Preserve ordinary history while removing only foreign encrypted reasoning

On a true provider transition, remove only provider-scoped encrypted reasoning and preserve normal
user, assistant, and tool history; same-provider Codex continuity remains intact.

**Rationale:** The live failure is an incompatible encrypted payload, not a reason to discard useful
session context.
**Source:** `10-CONTEXT.md`; `10-03-PLAN.md`.

### Keep live OPS-04 and OPS-05 proof blocking

Automated serialization, fake-SSE, header-isolation, and switch tests precede—but never replace—the
redacted rebuilt-bum live validation.

**Rationale:** Fixture success cannot establish daily-driver reliability against real dual OAuth and
provider behavior.
**Source:** `10-VALIDATION.md`; `10-05-PLAN.md`.

## Lessons

### Preserve Grok web search for research and reviews

Do not default to `--disable-web-search` for Grok research or review runs. Disable it only for an
explicit no-network or privacy constraint.

**Context:** User correction during Phase 10 cross-AI review setup.
**Source:** User direction, 2026-07-18.

### Use a direct review rather than permission-mode planning

`--permission-mode plan` is unnecessary when the requested output is an adversarial review. Reserve
it for a user-requested plan and use a direct source-grounded review prompt otherwise.

**Context:** User correction during Phase 10 cross-AI review setup.
**Source:** User direction, 2026-07-18.

### Treat external-review output as evidence, not invocation success

A usable JSON review needs a successful process result, non-error object, non-empty text, and a
non-cancelled/non-max-turn stop reason. Blank or cancelled output is an unavailable lane, never a
clean review.

**Context:** Initial reviewer attempts produced blank or cancelled output; a later direct JSON health
check returned `GROK_JSON_READY` with `stopReason: EndTurn` under the corrected defaults.
**Source:** Phase 10 cross-AI review orchestration, 2026-07-18.

### Split large source reviews into focused evidence slices

The full Phase 10 source corpus exceeded the effective response capacity of external review lanes,
but focused plan/source slices produced usable findings. Treat a focused review as scoped evidence,
record unavailable whole-corpus attempts explicitly, and independently verify each recommendation
against the source before revising plans.

**Context:** Focused Claude and Grok lanes exposed test, late-response, outbound-header, and
pre-compaction recovery gaps; one Grok suggestion about a single switch seam was rejected after
source verification showed a late-response race.
**Source:** Phase 10 cross-AI review and source-verification pass, 2026-07-18.

### Fully qualify Rust library tests when using `--exact`

Rust test harness names include their module path. A bare test function combined with
`cargo test --lib … -- --exact` can succeed while running zero tests. Plan commands must either
use the fully qualified harness name or add a non-vacuous discovery assertion before execution.

**Context:** The Phase 10 revision audit found false-green filters in Plans 01–04 and the
consolidated Plan 05 suite.
**Source:** Phase 10 command audit, 2026-07-18.

### Prove the claim at the turn-loop boundary

A helper-level recovery test can establish classification but cannot prove that the actual loop
avoids compaction and resubmission. When the requirement names an outbound request count or a
retry/compaction side effect, use an end-to-end mock turn that observes that boundary directly.

**Context:** The encrypted-content 400 recovery initially targeted `handle_sampling_failure`, while
the resubmit occurs later in the turn loop.
**Source:** Phase 10 source audit, 2026-07-18.

## Patterns

### Forward a reviewer flag through the entire GSD path

A selector flag must be documented, parsed by autonomous mode, parsed by convergence, detected by
review, recognized by reviewer selection, invoked, and represented in `REVIEWS.md` consensus.

**When to use:** Adding any new external reviewer to GSD.
**Source:** GSD autonomous/convergence/review audit, 2026-07-18.

### Follow shared configuration changes with literal-compatibility waves

When a Rust configuration type gains a field, update its core default, then production literals, and
then fixture literals so compilation and testing follow the dependency graph.

**When to use:** Cross-crate Rust configuration changes.
**Source:** `10-01-PLAN.md`; `10-06-PLAN.md`; `10-07-PLAN.md`.

### Repair reconstruction before weakening retry policy

Accumulate visible text deltas in the stream transformer and use them only when terminal output lacks
text; leave genuinely empty-response retry behavior bounded.

**When to use:** A retry loop follows a visibly successful streamed response.
**Source:** `10-RESEARCH.md`; `10-02-PLAN.md`.

## Surprises

### `--grok` was silently dropped by GSD convergence

The user-specified flag was not forwarded by autonomous mode and was unsupported by the review
workflow, even though a Grok delegation skill already existed.

**Impact:** The GSD skills and workflows now forward, detect, invoke, validate, and record Grok as a
reviewer; a dropped Grok lane cannot masquerade as no findings.
**Source:** GSD workflow audit, 2026-07-18.

### The Grok delegation skill had contradictory effort defaults

Its canonical examples used `--effort max`, but the installed Grok 4.5 CLI accepts
`--reasoning-effort high|medium|low` and rejects `max`.

**Impact:** The skill now uses the accepted `--reasoning-effort high` default.
**Source:** `grok --help`, Grok 0.2.103; Grok delegation skill audit, 2026-07-18.

### Final learning extraction must wait for execution evidence

The formal extraction workflow requires both plan and summary artifacts, while Phase 10 currently has
plans only.

**Impact:** This durable pre-execution note preserves current lessons without fabricating final
execution learnings.
**Source:** `gsd-extract-learnings` workflow; Phase 10 artifact inventory, 2026-07-18.
