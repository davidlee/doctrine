# PRD-013: Requirement Reconciliation

## Relations

<!-- Prose stand-in: doctrine has no structural relation surface for product→product
     or spec→ADR edges today, so these links live here, not as stored edges. The gap
     is tracked by IMP-016; the one edge that WILL be structural is the descending
     tech SPEC's `descends_from = "PRD-013"`. -->

- **Governed by** — ADR-003 (canonical change loop; observe → reconcile → close;
  explicit-authorship-not-derivation) and ADR-009 (slice lifecycle FSM with the
  `reconcile` state; the two-enum requirement/coverage truth model, vocabulary-only).
- **Peer of** — PRD-002 (owns the spec/requirement entities this capability writes
  into; this spec resolves its §8 open question), PRD-010 (owns the
  `knowledge_record` kinds — assumption/decision/question/constraint — a
  reconciliation may emit or cite; the reconciliation record itself is *not* one),
  PRD-001 (owns the slice lifecycle whose `reconcile` state this capability
  occupies).
- **Relates to (forward, not defined here)** — a **Drift Ledger (DL)**: spec-driver
  prior art for *mass* reconciliation of conflicting point-in-time claims across
  many overlapping documents, not a per-change artefact. Unimplemented in doctrine;
  reconciliation *surfaces* drift but does not mandate a stored ledger (narrow-PRD
  boundary). The **RV review-ledger** (IMP-001) is a sibling record family. The
  observed-coverage vocabulary (*planned / in-progress / verified / failed /
  blocked*) is inherited from spec-driver's `verification.coverage` block.
- **Realised by** (forthcoming) — a technical SPEC descending from this PRD
  (`descends_from = "PRD-013"`), mirroring PRD-011 → SPEC-001.
- **Constraint** — ADR-004 (relations stored outbound-only) is *why* the above are
  prose: no outbound surface exists for product specs or spec→ADR. See IMP-016.

## Definitions

The ubiquitous language of reconciliation. Terms marked ⌖ are doctrine-wide
glossary candidates — they recur in ADR-003 / ADR-009 and the descending tech spec,
and are defined here only because this spec is where the truth model is first made
whole; lift them to `glossary.md` once a second spec needs them.

- **Authored (normative) status** ⌖ — a requirement's accepted standing *as intent*,
  held on the requirement entity (e.g. accepted, verified, retired). The truth the
  spec asserts.
- **Observed coverage** (coverage evidence) ⌖ — what verification found about a
  requirement: one of *planned / in-progress / verified / failed / blocked*.
  Evidence, not authority; stored independently of authored status.
- **Two-tier truth** ⌖ — the standing separation of authored normative status from
  observed coverage; neither is stored as a function of the other.
- **Derivation** ⌖ — computing authored status as a function of coverage
  (`status = f(coverage)`). The forbidden mechanism.
- **Divergence** — a single observed mismatch between a requirement's authored
  status and its coverage evidence.
- **Drift** ⌖ — the *condition* of a spec carrying one or more unreconciled
  divergences. (Divergence is the observation; drift is the standing condition.)
- **Reconciliation** — the explicit, authored act of bringing requirement and spec
  truth back into coherence with observed reality, choosing accept, revise, or
  redesign.
- **Accept / revise / redesign** — the three reconciliation moves: *accept* the
  evidence as meeting intent; *revise* the authored truth to match reality;
  *redesign* (escalate) when the model itself, not an instance, is inadequate.
- **The explicit writer** — the single actor that authors reconciled truth in the
  loop: the `reconcile`-state actor of the slice FSM (ADR-009 conduct axis, default
  a human gate), sole author of reconciled requirement and spec truth.
- **Reconciliation record (REC)** — the durable, citable artefact of one
  reconciliation: what changed, why, and against what evidence. A distinct
  first-class record, *not* a PRD-010 `knowledge_record`.
- **Instance drift vs model-gap** — instance drift is a single requirement or spec
  out of step with reality, closed by authoring; a model-gap is the spec or
  governance *model itself* being inadequate, escalated to redesign.
- **Forward (planned) intent** — accepted normative intent recorded ahead of
  implementation, with lifecycle/coverage distinguishing *planned* from *verified*.
  Not drift, when grounded and distinguishable.
- **Coherence** — the state of a spec carrying no outstanding unreconciled drift
  against observed reality; the precondition closure gates on.
- **Owning specs** — the specs whose requirements a change bears on; the specs whose
  coherence is gated at closure.
- **Touched requirement** — a requirement a change bears on, for which observed
  coverage is established at audit.

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
**by explicit authorship, never by derivation**.

The distinction is *not* automation versus manual labour, and reconciliation does
not reject automated verification — it depends on it. A codebase holds two distinct
truths: **observed evidence** (what coverage and audit found — cheap, continuous,
largely automatable) and **authored normative intent** (what the spec promises and
accepts). The error reconciliation guards against is letting the first *become the
authority over* the second — spec-driver's `sync`, which derives
`requirement.status = f(coverage)`. Evidence is meant to *support* judgement, not
replace it: green unit tests inform whether the thing works; they are not a
substitute for booting it and checking. Coverage informs reconciliation; it does
not author truth.

The two truths must stay distinct because they demonstrably diverge — and at each
divergence a derivation function picks a wrong answer:

- **Green-but-wrong** — coverage proves the tests that exist pass, never that the
  right thing was tested or the promise met; `f()` derives *verified* for a
  requirement that is not.
- **Forward intent** — accepted intent may legitimately run ahead of
  implementation; no `f(coverage = planned)` yields *accepted-but-unbuilt*, so
  `sync` forces normative status to track code and erases the distinction.
- **Ambiguous coverage** — when several changes touch one requirement at mixed
  states, any `f()` is an arbitrary precedence rule masquerading as objectivity.

At each divergence, judgement chooses one of three moves — **accept** the evidence
as meeting intent, **revise** the authored truth to match observed reality, or
escalate to **redesign** when the model itself is inadequate. Automated
verification feeds all three; none is derivation.

The cost is honest: doctrine gives up automatic, zero-lag coherence and takes on
authoring discipline and the possibility of drift. The payoff is bounded by the
**closure gate** (REQ-103) — the one checkpoint that makes the discipline
non-optional, so authored truth cannot silently rot. In the common case where
evidence and intent already agree, reconciliation is only a confirming pass; the
capability earns its keep in the divergence cases — which is why drift-surfacing
(REQ-100) and forward-intent tolerance are load-bearing, not decoration. This is
doctrine's deliberate divergence from coverage-derived requirement status: the
decision is owned by ADR-003 §5 (amended by ADR-009 §3); this spec states its
product value.

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
- Reconciliation of both PROD and TECH specs. Both tolerate **forward intent**
  (accepted-but-unbuilt) provided *planned* stays distinguishable from *verified*
  and the intent stays grounded; forward intent is *more often* apt for PROD than
  TECH, but is not forbidden to TECH by rule.

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
`knowledge_record` kinds (assumption / decision / question / constraint);
reconciliation may emit or cite a `DEC` for a non-obvious call, but the
reconciliation record itself is not a `knowledge_record`. PRD-001 / ADR-009 own the
slice lifecycle; reconciliation lives in its
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
  *supports* judgement; it is never the authority — reconciliation depends on
  automated verification but is not derived from it.
- **Drift is surfaced, not silently repaired.** New evidence reveals divergence
  and prompts an explicit decision; it never auto-corrects the authored record by
  implication.
- **Forward intent is permitted, not privileged.** Accepted intent may be recorded
  ahead of implementation provided *planned* stays distinguishable from *verified*
  and the intent stays grounded in the codebase's reality. This tolerance is more
  often apt for PROD than TECH, but is not gated by kind; ungrounded or
  indistinguishable forward intent is drift, of either kind.
- **Closure requires coherence.** A change is not closed while its owning specs
  still describe a reality that did not ship. Reconcile precedes close — the closure
  gate is what bounds the cost of explicit authorship, making the discipline
  non-optional at the one checkpoint that matters so authored truth cannot silently
  rot.
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
  edited ahead of implementation to assert future reality as if shipped, except
  permitted forward intent — accepted-but-unbuilt intent that lifecycle/coverage
  state distinguishes from verified and that stays grounded (not gated by spec kind).

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

Each flow names its **trigger**, the **actor**, the **decision** taken, and the
**resulting state**. Behaviour here is product-altitude; CLI verbs and storage are
the tech spec's concern.

**Primary flow — establish observed evidence.** *Trigger:* a change is audited and
its **touched requirements** are identified. *Actor:* audit (agent or self).
*Activity:* for each touched requirement, observed coverage is established as
evidence — *planned / in-progress / verified / failed / blocked* — and each
**divergence** from authored status is identified. *Decision:* audit *identifies*
divergence and assembles evidence; it does **not** author spec truth. *Resulting
state:* assembled evidence plus an identified drift set, ready to reconcile.

**Primary flow — reconcile.** *Trigger:* an identified drift set with its evidence
(the slice enters the `reconcile` state). *Actor:* the **explicit writer** — the
`reconcile`-state actor (ADR-009 conduct axis, default a human gate), sole author of
reconciled truth. *Decision (per divergence):* choose one move — **accept** the
evidence as meeting intent, **revise** the authored requirement/spec truth to match
observed reality, or **redesign** (escalate) when the gap is in the model itself,
not the instance. *Activity:* author the chosen outcome. *Resulting state:*
requirement and spec truth coherent with the reconciled outcome, plus a durable
**reconciliation record (REC)** of what changed, why, and against what evidence.

**Primary flow — gate closure.** *Trigger:* a change is moved toward a terminal
status. *Actor:* close, holding the gate. *Decision:* are all **owning specs**
coherent — no outstanding unreconciled drift? *Resulting state:* coherent → may
reach terminal status; drifted → held short of terminal (gate strength is OQ-3).

**Alternate flow — model-gap escalation.** *Trigger:* reconciliation finds the spec
or governance *model itself* inadequate, not a single instance of drift.
*Decision:* redesign over instance-edit. *Resulting state:* escalated to design
(`reconcile → design`, ADR-009), visibly — never folded silently into a reconciled
outcome.

**Edge case — permitted forward intent.** A requirement (PROD or TECH) may carry
accepted-but-unbuilt intent provided lifecycle/coverage distinguishes *planned* from
*verified* and the intent stays grounded. This is not drift, and reconciliation does
not flag it. Forward intent is more often apt for PROD than TECH, but is not
forbidden to TECH by rule — ungrounded or indistinguishable forward intent *is*
drift, of either kind.

**Edge case — no drift.** When observed evidence already matches authored truth,
reconciliation is a confirming pass that still records coherence; closure proceeds.

**Failure mode — evidence unobtainable.** When coverage cannot be established for a
requirement, it is recorded as *blocked* and surfaced, never silently passed as
verified.

**Failure mode (forbidden) — derivation.** The system must never resolve a
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
tolerance is proven by confirming permitted forward intent — distinguishable and
grounded, of either kind — is not flagged as drift, while ungrounded or
indistinguishable forward intent is.

Where a check must reference a specific obligation, it cites the durable
requirement entity (REQ-NNN), never a mobile membership label. Coverage of the
functional and quality requirements is tracked against those entities, not
duplicated here.

## 8. Open Questions

Resolved (folded into the body above):

- **OQ-1 — reconciliation-record identity.** Resolved: the reconciliation record is
  a **distinct first-class record (REC)** — what changed, why, against what evidence
  — *not* a PRD-010 `knowledge_record`. Its lifecycle is an act record, not a
  belief/decision/question/rule, and PRD-010 §3 routes such artefacts away from the
  family. It *composes* PRD-010 — reusing the evidence-support sub-structure and
  emitting/citing a `DEC` for a non-obvious call. Its schema and id-namespace are
  deferred to the tech spec; PRD-010 is not a build prerequisite.
- **OQ-2 — record cardinality.** Resolved at product altitude: **one slice → one
  REC is the ideal**, but the model tolerates several changes reconciled in one
  record when the paperwork lags reality. The finer *coverage granularity* question
  (per-requirement scalar vs per requirement × contributing change) is deferred to
  the tech spec; spec-driver's per-(artefact × requirement × phase) coverage leans
  toward composable granularity.
- **OQ-4 — PROD vs TECH strictness.** Resolved: forward intent is permitted for
  **both** kinds provided *planned* stays distinguishable from *verified* and stays
  grounded; it is *more often* apt for PROD than TECH but is **not** gated by kind
  (§§2, 3, 6). The real guardrails are speculation disconnected from codebase
  reality and the difficulty of holistic coherence across interconnected specs in
  mixed states.

Open:

- **OQ-3 — closure-gate strength.** Is the gate a hard refusal, or surfaced
  divergence with explicit override (mirroring ADR-009's classify-not-jail posture
  for soft gates, where only the closure-seam topology is structurally enforced)?
  Blocks the gate's enforcement posture and its relationship to the ADR-009 seam.
