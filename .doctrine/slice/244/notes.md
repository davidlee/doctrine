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
fresh-as-of: 2026-08-04 · design/drafting (run rev 58) · 7df32e57

### Produced

- `DEC-127`, `EVD-012` — knowledge, bound into the run (`cp-15`, `cp-14`).
- `DEC-138` — **new**, via `inq-16` / `cp-16`. Refines `DEC-125`'s two arms,
  supersedes nothing. Amended twice after minting: once against `src/review.rs`,
  once when the amendment's own claim proved wrong.
- `ISS-309`, `ISS-310`, `IMP-393`, `IMP-394`, `IMP-395`, `IDE-048` — backlog.
- Design run sections `sec-1`..`sec-4` — materialised, watermark current at run
  rev 58. `sec-1` `ec82d845c010` (untouched since rev 45), `sec-2`
  `a8b38b1c19ee`, `sec-3` `ad7e18144a24`, `sec-4` `f62c19e67ad9`.
- Commits `dac3cb46`, `bff92556`, `58b428df`, `78378a08`, `1d7d264d`,
  `cbb0a7a5`, `7df32e57`.
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
  this session; `DEC-138` reversed a finding integrated two revisions earlier.

### Open

- **Handed over for a coherence pass** — see the continuation. Four reversals
  landed this session and cross-references are the known casualty (one was
  already caught: `sec-4` pointed at a `sec-3` rule that no longer existed).
- **Missing type declarations** — `ActKind` was used as a field type in two
  structs and never declared until this session. `ReviewRef` and the concluded
  marker are named in `sec-4` and not defined. A sweep is owed.
- **`sec-5` is unwritten** — `DEC-124`'s two channels (refusal remedy,
  edge-grained stage-entry receipt on the `Fragment` register,
  `design.rs:1832-1851`), where `DEC-123`'s injection rule is built. `sec-3` and
  `sec-4` reference all three of `DEC-122`/`123`/`124` but specify no channel.
  **Order not yet ruled**: before the codex round (so one round closes the
  document) or after (leaving `sec-5` unreviewed).
- **`sec-2`'s two clearances are stale** — the run cannot cross
  `drafting → reviewing` until they are re-recorded. Deliberately not
  re-recorded by an agent: under the incumbent's claimed arm it could be, and an
  agent asserting *the user accepts sufficiency* is the defect this slice exists
  to close.
- **The codex round is not run.** Thread `019fc628-0f72-73e0-bd25-3d99c05d0965`
  holds four rounds and last saw `sec-3` v4 / `sec-4` v1 — now v13 / v11.
  `sec-2` has never been externally reviewed and is now correct enough to be.
- **`ObservedFact`'s single member** — the seam has no enforced consumer this
  slice, since `GovernanceEdges` is named only by a `Pending` row. Stated
  honestly in `sec-3`'s carried-forward; left open rather than forced.
- **`DEC-138` has been amended twice and not re-checked against code since.**
  The per-finding predicate, the `answered` distinction and the concluded marker
  are all as-specified, not as-verified.
