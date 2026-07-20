---
quick_id: 260720-t38
slug: pager-lib-test-failures
date: 2026-07-20
status: complete
---

# Summary — 21 `xai-grok-pager --lib` failures fixed

`cargo test -p xai-grok-pager --lib`: **7142 passed, 0 failed, 10 ignored** (was 21 failed).

## Per cluster

1. **paste (16) + dashboard (2) — stale test support.** The render crate gained a
   `test-helpers` feature but the clipboard read seams were still gated on bare
   `#[cfg(test)]`, so hooks were compiled out for downstream consumers and reads
   returned empty. Six seams in `xai-grok-pager-render/src/clipboard/mod.rs` now use
   `#[cfg(any(test, feature = "test-helpers"))]`.
2. **event_loop (2) — real code bug.** `x11_primary_tool_available` used
   `std::ptr::eq` against `const ToolSpec` values; const promotion means references
   need not be pointer-identical, and `WL_SPEC` reached the X11-only path and tripped
   the `debug_assert!`. Replaced with semantic classification
   (`classify_x11_primary_tool`); unknown/Wayland specs return unavailable. Regression
   test added.
3. **dispatch (1) — stale test.** `SessionCreated` now emits an additional
   `RefreshProviderAuthStatus` effect (8, not 7). Test updated and strengthened to
   assert the new effect explicitly rather than just bumping the count.

No test deleted, skipped, or `#[ignore]`d. Delegated to Codex (gpt-5.6-sol/high);
full-suite verification run outside the sandbox.

## Out of scope (pre-existing, untouched)

`xai-file-utils`: `queue::tests::cleanup_orphans_uses_sidecar_age_for_pairs` fails
("expired-by-sidecar temp removed", queue.rs:6389). Separate crate, separate cause.
