---
status: complete
phase: 11-codex-effort-catalog-fidelity
source: [11-VERIFICATION.md]
started: 2026-07-21T20:15:00Z
updated: 2026-07-21T21:00:00Z
---

## Current Test

number: 2
name: In-flight-turn wire-shape immutability under concurrent model switch
expected: |
  Sign-off that `clamp_reasoning_effort`'s output-membership guarantee, the soft-clamp-never-error
  control flow in `model_switch.rs::apply`, and the raw-vs-effective field separation in
  `SessionHandle`/`model_switch.rs` are accepted as satisfying these prohibitions:
  (1) never widen the clamped effort beyond the model's catalog-advertised supported list,
  (2) never hard-fail a turn/switch because the previously active effort is unsupported,
  (3) never mutate the user's stored/sticky effort preference as a side effect of clamping
  a single request.
awaiting: complete

## Tests

### 1. Three phase-locked prohibitions hold as product policy
expected: Sign-off accepted per above.
result: passed — user accepted all three prohibitions as product invariants

### 2. In-flight-turn wire-shape immutability under concurrent model switch
expected: |
  Exercise a real mid-turn model switch while a request is genuinely in flight against a
  live/near-live backend and confirm the already-built in-flight request is not mutated,
  and the very next request clamps against the new model's supported list.
result: passed — live validation confirmed the in-flight request retained its original wire shape and the following request used the new model with catalog-clamped effort

## Summary

total: 2
passed: 2
issues: 0
pending: 0
skipped: 0
blocked: 0

## Gaps
