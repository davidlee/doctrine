# CLI-managed design runs and inquiry maps

## Context

The shipped `/design` skill declares a substantial workflow state machine, but
the agent is currently responsible for interpreting that machine, remembering
its cursor, activating nested behaviours, collecting durable results, and
recovering after compaction or a session boundary. Even frontier models follow
that contract unevenly, and the user cannot tell whether a prolonged design
interview is traversing a coherent inquiry structure or merely following local
conversational momentum.

RFC-021 accepts Doctrine ownership of canonical behaviour definitions, local
workflow semantics, resolution, and validation, while leaving the concrete
behaviour protocol and activation architecture to narrower contracts and
experiments. `/design` is a useful vertical slice because its current process is
complex, iterative, and partly deterministic while still requiring substantial
agent judgement.

SL-231 provides positive empirical evidence for one part of the approach. During
its design session, incremental `DEC`, `QUE`, and related records preserved the
semantic state of the design across multiple context compactions before
`design.md` was written. The resulting relation graph was not merely a final
index: it acted as durable working memory. This slice productises that useful
behaviour while keeping provisional inquiry structure separate from accepted
epistemic records.

## Scope & Objectives

### 1. Design-specific run coordination

- Introduce a Doctrine-owned run for the existing `/design` workflow, scoped to
  one slice and carrying stable run identity, current state, pending obligation,
  refresh boundary, and idempotent submission identity.
- Make Doctrine derive repository, slice-lifecycle, configuration, and linked
  record facts it can know mechanically rather than asking the agent to restate
  them.
- Resolve the next compact prompt fragment and transition contract from current
  run state while leaving open-ended inquiry, synthesis, drafting, and
  adversarial reasoning to the agent.
- Provide status, resume, and compact rehydration views sufficient to recover
  after context compaction or a new session without replaying the transcript.

### 2. Lightweight inquiry map

- Maintain a run-local, structured inquiry map representing the agent's current
  decomposition of open design questions.
- Give nodes stable run-local identity, one primary parent, concise question
  text, lifecycle status, and sparse dependency or activation references where
  a strict tree is insufficient.
- Make the active path, nearby frontier, blockers, resolved/open counts, and
  material map changes visible to the user without injecting the full map on
  every turn.
- Support a bounded adaptive traversal default: establish major branches
  breadth-first, pursue the most consequential or blocking arm depth-first, then
  reassess the frontier. Permit immediate user pin, defer, prune, breadth, and
  depth direction during the run.
- Treat the map as an inspectable agent proposal, not authored design truth.
  User-pinned or locked direction is distinguished from agent-proposed
  structure.

### 3. Incremental semantic checkpoints

- Integrate accepted design outcomes with the existing knowledge-record surface:
  unresolved questions become or retain `QUE` records, accepted design choices
  become `DEC` records, and carried assumptions become `ASM` records where
  those established semantics fit.
- Link durable checkpoints and evidence to the owning slice so Doctrine can
  reconstruct the accepted design state incrementally.
- Require each meaningful workflow checkpoint to disposition its semantic
  result by creating a record, linking an existing record, retaining an
  unresolved result, or explicitly marking the exchange non-durable. Do not
  manufacture a record for every conversational turn.
- Make creation and adoption one recoverable managed operation: reserve and
  journal new canonical identity before authored materialisation, or accept a
  supplied existing canonical record without duplication.
- Separate semantic content from a content-bound user-acceptance attestation;
  agent-authored payloads cannot declare their own accepted status.
- Keep section drafts, traversal state, delivery receipts, and other provisional
  process data out of the knowledge graph unless the design establishes an
  authored semantic reason for them.

### 4. Continuity and bounded delegation seam

- Define a compact handover/resume projection containing the active path,
  accepted decisions, open questions, assumptions, evidence references,
  blockers, and next obligation.
- Define the protocol boundary for assigning one bounded inquiry obligation to
  another session or agent and accepting an attributed proposal back into the
  coordinating run.
- Keep global transition authority with the coordinating run; a delegated
  worker proposes results and map changes but does not advance the workflow
  independently.
- Permit explicit conservative entry from an existing `design.md`: import
  sections as unreviewed drafts, direct non-terminal shaping QUEs as durable
  inquiries, and conventional `OQ-*` entries only as unverified prose
  proposals. Never infer procedural evidence or deduplicate by text similarity.

### 5. Thin skill adapter and governance descent

- Retain the existing static `/design` activation surface while it delegates
  deterministic mechanism to Doctrine. This slice does not use successful
  dynamic delivery as evidence that privileged activation can be removed.
- Reuse the prompt cascade where it fits rather than creating a parallel
  composition mechanism.
- Descend RFC-021 into the minimum product and technical contracts required
  before implementation, or revise the appropriate existing specifications if
  design establishes that they already own the boundary.
- Preserve the existing v1 review choreography: section-level human alignment
  followed by an integrated adversarial pass. Other exposition and review
  postures remain extension seams rather than v1 scope.

### Affected surface

- Behaviour and prompt guidance for `/design`, principally
  `plugins/doctrine/skills/design/**` and any framework-owned prompt fragments
  established by design.
- New or extended CLI, pure workflow semantics, runtime-state persistence, and
  rendering under `src/**`.
- Focused workflow, recovery, projection, hostile-input, and CLI tests under
  `src/**` and `tests/**`.
- Runtime artifacts beneath `.doctrine/state/**`; exact homes and schemas are
  design decisions.
- Product/technical specification and requirement artifacts needed to govern
  the behaviour-run contract.

## Non-Goals

- A generic workflow interpreter, autonomous design engine, or generalized
  recovery planner.
- Replacing the complete skill catalog, selecting RFC-021's final activation
  architecture, or removing privileged `/design` activation metadata.
- Moving other Doctrine skills onto the protocol in this slice.
- A new generic design-event entity or a parallel replacement for `DEC`, `QUE`,
  `ASM`, review, evidence, or observation records.
- Treating the SL-231 observation ledger as workflow state; adherence and
  friction observations may inform evaluation, but their aggregation and
  interpretation remain consumer-owned.
- A generalized posture algebra. Exposition/education modes and alternative
  human-versus-automated review choreography are deferred beyond preserving
  explicit extension seams.
- Harness-specific worker spawning, a general cross-filesystem write broker, or
  broad MCP mutation authority. The protocol boundary may be defined without
  delivering every transport in v1.
- A graphical design-tree UI or project-defined behaviour language.

## Risks, assumptions, and resolved design boundaries

- **R1 — ceremony and token tax.** A map or checkpoint protocol that costs more
  attention than it saves will train users and agents to bypass it. Compact
  projections and small deltas are core acceptance concerns.
- **R2 — false legibility.** A tidy tree can create confidence without improving
  reasoning. The map must expose provenance, material restructuring, unresolved
  branches, and unexplained traversal changes rather than certify coherence.
- **R3 — state-tier collapse.** Provisional inquiry state, accepted epistemic
  truth, raw adherence observations, and final design prose have different
  lifecycles and must not be folded into one convenient store.
- **R4 — authority laundering.** Agent-authored decomposition and proposed
  classifications remain proposals. Doctrine-derived facts, direct user
  direction, accepted decisions, and agent recommendations retain distinct
  provenance.
- **R5 — platform sprawl.** `/design` must remain the proving vertical slice;
  attractive general behaviour abstractions are follow-up candidates until a
  contrasting workflow supplies evidence that they generalise.
- **A1 — incremental records are useful.** The SL-231 session is sufficient
  evidence to treat semantic checkpointing as a promising mechanism, while the
  inquiry-map benefit remains a hypothesis to test.
- **A2 — static activation remains.** Existing skill activation is retained
  during the experiment, consistent with RFC-021's separation of behaviour
  ownership from activation policy.
- **A3 — existing epistemic kinds are the first durable sink.** `DEC`, `QUE`,
  and `ASM` are reused unless design demonstrates a semantic mismatch.
- **B1 — runtime recovery.** A schema-versioned, revision-guarded snapshot owns
  exact resume; authored records support explicitly weaker reconstruction.
- **B2 — specification boundary.** A narrow product and technical contract
  descend RFC-021. Exactly two existing specifications are amended, each within
  its own ownership: SPEC-023 gains one sealed hymn entry, and SPEC-019 gains an
  acknowledgement that a managed design run is a legitimate provenance for
  DEC/QUE/ASM records. No skill contract is amended — no specification owns a
  skill body, so the design skill's rewrite is implementation against a source
  target.
- **B3 — inquiry topology.** One primary parent plus sparse `needs`; lifecycle,
  cursor, and traversal remain orthogonal.
- **B4 — checkpoint admission.** A resolved node has an explicit semantic
  disposition. Accepted truth additionally requires a content-bound
  user-acceptance attestation.
- **B5 — delegation.** V1 defines transport-neutral attributed proposals while
  the coordinator remains sole writer; it does not spawn or broker.
- **B6 — prompt boundary.** One invariant stage hymn, at most one coarse
  obligation fragment, and one dynamic TurnEnvelope form the delivered prompt.

## Verification and closure intent

- Exercise a representative prolonged design interview through the coordinator,
  including map expansion, user-directed traversal, a resolved question, a
  retained open question, an accepted decision, and a non-durable disposition.
- Demonstrate recovery in a fresh context from Doctrine-owned state and linked
  records without replaying the transcript or manually reconstructing the
  inquiry frontier.
- Demonstrate that compact next-step projections remain bounded as the full map
  and semantic record graph grow.
- Demonstrate that user-pinned direction overrides the adaptive traversal
  default and that material agent-authored restructuring remains visible.
- Preserve existing slice lifecycle, knowledge-record, prompt-cascade, and
  shipped-skill behaviour outside the declared `/design` path.
- Deliver and mechanically verify the deterministic evaluation fixture,
  moderator protocol, rubric, evidence collectors, and assertions needed to
  exercise the relevant RFC-021 stages. These artifacts gate SL-233 closure.
  The live adopt/adhere/refresh/recover/complete agent exercise runs
  immediately post-close under CHR-049, once the changed skill and prompt
  assets are genuinely installed; successful outcome alone is not proof.

## Summary

Prove a CLI-managed `/design` run that externalises deterministic workflow
state, maintains a visible lightweight inquiry map, and incrementally links
accepted semantic checkpoints into Doctrine's existing knowledge graph. Keep
activation static, review choreography stable, and general behaviour machinery
out of v1 so the experiment can establish adherence, continuity, user steering,
and token-economy value before broader generalisation.

## Follow-Ups

- General behaviour identity, composition, resolution, validity, and protocol
  contracts for additional workflows under RFC-021.
- Activator experiments and privileged-catalog reduction based on measured
  notice/select/invoke/adopt/adherence evidence.
- Exposition postures and configurable human-versus-automated review
  choreography.
- Broader worker transports, MCP brokering, and orchestrator-managed delegation.
- Observation-based adherence and friction aggregation once the SL-231
  collection substrate and downstream analysis capability are available.
