# EntityFacets struct — created by SL-132, extended by later slices

SL-132 creates `src/facet.rs` with:
```
pub(crate) struct EntityFacets {
    pub(crate) estimate: Option<EstimateFacet>,
    pub(crate) value: Option<ValueFacet>,
}
```

Consumed by `format_show` (SL-132) and `build_priority_graph` (SL-133).

When SL-133 design proceeds, extend with `risk: Option<RiskFacet>`.
When SL-136 (tagging) proceeds, extend with `tags: Vec<String>`.

`run_show` constructs EntityFacets from `SliceDoc` fields. Future call
sites populate additional fields; `format_show` ignores fields it
doesn't render — no refactoring needed.
