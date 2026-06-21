Report written to `.doctrine/backlog/improvement/112/research-display.md`. Key findings:

**Core gap:** `SliceDoc` has `estimate: Option<EstimateFacet>` and `value: Option<ValueFacet>` (slice.rs:1013,1015), but `format_show` (slice.rs:1126) **never reads them**. The fields are dead data on the show path.

**JSON** already includes them transitively via the `Serialize` derive on `SliceDoc` — no change needed there.

**Display helpers available:**
- `estimate::display` has `format_estimate_normal` / `format_estimate_verbose` — gated by a `dead_code` expectation that explicitly says _"deferred to IMP-112"_ (estimate.rs:50-57).
- `value.rs` has **no display helpers at all** — needs a new `format_value_normal` written.

**Unit resolution** already works: `catalog/hydrate.rs` (lines 185-199) reads `doctrine.toml [estimation].unit` / `[value].unit` into a `Units` struct, defaulting to `"espresso_shots"` / `"magic_beans"`. The same `resolve_units` helper can be reused in `run_show`.

**No changes needed** for backlog (backlog.rs:1159) or governance (governance.rs:290) — those entity types don't carry estimate/value.