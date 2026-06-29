# IMP-210: fulfils(full) close-cascade hint in doctor/close

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Spun out of SL-176 design (Finish Axis B). SL-176 ships the `fulfils` label +
`{full, partial}` degree facet but deliberately **does not** build the consumer that
acts on it.

This item: make `doctor` / `/close` *read* `fulfils(full)` inbound edges on a backlog
item as a **closure hint** — "BACKLOG-NNN has a `fulfils(full)` inbound but is still
open: candidate for close." Pure read-path consumer change.

**Constraint (locked, RFC-003 F-6):** hint-not-auto. The cascade only *suggests*; a
human confirms. Degree does **not** aggregate (two `partial` inbounds ≠ one `full`), so
item-completion over a set of inbound `fulfils` edges is a judgement, never arithmetic.

Depends on SL-176 landing the `fulfils` vocabulary + degree facet first.
