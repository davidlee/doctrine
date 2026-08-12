# Row verdicts are the preservation bar

`SL-253` `inq-5` asked how behaviour preservation is demonstrated when the
rendering deliberately changes. `AGENTS.md` requires the existing suites green
**unchanged** for shared machinery; `DEC-190` names `RV-352`'s baseline as the
bar; `DEC-191` requires the output to change. Nothing reconciled them.

Three further shape changes taken in this design run compound it — `DEC-196`'s
`RowId` re-keying, `DEC-197`'s backend-parameter narrowing, `DEC-198`'s
floor/profile key split — and every one edits test source that names the moved
types. So *green unchanged* cannot be literally true here. Asserting it anyway is
how a slice redefines its own bar without saying so.

## The bar, in three layers

**Layer 1 — invariant.** The nineteen row-to-verdict mappings, the four
auxiliary claims and the two unrowed readings reproduce exactly. This is what
`RV-352` actually measured, and it is precisely what `DEC-191` does not touch:
the collapse and the rendering change, the row outcomes do not.

**Layer 2 — permitted, enumerated in advance.** The `outcome=` line (`DEC-191`),
the exit constants (`DEC-194`), and the row key spellings (`DEC-198` — rendering
is `{:?}` over `RowId`). Enumerating these up front is what stops layer 2
becoming a licence: **a difference not on this list is a regression.**

**Layer 3 — mechanical.** Test source naming moved types is translated, and
reviewed as a translation diff in which no *asserted value* moves.

## The instruments

`just gate`'s `test-all` names its packages (`-p doctrine -p cordage`) and
`crates/doctrine-control` sits outside every default selection (`RV-353` `F-4`,
on `ISS-342` / `ISS-343`). So this slice's evidence does not ride `just gate`,
and no attempt is made to wire the crate into it — `RV-352` `F-8`'s open
question about what a host failing the rows should report stays open.

- **`just capsule-check`** — the per-phase gate. Clippy plus the crate's suites,
  no `bwrap` needed, green at the end of every phase.
- **`just capsule-verify`** — the live nineteen-row run. A **phase exit
  criterion** for every phase touching the payload, and the default for any
  phase where it is arguable. Only a phase that plainly cannot reach a row omits
  it. The run is cheap on this host, so the bias is toward more often.

## The two artefacts

**A committed key translation table**, authored in the phase that changes
`RowId`. The post-split comparison against `EVD-022` is then mechanical rather
than a judgement call at audit time.

**A characterisation test** recording the row-to-verdict mapping as data,
written *before* the split and carried through it mechanically. It is scaffolding
with a short lifespan — written against types that are about to be deleted — and
that cost is accepted because it moves the failure from audit time to the phase
that caused it. The artefact comparison catches the same regression later; the
test catches it where it happened.

## The bracket

`EVD-022` is the pre-split half, captured at `4662e64eb` before any code lands,
because it could not be captured afterwards. The post-split run is its pair.
