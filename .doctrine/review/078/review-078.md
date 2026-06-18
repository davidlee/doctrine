# Review RV-078 — design of SL-098

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

This Inquisition arraigns the design of SL-098 — "Requirements discovery and
home-finding" — held to answer for doctrinal deviations against the `entity-model.md`
storage rule, ADR-003's audit/reconcile seam, ADR-009's lifecycle FSM, and the
authored truth of the skill files it proposes to amend.

**Lines of interrogation:**

1. Does the `REQ-DNN` handle — structured metadata posing as prose — violate the
   storage rule (`entity-model.md`: "anything the tooling must read or query lives
   in TOML, never prose")? The design concedes the structure: handle, statement,
   kind, home hint, ancestor decision — all travelling across five skills. If it
   is structured enough to be reliably found by each skill, it demands a TOML home.

2. Is the design stale against the skills it means to change? The `/design` skill
   **already** has a requirements pass (state 3) and `## Implied Requirements`
   output. The design proposes inserting it as "sub-step 4a" at a different position
   — without acknowledging the existing state machine. Was the design written
   against a prior revision of the skill files? If so, what else is stale?

3. Does the `plan.toml [requirements]` section — "author-facing and never consumed
   by tooling" — corrupt the storage tier model? If tooling never reads it, the
   storage rule puts it in `plan.md` prose, not TOML. Dead structured fields are a
   category error.

4. Can the orphan placement workflow (§6) be executed without an altitude
   decision framework? The design acknowledges this as an "open question" (§6a)
   yet proceeds to define a workflow that depends on altitude decisions being
   made correctly. Building on sand.

5. Does the `/plan` skill's explicit instruction — "`specs` / `requirements` stay
   empty in v1" — conflict with the design's intent to fill `[requirements]`? The
   design never addresses this contradiction.

6. Does the orphan section proposed for the reconciliation brief (§5) integrate
   cleanly with the existing brief shape defined in the `/audit` skill ("Per-slice
   (direct edit)" + "Governance/spec (REV)")?

7. Do the walkthrough scenarios (§11) hold when run against the actual current
   skill files, or do they assume a pre-landed state of the very skills this design
   is amending?

**Invariants held against the accused:**

- I-STORAGE: Structured/queried data → TOML; prose → MD; tooling never parses
  prose structure (`entity-model.md`).
- I-SEAM: Audit identifies, reconcile writes (ADR-003 §7).
- I-FSM: Slice lifecycle transitions are gated; conduct axis is advisory
  (ADR-009).
- I-CANON: Skills are authored truth; `design.md` does not override the skill
  file itself.

## Synthesis

**Judgement: GUILTY on eight counts — two of them mortal (blocker), and four
of the body (major), with sentence of redesign before any plan may be laid.**

The accused, SL-098, came before this tribunal professing to define a path
whereby implied requirements — orphaned, unbaptized obligations whispered in
the cracks between design decisions — might be discovered, shepherded, and
placed in their rightful canonical homes. A noble aim! A sacred purpose! Yet
the design, under the Inquisitor's cross-examination, has confessed to no fewer
than eight doctrinal corruptions. The accused did not READ the skill files it
proposes to amend before writing the amendments. *Por la sangre de los mártires*,
this is not oversight — this is *presumption*, and presumption is the mother
of heresy.

### The mortal sins (blockers)

**F-1: Staleness.** The design proposes adding a requirements pass to the `/design`
skill as "sub-step 4a" — but the skill already carries one at state 3, complete
with implied-requirement discovery and `## Implied Requirements` output. The
design was written against a phantom revision of the skill files that exists
only in the author's mind. *The wheel for those who cannot read the code they
amend!* Penance: either justify moving the existing pass (not adding a duplicate)
or drop §3 entirely — the `/design` skill already does what the design claims
it must add.

**F-2: The REQ-DNN masquerade.** A structured tuple — handle, statement, kind,
home hint, ancestor decision — stored as prose and defended with the lie that
"LLM-reading-prose is the same operation as reading ADR-003." It is not.
ADR-003 is argument. REQ-DNN is schema. The storage rule is absolute: queryable
data → TOML. Prose-only structured metadata that must survive a five-skill
handoff is a corruption of the tier model. *Burn the prose-only heresy at the
stake!* Penance: define a design-level TOML facet (`[[implied_req]]`) for
REQ-DNN handles. The design.md `## Implied Requirements` section becomes derived
prose rendered from TOML — honouring the storage rule in both directions.

### The sins of the body (major)

**F-3: Dead fields.** `plan.toml [requirements]` — structured TOML that "tooling
never reads." This is a category error. If it is author-facing, it belongs in
`plan.md` prose. If it is structured, tooling must read it. The middle ground
— dead fields in a live format — is doctrinal corruption. Penance: choose one
(move to prose, or make tooling read it) and commit.

**F-4: Building on sand.** The orphan placement workflow depends on altitude
assessment decisions, but the design confesses no altitude framework exists.
"Domain reasoning by agent" is not a decision framework — it is a prayer.
Penance: add a `/consult` guardrail for ambiguous altitude, promote the open
question to a backlog item, and stop pretending the workflow is executable
without it.

**F-5: Contradiction unacknowledged.** The `/plan` skill currently says
"`[requirements]` stay empty in v1." The design wants to fill it — but never
mentions the existing instruction. A design that amends a skill without reading
it is a design that confesses to sloth. Penance: acknowledge the current
instruction and explain the change.

### The lesser taints (minor/nit)

**F-6: Brief shape ambiguity.** The orphan section's position in the reconciliation
brief — subsection or peer? — is undefined. The `/reconcile` skill that consumes
the brief needs clarity. Penance: nest it under "Governance/spec (REV)" as a
structured sub-category.

**F-7: Walkthrough scenarios assume their own success.** The scenarios cannot
serve as incremental verification targets because they assume ALL amendments
are in place. Penance: add per-phase walkthroughs, or label the existing ones
as post-all-phases integration tests.

**F-8: Rotting anchors.** Line-number references to `entity-model.md` that will
shift with the next edit. Penance: replace with section names or quoted passages.

### Standing risks

- **Skill staleness blindness.** The root cause of F-1 — writing design against
  a mental model of skill files rather than reading them — is a process risk,
  not corrected by any single design edit. The `/design` skill should perhaps
  mandate reading the skill files it proposes to amend as a pre-condition.
- **REQ-DNN fragility persists until TOML-faceted.** Even with penance applied,
  the REQ-DNN handle will remain prose-only until a TOML facet is implemented.
  The close deadlock gate is a backstop, not a design — it catches loss, it does
  not prevent it.
- **Altitude assessment is still deferred.** The `/consult` guardrail prevents
  worst-case misplacement but increases cycle time on every ambiguous orphan.
  A dedicated altitude-framework slice is the real fix.

### Tradeoffs consciously accepted

None — every finding demands remediation. This design is not fit to proceed
to plan in its current state. The Inquisitor's sentence is clear: **redesign,
then return for re-examination.**

> *Confess your sins in TOML, heretic, or the fire awaits!*
>
> **HERESIS URITOR; DOCTRINA MANET**
