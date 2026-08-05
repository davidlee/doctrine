# QUE-206: Is the interview substrate separable from slice design

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## The question

Can — and should — the machinery that drives a structured, resumable agent–human
interview be factored out of the slice-design workflow, so that it either serves
an ad-hoc role anywhere it is useful, or at minimum serves a nominated set of
entity contexts (ADR, RFC, policy, product/tech spec)?

Two halves, and they fail independently:

- **Can it.** Is there a seam in `src/design_run/**` that separates a
  workflow-agnostic inquiry substrate from design-stage policy without a
  parallel implementation or a leaky parameterisation?
- **Should it.** Is a second consumer real enough to pay for the abstraction,
  and does an extracted substrate stay coherent when the thing it interviews
  about is not a design document?

## Why it is live now

The design-run system is large and the interview graph — the inquiry map, its
traversal, its resumable projection — is the part with the most obvious life
outside the workflow that grew it. Every governance kind Doctrine ships involves
the same shape: an agent proposes structure, a human directs and answers, the
exchange must survive compaction and a cold context, and the outcome must land
in a durable record rather than in chat scrollback. Today only `/design` gets
that; `/spec-tech`, `/spec-product`, ADR authoring and RFC shaping run the same
interaction as unstructured prose discipline.

SL-244 is the first round of change into this system, and its subject — a gate
condition that carries its own contract instead of being a fieldless enum
variant known only to `gate.rs` — moves policy from hard-coded knowledge toward
described knowledge. That direction plausibly *changes the answer here*: a gate
whose conditions describe themselves is closer to a substrate that a second
workflow could supply conditions to. Whether that is a real consequence or a
coincidence of shape is part of the question.

## Candidate seam

Sketch only — this is what the question is asking to be tested, not a proposal:

**Plausibly generic.** The inquiry map (nodes, provenance, prerequisite
cross-links per DEC-061), traversal and the ready/blocked frontier, the
revisioned runtime snapshot (DEC-059), the continuity projection for cold
resume, staleness cascade on a changed answer (IMP-386), and the disposition
requirement that a resolved node cannot resolve into silence (DEC-062).

**Plausibly design-specific.** The stage machine and its gate conditions,
section drafts and their fingerprints, the derived slice/repository/config fact
supply, the section-alignment-then-adversarial review choreography, attestation
and lock, and conservative import from an already-authored `design.md`.

The seam is not obviously clean: evidence subjects resolve from draft sections
today, which is exactly the coupling SL-244 is unpicking, and checkpointing
targets the epistemic record kinds (DEC/QUE/ASM) — which is *already*
workflow-neutral and is the strongest existing signal that a substrate exists.

## The hydra contrast

`jkaloger/hydra` (SPEC.md) is the same idea taken to the opposite extreme:
generic by construction, a durable decision tree in one git-tracked JSON file
per tree, addressed by short kebab handles, with a `sprout` / `cut` /
`cauterise` / `reopen` / `reparent` / `link` mutation surface, `next` over a
pre-order ready frontier, and a three-part `resume` (intent verbatim, whole-tree
skeleton, hydrated detail for `next` and its ancestors). Re-answering a head
transitively reopens descendants and dependents, keeping the old answer in
`prior`.

Its stated posture is the load-bearing difference: *"Hydra is a store with
invariants, not a policy engine."* It never reads question text, never rates
options, never dictates interview order. Doctrine's design run is the inverse —
its value is concentrated in exactly the policy hydra refuses (derived facts,
gates, attestation, disposition into typed records). So "factor out the
substrate" is not "build hydra"; it is the question of whether Doctrine's
substrate can be split along a hydra-like line while the policy that justifies
Doctrine's existence stays on the other side of it, per-workflow.

If the answer is that the generic residue *is* essentially hydra, that is a real
finding: it would mean the extraction buys little, because the valuable part
does not travel.

## What governance already says

PRD-019 (Managed design workflow) forecloses this by default rather than
leaving it open. Its out-of-scope names "moving other workflows onto the
protocol — design is the proving vertical, and generalisation waits on a second
workflow supplying evidence that anything here transfers", and it carries the
constraint: *"The capability is scoped to one workflow. Any abstraction claimed
to generalise must be justified by a second workflow, not by this one
succeeding."*

That is a discharge condition, not an answer. It says an argument from design's
success is inadmissible; it does not say the substrate is inseparable, and it
does not name which second workflow should be tried or what a cheap trial looks
like. This record holds the question PRD-019 deliberately left standing.

## What would settle it

Any one of these is admissible evidence; none has been produced:

1. A named second consumer carried far enough to expose which parts of the run
   it needs and which it must ignore — ADR authoring is the cheapest candidate
   (small, decision-shaped, already checkpoints into DEC).
2. A demonstration that the design-specific set above is small and injectable —
   or that it is not, which answers the question negatively and closes it.
3. Evidence from the ad-hoc direction: a real interview held outside any entity
   context (`/consult`, `/preflight`, an unscoped shaping conversation) that
   wanted resumable structure and had none.

Answering negatively is a legitimate outcome and should be recorded as such
rather than left open: "the substrate is the workflow" is a finding.

## Related

- PRD-019 — scopes the capability to one workflow and sets the discharge bar.
- SL-244 — first round of change into the design-run system; its
  self-describing gate conditions bear on separability.
- SL-233 — shipped CLI-managed design runs and inquiry maps.
- DEC-059, DEC-061, DEC-062 — snapshot, cross-link and disposition decisions
  that any extracted substrate would have to carry.
- IMP-386 — staleness cascade from a changed answer, the analogue of hydra's
  transitive reopen.
