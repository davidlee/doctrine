# QUE-001: Gating locus: role on shapes (M-Pr) vs distinct gates axis (M-E)

The core fork from RFC-008 D-a. Both mechanisms survive the S2 ("shape WITHOUT gate")
killer — the requirement that association must not be hostage to gating — because both
place gate-intent in the author's hands per edge.

## M-Pr — role on `shapes`

A closed role dimension `{gates, informs}` on the `shapes` RelationLabel, riding
ADR-016 / SL-149's role machinery. Gating is a **consumer projection** over the
`gates`-role subset of `shapes` edges. RFC-003-clean: graph-effect stays in the
consumer, intent rides a role. Risk: roles currently only refine `references`, not
`shapes` — extends the role grammar to a second label.

## M-E — distinct `gates` axis

`shapes` stays purely semantic; `gates` is a separate dep/seq axis alongside
`needs`/`after`. Fully orthogonal; "the dep/seq layer" taken literally. Risk: a second
near-duplicate axis (`gates` vs `needs`) — a record's `gates` is structurally an
inverted `needs`; coordinate with IMP-033.

## Dependencies

- D-a resolution is gated by the validation of ASM-001 (if M-Pr is chosen)
- D-c (DEC-001, outbound direction) is orthogonal but must be settled first or in tandem
- CON-001 (association ≠ gating) is the requirement both must satisfy

## References

- RFC-008 § Mechanism options, § S2 (the killer), § D-a
- ADR-016 — closed role dimension (M-Pr's foundation)
- IMP-033 — cross-kind dep/seq (M-E must coordinate)
- SL-158 — the parked implementation slice, unblocked by this answer
- SPEC-019 — epistemic record spec (consumer of the chosen mechanism)
