# REV REV-022 — ADR-015 value provenance amendment

Revision (ADR-013) — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

RFC-019 (comparison-based value elicitation, externally reviewed under RV-260)
changes the *provenance* of the `value` input to `value_dim` — the score
formula itself does not change. It also exposed two defects in ADR-015's
current prose:

1. **Stale absent-value semantics (RV-260 F-3).** ADR-015 states
   "absent value ⇒ `value_dim = 0`". The live engine disagrees: since SL-177,
   every value-bearing kind with no `[value]` facet scores at
   `DEFAULT_VALUE = 1.0` (`src/priority/graph.rs`), including in the burndown
   path. The absent-value *identity* is policy, not an implementation-owned
   numeric default — the ADR text states the wrong policy and must be
   corrected to the behaviour the corpus already lives under.
2. **Unstated value-source resolution.** With RFC-019's constraint layer,
   an item may have an authored magnitude, comparison-derived bounds, both,
   or neither. ADR-015 currently assumes a single source. The resolution
   policy (RFC-019 T3(a), codex-reviewed): **authored wins; projection fills
   absence; `DEFAULT_VALUE` fills the rest.** An authored magnitude is also a
   point anchor in the constraint layer — it propagates bounds to everything
   compared against it — and an authored value that conflicts with derived
   bounds is a **surfaced contradiction**, never a silent win. Projected
   values are derived at read time (pure over ledger + config + anchors),
   exactly parallel to the ADR's existing "no score is authored to disk".

Two amendment-scope questions RFC-019 explicitly refused to settle by
implication (RV-260 F-2), adjudicated at this REV's approval gate:

- **Q1 — `value set` on non-value-bearing kinds.** The authoring surface
  accepts `[value]` on records and governance kinds; the scoring consumer
  ignores it (records are not in `VALUE_BEARING`). Position taken: **warn,
  don't refuse** — the write is inert, refusal is a breaking surface change
  with migration cost, a warning surfaces the misunderstanding (A2: records
  have no intrinsic value) at the moment it happens. Implementation rides an
  RFC-019 phase, not this REV.
- **Q2 — RSK's dual participation.** `VALUE_BEARING` includes RSK, so risks
  score in `value_dim` *and* `risk_dim`; RFC-019 A4's "risk is a different
  currency" argument puts that in tension. Position taken: **retain, defer.**
  Stripping RSK from `VALUE_BEARING` changes live scoring for existing
  corpora and is not needed by RFC-019 (RSK is simply not admissible in
  value-domain comparisons). Revisit trigger: evidence that the double count
  distorts frontier ordering in practice.

Scope of the staged delta: one `modify` row (ADR-015). Applied as a
surfaced-for-manual hand-edit after approval; `doctrine boot` after landing.

### Change row 1 — modify ADR-015

**Edit 1.1 — §1 Base dimensions › absent-data list (stale prose fix).**

Before:

> - absent value ⇒ `value_dim = 0`;

After:

> - absent value on a value-bearing kind ⇒ `value = 1.0` (`DEFAULT_VALUE`,
>   SL-177) — unvalued work scores at unit value rather than disappearing
>   from `value_dim`; with comparison evidence (RFC-019), absent authored
>   value resolves through projection first (see *Value-source resolution*),
>   and projection's unanchored fallback reproduces this same default;

**Edit 1.2 — §1 Base dimensions › new subsection after the absent-data
list: value-source resolution.**

Insert:

> ### Value-source resolution (REV-022, RFC-019)
>
> The `value` input to `value_dim` resolves by provenance:
>
> 1. **Authored** `[value]` magnitude — wins outright, and additionally acts
>    as a point anchor in the comparison constraint layer, propagating bounds
>    to items compared against it.
> 2. **Projected** — items with comparison evidence but no authored value
>    take the deterministic point projection within their derived bounds
>    (RFC-019 constraint layer).
> 3. **Default** — items with neither take `DEFAULT_VALUE = 1.0`.
>
> An authored value that conflicts with comparison-derived bounds is a
> surfaced contradiction (diagnosed like a needs cycle), not a silent win.
> Projected values and bounds are derived at read time from the comparison
> ledger; they are never authored to disk (the same posture as scores).

**Edit 1.3 — §4 Config contract › add the value-fit domain.**

After the `[priority.estimate]` block entry, insert:

> ```toml
> [priority.value_fit]
> # projection knobs (implementation-owned)
> ```

And append to the durable-domains list:

> - value-source resolution order (authored → projected → default) is fixed
>   policy, not a knob; projection is deterministic given (ledger, config,
>   anchors); contradiction surfacing cannot be suppressed by configuration.

**Edit 1.4 — §5 Sort and explain contract › explain bullet.**

Before:

> - `explain` exposes the score breakdown: base, value dimension, risk
>   dimension, recursive leverage, one-hop optionality, and total.

After:

> - `explain` exposes the score breakdown: base, value dimension, risk
>   dimension, recursive leverage, one-hop optionality, and total. The value
>   line labels its source and, when projected, the bounds and evidence
>   weight — e.g. `value: 6.2 (projected, bounds (3.2, 9.1), 9 judgements)`
>   vs `value: 8.0 (authored)`.

**Edit 1.5 — Consequences › Neutral › future-work bullet.**

Before:

> - No per-item priority override is introduced. Overrides, cost/maintainability
>   dimensions, and score history remain future work.

After:

> - No per-item priority override is introduced beyond the authored value
>   anchor (authored-wins, REV-022). Cost/maintainability dimensions remain
>   future work (IDE-035 holds the trigger); the comparison ledger (RFC-019)
>   is a value-evidence history, but score history as such remains future
>   work.

**Edit 1.6 — References › append.**

> - RFC-019 — comparison-based value elicitation; source of the value-fit
>   provenance policy and the constraint layer this amendment admits.
> - RV-260 — external adversarial review of RFC-019; F-2/F-3 drove the
>   authoring-surface and absent-value corrections.
> - REV-022 — this amendment.
