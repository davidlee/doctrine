# REV REV-026 — ADR-015 tiered estimate-claim resolution

Revision (ADR-013) — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

RFC-020 Phase 2 (SL-222, design §7/E11) moves absolute estimate assignment
into the comparison ledger as first-class evidence — dated, attributed,
supersedable estimate-anchor rows (`frame = cost-anchor`, payload
`est_lower`/`est_upper`) — and dissolves REV-023's estimate-source ladder
into the T3 epistemic authority ladder. The motivating defect is IMP-290's
census applied to the estimate half: ~90% of authored `[estimate]` facets
are unattributed agent guesses, and the estimate is the *more* dangerous
half of the provenance gap — `est_cost` is the divisor of `value_dim`, so a
guessed range scales the whole ranking.

This REV rewrites ADR-015's estimate-source resolution policy and amends
SPEC-020's estimate-facet requirements so canon is never stale about what
`estimate set` does while the corpus is live (D12 mirror, SL-220
precedent). **Approval gates SL-222 PHASE-06 (the `est_cost` flip);
application lands with the flip.** REV-024 explicitly left REV-023
standing; this REV is its designed successor (SL-222 design E11).

Scope of the staged delta: twelve `modify` rows — ADR-015 (primary, prose
edits below), SPEC-020 + REQ-269/270/272/273/274/275/276/277/310 (normative
estimate-surface amendments per the disposition map), PRD-014 (claims-model
pointer widens to the estimate domain). All surfaced-for-manual hand-edits
after approval; `doctrine boot` after landing. REQ-271 (unit resolution) is
deliberately absent: unchanged — unit renders resolved claims.

### Change row — modify ADR-015 (primary)

**Edit 1.1 — §1 Base dimensions › *Estimate-source resolution* section
(REV-023's text) rewritten to the claim ladder.**

Before (excerpt, the section header and ladder):

> ### Estimate-source resolution (REV-023, SL-219)
>
> The `est_cost` input to `value_dim` resolves by provenance — **source
> precedence only, no numeric-dominance claim**:
>
> 1. **Authored** `[estimate]` bounds — an **operator pin**: a policy override
>    over accumulated sizing evidence. Wins outright, and additionally acts
>    (β-resolved) as a point anchor in the estimate constraint system,
>    propagating cost bounds to items compared against it.
> 2. **Projected** — items without authored bounds but with sizing evidence
>    in an anchored component take the deterministic point projection within
>    their derived bounds (RFC-019 constraint layer, estimate domain).
> 3. **Bare anchor** — items with neither take `max_upper(corpus) + margin`
>    (`1.0` empty-corpus fallback).

After (the successor section; final wording lands at apply):

> ### Estimate-source resolution (REV-026, SL-222)
>
> The `est_cost` input to `value_dim` resolves per item by the T3 epistemic
> authority ladder over estimate claims — **source precedence only, no
> numeric-dominance claim**; first hit wins:
>
> 1. **Anchored claim** — a resolved pin or human-tier estimate claim
>    (conflict means included): the claim's operative cost
>    (`estimate::operative_cost(bounds, skew)`, EPSILON-floored). Anchored
>    claims also anchor the estimate constraint system, propagating cost
>    bounds to items compared against them — the two seams the authored
>    facet previously occupied, now attributed and dated.
> 2. **Cost projection** — items with sizing evidence in an anchored
>    component take the deterministic point projection within their derived
>    bounds (RFC-019 constraint layer; non-Gauge provenances only; EPSILON
>    floor at the consumption seam).
> 3. **Agent-tier claim** — a resolved agent-authored claim's operative
>    cost: a prior below projection, above migration.
> 4. **Migrated-tier claim** — a resolved `rater = migrated` claim's
>    operative cost (unattributed imports sit at the bottom of asserted
>    evidence; migration never mints a pin).
> 5. **Bare anchor** — items with no ladder hit take the dominating
>    divisor (INV-2 below; `1.0` empty-corpus fallback).
>
> Same-tier conflict resolves to the per-field mean over the winning-tier
> multiset with the operative-cost conflict interval surfaced as a finding —
> corroboration and conflict are distinguished, never silent. The authored
> `[estimate]` facet is retired as an input: `estimate set` appends ledgered
> claims; `estimate pin` is the operator-policy override, now attributed and
> governed (interactive-TTY, worker-refused write class). The "records
> anchor too" sentence narrows to anchored-tier claims — a records-anchor
> delta disclosed as class-b behaviour change.
>
> **INV-2 restated for the claims era**: the bare anchor is computed over
> **every active unlensed estimate-anchor row's upper** — any tier, losing
> tiers and individual conflict rows included, plus transitional unmigrated
> facet uppers — and therefore dominates every active asserted range by
> construction; projected costs may exceed or undercut it; a bare item with
> no evidence keeps the dominating divisor (the ISS-057 anti-inversion
> intent preserved exactly).
>
> **Gauge never divides** (unchanged, claim vocabulary): gauge-tier
> placements are never fed to `est_cost`; a gauge-masked item carrying an
> asserted claim resolves at its claim tier — an asserted range beats a
> conventional gauge — else it scores at the bare anchor.
>
> **Positivity axiom** (unchanged, claim vocabulary): payload validation
> enforces `lower ≥ 0` at capture (`estimate::validate`, the single
> source); operative costs floor at `EPSILON`; every fed projection is
> strictly positive.
>
> **Fixed policy, not a knob**: the estimate ladder is policy.
> `demote_agent_evidence` widens to estimate claims — when set,
> agent/migrated-tier resolved costs leave sizing probe-eligibility intact;
> when unset they retire it. Probe targets require anchored-tier claims.
>
> **Transitional rungs deleted** (both domains): REV-024's Phase-2 clause is
> recorded as fired by this REV. During the migration window an unmigrated
> `[estimate]` facet resolves below migrated claims and fires a loud
> presence finding; at the deletion phase the facet read paths delete and
> the finding becomes magnitude-free with a remedy naming
> `scripts/migrate_estimate_facets.py` and `estimate set --rater human`. A
> never-migrated corpus re-ranks deterministically (facet-valued items fall
> to bare anchor) with loud findings — the scripts are committed and
> corpus-agnostic, so the remedy is real.

### Change rows — SPEC-020 + REQs: the per-REQ disposition map (RV-282 F-6)

Authored into the flip-gate spec amendment; final wording lands at apply.

| REQ | today | disposition | gate |
|---|---|---|---|
| REQ-269 (`EstimateFacet` model + finite-f64 normalisation) | facet struct | **rewrite**: the normalisation/validation contract re-targets the anchor-claim payload (`est_lower`/`est_upper`); `estimate::validate` survives as the single source | flip |
| REQ-270 (present-table hard validation) | parse-time | **rewrite**: capture-time validation matrix; absence of claims stays valid | flip |
| REQ-271 (project-wide unit resolution) | config | **unchanged** — unit renders resolved claims (no change row) | — |
| REQ-272 (parse→hydrate→catalog preservation) | facet plumbing | **retire at deletion**: the preserved thing becomes the ledger row; wire round-trip goldens are the successor obligation | deletion |
| REQ-273 (display without spread classification) | facet display | **rewrite**: resolved-claim range display, same no-classification posture | flip |
| REQ-274 (policy-free graph exposure) | raw per-node bounds | **rewrite**: the graph consumes resolved claims through the pipeline; per-node raw-facet exposure retires at deletion | flip + deletion |
| REQ-275 (presence gates no workflow predicate) | facet presence | **rewrite**: claim presence gates no workflow predicate either (probe-eligibility is knob-governed elicitation policy, not a workflow gate) | flip |
| REQ-276 (kind-agnostic + pure-layer) | facet | **rewrite**: claim capture stays kind-agnostic; resolution stays pure-layer | flip |
| REQ-277 (schema forward-compat) | `_extra` passthrough | **rewrite**: the wire's lossless-over-absent-optionals posture (E2) is the successor mechanism | flip |
| REQ-310 (confidence-band percentile framing for display) | config band over facet bounds | **rewrite**: the band frames resolved-claim bounds identically; still display-only, no entity-local field | flip |

Prose amendments (facet as authored surface, `set`/`clear` writer semantics,
hydration reader) ride the same gate. PRD-014's claims-model pointer widens
to name the estimate domain. PRD-011/SPEC-001 descent prose remain
reconciliation obligations, non-contradicting. RFC-020's Phase 2 row moves
to delivered-by-SL-222 at reconciliation (Phase 3 keeps it open).
