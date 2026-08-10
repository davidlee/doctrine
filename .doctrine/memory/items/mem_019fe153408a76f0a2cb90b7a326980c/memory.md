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
