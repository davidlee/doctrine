# Review RV-391 — design of SL-267

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

<!-- Pre-reading + lines of attack: what this review is probing, the invariants
     it must hold the subject to, and where the bodies are likely buried. Seeded
     at `review new`; the reviewer fills it before raising findings. -->

### Pass 2 — inquisition (2026-09-26, F-5..F-12)

A second, adversarial pass on design revision 28, held to `POL-002`, `ADR-005`,
`ADR-019`, `ADR-023`, `DEC-127` and the slice's own disciplines. Lines of
interrogation:

1. **Does the rule (`DEC-311`) admit every form the shipped corpus legitimately
   uses?** Checked: shipped memory keys, skill names.
2. **Are the audit's premises true against the corpus today?** Each `IMP-484` gap
   and each current-state claim re-read at source, not taken from the backlog prose.
3. **Is the reader test honest?** Does it run where a private id would *fail*
   to resolve, and does it cover every channel each swept path actually reaches?
4. **Does the design obey its own principles?** "One rule, one home" and "no third
   POL-002 rule" measured against what it mints.
5. **Is every site class covered?** Especially text addressed to maintainers,
   not client agents.
