# IMP-462: Design load-bears on research figures that cannot be re-read

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Observed

`SL-246`'s design cites measurements from `research/research.md` throughout, and
several rulings rest on them: `facets` costs ~30% of `full` (31.8 KB against
107 KB over fifteen records), per-kind fill rates (decisions 24%, questions 10%,
assumptions 37%, evidence 58%, constraints 60%), "population is all-or-nothing",
"4 of the 15 render nothing", `QUE-206` at 6.7 KB. `R1`, `R3`, `F3`, `DEC-149`'s
healthy-corpus ruling and `DEC-150`'s field-selection criterion all depend on
them, and the 6.7 KB figure is hard-coded into a marker string the design
specifies verbatim.

`research/` is **runtime tier: gitignored and untracked**. So:

- the cited figures cannot be re-read by anyone but the machine that produced
  them, and not by that machine after a clean;
- `doctrine slice research <N>` re-runs the round and produces *new*
  measurements against a corpus that has moved, not the ones the design cites;
- an auditor at `/audit` or `/close` therefore cannot verify a load-bearing
  number, only accept it.

`RV-370` ran seven adversarial rounds and verified counts (`F-22`), `file:line`
citations (`F-26`) and function signatures relentlessly against the tree. It
pointed none of that at the one artefact whose numbers decide whether the
capability is worth building. The gap is structural, not an oversight of that
review: there was nothing to point at.

## Why the current design is defensible, and where it stops

The tiering is deliberate and mostly right — a research round is a disposable
sweep, re-runnable by construction, and `/research`'s own contract says the
findings that must survive are **inlined** into the slice. `SL-246` does that:
`notes.md` says so explicitly and `§3.3` `F5` was rewritten at round 2 to state
its population as a checkable table rather than a bare count.

What inlining does not carry is **re-derivability**. An inlined number is a
claim in prose with no reproducible source, which is exactly the shape
`STD-003`-adjacent thinking objects to elsewhere: the reader cannot tell a
measured figure from a remembered one.

## Worth considering

- promote the *cited* figures — not the whole artefact — into an authored,
  diffable evidence record (`EVD`), with the corpus state they were taken
  against, so a later reader can tell what would invalidate them;
- or record the measurement *command* beside each figure so it is re-derivable
  even when the numbers move;
- or accept it and say so in the research contract, so authors know an inlined
  figure is an assertion rather than evidence.

The first is the one that makes a stale figure detectable; the third is honest
and cheap and makes nothing detectable.

## Provenance

`RV-370` round 8, raised as an advisory by the raiser rather than as a finding
(the ledger was closed, and the gap is the platform's rather than `SL-246`'s).
Recorded in `RV-370`'s Synthesis under *what this review did not reach*. Sibling
of [[IMP-461]] in kind: both are artefacts a design depends on with nothing
making their staleness visible.
