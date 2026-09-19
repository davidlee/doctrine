# Design-review finding routing convention and trial

## Context

`RFC-026` (*Design review response effectiveness*) gained proposal **`P10`** on
2026-09-19, built on its evidence entry **`E11`** — a single-rater classification
of 137 blocker and major findings across eleven design-review ledgers.

`E11`'s load-bearing result: **six severe findings in ten are mechanism
predictions** — the design asserted code-level detail that was wrong or
unestablished — and they split into *structural* (a thin implementation exposes
it) and *discriminating-input* (it compiles and passes ordinary tests; only a
hostile or edge-case input exposes it). Converged reviews were mostly structural;
non-convergent ones mostly discriminating-input. `E11` also found 31 findings
raised *against repair text* in the non-convergent group against 1 in the
comparison group.

`P10`'s reading: one prose loop is being asked to settle four different kinds of
question and is good at one. Its rule — at disposition, the author gives each
blocker or major finding **one route from a closed set** (`review`,
`demonstrate`, `probe`, `control`, `owner-fix`), and `demonstrate` / `probe` /
`control` findings are **not repaired in prose**. They become criteria on the
first phase, and the design gate may close with the obligation outstanding. That
is the mechanism aimed at repair-text findings: no repair text, no findings
against repair text.

This slice stands the convention up and fixes the trial's rules. It does **not**
run the trial — see *Non-Goals*.

### What the trial is, and what it is not

`P10` specifies the next three code-changing slices, taken consecutively under
eligibility rules fixed beforehand. It is a **combined intervention** — routing
overlaps with `RFC-026` `P2`'s sharpened delegation rule (*the design may argue,
it may not assert; a normative statement has one record and is included by
reference, never retyped*) — and **no causal separation is claimed**. Three
ledgers is a practical minimum for seeing the convention operate, not an
evidential threshold, so **there is no pass mark**.

## Scope & Objectives

### 1. The convention text, where disposition guidance already lives

Two shipped files already own this surface; the convention rides them rather than
minting a new normative surface:

- `install/design-prompts/reviewing.md` — the attack-surface fragment the design
  run delivers **every reviewing turn**, i.e. at the moment routing happens.
- `install/review-ledger.md` §4 (*Dispose + resolve*), which already publishes
  the disposition vocab the route sits alongside.

Delivery strength is a deliberate choice, not convenience. `P10`'s first trial
question is whether agents can choose and execute the right route without
repeated human steering, and its *would-kill* list includes *"the routes need the
owner to interpret"*. A convention behind an **elective** fetch (a standard's
body, reachable only via `doctrine standard show` or `/canon`) makes a miss
unreadable — routing failed, or the rule was never read? Three ledgers cannot
absorb a delivery confound on top of `E11`'s own. Delivery is therefore held at
maximum strength so it does not confound the thing under test, and **the
delivery-channel question is deferred, not answered** (see *Follow-Ups*).

The text marks itself provisional and cites `RFC-026`. Landing on `edge` is not
release; whether it ships to client installs is a separate decision the user
makes at tag time while the trial is live.

### 2. The recording shape

The route is **the one new fact**. `doctrine review dispose --disposition` is
free text (confirmed against `--help`), and `E11`'s parent entry `E1` found 61
distinct values in use, so nothing enforces the published vocab today.

The route and the existing vocab are **different axes** — the vocab records what
the responder did, the route records what instrument can settle the finding — so
both are preserved in the one field behind a fixed leading token:

```
--disposition "route:<route> <vocab>"     e.g.  route:probe fix-now
```

No schema change, no new kind, no tooling: the counting method is a regex over
ledger output.

### 3. The raiser-side ruling (the gate teeth)

`P10` says the design gate may close with a routed finding open. This needs an
explicit ruling, because `blocker` is the one severity that gates the target's
close and an unresolved blocker refuses the `audit→reconcile` transition.

The resolution: a routed finding **is** disposed — `DEC-138` settles that
disposition binds the responder's turn, not the raiser's assent — so what stays
open is the *obligation*, not the ledger row. The convention must therefore state
the raiser's side: **the raiser verifies a routed finding on the strength of the
criterion existing on the first phase, not on repair text, and the criterion must
be authored before the verify.**

Without that sentence the raiser's honest move is to contest, and the convention
is unimplementable without a code change — which would break the no-tooling
constraint outright.

### 4. Who states the adversary for a `probe`

`P10` leaves this open. The rule this slice fixes: **the responder states it in
`--response` at disposition**, one sentence in the form *must hold against X,
need not hold against Y*. The ledger-grain detector `P10` asks for is the
counting pass itself — a `probe`-routed finding whose response carries no
adversary clause is counted as such.

Rejected: stating it in slice scope. Too early (the adversary is not known at
scope time) and too coarse — `E11` records `RV-314` raising five separate
git-configuration routes against one mechanism.

### 5. Eligibility rules for the three trial slices

Fixed here, **before** the next code-changing slice starts, or selection bias
returns:

- The **next three code-changing slices by id, consecutively**, from this slice's
  landing commit.
- **Parked or abandoned slices stay in the count.**
- A ledger with **zero** severe findings is a legitimate datum, not an exclusion.
  Selecting on finding count would reintroduce exactly the bias `E11`'s
  hand-picked group already carries.
- `SL-260` itself is excluded by construction: the convention does not exist
  during its own design.

### 6. The counting method

Fixed here so the trial's end is mechanical rather than a fresh judgement call.
Per severe finding: the route; whether the instrument produced the evidence it
promised; whether the repair was later contested or drew a related finding. Per
ledger, **as counts and not shares**: artefact-prose findings, findings against
repair text, repeat contests, rounds, design line growth during review, later
audit findings, and whether the slice completed. Baseline is `E11`; slice scope
differs, so these are **not comparable defect rates**. A second rater
re-classifies.

Prefer a documented command over a script. A script, if one is needed, is the
only tooling this slice may add.

### 7. A committed home for probe evidence

One line, using what already exists. Under `.doctrine/slice/NNN/`, only
`research/`, `phases`, `handover.md` and `inquisition.md` are gitignored
(`.gitignore:47-50`) — **a sibling directory commits**. So a contained probe
script needs nothing but a name that is not `research/`. A codebase-wide probe
goes to a worktree branch via `doctrine worktree fork`.

### 8. The trial-report hook

This slice closes before the trial runs, so nothing would otherwise own *"in
three slices' time, count and write the evidence entry"*. A backlog chore,
gated `after` this slice, carries that obligation.

### What this slice deletes

Named deliberately: the corpus's characteristic failure is additive-only
apparatus — review ledger `RV-353` found one programme's artefact was 61%
measurement apparatus.

- **No new normative surface.** No new entity kind, no schema change, no new
  standard. An earlier draft of this scope proposed one and it was dropped: a
  standard alongside the shipped files is the two-surface posture `E11` measured
  at 24.5% artefact-prose findings, and `P2`'s own kill clause.
- **Prose repair for three of the five routes**, which is the point. Measurable
  as design line growth during review.
- **`IMP-324`** (*No durable sink for design-round probe evidence*) shrinks to
  item 7 above. For `demonstrate` and `control` the evidence becomes an ordinary
  test in the repo, which answers `IMP-324` for those routes outright; only a
  throwaway `probe` script needed a home, and it already had one.

## Non-Goals

- **Running the trial.** Its observations land later as a new `RFC-026` evidence
  entry, via the chore in item 8.
- **`IMP-463`** (*Measure design-review cost per stage*) — a sibling, not a part.
  `P10` already concedes the economic claim stays untested until something
  measures cost. Folding it in doubles the slice and couples a convention to a
  metrics script, against the `RV-353` guard above.
- **Any causal claim.** Combined intervention, no separation.
- **Release.** Landing on `edge` is not shipping to client installs.
- **A `reviewing.toml` step.** That file's header states the step-id-is-API
  contract; a new `[[step]]` is a versioned surface change for something the
  per-turn `reviewing.md` fragment already delivers.
- **Reconciling the 61-value disposition vocab** (`E1`). Real, and out of scope.
- **`RFC-027`'s adaptation envelope**, deferred; and `SL-258`, which is not the
  trial vehicle — it has no code, so it produces no mechanism findings.

## Affected surface

- `install/design-prompts/reviewing.md` — the convention clause.
- `install/review-ledger.md` — §4 route axis alongside the disposition vocab.
- `.doctrine/rfc/026/` — `P10`'s *Open* items resolve to items 4 and 5.
- `scripts/` — only if the counting method needs more than a documented command.

Shipped assets under `install/` are embedded (`src/asset_source.rs`), so the
convention reaches agents only after `cargo build` then `doctrine install`. That
step belongs in the landing procedure — a skipped rebuild leaves the convention
silently not in effect. (`cargo` does register the asset folder as a build
dependency, so the rebuild is not itself manual; see
`mem.pattern.build.rust-embed-no-rerun`.)

## Risks, assumptions, open questions

Doc-local, `SL-260`-scoped.

- **`R1` — the convention is read and still not followed.** That is a real trial
  result, not a defect, but it is only readable *because* delivery was held at
  maximum strength. If the text lands anywhere weaker mid-trial, the result
  becomes uninterpretable.
- **`R2` — three ledgers is a thin sample carrying a combined intervention.** No
  pass mark exists for exactly this reason. The trial answers *can it operate*,
  not *does it work*.
- **`R3` — the route vocabulary needs the owner to interpret.** One of `P10`'s
  own would-kill conditions. Observable at ledger grain during the trial.
- **`A1`** — design-run findings are already on the `RV` ledger (`SL-244` mints
  the pass on entry to `reviewing`; the blocking-findings derivation sources from
  the observed `RV`). So *"on the existing ledger"* is true today and
  `IMP-392` is not a prerequisite.
- **`A2`** — `--disposition` accepts free text and nothing validates it.
- **`OQ-1`** — should the convention name a default route for a finding the
  responder cannot classify, or require an explicit `review`? A default risks
  becoming the escape hatch `review-ledger.md`'s anti-escape guardrails already
  fight; requiring explicit choice risks stalling. Design's call.

## Verification / closure intent

Done is judged on the convention being **in effect and mechanically checkable**,
not on trial outcomes, which postdate this slice:

1. The convention text is present in both files, marked provisional, citing
   `RFC-026`.
2. A rebuilt-and-installed tree delivers it — verified through rendered output,
   not through the source file (a stale embed is silent).
3. The eligibility rules, the recording shape, the raiser-side ruling, the
   adversary rule, and the counting method are each stated in one place with no
   second copy.
4. The trial-report chore exists and is gated `after` this slice.
5. `P10`'s *Open* clause in `RFC-026` records where its two items were settled.

## Summary

<!-- Written at close. -->

## Follow-Ups

- **The delivery-channel question**, deliberately deferred: the trial tests the
  routes with delivery held at maximum strength, so it says nothing about whether
  the convention survives as a weaker-delivery artefact. Promoting it later to a
  standard or a project-local governance entry **changes the intervention** and
  must not be done silently on the strength of this trial.
- **Release decision** — whether the provisional convention ships to client
  installs, made at tag time while the trial is live.
- **`IMP-463`** — the cost instrument, sibling.
