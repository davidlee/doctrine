# Info-flow no-derivation invariants: wall at the function signature, not the import or return type

An invariant of the form "value B must never be *derived* from source A, except
through an out-of-band channel (e.g. human input)" is an **information-flow**
property. Rust's type system cannot express information flow (no taint/effect
tracking), so **no choice of types makes the forbidden derivation un-writable** —
if both A and B are in scope, a `match` over A emitting a B literal compiles and IS
the derivation:

```rust
let b = match a_source {            // compiles; IS an A->B derivation
    A::X => B::P,
    A::Y => B::Q,
    _    => prior_b,
};
```

Three places you might try to draw the wall, only the last is right:

- BAD **import boundary** ("the writer must not `use` module A") — fails when the
  writer legitimately needs A for something else (e.g. reads A to *prompt* a human).
- BAD **return-type boundary** ("A's functions never return a B") — fails to the
  `match`-launder above; raises the bar, proves nothing.
- GOOD **function-parameter boundary** — isolate B-selection into a pure fn whose
  parameter list **excludes every A-derived type**:
  `fn select_b(chosen_b: B, prior: B) -> B`. Inside it the compiler *does* prove
  no derivation — you can't use data you were never handed. This shrinks the
  laundering surface from "anywhere" to **one call site** (where the caller wires
  the `chosen_b` argument).

The residual is genuine and unavoidable: the **caller** can still launder before
the call (`let chosen = if a.bad() { B::P } else { args.b };`). Close it with two
more layers — (1) consume A into the out-of-band path (the prompt builder) so it's
not live at the write; (2) a **source-independence test** at the one call site:
hold the out-of-band input fixed, vary **every** A-derived input (not just the
obvious discriminant — the whole composite/aux state), assert B never moves.

**Honest limit:** info-flow is *approximated, never proven* in a non-effect-typed
language. Signature isolation makes the bulk type-impossible; the test covers the
one site left. The test *alone* is too weak (misses channels you didn't enumerate);
the type system *alone* cannot reach it. The combination is the guarantee.

**Why:** drawing the wall at import or return-type granularity is the common
mistake — both feel "structural" but are defeated by the in-line `match`. The
signature is the only granularity where "absence of the parameter" = "absence of
the data" = a real compiler-checked guarantee.

**How to apply:** when a design claims an X-never-derived-from-Y invariant is
"enforced structurally" or "type-level", check whether both types are in scope at
the write. If so, demand the layered wall: B-select fn with Y excluded from its
signature + Y consumed elsewhere + a Y-independence test at the residual site — not
an import ban and not a return-type argument. Surfaced on SL-044 (doctrine NF-001 /
the reconcile writer); the SPEC-002 coverage/status two-store split is the live case.
See [[mem.concept.doctrine.entity-engine]].
