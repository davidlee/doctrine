# Gate conditions carry their own contract

## Context

The design-run stage gate refuses forward moves against a closed vocabulary of
ten `Condition`s (`src/design_run/gate.rs`). A condition is a fieldless enum
variant. Its entire data content is which of ten names it is: `ALL`,
`is_derived()`, and `as_str()` are the whole API. What each condition *requires*,
what kind of subject it binds to, and how an agent discharges it exist nowhere the
program can reach — `governing-context-recorded` is specified in nine words at
`.doctrine/slice/233/design.md:268` and named in zero shipped guidance.

Three consequences, each already raised separately:

- An agent cannot ask what an edge requires. It reads `gate.rs`, or submits and
  reads the refusal — and `GateNotCleared` carries only `Vec<Condition>`, so the
  refusal can name the objection and never the remedy (`IMP-390`).
- Six of ten conditions are caller-*claimed* rather than derived. `satisfies()` is
  existential over evidence rows: the gate checks that someone asserted the
  condition against *some* subject whose bytes have not moved. The DEC-066
  fingerprint binding makes a claim **expire**; it never makes one **true**
  (`ISS-285`).
- Evidence subjects resolve only from draft sections, so a claim about governing
  context binds to an arbitrary section's fingerprint and is invalidated by prose
  edits unrelated to the fact claimed. The binding does no semantic work
  (`ISS-286`).

Measured cost: over CHR-049's run, 33 reads of `src/design_run/**` against 52
`doctrine design` calls, four of them specifically hunting gate conditions in
source, one reading the test suite to learn how to clear a gate (RFC-026 **E8.7**).
That ratio is a lower bound on the confusion — the source resolved it here, and an
installed client project has no source to read.

**Why this is actionable now.** `ISS-285` deferred its choice until "after
PHASE-16 has shipped, when there is operational evidence about whether the runbook
guard suffices on its own." SL-233 is done and CHR-049 is that evidence. `DEC-101`
was read as forbidding improvement here; its 2026-08-02 restatement establishes
that the open→closed type error constrains only where a condition's
*satisfaction* may be sourced, and says nothing about whether a condition may
describe itself.

**The pattern already exists in-tree.** `ReviewStanding` — derived, never stored,
one independently-repairable boolean per component — is the shape this slice
generalises. It covers exactly one of four edges today.

## Scope & Objectives

### 1. A condition states its own contract

> **Settled 2026-08-03 — `DEC-122`, `DEC-123`.** The fork below is closed. The
> contract ships as **embedded prose**, `customization = "fixed"` — the prose
> direction, sealed rather than overridable, which is what keeps `IMP-372`'s
> override seam out of this slice's path. **Structure rides a const table**
> beside `boundary_conditions`; the prose stays narrative rather than being
> parsed back into fields. So both candidates land, on the axis each is good
> for: the table carries what the program must reach, the prose carries what an
> agent must read. `IDE-047` holds the further idea of extracting structure
> *from* the contract prose; it is not this slice.
>
> The **working hypothesis below is refuted**, not deferred. Research thread 1
> found the seal/craft line coinciding with `is_derived()` for all ten members —
> but the coincidence is analytic, not empirical: the reasoning classifies as
> *craft* exactly those conditions with no engine enforcement, and "no
> enforcement" is `is_derived() == false` restated in `DEC-102` vocabulary. It
> confirms nothing. `DEC-120` replaced the boolean outright, so the hypothesis
> has no subject left.

Make what a condition requires, what subject kind it binds, and how it is
discharged into something the program can reach and render. What scope fixes is
that the nine words at `design.md:268` stop being prose in a design document, in
the register `mem.pattern.design.classify-at-authoring-not-from-behaviour`
prescribes: the artefact states the property, rather than a run inferring it.

**Where the contract lives is open, and the two candidates differ in kind.** It
may be Rust-side data (associated const, a trait, a table beside
`boundary_conditions`), or a **correspondence between the closed `Condition`
vocabulary and prompt prose** dispensed by the CLI at the moments it is needed —
the same register the design prompt pack and obligation runbooks already occupy.

Either way `DEC-101` is untouched: the closed set is the **key** and the contract
is the **value**, a total function out of ten known members. Nothing is narrowed
into the vocabulary. The prose direction carries a mirror hazard instead, and
naming it is part of this slice's work: overridable prose that states what
discharges a condition lets a project change what the gate *appears* to require
without changing what it *checks*.

**Working hypothesis, to be tested not assumed.** `DEC-102` seals an asset when a
project override would make its content *false* rather than merely *different*.
Applied per condition, that line may coincide with `is_derived()` — a derived
condition's prose describes an enforced check and is sealed; a claimed
condition's prose describes a convention nothing enforces and is craft. If the
seal line and the derivation line are the same line, one decision governs the
vocabulary, the asset policy, and `ISS-285`'s fork together.

### 2. Derive what is derivable

> **Amended 2026-08-03 — `DEC-126`.** Two of the three named below derive:
> `materialisation-current` and `blocking-inquiries-dispositioned`.
> **`required-sections-exist` retires** rather than deriving — it has no
> implementation to extend (grep returns the enum variant, the
> `boundary_conditions` row, the kebab token, and tests that record a *claim*),
> and which sections a design must have is craft under `DEC-102`, not
> Doctrine's to mandate. It is replaced by `drafting-readiness-attested`, since
> retiring it outright would leave `drafting → reviewing` guarded by
> `materialisation-current` alone, which is trivially true of an empty document.
>
> The objective is also narrower than its title now reads: the final count is
> **two Derived, seven Attested, zero Claimed**. Deriving is the minority
> outcome, not the default one.

Extend the `ReviewStanding` pattern leftward to the conditions the snapshot can
already answer — `RequiredSectionsExist`, `MaterialisationCurrent`, and
`BlockingInquiriesDispositioned` are framework-owned run state and derivable
without crossing any vocabulary boundary.

### 3. Attest what cannot be derived

> **Settled 2026-08-03 — `DEC-120`, `DEC-126`.** A distinct kind, and the
> load-bearing one. `DEC-120` gives three kinds — Derived (every input
> engine-authored), Attested (some input can only be authored by a human act,
> and the engine derives over the recorded artefact), Claimed (no artefact, only
> an existential scan — the defect class). `DEC-126` applies it: the design-run
> gate is **an attestation ledger, not a checker**, and the discriminator is not
> *is it a judgement* — nearly all are — but *does the actor's identity matter*.
>
> `user-accepts-sufficiency` and `user-acceptance-attested` are the same kind of
> act, as this objective suspected; they sit on opposite sides of `is_derived()`
> today only because that boolean tracked implementation coverage rather than the
> nature of the condition.

`UserAcceptsSufficiency` is a human act and must not be computed. Establish
whether it is the same kind of thing as a derived condition or a distinct kind,
and stop the vocabulary from implying they are interchangeable.

### 4. Settle the exploring pair

> **Decided 2026-08-03 — `DEC-121`.** `ISS-285`'s three options all read the pair
> as run state. It is not run state: `exploring.toml`'s five steps and
> `inquiring.toml`'s two are **all agent-solo**, so the only user contact across
> orientation and interrogation is one blanket `user-accepts-sufficiency` at the
> far end. The pair is the residue of an interaction `SL-233` specified in a
> bullet and never built. Both become Attested user checkpoints — a governance
> confirmation carrying the dismissal list, and an inquiry-graph review carrying
> the agent's blocking-set declaration.

`ISS-285` offers three options for `GoverningContextRecorded` and
`InitialConcernsRecorded`: specify them with a subject rule that means something,
derive them from framework-owned run state, or retire them and let the runbook
guard the edge alone. Deciding is `/design`'s work; this slice owns the decision
and its consequences, including `ISS-286`'s subject-rule half.

> **Delivered, not deferred — 2026-08-04, `DEC-139`.** The design first specified
> both checkpoints and left them unenforced pending `IMP-391`, which would have
> made this objective a specification rather than a settlement. `RV-344` `F-3`
> showed the slice ships the wire acts and the artefact storage regardless, so
> the guard was being declined for a reason the design itself falsified. Both
> conditions are enforced from this slice; `IMP-391` keeps the interaction. See
> Non-Goals for the corrected boundary and the accepted migration cost.

> **`ISS-286`'s subject-rule half is answered — 2026-08-03, `EVD-012`.** Not by
> a separate repair but as a consequence of `DEC-121`: an Attested condition
> binds to the artefact the attested act produced — for
> `governing-context-recorded`, the confirmed governance **edge set**. That is a
> set of entity ids, so the subject stops being an arbitrary prose section and
> stops carrying restated canonical content at all.
>
> `EVD-012` is the mechanical case, observed on this run. Passing this very edge
> required minting a `design.md` section to point evidence at, because
> `current_fingerprint` resolves subjects against sections only
> (`run.rs:1471-1478`). Three defects fall out and all three dissolve under
> `DEC-121`: the subject is unconstrained (any section clears any claimed
> condition); the name asserts what the check cannot see; and the bootstrap is
> inverted, a stage-1 boundary satisfiable only by a stage-3 artefact. The
> forced restatement is also a **single-source-of-truth violation** — canonical
> entity content duplicated into prose so that clearance has something to bind
> to. Recorded rather than merely noted, because this slice's own design run
> committed it under protest.

### 5. Make the requirement readable before it is violated

> **Amended 2026-08-03 — `DEC-124`.** The split is by **channel**, and the
> envelope is not the contract's home. The **refusal carries the remedy** — it
> has no byte budget, and naming what would satisfy an unmet condition is what
> `IMP-390` complains is missing. The **stage-entry receipt carries the
> contract** — the `Fragment` register, delivered once and re-sent only when it
> changes, which is the amortised answer to a budget the envelope cannot afford.
> **No digest**: the receipt is the unit, and a per-condition digest buys
> granularity nothing needs.

Surface a stage's unmet conditions — and what would satisfy them — on the turn
envelope and in the refusal, so an agent learns the contract without reading
source. Bounded by the envelope's existing byte budget.

### 6. Document the interactions, with diagrams

Ship spec-adjacent documentation carrying **canonical diagram(s)** of the stage
interactions this slice specifies — who acts, what artefact the act produces, what
derives over it. Its presence guides agents even where the tooling affordance has
not landed, which is the case for the whole of `IMP-391` until that ships.

This is a first-class deliverable, not a write-up. `SL-233`'s failure was
attending to everything except the interaction design that was most of its point;
the corrective is not more prose about mechanism but a stated, diagrammed account
of the interactions themselves.

> **Researched 2026-08-03 — research thread 4.** There is exactly **one** in-tree
> precedent for a diagram documenting a machine, and it is **generated**:
> `.doctrine/spec/tech/021/funnel-machine.md` is a `stateDiagram-v2` golden-pinned
> byte-for-byte to the `const` transition table in `src/funnel_machine.rs`. No
> mermaid, d2 or graphviz exists anywhere under `install/`. Since `SPEC-029` D1
> specifies the gate as the same kind of `const fn` table, a hand-rolled diagram
> is the avoidable mistake.
>
> Two remaining objectives-relevant facts. `SPEC-029`'s headings are exactly
> Overview / Responsibilities / Concerns / Hypotheses / Decisions — no stage list,
> no diagram, and an empty `interactions.toml`, so this is net-new material. And
> `install/dispatch-mechanics.md` / `install/review-ledger.md` are the right
> reader-facing template for a *who acts, what does the act produce* narrative,
> but neither carries a diagram, so shipping one there is net-new too and needs
> its own hand-edit-vs-generated policy.
>
> **Settled 2026-08-03 — `DEC-127`.** Published surface, generated. Thread 4's
> spec-sibling champion is refused: a client repo cannot read this repo's spec,
> and the gate-standing agent is always a repo-external consumer. Source under
> `install/` (already grafted, so no `flake.nix` change), a
> `publication/manifest.toml` entry, and a golden test pinning the render to
> `gate.rs` — the `funnel-machine.md` mechanism, transferred. `SPEC-029` cites the
> published address rather than holding a copy.

## Non-Goals

- **`next_obligation`'s writer** (`IMP-390` face 1). A separate wire question with
  a separate cost (a v1 envelope change, discarding the live run); it is worth
  little until the gate can say *why* it wants the advance.
- **A fetchable payload-schema surface** (`IMP-390` face 2). Genuinely independent,
  no gate dependency, deserves its own slice.
- **Deriving the traversal cursor** (`IMP-389`). Same family of complaint —
  declared where it should be derived — but a different subsystem and a different
  decision.
- **Runbook `set` mode and cursorless runbook rendering** (`IMP-373`). Adjacent and
  deferred with its own repayment condition.
- **Obliging agents to surface the inquiry map** (`ISS-299`). A prompt-asset
  problem, not an engine one.
- **Changing what the runbook guard does.** The runbook stays a separate guard on
  the same edges. This slice does not merge, map, or reconcile the two.

  > **Amended 2026-08-03 — `DEC-121`.** Still true of *reconciling* the guards,
  > and now explained rather than merely asserted: the step is agent-attested and
  > prompts the interaction, the condition holds the **user's** attested artefact.
  > Different actors, different bindings, different repairs. `IMP-391` adds steps
  > that prompt the new checkpoints; that is authoring within the existing
  > mechanism, not changing what the guard does.
- **Building the exploring-stage checkpoints** (`IMP-391`, spawned by `DEC-121`).
  This slice specifies the interactions, their contracts, and their artefact
  shapes, and ships the diagrams; the wire acts, the artefact storage, the runbook
  steps and the CLI rendering are the follow-on. Stated interim state: until
  `IMP-391` lands, `exploring → inquiring` passes on the runbook alone — no worse
  than the status quo it replaces, but a gap, not an oversight.

  > **Amended 2026-08-04 — `DEC-139`.** The boundary above is wrong about where
  > two of those four things landed, and `RV-344`'s `F-3` is what found it: the
  > design ships the wire acts and the artefact storage for all three checkpoint
  > acts — two of `IMP-391`'s five deliverables — so this slice was doing the
  > follow-on's work and still declining the guard. Corrected: **the wire acts
  > and the artefact storage are in this slice, and both exploring conditions are
  > enforced from it.** `IMP-391` keeps the *interaction* — the runbook steps
  > that prompt each checkpoint, the CLI rendering of each artefact, and the
  > empty-case affordance — and its first two bullets arrive done. The interim
  > state is correspondingly narrower: `exploring → inquiring` is guarded from
  > this slice, and what is unfinished is interaction quality, since until
  > `IMP-391` lands the user confirms an artefact the agent paraphrases.
  > Objective 4 is therefore delivered rather than deferred. The cost accepted
  > with it: both rows are cumulative, so no in-flight run can cross any forward
  > edge without acts no run holds, and the repair is by hand — `SL-243` is the
  > only live run owing one.
- **Migrating design-run findings onto `RV`** (`IMP-392`, spawned by `DEC-125`).
  `DEC-125` unifies findings on the `RV` review kind rather than the runtime
  `Finding` model; this slice specifies the condition that reads them, the
  follow-on does the migration. Stated interim state: `review-disposition-
  attested`'s `Conducted { review }` arm and the outstanding-findings severity
  summary are **unbuildable** until `IMP-392` lands. Named, not discovered later.
- **Extracting contract structure from the contract prose** (`IDE-047`, spawned by
  `DEC-123`). Structure rides a const table and the prose stays narrative; deriving
  one from the other is a further idea with its own argument to make.

## Affected Surface

Coarse and provisional — `/design` fixes the touch-set. Fenced as
`scope-relevant` selectors on the slice.

- `src/design_run/gate.rs` — the `Condition` vocabulary, `boundary_conditions`,
  `cumulative_conditions`, `ReviewStanding`, `satisfied`, `advance`.
- `src/design_run/facts.rs` — `Evidence` and `DerivedDesignFacts::satisfies`, the
  claim path and its liveness rule.
- `src/design_run/refusal.rs` — `GateNotCleared` and whatever carries a remedy.
- `src/design_run/render/envelope.rs` — where an unmet condition becomes readable,
  under the existing byte budget.
- `src/design_run/snapshot.rs` — `DerivedDesignFacts` is persisted snapshot state;
  changing what evidence means may version the snapshot.
- `src/design_run/run.rs` — subject resolution (`current_fingerprint`), the
  section-only rule `ISS-286` names.
- `.doctrine/spec/tech/029/` — SPEC-029 describes payload-claimed evidence as the
  design; moving conditions off that footing is a spec change.
- `install/design-prompts/**` — if a condition's contract becomes readable, the
  shipped guidance that never named these conditions is where it lands.

> **Added 2026-08-03 — decisions since scoping.** Still coarse; `/design` fixes
> the touch-set.
>
> - `src/design_run/attestation.rs` — `DEC-120`'s Attested kind derives over
>   recorded attestations, and `Reviewer` is where actor identity already lives.
> - `src/commands/design.rs` — the `Fragment` stage-entry receipt is `DEC-124`'s
>   contract channel (`:1832-1851`).
> - the `RV` review surface — `DEC-125` unifies findings there; the read path this
>   slice specifies lands against `RV`, not the runtime `Finding` (`IMP-392` does
>   the migration).
> - `.doctrine/spec/tech/029/` — beyond the spec revision already named, objective
>   6's diagram lands here as a **generated sibling artefact** if thread 4's
>   champion holds (`OQ-9`).
> - `publication/manifest.toml` — only if `OQ-9` resolves towards a *published*
>   surface. An embedded-but-undeclared asset is invisible; a declared asset with a
>   stale backing is a gate failure. No `flake.nix` graft is needed for material
>   under an existing embed root.

## Risks & Assumptions

- **Snapshot versioning.** `DerivedDesignFacts` persists to the run snapshot. If a
  condition's payload or the evidence shape changes, the snapshot schema version
  moves and existing runs are rejected at parse. The tier is gitignored and
  documented disposable, and one live run exists (SL-243's), so the cost is that
  run — real, bounded, and worth naming before it is paid.

  > **Repriced 2026-08-04 — `DEC-139`.** Still bounded to `SL-243`, and larger
  > than a schema move: enforcing the two exploring checkpoints bars every
  > in-flight run at its next forward move, because both are cumulative and no
  > run holds an exploring-stage act or can acquire one retroactively. `SL-243`
  > owes five acts by hand, not two. The design prices it; the user accepted it
  > as cheaper than a second round of design review.
- **The envelope has a byte budget.** Rendering unmet conditions competes with
  everything else the envelope carries, and `EX-14` already bounds runbook
  rendering for the same reason. "Explain every condition every turn" is not
  available; what gets said and when is a design decision.
- **`DEC-101` remains binding.** Nothing in this slice may source a condition's
  satisfaction from runbook steps or any other open vocabulary. The restatement
  clarified the rule's scope; it did not relax it.
- **Assumption: CHR-049 is adequate operational evidence** for `ISS-285`'s
  deferred choice. It is one moderated run. Per
  `mem.pattern.design.classify-at-authoring-not-from-behaviour`, it can establish
  adherence but not a universal claim about what an obligation *is* — so it
  informs the retire/keep decision without settling it by itself.
> **Resolved 2026-08-03 — `DEC-122`.** The override-seam risk below is retired
> as a *risk* and kept as a *stated assumption*: the seam does not exist, and
> this slice does not build it. Contracts ship embedded and `fixed`, with a
> citation — the pattern runbooks already use — so nothing here claims
> overridability and nothing inherits `IMP-372`. If the seam is built later, this
> choice becomes a constraint this slice need not have accepted; that is the
> honest cost of deferring, and it is the cheaper side.

- **The override seam does not exist.** `DEC-102` identified and *deferred* it:
  `customization` is parsed and displayed with no execution consumer outside the
  library view, and design assets resolve straight from the embed
  (`install::asset_text`), so overriding one requires a rebuild. If the contract
  lands in prose that projects may supersede, this slice inherits that deferred
  work — project-path lookup, framework fallback, precedence, fingerprint scope.
  Whether it can deliver a seam narrowed to one asset class, rather than the
  general one, is a scoping question for `/design`.
- **There is no prose interpolation anywhere.** The only interpolation in the
  subsystem is `runbook.rs`'s closed `PLACEHOLDERS` (`slice`, `run`, `repo_root`,
  `step`) substituted into verifier **argv elements**. It is a good precedent for
  the shape — closed vocabulary, unknown placeholder refused at parse — and has
  no prose consumer to extend.
- **Three prose systems, three loaders.** Hymns load through the cascade
  (`prompt check`), the design prompt pack (`Fragment`) resolves from the embed,
  and runbooks needed a second loader and validation domain of their own
  (`DEC-101`). Condition contracts must not become a fourth without argument.
- **Behaviour-preservation gate.** The design-run e2e suites
  (`tests/e2e_design_*.rs`) currently encode the claimed-condition path, including
  tests that bind conditions to `sec-01`. Some will legitimately change; each such
  change needs to be a deliberate, argued edit rather than a green-chasing one.

## Open Questions

Carried into `/design`, not answered here.

> **Dispositioned 2026-08-03 — end of the inquiry stage.** All eight are closed.
> The questions stay as written; this is the answer key, and the records hold the
> reasoning.
>
> | | question | disposition |
> |---|---|---|
> | `OQ-1` | what a contract consists of, and where it lives | `DEC-122` (embedded prose, `fixed`) + `DEC-123` (structure in a const table) |
> | `OQ-2` | is derived-vs-claimed the right partition | `DEC-120` — no; three kinds, and Claimed is the defect class |
> | `OQ-3` | `ISS-285`'s specify / derive / retire fork | `DEC-121` — none of the three; both become Attested user checkpoints |
> | `OQ-4` | where contract prose lives | `DEC-122` |
> | `OQ-5` | does a contract-carrying condition subsume `ISS-286`'s subject rule | yes — `DEC-121` + `DEC-126`, corroborated by `EVD-012`. Not a separate repair |
> | `OQ-6` | does `DEC-102`'s seal criterion coincide with `is_derived()` | **dropped, not answered.** Research thread 1 showed the coincidence is analytic; `DEC-120` removed the boolean. It sequences nothing |
> | `OQ-7` | delivered vs fetchable | `DEC-124` — by channel: refusal takes the remedy, the stage-entry receipt takes the contract, no digest |
> | `OQ-8` | how much of the override seam this slice delivers | `DEC-122` — none. Embedded and `fixed` defers it honestly |
>
> **`OQ-9`, raised and closed the same day — `DEC-127`.** Objective 6's
> **audience**, which decides where the diagram lives. Raised because research
> thread 4's champion was a generated sibling artefact in `SPEC-029`'s directory,
> costing no manifest entry, no embed change and no flake graft.
>
> **Closed against that champion.** A client repo has no access to this repo's
> spec — `.doctrine/` is not distributed — and objective 6's primary audience,
> the agent standing at the gate, is *always* a repo-external consumer. So the
> diagram ships on the **published** surface (embedded under `install/`, declared
> in `publication/manifest.toml`, reachable by `doctrine library show`) and it is
> **generated**, golden-pinned to `gate.rs`'s tables. One artefact, not two:
> publication answers *can the reader reach it*, generation answers *is what they
> reach true*, and an external consumer cannot check freshness for itself, so
> freshness must be structural rather than promised.
>
> The citation direction inverts from thread 4's proposal: `SPEC-029` cites the
> published address, because here the **spec** is the private artefact.
>
> **General rule, and a debt.** No shipped asset may cite a repo-private artefact
> — not a path, not an entity id. Entity ids are per-repo sequential, so a shipped
> asset citing `DEC-101` does not dangle in a client repo, it resolves to an
> unrelated record silently. `ISS-309` records that the corpus already violates
> this and owns the sweep plus the check that stops it returning.

- **OQ-1** — What does a condition's contract consist of, and where does it live —
  Rust-side data, or a correspondence to prompt prose? If prose: what must it
  interpolate, and how is the correspondence enforced (every condition has prose,
  and the prose describes the check that exists)?
- **OQ-2** — Is "derived vs claimed" the right partition, or does the vocabulary
  want three kinds (derived / attested-by-human / retired)? `UserAcceptsSufficiency`
  and `UserAcceptanceAttested` sit on opposite sides of `is_derived()` today while
  arguably being the same kind of act.
- **OQ-3** — `ISS-285`'s fork: specify, derive, or retire the exploring pair. The
  issue calls retirement "cleanest and the most disruptive".
- **OQ-4** — If a condition carries requirement prose, where does that prose live —
  Rust const data, an asset, or the spec — given `DEC-077`'s prose/mechanism split
  and the recompile-per-edit cost it weighs?
- **OQ-5** — Does a contract-carrying `Condition` subsume `ISS-286`'s subject rule,
  or is subject resolution a separate repair?
- **OQ-6** — Does `DEC-102`'s seal criterion, applied condition by condition,
  coincide with `is_derived()`? The working hypothesis says yes. If it holds, one
  line governs the vocabulary, the asset policy, and `ISS-285`'s fork; if it
  fails at any member, that member is where the interesting design is.
- **OQ-7** — Is a contract **delivered** (pushed on the turn envelope at the
  moment it applies, competing with the byte budget) or **fetchable** (pulled on
  demand, helping only an agent that knows to ask)? Probably both with different
  content — but which is which is a decision, not a default.
- **OQ-8** — How much of `DEC-102`'s deferred override seam must this slice
  deliver, and can it be narrowed to one asset class without prejudicing the
  general seam?

## Verification / Closure Intent

- An agent can learn what a stage transition requires **without reading
  `src/**` or the test suite** — the acceptance test for the whole slice, and the
  one the RFC-026 E8.7 instrument measures.
- A refusal names a remedy, not only an objection, for every condition it reports.
  `verify research-current` is the in-repo bar.
- Every condition is either derived from framework-owned state, explicitly
  attested as a human act, or retired — with no condition remaining that is
  satisfied by an unexamined claim against an unrelated subject, and none
  specified but left unenforced (`DEC-139`: every surviving row guards its edge,
  so there is no activation axis to carry an exception).
- The design-run suites stay green, with each intentional test change argued in
  the phase record rather than adjusted to fit.
- `ISS-285` and `ISS-286` are closable on this slice's evidence; `IMP-390`'s
  gate-condition face is discharged and its other faces remain open and
  attributed.

## Summary

<!-- Filled at close. -->

## Follow-Ups

<!-- Filled as they surface. -->
