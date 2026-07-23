# DEC-004: Harvest shape — shared doc + /notes entry

The end-of-work harvest is owned once by a PULL-tier reference doc
(`install/harvest.md`, ADR-005) with `/notes` as the single routed entry point.
No dedicated `/harvest` skill: its trigger ("end of a coherent unit") is already
`/notes`' boot-table row — two skills competing for one moment is a routing
defect. Chosen over (a) a new skill and (b) doc-only with no entry point.
Rationale + consequences: SL-215 `design.md` D1.
