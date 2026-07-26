# PRD-018: Observations

<!-- Reference forms: entity ids padded (REQ-059, ADR-004); doc-local refs bare
     (OQ-1 open question). See .doctrine/glossary.md § reference forms. -->

## 1. Intent

Agents and operators encounter small, repeated signals while doing work: a command
that causes avoidable retries, a workflow that obscures its next step, a harness
limitation, or a machine measurement that may later explain wasted effort. These
signals are useful in aggregate, but at capture time they are usually too raw to be
knowledge, too unanalysed to be evidence for a conclusion, and not yet a unit of
actionable work.

Without a dedicated home, raw occurrences either vanish, accumulate in shared prose
logs that cannot be searched or merged cleanly, or are prematurely promoted into
memory, knowledge records, or backlog items. All three outcomes destroy useful
frequency and context.

**Observations** provide the durable collection boundary: record each occurrence
cheaply and independently, retain whatever trustworthy context is available, and
make the corpus inspectable without interpreting it. The desired end state is that
recurring friction and execution effects can later be analysed from faithful raw
signals rather than reconstructed from anecdotes.

## 2. Scope

In scope:

- Recording a raw occurrence as a durable, independently addressable observation.
- Carrying minimal kind-specific content plus optional typed execution, work,
  correlation, provenance, and machine-measurement context.
- Inspecting observations directly and surveying them through structured filters
  and lexical search.
- Distinguishing a current resolved view from the complete correction history.
- Correcting an observation without erasing or rewriting the original occurrence.
- Allowing supported constrained agents to capture observations through a bounded
  capability.

Out of scope:

- Deduplication, clustering, frequency analysis, prioritisation, trend reporting,
  backlog-coverage reporting, or impact measurement.
- Automatically promoting observations into memories, knowledge records, backlog
  items, or other authored entities.
- Consumer processing state such as analysed, triaged, consumed, or aggregated.
- Fabricated or agent-estimated usage telemetry, efficiency scores, cost models, or
  cross-model benchmark claims.
- Automated retention, compaction, archival, hosted telemetry, or a user interface.

Boundary: an observation records that something occurred. It does not state what the
occurrence means, whether it is important, whether it is duplicate evidence, or what
work should follow. Those judgements belong to downstream consumers and authored
entities.

## 3. Principles

- **Capture precedes interpretation.** Recording never requires a duplicate search,
  taxonomy decision, root-cause claim, or backlog disposition.
- **Repeated occurrences remain repeated.** Similar content does not imply identity;
  consolidation is a downstream analytical act.
- **Unknown stays unknown.** Missing context is omitted, not inferred, defaulted, or
  rendered as measured zero.
- **Raw signals are immutable.** Correction adds explicit history; it never silently
  rewrites the occurrence that was captured.
- **Collection does not own consumption.** Every analyser, triager, or aggregator
  owns its own processing state outside the observation.
- **Measurement claims carry their boundary.** Machine measurements are retained
  only with enough source and scope to avoid implying comparability they do not
  possess.

## 4. Requirements

The functional and quality requirements this capability must satisfy are recorded
as requirement entities and appear under the synthesized Requirements section
below. This section carries only the constraints and invariants that bound every
valid implementation.

Constraints:

- An ordinary friction observation must require no classification beyond its kind
  and concise symptom.
- Observation capture must not require a central allocator, shared append target,
  pre-capture search, or automatic Git operation.
- Optional usage data must originate from a machine-readable source and declare its
  measurement boundary; absence remains valid.
- A constrained capture capability must not become an arbitrary filesystem or
  correction capability.

Invariants:

- Each occurrence has one stable identity and remains independently recoverable.
- Different identities are never collapsed because their contents are similar.
- Correction history remains inspectable after supersession or retraction.
- The default current view never silently applies an invalid correction.
- Consumer processing state never mutates the observation.

## 5. Success Measures

- An agent can record a friction occurrence with a concise summary in one bounded
  interaction and without first surveying the corpus.
- Concurrent captures create independent records rather than contending on shared
  prose.
- A later analyst can find repeated terms and filter by reliably captured execution
  context without re-reading one unbounded document.
- A retry can safely identify the same intended occurrence without producing an
  overwrite or accidental duplicate.
- A malformed record or correction is visible as a diagnostic without making the
  remaining corpus unavailable.
- Supported confined agents can record observations without receiving general write
  authority.

## 6. Behaviour

Primary flow — record an occurrence: a caller supplies an observation kind and the
kind's minimal payload. The system assigns or accepts stable identity, captures
allowlisted context it can know reliably, stores one immutable observation, and
returns an addressable receipt.

Alternate flow — explicit context: a caller supplies typed context or measurement
fields. Explicit facts take precedence over automatic enrichment. Missing automatic
context does not block capture; invalid explicit facts do.

Primary flow — inspect the corpus: a caller addresses one observation directly or
lists and searches the collection by time, kind, text, and typed context. Collection
views default to current resolved observations; history remains explicitly
available.

Correction flow: a trusted operator supersedes an observation with an existing
compatible replacement or retracts an exact observation. The system records the
correction separately and retains every original record.

Guard — invalid correction: a dangling, cyclic, conflicting, or otherwise invalid
correction is surfaced and has no effect on the current view. Valid observations
remain available.

Guard — constrained capture: a confined agent may create approved primary signal
kinds in the registered collection root but cannot choose arbitrary paths or create
correction controls.

Edge cases: two observations with identical text remain distinct occurrences;
late-arriving usage is a separately correlated observation; unknown optional context
remains absent; a retry with the same identity and caller intent returns the
existing observation, while different intent under that identity is refused.

## 7. Verification

Verification proves that raw occurrences can be captured independently, recovered
by stable identity, searched without interpretation, and corrected without loss.
Concurrency and retry scenarios establish the identity and merge-safety obligations
in REQ-402. Capture-path exercises establish that the minimal flow and optional
context behaviours in REQ-397 and REQ-400 do not turn bookkeeping into a
precondition.

Inspection exercises prove the current/history distinction and deterministic,
failure-tolerant visibility required by REQ-398 and REQ-404. Correction exercises
prove REQ-399 while retaining original records. Confinement checks prove REQ-401
without granting arbitrary write or correction authority. Hostile-input and failed
enrichment scenarios establish the quality bar in REQ-403 and REQ-404.

Coverage is tracked against the durable requirement entities, never duplicated by
their mobile membership labels.

## 8. Open Questions

- OQ-1 — Which harnesses expose trustworthy usage measurements, at what boundary,
  and with what completeness? QUE-176 tracks the investigation; absence does not
  block ordinary observations.
- OQ-2 — What corpus growth warrants automated retention or archival rather than
  manual partition-level operations? This remains deliberately unanswered until
  real collection growth supplies evidence.
- OQ-3 — Which reporting and aggregation views best turn observations into
  prioritisation evidence without erasing frequency or exposure denominators? This
  belongs to a follow-up capability, not collection.
