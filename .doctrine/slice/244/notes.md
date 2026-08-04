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
fresh-as-of: 2026-08-04 · design/reviewing (run rev 85) · 9d54a8f8

### Produced

- `DEC-127`, `EVD-012` — knowledge, bound into the run (`cp-15`, `cp-14`).
- `DEC-138` — **new**, via `inq-16` / `cp-16`. Refines `DEC-125`'s two arms,
  supersedes nothing. Amended twice after minting: once against `src/review.rs`,
  once when the amendment's own claim proved wrong.
- `ISS-309`, `ISS-310`, `IMP-393`, `IMP-394`, `IMP-395`, `IDE-048` — backlog.
- Design run sections `sec-1`..`sec-4` — materialised, watermark current at run
  rev 64. `sec-1` `8384bae959cb`, `sec-2` `919384dd4d0d`, `sec-3` `148796c1274c`,
  `sec-4` `c6f65176c9bf`. All four moved this session; none is externally
  reviewed at its current bytes.
- Coherence pass `F1`..`F11` — raised as prose, ruled in by the user, integrated,
  then swept for siblings per
  `mem.pattern.review.sweep-defect-class-not-instance`; three siblings found and
  fixed. Design-only session, no code touched, so no `check gate` beat is owed.
- Commits `dac3cb46`, `bff92556`, `58b428df`, `78378a08`, `1d7d264d`,
  `cbb0a7a5`, `7df32e57`, `91a00746`, `35c1e161`, `d784b808`. All `.doctrine`
  committed promptly and path-limited (`SL-241` holds `rfc/026` dirty).
- `IMP-392` body rewritten — it carried the pre-`DEC-138` waiver semantics, and
  now names the concluded-pass marker as the one thing `SL-244` asks it to *add*
  rather than merely expose.
- One friction observation, `.doctrine/observations/records/6b/`.

### Learned

Carried forward from the prior harvest and still standing: `EVD-012` binds gate
evidence to `design.md` sections only; `ISS-309` id-collision; `ISS-310` is an
unbuilt `DEC-073`, so no decision record is owed; `DEC-066` (not `DEC-074`)
mandates the integrated pass; `DerivedInput` is built before `apply`, so node
coverage compares material; three readers of the attestation set and
`live_reviews` must NOT be repaired; `Attestation` has no turn/sequence, so lane
order is declared not enforced; `can_advance` and `boundary_conditions` are two
hand-written matches, settled by a closed `Advance`; the run has two transition
relations differing in kind; `LockAcceptance` is subsumed by `CheckpointAct`;
the review policy cannot be a security boundary; `review_standing` holds two
structurally different currency derivations ten lines apart; checkpoint
acceptances do not reach the snapshot; `InquiryNode` carries no fingerprint.

New this session, all read off the tree rather than reasoned:

- **`run.rs:334-347` writes `review.acceptance` from ANY run-level
  `AcceptanceDeclaration`, at any stage, last-write-wins.** So `LockAcceptance`
  is not the lock's acceptance — it is the run's single unnamed acceptance slot.
  `SL-243` sits at `drafting`, never locked, holding one whose basis is its own
  `inquiring → drafting` settle-and-advance. This **corrects** the prior
  harvest's "checkpoint acceptances do not reach the snapshot": the `cp-`-scoped
  one does not, the run-level declaration does.
- **`SL-243` holds `[review] attestation = []`** — zero section attestations.
  `sec-4`'s claim that it attested adversarially and is repaired by declaring
  `AdversarialOnly` was false in three places, now corrected. Its real cost is a
  re-given acceptance, and `user-accepts-sufficiency` is what will bar it.
- **`derived_status` (`review.rs:1009-1022`) is severity-blind** — it reads
  `status`, never `severity`, so `await == Responder` fires on an open `nit`.
  Its own doc calls it a priority summary, *"never an exclusive gate"*. Anything
  binding a gate to `await` is wrong by construction.
- **The `answered` state is the whole of `DEC-138`.** `contest` moves
  `answered → contested` (`review.rs:2354`), so contest DOES re-block —
  the property is that the responder always holds a *clearing* act, not that
  contest is harmless. D-C9b's `doc_unresolved_blockers` (`review.rs:1490`) is
  the same filter minus the state restriction, so it counts `answered` and
  leaves the responder with no act that opens the gate.
- **A clean pass is structurally identical to one never run** — `findings: []`,
  `status: Done`, `await: None` either way. The `## Synthesis` section
  distinguishes them at the *artefact* tier, which is why the marker had to be
  new structured state rather than a test over what exists.
- **`--as` is cooperative role assertion, not a security boundary**
  (`review.rs:2813`, `ADR-007`), and an `AcceptanceDeclaration` cannot
  distinguish a user's payload from an agent's (`DEC-088`). Any claim that an
  agent *cannot* author an act is stronger than this system delivers; the
  achievable property is *in the user's name, and leaving a change row*.
- **Design-run record minting**: a record rides an inquiry node's disposition via
  a `cp-` subject (`disposes` + `dispose: {form: create, …}`). Declaring the node
  and disposing it **cannot** share one batch — the disposition refuses with
  `unknown node`. Two applies. `provenance` is internally tagged:
  `{"provenance": "user-directed"}`.
- **Knowledge facets are hand-edited.** `knowledge new` and the run's `create`
  disposition both seed facets empty; there is no `knowledge edit` verb. Status
  arrives `accepted` when the disposition carried an `AcceptanceDeclaration`.
- **Editing `sec-2` invalidated its two clearances** (`initial-concerns-recorded`,
  `user-accepts-sufficiency`) — `EVD-012` biting live, since clearance binds to a
  `design.md` section and those conditions are about the inquiry graph.
- `mem.pattern.review.observation-right-prescription-wrong` and
  `mem.pattern.feedback.rule-findings-in-dependency-order` — both earned again
  in the prior session; `DEC-138` reversed a finding integrated two revisions
  earlier.

New this session, all read off the tree:

- **`apply` takes `DerivedInput` as a parameter** and threads it into the
  declaration handlers (`run.rs:224-230`). So an admission-time check reads
  shell-observed state without the pure layer touching disk — which is what let
  `DEC-138`'s *admissible* stand verbatim instead of being moved to the gate.
  **Memory candidate**, not yet recorded.
- **`ObservedFact` is a currency mechanism, not a general external-input
  channel.** It compares a stored fingerprint against a refreshed one; a fact
  evaluated *fresh* needs a `DerivedInput` field instead. Getting that backwards
  is what made the first `F1` recommendation wrong.
- **`run.rs:1062` replaces an `Attestation` by its own `id`**, not by act or
  subject — load-bearing, since two lanes reviewing one section must coexist
  under a both-lanes policy.
- **The `RV` TOML carries an explicit no-status-because-derived refusal**, so the
  concluded-pass marker has to be argued against it (an event, not a function of
  the finding set) rather than merely specified.
- **`IntegratedReview` is a third incumbent in `ReviewGroup`**
  (`snapshot.rs:330`), and the sole input to `integrated_current` — the currency
  lamp.
- **`SL-243` holds three declared sections and has never been materialised** — no
  `design.md` on disk, `attestation = []`, one acceptance covering all three.
- `mem.pattern.review.sweep-defect-class-not-instance` earned itself sharply: the
  `F7` repair contained a surviving sibling **of its own class**, one record
  shape over.

### Round 5 (2026-08-04) — sections 5 and 6 written, codex round run and closed

- **`sec-5` written** — the contract channels. `DEC-124`'s refusal remedy and
  stage-entry receipt, where `DEC-123`'s injection rule is built.
- **`sec-6` written** — the published stage-machine diagram, `DEC-127`'s
  objective 6, which had a decision and no section. Raised by codex as `F-2`.
- **Codex round 5 run and closed.** Thread
  `019fc628-0f72-73e0-bd25-3d99c05d0965`, whole document, 14 findings
  (2 blocker, 11 major, 1 minor). **All 14 ruled in by the user**, integrated in
  dependency order across five commits, then swept by class — one surviving
  sibling (sec-4's snapshot-change count, invalidated by the `ReviewPass` repair).
- **`ReviewPass` is the round's keystone**: `F-1` (a disposition bound to no
  pass) and `F-11` (warnings with no derivable input) both wanted the same
  missing record. It also retires `IntegratedReview` with this slice rather than
  with `IMP-392`.
- **Two findings had a right observation on a wrong ground**, and both were ruled
  that way rather than wholesale: `F-8` cited `DEC-125` for an admission rule
  that record does not carry (the real precedent is
  `Refusal::AcceptanceBasisMissing`), and `F-14` named `RunbookNotDischarged` as
  the multiline-`Display` precedent when it is `VerifierFailed`.
- **One finding codex missed**, raised by me and integrated with the batch:
  `DEC-124` says `IMP-390`'s gate-condition face is discharged by the refusal
  leg, and `sec-5` built that leg without naming it.
- **`CHR-049` is now priced from the corpus** — four sessions, halfway through
  one run, ten `cluster:design-run` items minted 2026-08-01/02 inside the
  exercise window. The document's one unfalsifiable claim is falsifiable.
- Section digests at run rev 76: `sec-1` `8384bae959cb`, `sec-2` `c2dc33a4012a`,
  `sec-3` `223a192e4df1`, `sec-4` `4ffc3681008f`, `sec-5` `db3366a01496`,
  `sec-6` `0d739cd3f4cb`. Commits `2e05ac8f`, `87336de9`, `f596e797`,
  `c29da1e9`, `d4d8d80b`, `3e0ef4b5`.

### Round 6 (2026-08-04) — RV-344 opened, nine of ten findings integrated

- **`RV-344` opened** — design facet, target `SL-244`, raiser posture `holistic`,
  primed from 60 selector paths. Deliberately **two passes on one ledger**, and
  deliberately neither an inquisition nor per-section: codex has run that lens
  over every section repeatedly. Pass 1 (architecture / internal coherence over
  the document as one artefact plus its cited records) is **done**. Pass 2
  (landing terrain — parallel-implementation risk, in-tree citation
  verification, unbuilt-vs-misread dependencies, selector coverage) is
  **specified in the ledger's `## Brief` and not yet run**; it is for after the
  design nominally locks, and the user's stated preference is Sonnet for it.
- **Ten findings raised, all ten independently verified by me before disposal** —
  six checked at source (`F-1`, `F-2`, `F-3`, `F-4`, `F-7`, `F-8`), four against
  the cited design lines. None spurious.
- **Nine disposed `design-wrong` and verified.** `F-3` remains **open** — see
  Open below.
- **`ISS-314` raised** — `derived_status` (`review.rs:1010`) returns
  `(Done, None)` on an empty ledger, against `ADR-007` D-C8 which fixes it at
  `active`/`raiser` and states the reason. Two tests pin the violating value.
  The user is driving it in parallel; its body carries an explicit table
  separating it from `SL-244`'s premise so a fixer does not scope-creep.
- **`F-8` sharpened past what the pass found.** The design argued vacuous
  `Conducted` satisfaction from `derived_status`; the real source is that
  `DEC-138`'s predicate is **universally quantified over the finding set** and
  never reads `status`, so it is vacuously true on an empty ledger and the
  `ISS-314` fix leaves the hazard untouched. **This supersedes the prior
  harvest's Learned bullet** *"a clean pass is structurally identical to one never
  run — `findings: []`, `status: Done`, `await: None`"*: the first clause stands,
  the `(Done, None)` half is an incumbent defect, not a specification.
- **`F-1`'s producer traced to `DEC-086`** via `DEC-125`'s rationale —
  *"checkpoints mint authored entities from inside `apply` via `DEC-086`'s
  journalled intent"*, live in `src/commands/design.rs` (`journal_intent`,
  `journalled_intent`, `write_journal`). The design cited neither `DEC-083` nor
  `DEC-086`. **`SL-244` mints the pass** (user, 2026-08-04); `IMP-392` loses that
  bullet.
- **`DEC-139` minted** — the exploring checkpoints are enforced on this slice.
  Carries the cumulative-reach argument, the accepted hand-repair cost, and the
  consequence that `Activation` retires with no members.
- **Advancing the run is itself a specimen.** `drafting → reviewing` refused with
  all six cumulative conditions outstanding; four had died because `sec-1` and
  `sec-2`'s *prose* moved. **This supersedes the prior Open bullet** that the
  three user acts were "deliberately not re-recorded by an agent": they were
  re-bound this session to cross the edge, on the user's instruction to advance,
  with the map unchanged. Under this design that invalidation does not occur at
  all — `user-accepts-sufficiency` binds to `InquiryMap`, not to a section.
- **Friction: a spawned `Agent`-tool subagent cannot run `Bash` here.** A
  `worktree-jail: cwd-not-a-worktree` PreToolUse hook rejects
  `/workspace/doctrine`. Pass 1 completed by reading entities raw and using
  `Monitor` as a shell. **Pass 2 needs `grep` over `src/` and will hit this
  harder.** Observation `.doctrine/observations/records/42/`.
- Commits `6e012172`, `07de5711`, `05b894bc`, `e844b03b`, `ef78b50f`,
  `b5293523`, `afa890a4`, `56502ed5`.

### Round 7 (2026-08-04) — `F-3` integrated, `RV-344` pass 1 closed, design locked

- **`F-3` integrated and the ripple swept.** `DEC-139` carried into all six
  sections at `f0ecce6a`: `Activation` retired with no members, the classification
  table and `Contract` lose the column and field, the enforced-set arithmetic
  moves to 2 / 4 / 6 / 8, the stage-entry receipt loses its
  not-yet-enforced second half, and the rendered block gains the two exploring
  contracts. Disposed `design-wrong` and verified — **ten of ten findings
  verified; `RV-344` is `done`.**
- **Four governance edits at `62f8ff6f`**: `slice-244.md` Non-Goals amended,
  `IMP-391` narrowed to the interaction with its first two bullets marked done,
  `IMP-392` loses the `RV`-minting bullet, and `DEC-138` plus `IMP-392` both lose
  the superseded `(Done, None)` argument.
- **A coherence sweep over round 6's own integrations found four defects**
  (`43582c96`), all in or beside `sec-5`'s new remedy subsection. *"For eight of
  the nine rows the discharge is one act by one actor"* was false twice — two
  `Engine` rows name no act and `initial-concerns-recorded` names two acts by two
  actors. The complaint table claimed nine promises need the complaint while its
  own last row says the remedy keeps one. The arms-bearing remedy silently broke
  the stated remedy-equals-discharge equality, since that row now renders three
  lines against the receipt's one. And post-`AgentAct`/`AgentActKind` split, two
  places still attributed unrepresentability to the discriminant rather than to
  the payload type. **The lesson is the sweep itself**: an integration round is a
  writing round, and its own output wants the class-sweep it applied to the
  findings.
- **Two field-set changes went beyond `F-3`** and are recorded in `sec-5` rather
  than folded in: the rendered contract block now injects a rule's observed facts,
  and renders coverage on every attested row rather than eliding `Artefact`. Both
  were invisible while `governing-context-recorded` was unenforced; the second was
  an inconsistency the old block already carried.
- **The gate crossed on the user's instruction** (2026-08-04, *"do the clearance
  gate boogie"*) — six clearances re-recorded at rev 82, `drafting → reviewing` at
  rev 83. This is the second time the six have been re-bound by an agent; unlike
  round 6 it carries an explicit instruction rather than a standing acceptance.
  Under this slice's own design that invalidation does not happen at all —
  `user-accepts-sufficiency` binds to `InquiryMap`, not to a section.
- **Scope and selectors reconciled for the reviewing runbook** (`8db053d9`,
  `9d54a8f8`): objective 4 amended to delivered-not-deferred, the snapshot risk
  repriced at five hand acts, the closure intent extended to bar a
  specified-but-unenforced condition; three selector notes moved.
- **Friction: `adopt_authored` makes the caller reimplement Doctrine's own
  section parsing and hashing.** `design show` prints 12-char prefixes and
  `AdoptionMarkersInvalid` names ids without digests, so re-adopting a
  hand-edited `design.md` means reading `document.rs::parse` and
  `commands/design.rs::authored_sections` and writing a script. Observation
  `.doctrine/observations/records/1b/`. **Positive control matters here**: the
  reimplementation was proved against a known-good commit's recorded digests
  before use, because getting it wrong mismatches every section at once and looks
  exactly like a moved document.

### Open

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
- **`CHR-049` is settled** — priced from the corpus in round 5, so the design
  carries no unverified claim of its own.
