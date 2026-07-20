# Phase 9 — Live Dual-Login UAT Checklist

**Status:** **REQUIRED gate** (not advisory). Plan 04 fills rows; Plan 05 promotes signed PASS into `09-VERIFICATION.md`.

**Operator:** Cristian (default per `09-CONTEXT.md` D-15)  
**Environment:** real dual OAuth under `~/.bum` **or** isolated temp `BUM_HOME` (D-04, D-15)  
**Secrets policy (D-12):** never paste tokens, refresh tokens, Authorization headers, or full `auth.json` into notes, commits, or this file. Redacted status only (`usable` / provider keys present).  
**Gate rule (D-16):** all OPS-03..06 rows must PASS with **live** evidence — fixture green, mock headless, or automated residual alone **does not count**.  
**Locked decisions honored:** D-05 (productive tools), D-06 (same-process switch), D-07 (both spawn dirs), D-08 (default models), D-09/D-10 (sign-off + gaps), D-12 (secrets), D-15 (human runs live), D-16 (no fixture waive).

**Preflight helper (non-secret):**  
`.planning/phases/09-daily-driver-end-to-end-validation/scripts/uat-preflight.sh`

### Agent preflight notes (Plan 04 Task 1 — not live PASS)

| Field | Value (agent-filled; redacted env only) |
|-------|----------------------------------------|
| Preflight run (UTC) | 2026-07-18T04:29:35Z (initial) · **refreshed 2026-07-18** at Phase 9 execute resume |
| Binary path | `target/debug/bum` (executable) |
| Binary version | `bum 0.1.220-alpha.4 (9a5c2ee)` |
| Git commit (short) | `2880636` (HEAD; binary reports `9a5c2ee`) |
| Git commit (full) | `2880636a373f7889d25f5a4cf99e53a223a45fff` |
| `uat-preflight.sh` secret gates | PASSED (no tracked auth.json / credential basenames; phase-diff clean of secret shapes) |
| Automated residual (initial Task 1) | GREEN — `p9_` (1), `p6_dual_login` (3), list both `p7_isolation_*` dirs then aggregate `p7_isolation` (4), `home_isolation` (1) |
| `~/.bum` auth.json | **present** — `auth status`: xAI usable + Codex usable (redacted; no tokens) |
| Phase 10 live evidence (cross-ref only) | OPS-04 + OPS-05 operator **PASS** recorded in `10-VALIDATION.md` §Live OPS evidence (2026-07-18). **Not auto-copied into OPS rows below** — operator confirms/promotes into this checklist (D-15). OPS-06 still open. |
| Live OPS marks | Agent does not invent PASS (D-15, D-16). OPS-03 filled PASS earlier; OPS-04/05 still need operator promote/reconfirm; OPS-06 not run. |
| Chrome sample (informational only) | `--help` head shows product line `bum TUI` but also residual `Usage: grok` / `~/.grok` example chrome — **operator judges** C1–C3 PASS vs BLOCKER DECISION (C1-M4); agent does not auto-pass |

**Outcome taxonomy (C1-L5 — HARD for every OPS overall row):** use exactly one of  
`PASS` · `PRODUCT BLOCKER` · `AUTH/ACCOUNT BLOCKED` · `PROVIDER OUTAGE/LIMIT`  
Only **PASS** feeds Plan 05 hybrid GREEN. Do not collapse outage/auth into PASS or into PRODUCT BLOCKER without evidence. Fixture/auto residual green never substitutes live PASS (D-16).

---

## Disposable workspace (C1-M3 — required)

Live **edit/shell** steps must target a **disposable** git worktree or throwaway fixture directory/file.  
**Do not** treat the primary `bum` / `grok-build` development checkout as the only edit target without an explicit disposable path.

| Field | Value (operator fills) |
|-------|------------------------|
| Disposable workspace path | |
| How created (worktree / temp dir / fixture file) | |
| Initial `git status --short` (or fixture baseline) | |
| Final `git status --short` after matrix | |
| Cleanup verified (worktree removed / fixture deleted)? | ⬜ |

**Suggested setup (operator chooses one):**

```bash
# Option A — disposable git worktree under a temp parent
DISPOSABLE="$(mktemp -d /tmp/bum-uat-ws-XXXXXX)"
git worktree add "$DISPOSABLE" HEAD
cd "$DISPOSABLE"
git status --short   # record baseline (expect clean or known)

# Option B — throwaway fixture file only (minimal)
DISPOSABLE="$(mktemp -d /tmp/bum-uat-fixture-XXXXXX)"
echo "uat baseline $(date -u +%Y-%m-%dT%H:%M:%SZ)" > "$DISPOSABLE/UAT_FIXTURE.txt"
cd "$DISPOSABLE"
# record baseline: ls -la + content hash or cat of fixture
```

After the full OPS matrix, re-run `git status --short` (or re-list fixture) and record cleanup.

---

## Preflight — binary and auth

Run before any OPS row. Prefer `scripts/uat-preflight.sh` to print steps + run secret gates; login remains operator-only.

| Step | Action | Expected | Pass? | Notes |
|------|--------|----------|-------|-------|
| P1 | `cargo build -p xai-grok-pager-bin` (or use already-built binary) | Binary at `target/debug/bum` (or installed `bum`) | ⬜ | Agent: binary ready at `target/debug/bum` — commit `eecbd91` / `bum 0.1.220-alpha.4` (operator still checks Pass?) |
| P2 | Network + xAI account + ChatGPT/Codex account usable | IdP + API reachable | ⬜ | |
| P3 | Optional isolation: `export BUM_HOME="$(mktemp -d /tmp/bum-uat-home-XXXXXX)"` **or** use real `~/.bum` | Credentials only under product home | ⬜ | Path: |
| P4 | `./target/debug/bum login` (xAI default) | Login succeeds; slot usable | ⬜ | |
| P5 | `./target/debug/bum login --provider codex` | Login succeeds; Codex slot usable | ⬜ | |
| P6 | `./target/debug/bum auth status` | Both providers **usable** — **no token dump** in terminal notes or this file | ⬜ | Redacted only |
| P7 | Record binary version / git commit | Traceable evidence for sign-off (D-09) | ⬜ | Agent prefill: `bum 0.1.220-alpha.4` @ `eecbd91` (full `eecbd9106d03b1eb5d6941ee070db06ff7257bba`) |

---

## Preflight — CLI chrome (C1-M4 — required)

Product must present as **bum** for daily-driver identity. Residual stock Grok product chrome that would confuse identity is an explicit **blocker decision** for Plan 04 Task 3 (in-phase chrome fix per D-03) — do not silently ignore.

| Step | Action | Expected | Outcome | Notes |
|------|--------|----------|---------|-------|
| C1 | `./target/debug/bum --help` (or `target/debug/bum --help`) | Primary product name / about presents **bum** (clap `name = "bum"`) | ⬜ PASS / ⬜ BLOCKER DECISION | |
| C2 | Sample login / auth status chrome | Labels usable for dual login (`login`, `--provider codex`, `auth status`); not ~/.grok-only guidance that would misroute daily use | ⬜ PASS / ⬜ BLOCKER DECISION | |
| C3 | Note `/model` (or switch) labels in TUI/help | Switch path discoverable without stock-only Grok product framing | ⬜ PASS / ⬜ BLOCKER DECISION | |

**Decision rule:** residual product-visible Grok chrome that breaks daily-driver identity → mark **BLOCKER DECISION**, fix in-phase (Plan 04), re-run this table. Cosmetic internal strings that never surface to the operator may be noted without blocking.

**Chrome preflight outcome (summary):** ⬜ PASS · ⬜ BLOCKER DECISION (describe: ________)

---

## Preflight — secrets (C1-L4, C2-L1, C3-L1 — required)

Before commit and after UAT notes:

| Check | Rule | Pass? |
|-------|------|-------|
| S1 | No tracked/staged `auth.json` under phase dir or repo | ⬜ |
| S2 | No opaque credential dumps in UAT notes (no JWT paste, no private keys, no full auth document) | ⬜ |
| S3 | Operator will scan **phase diff** before commit (not only JWT/private-key regex mental check) | ⬜ |
| S4 | Path guard (scoped basenames only — **not** bare `*token*`): no tracked `auth.json`, `credentials.json`, `.oauth`, `*.pem` / `*.p12` / `*.pfx`, `id_rsa` / `id_ed25519` / `id_ecdsa` | ⬜ |

```bash
# Scoped credential/artifact path guard (C3-L1) — fail if any hit
git ls-files | rg -qi '(^|/)(auth\.json|credentials\.json|\.oauth|[^/]+\.pem|[^/]+\.p12|[^/]+\.pfx|id_rsa|id_ed25519|id_ecdsa)($|/)' \
  && echo "FAIL: credential path in tree" || echo "OK: no credential basenames tracked"

# Phase-diff content scan (C2-L1) — fail if secret-shaped content in phase working tree/index
P=.planning/phases/09-daily-driver-end-to-end-validation
{ git diff HEAD -- "$P"; git diff --cached -- "$P"; } | \
  rg -n 'eyJ[A-Za-z0-9_-]{20,}\.[A-Za-z0-9_-]{10,}|BEGIN (RSA |OPENSSH )?PRIVATE|"access_token"[[:space:]]*:[[:space:]]*"[^"]{20,}"|"refresh_token"[[:space:]]*:[[:space:]]*"[^"]{20,}"' \
  && echo "FAIL: secret-shaped content in phase diff" || echo "OK: phase diff clean of secret shapes"
```

`uat-preflight.sh` runs these gates fail-closed. Re-run before any commit of UAT evidence.

---

## OPS-03 — xAI productive session (D-05, D-08)

**Default model:** `grok-build` (record actual if different). Prefer TUI daily path; headless `-p`/`-m` allowed if TUI blocked.

**Workspace:** all read/edit/shell against the **disposable** path from C1-M3.

| Step | Action | Expected | Pass? | Notes (model id, when, path) |
|------|--------|----------|-------|------------------------------|
| 1 | Start TUI on current xAI model (prefer `./target/debug/bum` from disposable cwd) | Session starts; product presents as bum | ☑ PASS | Model: grok-build (xAI); operator 2026-07-18 |
| 2 | Ask agent to **read** a real file in disposable workspace | Tool succeeds on **real** backend (not fixture mock) | ☑ PASS | Live backend OK |
| 3 | **Edit** or **shell** one productive change in disposable workspace | Succeeds on real backend | ☑ PASS | Live productive turn OK |

**OPS-03 overall (C1-L5):** ☑ PASS · ⬜ PRODUCT BLOCKER · ⬜ AUTH/ACCOUNT BLOCKED · ⬜ PROVIDER OUTAGE/LIMIT  
Operator confirmed 2026-07-18: Grok/xAI works fine for productive session.

---

## OPS-04 — Codex / GPT-5.6 productive session (D-05, D-08, D-10)

**Default model:** `gpt-5.6-sol` (record actual GPT-5.6 catalog entry if different). Prefer TUI.

| Step | Action | Expected | Pass? | Notes |
|------|--------|----------|-------|-------|
| 1 | Switch or start on GPT-5.6 catalog entry (default `gpt-5.6-sol`) | Model/provider routes Codex path | ☑ PASS | GPT-5.6 / Codex path; operator live 2026-07-18 (Phase 10 re-verify) |
| 2 | **Read** a real file in disposable workspace | Tool succeeds on real backend | ☑ PASS | Daily-driver tools usable after wire fixes |
| 3 | **Edit** or **shell** productive change as supported | Daily-driver tools work | ☑ PASS | Productive turn + tool path OK |
| 4 | Capability gaps (D-10) | Document honestly if Codex/OpenAI cannot support a Grok-only tool; remaining tools still clear daily-driver bar | ☑ noted | Effort menus present (low…xhigh); ultra/soft-clamp polish → Phase 11. Codex thinking UI often none (expected) |

**OPS-04 overall (C1-L5):** ☑ PASS · ⬜ PRODUCT BLOCKER · ⬜ AUTH/ACCOUNT BLOCKED · ⬜ PROVIDER OUTAGE/LIMIT

**Evidence source (fast path — promote Phase 10):** Operator live PASS recorded in `.planning/phases/10-codex-responses-wire-parity/10-VALIDATION.md` §Live OPS evidence (Plan 10-05 Task 2), date 2026-07-18. Criteria met: GPT-5.6 turn completes once (**no post-success retry**), read/edit/tool usable, dual-login.

**Historical blockers (resolved — kept for audit, not current status):**
- System-messages 400 → fixed (system → top-level `instructions`); no longer active.
- Post-success retry storm → fixed Phase 10 (delta-only terminal / empty retry).
- Effort catalog gaps → quick catalog fix 2026-07-18; residual ultra polish → Phase 11.

---

## OPS-05 — Mid-session switch (same CLI process) (D-06)

Must be **same process** — no restart of the CLI between steps.

| Step | Action | Expected | Pass? | Notes |
|------|--------|----------|-------|-------|
| 1 | Productive xAI turn (read or edit/shell) | Completes on real backend | ☑ PASS | Model before: Grok / xAI |
| 2 | `/model` (or equivalent) → GPT-5.6 without restarting CLI | Switch accepted; no false missing-provider when both logged in | ☑ PASS | Model after: **gpt-5.6-luna** (Codex) |
| 3 | Productive Codex turn | Completes on real backend | ☑ PASS | No encrypted-content 400; no store-false item-id 404 after fix |
| 4 | (Optional) reverse switch GPT → xAI | Works without restart | ⬜ optional | Not required for overall PASS |

**OPS-05 overall (C1-L5):** ☑ PASS · ⬜ PRODUCT BLOCKER · ⬜ AUTH/ACCOUNT BLOCKED · ⬜ PROVIDER OUTAGE/LIMIT

**Evidence source (fast path — promote Phase 10):** Operator retest PASS in `10-VALIDATION.md` §OPS-05 (2026-07-18): Grok → **gpt-5.6-luna**; ordinary context usable; earlier `rs_*` store-false 404 fixed via `strip_input_item_ids_for_store_false`. Encrypted-content 400 on provider switch fixed Phase 10 history sanitizer.

**Historical blockers (resolved):** system-messages 400 (same as OPS-04); encrypted decrypt 400 on Grok→Codex; store-false id 404.
---

## OPS-06 — Cross-provider spawn both directions (D-07)

**Both directions mandatory.** One direction alone is insufficient. Use NL **or** Task spawn with explicit **model + effort**. Parent model must remain unchanged after child returns.

Structured evidence placeholders (Plan 04 fills; C1-M6 cross-ref):

### Direction A — Grok parent → Codex child

**Run 1 (2026-07-20, operator live — FAILED):**

| Field (C1-M6) | Value (operator fills; leave blank until live) |
|---------------|-----------------------------------------------|
| parent_model | Grok 4.5 (high effort) — xAI |
| child_model | gpt-5.6-sol (Codex) |
| effort | low |
| Spawn path (NL / Task) | NL → Task tool (4 attempts: 1 foreground, 3 background) |
| result_returned | ☑ no (start-without-return is **not** PASS) |
| parent_model_after | Grok 4.5 (unchanged — parent stability OK) |
| error_class | PRODUCT BLOCKER — foreground: serialization error `unknown variant response.metadata` on Codex SSE decode in child path; background: child hangs at turn 1 with 0 tool calls (decode error swallowed), operator killed after ~5–13 min |
| Outcome | ⬜ PASS · ☑ PRODUCT BLOCKER · ⬜ AUTH/ACCOUNT BLOCKED · ⬜ PROVIDER OUTAGE/LIMIT |

In-phase fix required (spawn path — allowed scope per Session notes). Re-run table after fix.

**Run 2 (2026-07-20, operator live, after in-phase fixes — PASS):**

| Field (C1-M6) | Value |
|---------------|-------|
| parent_model | grok-4.5 (xAI) — TUI session |
| child_model | gpt-5.6-sol (Codex) |
| effort | low |
| Spawn path (NL / Task) | NL → Task tool |
| result_returned | ☑ yes — child returned exact fixture contents (`uat baseline 2026-07-20T09:45:51Z`) |
| parent_model_after | grok-4.5 (confirmed via /model — unchanged) |
| error_class | — |
| Outcome | ☑ PASS |

In-phase fixes that unblocked Run 2 (committed this session; root causes were product blockers, not provider outage):
1. Tolerant skip of unknown Responses SSE frame types (new backend `response.metadata` event killed child streams) — `xai-grok-sampler/src/client.rs`.
2. Subagent child sessions hard-coded `ResponsesWireProfile::Disabled`; now derived from the child's resolved model + auth (TrustedCodex for Codex-OAuth children) — `xai-grok-shell/src/agent/{config,subagent/mod}.rs`.
3. TrustedCodex profile now strips `temperature` / `top_p` / `max_output_tokens` (ChatGPT backend 400s "Unsupported parameter") and adds `strict` on function tools + `prompt_cache_key` (official Codex CLI parity) — `xai-grok-sampler/src/client.rs`.
4. Backend now sends terminal `response.completed` with empty `output` after streaming tool-call items; final response is reconstructed from accumulated `response.output_item.done` items (fixes `empty_response (no_visible_content)` retry storm on all tool-using Codex turns, main sessions included) — `xai-grok-sampler/src/stream/responses.rs`.

### Direction B — Codex parent → Grok child

**Run (2026-07-20, operator live — PASS):**

| Field (C1-M6) | Value |
|---------------|-------|
| parent_model | gpt-5.6-sol (Codex) — same TUI session, switched via /model |
| child_model | grok-4.5 (xAI) — **requested `grok-build`, effective child ran grok-4.5** (see deferred note) |
| effort | default |
| Spawn path (NL / Task) | NL → Task tool |
| result_returned | ☑ yes — child returned exact fixture contents (`uat baseline 2026-07-20T09:45:51Z`) |
| parent_model_after | gpt-5.6-sol (confirmed via /model — unchanged) |
| error_class | — |
| Outcome | ☑ PASS (cross-provider Codex parent → xAI child satisfied; slug substitution deferred) |

Deferred (non-blocking, recorded): with a Codex parent, the Task allow-list rejects `grok-build` ("not an available model slug; supported Grok model is grok-4.5") in both TUI and headless — parent asked operator, operator approved `grok-4.5`. Root cause: `visible_for_auth` evaluates xAI models against the parent session's Codex auth mode (`is_session_auth` is session-global, not per-provider), hiding session-auth-only xAI slugs like `grok-build`. Fix candidate: derive Task-model visibility from each model's own provider auth slot. → Phase 11/12 backlog.

| Step | Action | Expected | Pass? | Notes |
|------|--------|----------|-------|-------|
| 1 | Parent Grok → child Codex (NL or Task + model + effort) | Child completes; results return; parent model unchanged | ☑ PASS | Run 2 above, 2026-07-20 |
| 2 | Parent Codex → child Grok | Same | ☑ PASS | 2026-07-20 |

**OPS-06 overall (C1-L5):** ☑ PASS · ⬜ PRODUCT BLOCKER · ⬜ AUTH/ACCOUNT BLOCKED · ⬜ PROVIDER OUTAGE/LIMIT  
(Both directions PASS with result_returned=yes and parent_model_after matching parent_model — operator live, 2026-07-20.)

---

## Session notes

- A **single dual-login session** may cover multiple OPS rows when natural (CONTEXT discretion) — still fill **each** row above.
- Prefer TUI for OPS-03..05; OPS-06 may use NL or explicit Task.
- Record blockers with enough detail for in-phase fix (routing, credentials, switch gate, spawn, daily tools) — not feature expansion.

### Operator session 2026-07-18 (partial matrix — historical)

- Early session: OPS-03 PASS; OPS-04/05 product blockers (system messages, retry storm, encrypted history, store-false ids).
- Phase 10 wire parity fixed those; operator re-verified **OPS-04 PASS** and **OPS-05 PASS** (see `10-VALIDATION.md`).

### Fast path resume (Phase 9 execute — 2026-07-18)

- **OPS-03:** PASS (this file).
- **OPS-04 / OPS-05:** Promoted to PASS from Phase 10 live operator evidence (cross-ref above) per operator request “fast path.”
- **OPS-06:** **still operator-only** — both spawn directions not yet run. Do not mark overall hybrid GREEN until OPS-06 PASS + sign-off.
- **Thinking UI / effort ultra:** capability notes only; not OPS gate failures.
- Dual OAuth under `~/.bum`: both slots usable at resume (redacted).

---

## Sign-off (D-09, D-10, D-12; C1-M3, C1-M4)

| Field | Value |
|-------|-------|
| Operator | Cristian |
| Date (UTC) | 2026-07-18 (OPS-03..05); OPS-06: **2026-07-20** |
| Binary version / commit | OPS-06: `bum 0.1.220-alpha.4 (5b68602)` + uncommitted in-phase fixes (committed immediately after this sign-off; see Run 2 fix list) |
| Models under test | xAI: `grok-build` / `grok-4.5` / Codex: GPT-5.6 (incl. **gpt-5.6-luna** on OPS-05, **gpt-5.6-sol** on OPS-06 both dirs) |
| Capability gaps documented | Codex thinking often none; effort ultra polish → Phase 11 |
| Secrets committed? | **No** (must remain No) |
| Disposable workspace path | `/tmp/bum-uat-fixture-Fu3LE3` (throwaway fixture file, Option B; cleanup after session close) |
| Chrome preflight outcome | ☑ PASS (cosmetic) — residual `Usage: grok` / `~/.grok` in `--help` accepted as non-blocking; product line is `bum TUI`; full residual-string rebrand sweep is the immediately following task |
| OPS-03 (C1-L5) | ☑ PASS · ⬜ PRODUCT BLOCKER · ⬜ AUTH/ACCOUNT BLOCKED · ⬜ PROVIDER OUTAGE/LIMIT |
| OPS-04 (C1-L5) | ☑ PASS · ⬜ PRODUCT BLOCKER · ⬜ AUTH/ACCOUNT BLOCKED · ⬜ PROVIDER OUTAGE/LIMIT |
| OPS-05 (C1-L5) | ☑ PASS · ⬜ PRODUCT BLOCKER · ⬜ AUTH/ACCOUNT BLOCKED · ⬜ PROVIDER OUTAGE/LIMIT |
| OPS-06 both dirs (C1-L5) | ☑ PASS · ⬜ PRODUCT BLOCKER · ⬜ AUTH/ACCOUNT BLOCKED · ⬜ PROVIDER OUTAGE/LIMIT |
| Hybrid gate note | All OPS-03..06 rows PASS with live operator evidence (D-16 satisfied). |
Signed live PASS rows become evidence for `09-VERIFICATION.md` (Plan 05).

---

## Anti-patterns (do not do)

| Anti-pattern | Why forbidden |
|--------------|---------------|
| Environment skip / “could not run” treated as pass | D-16 — blocks human path instead |
| Only one OPS-06 spawn direction | D-07 — both mandatory |
| Pasting tokens, JWT, private keys, full `auth.json` | D-12 |
| Mock / fixture / headless mock as live OPS pass | D-16 / hybrid bar |
| Live edits only in primary checkout without disposable path | C1-M3 |
| Ignoring residual product-visible Grok chrome without blocker decision | C1-M4 |
| Bare `*token*` path bans that flag source like `token.rs` | C3-L1 — use scoped basenames only |
| Agent auto-marking UAT PASS without operator | D-15 |
| Committing secret-shaped content in phase diffs | C2-L1 |

---

## Quick command reference

```bash
export BUM_HOME="${BUM_HOME:-$HOME/.bum}"   # or mktemp -d isolation
cargo build -p xai-grok-pager-bin
./target/debug/bum --help                   # chrome preflight
./target/debug/bum login                    # xAI default
./target/debug/bum login --provider codex
./target/debug/bum auth status              # both usable; no token dump
# Prefer TUI from disposable workspace:
# cd "$DISPOSABLE" && /path/to/repo/target/debug/bum
# Optional headless single turn (fallback only):
# ./target/debug/bum -p "read UAT_FIXTURE.txt and summarize" -m grok-build --always-approve
```
