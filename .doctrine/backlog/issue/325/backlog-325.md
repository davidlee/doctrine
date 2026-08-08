# ISS-325: coverage entry is not bound to its check definition digest

`CoverageEntry` persists `check: Option<VtCheck>` **verbatim and undigested**.
A coverage entry therefore goes stale only when its *evidence's* git anchor
ages. Change the binding — swap the matcher pattern, change the command,
retarget the alias — and the prior `Verified` stands unchallenged.

So "the criterion did not change" and "the proof did not change" are
indistinguishable to the engine, and only the second is currently checked.

**The mechanism already exists one subsystem over.** `Step::material()`
(`src/design_run/runbook.rs`) builds a netstring-framed, version-tagged digest
over a step's whole definition, so any edit makes the discharge stale by
construction. `DEC-101`: *an id solves reference, not equivalence.*

Positive control for the absence: `digest` appears 34× in `runbook.rs`, **0×**
in `coverage.rs`.

**Sequencing.** This is a schema change to a persisted type. It should be
governed by the SPEC-002 amendment that CHR-058 produces, not landed ahead of
it — the requirements governing this seam currently assert nothing.

Related: IMP-409 is the same missing identity seen from the criterion end (no
authored link from a VT row to the EX row it proves). Two studies converged on
this pair independently — `.doctrine/rfc/027/proof-binding-study/conclusion.md`
§ R1 and `.doctrine/rfc/027/obligation-study/conclusion.md` § 2. RFC-027 records
it as the only unowned fact either study found.
