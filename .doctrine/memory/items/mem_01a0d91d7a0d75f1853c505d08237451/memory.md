Under the RFC-026 P10 trial, a `blocker` or `major` finding on a design-review ledger also carries a **route**: which instrument can settle it (`review | demonstrate | probe | control | dedupe | refresh`). The route and the disposition are different axes. The disposition (`aligned | fix-now | design-wrong | follow-up | tolerated`) says what the responder did.

Each has its own flag, and both sets are closed and validated on write:

```
doctrine review dispose RV-NNN --finding F-n --as responder \
  --disposition fix-now --route refresh --response - <<'X'
...
X
```

`--route` is optional on `dispose` and `amend`; omitting it keeps the finding's current route. The MCP `review_dispose`/`review_amend` tools take the same two fields.

**Refused now:** a `route:` prefix inside `--disposition` (`route:probe fix-now`), and a route-only disposition (`route:probe`). The refusal names `--route`. This replaces the earlier prose form, where the route rode as the first token of the disposition string and a dropped vocab token went unnoticed (seen on RV-391 pass 2, SL-267). The closed vocab closed that footgun.

**`owner-fix` is retired** (SL-270, DEC-330): split into `dedupe` (two live accounts of one fact) and `refresh` (a record lags the thing it describes). Writing it is refused; a ledger that already holds it still reads verbatim.

**Reading it back:** the table view of `review show` does not render the route. Use `doctrine review show RV-NNN --json` (`.review.finding[].route`) or the MCP `review_show` output (`findings[].route`, absent while unset). An older ledger still shows the legacy `route:` prefix inside its disposition string, which reads verbatim.

Minor/nit findings take the disposition alone.

Related: `reference/review-ledger.md` ("Route axis"), the design-review rules in `install/design-prompts/reviewing.md`.

**The design-run lock reads it** (SL-270, DEC-326): the lock refuses while a disposed `blocker`/`major` finding on the design review has no known route (absent, a legacy `route:` prefix, or `owner-fix`), naming each as `F-n (<reason>)`. Repair: answered → `review amend RV-NNN --finding F-n --route <route> --response … --note …`; contested → dispose again with `--route`; verified → raiser reopens, responder disposes again with `--route`.