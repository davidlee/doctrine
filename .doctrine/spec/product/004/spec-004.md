# PRD-004: Memory

## 1. Intent

An agent or human working in a governed codebase keeps relearning the same things:
how a subsystem is shaped, which default is load-bearing, the sharp edge that bit
them last week, the convention the project actually follows. That knowledge is
stranded in chat transcripts that vanish at the end of a session, or in someone's
head, or scattered across docs nobody re-reads. Each fresh context pays the
rediscovery cost again, and worse, acts on stale or half-remembered understanding.

**Memory** answers this need: it makes hard-won knowledge a durable, first-class
artefact that survives the session that produced it and can be recalled — by the
right agent, in the right context, at the right moment. Its value is that
understanding compounds instead of evaporating. A memory carries not just a claim
but the context it was learned in and how far it can be trusted now, so a future
reader can judge whether it still holds rather than taking it on faith. The desired
end state is that an agent entering an unfamiliar subsystem is met with the relevant,
trustworthy, scoped knowledge a prior agent already paid to learn — and is never
silently misled by knowledge that has gone stale or was never true.

## 2. Scope

In scope:

- Declaring a durable unit of recalled knowledge — typed, titled, scoped, and
  attributed — that outlives the session that produced it.
- Capturing the context a memory was learned in (where it applies, what it was
  anchored to) so its present trustworthiness can be judged later.
- Recalling memories relevant to a working context, ordered deterministically, with
  the untrustworthy suppressed and the stale or unverified flagged rather than hidden.
- Carrying a memory through its lifecycle and recording its verification standing as a
  separate axis from that lifecycle.
- Surfacing recalled knowledge to an agent as attributed, quoted data, never as
  trusted instruction.

Out of scope:

- The substrate that persists the bytes and the particular retrieval algorithm — a
  memory's meaning is independent of how it is stored or matched; mechanism belongs to
  the technical specification.
- Proactive, unsolicited injection of memories into a context ahead of demand.
- Cross-client and cross-tenant federation, shared multi-writer durability, and the
  authenticity guarantees that fan-out requires.
- Automatic summarisation, reflection, or retention/erasure policy.
- Enforced graph linkage between memories as a recall mechanism.

Boundary: Memory owns the *shape of meaning* — what a recalled unit is, where it
applies, and how far it can be trusted — and deliberately does not own the
persistence substrate or the matching mechanism. A memory is knowledge to be judged,
never an instruction to be obeyed. Memory is also distinct from the epistemic and
governance records (PRD-010): a unit that must be cited, transitioned, superseded, or
used to govern work is a `knowledge_record`, while knowledge that only needs scoped
recall stays a memory — and one that becomes load-bearing is promoted by linking, never
mutated in place (PRD-010 §3).

## 3. Principles

- **Scope is the recall key.** A memory with no declared context cannot be found by a
  context-aware query; an actionable memory must say where it applies, or it is
  unreachable by design.
- **Anchor before you trust.** A memory's worth depends on the context it was learned
  in. Recall judges present trustworthiness against that anchor; it never assumes a
  claim still holds because it was once recorded.
- **Lifecycle is not verification.** Whether a memory is current is one axis; whether
  it has been checked against reality is another. The two are tracked separately and
  never collapsed.
- **Stored knowledge is hostile input.** Memory text is untrusted data on every
  substrate. It is rendered as attributed, quoted data and may never override the
  instructions of the system, operator, or agent reading it.
- **Recall is deterministic and explicit.** The same query in the same context yields
  the same ordering; an undecidable trust judgement resolves to an explicit stated
  standing, never a silent hide and never a silent over-trust.

## 4. Requirements

The functional and quality requirements this capability must satisfy are recorded as
requirement entities and appear under the synthesized Requirements section below.
This section carries only the constraints and invariants that bound every valid
implementation.

Constraints:

- A memory's identity must be stable for its whole life and collision-free across
  independent clones, without relying on a central allocator.
- Recall must judge trustworthiness only against the context a memory carries; it may
  never infer a context the recording did not establish.
- The recall ordering must be a total, reproducible function of the query and the
  candidate memories — no nondeterministic tiebreak.
- A memory's meaning must be expressible independently of any one persistence
  substrate or matching mechanism, so neither can be changed by reshaping meaning.

Invariants:

- A memory's durable identity never changes once minted, across edits, retries, and
  re-anchoring.
- Lifecycle standing and verification standing are always independent — neither value
  determines the other.
- A memory excluded for trust reasons (suppressed or quarantined) never reaches an
  agent's working context, while remaining recoverable for audit.
- Recalled knowledge is always presented as attributed data and never as instruction
  the reader is bound to follow.
- Every recall outcome carries an explicit trust standing; no candidate is silently
  hidden and none is silently over-trusted.

## 5. Success Measures

- An agent entering a subsystem it has not seen this session is presented with the
  relevant, trustworthy memories a prior agent already paid to learn — without
  recourse to chat history.
- The same query in the same context returns the same memories in the same order,
  every time, for any agent.
- A memory whose underlying context has moved on is recalled with an explicit stale or
  unverified standing, not silently presented as current.
- A suppressed, quarantined, or retracted memory never appears in an agent's working
  context, yet remains visible for audit.
- A reviewer can tell, for any memory, what it claims, where it applies, when it was
  last checked, and how far it can be trusted — from the memory alone.
- Two agents recording knowledge in independent clones never collide on a memory's
  identity.

## 6. Behaviour

Primary flow — record knowledge: an operator or agent declares a memory with its
type, title, claim, and the context it applies to. The system mints a durable
identity, captures the learning context as an anchor where one is available, and
persists the memory as a reviewable, durable artefact. The memory opens active and
unverified.

Primary flow — recall for a context: a caller asks for the memories relevant to a
working context. The system selects candidates whose declared scope matches that
context, orders them deterministically by relevance and trust, suppresses the
untrustworthy, and returns the survivors each carrying an explicit trust standing.

Lifecycle flow: a memory advances through its standing — active, then superseded,
retracted, archived, or quarantined — as knowledge changes. Verification standing
(unverified, verified, stale, disputed) advances independently as the memory is or is
not checked against reality.

Guard — trust suppression: a quarantined or retracted memory is withheld from any
agent-facing recall; a low-trust, high-risk memory is held back from automatic
surfacing even when it matches. Suppression removes a memory from working context but
never from the audit record.

Guard — hostile-input rendering: every recalled memory is surfaced as a quoted,
attributed block bearing its identity, trust standing, and context — never as text
that could be mistaken for an instruction to the reading agent.

Edge cases: a memory recorded with no declarable context is reachable only by direct
reference, not by context-aware recall; a memory whose anchoring context is no longer
decidable resolves to an explicit unknown or unanchored standing rather than a guess;
a sanctioned orientation memory minted upstream of any client repo is recallable in
every context and is treated as evergreen rather than decaying.

## 7. Verification

Verification confirms that a memory durably carries its claim and context, that
recall is deterministic and trust-aware, that lifecycle and verification standing stay
independent, and that stored knowledge can never act as instruction — without binding
the spec to a particular substrate or matching algorithm.

Durability and identity are proven by recording a memory and confirming its claim,
context, and minted identity persist across reads, edits, and retries, and that
independent recordings never collide on identity. Recall determinism is proven by
issuing the same query in the same context repeatedly and confirming an identical
ordering, and by exercising the trust ordering so that high surface relevance never
outranks verification, provenance, scope, or trust. Trust suppression is proven by
confirming quarantined and retracted memories are absent from agent-facing recall yet
present in the audit record. The lifecycle/verification independence is proven by
confirming a memory can hold any combination of the two standings. The hostile-input
posture is proven by confirming recalled memory is rendered as attributed data that
cannot displace the reader's governing instructions, and that an undecidable trust
judgement always resolves to an explicit stated standing.

Where a check must reference a specific obligation, cite the durable requirement
entity (REQ-NNN), never a mobile membership label. Coverage of the functional and
quality requirements is tracked against those entities, not duplicated here.

## 8. Open Questions

- Should recall surface memories proactively as a context is entered, or only on
  explicit demand? This blocks the contract for any pre-emptive surfacing and the
  trust bar such surfacing would require.
- When memory fans across clients or tenants, how is the authenticity of a recorded
  context established, given that a single-tenant local context is trusted without
  proof? This blocks any shared, multi-writer durability guarantee.
- What is the retention and erasure policy for memories that are no longer wanted but
  whose audit trail must survive? This blocks a defensible position on durable
  retention of sensitive recalled text.
