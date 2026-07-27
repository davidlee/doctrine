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
fingerprint of `design.md` as Doctrine last left it — beside the revision.
Every mutating verb fingerprints the authored file on entry and refuses when it
diverges from the watermark, then re-fingerprints immediately before writing and
refuses rather than writes when the bytes moved during the invocation.

The two windows are not redundant. The entry check catches an edit that landed
before the invocation; the pre-write check catches one that landed during it.
`src/review.rs::with_turn` runs exactly this protocol for the review ledger
(entry CAS at step 3, pre-write CAS at step 5, its crash window tested through
an injectable mid-turn hook), so this is a copied and tested shape rather than a
third bespoke state-model mechanism.

Two boundary rules keep it honest: an absent `design.md` before first
materialisation is *cold*, not divergent; and explicit re-adopt is the only path
that re-baselines the watermark — no verb silently adopts a foreign edit.

Raised as F-19 on RV-315 from plan-grounding research
(`research/raw/plan-review-ledger-analogue.md`), which recommended the
two-window pattern explicitly. Implementation belongs to the persistence phase
alongside the revision CAS and submission idempotency.
