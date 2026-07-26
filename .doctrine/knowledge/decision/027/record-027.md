# DEC-027: Corpus-aware verify gate leaves SL-230 into SL-232

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## The decision

SL-230 narrows to the **memory body-write seam** — CLI and MCP body verbs, and
the attestation invalidation that a writable body makes necessary. The
**corpus-aware `verify` gate** — the claim-surface constructor and everything
downstream of it — leaves SL-230 and becomes **SL-232**, along with the SPEC-007
amendment (REV-034) that the gate's contract change requires.

Taken by the user at RV-307 round 8, on the responder's recommendation.

| Stays in SL-230 | Moves to SL-232 |
|---|---|
| Obj 1–2 body-write, CLI + MCP (D1, D2, D7) | Obj 3 corpus-aware verify gate (D3, D9, D10, D11) |
| Obj 5 invalidation (D4, D8, D5, `claim_snapshot`) | Obj 4 REV-034 — SPEC-007 + REQ-147 |
| I5, E1, E3, R1–R5 | Obj 6 — the dirty refusal naming `--allow-dirty` |
| OQ-1 (closed here as D8), OQ-4, OQ-7 | I1–I4, I6–I9; E2, E4–E9, E11–E13; R6–R8; OQ-2/3/5/6 |
| T1–T6, T12, T12b, T13, T15, T16, T20, T20b, T21, T22, T29 | The claim-surface algorithm (§ 5.2), and T7–T11, T14, T17–T19, T23–T39 |

**Correction (2026-07-26, SL-230's confirming pass).** The four rows above
replace a first-cut sketch taken before the id-level partition was worked out.
It read "*I1–I6, E1–E6, R1–R4, R6* stay" and placed **objective 6 in the *Stays*
column** — both contradicting this record's own prose, which says the gate "and
everything downstream of it" leaves. Read against the pre-split design text, the
executed partition is the one above: every id SL-230 moved is gate machinery
(I1 `capture()` frames, I2 `write-tree`, I3/I4 the dirty refusal and
`--allow-dirty`, I6/I7 the attested blob and tracked-file surface, E2 masters,
E4 `memory/`-absent, E5 `scope.commands`, E6 the empty-scope claim surface) and
every id it kept is body-write (I5 `thread_expiry`, E1 thread-vanish, E3 the `-`
sentinel). Nothing moved to the wrong slice; this table mis-described what moved.
Two id notes so they are not re-derived: **E10 was never minted** (the series ran
E1–E9, E11–E13) and **E12 was withdrawn by DEC-020**, not moved. SL-230's new ids
begin at I10 / E14 / T40; next free are I14 / E15 / T43.

## Why — the two halves never converged at the same rate

RV-307 ran **eight rounds and 39 findings** against one design document. Sorted
by half, the two halves are not the same artefact:

| Half | Findings | State |
|---|---|---|
| Body-write + invalidation | 7 — F-3, F-7, F-8, F-9, F-10, F-12, F-17 | **all verified**, none contested, quiet since round 4 |
| Corpus-aware verify gate | 30, incl. every open blocker and all six live contests | still producing decision- and mechanism-level defects at round 8 |

(Two more — F-4, F-5 — are the governance/REV-034 axis, verified, and travel
with the gate. 7 + 30 + 2 = 39.)

**Correction (2026-07-26, SL-230's confirming pass).** The membership above was
wrong in one place and the arithmetic in another; the shape of the argument is
untouched. **F-14 is not a body-write finding** — it lands on I6 and T24, both
SL-232's ("*T24 proves only that some blob exists at the path*", not that the
committed blob equals the attested body). **F-7 is** — it lands on R5, masters
uncovered by every invalidation path, which SL-230 retains. Because F-14 was named
local here and appeared in no inherited list, it was an orphan owned by neither
slice; **SL-232 must pick it up.** The gate half is therefore 30, not 29, which is
what makes the total reconcile to 39.

The finding rate did not decay: round 7 produced 5 new findings (2 blockers) and
6 returned contests; round 8 produced 4 new (2 blockers) and 6. Every one of
round 8's was on the gate.

## What round 8 established that forced this

Two blockers, neither a text defect:

- **F-36** — DEC-020 requires `validate` to raise every non-contributing scope
  entry, but D11 leaves `validate` with no contribution probe at all: it keeps a
  historical, `scope.paths`-gated seam that cannot implement T35. The ruling was
  applied normatively without a mechanism. Supplying one is an undesigned second
  per-entry git path plus a corpus-wide continuation policy (F-29's shape).
- **F-37** — the design's premise that a non-resolving entry contributes nothing
  is false. Three routes reproduced (git 2.54.0): `missing/../link`, a sparse
  checkout, and a `scope.paths` literal whose filename contains `*` (the shape
  rule reads the star as a wildcard and skips whole-path resolution). Each
  contributes while bypassing canonicalisation and reads clean against a dirty
  target — restoring the false attestation I9 exists to close.

Together they say the claim surface is a harder problem than a body-write slice
can carry: one needs a new mechanism, the other says totality was asserted over
an under-probed domain.

## What this is not

**Not an abandonment of the gate, and not a quality judgement against it.** The
29 findings are the most valuable output of this slice; SL-232 inherits the
design text, the decisions, and the reviewed reasoning intact rather than
restarting. Nor is it a lock-with-residual-risk: F-37 is a live false-attestation
route, which is the exact defect the work exists to close, so shipping the gate
as-is was never on the table.

**Not a deferral of the schema question.** DEC-020 already established that the
stable answer to non-contribution is a **declared** boundary rather than a
derived one — a persisted field. SL-232 is the slice where that can be scoped
properly alongside IMP-318 and QUE-173, which DEC-020 said should travel
together. Splitting is what makes that possible; inside SL-230 it was permanently
out of scope.

## The tradeoff accepted — R4, unmitigated in the interim

D4's rationale was that mandatory re-verification is *"affordable only because
the gate relaxation makes verifying cheap; the halves pay for each other."* The
split breaks that pairing: SL-230 ships invalidation against today's stricter
gate, so every claim-field edit costs a re-verify with no relaxation to offset
it. R4 is therefore carried **unmitigated** until SL-232 lands.

Accepted deliberately. R4 is friction, not incorrectness, and the state it
replaces has that same friction **plus** a stamp that survives a claim change —
observed live, a memory verified at `933b747c` holding `verified_sha` through a
committed body edit. The split strictly improves the status quo on correctness
and leaves the friction where it already was. Sequencing SL-232 next keeps the
window short.

## Consequences for RV-307

The ledger **stays attached to SL-230**. It is append-only and it reviewed *that*
document at that time; retro-pointing it at SL-232 would falsify the record.

Its still-live findings are disposed against SL-230 as **descoped** — not fixed,
not tolerated — naming SL-232 as the surface that inherits them. SL-232 opens its
own ledger when its design is authored, seeded from them. The distinction
matters: a descoped finding is unanswered work with a new owner, and must not
read as a resolved one.

## Relations

- SL-230 (narrowed), SL-232 (created), RV-307 (F-36, F-37, F-38, F-39).
- REV-034 — the SPEC-007 + REQ-147 amendment, re-pointed to SL-232.
- DEC-020 — non-contribution classification already left SL-230; this decision
  moves the surface that question lived on.
- IMP-317, IMP-318, QUE-173, QUE-175 — the deferred schema and dataflow work,
  all of which now has a home slice.
