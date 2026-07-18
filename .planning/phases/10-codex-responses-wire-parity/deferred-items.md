# Phase 10 deferred items

## Found during Plan 10-05 Task 1 (2026-07-18)

### Workspace `cargo fmt --all -- --check` FAIL

- **Class:** Pre-existing / out-of-scope hygiene (not a Phase 10 wire-parity product defect)
- **Evidence:** Exit code 1; ~68 files would reformat (pager, shell auth, config, models, tools, update, sampler/client.rs, sampling-types/conversation.rs, shell tests including model_switch_gate, etc.)
- **Impact:** Focused Phase 10 behavior and compile commands all PASS; only the workspace-wide fmt gate fails
- **Do not:** Run mass `cargo fmt --all` as a silent fix during validation — Phase 10 process note warns about formatter spill across unrelated tracked files
- **Remediation:** Separate chore/PR with explicit format-only scope, or accept fmt check as deferred until a dedicated style wave
- **Owning plan for remediation:** None in Phase 10 — track as workspace hygiene; re-check before milestone ship if required by verify-work
