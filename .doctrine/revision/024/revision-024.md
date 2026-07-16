# REV REV-024 — ADR-015 tiered value-claim resolution

Revision (ADR-013) — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

RFC-020 Phase 1 (SL-220, design D1–D14) moves absolute value assignment into
the comparison ledger as first-class evidence — dated, attributed,
supersedable anchor-claim rows — and dissolves REV-022's anchors-win posture
into the T3 epistemic authority ladder. The motivating defect (IMP-290): ~90%
of authored `[value]` facets are unattributed agent guesses that today
constitutionally outrank the comparison evidence built to calibrate them.

This REV rewrites ADR-015's value-source resolution policy and amends the
canon that describes the `[value]` surface (SPEC-020's value-facet
requirements, PRD-014's value framing) so canon is never stale about what
`value set` does while the corpus is live (SL-220 design D12). **Approval
gates SL-220 PHASE-05 (the resolver flip); application lands no later than
SL-220 PHASE-07 (the migration census).** Earlier SL-220 phases are strictly
additive and REV-independent.

Estimate-source resolution (REV-023) is deliberately untouched: the
inter-domain asymmetry during the interregnum is named and accepted; RFC-020
Phase 2 brings the estimate successor REV.

Scope of the staged delta: seven `modify` rows — ADR-015 (primary, prose
edits below), SPEC-020 + REQ-278/279/280/286 (normative value-surface
amendments), PRD-014 (claims-model pointer). All surfaced-for-manual
hand-edits after approval; `doctrine boot` after landing.

### Change row — modify ADR-015 (primary)

**Edit 1.1 — §1 Base dimensions › *Value-source resolution* section
(REV-022's text) rewritten to the T3 ladder.**

Before:

> ### Value-source resolution (REV-022, RFC-019)
>
> The `value` input to `value_dim` resolves by provenance:
>
> 1. **Authored** `[value]` magnitude — wins outright, and additionally acts as a
>    point anchor in the comparison constraint layer, propagating bounds to items
>    compared against it.
> 2. **Projected** — items with comparison evidence but no authored value take the
>    deterministic point projection within their derived bounds (RFC-019
>    constraint layer).
> 3. **Default** — items with neither take `DEFAULT_VALUE = 1.0`.
>
> An authored value that conflicts with comparison-derived bounds is a surfaced
> contradiction (diagnosed like a needs cycle), not a silent win. Projected values
> and bounds are derived at read time from the comparison ledger; they are never
> authored to disk (the same posture as scores). Setting `[value]` on a
> non-value-bearing kind (knowledge records, governance) warns: the write is
> scoring-inert — a record's worth is derived from what it unlocks, not authored
> (REV-022 Q1).

After:

> ### Value-source resolution (REV-024, RFC-020)
>
> The `value` input to `value_dim` resolves per item by the epistemic
> authority ladder, first hit wins:
>
> 1. **Anchored claim** — the resolved pin- or human-tier anchor claim from
>    the comparison ledger (`value pin` / `value set --rater human`). Wins
>    outright and additionally acts as a point anchor in the comparison
>    constraint layer, propagating bounds to items compared against it.
>    Pin outranks human claim within the tier resolution.
> 2. **Comparison projection** — items with comparison evidence but no
>    anchored claim take the deterministic point projection within their
>    derived bounds (RFC-019 constraint layer). Anchors feeding the
>    constraint layer come from pin/human tiers only — agent-tier magnitudes
>    never shape projection (anti-laundering, RFC-020).
> 3. **Agent-tier claim** — an attributed agent claim resolves as a prior
>    below projection: a number, not an answer.
> 4. **Migrated-tier claim** — an unattributed magnitude imported from a
>    retired `[value]` facet (rater = migrated, observed-at only); ranks
>    below attributed agent claims.
> 5. **Unmigrated `[value]` facet** *(transitional)* — consulted only when
>    zero claim rows exist for the item; presence fires an `UnmigratedFacet`
>    finding. This rung deletes when both domains have migrated (RFC-020
>    Phase 2 exit criterion).
> 6. **Default** — `DEFAULT_VALUE = 1.0`.
>
> Same-tier conflict never resolves silently: distinct active magnitudes in
> the winning tier resolve to their arithmetic mean over the full row
> multiset, render the disagreement interval as bounds, and fire a
> `ClaimConflict` finding (a conflicted pin is a *contested pin*); resolution
> is a human appending an explicitly superseding row. A pin/human anchor
> conflicting with comparison-derived bounds remains a surfaced
> contradiction, not a silent win. Resolved values, bounds, and projections
> are derived at read time from the ledger; they are never authored to disk
> (the same posture as scores). Claiming value on a non-value-bearing kind
> warns: the capture is lossless but scoring-inert — a record's worth is
> derived from what it unlocks (REV-022 Q1, unchanged posture).

**Edit 1.2 — §4 Config contract › durable-domains list, value bullet.**

Before:

> - value-source resolution order (authored → projected → default) is fixed
>   policy, not a knob; projection is deterministic given (ledger, config,
>   anchors); contradiction surfacing cannot be suppressed by configuration;

After:

> - value-source resolution order (pin > human claim > projection > agent
>   claim > migrated claim > unmigrated facet (transitional) > default) is
>   fixed policy, not a knob; claim resolution and projection are
>   deterministic given (ledger, config); conflict and contradiction
>   surfacing cannot be suppressed by configuration;
>   `[priority.compare] demote_agent_evidence` widens to claims: when set,
>   agent- and migrated-tier resolved values never retire elicitation —
>   the item stays probe-eligible;

**Edit 1.3 — Consequences › per-item override bullet.**

Before:

> - No per-item priority override is introduced beyond the authored value anchor
>   (authored-wins, REV-022). Cost/maintainability dimensions remain future work
>   (IDE-035 holds the trigger); the comparison ledger (RFC-019) is a
>   value-evidence history, but score history as such remains future work.

After:

> - The per-item priority override is the **pin** (REV-024): a governed,
>   attributed, supersedable ledger row admitted only through the
>   operator-gated `value pin` verb (interactive-TTY + worker-refused write
>   class) — never a side effect of typing a number. Cost/maintainability
>   dimensions remain future work (IDE-035 holds the trigger); the comparison
>   ledger (RFC-019/020) is now the value-evidence history for absolute and
>   relative judgements alike; score history as such remains future work.

### Change rows — SPEC-020 value-surface requirements + prose

Normative amendments (exact text finalised at apply; the requirement acceptance
criteria move from "authored facet" to "ledgered claim" semantics):

- **REQ-278** (ValueFacet model/parse seam): the `[value]` facet parse seam is
  retired as the value surface; the wire-level anchor-claim row (`form =
  anchor`, value payload `{magnitude}`: finite f64) becomes the normative
  capture shape. The facet parse survives transitionally to serve ladder
  rung 5 and the migration census only.
- **REQ-279** (present-facet validation): validation moves to the capture
  matrix on the claim row (magnitude finite, mirrors `value::validate`;
  provenance rules: mandatory rater, pin ⇒ human ∧ anchor, migrated ⇔
  observed_at). Absent claims remain valid — bookkeeping never blocks work.
- **REQ-280** (project-wide value unit): unchanged semantics, re-stated over
  claim magnitudes rather than facet values.
- **REQ-286** (policy-free hydration exposure): the hydration seam stops
  exposing authored `[value]` magnitudes as the value surface; value reaches
  consumers as resolved `(value, provenance)` from the comparison pipeline.
  The policy-free contract posture is unchanged.
- **SPEC-020 prose**: the Value facet sections (`value set` writer semantics,
  facet-as-value-surface, hydration reader) rewritten to the claims model;
  `value set` appends an anchor-claim row, `value clear` appends tombstones,
  correction is supersession, `value pin` is the operator-gated admission
  path.

### Change row — PRD-014

The value half of PRD-014 points at the claims model: "an optional,
project-local single-magnitude claim" becomes a *ledgered* claim — dated,
attributed, supersedable — rather than an entity-file facet; validation and
absence semantics unchanged in force; aggregation/rollup exclusions unchanged.

### Deliberately untouched

- **Estimate-source resolution** (REV-023's ladder) — stands until RFC-020
  Phase 2's own REV; asymmetry during the interregnum is deliberate.
- **ADR-018 / burndown semantics** — containment stays capture-only
  (RFC-020 RV-275 F-2); aggregation modes gate on OQ-1.
- Additive/documentary spec work (claim-schema retention REQs,
  PRD-011/SPEC-001 descent prose) — reconciliation obligations of SL-220,
  not staged here (design D12).
