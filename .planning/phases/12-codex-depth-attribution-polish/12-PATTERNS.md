# Phase 12: Codex Depth & Attribution Polish - Pattern Map

**Mapped:** 2026-07-21
**Files analyzed:** 30 likely new/modified files, plus 4 verify-only runtime/legal surfaces
**Analogs found:** 30 / 30

## Scope Interpretation

Phase 12 is a documentation-contract and regression-gate phase. The closest patterns are already in the tree: the pager embeds its user guide through `include_str!`, Phase 8 combines positive and negative identity assertions with non-vacuous static scans, and `xai-grok-agent` tests toolset composition next to the preset builders. No new transport, tool implementation, dependency, or notice text is implied by the locked scope.

The planner should treat all 22 files registered in `USER_GUIDE` and both files registered in `REFERENCE_DOCS` as shipped product surfaces. The user-guide `README.md` is a repository discovery index rather than a registered runtime document, but it should be corrected and should link to the canonical capability matrix.

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `README.md` | component (documentation entry point) | transform | existing `README.md:3-40,85-93,135-141` | exact |
| `crates/codegen/xai-grok-pager/docs/user-guide/README.md` | component (documentation index) | transform | `README.md:21-40,85-93` | role-match |
| `crates/codegen/xai-grok-pager/docs/user-guide/01-getting-started.md` | component (embedded guide) | transform | `README.md:3-40` + Phase 8 allowlist | role-match |
| `crates/codegen/xai-grok-pager/docs/user-guide/02-authentication.md` | component (embedded guide/capability matrix) | transform | `README.md:28-35,71-93` | exact |
| `crates/codegen/xai-grok-pager/docs/user-guide/03-keyboard-shortcuts.md` | component (embedded guide) | transform | `README.md:3-40` + Phase 8 allowlist | role-match |
| `crates/codegen/xai-grok-pager/docs/user-guide/04-slash-commands.md` | component (embedded guide) | transform | `README.md:3-40` + Phase 8 allowlist | role-match |
| `crates/codegen/xai-grok-pager/docs/user-guide/05-configuration.md` | component (embedded guide) | transform | `README.md:23-40,68-69` + Phase 8 allowlist | role-match |
| `crates/codegen/xai-grok-pager/docs/user-guide/06-theming.md` | component (embedded guide) | transform | `README.md:3-40` + Phase 8 allowlist | role-match |
| `crates/codegen/xai-grok-pager/docs/user-guide/07-mcp-servers.md` | component (embedded guide) | transform | `README.md:3-40` + Phase 8 allowlist | role-match |
| `crates/codegen/xai-grok-pager/docs/user-guide/08-skills.md` | component (embedded guide) | transform | `README.md:3-40` + Phase 8 allowlist | role-match |
| `crates/codegen/xai-grok-pager/docs/user-guide/09-plugins.md` | component (embedded guide) | transform | `README.md:3-40` + Phase 8 allowlist | role-match |
| `crates/codegen/xai-grok-pager/docs/user-guide/10-hooks.md` | component (embedded guide) | transform | `README.md:3-40` + Phase 8 allowlist | role-match |
| `crates/codegen/xai-grok-pager/docs/user-guide/11-custom-models.md` | component (embedded guide) | transform | `README.md:3-40` + Phase 8 allowlist | role-match |
| `crates/codegen/xai-grok-pager/docs/user-guide/12-project-rules.md` | component (embedded guide) | transform | `README.md:3-40` + Phase 8 allowlist | role-match |
| `crates/codegen/xai-grok-pager/docs/user-guide/13-memory.md` | component (embedded guide) | transform | `README.md:3-40` + Phase 8 allowlist | role-match |
| `crates/codegen/xai-grok-pager/docs/user-guide/14-headless-mode.md` | component (embedded guide) | request-response | `README.md:52-69` + Phase 8 CLI residuals | role-match |
| `crates/codegen/xai-grok-pager/docs/user-guide/15-agent-mode.md` | component (embedded guide) | request-response | `README.md:3-40` + Phase 8 allowlist | role-match |
| `crates/codegen/xai-grok-pager/docs/user-guide/16-subagents.md` | component (embedded guide) | event-driven | `README.md:3-40` + Phase 8 allowlist | role-match |
| `crates/codegen/xai-grok-pager/docs/user-guide/17-sessions.md` | component (embedded guide) | CRUD | `README.md:3-40` + Phase 8 allowlist | role-match |
| `crates/codegen/xai-grok-pager/docs/user-guide/18-sandbox.md` | component (embedded guide) | request-response | `README.md:3-40` + Phase 8 allowlist | role-match |
| `crates/codegen/xai-grok-pager/docs/user-guide/19-plan-mode.md` | component (embedded guide) | event-driven | `README.md:3-40` + Phase 8 allowlist | role-match |
| `crates/codegen/xai-grok-pager/docs/user-guide/20-background-tasks.md` | component (embedded guide) | event-driven | `README.md:3-40` + Phase 8 allowlist | role-match |
| `crates/codegen/xai-grok-pager/docs/user-guide/21-terminal-support.md` | component (embedded guide) | request-response | `README.md:3-40` + Phase 8 allowlist | role-match |
| `crates/codegen/xai-grok-pager/docs/user-guide/22-permissions-and-safety.md` | component (embedded guide) | request-response | `README.md:3-40` + Phase 8 allowlist | role-match |
| `crates/codegen/xai-grok-pager/docs/hooks-and-plugins.md` | component (embedded reference) | event-driven | user-guide docs + `docs.rs:165-177` | exact |
| `crates/codegen/xai-grok-pager/docs/custom-hooks.md` | component (embedded reference) | event-driven | user-guide docs + `docs.rs:165-177` | exact |
| `crates/codegen/xai-grok-pager/src/docs.rs` | utility + test | file-I/O / transform | existing `docs.rs:37-44,48-177,254-357`; Phase 8 product tests | exact |
| `crates/codegen/xai-grok-agent/src/config.rs` | config + test | transform | existing preset-composition tests in `config.rs:1634-1724` | exact |
| `.planning/phases/12-codex-depth-attribution-polish/12-VALIDATION.md` | config (validation contract) | batch | existing Phase 12 validation + Phase 8 gate inventory | exact |
| `.planning/phases/12-codex-depth-attribution-polish/12-PHASE-GATE.md` | test/runbook | batch | `.planning/phases/08-quiet-fork-rebrand-polish/08-PHASE-GATE.md` | exact |

## Pattern Assignments

### `README.md` (documentation entry point, transform)

**Analog:** the existing root README already distinguishes product identity from upstream lineage and links both notice layers.

**Product identity and lineage pattern** (`README.md:21-40`):

```markdown
This repository is the Rust workspace for the **bum** CLI/TUI and agent
runtime. It keeps the Grok Build harness lineage (crates remain `xai-grok-*`,
model catalog still includes **Grok Build (xAI)** / `grok-build`) and evolves
it into a private multi-provider product:

| Concern | bum |
|---------|-----|
| CLI binary | `bum` |
| Product name | **bum** / **BUM** (*Build Using Multiagents*) |
```

Copy this distinction for the new Codex disclaimer: bum uses ChatGPT/Codex OAuth and compatible APIs, but is not the stock OpenAI Codex CLI. Keep the legitimate Grok/Codex model and provider brands.

**Discoverability pattern** (`README.md:85-93`):

```markdown
## Documentation

In-tree user guide (pager crate):

[`crates/codegen/xai-grok-pager/docs/user-guide/`](crates/codegen/xai-grok-pager/docs/user-guide/)
```

Replace the stale `~/.grok` caveat at lines 91-93 with a direct link to the canonical capability-matrix anchor in `02-authentication.md`. Do not duplicate the whole matrix in the README.

**Notice-link pattern** (`README.md:135-141`):

```markdown
- [`THIRD-PARTY-NOTICES`](THIRD-PARTY-NOTICES) — ... in-tree source ports
  (including openai/codex ...)
- [`crates/codegen/xai-grok-tools/THIRD_PARTY_NOTICES.md`](...)
  — crate-local notice for the codex and opencode ports
```

Preserve these links. README/doc-only changes do not trigger notice edits.

---

### `02-authentication.md` (canonical capability disclosure, transform)

**Analog:** the compact concern table in `README.md:28-35`, placed inside the already embedded Authentication guide.

**Table pattern:**

```markdown
| Concern | bum |
|---------|-----|
| Auth | xAI OAuth (Grok) + ChatGPT/Codex OAuth (GPT), dual slots |
| Routing | Per-model provider selection (mixed picker) |
```

Use one concise matrix with columns such as `Area`, `xAI/Grok in bum`, `ChatGPT/Codex in bum`, and `Compatibility note`. It must carry stable semantic markers for:

- bum is not the stock OpenAI Codex CLI;
- separate xAI and ChatGPT/Codex OAuth slots under `~/.bum` / `BUM_HOME`;
- Responses over HTTP POST + SSE is supported;
- Responses WebSocket incremental transport is deferred and non-blocking;
- trusted Codex OAuth uses bum-owned `originator`/session metadata, while BYOK/custom/xAI routes do not inherit it;
- a normal bum session keeps bum's tool harness across compatible model switches;
- the optional Codex preset contains bash plus Codex-derived read, `apply_patch`, list, and grep ports;
- `codex:apply_patch` is behaviorally compatible but uses JSON `{ patch }`, not a promise of stock CLI wire/name parity;
- broad Codex tool-name parity is deferred;
- Phase 10 already covered the live daily-driver path, so this docs-only phase has no new live dual-login gate.

The file currently contains stock product/executable/global-home wording (for example `02-authentication.md:1-45`). Correct that wording while classifying internal compatibility knobs such as `GROK_OIDC_*` separately; do not blindly rename them.

---

### Embedded user-guide identity sweep (22 guide files, transform/request documentation)

**Applies to:** `01-getting-started.md` through `22-permissions-and-safety.md` exactly as registered in `USER_GUIDE` at `docs.rs:48-159`.

**Analog:** Phase 8's explicit positive/negative identity boundary, not a global search-and-replace.

**Positive + negative assertion pattern** (`crates/codegen/xai-grok-pager/src/views/welcome/mod.rs:2496-2511`):

```rust
assert_eq!(PRODUCT_BADGE_LABEL.trim(), "bum");
assert!(
    !PRODUCT_BADGE_LABEL.contains("Grok Build"),
    "badge must not stock product: {PRODUCT_BADGE_LABEL:?}"
);
```

For prose edits, change product subjects, executable examples, install/update commands, and user-global home paths to bum. Preserve the meaning of each page; this is an identity correction, not a feature rewrite.

**Explicit allowlist pattern** (`08-PHASE-GATE.md:255-265`):

```markdown
| Model label `Grok Build (xAI)` / id `grok-build` | D-02 model brand |
| Crate/package `xai-grok-*` | D-03 internal names |
| Managed host `grok.com` | Host identity, not CLI binary name |
| Dev env knobs `GROK_*` | D-13 leave-internal |
```

Extend that classification for Phase 12 with provider/model names (`Codex`, `GPT-5.6 Sol (Codex)`), historical fork lineage, legal notices, project-local `.grok/` compatibility directories, and internal crate/type/env identifiers. Do not allow statements that bum itself is Grok Build or Codex CLI.

---

### `docs/user-guide/README.md` (guide index, transform)

**Analog:** root README identity wording (`README.md:3-8`) and existing Markdown index table (`docs/user-guide/README.md:11-17`).

```markdown
# bum — Build Using Multiagents

**bum** is a full-product fork of the Grok Build terminal AI coding agent.
It is a multi-provider daily driver: one CLI (`bum`), dual OAuth ...
```

Rename the index's product subject to bum, update the headless example at current line 44 from `grok -p` to `bum -p`, and make Authentication's description mention the provider capability matrix so it is discoverable. Keep the existing relative link to `02-authentication.md`.

---

### `docs/hooks-and-plugins.md` and `docs/custom-hooks.md` (embedded references, event-driven)

**Analog:** registration in `docs.rs:165-177` proves these are runtime content even though they are outside `docs/user-guide/`.

```rust
static REFERENCE_DOCS: &[Doc] = &[
    Doc {
        filename: "hooks-and-plugins.md",
        ...
        content: include_str!("../docs/hooks-and-plugins.md"),
    },
    Doc {
        filename: "custom-hooks.md",
        ...
        content: include_str!("../docs/custom-hooks.md"),
    },
];
```

Apply the same identity/executable sweep as the numbered guides. The pager test must iterate `USER_GUIDE.iter().chain(REFERENCE_DOCS.iter())`; testing only the extracted numbered docs misses these two shipped references.

---

### `crates/codegen/xai-grok-pager/src/docs.rs` (embedded-doc tests, file-I/O/transform)

**Analog:** current table-wide validation tests at `docs.rs:254-299` and direct lookup tests at `docs.rs:301-318`.

**Whole-inventory loop** (`docs.rs:258-277`):

```rust
#[test]
fn user_guide_entries_are_valid() {
    for doc in USER_GUIDE {
        assert!(!doc.content.is_empty(), "Doc {} is empty", doc.filename);
        assert!(
            doc.content.starts_with('#'),
            "Doc {} should start with a markdown header",
            doc.filename
        );
    }
}
```

Add `p12_embedded_docs_use_bum_product_identity` beside these tests. Iterate both static tables, report `doc.filename` in every failure, and use semantic forbidden patterns with an explicit allowlist/classification. Avoid a blanket ban on the words `Grok`, `Codex`, or `.grok`.

**Positive capability contract** (`docs.rs:301-318` lookup style):

```rust
let auth = get_howto_doc("Authentication").expect("embedded auth guide");
for required in [
    "not the stock OpenAI Codex CLI",
    "HTTP",
    "SSE",
    "WebSocket",
    "deferred",
    "originator",
    "apply_patch",
] {
    assert!(auth.contains(required), "missing capability marker: {required}");
}
```

Name it `p12_capability_disclosure_is_embedded_and_complete`. Assert stable semantic markers, not an entire paragraph. This makes the test non-vacuous and proves the canonical matrix is in content compiled into the pager.

**Extraction behavior to preserve** (`docs.rs:217-230`):

```rust
pub fn extract_user_guide_docs(grok_home: &std::path::Path) {
    let docs_dir = grok_home.join("docs").join("user-guide");
    ...
    for doc in USER_GUIDE {
        std::fs::write(docs_dir.join(doc.filename), doc.content)
    }
}
```

No delivery seam or new document registration is needed when the matrix is placed in existing `02-authentication.md`.

---

### `crates/codegen/xai-grok-agent/src/config.rs` (toolset identity test, transform)

**Analog:** preset resolution/composition tests at `config.rs:1634-1724`.

**Production composition to lock** (`config.rs:338-353`):

```rust
fn codex_toolset() -> ToolServerConfig {
    ToolServerConfig {
        tools: vec![
            bash_tool_config(),
            (&codex::CodexReadFileTool).into(),
            (&codex::ApplyPatchTool).into(),
            (&codex::CodexListDirTool).into(),
            (&codex::CodexGrepFilesTool).into(),
            ...
        ],
        behavior_preset: None,
    }
}
```

**Existing identity-test style** (`config.rs:1674-1695`):

```rust
let gc = toolset_for_preset("grok-computer").unwrap();
let gb = toolset_for_preset("grok-build").unwrap();
let gb_ids: std::collections::HashSet<&str> =
    gb.tools.iter().map(|t| t.id.as_str()).collect();
assert!(!gc.tools.is_empty());
```

Add `p12_codex_toolset_identity` in the existing `#[cfg(test)] mod tests`. Resolve `codex` through public `toolset_for_preset`, collect IDs, and assert:

- `ToolConfig::from(&codex::ApplyPatchTool).id` is present and remains `Codex:apply_patch`;
- the default `grok-build` preset retains `ToolConfig::from(&grok_build::SearchReplaceTool).id`;
- the test does not rename either preset or require stock Codex CLI tool names.

The default bum composition is at `config.rs:263-285`; test the real preset builder rather than duplicating a string-only fixture.

**Tool implementation facts for documentation, not a production edit** (`apply_patch/tool.rs:97-102,244-289`):

```rust
pub struct ApplyPatchInput {
    pub patch: String,
}

fn tool_namespace(&self) -> ToolNamespace {
    ToolNamespace::Codex
}

fn id(&self) -> ToolId {
    ToolId::new("apply_patch").expect("valid tool id")
}
```

Re-run the existing apply-patch suite. Do not add a second patch tool or alter its wire shape.

---

### `12-PHASE-GATE.md` (batch validation runbook)

**Analog:** `.planning/phases/08-quiet-fork-rebrand-polish/08-PHASE-GATE.md`.

**Discover-before-execute helper** (`08-PHASE-GATE.md:35-49`):

```bash
discover() {
  local pkg="$1" filter="$2"; shift 2
  local extra=("$@")
  local n
  n=$(cargo test -p "$pkg" "${extra[@]}" -- --list 2>/dev/null | grep -c "$filter" || true)
  echo "discover $pkg $filter (${extra[*]:-}): n=$n"
  test "$n" -ge 1
  cargo test -p "$pkg" "${extra[@]}" "$filter" -- --nocapture
}
```

Use this for pager `p12_` and agent `p12_codex_toolset_identity`; a bare filtered test can pass with zero matches. Then run the existing apply-patch and trusted-originator filters explicitly.

**Non-vacuous static identity pattern** (`08-PHASE-GATE.md:136-167`):

```bash
for f in "${RESIDUAL_FILES[@]}"; do
  test -f "$f"
  if rg -n "$STOCK_PAT" "$f"; then
    echo "RESIDUAL FAIL: stock product CLI chrome still present in $f"
    exit 1
  fi
done

rg -n 'bum login|bum plugin|bum mcp|return to bum' "${RESIDUAL_FILES[@]}"
```

Phase 12 should enumerate the exact README, 22 `USER_GUIDE` files, and 2 `REFERENCE_DOCS` files; assert required positive markers and forbidden semantic claims. Record the allowlist alongside the patterns. Do not use a broad repository-wide `rg 'Grok|Codex'` gate.

The gate should also verify:

- both notice files still contain the OpenAI/Codex apply-patch attribution;
- `CODEX_ORIGINATOR` remains `bum` and the trusted-route regression is green;
- no Phase 12 production diff introduces WebSocket transport, stock `originator=codex_cli_rs`, broad tool renames, or new notice-triggering derived code;
- `cargo fmt --all -- --check` passes;
- no live OAuth/network check is required unless runtime/wire behavior actually changed.

Keep required command chains fail-closed (`set -euo pipefail`, no trailing broad `|| true`), following `08-PHASE-GATE.md:27,78-89`.

---

### `12-VALIDATION.md` (validation contract, batch)

**Analog:** the current Phase 12 file already has the correct per-task map and Wave 0 inventory. Update it as tests/gate land, preserving its `p12_` filters and no-live-UAT rule. Set `nyquist_compliant: true` and `wave_0_complete: true` only after the named tests and gate exist and execute non-vacuously.

## Shared Patterns

### Product Identity Is Separate From Provider Attribution

**Sources:** `README.md:23-40`; `08-PHASE-GATE.md:255-265`

Apply to every documentation surface. Product/executable/global-home claims use bum / `bum` / `~/.bum`. Provider/model, historical lineage, legal, internal crate/type/env, hosted-domain, and project-local compatibility references remain when accurate.

### Embedded Content Is Runtime Content

**Source:** `crates/codegen/xai-grok-pager/src/docs.rs:37-44,48-177,208-230`

Every registered Markdown file is compiled into the binary; numbered user guides are additionally extracted under the configured product home. A README-only correction is insufficient.

### Route-Scoped bum Attribution Must Be Preserved

**Sources:** `crates/codegen/xai-grok-shell/src/auth/codex/mod.rs:50-63`; `crates/codegen/xai-grok-shell/src/session/acp_session_tests/codex_reconstruct_refresh_tests.rs:337-387`

```rust
pub const CODEX_ORIGINATOR: &str = "bum";

assert_eq!(header_value(&first.extra_headers, "originator"), Some("bum"));
```

This is a regression dependency, not a planned production edit. The phase gate should re-run it rather than duplicating or refactoring the trusted-route matrix.

### Existing Third-Party Attribution Is the Baseline

**Crate-local source** (`crates/codegen/xai-grok-tools/THIRD_PARTY_NOTICES.md:14-25`):

```markdown
### openai/codex

The tool implementations under `src/implementations/codex/` (`apply_patch`,
`grep_files`, `list_dir`, `read_file`) are ported from the
[openai/codex](https://github.com/openai/codex) project
```

**Root source** (`THIRD-PARTY-NOTICES:16631-16649`) names the upstream repository, handlers/apply-patch paths, OpenAI copyright, and all four modules.

Apply to the phase gate and README links. Normally neither notice file changes. Edit them only if substantial new copied/derived code or constants land.

## Verify-Only / Normally Unchanged Files

| File | Role | Data Flow | Why no planned edit |
|---|---|---|---|
| `crates/codegen/xai-grok-tools/src/implementations/codex/apply_patch/**` | service/tool | file-I/O | Existing implementation and focused suite are green; locked scope says validate, do not replace. |
| `crates/codegen/xai-grok-shell/src/auth/codex/mod.rs` | config | request-response | `CODEX_ORIGINATOR` already equals `bum`. |
| `crates/codegen/xai-grok-shell/src/session/acp_session_impl/sampler_turn.rs` | service | streaming/request-response | Trusted route already inserts bum-owned metadata; no transport change is in scope. |
| `THIRD-PARTY-NOTICES` and `crates/codegen/xai-grok-tools/THIRD_PARTY_NOTICES.md` | config/legal | transform | Existing Codex ancestry is complete; verify statically and avoid notice churn without a trigger. |

## No Analog Found

None. Every planned file has an in-tree analog. The capability matrix itself is new content, but its placement, Markdown table form, embedding, discoverability, and contract-test patterns all exist in the current README/pager/Phase 8 surfaces.

## Metadata

**Analog search scope:** `README.md`, pager embedded docs and tests, agent preset configuration/tests, Codex apply-patch implementation, shell Codex identity tests, Phase 8/9 gates, root and crate-local notices

**Files scanned:** 30 planned surfaces plus focused runtime/legal analogs

**Pattern extraction date:** 2026-07-21
