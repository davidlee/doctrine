# ISS-023: Survey pending requirements — confirm delivered, transition to active

## Outcome (2026-06-18)

**101 requirements transitioned `pending → active`** across 14 specs. 5 remain genuinely pending.

### Transitions by spec

| Spec | Before (pending) | After (pending) | Transitions |
|------|-------------------|-------------------|-------------|
| PRD-001 (Slices) | 6 | 0 | 6 |
| PRD-002 (Specifications) | 8 | 0 | 8 |
| PRD-003 (Skills) | 6 | 0 | 6 |
| PRD-004 (Memory) | 6 | 0 | 6 |
| PRD-005 (Reservation) | 6 | 2 | 4 |
| PRD-006 (Install) | 6 | 0 | 6 |
| PRD-007 (Boot) | 6 | 0 | 6 |
| PRD-008 (ADRs) | 6 | 0 | 6 |
| PRD-009 (Backlog) | 13 | 0 | 13 |
| PRD-010 (Knowledge) | 10 | 1 | 9 |
| PRD-011 (Priority) | 13 | 1 | 12 |
| PRD-013 (Reconciliation) | 9 | 0 | 9 |
| SPEC-001 (Priority Engine) | 8 | 2 | 6 |
| SPEC-002 (Recon Engine) | 4 | 0 | 4 |
| **Total** | **113** | **5** | **108** |

(SPEC-002 already had 9 active before this survey.)

### Remaining genuinely pending (5)

| Requirement | Spec | Label | Gap |
|-------------|------|-------|-----|
| REQ-021 | PRD-005 | FR-003 | Reach selection — only `LocalFs` backend, no git-ref/shared-remote |
| REQ-022 | PRD-005 | FR-004 | Survey with holder+time — no claim metadata, no `reservation list` |
| REQ-065 | PRD-010 | FR-006 | Spawn backlog item from knowledge record — no dedicated CLI verb |
| REQ-258 | PRD-011 | FR-009 | Cross-kind dep/seq write-side allowlist validation not implemented |
| REQ-093 | SPEC-001 | FR-004 | Architectural triggers — authored field exists but no path-glob actionability mask |
| REQ-094 | SPEC-001 | NF-001 | Cache stamping — input signature not computed, cache not implemented |

### Method used

Six parallel scouts surveyed codebase for delivery evidence, then `doctrine spec req status <REQ> --to active` for each confirmed delivery.

### Notes

- PRD-005's REQ-021/022: core claim primitives work (atomic mkdir, retry loop) but remote-reach and operator-visible metadata are unbuilt.
- PRD-010's REQ-065: relation seam exists, but no dedicated `knowledge spawn` verb wires the flow.
- PRD-011's REQ-258: read side fully wired (cross-kind dep/seq edges reach the priority engine), but no author-time allowlist gate.
- SPEC-001's REQ-093/094: triggers and cache stamping were deferred; the denylist test and deterministic compute are delivered (REQ-095, REQ-078).
