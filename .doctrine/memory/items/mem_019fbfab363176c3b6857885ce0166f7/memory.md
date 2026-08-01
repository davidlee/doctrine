When a cheap-tier reviewer has produced findings, running it **again** is a
legitimate alternative to escalating to an expensive model. Nothing about an
expensive run is guaranteed either. But the two are not interchangeable, and the
difference is what you may conclude.

**Two passes from the same model are correlated samples. They buy variance, not
bias.**

- Where the model is merely *sloppy* on a file — skims it, misses a branch, talks
  itself out of something its own evidence showed — a second pass has a genuine
  chance of not repeating the error.
- Where the model *systematically* misreads a construct, every pass agrees. You
  then hold N corroborating reports of the same wrong thing, which is worse than
  one, because it feels checked.

So the inference is asymmetric: **disagreement is strong signal; agreement is
weak evidence.** A second pass is not a confirmation device. It is a triage
instrument — it tells you where the expensive attention should go, which is
exactly the job a top tier is too expensive to do across a whole range.

## Operational rules

- Launch the passes **concurrently**, so neither can anchor on the other.
- Make each pass report **what it checked and found clean, with the command that
  shows it clean** — not only its candidates. Divergence is computable only over
  the surface both actually covered, and false cleans hide there.
- **Do not average the passes.** Take the union of claims, and treat the
  disagreements as the work list. A finding only one pass raised is not weaker for
  that; it may be the pass that read the file properly.
- Beware a shared output path: if the prompt hardcodes where to write, the second
  pass silently overwrites the first.

## Measured (SL-233 S2, `RV-342`, four raisers over two phases)

- **Agreement was worthless.** Both passes on one phase converged on the same
  finding — which was not in the reviewed range at all. Only one pass checked
  scope. Correlated agreement on an out-of-scope claim.
- **Disagreement carried everything.** One pass killed the other headline
  candidate by testing scope; adjudication killed the second pass's headline
  (every fact true, conclusion wrong); and only one pass found the historical
  defect that turned out to be decisive.
- **Neither pass produced the major finding.** It came from reading the two
  against each other.

Detail and the full case record: `IMP-024` §4a and §4b.

See also [[mem.pattern.harness.grep-negative-needs-positive-control]] — the same
epistemic failure one level down, and the thing that most often makes a cheap
pass's *clean* verdict worthless.
