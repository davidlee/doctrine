# DEC-092: Authored watermark guards the authored tier

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

The SL-233 design run holds two truth tiers and until now guarded only one of
them at write time. DEC-059's monotonic revision lives inside the gitignored
runtime snapshot and compare-and-swaps runtime writers against each other; a
user editing authored `design.md` never touches it, so it cannot move. DEC-072
records materialise's output fingerprint and refuses to overwrite a foreign
edit, which is correct but scoped to one verb. DEC-066 binds evidence to
section fingerprints, which answers *which clearance died* rather than *may I
write at all*.

The gap is the interval between two applies. A hand-edit landing there is
invisible to the run: stages advance and gates clear against prose that no
longer says what the snapshot believes it says, and nothing surfaces until
someone materialises.

The runtime snapshot therefore carries an **authored watermark** — the
fingerprint of `design.md` as Doctrine last left it — beside the revision,
under three rules that are deliberately not one rule.

**1. Ordinary mutation entry-refuses divergence.** A mutating verb fingerprints
the authored file on entry and refuses when it differs from the watermark,
rather than clearing another gate against prose the snapshot no longer
describes. An absent `design.md` before first materialisation is *cold*; absent
after it is divergent.

**2. `adopt_authored` is the sole lawful crossing, and it is a protocol, not a
bypass.** Re-adoption exists precisely because the bytes diverged, so a
universal entry refusal would make it self-refusing and would re-wedge the
live-run recovery path that RV-315 F-8 verified. The exception is therefore
specified rather than assumed: the caller declares the exact current
fingerprint and it must match what Doctrine reads; the complete stable-marker
map must validate; affected evidence is invalidated under DEC-066 with no
clearance inherited across the crossing; and the watermark re-baselines only
after the candidate validates in full. An invalid or stale adoption changes
neither runtime clearance nor the watermark. An informal "re-adopt bypasses the
guard" would let a caller launder arbitrary foreign bytes, which is the failure
this record exists to prevent.

**3. The pre-write re-check narrows the window; it does not close it.**
Immediately before the snapshot write the verb re-reads and re-fingerprints,
and abandons the write on divergence. Doctrine holds no lock against a human
editor, so an edit landing between that comparison and the atomic rename is not
caught by *this* invocation — it is caught by rule 1 at the next one. The
guarantee is delayed detection, never silent acceptance.

Rule 3's guarantee is further bounded by effect ordering, and the bound is
stated rather than papered over. For a checkpoint-bearing `apply`, DEC-083 and
DEC-086 order the journal, the reserved canonical ID, and the materialised
record *before* the design snapshot, and authored knowledge is never rolled back
to repair a runtime failure. An abandoned write therefore promises that the run
does not advance — no snapshot, no stage or gate movement — not that nothing was
written. Journalled effects remain and are recoverable without duplication.

`src/review.rs::with_turn` runs the same entry-then-pre-write comparison for the
review ledger (entry CAS at step 3, pre-write CAS at step 5, its window tested
through an injectable mid-turn hook). The shape is borrowed, but the borrowing
is partial and the differences are exactly why rules 2 and 3 read as they do:
`with_turn` holds a writer lock and hashes the very file it atomically replaces,
whereas a design run hashes `design.md` while writing a runtime snapshot, a
checkpoint journal, and possibly an authored record. Claiming `with_turn`'s
stronger same-file guarantee for a cross-file, multi-effect operation would be
a borrowed word, not a borrowed mechanism.

Raised as F-19 on RV-315 from plan-grounding research
(`research/raw/plan-review-ledger-analogue.md`), which recommended the
two-window pattern explicitly. The first formulation stated rule 1 universally
and inherited `with_turn`'s "nothing written" wording; the raiser contested it
on both counts and raised F-20 for the regression of F-8. Rules 2 and 3, and
the effect-ordering bound, are that contest's product. Implementation belongs to
the persistence phase alongside the revision CAS and submission idempotency.
