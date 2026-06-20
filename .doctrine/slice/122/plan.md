# Implementation Plan SL-122: RFC kind: first-class discussion artifact

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Five phases take RFC from "no such kind" to "shipped, governed kind". The spine
is **kind-first, then the axes that hang off it** (lifecycle, relations,
visibility), then the **governing ADR last** so it records the as-built decision
rather than a forecast. The design (§1–§5) is canon; each phase implements one of
its locked sections.

## Sequencing & Rationale

**PHASE-01 (foundation) is the hard dependency for everything.** Until RFC is a
registered, mintable, KINDS-known kind with a committable tree, nothing else can
attach. It bundles the **complete F1 registration set** — not a subset:
`integrity::KINDS`, the `catalog::scan::outbound_for` **arm**, install/manifest,
and the `.gitignore` negation (the silent-uncommittable trap, F5). The
outbound_for arm is non-negotiably here, not in PHASE-03: a KINDS row without its
arm makes every debug-build corpus scan panic on the `unrouted KINDS prefix`
`debug_assert!` the moment an RFC is minted (`scan.rs:68-83`, the REV precedent
spells this out). A stub `Ok(vec![])` body satisfies the routing; PHASE-03 fills
it. PHASE-01 also lands the **status field** (default `open`) and the
status-bearing read path — these are entity-model foundation (design §2), tied to
the *presence* of the field, not lifecycle features, so building `show` here on
that path avoids a mid-slice reader migration. EX-3/EX-4/EX-6/EX-7 gate these;
EX-5 holds the behaviour-preservation line on the shared engine; VA-2 pins the A2
GovKind-boundary check (stop-and-`/consult` if the engine resists a 13th
data-only kind).

**PHASE-02 (lifecycle machine) and PHASE-03 (relations) are independent of each
other**, both depending only on PHASE-01. PHASE-02 is now narrowed to the status
*machine* on top of PHASE-01's field — the `RFC_STATUSES` set, the transition
verb, and `rfc list` filtering — all self-contained within `rfc.rs`. PHASE-03's
relations reach across into `relation.rs`, `catalog/scan.rs`, and `revision.rs` —
the wider blast radius goes second once the kind is otherwise stable. (If
parallelised later, they are largely file-disjoint except both touch `rfc.rs`.)

**PHASE-03 carries the two genuinely subtle pieces.** RFC's own edges are a near-
trivial `sources`-set addition (design §1 Decision 1), but the REV→RFC
`originates_from` edge (Decision 2) must be *revision-owned* (TypedVerbOnly) so
generic `doctrine link` cannot bypass REV's change discipline (codex F2). VT-3
pins that refusal explicitly — it is the whole point of the tier choice. The edge
ships as a single, pinned surface: the `revision new --originates-from` creation
flag (a provenance ref, no `[[change]]` payload), mirroring `revises`; a
standalone `revision originates-from` verb is a deferred candidate, not this
phase. PHASE-03 also *fills* the stub outbound_for arm PHASE-01 routed (EX-2): the
routing already exists, so this is a body swap, not new dispatch — skip it and RFC
outbound edges silently degrade to empty in release.

**PHASE-04 (status surface) depends on PHASE-02** (needs the open/resolved
distinction to list "unresolved"). It is kept separate from PHASE-02's `rfc list`
because it edits a different surface (`status.rs`, with its serialized envelope
and empty-state rule) and must honour the visibility split: RFC appears in the
work/awareness dashboard but never in boot's governance sections (VT-3 guards the
boot snapshot byte-for-byte).

**PHASE-05 (ADR) is last by design.** The decision is already made (the design
locked it), but the ADR is the durable canon record and should cite the *actual*
shipped kind and REV edge. It amends ADR-013 via governance `related` + prose —
not `supersedes`, which would wrongly deprecate a still-accepted ADR. Acceptance
(VH-1) is a human gate; the slice's closure intent ("governing ADR authored and
accepted") resolves here.

## Notes

- Phase boundaries follow the design's section seams (§1→03, §2 field→01 +
  machine→02, §4→04, §5→05), so a drifting phase is easy to trace back to its
  governing decision. Note §2 splits: the status *field + read path* are
  foundation (PHASE-01 EX-7), the status *machine* is PHASE-02.
- The codex adversarial findings are distributed to where they bite: **the full
  F1 registration set (KINDS + outbound_for arm + manifest)** lands together in
  PHASE-01 (the arm cannot trail to PHASE-03 — `scan.rs:68-83` debug-asserts), F5
  in PHASE-01, F2 in PHASE-03 (VT-3), F4 in PHASE-02 + PHASE-04 (incl. the
  serialized-envelope VT-4), F6 in PHASE-05.
- No new engine abstraction is introduced (A2): RFC is the ~13th Kind-is-data
  kind. PHASE-01 VA-2 explicitly checks this — if a `GovKind`-wrapper assumption
  that every kind is governance-or-slice surfaces, stop and `/consult` rather than
  bending the engine.
