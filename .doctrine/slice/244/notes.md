# Notes SL-244: Gate conditions carry their own contract

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Design surface triage
<!-- exploring/explore.triage, 2026-08-03. Detail lives in slice-244.md (scope,
     OQ-1..OQ-8) and research/research.md (threads, deltas). This is the ordering
     judgement over them, not a restatement. -->

**Constraining governance** — `DEC-101` (open→closed narrowing is a type error;
constrains satisfaction *sourcing* only), `DEC-102` (seal when an override would
make content false), `DEC-066`/`DEC-067` (evidence liveness, cumulative
revalidation), `STD-001`, `SPEC-029` (owns the gate table and describes evidence
as payload-claimed — the certain revision candidate), `ADR-001` (layering).
Checked not-applicable with reasons: research.md § Thread 1.

**The one structural fact the design turns on.** `Condition` is payload-free, and
that has already cost the codebase a refusal variant: `RunbookNotDischarged`
exists because `GateNotCleared`'s `Vec<Condition>` has *"nowhere for a step
identity to ride"* (`refusal.rs:166-170`). The premise is in-tree and argued by
the incumbent, not asserted by this slice.

**Shaping decisions, in the order they unlock each other.**

1. **What the engine should check** (`OQ-2`/`OQ-3`). Everything else is
   downstream. Research finding 1 established that the seal/craft line, the
   derived/claimed line, and `ISS-285`'s fork are all restatements of *does the
   engine check this?* — so `OQ-6` sequences nothing and must be dropped as a
   simplifier.
2. **Where the contract lives** (`OQ-1`/`OQ-4`) — Rust-side data vs prose
   correspondence. Now decidable on cost: prose needs no enum change but inherits
   `IMP-372` the moment overridability is claimed; Rust data has the
   `boundary_conditions` precedent but costs `Condition` its fieldless
   `Copy`/`Ord`/serde shape at every match site.
3. **Which channel carries it** (`OQ-7`) — splits by channel, not content. The
   refusal has no byte budget; the envelope has a hard one with `clearances`
   already riding it uncapped; the `Fragment` receipt is a third, amortised
   register (`commands/design.rs:1832-1851`).
4. **Subject rule** (`OQ-5`, `ISS-286`) — plausibly separable, and entangled with
   three inconsistent fixture conventions across four e2e suites.

**Risks carried into the design.** Snapshot versioning costs the one live run
(SL-243's, gitignored tier). Envelope byte budget. Three prose systems already
have three loaders — a fourth needs an argument. `is_derived()` asymmetry is
`IMP-361`'s known deferred gap, not a discovery. CHR-049 is one moderated run:
adequate to inform `ISS-285`'s deferred choice, not to settle it alone.

**Assumption.** `DEC-102`'s override seam does not exist; the available move is
embedded-and-`fixed`-with-a-citation, the pattern runbooks already use.

**Applied practice.** `mem.pattern.design.classify-at-authoring-not-from-behaviour`
(state the property in the artefact, don't infer it from a run) and
`mem_019faca1f05277729cb407f8d4487206` (ratify the incumbent before specifying a
format — here the kebab token, load-bearing four ways, and the two prose stores).

## Drafting entered — handover state (2026-08-03, superseding the exploring note)

Run `dr-019fc4dd-e049-7db0-8cea-9af8ff970810`, revision 36, stage **`drafting`**.
**15 inquiry nodes, 0 open, 15 resolved; exploring and inquiring runbooks both
discharged.** Re-enter with `doctrine design resume 244`.

**What the exploring→inquiring crossing cost, and why it is a result.** The edge
could not be cleared as written: evidence resolves its subject only against
`design.md` sections (`run.rs:1471-1478`) and the run had none, sections being a
drafting artefact. Two sections were authored to pass it — `sec-1` governing
context, `sec-2` concerns — held to citation-plus-judgement to narrow the
duplication. `EVD-012` records the whole thing, including the user's assessment
that forcing canonical entity content into prose violates single-source-of-truth
and is poorly conceived independent of that. It is bound into the run at `inq-14`
→ `cp-14`, so `resume` carries it. `SL-244`'s `OQ-5` (does a contract-carrying
condition subsume `ISS-286`'s subject rule?) is answered **yes, as a consequence
of `DEC-121`**, not as a separate repair.

**`explore.research` regressed and was right to.** Objective 6 entered scope
after the research baseline was stamped. Research thread 4 (documentation
surfaces) closed the gap before the restamp; its finding drove `DEC-127`.

**Nine decisions now, not seven.** `DEC-120`…`DEC-126` as before, plus:
`DEC-127` (objective 6 ships published **and** generated — a client repo cannot
read this repo's spec, so thread 4's spec-sibling champion is refused; the
citation direction inverts and `SPEC-029` cites the published address).

**Spawned since:** `ISS-309` (shipped assets cite repo-private ids that collide
in client repos — the id half resolves to a *different* record in a client repo
rather than dangling), `IMP-393` (`show` is reader-facing for every kind except
`design`, where it is the writer's turn envelope), `IMP-394` (collaborator
orientation asset), `IMP-395` (skills hand collaborators their literal
invocation and binary).

**Scope is reconciled.** `slice-244.md` carries an answer key for `OQ-1`…`OQ-9`,
all dispositioned. `OQ-6` is recorded as **dropped, not answered**.

This section holds only what the run cannot carry.

**Seven decisions, in dependency order.** `DEC-120` (kinds: derived / attested /
claimed, plus the 2026-08-03 sharpening that Attested names *input provenance*,
not a second way of deciding) → `DEC-121` (the exploring pair becomes attested
user checkpoints) → `DEC-122` (contracts ship as embedded prose, `fixed`) →
`DEC-123` (structure rides a const table, prose stays narrative) → `DEC-124`
(refusal carries the remedy, stage-entry receipt carries the contract, no digest)
→ `DEC-125` (findings unify on `RV` with a section reference) → `DEC-126` (what
the gate should check — the classification, and the answer to the slice's root
question).

**Spawned, all with contracts already written:** `IMP-391` (build the
exploring-stage checkpoints), `IMP-392` (the `RV` finding migration), `IDE-047`
(structure extracted from contract prose).

**The one thing a drafting agent must not re-litigate.** `DEC-126` is specified
against the `RV`-backed finding model, not the runtime `Finding`. Writing
contracts against a record `DEC-125` deletes is the mistake `DEC-121` caught on
the other edge.

**Where the design will be hardest** (`DEC-125`): a section's fingerprint moves,
`DEC-066` invalidation is snapshot-internal, and `RV` has no fingerprint concept.
The `design materialise` authored-watermark pattern is the shape to follow.

**Interim states this slice knowingly ships**, both stated rather than accidental:
`exploring → inquiring` passes on the runbook alone until `IMP-391`; the
`Conducted { review }` arm and the severity summary are unbuildable until
`IMP-392`.

**Still owed by the slice:** objective 6 — now specified by `DEC-127` as a
**published, generated** artefact: source under `install/` (already grafted, so
no `flake.nix` change), a `publication/manifest.toml` entry, and a golden test
pinning the render to `gate.rs`'s tables, following
`.doctrine/spec/tech/021/funnel-machine.md`. This will be the first diagram in
the shipped corpus, so the hand-edit-vs-generated policy is net-new.

Two things belong in it that appear in no shipped guidance and that an agent
currently discovers by being refused: that a productive integrated pass
invalidates its own clearance under whole-map currency, and that review
nevertheless terminates when the user declines another round (`RFC-026` E3), so
staleness informs that decision rather than barring it.

**Correction carried forward:** an earlier turn claimed the missing
promotion leg was captured as `ISS-303`. It was not — nothing was created, and
`ISS-303` is an unrelated existing issue. `DEC-125` supersedes that framing
entirely (unification dissolves the promotion leg), so nothing is outstanding.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-08-05 · PHASE-05 completed · eff3c8e3

### Produced

- **PHASE-05 COMPLETE — the condition model.** Fourteen tasks, in `D4`'s order
  (`T9` ahead of `T5`–`T8`, because both resolve `ActKind → ActRequirement` and
  the generated `CONTRACTS` table is the only route). The movement: one
  `macro_rules!` takes the nine-row vocabulary once and emits `Condition`,
  `Condition::ALL`, `CONTRACTS` and `boundary_conditions` plus a per-row const
  assertion (`3ed419d1`, `902a3139`); the eight acts that table names persist in
  two new snapshot groups replaced **by act** (`a91d04a2`), with wire types that
  carry the claim and nothing the engine fills (`fbb4685e`); admission owns the
  rule/record correspondence and raises every fault rather than the first
  (`d8fdc1ec`, `85526479`); `apply` runs admit → mutate → re-observe →
  construct → evaluate (`99c93cf3`); the shell projects the governance edge set
  (`8c76d2a8`); the fixtures acquire their acts while the incumbent scan still
  runs (`56b6df0c`, `b129d5ca`); `satisfied` returns a diagnosis instead of a
  boolean (`6087abe2`, `2d93b629`, `00494bce`); `Evidence` and its five
  dependents retire and two feeds re-source (`3c338165`); the change log carries
  the disposition's arm (`33968c29`); the ladder runs on acts end to end
  (`29437777`); and close-out (`ed4f0764`, `eff3c8e3`).
  **Every PHASE-05 `VT` row PASSes**; `doctrine check gate` exit 0; clippy clean
  at every commit. Measured at close: whole binary 4330 green,
  `design_run::` 121, `e2e_design_review` 138, `e2e_design_state` 163,
  `e2e_design_delegation` 128.
  Findings `F1`–`F63` on the sheet. The three agent criteria each found
  something and that is the argument for running them — `VA-1` a task whose e2e
  edits were never tabled (`F61`), `VA-2` one second spelling and one orphan to
  adjudicate (`F62`), `VA-3` a clause the plan said lands live and which did not
  until close-out wrote it (`F63`). A fourth close-out read — conformance —
  found five code paths this slice edited that no *design-target* selector
  claimed, `src/design_run/**`'s scope-relevant row being invisible to that map
  (`F64`, declared at `7973b45c`); conformance now reports no undeclared code
  path. Four durable memories recorded (the
  clippy-skips-macro-generated-bodies fact, the unconstructible-variant dead_code
  blind spot, the flatten/retirement trap, the per-compilation-unit `expect`
  trap) and six friction observations.

- **PHASE-04 COMPLETE** — all eleven tasks landed, in `F1`'s order rather than
  the sheet's listing order (the `int-` retirement cannot precede the mint that
  replaces it, so `T2`/`T3`/`T7` are one movement). Done: `T1` `ReviewRef` +
  `ReviewPass` with `pass_over` in the shared fixture (`b1907148`); `T8`
  `undisposed_blockers` (`ce0885f2`); `T9` `ObservedReview` +
  `review::observe_pass` (`7d1df4ba`); `T4` the DEC-086 id-claim midpoint on the
  prebuilt placement path plus `materialise_prebuilt_at` (`510a966c`); `T5`
  `mint_review` / `materialise_review_at` refactored out of `run_new`
  (`4b5c9ad0`); `T6` the journal widened to `IntentSubject` + `MintKind` with the
  D4 rename set (`6a62ba95`); `[T2+T3+T7]` as one movement — the mint on
  entry to `reviewing`, `IntegratedReview` retired whole, `integrated_current`
  re-sourced onto `pass.covered` (`4943333a`); `T10` `VA-1`'s two fault-injected
  e2e rows (`c0d41a7e`, sheet `F5` — landed in `tests/e2e_design_review.rs`, not
  the checkpoint suite); `T11` close-out (`ae9cac57`). `EX-7` settled and `VT-4`
  amended (`eb408a0d`), reasoning as `DEC-140` (`91d709a6`, `2056b36b`).
  A late detour — `29c12e05` dropped `D3`'s intent-key alias and `7e2b768d`
  restored it (sheet `F6`), which unmasked `ISS-315` (sheet `F7`).
  Counts: entity 33 → 38, `review` 84 → 87, `design_run` unit 77 → 79,
  `e2e_design_review` 86 → 92, `e2e_design_checkpoint` 87 **unedited** (T6's
  control, and `T10` kept it that way). `verify-vt` `VT-1`…`VT-4` all PASS after
  the flip. `doctrine check gate` green; clippy clean at every commit.
  Memory recorded: `mem.fact.entity.crash-orphaned-claim-dir` (`f1af3c31`).

- **PHASE-03 done** — `ISS-310` closed. `ReviewPolicy` (four variants, serde-
  defaulted) sits on the run header; `ActorClass` gains one direction from
  `Reviewer` as an `impl From`; `ReviewPolicy::lanes` is the single home of
  membership. `DesignSnapshot::missing_lanes` / `sections_unreviewed` are the one
  derivation, and both the gate (`review_standing`) and the envelope
  (`review_outstanding`) read it, so they cannot disagree by construction. The
  policy change is the eighth writer act, riding an `AcceptanceDeclaration`, with
  a `ReviewPolicyChanged` row naming both values and the policy rendered on
  `RunLine`. `Attestation::reviewer`'s `expect(dead_code)` is deleted — the
  phase's own proof the defect closed. Unit suite 69 → 76; `tests/` touched in
  three files, each edit forced by an exhaustive table test or by `VT-5`.
  `doctrine check gate` exit 0, all five `verify-vt` rows PASS. Commits
  `bbcba0d3`, `cc6b48cb`, `1edc1377`, `a0a271ea`, `78a037a2`, `69a82423`,
  `b7c97434`. Two departures and one conformance-ordering slip — see Open.
- **PHASE-02 done** — `ContentCoverage<T>` is generic and owns `diff`, with
  `is_current` defined through it; `NodeMaterial` and `InquiryMap::materials()`
  project the question graph beside `SectionGroup::fingerprints`. Both incumbents
  spell `ContentCoverage<Fingerprint>`; nothing under `tests/` touched, unit
  suite 67 → 69, `doctrine check gate` exit 0, `verify-vt` `VT-1`/`VT-2` PASS,
  conformance 0 undeclared. Commits `847b2eec` (code), `b2f86268` (selector).
  `VA-1` discharged against the two **real** runs in the tree — see Open.
- **PHASE-01 done** — `Advance` closes the forward relation; `boundary_conditions`
  and `boundary_runbook` are edge-keyed and total, `can_advance` retires, and
  `forward_runbook` stops re-deriving the table. Behaviour-preserving: no file
  under `tests/` touched, unit suite 65 → 67, `doctrine check gate` exit 0,
  `verify-vt` `VT-1`/`VT-2` PASS. Commits `3baaa564` (code), `e454e5f2` (notes),
  `5b4ecedf` (selector). Two departures from the letter of `EX-2` — see Open.
- **Design locked** at run rev 89, six sections, `RV-344` pass 1 `done` (ten of
  ten findings verified). `DEC-139` minted and swept through all six sections;
  `IMP-391` narrowed, `IMP-392` reduced to the concluded marker plus the
  finding-set migration. Plan authored, repaired against a code-surface sweep,
  and approved.
- Minted across the design stage: `DEC-127`, `DEC-138`, `DEC-139`, `EVD-012`;
  `ISS-309`, `ISS-310`, `ISS-314`, `IMP-393`, `IMP-394`, `IMP-395`, `IDE-047`,
  `IDE-048`.
- Three friction observations — `.doctrine/observations/records/{6b,42,1b}/`.
  `42` (an `Agent`-tool subagent cannot run `Bash` under the worktree-jail hook)
  is the one that bites again if pass 2 is ever run through a subagent.
- All `.doctrine` committed promptly and path-limited throughout. `SL-241` holds
  `rfc/026` dirty; `IMP-398`'s backlog dir is untracked and belongs to the user.

### Learned

Tree facts read off the source, still load-bearing for the phases that consume
them. Cited by phase so a reader knows why each is here.

**PHASE-04/05 (from PHASE-02)** — `ContentCoverage<T>` is generic and its
`diff(&current) -> Vec<DesignId>` is id-ordered, so `CoverageStale::moved` reads
straight off it; `is_current` is `diff(..).is_empty()`, so there is no second
comparison to keep in step. `NodeMaterial` and `InquiryMap::materials()` exist
and are gated `expect(dead_code, reason = "SL-244 PHASE-05")` — PHASE-05 is the
declared first reader. `InquiryMap` has no `remove`, so a map that lost a node is
expressed by construction, not by mutation.

**PHASE-05 (from PHASE-03)** — the data PHASE-05's contract table reads is
built and live. `ActorClass { User, Agent, Adversarial }` and
`ReviewPolicy::lanes()` exist in `attestation.rs`; `RequiredActor::RunPolicy`
resolves through `lanes()`, and `Refusal::SectionsUnreviewed { subjects }` reads
`DesignSnapshot::sections_unreviewed() -> Vec<(DesignId, ActorClass)>` directly —
it is already id-ordered and already the gate's own answer, so the refusal and
the verdict cannot drift. `ReviewStanding` is untouched and still four `Copy`
booleans; PHASE-05 rewrites `satisfied` and `advance` anyway. No const table was
built here (`RequiredActor`, `ActRequirement`, the rows) — that is PHASE-05's.
`ActorClass::Agent` has no constructor yet and needs no `expect(dead_code)`:
`derive(Deserialize)` counts as a construction site.

**PHASE-03 (spent, kept for the readers it names)** — `Attestation` carries no
turn, sequence or timestamp, so lane order is declared and never enforced.
`run.rs:1062` replaces an `Attestation` by its own `id`, not by act or subject —
load-bearing, since two lanes reviewing one section must coexist. The attestation
set has exactly three readers; `live_reviews` is the one that must not be
policy-filtered, and is now `pub(super)` with that exclusion stated at both the
function and the call site.

**PHASE-04** — `IntegratedReview` is a third incumbent in `ReviewGroup`
(`snapshot.rs:330`) and the sole input to `integrated_current`. The `RV` TOML
carries an explicit no-status-because-derived refusal, so the concluded-pass
marker had to be argued as an event rather than as a function of the finding
set. `derived_status` (`review.rs:1009-1022`) is severity-blind — it reads
`status`, never `severity` — and its own doc calls it *"never an exclusive
gate"*; anything binding a gate to `await` is wrong by construction. The
`answered` state is the whole of `DEC-138`: `contest` moves
`answered → contested` (`review.rs:2354`), so contest does re-block, and
`doc_unresolved_blockers` (`review.rs:1490`) counts `answered`, leaving the
responder with no clearing act.

**PHASE-05** — `apply` takes `DerivedInput` as a parameter and threads it into
the declaration handlers (`run.rs:224-230`), which is what lets an
admission-time check read shell-observed state without the pure layer touching
disk. `ObservedFact` is a *currency* mechanism, not a general external-input
channel — a fact evaluated fresh needs a `DerivedInput` field instead, and
getting that backwards is what made the first `F1` recommendation wrong.
`LockAcceptance` is subsumed by `CheckpointAct`. `--as` is cooperative role
assertion (`review.rs:2813`, `ADR-007`) and an `AcceptanceDeclaration` cannot
distinguish a user's payload from an agent's (`DEC-088`), so *an agent cannot
author this act* is stronger than the system delivers; the achievable property
is *in the user's name, and leaving a change row*. `DEC-138`'s predicate is
universally quantified over the finding set and never reads `status`, so it is
**vacuously true on an empty ledger** and the `ISS-314` fix leaves that hazard
untouched — a clean pass is structurally identical to one never run, and only
the `## Synthesis` section distinguishes them at the artefact tier.

**Operating the run** (owed by `SL-243`'s hand repair, not by a phase) —
`run.rs:334-347` writes `review.acceptance` from ANY run-level
`AcceptanceDeclaration`, at any stage, last-write-wins, so `LockAcceptance` is
the run's single unnamed acceptance slot rather than the lock's. `SL-243` sits
at `drafting` holding three declared sections, never materialised, with
`attestation = []`. A record rides an inquiry node's disposition via a `cp-`
subject; declaring the node and disposing it cannot share one batch. Knowledge
facets are hand-edited — there is no `knowledge edit` verb.

**Process, earned twice each**:
`mem.pattern.review.sweep-defect-class-not-instance` (the `F7` repair contained
a surviving sibling of its own class, and round 6's integrations wanted the
class-sweep they had just applied to the findings — an integration round is a
writing round), `mem.pattern.review.observation-right-prescription-wrong`, and
`mem.pattern.feedback.rule-findings-in-dependency-order`. Both `verify-vt`
UNATTRIBUTABLE-until-completed and conformance-undeclared already have
memories; PHASE-01 confirmed them rather than teaching anything new.

### Open

- **PHASE-05's one deferred assertion, with its attribution.** A `Conducted`
  disposition **admitted** over a ledger that has concluded lands with
  `IMP-392`: `PassFacts::concluded` is hard-`false` (`review.rs:1615-1621`,
  which says so in as many words) because no verb sets the marker and no field
  holds it, so no `Conducted` arm is admissible on the live path and `Waived` is
  the deliberately available exit meanwhile. The *refusal* is asserted at the
  unit tier against a synthetic observation, three cases including a concluded
  marker on some other ledger. Sheet `F63`. `EX-7`'s prediction below held for
  the second clause and **not** for the third — see the amendment on that bullet.
- **PHASE-05 broke the snapshot shape once, deliberately (`EX-14`), and the bill
  is outstanding.** The act groups arrived and the evidence rows and
  `LockAcceptance` left, so `SL-243`'s run needs a five-act hand repair at its
  next forward move. Priced and accepted with the user 2026-08-04. The benign
  half is measured at sheet `F48`: the retired `[gate.*]` keys on live runs are
  *ignored* on read, not a second break.
- **A user-visible render change shipped in PHASE-05**: `SectionRow` no longer
  carries `clearances=`, which retired with the store it read (`EX-11`). Stated
  in the criterion rather than discovered in a diff, and it takes an uncapped
  list out of the envelope budget; the act list that replaces it in `resume` is
  bounded at eight by construction (`D5`).
- **`VT-8`'s PASS is not by itself evidence, and `/audit` should read `F58`
  before `F47`.** The row read PASS from `T10` onward on a keyword scan over
  `T8`'s tokens, then on a structural impossibility that `F57` showed was not
  one. What backs it now is `T13`'s chokepoint guard on `Fixture::payload` plus
  the ladder `lock_admits_with_all_four_present` already drove — argued as a
  deletion and a guard rather than a second ladder test. Two hand-rolled payloads
  in `e2e_design_state.rs` bypass the chokepoint by construction, stated not fixed.
- **`review.findings` is write-only after PHASE-05 and is left standing.** Its
  one reader retired with the evidence model; the blocking-findings derivation
  sources from the observed `RV` pass. Adjudicated at `VA-2` (sheet `F62`) as
  outside what that criterion forbids, and noted on `IMP-392` — where the removal
  is now a plain deletion rather than a migration of a live consumer.
- **`EX-7` is SETTLED: the finding set is readable today, so `IMP-392` narrows
  to the concluded-pass marker alone.** Evidence in tree: `FindingStatus` is
  `{Open, Answered, Contested, Verified, Withdrawn}`, `parse_finding_status` and
  `Severity::parse` are both live, and `grep -c concluded src/review.rs` is `0`
  against a positive control of `3`. Recorded in `plan.md` § *One dependency the
  plan flags rather than re-decides*. **Consequence PHASE-05 inherits:** its
  `VA-3` clause fires — the second and third deferred assertions (an undisposed
  blocker holding the edge; the contest/re-dispose cycle clearing) land live in
  PHASE-05 rather than waiting on `IMP-392`; only the first (`Conducted` over an
  unconcluded RV refused at admission) stays deferred.
  **Amended 2026-08-05 at PHASE-05 `T14`.** The prediction held for the second
  clause — `waiver_clears_over_live_findings_and_dismisses_none`'s second leg
  asserts `Cause::BlockersUndisposed` over the arm that reads the ledger — and
  **not** for the third: every test of the `Conducted` arm approached it from a
  failure and `cleared()` disposes by `Waived`, so that arm's positive case was
  asserted nowhere until `a_re_disposed_blocker_clears_the_edge_again`. A
  criterion saying a clause *lands live* is not evidence that it did.
- **`VT-4` was amended mid-phase, and `/audit` should read the amendment rather
  than the original.** Its clause *and the gate reads that as unmet* presumed
  `review-disposition-attested`, a condition PHASE-05 owns along with the act
  that names a review at all — so PHASE-04 had no reader and the clause could
  only have been satisfied hollowly. PHASE-05's `VT-9` already carries it
  verbatim, so it was dropped rather than duplicated, and `test_file` moved to
  `src/review.rs` where an unreadable ref can exist. Consulted with the user
  before the edit; reasoning as `DEC-140`. `EX-5` is otherwise met in full.
- **`EX-6b`'s second half is unobservable today, and that is why it argues from
  coupling.** The review-level `derived_status` guard could never fire while
  `undisposed_blockers` returns anything, because any `open` or `contested`
  finding forces `Active`. Pinned by `the_predicate_does_not_read_review_level_status`.
  So *not inheriting the guard* is a coupling decision (ADR-007 D7: the
  review-level status is a display summary, never a gate), not a wrong-answer
  fix. `/audit` should not expect a behavioural difference to be demonstrable.
- **`Condition::IntegratedReviewPresent`'s unmet mode narrowed to stale-only**
  (sheet `F2`), and the e2e suite is re-levered accordingly — **done**, in
  `4943333a`. Entry to `reviewing` always mints a pass, so presence is
  guaranteed by construction and only currency can falsify the condition.
  `Component::Integrated` now declares nothing; `refuses_without` stales the
  pass with a section edit in its own submission (the lock payload re-attests
  afterwards, so the refusal still isolates one condition), and the
  mandatory/opt-in test clears the condition again by **re-entering
  `reviewing`** — now the only way to clear it. This is `EX-8`'s *the currency
  lamp's input now exists* arriving as an argued behaviour change in
  `tests/e2e_design_review.rs`, not a weakening.
- **`D3`'s intent-key compat was dropped and then RESTORED the same day, and the
  round trip is the finding.** `29c12e05` deleted the
  `serde(alias = "checkpoint")` on the argument that `D3`'s stated reason was
  speculative — the sheet justified it as *"a crash from the previous binary is
  what it is resumed from"*, and neither `design.md` nor `plan.toml` asks for
  that (`EX-4` and `VA-1` are same-binary claims about resuming the *same*
  submission). That argument was **right about `D3`'s reason and wrong about the
  alias**. `RecoveryIntent` is serialised into the design-run **snapshot** —
  `[[checkpoint.intent]]` in `design.toml` — not only into the per-submission
  journal `complete_journal` clears, and a run spans weeks, so a snapshot
  routinely outlives the binary that wrote it. Dropping the alias made
  `doctrine design show` fail outright on this repo's own live runs: `SL-243`
  carries 9 pre-`DEC-125` intent rows, `SL-244` carries 16, including this
  slice's own locked run. Reverted in `7e2b768d`, with the doc now stating the
  reason it actually has. **Pinned at the tier that broke** —
  `a_snapshot_written_before_the_intent_subject_key_still_parses`
  (`src/design_run/snapshot.rs`, beside the `ReviewPolicy` precedent), not over
  `RecoveryIntent` alone; negative control run, it fails with the alias removed.
  The lesson for `/audit` is not "the alias is justified" — it is that the
  sheet's *reason* for it was never the load-bearing one, and a compat claim
  whose stated justification is wrong survives exactly until someone acts on the
  justification. `T6`'s control is untouched throughout:
  `e2e_design_checkpoint` green **unedited**, at 87.
- **`ISS-315` — `design show 244` is still broken, by `PHASE-04`, and it is not
  the alias.** Unmasked while fixing the above: `4943333a` retired
  `ChangeEvent::IntegratedReviewRecorded` per `EX-2`, but `SL-244`'s own change
  log already held one such row from when the run declared `int-1`. `ChangeEvent`
  deserialises strictly, so one unrecognised row fails the whole snapshot.
  `EX-2` is right about the **write** path; the change log is append-only
  **history**, and the vocabulary it writes is not the vocabulary it must read.
  Same class as the alias above, and the alias is the near-sibling that got it
  right. Filed as `ISS-315` with three candidate resolutions; **not decided
  here** — what a closed vocabulary owes its own history is a governance
  question. No authored artefact is at risk (this is a runtime-state read), and
  `PHASE-05` does not depend on reading the run.
- **PHASE-04's `VA-1` is evidence on the tree, and it sits in
  `tests/e2e_design_review.rs` rather than the sheet's
  `tests/e2e_design_checkpoint.rs`.** Two crash points, mirroring the pair the
  checkpoint suite runs over the knowledge arm, because the two windows promise
  different things: before DEC-086's id journal only *no record nothing can name*
  holds — the dead claim's directory outlives a hard exit, which is the tolerated
  reservation — and from the id journal onward the exact reserved id holds. The
  second is what exercises `review::materialise_review_at` on the tree instead of
  only in a unit test. **The departure is a strengthening, not a shortcut:** the
  ladder to `reviewing` is the review fixture's, so minting there cost two lines
  of fault env instead of a second copy of that ladder — and it leaves the
  checkpoint suite byte-unchanged, which is the very control `T6` rests on. Note
  the `VA-1` above this one is **PHASE-02's** (the snapshot round-trip harness);
  they are different criteria in different phases.
- **DEC-086's step 5 is not universal, and `MintKind` now says so.** A minted RV
  has no status to set (ADR-007 D-C8 derives it from the ledger) and authors its
  own `reviews` edge, so `MintKind::has_record_effects()` gates
  `apply_record_effects`. The RV's facet (`design`) and target (`SL-NNN`) are
  **derived, not declared** — one design run per slice — so no payload can name
  a different subject for the pass.
- **Two shell surfaces carry `expect(dead_code)` naming PHASE-05**, both
  expected to go live rather than be silenced further: `review::observe_pass`
  and `design_run::run::ObservedReview`. `undisposed_blockers`'s own `expect`
  was deleted the moment `observe_pass` read it, per the inverse trap.
- **`VA-1`'s evidence is recorded, not reproducible from the tree.** It required
  a *real* run rather than a fixture, and runs live in gitignored state, so the
  harness that parsed and re-serialised `.doctrine/state/slice/{243,244}/design.toml`
  before and after was a scratch test, run twice and deleted (phase sheet D2).
  The finding stands in the sheet's Outcome: `diff -r before after` empty, with
  `slice/243` carrying the stored `[review.acceptance.covered.covered]` the
  criterion protects. `/audit` either accepts that record or re-runs the harness;
  it cannot read the proof off the diff.
- ~~**PHASE-01's recorded e2e baseline does not reproduce.**~~ **RESOLVED in
  PHASE-03 (sheet F1).** The four `e2e_design_*` binaries `#[path]`-include
  `src/design_run/mod.rs`, and therefore its `#[cfg(test)] mod tests`. So each
  binary's count is *its own `#[test]`s + 2 (`common::test_support`) + the whole
  design-run unit suite*, and **adding one unit test moves four e2e counts by one
  each**. The tuples were correct when taken and drift by the unit-suite delta;
  they were also recorded from a multi-`--test` invocation, which does not report
  in flag order, which scrambled which number belonged to which binary. An e2e
  count is a derived number, not an independent control — measure per binary, and
  compare green to green.
- **`NodeMaterial` and `InquiryMap::materials()` have no production reader until
  PHASE-05**, and carry the slice-tagged `cfg_attr(not(test), expect(dead_code, …))`
  naming it — the same class as `Advance::ALL`, expected to go live rather than be
  silenced further.
- **PHASE-01 departed from the letter of `EX-2` twice, and `/audit` adjudicates
  both.** (a) `can_advance` is **deleted**, not kept as a one-line alias: the
  clause's purpose is *so the forward graph is written once*, and it is —
  `Advance::between(from, to).is_some()` is inlined at `advance`, its sole
  production caller. `VA-1` enumerates the other two sites and this phase
  disposes of both, so the function ends with no caller, and a zero-caller
  `pub(crate) fn` is dead code clippy denies here. (b) `boundary_runbook` is
  re-keyed to `Advance` too and **drops its `Option`**: `EX-4` needs the
  selector to choose the edge via `from_stage`, and the alternative was
  `Advance` growing `from()`/`to()` accessors purely to reconstitute the pair
  the type exists to replace. Behaviour-preserving — `advance` reaches that call
  only past the legality check, so the `is_some` guard it drops was provably
  always-true; `design.md:2967` (*both the runbook selector and the contract
  block take it*) is the design's own reading. Two callers genuinely need the
  `Option` because they ask an arbitrary pair, and now spell it
  `Advance::between(..).map(boundary_runbook)`. Either is a one-line reversion.
- **`Advance::ALL` has no production reader until PHASE-06/08** and carries the
  tree's slice-tagged `cfg_attr(not(test), expect(dead_code, …))` naming them.
  Expected to become live, not to be silenced further.
- **PHASE-03 departed from the letter of two criteria, and `/audit` adjudicates
  both.** (a) `VT-1` says *the standing names the missing lane*; `ReviewStanding`
  is `Copy` and passed by value into `satisfied` and `advance`, so carrying a
  `Vec` there breaks both signatures — signatures PHASE-05 rewrites regardless.
  Instead `DesignSnapshot::sections_unreviewed() -> Vec<(DesignId, ActorClass)>`
  is the derivation and `sections_attested` is defined *through* it, exactly as
  `ContentCoverage::is_current` is defined through `diff`. PHASE-05's
  `Refusal::SectionsUnreviewed { subjects }` reads it directly. Confirmed by the
  user before execution (sheet D1). (b) `EX-5` says the change rides *the
  attestation via `AcceptanceAttestation::bind`*; no call is made, because
  nothing can hold what it returns until PHASE-05's act groups arrive, and the
  one existing home — `review.acceptance` — would make a policy change satisfy
  `user-acceptance-attested`. What is delivered is what the criterion is
  observably for: the `AcceptanceDeclaration` is a REQUIRED field (a payload
  omitting it is refused at the wire), an empty basis is refused through the
  incumbent `AcceptanceBasisMissing`, and the change is logged. `/audit` should
  decide whether PHASE-05's act record adopts the attestation (sheet D2).
- **`live_reviews` (`run.rs`) is `pub(super)` for its test, not for a caller.**
  `VT-4` names it; it has no production caller outside `apply`. Widening
  `invalidation_rows` instead would have meant widening `Pending` too. The
  exclusion it embodies — invalidation is never policy-filtered — is stated at
  the function and at the call site, and a positive control confirmed the test
  fails when the predicted sweep is applied (sheet F3).
- **PHASE-03's conformance reads 2 undeclared, both this slice's own authored
  files.** `notes.md` and `slice-244.toml` rode a commit inside the phase's code
  boundary instead of a post-flip harvest commit, which is what PHASE-01 and
  PHASE-02 did. Cosmetic; the ordering is the lesson (sheet F4). Re-recording the
  delta to exclude them would also exclude a real code change. **PHASE-04 reads
  the same way, more so**: its boundary spans the plan amendment, `DEC-140`, and
  two harvest/selector commits, all authored `.doctrine/**` inside the code
  boundary. Deliberate — a handover is exactly where uncommitted authored state
  goes missing, so it was committed when written rather than held to the phase
  flip. `slice phase` warned at the flip that the boundary spans 12+ commits.
  Already-explained noise at `/audit`, not a defect.
- **`RV-344` pass 2 is specified in the ledger's `## Brief` and will not be run.**
  The user's call (2026-08-04): not because there is nothing left to find, but
  because it comes out cheaper against real code than against prose. Its four
  lines of attack — parallel-implementation risk, verifying every in-tree
  `file:line` the design load-bears on, unbuilt-vs-misread dependencies, and
  selector coverage — are therefore **carried into implementation**, and the
  phases that touch each surface are where they land. The brief is the standing
  record; it does not need re-deriving.
- **No external round has seen the repairs.** Round 6 was deliberately internal —
  the whole-document coherence lens no per-section codex round could apply — and
  every section has moved since. The user's position is that per-section external
  review is done. This is a *stated* residual, not an oversight.
- **`ObservedFact`'s second member** — `DEC-139` gives the seam an enforced
  consumer, so the *no consumer* half is struck. What stays open is whether a
  second member is ever wanted; until one exists the refresh/compare/absence
  semantics are specified against a single case.
- **`SL-243` owes five acts by hand** at its next forward move —
  `GovernanceConfirmed`, `GraphReviewed`, `BlockingSetDeclared`,
  `SufficiencyAccepted`, `DraftingReady`. Falls due when this slice's code lands,
  not before. Priced in `sec-2` and in the slice's risk register.
- **`ISS-314` is the user's, in parallel** — `derived_status` returning
  `(Done, None)` on an empty ledger against `ADR-007` D-C8. Separate from this
  slice's premise by construction; its body carries the table that says so.
