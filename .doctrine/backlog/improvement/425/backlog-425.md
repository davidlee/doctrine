# IMP-425: Give production closure expansion a live exercise

`resolved_closure_root` → `expand_closure_root` → `closure_members`
(`crates/doctrine-control/src/backend/bubblewrap.rs:895 / :910 / :837`) is the
production path that turns a declared `closure-root` into bound readable inputs.
It has **zero live exercise**: every test driving it supplies a `FixtureQuery`
double, so what is proven is that the parsing and refusal logic behaves against a
table, not that the spawned-resolver seam works against a real host.

`REV-051` ruled `REQ-459` criterion 2 partial partly for want of production
evidence. This is one of the gaps behind that ruling.

## Where this came from

`SL-252` intended to close it by having the conformance fixture declare
`closure-roots` and a resolver, so the fixture's own capsules would be built
through the production path (`DEC-186`). Two things stopped it:

- `DEC-188` took `DEC-185`'s S3 fallback, so the fixture declares no closure
  roots at all and there is nothing for a resolver to compute.
- `DEC-186`'s chosen mechanism — a fixture-written `sh` script using
  `LD_TRACE_LOADED_OBJECTS` — was refuted on review and withdrawn. Read
  `DEC-186`'s withdrawal section before proposing any successor; it records five
  specific defects, of which the sharpest is that a pathless trace line may be an
  unresolved dependency rather than a virtual object, so discarding it silently
  converts a broken closure into a successful one.

## What a successor must carry

From `DEC-186`'s withdrawal, and true regardless of mechanism:

- the resolver must **echo the queried root** — `expand_closure_root` binds only
  what the resolver returns and never adds the realised root it was given;
- **exit status dominates plausible stdout**, checked before parsing;
- unknown or unresolved forms **fail closed**, never filter away;
- if closure resolution cannot be established, the readable set must **not**
  widen to compensate;
- whatever host capability the mechanism depends on is declared under `POL-002`
  facet (3) and preflighted, with an explicit unavailable outcome on absence.

## Sequencing

Probably blocked behind `ISS-344` — while readable inputs are bound at their
resolved path, a closure-shaped readable set cannot run at all on a
store-managed host, so an exercise built on one would be testing a shape nothing
can use. And `RSK-231` may reframe the question entirely: if the containment
primitive changes, the declared-inputs-plus-closure-resolver model may not
survive to need exercising.

Do not revive this as a standalone slice ahead of that architectural call.
