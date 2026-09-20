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

Three shipped files carry the convention — two that already own this surface,
and the plan skill where the obligation fires a third time (added under `RV-371`
`F-4`). It rides them rather than minting a new normative owner:

- `install/design-prompts/reviewing.md` — the attack-surface fragment the design
  run delivers on **every reviewing turn**, i.e. at the moment routing happens.
  Precisely: the fragment is emitted every turn, and its *body* is elided when
  the caller declares a current `name@digest` receipt
  (`src/commands/design.rs:2837-2861`). So the agent either holds the current
  bytes or is re-sent them, and an edit invalidates every held receipt. That is
  a stronger guarantee than "re-sent each time", not a weaker one.
- `install/review-ledger.md` §4 (*Dispose + resolve*), which already publishes
  the disposition vocab the route sits alongside.

**`DEC-103` (*Instruction is delivered at the point of effect*) requires both
surfaces; it does not merely tolerate them.** Routing bites at two moments — when
the responder composes a disposition (the reviewing fragment) and when anyone
reads the disposition vocabulary (the ledger doc) — and `DEC-103` corollary 2
holds that *"an obligation firing at several moments is hung at EVERY one of
them, not demoted to prose"*, while corollary 1 holds that *"DRY is the wrong
model for agent instruction"*. Writing the convention once and referencing it
from the other surface is precisely the deletion `DEC-103` was written against.
This is the warrant; the trial-design argument below is support that reaches the
same place independently.

Delivery strength is also a deliberate choice, not convenience. `P10`'s first
trial question is whether agents can choose and execute the right route without
repeated human steering, and its *would-kill* list includes *"the routes need the
owner to interpret"*. A convention behind an **elective** fetch (a standard's
body, reachable only via `doctrine standard show` or `/canon`) makes a miss
unreadable — routing failed, or the rule was never read? Three ledgers cannot
absorb a delivery confound on top of `E11`'s own. Delivery is therefore held at
maximum strength so it does not confound the thing under test, and **the
delivery-channel question is deferred, not answered** (see *Follow-Ups*).

**The surfaces carry different text, written for different moments, and only
one of them asserts anything** (`DEC-268`, sharpened by `RV-371` `F-7`). `reviewing.md` holds the **operative rule** — it fires exactly where
routing happens and its `customization` is `fixed`, so no client install can
customise it away. `review-ledger.md` §4 holds the route axis alongside the
existing disposition vocab, the recording shape, a pointer to the fragment, and
an **explicit scope line**: provisional, design-review ledgers, under the
`RFC-026` trial. That scope line is load-bearing — that doc states in its own
header that it owns the *invariant* protocol shared by **every** review skill, so
an unqualified route axis there would bind audit and code-review ledgers too,
widening the intervention past `P10`'s trial population and making the counting
pass's denominator wrong. The `customization` asymmetry points the same way:
`reviewing.md` is `fixed`, `review-ledger.md` is `customizable`
(`publication/manifest.toml:254-261`, `:443-451`), so a client that customised
the ledger doc keeps the fragment and loses §4 — the rule must live where it
cannot be lost. `plugins/doctrine/skills/plan/SKILL.md` holds the third delivery:
a transcription pointer, likewise non-asserting, and likewise unlosable, since
`copy_skill` rewrites every skill file on each install
(`src/install.rs:2036-2038`). An earlier draft had §4 restate the closed route
set; that is the `P2` violation `RV-371` `F-7` caught, and the restatement is
gone — a sixth route is now a one-file edit.

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

**The convention applies to `blocker` and `major` findings only**; `minor` and
`nit` dispositions are unchanged (`DEC-267`). Without that boundary a responder
may route every severity, and the counting pass selects on severity, so the trial
would mix populations.

**Every severe finding carries an explicit route; there is no default**
(`DEC-265`). `review` is a legitimate explicit choice, not a fallback. Where the
responder genuinely cannot classify a finding, the convention adds no new escape
— it cites the guardrail that already ships at `install/review-ledger.md:164-166`
(*"Unresolved ambiguity after reading the design and governance → stop and
`/consult`. Do not improvise a disposition."*). A default to `review` would be
worse than a gap: `review` is the status quo prose loop, so a free default
silently nulls the intervention, and the counting pass could not tell a `review`
chosen on the merits from a route never chosen.

**The route set stays prose-only, and the design says so** (`DEC-266`). Every
other closed vocabulary here carries `parse` + `ALL` + lockstep tests
(`Severity`, `FindingStatus`, `Fragment`); this one cannot, because that is a
`src/` change and therefore the slice's own tripwire. Stated, not left as an
omission a reviewer finds.

No schema change, no new kind, no tooling: the counting method is a regex over
ledger output.

### 3. The raiser-side ruling (the gate teeth)

**Name the gate precisely — two exist, and they differ by exactly one state.**
The design run's `reviewing → locked` gate blocks only on blockers in `open` or
`contested` (`undisposed_blockers`, `src/review.rs:1705`). A routed finding is
`answered` from the moment it is disposed, so **it never held the design lock**:
`P10`'s *"the design gate may close with them open"* is already true today and
needs no ruling. The gate that does bind is the **slice close** —
`doc_unresolved_blockers` (`src/review.rs:1669`), which refuses `audit→reconcile`
and `reconcile→done` while a blocker is short of `verified`. The split is
deliberate, and `DEC-138` requires the asymmetry be stated wherever it is relied
on rather than left to read as a bug. The convention states it.

So the ruling this slice owes is the **raiser's**, at slice close — not a
dispensation for the design lock. Its resolution: a routed finding **is** disposed — `DEC-138` settles that
disposition binds the responder's turn, not the raiser's assent — so what stays
open is the *obligation*, not the ledger row. The convention states the raiser's
side: **for a routed finding, `verify` asserts that the obligation was correctly
transcribed onto a phase criterion — not that the defect is repaired** — and the
convention names that as a redefinition of an existing verb rather than letting
two readings coexist (`DEC-263`).

**That verify is deferred past the design lock.** Phases postdate the lock
(`design → locked`, then `slice plan`, then `slice phases`), so at design-review
time there is no phase to carry a criterion and the raiser cannot check one
exists. Nothing breaks: the lock needs the raiser's `conclude` marker and the
`review-disposed` act, and `review conclude` states in its own help that *"open
findings are fine: disposing them is the responder's work afterwards."* The
routed finding stays `answered` through the lock and is verified later, against
the slice close — which is the gate named above.

The weaker claim is safe rather than an escape hatch for three reasons. It is
**self-labelling**: the `route:` token sits in the same immutable field as the
status, so a reader sees `route:probe` and knows which claim that `verified`
carries. The evidential weight **transfers rather than evaporating**: a criterion
carries a `VT`/`VA`/`VH` mode and gates phase completion through machinery that
already exists — which is why `P10` said *criteria on a phase* and not *a note
somewhere*. And the redefinition is **stated**, so the trial's own counting is
not corrupted by two readings of `verified`.

Without this ruling the raiser's honest move is to contest, and the convention is
unimplementable without a code change — which would break the no-tooling
constraint outright.

**When a routed finding may stay open is a bounded question, and `P10` already
bounds it.** The convention carries its test rather than leaving *"may close with
the obligation open"* unbounded: a finding **must be settled first when the next
step adds external reliance, durable state, authority or exposure, dependency
spread, or governing meaning on top of the thing in doubt**; otherwise the
bounded next step may proceed with the finding named — measuring the cost to
regain an *accepted* state, not the cost to regenerate a diff. This is not
decoration. `P10`'s own *would-kill* list includes *"a protected boundary is
crossed while one is open"*, which is this test's violation condition; dropping
the test would silently remove a kill criterion from the trial.

**Accumulation reopens the design decision.** `P10`: *"when several of them
attack one mechanism, that changes the argument and reopens the design decision;
another test is not a disposition."* A second `demonstrate` or `probe` against a
mechanism that already carries one is not a route — it is evidence the design
decision itself is wrong. **The rule hangs on both parties** (`DEC-264`,
`DEC-103` corollary 2 applied to the slice's own headline warrant): the
responder's half, at disposition, is *do not route a second finding against a
mechanism that already carries one*; the raiser's half, at the reviewing turn, is
*contest rather than verify*. They are not redundant — the responder's half is
advisory and prevents the mess, the raiser's half has teeth, because `contest`
moves the finding to `contested` and `contested` blocks the design lock. **This
is the one clause of the convention that fires preventively.**

**Post-`verified`, accumulation cannot reopen the disposition.** No verb
transitions a finding out of `verified` (`src/review.rs:789-807`), so a verified
disposition is the immutable audit-time record. The remedy is a prose amendment
on the `RV` `.md` stating what changed and why, with the verified disposition
standing as the record of what was decided then. One sentence in the convention,
because it is the case a reader will hit and get wrong.

**Almost none of this is enforced, and that is recorded rather than implied.**
`DEC-103`'s residue clause requires an obligation with no locatable delivery
moment to stay prose *and be labelled unenforced-by-construction*, each item
carrying why. `CON-006` is that register: ten clauses, each with the code site
proving it. Two clauses have any mechanical force and they are asymmetric — the
raiser half of the accumulation rule, which fires preventively at the design
lock; and transcription, which is caught transitively at the slice close and is
therefore **audit-grade only**, arriving many agent sessions after the phase it
should have guarded has already run. It detects a dropped obligation; it does not
stop the defect going unguarded through execution. The convention text carries a
one-line pointer to `CON-006` on both surfaces — the *fact* of non-enforcement is
delivered at the point of effect; the enumeration stays pull-tier.

### 4. What each instrument route owes, and who states it

All of it goes in `--response` at disposition, by the responder.

- **`probe` — the adversary.** `P10` leaves this open; the rule this slice fixes
  is one sentence in the form *must hold against X, need not hold against Y*.
  Rejected: stating it in slice scope — too early (the adversary is not known at
  scope time) and too coarse, since `E11` records `RV-314` raising five separate
  git-configuration routes against one mechanism.
- **`control` — the named fault** (`DEC-267`). `P10`: *"name the concrete
  incorrect candidate the check must reject, and observe it rejected"*; a control
  establishes discrimination against a **named** fault, not completeness against
  all faults. The exact symmetric obligation to `probe`'s adversary, and its
  absence is `P10`'s would-kill *"a negative control is reported as rejected when
  it was not"*.
- **`owner-fix` — the sweep** (`DEC-267`). `P10`: *"remove the duplicate, verify
  the surviving owner, sweep the affected class."* The third clause is the one
  that is not implied by the route's own definition, and it is the twin-arm
  failure mode observed four times in one review (`RV-370`/`SL-246`), where a
  repair inherits the finding's scope and leaves its twin while reading as
  complete.
- **All three instrument routes — a criterion sketch and a placement
  constraint** (`DEC-271`). The responder cannot name a host phase: phases are
  devised at planning, well after disposition. What *is* knowable at disposition
  is what the criterion must assert, and what the obligation needs of its host
  (*"must run after the parser exists"*, *"needs a worktree branch"*). The
  placement decision belongs to whoever writes the plan, who reads both off the
  `RV` ledger they must already open in order to transcribe — so the plan-time
  half is delivered without a third normative surface. The planner places the
  obligation on the **earliest phase that satisfies the stated constraint**
  (`DEC-272`), first phase by default; how late it may go is bounded by the
  settle-first test above, and an obligation with no admissible host is a routing
  error handled by `contest`.

**Form, not just content** (`DEC-269`). All of the above is written as plain
prose with no code spans — no backticks, no `$` — and the responder reads the
stored value back with `review show --json` before moving on. `--response` is
free text passed through a shell, where a backtick span is command substitution:
the `dispose` succeeds, the receipt reads clean, and the quoted spans are stored
empty. The ledger is turn-based with no amend verb, so the damage is permanent
until the raiser contests. This is a pre-existing hazard that SL-260 makes
expensive, because `--response` becomes the recording site for the one fact
`P10` flagged *Open*. The rule dodges it rather than repairing it: a shell-quoting
recipe in a shipped reference doc would lean doctrine's guidance on the host's
shell, and the content rule is shell-agnostic.

The ledger-grain detector `P10` asks for is the counting pass itself — a
`probe`-routed finding whose response carries no adversary clause is counted as
such. **An eaten clause and an absent one count the same** (`DEC-270`): the trial
measures whether the convention produced a usable clause, not why it did not, and
attributing a miss is a blame question the trial does not ask. The residual risk
is confounding rather than detection, so the guard is on the analysis and belongs
to the trial-report chore (item 8): check for response truncation before reading
a high miss rate as a routing failure.

Every rule in this section is unenforced by construction — nothing parses
`--response`. See `CON-006`.

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

**No script.** The method is a documented command over existing JSON output,
verified by running it during the pre-design research round:

```sh
for rv in 365 368 370; do doctrine review show RV-$rv --json; done \
  | jq -r '.review as $r | $r.finding[]
      | select(.severity=="blocker" or .severity=="major")
      | [$r.id, .id, .severity,
         ((.disposition//"«none»")|split(" ")[0]),
         ((.disposition//"")|test("^route:"))] | @tsv'
```

One row per severe finding, carrying the route prefix as a tested boolean. The
`// ""` guard is load-bearing: an undisposed finding has a null disposition and
`split` fails without it. So the earlier *"a script, if one is needed"* hedge
resolves — **this slice adds no tooling at all**, which is the stronger
constraint and is kept as one (see *What this slice deletes*).

### 7. A committed home for probe evidence

One line, using what already exists. Under `.doctrine/slice/NNN/`, only
`research/`, `phases`, `handover.md` and `inquisition.md` are gitignored
(`.gitignore:47-50`) — **a sibling directory commits**. So a contained probe
script needs nothing but a name that is not `research/`. A codebase-wide probe
goes to a worktree branch via `doctrine worktree fork`.

**The name is `probes/`** (`DEC-273`) — created on demand, never eagerly. It
states one route rather than a category, so it cannot become a dumping ground for
evidence that belongs in the test suite, which is what `IMP-324`'s shrinkage
depends on; and it matches the route token literally, so `route:probe` → `probes/`
needs no mapping remembered. One trap to carry into the convention text: a file
named `handover.md` inside `probes/` would still be gitignored, because
`.gitignore:29` matches that name at any depth.

### 8. The trial-report hook

This slice closes before the trial runs, so nothing would otherwise own *"in
three slices' time, count and write the evidence entry"*. A backlog chore,
gated `after` this slice, carries that obligation.

### What this slice deletes

Named deliberately: the corpus's characteristic failure is additive-only
apparatus — review ledger `RV-353` found one programme's artefact was 61%
measurement apparatus.

- **No new normative owner.** No new entity kind, no schema change, no new
  standard. The convention is delivered at three moments and its normative
  content is owned by exactly one of them — the reviewing fragment; the other
  two surfaces point at it and assert nothing (`RV-371` `F-7`). An earlier draft of this scope proposed one and it was dropped: a
  standard alongside the shipped files is the two-surface posture `E11` measured
  at 24.5% artefact-prose findings, and `P2`'s own kill clause.
- **All tooling** — not *minimal* tooling, none. `STD-001` (magic strings),
  `STD-003` (no silent skip) and `POL-002` (platform independence) each scope to
  `src/` or to shipped reader behaviour, so the slice as scoped lands outside
  **those three**. **If this slice acquires a `src/` change, all three newly bind
  and the no-tooling claim has failed.** That is a tripwire, not a preference.
  It is not a claim that no standard binds: `STD-002` (short titles, ids not
  slugs) scopes to *"all authored doctrine entities … [and] every reference to
  an entity"* with no `src/` precondition, so it binds this slice's prose and
  records throughout. An earlier form of this bullet swept `STD-002` into the
  tripwire and read as though the slice were outside every standard; `RV-371`
  `F-1` corrected it.
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

- `install/design-prompts/reviewing.md` — the convention clause (the sole
  normative owner), and a provisional carve-out on the stand-alone rule at
  `:63-64` (`RV-371` `F-4`).
- `install/review-ledger.md` — §4 route axis alongside the disposition vocab,
  placed after the terminal-close block and deferring it for routed findings
  (`RV-371` `F-5`); non-asserting (`RV-371` `F-7`).
- `plugins/doctrine/skills/plan/SKILL.md` — the transcription pointer, added on
  the owner's ruling against `RV-371` `F-4`. Closes `OQ-3`/design `Q3`.
- `.doctrine/rfc/026/` — `P10`'s *Open* items resolve to items 4 and 5.

Shipped assets under `install/` and `plugins/` are embedded
(`src/asset_source.rs:21`, `src/install.rs:20`), so the convention reaches agents
only after `cargo build` then `doctrine install`. That
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
3. Each of these is stated in one place with no second copy: the eligibility
   rules; the recording shape; the severity boundary and the no-default rule; the
   raiser-side ruling and its deferral past the design lock; `P10`'s settle-first
   test; both halves of the accumulation rule and the post-`verified` residue; the
   per-route obligations (`probe` adversary, `control` named fault, `owner-fix`
   sweep, criterion sketch and placement constraint); the plain-prose form rule;
   the placement rule; the compound-finding precedence; and the collection
   procedure. **No exception.** The obligation is *hung* at three moments, per
   `DEC-103` corollary 2; its content is *owned* once, per `RFC-026` `P2`. The
   ledger entry and the plan pointer must each be checkable as non-asserting:
   remove the fragment and neither still tells you how to route (`RV-371`
   `F-7`). §4 of the ledger doc carries the design-review scope line.
4. `CON-006` exists and is cited from all three surfaces, so the fact of
   non-enforcement is delivered at the point of effect and the enumeration is
   reachable. Nothing in the convention implies an enforcement it does not have —
   in particular the slice close is described as a *forcing moment* on the verify
   act and never as a check on transcription (`RV-371` `F-2`, `F-3`).
5. The trial-report chore exists, is gated `after` this slice, and **cites**
   `DEC-276` for the collection procedure rather than restating it (item 3 binds
   that procedure too), carrying in its own right only the checklist line for
   the at-conclusion capture of `review status` (`RV-371` `F-8`).
6. `P10`'s *Open* clause in `RFC-026` records where its two items were settled.
7. No file under `src/` is touched.

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
- **Re-decide the convention's scope at trial conclusion** (owner, 2026-09-19).
  §4's copy is scoped to design-review ledgers for the trial. If the mechanisms
  are adopted, whether they extend to audit and code-review ledgers is an open
  decision, not an automatic consequence of a successful trial.
- **Code-backing the route set** — available if the trial validates the
  convention (`DEC-266`). Taking it changes the intervention and newly binds
  three standards; it is not licensed by a good trial result on its own.
- **Hanging the promotion obligation in the `/plan` surface** (`DEC-271`). The
  purest `DEC-103` reading, declined here on surface cost. This is the named next
  move if the trial observes routed obligations dropped between plan and
  execute — the gap the close gate detects too late to prevent.
