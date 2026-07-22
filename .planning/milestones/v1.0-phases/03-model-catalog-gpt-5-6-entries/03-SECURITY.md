---
phase: 3
status: SECURED
asvs_level: 1
threats_closed: 12
threats_open: 0
reviewed: 2026-07-16
---

# Phase 3 Security Audit

**Verdict:** SECURED  
**Threats closed:** 12/12  
**ASVS:** Level 1 · block_on high

## High mitigations present

| ID | Mitigation |
|----|------------|
| T-03-01 | Remote parse hardcodes `provider: ModelProvider::Xai` |
| T-03-05 / 05b / 05c | Codex remove-then-append with bundled authority and order |
| T-03-09 | ACP `meta.provider` from in-process `info.provider` only |

## Accepted risks (plan)

| ID | Note |
|----|------|
| T-03-02 | User config provider override same trust as BYOK |
| T-03-07 / T-03-08 | GPT listed without Codex login (Phase 6 gate) |
| T-03-SC | No new dependencies |

## Out of phase

Wrong GPT inference URLs until Phase 4 routing (catalog-only scope).
