# Codex / ChatGPT Responses wire contract (from official CLI)

**Status:** Living reference for bum Codex integration  
**Source tree:** `/home/cristian/bum/codex` (`codex-rs`, researched 2026-07-18)  
**Why:** Live UAT hit `400 {"detail":"System messages are not allowed"}` on  
`https://chatgpt.com/backend-api/codex/responses` when bum put `role: system` in `input[]`.  
**Audience:** Implementers of bum’s Codex path (sampler conversion, auth, effort, tools).

---

## 1. Hard rules (do / don’t)

| Do | Don’t |
|----|--------|
| Put the **base agent/system prompt** in top-level **`instructions`** | Put `role: "system"` (or Chat Completions system) in **`input[]`** |
| Use **`user` / `assistant` / `developer`** roles in `input` | Assume Chat Completions message roles map 1:1 |
| Content parts: `{"type":"input_text","text":"..."}` (or typed images) | Bare string `content` without type tags where the API expects parts |
| ChatGPT OAuth → `POST https://chatgpt.com/backend-api/codex/responses` | Route Codex OAuth through xAI cli-chat-proxy |
| `store: false` on ChatGPT path; resend full `input` each HTTP turn | Rely on HTTP `previous_response_id` alone (WS-only incremental) |
| Soft-clamp unsupported effort; send supported levels for GPT-5.6 | Hard-fail the turn with “effort not supported” for catalog models that list levels |
| `function_call.arguments` as a **JSON string** | Nested JSON object for arguments |

---

## 2. Endpoints

| Auth | Default base | Responses URL |
|------|--------------|---------------|
| ChatGPT OAuth / PAT / agent identity | `https://chatgpt.com/backend-api/codex` | `POST …/codex/responses` |
| API key | `https://api.openai.com/v1` | `POST …/v1/responses` |

**Codex sources:**  
`codex-rs/model-provider-info/src/lib.rs` (`CHATGPT_CODEX_BASE_URL`),  
`codex-rs/codex-api/src/endpoint/responses.rs` (path `"responses"`).

Also used by Codex (not required for bum MVP):  
`/responses/compact`, `/memories/trace_summarize`, `/realtime/calls`, Responses-over-WebSocket (`wss://…/codex/responses` + `OpenAI-Beta: responses_websockets=2026-02-06`).

---

## 3. Auth & headers (ChatGPT path)

| Header | Required | Value |
|--------|----------|--------|
| `Authorization` | yes | `Bearer <chatgpt_access_token>` |
| `ChatGPT-Account-ID` | when known | account / workspace id |
| `Content-Type` | yes | `application/json` |
| `Accept` | streaming | `text/event-stream` |
| `originator` | Codex sends | e.g. `codex_cli_rs` (bum may use its own product originator) |
| `session-id` / `thread-id` | Codex sends | UUIDs for session stickiness |
| `x-client-request-id` | Codex HTTP | often = thread id |
| `OpenAI-Beta` | **WS only** | `responses_websockets=2026-02-06` |

**Sources:** `model-provider/src/bearer_auth_provider.rs`, `login/src/auth/default_client.rs`, `core/src/client.rs`.

---

## 4. Request body shape

Authoritative DTO: `ResponsesApiRequest` in `codex-rs/codex-api/src/common.rs`.

```json
{
  "model": "gpt-5.6-sol",
  "instructions": "<BASE SYSTEM PROMPT ONLY — not AGENTS.md>",
  "input": [
    {
      "type": "message",
      "role": "developer",
      "content": [{ "type": "input_text", "text": "<policy / tools context>" }]
    },
    {
      "type": "message",
      "role": "user",
      "content": [{ "type": "input_text", "text": "user turn" }]
    }
  ],
  "tools": [ { "type": "function", "name": "…", "description": "…", "strict": true, "parameters": { } } ],
  "tool_choice": "auto",
  "parallel_tool_calls": true,
  "reasoning": { "effort": "low" },
  "store": false,
  "stream": true,
  "include": ["reasoning.encrypted_content"],
  "prompt_cache_key": "<session-id>"
}
```

### 4.1 `instructions` vs `input`

| Content | Field | Role in `input` |
|---------|-------|-----------------|
| Model / agent **base** system prompt | **`instructions`** | — |
| Permissions, skills, config developer text | `input` | **`developer`** |
| AGENTS.md / environment context | `input` | **`user`** (Codex style) |
| User turns | `input` | **`user`** |
| Assistant history | `input` | **`assistant`** (`output_text` parts) |
| Tool calls / outputs | `input` | function_call / function_call_output / custom_* |

Official builder: `codex-rs/core/src/client.rs` → `build_responses_request`.  
There is **no** client filter that strips system roles — Codex **never emits** them on the live path.  
Server error `"System messages are not allowed"` is ChatGPT Codex validation.

### 4.2 Continuity

- ChatGPT/OpenAI: **`store: false`** → client resends full history every HTTP turn.
- **`previous_response_id`**: used on **WebSocket** incremental turns when prefix matches; not the primary HTTP contract.
- **`prompt_cache_key`**: session id for cache stickiness.

---

## 5. Reasoning effort

| Topic | Official Codex behavior |
|-------|-------------------------|
| Wire field | `reasoning.effort` (optional) |
| Values | `none`, `minimal`, `low`, `medium`, `high`, `xhigh`, `max`; UI **ultra → wire max** |
| GPT-5.6 catalog | **Does support effort** via `supported_reasoning_levels` (sol: default **low**; terra/luna: often **medium**) |
| Unsupported handling | **Soft-clamp** to supported list / default — **never** hard-error the turn |
| `reasoning.summary` | Only if model supports it **and** summary ≠ `none`; GPT-5.6 defaults often **omit** |
| `include` | Always `["reasoning.encrypted_content"]` in Codex |

**bum implication:** “Effort levels are not supported” for GPT-5.6 on Codex path is a **catalog/UI capability bug**, not an API prohibition. Prefer non-empty `supported_reasoning_levels` (or equivalent) for GPT-5.6 entries; soft-skip only when the list is empty.

**Sources:** `protocol/src/openai_models.rs`, `core/src/client.rs`, `core/src/session/turn_context.rs`, `models-manager/models.json`.

---

## 6. Tools (daily-driver names in stock Codex)

| Tool | Wire kind | Name |
|------|-----------|------|
| Shell (classic) | `function` | `shell_command` |
| Unified exec | `function` | `exec_command` + `write_stdin` |
| Patch | **`custom` (freeform)** | `apply_patch` (raw text, not JSON args) |

bum uses its own tool surface (Grok-build / ports). Do **not** rename bum tools to match Codex unless adopting Codex tool parity as a product goal (future phase). Wire **shape** still must be Responses-style `type: function` with top-level `name` / `parameters` / `strict`.

---

## 7. bum mapping (current + target)

### Bug fixed (Phase 9 / Plan 04 Task 3)

| Before | After |
|--------|--------|
| `ConversationItem::System` → `input[]` `role: system` | System text → `CreateResponse.instructions` |
| `instructions: null` | Populated from all system items (joined with `\n\n`) |
| ChatGPT Codex 400 | Aligned with official CLI |

**Code:** `crates/codegen/xai-grok-sampling-types/src/conversation.rs`  
`From<&ConversationRequest> for rs::CreateResponse`, `extract_responses_instructions`, `build_responses_input`.

Chat Completions conversion is unchanged (system role still valid there).

### Effort catalog (quick fix landed 2026-07-18)

Embedded `default_models.json` now sets for `gpt-5.6-{sol,terra,luna}`:

- `supports_reasoning_effort: true`
- Per-model `reasoning_efforts` menu: **low / medium / high / xhigh** (Codex official ladder; bum maps `max`→`xhigh` on parse)
- Defaults: **sol=low**, **terra/luna=medium** (matches codex-rs `models.json`)

This enables the TUI/session effort picker after switch to Codex. Deeper work (ultra UI, summary defaults, mid-switch soft-clamp polish) remains Phase 11.

### Still open (recommended follow-on phases)

See ROADMAP Phases **10–12** (Codex wire depth, remaining effort/catalog polish, tool/attribution parity).

---

## 8. Minimal reproduction of the UAT failure

1. Dual login under `~/.bum` (xAI + Codex).  
2. Switch model to GPT-5.6.  
3. Send any user turn while conversation history includes a system item.  
4. Pre-fix client: body has `input[0].role = "system"` → **400**.  
5. Post-fix: `instructions` holds system text; `input` has only user/assistant/tool items.

---

## 9. Attribution / license note

Official Codex CLI is a separate product under its own license (`/home/cristian/bum/codex/LICENSE`).  
This document describes **wire behavior observed in source** for interoperability.  
It is **not** a copy of Codex prompts, trademarks as product chrome, or a claim that bum is Codex.  
Product naming remains **bum**; model brands (GPT-5.6, etc.) stay as provider identity.  
Do not ship `originator: codex_cli_rs` unless intentionally mimicking Codex client identity (prefer bum originator).

---

## 10. Key source index (absolute)

| Concern | Path |
|---------|------|
| Request DTO | `/home/cristian/bum/codex/codex-rs/codex-api/src/common.rs` |
| Request assembly | `/home/cristian/bum/codex/codex-rs/core/src/client.rs` |
| Prompt / turn | `/home/cristian/bum/codex/codex-rs/core/src/session/turn.rs` |
| History roles | `/home/cristian/bum/codex/codex-rs/core/src/context_manager/history.rs` |
| ChatGPT base URL | `/home/cristian/bum/codex/codex-rs/model-provider-info/src/lib.rs` |
| Auth headers | `/home/cristian/bum/codex/codex-rs/model-provider/src/bearer_auth_provider.rs` |
| SSE | `/home/cristian/bum/codex/codex-rs/codex-api/src/sse/responses.rs` |
| Item models | `/home/cristian/bum/codex/codex-rs/protocol/src/models.rs` |
| Effort enum / catalog | `/home/cristian/bum/codex/codex-rs/protocol/src/openai_models.rs`, `models-manager/models.json` |
| Tools wire | `/home/cristian/bum/codex/codex-rs/tools/src/tool_spec.rs` |

---

*Generated from official Codex source exploration for bum Phase 9 blocker fix and Phase 10+ planning. Re-verify against upstream if Codex CLI moves major wire versions.*
