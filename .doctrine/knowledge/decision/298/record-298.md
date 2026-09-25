A pure `route(prefix) -> Option<Route>` names every row of `kinds::KINDS`,
matching on the named prefix constants, and falls through to `None` with no
assertion; dispatch is a compiler-exhaustive `match Route`; a test asserts every
`KINDS` row routes, with a negative control asserting a synthetic unrouted prefix
returns `None`.
