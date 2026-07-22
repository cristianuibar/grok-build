# Session handoff — Phase 9 / Codex path (2026-07-18)

**Repo:** `/home/cristian/bum/grok-build` · product **bum**  
**Session end:** clean wrap; remote pushed with Codex fixes + planning docs  
**Operator:** Cristian

---

## What shipped this session

| Commit (local→main) | Change |
|---------------------|--------|
| `765b9a0` | Responses: system prompts → top-level `instructions`; never `role:system` in `input[]` (Codex 400 fix) |
| `d843601` | Research doc `CODEX-RESPONSES-WIRE.md`; ROADMAP phases **10–12** |
| `6b722ee` | STATE after system-instructions fix |
| `638e8e8` | GPT-5.6 Sol/Terra/Luna: `supports_reasoning_effort` + low/medium/high/xhigh menus (Codex catalog defaults) |
| (wrap) | UAT notes, STATE, this handoff |

**Research SoT:** `.planning/research/CODEX-RESPONSES-WIRE.md` (from `/home/cristian/bum/codex` codex-rs).

---

## Live UAT status (honest)

| OPS | Status | Evidence |
|-----|--------|----------|
| **OPS-03** xAI | **PASS** | Operator: Grok productive session OK |
| **OPS-04** Codex | **Not PASS** | Switch + effort OK; “hi” **shows reply** but **TUI retries** after (empty/terminal classification suspected) |
| **OPS-05** mid-session | **Partial** | Codex↔Codex OK; **Grok→Codex** after Grok history → 400 encrypted content verify/decrypt |
| **OPS-06** both spawn dirs | **Not run** | Blocked until 04/05 solid |

Full notes: `09-UAT.md` § Operator session + session notes.

**Do not** claim hybrid Phase 9 GREEN or mark Plan 04/05 complete until OPS-04..06 PASS live (D-16).

---

## Learnings (carry into Phase 10+)

### Wire (codex-rs)

1. Base system → **`instructions`**, not `input` system role.  
2. ChatGPT path: `POST https://chatgpt.com/backend-api/codex/responses`, `store: false`, full history each HTTP turn.  
3. GPT-5.6 **does** support effort via catalog menus (low…xhigh; ultra/max in official CLI).  
4. Encrypted reasoning is **provider/session-scoped** — replaying foreign `encrypted_content` after model/provider switch fails decrypt.  
5. Official detector-style message may say **“encrypted content”** (space); our `is_encrypted_content_error` only matches `encrypted_content` (underscore) — may mis-retry.

### Product bugs to fix next (P0)

1. **Post-success retry storm** on Codex turns (user sees text; turn keeps retrying).  
   - Suspect: `ConversationResponse::empty_reason` / `ReasoningOnly` resampling, or missing/mismatched terminal stream event.  
   - Code: `xai-grok-sampling-types` empty_reason; `xai-grok-sampler` `drive_l2` / Empty retry path.

2. **Grok→Codex history:** strip or omit incompatible `ConversationItem::Reasoning` with `encrypted_content` on **provider** switch (or never send non-Codex encrypted blobs to Codex).  
   - Model switch today does not rewrite history (`model_switch.rs`).  
   - Official Codex never mixes foreign encrypted reasoning into `input[]`.

### P2 / later

- Codex thinking UI (summary often none; encrypted not shown).  
- Soft-clamp effort on switch; optional ultra UI.  
- Residual `Usage: grok` chrome (operator judge PASS vs BLOCKER).  
- Phases 11–12 as planned in ROADMAP.

---

## Next session (recommended)

```text
Repo: /home/cristian/bum/grok-build
Start Phase 10: Codex Responses wire parity

Read first:
- .planning/phases/09-daily-driver-end-to-end-validation/09-SESSION-HANDOFF.md
- .planning/research/CODEX-RESPONSES-WIRE.md
- .planning/ROADMAP.md (Phase 10–12)
- .planning/phases/09-daily-driver-end-to-end-validation/09-UAT.md

Goals:
1. Plan+execute Phase 10: (a) no retry after successful Codex text; (b) strip foreign encrypted reasoning on provider switch; (c) fix encrypted-error string match
2. Rebuild ./target/debug/bum; re-run live OPS-04, OPS-05 (Grok→Codex), then OPS-06
3. Resume Phase 9 Plan 04 finalize + Plan 05 hybrid only after OPS-03..06 PASS

Commands: /gsd-plan-phase 10   or   /gsd-discuss-phase 10
```

---

## Commands for operator re-test (after Phase 10)

```bash
cd /tmp && DISPOSABLE=$(mktemp -d /tmp/bum-uat-ws-XXXXXX)
git -C /home/cristian/bum/grok-build worktree add "$DISPOSABLE" HEAD
cd "$DISPOSABLE"
/home/cristian/bum/grok-build/target/debug/bum
# OPS-04: /model gpt-5.6-sol → hi (must complete once) → read/edit file
# OPS-05: grok → hi → /model gpt-5.6-sol → hi (must not encrypted 400)
# OPS-06: both spawn directions
```

Never paste tokens into git or chat.

---

*Handoff written 2026-07-18 session wrap.*
