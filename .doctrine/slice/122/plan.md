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
attach. It deliberately bundles the *unsexy* registration surfaces the codex pass
flagged (F1): `integrity::KINDS`, install/manifest, and the `.gitignore` negation
(the silent-uncommittable trap, F5). Those are cheap individually but fatal if
missed, so they gate the phase via EX-3/EX-4 rather than being assumed "free".
EX-5 holds the behaviour-preservation line on the shared engine.

**PHASE-02 (lifecycle) and PHASE-03 (relations) are independent of each other**,
both depending only on PHASE-01. They are ordered lifecycle-before-relations
because the status machine is self-contained within `rfc.rs`, whereas relations
reach across into `relation.rs`, `catalog/scan.rs`, and `revision.rs` — the wider
blast radius goes second once the kind is otherwise stable. (If parallelised
later, they are largely file-disjoint except both touch `rfc.rs`.)

**PHASE-03 carries the two genuinely subtle pieces.** RFC's own edges are a near-
trivial `sources`-set addition (design §1 Decision 1), but the REV→RFC
`originates_from` edge (Decision 2) must be *revision-owned* (TypedVerbOnly) so
generic `doctrine link` cannot bypass REV's change discipline (codex F2). VT-3
pins that refusal explicitly — it is the whole point of the tier choice. The
catalog-scan prefix dispatch (EX-2/VT-4) is the other F1 surface: skip it and RFC
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

- Phase boundaries follow the design's section seams (§1→03, §2→02, §4→04,
  §5→05), so a drifting phase is easy to trace back to its governing decision.
- The codex adversarial findings are distributed to where they bite: F1/F5 in
  PHASE-01 + PHASE-03, F2 in PHASE-03 (VT-3), F4 in PHASE-02 + PHASE-04, F6 in
  PHASE-05.
- No new engine abstraction is introduced (A2): RFC is the ~13th Kind-is-data
  kind. If PHASE-01 surfaces a `GovKind`-wrapper assumption that every kind is
  governance-or-slice, stop and `/consult` rather than bending the engine.
