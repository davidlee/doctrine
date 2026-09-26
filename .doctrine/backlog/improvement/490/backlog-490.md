# IMP-490: review: finding index on show + list --target (RFC-032 0c quick win)

RFC-032 (*Review ledger effectiveness*) sequences this as step 0c — a **quick
win**, not a slice. The RV read-back surface was never designed: `review show`
renders the brief and a finding *count*, so the entire finding tier is reachable
only via `--json`. Nine recorded friction observations across four slices
(RFC-032 `research.md` F1). Split out of `IMP-475`, whose `show --findings`
opt-in was rejected by the RFC's `D5` — findings should be the default, not a
flag.

## Scope

Two renders over **existing** ledger fields; no schema change.

1. `review show` renders the **finding index** by default: one row per finding —
   `id │ severity │ status │ disposition │ title` — under the existing
   `findings: N (raiser … · responder …)` line.
2. `review list --target <ref>` admits only reviews whose `reviews` edge targets
   that subject. The filter is on the bare `[target].ref`, so `SL-024` also
   admits a `SL-024@PHASE-03` edge. Also exposed on the MCP `review_list` tool.

## Approach (the D5 quick-win lane)

Both renders read one **view struct** — `ReviewView`, built once from the
authored `ReviewDoc` by `ReviewView::of`, carrying the derived status/await and
`Vec<Finding>`. Nothing formats the raw doc; the index renders through the shared
`listing::render_table`, not bespoke formatting. The RFC's `D4` (a ledger module)
can move this projection without rewriting a renderer, and the `D5` remainder
joins the same struct.

Deferred to RFC-032 slices 1–2 (not this item): the `--finding F-n` detail view,
the `--status`/`--severity`/`--open` filters, the census (`IMP-477`), and the
CLI/MCP JSON unification (`IMP-476`, `D5` remainder).
