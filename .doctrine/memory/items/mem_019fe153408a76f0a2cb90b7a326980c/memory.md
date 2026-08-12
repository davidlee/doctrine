When a hand-written `match` maps an enum's variants to some payload — CLI args
to edit records, config fields to a request, a schema row to a writer — the
failure mode is **drift**: a field exists on the type but no arm maps it. The
mapping still compiles and silently drops data.

Half of that is free if you destructure **exhaustively**:

```rust
KnowledgeFacetEdit::Decision {
    ref target, ref context, ref choice, ref alternatives,
    ref rationale, ref consequences, ref decided_by, ref decided_on,
} => ( … )                       // no `..`
```

A struct-variant pattern that omits a field without `..` does not compile. So
**adding** a field to the variant and forgetting the arm is a build failure, at
the exact line, naming the field. Under `unused = "deny"` you get the other
direction too: binding a field and never using it is also a build failure.

What remains uncovered is **deletion** drift — someone removes the mapping line
*and* relaxes the pattern to `..`, which compiles clean. Only an end-to-end
test that drives the real input and reads the real output sees that. In SL-249
PHASE-04 the two criteria split exactly along this line: the name oracle
(`VT-4`, comparing declared clap flags to the table) stayed green under the
injected defect, while the argv-driven round-trip (`VT-1`) went red. That
asymmetry is the point of having both.

**So:** destructure without `..` in any hand-written mapping, and spend your
one integration test on the deletion case. Reach for `..` only when the variant
genuinely has fields the arm must not care about — and know you have traded the
compiler's guarantee for a test's.


## The construction-side sibling — `..` voids the guarantee in both directions

The same trade exists at **construction** sites, and it is easier to miss because
the ergonomic edit looks like tidying rather than like weakening a check.

A design that says *"adding a field to this struct fails to compile at every
construction site"* — and treats that as the mechanism enforcing an invariant —
is relying on a guarantee that holds **only** while no construction site uses
functional update (`..other`, `..Default::default()`) and the type derives no
`Default`. Either edit silently voids it. Neither produces a warning. The
invariant then rests on nothing, and nothing says so.

`SL-253` `D2` is the case: `Floor` is a struct with one field per member
precisely because `DEC-195` *requires* that adding a member fail to compile, and
a `BTreeMap` or a `const ALL: [_; N]` array would give no such guarantee. The
struct does give it — conditionally, on a condition the design had not stated.

**So:** when a compile-time guarantee is load-bearing for a decision rather than
merely convenient, write the ban down where the decision lives — this type
derives no `Default` and is never built with `..` — so the next person to reach
for the ergonomic form is contradicting a stated rule rather than an unstated
assumption. This is the same family as a guard nobody has watched fail: see
[[mem.pattern.testing.classify-the-expectation-before-trusting-the-assertion]]
for the assertion-side version.