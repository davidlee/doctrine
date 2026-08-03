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
fresh-as-of: 2026-08-03 · design/drafting (run rev 43) · 8e23952c

### Produced

- `DEC-127`, `EVD-012` — knowledge, both accepted/captured and bound into the run
  (`cp-15`, `cp-14`).
- `ISS-309`, `ISS-310`, `IMP-393`, `IMP-394`, `IMP-395` — backlog.
- Design run sections `sec-1`, `sec-2`, `sec-3`, `sec-4` — materialised, watermark
  current (rev 43). `sec-3` is v4 (two review rounds integrated); `sec-4` is v1.
- Research thread 4 + `research/raw/documentation.md`; baseline restamped twice.
- One friction observation, `.doctrine/observations/records/6b/`.

### Learned

- `EVD-012` — gate evidence binds only to `design.md` sections.
- `ISS-309` — shipped assets cite repo-private ids; per-repo sequential ids make
  this resolve wrongly rather than dangle.
- `ISS-310` — `sections_attested` ignores `Reviewer`. **Wider than the issue
  says:** `review_outstanding` (`render/envelope.rs:799-806`) matches subject +
  fingerprint only, so the envelope reports a section reviewed on an adversarial
  attestation. Fixing `satisfied` alone yields a gate and a render that disagree.
- **`DEC-074` grants an adversarial-only *run policy*** — "the user may change
  the run policy to adversarial-only or require both lanes in either order". So
  requiring human at the gate **removes a capability**, it does not harden a
  default; and it supplies a fourth `ISS-310` candidate nobody had — per-run
  policy, which keeps `DEC-074` intact and puts the actor in run data.
- **Checkpoint acceptances do not reach the snapshot.** The review group holds
  exactly `attestations` / `integrated` / `acceptance: LockAcceptance`
  (`snapshot.rs:320-333`); a checkpoint's `AcceptanceAttestation`
  (`commands/design.rs:847`) rides a `CheckpointPlan` and is journalled. Four of
  `sec-4`'s eight acts therefore have no home a `DesignSnapshot`-reading gate
  can see.
- `InquiryNode` carries no fingerprint (`inquiry.rs:204-222`) and `DerivedInput`
  no node digest map — so `sec-3`'s `Coverage::InquiryMap` names a map that
  cannot presently be built.
- The in-tree type is **`LockAcceptance`**, not `AcceptedDesign` — mis-named in
  both `sec-3` and `sec-4`.

### Open

- Whether `ObservedFact` justifies its seam with one member.
- **`review-disposition-attested`'s cumulative reach is not enforceable** until
  `IMP-392` gives it the `RV`-backed finding set to bind to. Recorded in `sec-3`
  as a gap, deliberately not asserted as a guarantee.
- **Seven findings from the joint `sec-3` v4 + `sec-4` review await the user's
  ruling.** Nothing integrated. Two blockers — the `GovernanceEdges` projection
  wrongly excludes `references --role concerns`, and four acts have no snapshot
  home. Five majors — the human rule unreconciled across render surfaces,
  co-location is not ordering, `InquiryMap` unbuildable, the macro still admits
  an illegal edge pair, `AgentDeclaration` underspecified. Codex thread
  `019fc628-0f72-73e0-bd25-3d99c05d0965` holds the whole chain.
- **`ISS-310` is reopened by its premise**, not by its ruling — see `DEC-074`
  under Learned. The user ruled `human` believing it additive; it is not.
- **The `ISS-310` answer wants a decision record.** Decided by the user with
  alternatives weighed and a cost accepted; currently recorded only in `sec-4`'s
  prose. No open inquiry node to bind it to, so it needs a route.
- Next section after `sec-4` clears: the contract's two channels
  (`DEC-122`/`DEC-124`) — where `DEC-123`'s injection requirement is built.

**`ISS-310` was ruled `human`** — `section-attestations-current` requires a human
section review, on a start-strict argument (`human → either` widens compatibly;
`either → human` breaks runs that cleared loose). `SL-243` needs manual repair
either way. **The ruling now needs re-confirming**, because one of its three
supports was false: `DEC-074` does not make the integrated pass mandatory (it is
about section posture) and it explicitly permits an adversarial-only run policy.
So `human` deletes a granted capability rather than hardening a default, and a
fourth candidate — per-run policy — exists and is already the shipped model.

**The `const`-ness tradeoff is closed**, not open: `boundary_conditions` is
unchanged and still `const`; both filters land in `cumulative_conditions`, which
was already non-`const`. The three-option passage was deleted rather than
decided.

**The `const`-ness tradeoff is closed**, not open: `boundary_conditions` is
unchanged and still `const`; both filters land in `cumulative_conditions`, which
was already non-`const`. The three-option passage was deleted rather than
decided.
