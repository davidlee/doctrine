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

Extend the `ReviewStanding` pattern leftward to the conditions the snapshot can
already answer — `RequiredSectionsExist`, `MaterialisationCurrent`, and
`BlockingInquiriesDispositioned` are framework-owned run state and derivable
without crossing any vocabulary boundary.

### 3. Attest what cannot be derived

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

### 5. Make the requirement readable before it is violated

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

## Risks & Assumptions

- **Snapshot versioning.** `DerivedDesignFacts` persists to the run snapshot. If a
  condition's payload or the evidence shape changes, the snapshot schema version
  moves and existing runs are rejected at parse. The tier is gitignored and
  documented disposable, and one live run exists (SL-243's), so the cost is that
  run — real, bounded, and worth naming before it is paid.
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
  satisfied by an unexamined claim against an unrelated subject.
- The design-run suites stay green, with each intentional test change argued in
  the phase record rather than adjusted to fit.
- `ISS-285` and `ISS-286` are closable on this slice's evidence; `IMP-390`'s
  gate-condition face is discharged and its other faces remain open and
  attributed.

## Summary

<!-- Filled at close. -->

## Follow-Ups

<!-- Filled as they surface. -->
