---
phase: 01-product-identity-isolated-home
reviewed: 2026-07-16T03:57:12Z
depth: deep
files_reviewed: 22
files_reviewed_list:
  - crates/codegen/xai-grok-config/src/paths.rs
  - crates/codegen/xai-grok-config/src/lib.rs
  - crates/codegen/xai-fast-worktree/src/db/mod.rs
  - crates/codegen/xai-fast-worktree/src/db/tests.rs
  - crates/codegen/xai-grok-workspace/src/worktree/mod.rs
  - crates/codegen/xai-grok-pager-render/src/util.rs
  - crates/codegen/xai-grok-pager-bin/Cargo.toml
  - crates/codegen/xai-grok-pager-bin/tests/home_isolation.rs
  - crates/codegen/xai-grok-pager/tests/grok_home_ignore_grok_home.rs
  - crates/codegen/xai-grok-agent/src/discovery.rs
  - crates/codegen/xai-grok-agent/src/plugins/discovery.rs
  - crates/codegen/xai-grok-shell/src/bundle.rs
  - crates/codegen/xai-grok-shell/src/config/mod.rs
  - crates/codegen/xai-grok-shell/src/leader/mod.rs
  - crates/codegen/xai-grok-shell/src/leader/lock.rs
  - crates/codegen/xai-grok-update/src/auto_update.rs
  - crates/codegen/xai-grok-shell-base/src/util/changelog.rs
  - crates/codegen/xai-grok-sandbox/src/paths.rs
  - crates/codegen/xai-grok-test-support/src/env.rs
  - crates/codegen/xai-grok-pager-pty-harness/src/env.rs
  - crates/codegen/xai-grok-mcp/src/credentials.rs
  - crates/codegen/xai-grok-hooks/src/trust.rs
findings:
  critical: 0
  warning: 4
  info: 5
  total: 9
status: issues_found
---

# Phase 1: Code Review Report

**Reviewed:** 2026-07-16T03:57:12Z  
**Depth:** deep  
**Files Reviewed:** 22  
**Status:** issues_found  

## Summary

Adversarial review of Phase 1 product-identity / isolated-home changes (BUM_HOME SoT, binary ship name `bum`, twin resolvers, harness sandboxes, agent/plugin discovery isolation, hermetic proof).

**What holds:** The product-home SoT (`resolve_product_home` / `grok_home()`) is BUM_HOME-only; production `var(_os)?("GROK_HOME")` product-root reads are gone from config/fast-worktree/shell/voice paths; managed leaf is `bin/bum`; agent user dirs no longer scan stock `~/.grok`; `user_plugin_dirs` does not include legacy `~/.grok/plugins`; harnesses set `BUM_HOME` + resolve `CARGO_BIN_EXE_bum`; hermetic `home_isolation` traps GROK/CODEX homes.

**What does not:** Several edge-case contract and path-consistency defects remain — empty `BUM_HOME` handling vs documented `user_grok_home` semantics, multi-resolver divergence when no home resolves, relative override paths, and incomplete gh-release legacy symlink detection for the new product tree. None of these is a shipping blocker for the happy path (`HOME` set, non-empty `BUM_HOME` or default `~/.bum`), but they are real defects under adversarial edge conditions.

## Narrative Findings (AI reviewer)

## Warnings

### WR-01: `user_grok_home` treats empty `BUM_HOME` as resolvable and can return cwd-relative `.bum`

**File:** `crates/codegen/xai-grok-config/src/paths.rs:71-74`  
**Issue:** Docs claim `user_grok_home` never falls back to a cwd-relative `.bum`, and that it is `Some` only when `$BUM_HOME` is set or a home directory is found. Implementation uses `var_os("BUM_HOME").is_some()`, so **empty** `BUM_HOME=""` counts as set. Combined with `resolve_product_home` (which treats empty as absent), when both empty override and no `home_dir` apply:

1. `user_grok_home` → `Some(...)` (empty env key is present)
2. `grok_home()` → `resolve_product_home(Some(""), None)` → `./.bum` (cwd-relative)

Callers that **scan** user-global resources (hooks disable file, MCP credentials, agent discovery via `user_grok_home`) can therefore treat a project's cwd tree as user-global product home — exactly the failure mode the API was designed to prevent. Twin/changelog correctly filter empty with `!v.is_empty()`.

**Fix:**
```rust
pub fn user_grok_home() -> Option<PathBuf> {
    let bum = std::env::var_os("BUM_HOME").filter(|v| !v.is_empty());
    #[allow(deprecated)]
    let resolvable = bum.is_some() || std::env::home_dir().is_some();
    // Prefer pure resolution so empty BUM_HOME cannot force cwd fallback:
    if !resolvable {
        return None;
    }
    if let Some(v) = bum {
        return Some(PathBuf::from(v));
    }
    #[allow(deprecated)]
    Some(resolve_product_home(None, std::env::home_dir()))
    // Do not call grok_home() here if that would create_dir_all on cwd/.bum
}
```
Also add a unit test: `BUM_HOME=""`, inject `user_home=None` path (or process with no home), assert `user_grok_home()` is `None`.

### WR-02: Product-home resolvers diverge when no home / no `BUM_HOME`

**File:** `crates/codegen/xai-grok-config/src/paths.rs:22-33`  
**File:** `crates/codegen/xai-fast-worktree/src/db/mod.rs:355-367`  
**File:** `crates/codegen/xai-grok-workspace/src/worktree/mod.rs:627-632`  
**Issue:** Three “product home” paths disagree in the no-home corner:

| Resolver | No `BUM_HOME`, no user home |
| --- | --- |
| `resolve_product_home` / `grok_home()` | `cwd/.bum` (and `create_dir_all`) |
| `xai_fast_worktree::resolve_grok_home` | `Err(...)` |
| workspace `worktree::grok_home()` | `/tmp/.bum` via `unwrap_or_else` |

A single process can therefore put auth/config/sessions under `./.bum` while worktree DB/checkouts land under `/tmp/.bum`. Shared `/tmp/.bum` is also multi-user writable (symlink / data cross-talk risk) when `HOME` is unset (containers, scrubbed env).

**Fix:** Pick one contract for “unresolvable home” (prefer fail closed / `Err` / no writes) and share it. Minimal alignment:

```rust
// workspace worktree — match twin, do not invent /tmp/.bum
fn grok_home() -> std::path::PathBuf {
    xai_fast_worktree::resolve_grok_home().unwrap_or_else(|_| {
        // Same shape as config only if intentional; better: propagate / skip worktree ops
        std::env::temp_dir().join(format!("bum-{}-home", std::process::id()))
    })
}
```

Better: make workspace return `Result` when product home cannot resolve, and stop creating product trees under shared `/tmp`.

### WR-03: Non-empty relative `BUM_HOME` is not normalized (cwd-sensitive product root)

**File:** `crates/codegen/xai-grok-config/src/paths.rs:26-29`  
**File:** `crates/codegen/xai-fast-worktree/src/db/mod.rs:356-359`  
**Issue:** When `BUM_HOME` is set and non-empty, both SoT and twin return `PathBuf::from(v)` **without** absolutizing or dunce-canonicalizing. Default path *does* canonicalize user home. If override is relative (`BUM_HOME=./data/bum`) and any subsystem `chdir`s (worktree checkout, tools), subsequent `join`s resolve under a different physical tree. `OnceLock` freezes the first relative path string, but open handles and later `resolve_grok_home` re-reads can still disagree after cwd changes. Path-handling security/robustness gap for the override path.

**Fix:**
```rust
if let Some(v) = bum_home.filter(|v| !v.is_empty()) {
    let p = PathBuf::from(v);
    return dunce::canonicalize(&p).unwrap_or_else(|_| {
        // Absolutize even if path does not exist yet
        std::env::current_dir().map(|cwd| cwd.join(&p)).unwrap_or(p)
    });
}
```
Apply the same normalization in the twin `resolve_grok_home`.

### WR-04: GH-release installer still only rewrites system links whose targets contain `.grok/downloads/`

**File:** `crates/codegen/xai-grok-update/src/auto_update.rs:1957-1969`  
**Issue:** After cutover, downloads live under `$BUM_HOME/downloads` (typically `~/.bum/downloads`). The post-install loop that updates `/usr/local/bin/{grok,agent}` only matches `target_str.contains(".grok/downloads/")`. Links into `~/.bum/downloads/...` are skipped, so a legacy system path that was already migrated under product home will not be refreshed. Managed `bin/bum` swap is correct; this is the residual system-link branch.

**Fix:**
```rust
let under_product_downloads = target_str.contains(".bum/downloads/")
    || target_str.contains(".grok/downloads/");
if under_product_downloads && !target_str.ends_with("grok-latest") {
    let _ = atomic_symlink_swap(&binary_path, &system_link).await;
}
// Consider also ["bum", "agent"] for /usr/local/bin once installers ship `bum`.
```

## Info

### IN-01: Clap / chrome still product-name `grok`

**File:** `crates/codegen/xai-grok-pager/src/app/cli.rs:385-387`  
**Issue:** Binary ship name is `bum`, but clap `name = "grok"` and about text remain Grok Build — matches deferred Phase 8 chrome (`bum version` still brands as grok). Not an isolation writer bug.  
**Fix:** Phase 8 rebrand: `name = "bum"`, about string, version display.

### IN-02: Completions still written as `grok.*` under product home

**File:** `crates/codegen/xai-grok-update/src/auto_update.rs:1266-1269`  
**Issue:** `regenerate_completions` writes `completions/bash/grok.bash`, `zsh/_grok`, fish `grok.fish` even though the managed binary leaf is `bum`. Stale comment still mentions GROK_HOME override (line 1261).  
**Fix:** Rename completion basenames to `bum` when shell completion command surface is rebranded; update comment to BUM_HOME.

### IN-03: Stale isolation comments still describe `$GROK_HOME` / legacy `~/.grok` user plugins

**File:** `crates/codegen/xai-grok-agent/src/plugins/discovery.rs:340-342`  
**File:** `crates/codegen/xai-grok-plugin-marketplace/src/git.rs:4` (module docs)  
**Issue:** Code correctly uses `user_grok_home()` / `g.join("plugins")` only; comments still say `$GROK_HOME/plugins` and “legacy ~/.grok/plugins”. Misleads future edits into reintroducing stock scans.  
**Fix:** Rewrite comments to `$BUM_HOME/plugins` / product home only; drop “legacy ~/.grok” claim (already removed; unit test `user_plugin_dirs_are_grok_and_claude_only_no_legacy` proves it).

### IN-04: Hooks trust / disable docs lost path text after rebrand edits

**File:** `crates/codegen/xai-grok-hooks/src/trust.rs:42-43, 60`  
**Issue:** Doc comments read “Disabled hooks are listed in , one hook name per line” and “Adds to .” — path fragments stripped (likely during `~/.bum` substitution). Error strings correctly teach `$BUM_HOME` / `~/.bum`.  
**Fix:** Restore full paths in rustdoc, e.g. `` `$BUM_HOME/disabled-hooks` ``.

### IN-05: Hermetic isolation test only runs `bum version`

**File:** `crates/codegen/xai-grok-pager-bin/tests/home_isolation.rs:97-98`  
**Issue:** Strong trap/snapshot design, but the child only executes `version`. Does not exercise auth write, session persistence, marketplace cache, or worktree DB open — residual risk that some writer still keys off ambient paths only under interactive/agent modes. Phase verification supplements this with static gates; still a coverage gap.  
**Fix:** Optional second scenario (feature-gated or ignored if slow): `bum login --help` / headless agent one-shot with mock URLs under same traps, or unit-level inventory of writers already covered by SoT `grok_home()`.

---

### Positive observations (not findings)

- Pure `resolve_product_home` + empty-override fall-through tests are solid.
- Twin `resolve_grok_home` uses `var_os` + non-Unicode coverage (Unix).
- Agent `user_agent_dirs` only uses product home + `~/.claude` — stock `~/.grok` excluded with regression asserts.
- Sandbox writable product path re-exports config SoT (no second resolver).
- Managed bin leaf aligned across leader + `swap_managed_bin_links`.

---

_Reviewed: 2026-07-16T03:57:12Z_  
_Reviewer: Claude (gsd-code-reviewer)_  
_Depth: deep_
