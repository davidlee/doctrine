# IMP-409: No authored link from a VT criterion to the EX criterion it proves

A phase's exit criteria (`EX-N`) are its obligations; its verification criteria
(`VT`/`VA`/`VH`) are how they are proved. **No authored field connects them.**
The mapping lives only in the author's prose, so no consumer can derive which
obligations are proved, which are unproved, or what an obligation's evidence
is.

This is the one unowned fact the 2026-08-08 obligation study found — one
candidate in eight. Every other candidate obligation field restated an existing
owner: identity and order by `REQ-441`, statement by `EX-N.text`, requirement
links by the per-phase `requirements` array (ISS-321), refinement lineage by
`REQ-442`, validation by `REQ-443`, phase actionability by
`compute_next_phases()`.

**It is a field on an existing row, not a new entity.** Whether it belongs on
the VT row, on the EX row, or as a separate mapping is a specification
question and is deliberately not settled here.

**Home.** The "Phase plan surface" component spec that IMP-382's `/spec-tech`
half will author. That half is unwritten, which is the open window for it.

Related: ISS-325 is the same missing identity seen from the binding end — a
coverage entry not bound to its check definition. Both studies converged on the
pair independently.

Evidence: `.doctrine/rfc/027/obligation-study/conclusion.md` §§ 2, 6 (R1);
`.doctrine/rfc/027/proof-binding-study/conclusion.md` § "Relation to brief 03".
