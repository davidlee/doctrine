Under the RFC-026 P10 trial, a `blocker` or `major` finding on a design-review ledger carries a route written as the first token of its disposition, of the form `route:<route> <vocab>` - e.g. `route:owner-fix fix-now`. The route and the vocab are different axes: the route says what instrument can settle the finding (review | demonstrate | probe | control | owner-fix); the vocab says what was done.

The footgun: `doctrine review dispose` accepts a disposition that names only the route (`route:owner-fix`), so the vocab token is silently droppable and nothing rejects it. Seen on RV-391 pass 2 (SL-267): four re-dispositions recorded the route but not `fix-now`; the raiser noted it in the synthesis rather than contesting, and logged the tool's non-rejection as friction.

Always include the vocab token. Minor/nit findings take the disposition alone (no route required).

Related: `reference/review-ledger.md` (the protocol), the design-review rules in `install/design-prompts/reviewing.md`.