# Review RV-344 — design of SL-244

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

### What this review is *not*

Not an inquisition, and not a per-section correctness pass. Every one of the six
sections has already been through repeated external adversarial review (codex),
and those passes are the reason the design reads as it does. Re-running that lens
would re-find what is already fixed and burn attention the design's reader does
not have left.

What has **never** been done is a pass over the document *as one artefact*, and a
pass against the *terrain it lands in*. Those are the two this ledger carries.

### Pass 1 — architecture and internal coherence (now, pre-lock)

Reads the design doc plus the knowledge records it cites. No code-terrain claim.

Lines of attack:

1. **Internal coherence across sections.** The document was written
   section-by-section over many turns, and each section revises what the ones
   before it assumed — `sec-4` retires three types `sec-3` reasoned over,
   `sec-5` corrects `DEC-124`'s arithmetic, `sec-6` re-pins `DEC-127`'s citation.
   Does any earlier section still assert what a later one withdrew? Does any
   *forward* promise ("`sec-5` carries what the retirement costs") go unkept?
2. **Modelling and type appropriateness.** Three record shapes for eight acts,
   `Coverage`/`CoveredSet` as separate rule-vs-record types, `Unmet`/`Cause`
   vs `ActFault` deliberately unmerged, `ObservedFact` with one member and no
   enforced consumer. Are these the right cuts, or is any of them a distinction
   the program cannot use? The design argues each; the question is whether the
   argument survives being read against the others at once.
3. **Against the requirements.** `SL-244`'s six scope objectives, and whether
   the design delivers each — or has quietly narrowed one. Objective 6 is the
   only one a single section owns; the other five are distributed.
4. **Against the decisions.** Every `DEC-` the design cites, and whether the
   design obeys, correctly amends, or silently contradicts it. The design amends
   `DEC-124` once explicitly and re-reads `DEC-127`'s pin once; those are the
   declared cases. Undeclared ones are findings.
5. **Governance.** `ADR-001` (leaf ← engine ← command, no cycles — the new
   `artifact.rs` claims leaf tier with out-degree into `gate` only), `ADR-004`
   (relations outbound-only — the `GovernanceEdges` projection depends on it),
   `ADR-010` (unify the contract and write seam, keep storage bespoke — `sec-4`
   narrows this rule and the narrowing is load-bearing), `STD-001` (the kebab
   token is single-sourced in four places), `POL-002`.
6. **Overclaim.** Places where the prose asserts more than the mechanism
   delivers. The design has a stated habit of catching these about itself
   (`review policy is a declaration of intent, not a security boundary`); the
   question is whether it caught them all.

### Pass 2 — landing terrain (after nominal lock)

Reads the design against `src/design_run/**`, `src/commands/design.rs`,
`src/review.rs` and the shipped asset corpus.

Lines of attack:

1. **Parallel implementation risk.** The design adds `CheckpointAct`,
   `AgentDeclaration`, `ObservedReview`, `ObservedFact`, `Unmet`/`Cause`,
   `ActFault` and `artifact.rs`. Does any of them duplicate a seam that already
   exists — in `design_run`, in `review.rs`, in `funnel_machine.rs`, or in the
   entity engine? `ADR-010`'s rule and the project's no-parallel-implementation
   standing order are the pins.
2. **Situational awareness.** Every in-tree citation the design load-bears on
   (`attestation.rs:36-41`, `facts.rs:95-99`, `snapshot.rs:419-433`,
   `refusal.rs:166-170`, `run.rs:1520-1536`, `envelope.rs:809-822`,
   `design.rs:1832-1851`, `prompt.rs:188-195`, `review.rs:1490`,
   `asset_source.rs:126-148`, …). Does each say what the design says it says?
3. **Reachability of what it assumes exists.** `IMP-392`'s finding set and
   concluded marker, `IMP-391`'s checkpoints, `DEC-073`'s policy — the design
   specifies against all three. Is each dependency correctly characterised as
   *unbuilt* rather than *misread*?
4. **Selector coverage.** The 19 recorded `design-target` selectors against what
   the implementation would actually have to touch.

### Standing note on the incumbent

The run itself is a specimen. Advancing `drafting → reviewing` required
re-claiming all six cumulative conditions, four of which had gone stale because
sec-1 and sec-2's *prose* was revised — including `user-accepts-sufficiency`, a
user act about the *inquiry map*, invalidated by an edit to a section. Under this
design that invalidation does not happen. Worth holding as evidence that the
model is aimed at a real defect, not as a finding.
