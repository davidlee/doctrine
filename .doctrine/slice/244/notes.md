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
fresh-as-of: 2026-08-04 · PHASE-02 · b2f86268

### Produced

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

**PHASE-03** — `review_standing` holds two structurally different currency
derivations ten lines apart. `Attestation` carries no turn, sequence or
timestamp, so lane order is declared and never enforced. `run.rs:1062` replaces
an `Attestation` by its own `id`, not by act or subject — load-bearing, since two
lanes reviewing one section must coexist. The attestation set has exactly three
readers and `live_reviews` must **not** be repaired with them.

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

- **`VA-1`'s evidence is recorded, not reproducible from the tree.** It required
  a *real* run rather than a fixture, and runs live in gitignored state, so the
  harness that parsed and re-serialised `.doctrine/state/slice/{243,244}/design.toml`
  before and after was a scratch test, run twice and deleted (phase sheet D2).
  The finding stands in the sheet's Outcome: `diff -r before after` empty, with
  `slice/243` carrying the stored `[review.acceptance.covered.covered]` the
  criterion protects. `/audit` either accepts that record or re-runs the harness;
  it cannot read the proof off the diff.
- **PHASE-01's recorded e2e baseline does not reproduce.** Its sheet lists
  `74/4/74/74/106` for the five design e2e binaries; at `37fcf657` they measure
  `108/76/4/76/76`, all green, and PHASE-02 measured its own baseline rather than
  inheriting one. No test fails at either commit — recorded so it is not read as
  a regression.
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
