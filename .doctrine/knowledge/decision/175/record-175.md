# DEC-175: PRD-010 kind set is a stale enumeration, not a violated rule

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## The clause, read whole

> The kind set is exactly the four initial kinds — assumption (`ASM`), decision
> (`DEC`), question (`QUE`), constraint (`CON`), three-character per-kind
> prefixes — **and may not be extended without a reserved id.**

Read only to the dash, it forbids seven kinds. Read to the end, it *permits*
extension and names the precondition. "Initial" says the same thing again.

## The precondition was met

`SL-159` (EVD, HYP) and `SL-197` (CPT) each registered a `KindRef` in the
corpus-wide `KINDS` table — `EVIDENCE_KIND`, `HYPOTHESIS_KIND`, `CONCEPT_KIND`
— with its own three-character prefix, its own directory under
`.doctrine/knowledge/<kind>/`, and its own reservation namespace, so `EVD-001`
and `CPT-001` allocate independently. `src/integrity.rs` carries an explicit
collision check confirming the additions clash with no existing corpus prefix.

That is what "a reserved id" asks for, and it was done.

## Why the distinction is worth a record

Three reasons.

It changes the amendment's shape — a refresh of a stale sentence, not the
reversal of a broken rule — and therefore its cost and its tone.

It keeps the REV accurate. Governance that claims a rule was violated when it
was followed is itself wrong, and this REV's whole purpose is to stop
`SPEC-019` and `PRD-010` misleading their next reader.

And it is an instance of the very class this slice studies: a statement whose
front half went stale while its back half stayed true, which nobody re-read as
a whole. SL-249's own triage made exactly that mistake against exactly that
sentence, four hours earlier.

## What still needs amending

The enumeration. It named four kinds when there were seven, and it is what
misled this slice into scoping a larger REV than the evidence supports. The
extension rule after it survives verbatim.
