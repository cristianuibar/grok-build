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
| Preflight run (UTC) | 2026-07-18T04:29:35Z |
| Binary path | `target/debug/bum` (executable) |
| Binary version | `bum 0.1.220-alpha.4 (eecbd91)` |
| Git commit (short) | `eecbd91` |
| Git commit (full) | `eecbd9106d03b1eb5d6941ee070db06ff7257bba` |
| `uat-preflight.sh` secret gates | PASSED (no tracked auth.json / credential basenames; phase-diff clean of secret shapes) |
| Automated residual (this run) | GREEN — `p9_` (1), `p6_dual_login` (3), list both `p7_isolation_*` dirs then aggregate `p7_isolation` (4), `home_isolation` (1) |
| `~/.bum` auth.json | **absent** — dual OAuth not yet present (operator must complete P4–P6) |
| Live OPS marks | **none** — agent does not mark OPS-03..06 PASS (D-15, D-16) |
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
| 1 | Switch or start on GPT-5.6 catalog entry (default `gpt-5.6-sol`) | Model/provider routes Codex path | ⬜ PASS (switch accepted) | Model: GPT-5.6 / Codex path; operator: switch works |
| 2 | **Read** a real file in disposable workspace | Tool succeeds on real backend | ⬜ FAIL | See PRODUCT BLOCKER below — turn dies before tools |
| 3 | **Edit** or **shell** productive change as supported | Daily-driver tools work | ⬜ blocked | Not reached (step 2 fails) |
| 4 | Capability gaps (D-10) | Document honestly if Codex/OpenAI cannot support a Grok-only tool; remaining tools still clear daily-driver bar | ⬜ | Effort levels not supported on Codex (UI/catalog vs provider); separate from 400 below |

**OPS-04 overall (C1-L5):** ⬜ PASS · ☑ PRODUCT BLOCKER · ⬜ AUTH/ACCOUNT BLOCKED · ⬜ PROVIDER OUTAGE/LIMIT

**PRODUCT BLOCKER (operator 2026-07-18, live dual-login UAT) — fix landed in-phase:**
- After switch to Codex/GPT, first assistant turn fails hard (no productive tools).
- Error (redacted): `API error (status 400 Bad Request): {"detail":"System messages are not allowed"}`
- Request URL: `https://chatgpt.com/backend-api/codex/responses`
- Turn failed ~0.8s; retry same failure.
- **Root cause:** system items were serialized into Responses `input[]` as `role: system` with `instructions: null`. Official Codex CLI never does this — base prompt goes in top-level `instructions` (see `.planning/research/CODEX-RESPONSES-WIRE.md`, sourced from `/home/cristian/bum/codex` codex-rs).
- **Fix (Plan 04 Task 3):** `xai-grok-sampling-types` lifts `ConversationItem::System` into `CreateResponse.instructions` and excludes system from `input[]`. Unit tests assert no system role in input.
- **Operator re-verify required:** rebuild binary, re-run OPS-04 steps 2–3; clear blocker to PASS if live tools work.
- **Secondary (effort):** Was “effort not supported” because catalog lacked `supports_reasoning_effort` / menus. **Quick fix 2026-07-18:** GPT-5.6 entries in `default_models.json` now advertise low/medium/high/xhigh (sol default low; terra/luna medium). Re-check after rebuild. Remaining polish (ultra, soft-clamp edge cases) → Phase 11.

---

## OPS-05 — Mid-session switch (same CLI process) (D-06)

Must be **same process** — no restart of the CLI between steps.

| Step | Action | Expected | Pass? | Notes |
|------|--------|----------|-------|-------|
| 1 | Productive xAI turn (read or edit/shell) | Completes on real backend | ⬜ | Model before: |
| 2 | `/model` (or equivalent) → GPT-5.6 without restarting CLI | Switch accepted; no false missing-provider when both logged in | ⬜ | Model after: |
| 3 | Productive Codex turn | Completes on real backend | ⬜ | |
| 4 | (Optional) reverse switch GPT → xAI | Works without restart | ⬜ | |

**OPS-05 overall (C1-L5):** ⬜ PASS · ☑ PRODUCT BLOCKER · ⬜ AUTH/ACCOUNT BLOCKED · ⬜ PROVIDER OUTAGE/LIMIT  
(xAI side works; Codex productive turn after switch fails with same System-messages 400 as OPS-04 — same product defect.)

---

## OPS-06 — Cross-provider spawn both directions (D-07)

**Both directions mandatory.** One direction alone is insufficient. Use NL **or** Task spawn with explicit **model + effort**. Parent model must remain unchanged after child returns.

Structured evidence placeholders (Plan 04 fills; C1-M6 cross-ref):

### Direction A — Grok parent → Codex child

| Field (C1-M6) | Value (operator fills; leave blank until live) |
|---------------|-----------------------------------------------|
| parent_model | |
| child_model | |
| effort | |
| Spawn path (NL / Task) | |
| result_returned | ⬜ yes · ⬜ no (start-without-return is **not** PASS) |
| parent_model_after | (must match parent_model for PASS) |
| error_class | if failed: PRODUCT BLOCKER / AUTH/ACCOUNT BLOCKED / PROVIDER OUTAGE/LIMIT |
| Outcome | ⬜ PASS · ⬜ PRODUCT BLOCKER · ⬜ AUTH/ACCOUNT BLOCKED · ⬜ PROVIDER OUTAGE/LIMIT |

### Direction B — Codex parent → Grok child

| Field (C1-M6) | Value (operator fills; leave blank until live) |
|---------------|-----------------------------------------------|
| parent_model | |
| child_model | |
| effort | |
| Spawn path (NL / Task) | |
| result_returned | ⬜ yes · ⬜ no (start-without-return is **not** PASS) |
| parent_model_after | (must match parent_model for PASS) |
| error_class | if failed: PRODUCT BLOCKER / AUTH/ACCOUNT BLOCKED / PROVIDER OUTAGE/LIMIT |
| Outcome | ⬜ PASS · ⬜ PRODUCT BLOCKER · ⬜ AUTH/ACCOUNT BLOCKED · ⬜ PROVIDER OUTAGE/LIMIT |

| Step | Action | Expected | Pass? | Notes |
|------|--------|----------|-------|-------|
| 1 | Parent Grok → child Codex (NL or Task + model + effort) | Child completes; results return; parent model unchanged | ⬜ | |
| 2 | Parent Codex → child Grok | Same | ⬜ | |

**OPS-06 overall (C1-L5):** ⬜ PASS · ⬜ PRODUCT BLOCKER · ⬜ AUTH/ACCOUNT BLOCKED · ⬜ PROVIDER OUTAGE/LIMIT  
(Requires **both** directions PASS with result_returned=yes and parent_model_after matching parent_model.)

---

## Session notes

- A **single dual-login session** may cover multiple OPS rows when natural (CONTEXT discretion) — still fill **each** row above.
- Prefer TUI for OPS-03..05; OPS-06 may use NL or explicit Task.
- Record blockers with enough detail for in-phase fix (routing, credentials, switch gate, spawn, daily tools) — not feature expansion.

### Operator session 2026-07-18 (partial matrix)

- **xAI / Grok:** works fine (productive path OK).
- **Codex switch:** model switch accepted; **effort levels not supported** (note as capability/UI gap).
- **Codex talk:** every turn → 400 `System messages are not allowed` on `chatgpt.com/backend-api/codex/responses`.
- **OPS-06:** not completed; Direction A (Grok→Codex child) and Direction B (Codex parent) expected blocked until system→`instructions` wire fix.
- **Disposition options discussed:** (A) in-phase Plan 04 Task 3 product fix on Responses conversion, or (B) dedicated follow-on phase for Codex Responses wire compatibility — hybrid Phase 9 GREEN requires A (or equivalent) before OPS-04/05/06 can PASS.

---

## Sign-off (D-09, D-10, D-12; C1-M3, C1-M4)

| Field | Value |
|-------|-------|
| Operator | |
| Date (UTC) | |
| Binary version / commit | Agent prefill: `bum 0.1.220-alpha.4` @ `eecbd91` (operator confirms / updates if rebuilt) |
| Models under test | xAI: `grok-build` (or: ___) / Codex: `gpt-5.6-sol` (or: ___) |
| Capability gaps documented | |
| Secrets committed? | **No** (must remain No) |
| Disposable workspace path | |
| Chrome preflight outcome | ⬜ PASS · ⬜ BLOCKER DECISION |
| OPS-03 (C1-L5) | ⬜ PASS · ⬜ PRODUCT BLOCKER · ⬜ AUTH/ACCOUNT BLOCKED · ⬜ PROVIDER OUTAGE/LIMIT |
| OPS-04 (C1-L5) | ⬜ PASS · ⬜ PRODUCT BLOCKER · ⬜ AUTH/ACCOUNT BLOCKED · ⬜ PROVIDER OUTAGE/LIMIT |
| OPS-05 (C1-L5) | ⬜ PASS · ⬜ PRODUCT BLOCKER · ⬜ AUTH/ACCOUNT BLOCKED · ⬜ PROVIDER OUTAGE/LIMIT |
| OPS-06 both dirs (C1-L5) | ⬜ PASS · ⬜ PRODUCT BLOCKER · ⬜ AUTH/ACCOUNT BLOCKED · ⬜ PROVIDER OUTAGE/LIMIT |
| Hybrid gate note | Fixture/auto residual green does **not** substitute any failed live row (D-16) |

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
