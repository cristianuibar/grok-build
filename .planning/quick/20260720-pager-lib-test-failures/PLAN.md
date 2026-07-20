---
quick_id: 260720-t38
slug: pager-lib-test-failures
date: 2026-07-20
status: in-progress
---

# Fix 21 `xai-grok-pager --lib` test failures

Three clusters, likely three distinct causes:

1. **paste (16 tests)** — `app::agent_view::paste::paste_key_tests::*`. Inserts yield `""`;
   deferred-probe expectations inverted. Determine whether the paste pipeline regressed or
   the tests encode a stale contract.
2. **event_loop (2 tests)** — panic in `xai-grok-shared/src/clipboard.rs:1662`
   `debug_assert!(std::ptr::eq(spec, &XSEL_SPEC))`. `x11_primary_tool_available` is reached
   with a non-X11 spec (WL_SPEC). Code bug, not test.
3. **dashboard (2 tests) + dispatch (1 test)** — same paste-deferred-context contract;
   `session_created_sets_session_id` off-by-one (8 vs 7).

Precedent: commit 4a78ded (stale shell lib tests updated to current behavior).

Acceptance: `cargo test -p xai-grok-pager --lib` green; no test deleted or `#[ignore]`d to pass.
