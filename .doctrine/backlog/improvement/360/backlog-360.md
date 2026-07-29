# IMP-360: Section reorder produces no change row

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

SL-233 PHASE-14 makes hand-reordering sections in `design.md` a **supported
edit**: re-adoption takes document order as authoritative and renumbers `seq`
to the marker sequence (`EX-7`). A user can therefore reorder the document,
re-adopt, and have the run accept it.

But re-adopting a purely reordered document produces a lawful revision with
**zero material change rows**. `ChangeEvent` has no member meaning "order
moved", and the marker-grammar sketch's §(d) vocabulary is closed. So:

- the reorder is invisible in the change log;
- any envelope reader diffing revisions sees nothing;
- the `seq` counter has nonetheless advanced (fresh values are claimed in
  marker order on every re-adopt, per PHASE-14 D4), so state moved while the
  record says nothing did.

Left alone deliberately in PHASE-14 — extending a closed vocabulary is out of
that phase's scope and would want its own design pass. Filed so the gap is not
rediscovered from the symptom.

Open question for whoever takes it: whether the answer is a new `ChangeEvent`
member, or whether a pure reorder should instead be *refused* as a no-op
adoption. PHASE-14's `EX-7` argues against refusal ("refusing it would make the
authored tier less editable than a plain file"), which points at the vocabulary.

Surfaced by SL-233 PHASE-14 (worker finding F-6).
