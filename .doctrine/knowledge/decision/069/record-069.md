# DEC-069: Measurement surface is selectors plus uncovered symlink closure

Taken during SL-232's design round, in response to RV-314 F-1, F-10 and F-11.

## The reframing this turns on

The design's I9 guaranteed that *every path in the claim surface is a real
tracked index entry*. That is a **soundness** property — nothing false gets in.
The false-attestation hazard is the opposite shape: **completeness** — real
evidence omitted from the surface, so nothing ever probes it. Both RV-314
blockers are completeness failures, and they slipped past eight adversarial
rounds because the invariant was watching the wrong direction.

The cause is a reuse. `DEC-053`'s index-first rule was built to answer *"does
this entry contribute?"* — a **reporting** question — and § 5.2 then used the
same instrument to build the **measurement** surface. Two questions, one
instrument. That is the error the design's own § 5.7 identifies four separate
times and answers with *"none — record it at the source"*; this is its fifth
instance, unnoticed because both questions look like *"which paths?"*.

## Why the legs were never the problem

Enumerating the HEAD × index × worktree cube (18 states, 16 of them dirty)
shows the three probe legs detect every dirty state. The decisive one is
`HEAD=A, index=B, worktree=A`: tracked diff `0`, untracked `0`, index diff `1`
— which also proves the index leg is not redundant.

So the measurement was never deficient. The **pathspec domain** was: under pure
index-first, an index-detached path is absent from the surface, and no leg is
ever asked about it. Widening the domain closes all three routes at once, which
is why F-1, F-10 and F-11 are one repair rather than three.

## What stays untouched

`DEC-053` survives in full for contribution reporting. No `realpath`, no
character-based shape classification, no whole-component-prefix rule — the
three totality failures that motivated it are not reopened. `RV-307 F-27`'s
history-vs-now cut is likewise untouched: this is a *now* question throughout,
and nothing here widens `commits_touching`.

`E15`/`R-H` is unchanged. A path reachable only through a symlinked directory
matches nothing in HEAD or the index, and `ls-files --others` does not descend
symlinked directories, so the known boundary neither closes nor widens.

## Sequencing

`RV-314 F-7`'s lexical-guard repair is a **prerequisite**, not a neighbour: an
unguarded derived target returns exit 128 on all three legs, so this decision
triples the failing command surface until the guard applies recursively to every
derived path.

Bounded by DEC-070 (the evidence domain) and DEC-071 (the atomicity boundary).
Neither is optional — without DEC-070 this decision is undefined against
`--exclude-standard`, and without DEC-071 its invariant claims an atomicity the
probes do not have.
