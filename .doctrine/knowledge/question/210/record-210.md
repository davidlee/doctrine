# QUE-210: Can a large design maintain parallel enumerations without a machine guard?

## The question

`SL-248`'s `design.md` is ~4600 lines and carries the same enumeration in
several places by construction: `sec-7` table A's rows, `sec-2`'s properties
table, `sec-2`'s numbered invariant list, `sec-2`'s channel ledger, `sec-7`'s
`PropertyRemoval` variants and delta table, `sec-7`'s executed test titles,
`sec-8`'s requirement-closure table, and `DEC-156`'s count. A row added in one
must appear in all the others it touches.

**It does not stay consistent.** `RV-346` `F-28` raised this as a defect in round
4 and the remediation was a single-source rule for the *numeral* — the count
lives in table A and no section restates it. Round 5 then added two rows, and a
pre-brief sweep before round 6 found **ten** defects, eight of them this same
class, including two inside the very section that documents `F-28`. The
single-source rule worked exactly as far as it reached and no further: it fixed
the number and did nothing for the row contents, the invariant list, the test
titles, the delta table, or the record.

So: is there a structural answer available at design altitude, or is the honest
ruling that a document of this size maintains parallel enumerations by reader
discipline — in which case the design should say so where it currently implies
otherwise?

## Why it is not already answered

The design *proposes* its own guard and the guard cannot fire yet. `sec-7` states
that its executed test list "is asserted to be exactly the row set by
`every_row_id_is_covered_by_exactly_one_table`, so a row added without a title is
a red test rather than a silent gap". That test does not exist — the suite is not
built — so when round 5 added rows 12 and 13 without titles, nothing caught it.
A guard that lives in the implementation protects the implementation from
drifting from the design; it protects the design from nothing during the months
the design is the only artefact.

That asymmetry is the question's real content. Every candidate answer has it:

- **Derive rather than duplicate.** Make `sec-2`'s tables explicit projections of
  `sec-7`'s, stated as such, so a reader knows which is authoritative. Cheap, and
  only converts silent drift into visible staleness.
- **Reduce the number of enumerations.** Some of the eight exist for reader
  convenience rather than necessity. Cutting them cuts the surface, at a cost in
  navigability for a document already hard to hold.
- **A lint over the authored prose.** Parse the tables out of `design.md` and
  assert their agreement. This is the only candidate that actually fires during
  design, and it is a real tool with a real cost, and it is doctrine-general
  rather than `SL-248`-specific.
- **Say it is reader discipline and stop implying otherwise.** Honest, and
  concedes that the next round of additions drifts again.

## What would settle it

`RV-346` round 6 line 6 puts the question to the external reviewer. That produces
a ruling for `SL-248`. Whether it generalises — whether doctrine should carry a
mechanism for authored enumerations that must agree — is the part that outlives
this slice, and is why this is recorded here rather than only on the ledger.

If the answer is the lint, it is `backlog new` work against doctrine itself and
not `SL-248` scope.

## Evidence

The ten defects and their commits are the data: `48bc59a44`, `9ffe8a1d6`,
`b504edc95`. Note that three were found by cheap scout agents and one by
executing a probe — **none** by re-reading, which is `sec-9` `R3`'s claim about
property suites turning out to hold for the document's bookkeeping too.

Relates to [[SL-248]] (`sec-2`, `sec-7`, `sec-9` `R3`), [[DEC-156]] (the count
whose corrections keep exposing this), [[RV-346]] `F-28` and round 6 line 6.

## The answer — RV-346 round 6, 2026-08-08

**It cannot be fully fixed at design altitude, and the design now says so.** The
external reviewer's ruling, taken as written: a document of this size maintains
its parallel enumerations by discipline, and the mitigation available is naming
which one is authoritative rather than pretending a guard exists.

What `SL-248` did with it — the first and fourth candidates above, together, and
neither alone:

- Table A is **normative**; every other enumeration is a **manually maintained
  projection**, labelled as one where it appears, and wrong by default when it
  disagrees (`sec-7` § *Table A is the inventory; everything else is a
  projection*).
- Exactly **one** projection is machine-checked, and it is narrower than the
  design had been claiming: the `Property` enum has one variant per row and keys
  the verdict, so a row the suite can construct that the enum cannot name is a
  compile error. That fences the enum against the *code's* tables. It says
  nothing about the prose.
- The design stops implying otherwise. `every_row_id_is_covered_by_exactly_one_table`
  was cited **as enforcement in the remediation of the finding it was meant to
  close** while being an unwritten planned test — which is why the class survived
  two corrections. It stays planned and is worth writing, but it is no longer
  claimed to protect the document.
- `sec-9` `R9` carries the residual with the altitude stated.

**The generalisable part is unchanged and is deliberately not answered here.**
Whether doctrine should carry a mechanism for authored enumerations that must
agree — the lint over authored prose — is the candidate that would actually fire
during design, and it remains doctrine-general work outside `SL-248`'s scope. It
is not filed as backlog work, because the ruling is that the cheaper mitigations
suffice for this document; a future design that drifts again under the projection
rule is the evidence that would justify the tool.

Confirming datum from the same round: round 6's own credential-row split forced a
hand-edit of three of the four projections in one change. The projection rule
made that visible and did not prevent it, which is exactly the altitude claimed.
