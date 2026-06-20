# REV REV-005 — reconcile SL-104

Revision (ADR-013) — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

**Origin.** SL-101 introduced `EstimationConfig.{lower,upper}_confidence`,
`resolve_confidence`, and `DEFAULT_*_CONFIDENCE` into `src/estimate.rs` as dead
code with **no governing requirement**. SL-104's audit (RV-114, finding **F-3**)
confirmed the residue is real and undischarged: the `expect(dead_code)` reason
strings cite a confidence requirement "landing at SL-104 reconcile" — a descriptive
placeholder for a REQ that did not yet exist. This REV mints that requirement and
amends SPEC-020 to home it, per design D2 (governance routes through a Revision
folded into reconcile, ADR-013) — mirroring **REV-002**, which amended this same
spec at SL-101's reconcile.

**Classification.** Functional (FR-011) — user decision (2026-06-20), resolving
design OQ-1. The requirement defines *behaviour* (band resolution from config +
display framing), not merely a quality attribute.

## Reconcile narrative (SL-104)

- **[RV-114 finding F-3] — introduce FR-011 (confidence band).** The percentile
  model is now spec-legitimate: an estimate's `lower`/`upper` are read as a
  project-wide confidence band (`lower` at P-low, `upper` at P-high), resolved from
  `doctrine.toml [estimation].lower_confidence`/`upper_confidence`, defaulting
  `0.1`/`0.9`; each bound finite, in `[0,1]`, `low < high`. **Display-framing only —
  no gating, aggregation, or normalization effect in v1; no entity-local confidence
  field; estimate-only** (the value facet is a single magnitude with no band). The
  existing `resolve_confidence` becomes spec-traceable.

- **[RV-114 finding F-3] — modify SPEC-020 (prose).** Surface the new requirement in
  the spec body. Two prose edits (before/after below).

### SPEC-020 prose edit — before/after (the `modify` row, surfaced-for-manual)

**Edit 1 — new responsibility bullet** (append under the existing estimate
responsibilities in the `responsibilities:` block / spec frontmatter prose):

> *After:* add —
> "Resolve a project-wide **confidence band** for estimate bounds from
> `doctrine.toml [estimation].lower_confidence`/`upper_confidence` (defaults
> `0.1`/`0.9`; each finite, in `[0,1]`, `low < high`), framing `lower`/`upper` as
> P-low/P-high percentiles for display; the band has no gating, aggregation, or
> normalization effect and no entity-local field."

**Edit 2 — new `### Confidence band resolution` subsection** under the
"Project-wide unit resolution" area of `spec-020.md`:

> *After:* add —
> "### Confidence band resolution
>
> An estimate's `lower`/`upper` bounds carry a project-wide **percentile reading**:
> `lower` sits at P-low, `upper` at P-high. The band resolves from
> `doctrine.toml [estimation].lower_confidence`/`upper_confidence`, defaulting
> `0.1`/`0.9`, each bound finite, in `[0,1]`, with `low < high`. The band **frames**
> the bounds for authoring and display only — it drives **no** predicate,
> aggregation, normalization, or validation in v1 (FR-011). Confidence is
> estimate-only: the value facet is a single magnitude with no band, and `[value]`
> config carries no confidence fields."

### Deferred to /close (not this REV) — code citation follow-through

The 3 `expect(dead_code, reason=…)` strings in `src/estimate.rs`
(`DEFAULT_LOWER_CONFIDENCE`, `DEFAULT_UPPER_CONFIDENCE`, `resolve_confidence`, plus
the `mod display` framing) currently cite "the confidence requirement landing at
SL-104 reconcile". That code lives on the **dispatch bundle `review/104`** (the
immutable audited evidence ref, `35d16875`), **not** `main` — so it cannot be
rewritten here without either mutating the audited bundle or editing stale `main`
code. The one-line citation rewrite to the concrete `REQ-NNN`/FR-011 is therefore
**deferred to `/close`**, applied on `main` after close integrates the bundle.
Recorded in RV-114 `## Reconciliation Outcome` so it cannot be lost.
