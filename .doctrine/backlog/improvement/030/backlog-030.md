# IMP-030: Slice B: reconcile writer + closure gate + NF-001 coverage->status-writer import-edge enforcement (SL-042 dependent)

The dependent half of SPEC-002, deliberately excluded from SL-042 (the observe
substrate). SL-042 establishes only the structural preconditions; this item is
the **reconcile/close** half that consumes them.

## Scope (from SL-042 design §3/§4/§5.5)

- **Reconcile writer** — the verb that authors `REC-NNN` deltas and writes
  requirement-status / spec truth explicitly (REQ-105, the dual of SL-042's
  REQ-114 negative). The only path allowed to move authored status.
- **Closure gate** — the reverse corpus-scan gate (SL-040 D-C9b grain, no reverse
  index) that reads the composite/drift verdict at slice close.
- **NF-001 import-edge enforcement (load-bearing).** SL-042 holds NF-001
  structurally (Verdict return type, two-enum non-reference, distinct stores) but
  its import-edge clause — *no edge from `coverage` to a requirement-status
  writer* — is **vacuous** there because no writer exists. This item introduces
  the writer it must wall off, so the guard becomes load-bearing here.

## Carries the SL-042 deferrals that resolve at the consumer

- **EX-2 dead-code (RV-003 F-1).** The coverage leaf + scan shell are dead in the
  clippy(bins/lib) build until a real consumer wires them. The self-clearing
  `#![cfg_attr(not(test), expect(dead_code))]` on `coverage.rs` / `coverage_scan.rs`
  and the item-level one on `git::head_sha` **retire automatically** when this
  reader references them — EX-2's "genuinely used in the non-test build" lands here.
- **Perf triggers (RSK-006).** The conditioned reverse-index / staleness-batching
  triggers are sized against *this* reader's query volume — see RSK-006.

Governance: SPEC-002 (D1–D9), ADR-003 (canonical loop, explicit-authorship), ADR-009.
Realises REQ-105. Dependent of SL-042. See `design.md` §3 "Slice B".
