# IMP-024: Large-review funnel: fan parallel raisers into one RV ledger (ADR-007 Neutral seam, composes ADR-006 spatial + ADR-007 temporal)

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Operational notes — RV-324, run by hand 2026-07-30

A large review was fanned across eight parallel raisers into one ledger, by hand,
with no funnel support. It worked. These are the observations a design should
answer to, not a proposed design.

**Shape that worked.** Eight raisers over ~4.2k changed lines (19 files), two
model tiers, one RV. Five mechanical *census* passes and two *review* buckets on
a cheap model; one adversarial pass on a top-shelf model; the orchestrator raised
every finding serially. Six findings survived to the ledger — 2 blocker, 2 major,
2 minor.

### 1. The split that made a cheap tier safe

The load-bearing rule was **give the cheap tier only work whose output can be
mechanically re-verified.** Not "give it the easy parts" — give it the parts
where being wrong is *detectable*. Enumeration, census, and correspondence tables
qualify; "is this subtly wrong" does not.

Two enforcement details made it hold:

- every claim ships **the command that reproduces it**; a claim without one is
  discarded unread;
- every negative result ships **a positive control**. This is not ceremony — in
  the prior session on this code, four of four reported zero-hit results were
  wrong until a control was added. The cheap tier followed it and self-caught.

### 2. Adjudication is a distinct stage, and it earned its keep

The cheap tier produced a confident, well-evidenced finding that
`ApplyRequest::writer_act()` omitted a field and nothing enforced the
correspondence. Every fact in it was true. The *conclusion* was wrong — the
top-shelf pass rejected it with reasoning (`writer_act()` is consulted only when
`act.is_proposal()`, so coordinator acts may lawfully coexist with coordinator
declarations). Only the test-coverage half survived.

A funnel that let raisers write straight to the ledger would have landed that as
a finding and cost a disposition round. **Evidence and findings are different
kinds, and the seam between them is where the funnel's value is.** Cheap raisers
should emit *candidate* findings into a staging tier; promotion to the ledger is
a judgement act.

Corollary: the top-shelf pass should receive the cheap tier's tables explicitly
framed as *evidence to verify, not conclusions to trust*. That framing was in the
prompt and the model used it exactly as intended.

### 3. Serial raising, parallel finding

Findings were raised by one writer, serially, after all raisers returned. This
was deliberate and is worth preserving: the ledger is append-only with derived
status, and N concurrent `review raise` calls contend on it. It also puts dedup
in one place — all six findings had to be checked against 32 already logged
during execution, plus a disclosed-and-adjudicated set. **Dedup is not
parallelisable**; it is inherently a whole-set operation.

This maps cleanly onto ADR-006's sole-writer posture: raisers fan out spatially,
the ledger write is funnelled.

### 4. Redundancy is cheap and it corroborated

Two independent cheap raisers hit the same `WRITER_ACTS` gap and graded it
differently (minor / nit). Independent convergence on a finding is a real
confidence signal a funnel could compute for free. Divergent *severity* on a
converged finding is also signal — it marks the ones a human should grade.

#### 4a. A second cheap pass is a real tier — but read its output backwards

Amended 2026-08-02 during the SL-233 campaign, from the owner's ruling: a second
cheap pass is a *legitimate* alternative to escalating, not merely a fallback.
Nothing about an expensive model's output is guaranteed either. But the two are
not interchangeable, and the difference is what you can conclude:

**Two passes from the same model are correlated samples. They buy you variance,
not bias.**

- Where the model is merely *sloppy* on a given file — skims it, misses a branch,
  talks itself out of something — a second pass has a genuine chance of not
  repeating the error. This is the class redundancy catches, and it is not a
  small class.
- Where the model *systematically* misreads a construct, every pass agrees. You
  now hold N corroborating reports of the same wrong thing, which is worse than
  one report, because it feels checked.

So the inference is asymmetric, and §4's framing above understates it:
**disagreement is strong signal; agreement is weak evidence.** A second pass is
therefore not a confirmation device. It is a **triage instrument** — its job is
to say *where the expensive attention should go*, which is precisely the job a
top tier is too expensive to do across a whole range. Run it to find the seams,
then spend the top tier only there.

The SL-233 S1 session supplies a case squarely in the catchable class. The
PHASE-08 raiser marked `RV-325` F-17 HONOURED while **its own positive control**
showed `exploring.toml` carrying zero completion conditions; the excuse it
constructed ("authored before the gate") did not survive reading the ruling. That
row, overturned by hand, became `RV-341` F-1 — the campaign's only major finding
to that point. A model contradicting evidence it had itself produced is a
variance failure, not a bias failure, and is exactly what a second independent
pass has a fair chance of catching.

Two operational consequences:

- **Ask each pass to report what it checked and found *clean*, with the command
  that shows it clean** — not just its candidates. Divergence is only computable
  over the surface both passes actually covered, and the F-17 case lived in a
  clean verdict, not in a candidate.
- **Do not average the passes.** Take the union of their claims and the
  *disagreements* as the work list. A finding only one pass raised is not thereby
  weaker; it may simply be the pass that read the file properly.

#### 4b. Measured: SL-233 S2, two phases, two passes each

Run 2026-08-02 on `RV-342`. Four raisers, same model, same prompts, same range;
the two passes launched concurrently so neither could anchor on the other. The
result is a cleaner argument for §4a than the theory was.

**Agreement was worthless.** Both PHASE-04 passes independently raised the same
`serde_json::to_string(declaration).unwrap_or_default()` at `commands/design.rs:836`.
Convergence — and both were wrong about what mattered: the line is **not in the
PHASE-04 range at all**. Only pass B checked, with
`git diff <range> -- <file> | grep`, and said so. Pass A raised it as its
headline candidate without ever testing scope. Two correlated samples agreeing
on an out-of-scope finding is precisely the false corroboration §4a warns about.

**Disagreement carried everything.**

- Pass B killed pass A's headline candidate (the scope check above).
- Adjudication killed pass B's headline candidate: `TraversalDeclaration::is_empty()`
  ignores `authority` while gating `skip_serializing_if`, which pass B called
  round-trip data loss. Every fact was true. Nothing serializes an `ApplyRequest`,
  and `direct_traversal` consults `authority` only inside the pin / cursor /
  posture arms, so an authority-only declaration asserts nothing and `is_empty()`
  is correct by design. §2's case, reproduced exactly.
- Pass B alone found that PHASE-16 shipped a real three-way bound disagreement
  (validator 64, declaration `Token` 32, constructor `label` 16), already repaired
  two phases later. Pass A missed it; pass B rated it `blocker` while itself
  noting it was fixed, which is the wrong severity for an audit of what ships.

**Neither pass produced the finding that mattered.** `RV-342` F-1 — that nothing
verifies declaration against construction, and the test that appears to is
tautological — came from reading the two passes *against each other*: pass B's
historical instance supplied the demonstrated escape for a structural gap pass A
had walked past and called clean. The second pass did not confirm anything. It
changed where the expensive attention went, which is the whole claim of §4a.

**Net.** Of five candidates across the four raisers, two were out of scope or
refuted, two survived as `F-2` / `F-3`, and the major finding was synthesised
rather than reported. Cost: four cheap runs, ~26 minutes wall-clock, fully
parallel. Cheap enough that not doing it is the expensive choice.

#### 4c. Corrected at S3: the "worthless agreement" was a mis-attribution

Amended 2026-08-02 after SL-233 S3. §4b's headline example needs one fact added,
and it sharpens the lesson rather than weakening it.

The `serde_json::to_string(declaration).unwrap_or_default()` line that both
PHASE-04 passes converged on **was a real defect**. It is simply not PHASE-04's:
it is added by **PHASE-05**, and both S3 PHASE-05 passes raised it independently
and in range. It is now `RV-342` F-6.

So the correct reading of that convergence is not *two passes agreed on a
non-finding*. It is:

- **the scope check was right and load-bearing** — pass B was correct that the
  line is not in the PHASE-04 range, and discarding it there was correct;
- **but "out of my range" is not "not a defect"**, and S2 treated it as though
  it were. A candidate ruled out of scope should be *reassigned*, not dropped.
  Nothing in the S2 rig had anywhere to put it.
- **agreement was still weak evidence** — both passes agreed on the wrong
  *phase*, which is exactly the systematic-misread class §4a describes. They
  shared the error; only the scope test caught it.

Two operational consequences, both cheap:

- **Keep a cross-session out-of-scope spool.** When a pass raises something real
  but out of range, park it against the phase whose range does contain it. S3
  recovered this one only because the same line resurfaced under a different
  phase; had PHASE-05 been reviewed before PHASE-04, it would have been lost.
- **Phase-scoped ranges plus a mandatory scope gate are what located it.** The
  S3 template requires every candidate to ship the
  `git diff <range> -- <file>` proving its line is in range. That is what turned
  a discarded S2 candidate into a raised S3 finding.

#### 4d. Proving a test pins nothing: mutate, then control the mutation

S3 confirmed `RV-342` F-5 — an assertion satisfied by a substring of an
unrelated line — by **deleting the behaviour the test names and watching the
test still pass**. That is the only evidence that settles a "this guard cannot
fail" claim; argument from reading is not enough, and this campaign has twice
had confident readings turn out wrong.

The half that is easy to skip: **a passing mutant proves nothing until you show
the harness can fail.** A build that silently did not rebuild, a filter that
matched no tests, a binary the e2e spawns from a stale path — all present as a
green run. So follow the mutation with a *deliberately fatal* second mutation
and confirm it goes red. Both mutations together cost one extra compile and
convert an argument into a measurement.

This is the same discipline as the positive-control rule for negative greps,
applied to a negative claim about a test.

### 5. What a funnel must supply that hand-running did not

- **A staging tier.** Raisers wrote findings files to a scratch directory; the
  ledger was written later by hand. That intermediate is the missing kind.
- **A dedup corpus.** The single highest-ROI pass compressed 81KB of phase sheets
  into 36 lines of prior findings that every downstream raiser reused. A funnel
  should build this once and inject it, not hope each raiser reads the sheets.
- **Cost asymmetry as a first-class input.** The budget constraint was *number of
  top-shelf invocations*, not context. One call did two buckets plus adjudication
  because the cheap tier had pre-computed its enumeration. Routing should be by
  verifiability and invocation cost, not by surface size.
- **Confinement.** Raisers are read-only by nature. Dropping the worktree fork
  (nothing is written) removed the entire ISS-034 wrong-base hazard class. A
  review funnel does **not** need `/dispatch`'s worktree machinery, and should not
  inherit it — see `scripts/pi-review.sh`.

### 6. Sharp edges hit while running it

- Log size is not a progress signal. pi's rpc stream re-serializes accumulated
  state per event (50–150MB/turn is normal); reading that as a runaway cost four
  in-flight raisers. A funnel must surface progress as typed events.
- Orphaned raiser processes block a `wait` on the spawner, so completion reports
  arrive late and look like "still running".
- Both are recorded as friction observations, and both are fixed in
  `scripts/pi-review.sh` rather than worked around.

### Links

- Live instance: **RV-324** (SL-233 PHASE-11/12/10 implementation review), ledger
  `## Brief` carries the method and the tier split.
- Spawn tool: `scripts/pi-review.sh` — confined, read-only, no worktree fork.
- Id allocation across trees while a coordination branch is live: **ISS-279**.
