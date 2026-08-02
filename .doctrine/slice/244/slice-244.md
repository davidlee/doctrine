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

Give `Condition` the data it needs to explain itself: what the condition requires,
what subject kind it binds to, and how it is discharged. This is enrichment of a
closed vocabulary by its own authors — no open-vocabulary term is narrowed into
it, and the `DEC-101` rule is untouched.

The shape of that payload (associated const data, a trait, a table beside
`boundary_conditions`) is `/design`'s decision, not fixed here. What scope fixes
is that the nine words at `design.md:268` become data the program can reach and
render, in the register `mem.pattern.design.classify-at-authoring-not-from-behaviour`
prescribes: the artefact states the property, rather than a run inferring it.

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

`ISS-285` offers three options for `GoverningContextRecorded` and
`InitialConcernsRecorded`: specify them with a subject rule that means something,
derive them from framework-owned run state, or retire them and let the runbook
guard the edge alone. Deciding is `/design`'s work; this slice owns the decision
and its consequences, including `ISS-286`'s subject-rule half.

### 5. Make the requirement readable before it is violated

Surface a stage's unmet conditions — and what would satisfy them — on the turn
envelope and in the refusal, so an agent learns the contract without reading
source. Bounded by the envelope's existing byte budget.

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
- **Behaviour-preservation gate.** The design-run e2e suites
  (`tests/e2e_design_*.rs`) currently encode the claimed-condition path, including
  tests that bind conditions to `sec-01`. Some will legitimately change; each such
  change needs to be a deliberate, argued edit rather than a green-chasing one.

## Open Questions

Carried into `/design`, not answered here.

- **OQ-1** — What does a condition's contract consist of? Requirement text alone,
  or also a subject rule, a discharge procedure, and a remedy string?
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
