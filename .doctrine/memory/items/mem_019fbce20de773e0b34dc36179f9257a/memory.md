# A claim-withdrawal sweep is repo-wide or it is nothing

Evidence: RV-325 (SL-233), finding F-11 — a **blocker contested twice** across
three rounds, each time for the same reason at a wider radius.

A ruling withdrew a claim (`set` mode is assigned to PHASE-08). The withdrawal
was written into the artefacts that **argue** the ruling — the sketch, the
diagram, two knowledge records — and each round found another surface still
asserting the withdrawn claim as live:

| round | what was missed | why |
|---|---|---|
| 5 | DEC-101's body, PHASE-16 `EX-7`/`EX-14` | the ruling landed only in the artefacts that argue it |
| 6 | DEC-101's **TOML tier** — `knowledge show` emitted the live assignment beside its own withdrawal | the repair amended the prose tier only ([[mem.pattern.doctrine.amend-knowledge-both-tiers]]) |
| 7 | `src/design_run/runbook.rs`, `plan.md` | the sweep's **scope** was `plan.toml` + `sketches/` + the knowledge dirs; `src/` was never in it |

The round-6 sweep's *pattern* was fine — `runbook.rs:455` would have matched it.
Only the **scope** was wrong. Run repo-wide at round 7, the same pattern found
**three more** surfaces nobody had cited, two of them in files an earlier repair
had already edited: a *third* site in `runbook.rs`, and a restatement four
sections below the withdrawal block in the same sketch.

## The move

1. **Ask which record GOVERNS the ruling, not only which artefacts argue it.**
   Governing records — the accepted decision, the phase criteria, shipped source
   and its doc comments — are where a stale claim does damage. The sketch that
   argues it is the easiest to remember and the least load-bearing.
2. **Both tiers, in both directions.** [[mem.pattern.doctrine.amend-knowledge-both-tiers]]
   covers the common case (amendment lands in the `.md`, `[facet]` stays stale).
   The mirror case bites too: DEC-104 carried its entire reopening condition in
   the **TOML tier only**, so an author reading the `.md` sees nothing to amend
   and concludes the record is clean. Check with `doctrine <kind> show <ID>`
   grepped for the withdrawn phrase — that is the only view that spans both.
3. **Sweep AFTER repairing, and repo-wide**: `grep -rn` from the repo root minus
   `target/`. The repair is what tells you which phrase to grep for; the scope is
   the whole tree because a withdrawn claim reaches shipped source and doc
   comments, not just the design corpus.
4. **Grep the cousins.** A withdrawn claim survives in paraphrase — sweep each
   distinct clause of it separately, and remember a negative grep is worthless
   without a positive control.

## Why it recurs

A correct, verified repair *feels* complete, and the finding names only the
surfaces it happened to see. Treating the cited list as the extent of the class
is the error every round of this ledger repeated. Same failure in code cleanup:
[[mem.pattern.review.sweep-defect-class-not-instance]].
