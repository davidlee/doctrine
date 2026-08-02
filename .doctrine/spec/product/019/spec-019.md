# PRD-019: Managed design workflow

<!-- Reference forms: entity ids padded (REQ-059, ADR-004); doc-local refs bare
     (OQ-1 open question). See .doctrine/glossary.md § reference forms. -->

## 1. Intent

Doctrine's design stage declares a substantial workflow — states, obligations,
nested behaviours, and durable outcomes — but the agent carries all of it. It
interprets the state machine, remembers its own cursor, decides when a nested
behaviour applies, collects the durable results, and reconstructs everything
after a context compaction or a session boundary. Even frontier models follow
that contract unevenly, and the failure is quiet: a design interview that has
drifted into local conversational momentum looks, turn by turn, exactly like one
traversing a coherent inquiry structure. The user cannot tell which they are in.

This capability moves the deterministic half of that workload to Doctrine. The
framework holds the run's state, prescribes the next obligation, derives the
facts it can know mechanically, and gives the user a legible view of what is
open, what is blocked, and where the conversation currently is. The agent keeps
the half that is genuinely judgement — inquiry, synthesis, drafting, adversarial
reasoning.

The second problem is loss. Design sessions outlive context windows. Today the
semantic state of a long design lives in the transcript until `design.md` is
written, so a compaction can cost hours of settled reasoning. Incremental
epistemic records already demonstrated the fix in practice: during SL-231, the
decision and question records written as the design progressed preserved its
semantic state across multiple compactions and acted as durable working memory
rather than a closing index. This capability productises that behaviour while
keeping provisional inquiry structure separate from accepted epistemic truth.

The desired end state: a design run that a user can inspect at any moment, that
an agent can resume from without the transcript, and whose accepted outcomes are
already durable records by the time the design document is written.

This capability descends RFC-021, which accepts Doctrine ownership of canonical
behaviour definitions, local workflow semantics, resolution, and validation
while leaving activation architecture and the general behaviour protocol to
narrower contracts and experiments.

## 2. Scope

**In scope.** One managed workflow — the design stage — scoped to one slice at a
time. Run coordination: identity, stage, pending obligation, refresh boundary,
idempotent submission. Mechanical derivation of repository, slice-lifecycle,
configuration, and linked-record facts. A run-local inquiry map with provenance
and directable traversal. Semantic checkpointing into the established epistemic
record kinds. A continuity projection for compaction and session boundaries. The
protocol boundary for delegating a bounded obligation and accepting an attributed
proposal back. Conservative entry from an already-authored design document. The
section-alignment-then-adversarial-pass review choreography and the acceptance
that makes a lock legitimate.

**Out of scope.** A generic workflow interpreter, an autonomous design engine, or
a generalized recovery planner. Moving other workflows onto the protocol —
design is the proving vertical, and generalisation waits on a second workflow
supplying evidence that anything here transfers. Changing how behaviours are
activated: existing static activation is retained, and successful dynamic
delivery within this capability is not evidence that privileged activation can be
reduced. A new epistemic record kind, or any parallel replacement for the
decision, question, assumption, review, or evidence records that already exist.
A generalized posture algebra — exposition and education modes, and alternative
human-versus-automated review choreography, remain extension seams. Harness-
specific worker spawning or a general cross-filesystem write broker. A graphical
design-tree interface, and any project-defined behaviour language.

**Boundaries.** The map is an inspectable agent proposal, never authored design
truth. The design document remains the authored artefact; the run coordinates its
production and does not replace it. Adherence and friction signals may inform
evaluation of this capability, but their aggregation and interpretation belong to
whoever consumes them, not to the run.

## 3. Principles

- **Doctrine does not ask what it can derive.** A fact the framework can read
  from the repository, the slice, or the configuration is supplied, never
  requested. Asking the agent to restate it spends attention and invites drift.
- **Decomposition is a proposal until a human says otherwise.** Agent-proposed
  structure, Doctrine-derived fact, direct user direction, and attested
  acceptance are four different things and stay distinguishable. Collapsing them
  launders authority.
- **A payload cannot declare its own acceptance.** Accepted truth requires an
  attestation bound to the content accepted. Semantic content and the assertion
  that a human accepted it are separate acts.
- **Provisional state and accepted truth do not share a store.** Traversal
  cursors, section drafts, and delivery receipts have a different lifecycle from
  epistemic records, and folding them together costs the ability to discard one
  without the other.
- **A legible map is not a correct one.** Structure that looks tidy can raise
  confidence without improving reasoning. The map's job is to expose unresolved
  branches, material restructuring, and unexplained traversal changes — not to
  certify coherence.
- **Ceremony that costs more than it saves is a defect.** A protocol users and
  agents route around has failed, whatever its other properties. Compact
  projections and small deltas are correctness concerns here, not tuning.

## 4. Requirements

The functional and quality requirements this capability must satisfy are recorded
as requirement entities and appear under the synthesized Requirements section
below. This section carries only the constraints and invariants that bound every
valid implementation.

Constraints:

- The durable sink for accepted design outcomes must be the established
  decision, question, and assumption record kinds, unless a semantic mismatch is
  demonstrated rather than assumed.
- No mechanism may require reducing or removing privileged static activation of
  the design behaviour.
- Global transition authority stays with the coordinating run: a delegated worker
  proposes results and map changes, and never advances the workflow itself.
- Conservative entry from an authored design may never infer procedural evidence
  or deduplicate by text similarity — an imported section is an unreviewed draft
  until something says otherwise.
- The capability is scoped to one workflow. Any abstraction claimed to generalise
  must be justified by a second workflow, not by this one succeeding.

Invariants:

- Every resolved inquiry carries exactly one explicit semantic disposition; a
  node cannot resolve into silence.
- Authored knowledge is never deleted or rolled back to repair a runtime failure.
- A canonical identity, once reserved and journalled, is resumed on recovery and
  never re-minted — recovery does not invent identity.
- The provenance of a map node survives every restructuring: user-directed
  structure never becomes indistinguishable from agent-proposed structure.
- The run's view of the authored design is either current or refused; the
  workflow never advances on state it has silently diverged from.

## 5. Success Measures

- A user, at any point in a long design session, can see the active path, the
  nearby frontier, what is blocked, and what remains open — without asking the
  agent to summarise, and without the whole map being pushed into the turn.
- An agent resuming after a context compaction reaches the same next obligation
  the pre-compaction agent had, from the run's own projection rather than from
  transcript recall.
- The measured token cost of a normal turn is bounded by named limits and is
  recorded as a figure that can be compared across runs — the ceremony question
  is answered with a number, not an impression.
- The accepted outcomes of a design exist as durable records before the design
  document is written, so a session lost mid-design loses drafting work rather
  than settled reasoning.
- A reviewer can tell, from the run alone, which parts of a design were directed
  by the user, which were proposed by the agent, and which were accepted — and
  the three are not confusable.
- Agents and users do not route around the protocol. Sustained bypass is the
  signal that the ceremony cost exceeded its value, and it counts against this
  capability rather than against its users.

## 6. Behaviour

**Primary flow.** A run starts against a slice whose design is not yet locked.
Doctrine derives what it can, establishes the run's identity, and prescribes the
first obligation. The agent conducts inquiry, proposing map structure and
submitting results; each submission carries the run's identity, the revision the
agent believes it is acting on, and a submission identity that makes a retry
safe. Doctrine validates the whole submission before any of it lands, advances
the run, and prescribes the next obligation. As inquiries resolve, each carries a
semantic disposition — becoming a record, adopting an existing one, retaining an
explicitly unresolved outcome, or being marked non-durable. When inquiry is
sufficiently settled the run moves to drafting, then to review, where sections
are aligned with the human before an integrated adversarial pass. A lock requires
an explicit acceptance bound to the accepted content.

**Traversal.** The default is bounded and adaptive: establish the major branches
breadth-first, pursue the most consequential or blocking arm depth-first, then
reassess the frontier. The user may override at any point — pin an arm, defer
one, prune one, or switch posture — and those directions are recorded as
user-directed, not folded into the agent's proposed structure.

**Resume.** After a context compaction or a new session, the run projects a
compact continuity view: active path, accepted decisions, open questions,
assumptions, evidence references, blockers, and the next obligation. This is the
recovery path, and it does not require the transcript.

**Entry from an existing design.** A design document authored outside a managed
run can be entered conservatively: sections import as unreviewed drafts, and
non-terminal shaping questions become durable inquiries. Conventional inline
open-question entries import only as unverified prose proposals. Nothing is
inferred as evidence and nothing is merged on textual similarity.

**Delegation.** One bounded inquiry obligation may be assigned elsewhere and its
attributed proposal accepted back. The delegate proposes; the coordinating run
decides. This is a protocol boundary, not a spawning mechanism.

**Regression.** A run may move backward deliberately, carrying a recorded reason.
Returning forward re-earns every applicable gate against current content —
clearance is not inherited across a regression, and no ceremonial replay
substitutes for it.

**Guards and failure modes.** A submission naming a revision the run has moved
past is refused with a conflict report rather than silently merged. A retried
submission resumes; the same submission identity carrying different content is
refused; one presented outside the replay window is refused as expired rather
than treated as new. A validation failure leaves the run's state exactly as it
was. If the authored design has changed underneath the run, entry is refused
until the divergence is dispositioned, and a change arriving between validation
and the write abandons that write rather than advancing on stale state. In every
case, effects already journalled remain and stay recoverable — the guarantee is
that the run does not advance, not that nothing happened.

## 7. Verification

The workflow's own semantics — legal stage moves, regression, inquiry lifecycle,
cycle refusal, derived-not-stored blocking, and the refusal of a resolved
inquiry that carries no disposition — are verified against values alone, with
derived facts injected. That layer touches no clock, randomness, repository, or
filesystem, so its verification is exhaustive rather than sampled, and the purity
itself is checked (REQ-436).

Coordination and recovery obligations are verified end to end against a real run:
schema-version rejection, revision conflict, submission-identity replay,
divergence refusal, the abandoned write, and recovery to a reserved identity.
Each failure mode is verified by the *state that survives it*, not merely by an
error being returned — a guard can be present and still self-refuse, so the
assertions compare against pre-call values (REQ-429, REQ-430, REQ-434).

The bounding obligation (REQ-424, REQ-437) is verified against a fixture large
enough to exceed every limit before projection; a bound whose fixture never
reaches it is unproven. The measured cost is recorded as a figure so that
successive runs are comparable, which is what makes the ceremony question
falsifiable rather than rhetorical.

Provenance and acceptance obligations (REQ-418, REQ-425, REQ-427) are verified
structurally: user-directed and agent-proposed structure are distinguishable by
type rather than by convention, and accepted status is unreachable without an
attestation bound to the accepted content. A test that can pass while those two
collapse is not verification of this capability.

Recovery-shaped obligations (REQ-416, REQ-431) require deterministic fault
injection at the points they claim to protect. Without a seam a test can drive,
the claims are untestable and any test asserting them is theatre.

The judgement half is not mechanically verifiable and is not claimed to be. Its
proof is a live human-in-the-loop exercise against installed bytes, after this
capability ships, comparing a managed design against the unmanaged baseline.

## 8. Open Questions

- **OQ-1 — discovery of applicable existing knowledge.** A checkpoint may adopt
  an existing canonical record instead of creating one, but reliably surfacing a
  relevant, not-yet-linked record at the right conversational moment is a
  retrieval-quality problem this capability does not solve. It offers known
  linked context and does not claim comprehensive discovery. Blocks any claim
  that adoption prevents duplicate records in general.
- **OQ-2 — authority of dynamically resolved guidance.** Boot establishes strong
  routing obligations, but the general question of how much authority
  dynamically delivered behaviour carries relative to privileged static context
  is left open by RFC-021 and is not settled here. It does not block this
  capability, but it bounds what a successful outcome may be taken to prove.
- **OQ-3 — whether the inquiry map earns its cost.** Incremental semantic
  checkpointing has positive evidence from a real design session. The inquiry
  map does not; it is a hypothesis that structured decomposition improves
  reasoning rather than merely displaying it. This is the question the evaluation
  exists to answer, and a negative answer should retire the map rather than
  motivate more of it.
