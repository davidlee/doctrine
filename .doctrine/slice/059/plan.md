# Implementation Plan SL-059: Knowledge records: standalone four-kind entity surface

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Four phases, each ending green, each gated on the prior. The cut follows the
design's natural layering: a self-contained pure data layer (PHASE-01), the
engine wiring that admits it to the corpus (PHASE-02), the command surface on
top (PHASE-03), and the disambiguation/plumbing tail the new kind forces
(PHASE-04). The dominant constraint shaping the order is **behaviour
preservation** (NF-001): the engine `Kind` is data, not a trait, so this slice
adds table rows, not engine changes — every phase keeps the existing
slice/ADR/spec/backlog/memory/relation suites green unchanged, and the riskier
wiring phases carry that as an explicit exit gate.

## Sequencing & Rationale

**PHASE-01 builds the whole pure layer before any wiring** because it is the one
part that stands up and proves itself in isolation. The module, the four `Kind`
descriptors, the per-kind status vocabularies, the typed facet enum-of-structs,
the closed value-enums, the three-layer parse, and the scaffold templates are
all internally coherent and testable (round-trip, the optional `"" -> None`
seam, the drift canaries, the seed-status anti-drift, the scaffold fileset)
without the entity ever appearing in `integrity::KINDS`. Splitting types from
parse from scaffold would only create sub-phases that cannot go green alone; the
backlog twin was grown the same way. Keeping the pure layer whole also means the
corpus is never in a half-wired state across a phase boundary.

**PHASE-02 is the load-bearing wiring phase, and its three edits must land
together.** Admitting the four kinds to `KINDS` is what makes them real to the
corpus-wide machinery — but the moment a `KINDS` row exists without a matching
`outbound_for` arm, every debug-build graph scan panics on the unrouted-prefix
`debug_assert` once any record exists. So the empty four-prefix arm (routing,
not relations — Slice B replaces it with the real accessor) co-lands with the
rows. The priority partition lands here too: it reads the `*_STATUSES` consts
from PHASE-01 and declares the never-`Workable` posture (NF-003). This phase
also names the **scan-side** total-dispatch guarantee (the partner to the
outbound-side one): the all-`KINDS` scan visits the new trees even on a
record-less repo and stays benign only because the id scan tolerates a missing
directory — a guarantee worth a regression tripwire, not just an assumption.
Behaviour preservation is the phase's standing exit gate.

**PHASE-03 puts the command surface on the wired engine.** It needs the kinds
live (PHASE-02) so the cross-kind list, the prefix→kind resolution, and the
conformance row all have a real corpus to act on. The four verbs ride the shared
listing spine and the uniform `<kind> <verb>` grammar rather than a bespoke
surface; the only genuinely new mechanism is the kind-relative `--status`
validation (a union known-set across the four vocabularies) and the foreign-kind
state refusal on transition. Goldens and the conformance-matrix row pin the
surface byte-for-byte.

**PHASE-04 is the tail the numbered DEC kind forces, deliberately placed last.**
The `decision_ref` disambiguation is only *warranted* once DEC is a numbered
2-part kind (PHASE-02) — before that the 2-part `DEC-NNN` form is unambiguous and
there is nothing to clarify. The edits are clarity-only: the label stays
`Unvalidated`, so external 3-part the external decision register cites survive untouched, and the
fixture edits move value and assertion in lockstep — no engine behaviour
changes. The authored-entity install/gitignore plumbing rides here too; it need
not precede the earlier phases (their tests run in temp dirs that never touch the
real tree) but must not be forgotten, since without the gitignore negation the
record tree is silently uncommittable.

## Notes

- **Out of scope (the slice ships alone):** no relation/spawn seam, no
  supersession, no direct gating, no memory↔record seam. The single relation-layer
  concession is PHASE-02's *empty* `outbound_for` arm — pure routing so the
  KINDS-driven dispatch stays total; Slice B swaps it for the real reader.
- **PHASE-02 coupling is the one hard sequencing rule:** the `KINDS` rows and the
  `outbound_for` arm cannot be split across commits — a row without the arm is a
  debug-build panic.
- **The design (`design.md`, locked at `6fc5020`) is the higher authority** for
  every mechanism here; this plan only sequences it. Where a phase's verification
  cites a design finding (F-A1/F-A2/F-A5/F-A6/F-A7), the design section is the
  rationale of record.
