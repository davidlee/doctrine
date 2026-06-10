# PRD-013: Requirement Reconciliation

<!-- Reference forms: entity ids padded (REQ-059, ADR-004); doc-local refs bare
     (OQ-1 open question). See .doctrine/glossary.md § reference forms. -->

## 1. Intent

A governed codebase states its normative intent in evergreen specs — what each
requirement promises, and the bar by which a shipped change is judged correct. But
code moves, and what ships diverges from what was written: a requirement is now
verified, another is contradicted by the implementation, a third was never
reached. That divergence is **drift**, and today nothing systematically closes it.
The canonical loop (ADR-003) names the missing step — *observe what shipped, then
reconcile the specs to it* — but the capability is unbuilt: a slice can reach
`done` while its owning specs still describe a reality that never arrived.
"Implemented now, fix the specs later" is drift, not closure, and without
reconciliation it is the silent default.

**Requirement reconciliation** answers this need. It is the capability that brings
evergreen requirement and spec truth back into coherence with shipped reality —
**by explicit authorship, never by derivation**. Observed evidence (what the audit
found, what coverage confirmed) is a *prompt to reconcile*, not a function whose
output overwrites authored truth. Its value is that a closed change leaves behind
specs an agent can trust: requirement standing reflects what actually shipped, the
act that changed it is durable and citable, and no truth was ever rewritten by
precedence, timestamp, or artefact-kind. This is doctrine's deliberate divergence
from coverage-derived requirement status — the differentiator stated as product
intent rather than buried in an ADR.

## 2. Scope

In scope:

- Establishing **observed coverage evidence** for the requirements a change
  touches — the observed half of the truth model (planned / in-progress / verified
  / failed / blocked), distinct from authored normative status.
- **Surfacing drift** between authored requirement and spec truth and the observed
  coverage evidence, as a prompt to reconcile.
- The explicit, authored act of **reconciling** — bringing requirement lifecycle
  status and spec truth to the accepted reconciled outcome through a single
  explicit writer.
- A durable, citable **reconciliation record** of what changed, why, and against
  what evidence.
- **Gating closure** on reconciled coherence: a slice does not reach a terminal
  status while its owning specs remain drifted.
- Reconciliation of both PROD and TECH specs, at the differing strictness their
  forward-intent tolerance allows.

Out of scope:

- The engine's **how** — coverage storage layout, the writer's mechanism, the
  drift-detector algorithm, the reconciliation record's schema, and CLI verb
  shapes. Those belong to the descending technical spec, not this product intent.
- **Contracts** — the deterministic observed-truth corpus. A deferred efficiency
  mechanism (ADR-003 §11); reconciliation must not depend on it existing.
- **Per-slice change design** — a single change's how lives in that slice's design,
  not here.
- **The slice lifecycle vocabulary itself** — owned by PRD-001 / ADR-009.
  Reconciliation *occupies* the `reconcile` state; it does not define the FSM.
- **The spec and requirement entities themselves** — their identity, membership,
  and corpus integrity are owned by PRD-002. Reconciliation *writes into* them; it
  does not define them.

Boundary: PRD-002 owns the spec and requirement *entities*; PRD-013 owns the *act
of bringing them back into coherence with shipped reality*. PRD-010 owns the
record *kinds* (drift, decision) reconciliation may emit; reconciliation *uses*
them. PRD-001 / ADR-009 own the slice lifecycle; reconciliation lives in its
`reconcile` state. This spec resolves the PRD-002 §8 open question — *where is a
requirement's lifecycle standing authoritative* — in favour of **authored on the
requirement, reconciled explicitly, never derived from coverage**.

## 3. Principles

- **Explicit authorship, never derivation.** Requirement and spec truth is
  authored and reconciled by an explicit writer. There is no
  `status = f(coverage)` mapping — no function whose output overwrites authored
  truth by precedence, timestamp, or overlay. This is the deliberate divergence
  from spec-driver's `sync`, and doctrine's differentiator.
- **Observation precedes reconciliation.** Specs are not edited aspirationally
  ahead of implementation to assert future technical reality. The change ships,
  audit observes, reconcile writes — in that order.
- **Two truths, two tiers.** Authored normative requirement status and observed
  coverage evidence are distinct and never collapsed into one another. Evidence
  informs reconciliation; it is not authority.
- **Drift is surfaced, not silently repaired.** New evidence reveals divergence
  and prompts an explicit decision; it never auto-corrects the authored record by
  implication.
- **Closure requires coherence.** A change is not closed while its owning specs
  still describe a reality that did not ship. Reconcile precedes close.
- **Every reconciled change is attributable.** A status or spec change made by
  reconciliation traces to a durable record of what changed and why — truth is
  never rewritten anonymously.

## 4. Requirements

The functional and quality requirements this capability must satisfy are recorded
as requirement entities and appear under the synthesized Requirements section
below. This section carries only the constraints and invariants that bound every
valid implementation.

Constraints:

- Reconciliation must not derive truth by precedence, timestamp, or artefact-kind;
  authored status is reconciled only by explicit authorship.
- Reconciliation must not depend on a contracts corpus existing; until contracts
  land, observed truth rests on audit close-reading and memory git-anchors.
- Observed coverage evidence and authored requirement status occupy distinct
  storage tiers; neither may be stored as a function of the other.
- Reconciliation writes only after audit establishes observed truth; specs are not
  edited ahead of implementation to assert future technical reality, except the
  PROD planned-intent that lifecycle/coverage state distinguishes from verified.

Invariants:

- A requirement's authored lifecycle status changes only through an explicit
  reconciliation act; no derivation function exists in the system.
- Observed coverage is evidence and a prompt to reconcile; it never overwrites
  authored truth by implication.
- A closed slice's owning specs are coherent with the reconciled outcome —
  closure and outstanding drift on those specs are mutually exclusive.
- Every reconciled status or spec change is attributable to a durable
  reconciliation record.

## 5. Success Measures

- After a slice closes, an agent reading its owning specs sees requirement
  standing that reflects what actually shipped — without recourse to the slice's
  audit notes or chat history.
- Drift between authored requirement truth and observed coverage is surfaced
  before closure, not discovered later by a confused reader.
- Every change to a requirement's authored status is traceable to an explicit
  reconciliation act; none arose by derivation, precedence, or timestamp.
- A reviewer can confirm that a closed slice's owning specs carry no outstanding,
  unreconciled drift.
- A reconciliation that finds the model itself inadequate (not mere instance
  drift) is visibly escalated rather than forced into a spec edit.

## 6. Behaviour

Primary flow — establish observed truth: as a change is audited, the observed
coverage of each requirement it touches is established as evidence (verified,
failed, blocked, in-progress, or planned), and divergence from the authored status
is identified. Audit *identifies* drift and assembles the evidence; it does not
write spec truth.

Primary flow — reconcile: the explicit writer takes the identified drift and its
evidence and authors the reconciled outcome — updating requirement lifecycle
status and the touched spec truth to the accepted result — and records a durable
account of what changed, why, and against what evidence. This writer is the sole
author of reconciled requirement and spec truth in the loop.

Primary flow — gate closure: closing a change confirms that its owning specs are
reconciled and carry no outstanding drift. A change with unreconciled owning specs
is not coherent and does not reach a terminal status.

Alternate flow — model-gap escalation: when reconciliation finds the spec or
governance *model itself* inadequate — not a single instance of drift — the
divergence is escalated for redesign rather than absorbed by editing the instance.

Edge case — permitted forward intent: a PROD requirement may carry planned or
newly-accepted intent ahead of full verification, provided lifecycle and coverage
state distinguish *planned* from *verified*. This is not drift, and reconciliation
does not flag it. TECH requirements carry no such tolerance and are reconciled
from observed implementation.

Edge case — no drift: when observed evidence already matches authored truth,
reconciliation is a confirming pass that still records coherence; closure
proceeds.

Failure mode — evidence unobtainable: when coverage cannot be established for a
requirement, it is recorded as blocked and surfaced, never silently passed as
verified.

Failure mode (forbidden) — derivation: the system must never resolve a
requirement's authored status as a function of its coverage. Any path that would
rewrite authored truth by precedence, timestamp, or overlay is a defect, not a
shortcut.

## 7. Verification

Verification confirms that reconciliation brings spec truth back into coherence
with shipped reality by explicit authorship, that observed evidence never
overwrites authored truth by implication, that closure is gated on coherence, and
that no derivation path exists — without binding the spec to a particular
implementation.

The authorship invariant is proven by confirming that every change to a
requirement's authored status passes through the explicit reconciliation act and
is attributable to a reconciliation record, and that no function maps coverage to
authored status anywhere in the system. The two-tier separation is proven by
confirming observed coverage and authored status are stored independently and that
neither is derived from the other. Drift surfacing is proven by seeding divergence
between authored truth and observed evidence and confirming it is reported as a
prompt to reconcile rather than auto-corrected. The closure gate is proven by
confirming a change with outstanding unreconciled drift on its owning specs cannot
reach a terminal status, while a reconciled change can. The forward-intent
tolerance is proven by confirming permitted PROD planned-intent is not flagged as
drift while equivalent TECH divergence is.

Where a check must reference a specific obligation, it cites the durable
requirement entity (REQ-NNN), never a mobile membership label. Coverage of the
functional and quality requirements is tracked against those entities, not
duplicated here.

## 8. Open Questions

- OQ-1 — The reconciliation record's identity: is it a new first-class entity, a
  PRD-010 `knowledge_record` (drift / decision kind), or an artefact in the slice
  bundle? Blocks the descending engine tech spec's artefact model and determines
  whether PRD-010 is a build-time prerequisite of reconciliation.
- OQ-2 — Coverage granularity: is observed coverage tracked per requirement only,
  or per (requirement × contributing change) so several slices touching one
  requirement compose without clobbering each other? Blocks the coverage
  substrate's shape and relates to PRD-002 §8's capability-grouping question.
- OQ-3 — Closure-gate strength: is the gate a hard refusal, or surfaced divergence
  with explicit override (mirroring ADR-009's classify-not-jail posture for soft
  gates, where only the closure-seam topology is structurally enforced)? Blocks
  the gate's enforcement posture and its relationship to the ADR-009 seam.
- OQ-4 — PROD vs TECH strictness: how is the §9 difference — PROD tolerates
  forward intent, TECH is reconciled from observed implementation — expressed as
  product behaviour rather than discipline? Blocks the per-kind reconciliation
  rules.
