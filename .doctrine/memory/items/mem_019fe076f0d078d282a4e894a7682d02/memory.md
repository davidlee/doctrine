# Staging a type ahead of its consumer: check the derives before reaching for `expect(dead_code)`

[[mem.pattern.lint.dead-code-staged-ahead-cfg-test]] says to gate an item that
lands a phase before its consumer with
`#[cfg_attr(not(test), expect(dead_code, reason = "…"))]`, so it can ship
without a blanket suppression. Sound — but its precondition is that **nothing**
reads the item under `cfg(not(test))`, and a `struct` with value-semantics
derives does not meet it.

**Measured (SL-248 PHASE-02, `src/interpretation.rs`).** `InterpretationPolicy`
derives `Debug, Clone, PartialEq, Eq`. Its fields' only *hand-written*
production readers were two tasks away, so the attribute went on the struct.
rustc rejected it immediately:

    error: this lint expectation is unfulfilled
    `-D unfulfilled-lint-expectations` implied by `-D warnings`

The **derived `Clone` and `PartialEq` impls read every field**, so the fields
are live from the moment the type exists. `Debug` alone does *not* count —
rustc deliberately ignores it for dead-code purposes — so "a derive keeps it
alive" is not a reliable rule of thumb in either direction.

**How to decide, cheaply.** Do not reason about it; compile. Add the item
without the attribute and see whether `dead_code` actually fires. Under
`warnings = "deny"` both outcomes are hard errors, so the compiler answers in
one cycle either way — and an unfulfilled `expect` is the *more* confusing
error of the two, because it reads as if the lint config is wrong rather than
the premise.

**Where the original pattern still applies:** free functions, consts, and
structs carrying no derive beyond `Debug` — items with genuinely no reader
until the next phase. See also
[[mem.pattern.lint.dead-code-blanket-masks-siblings]] and
[[mem.pattern.lint.expect-not-allow]].


## Second confirmation, and a second way the expect comes back unfulfilled

**SL-249 PHASE-03** staged a whole table (`FacetField` / `FieldShape` /
`facet_fields` + seven row consts) one task ahead of its production consumer and
hit this twice more. Both times the compile-don't-reason rule above was what
resolved it; both times a *prediction* written into the phase sheet was wrong.

**(a) A never-constructed struct subsumes its fields' deadness.** A per-field
`#[cfg_attr(not(test), expect(dead_code, …))]` on `FacetField::shape` came back
unfulfilled while the struct itself was still dead. rustc reports the struct
(`struct FacetField is never constructed`) and stops; the field's own deadness is
never separately diagnosed, so there is nothing for the field's expect to fulfil.
**A field of a dead struct must NOT carry its own expect** — put it on the struct
only, and revisit once the struct is constructed. This qualifies
[[mem.pattern.lint.dead-code-staged-ahead-cfg-test]]'s "every item in a staged
chain carries its own expect": every *separately diagnosed* item does, and a
field of a dead struct is not one.

**(b) The derives rule bit again, in the opposite direction from the plan.** The
sheet predicted `FacetField::shape` and `FieldShape` would *stay* dead after the
production consumer landed, because that consumer reads only `.name` and never
the shape column. They went live anyway: `#[derive(Debug, Clone, Copy, PartialEq,
Eq)]` generates code reading every field, exactly as this memory records. All 14
staged expects retired in one build.

The generalisation across both: **deadness is a property of the item's whole
reachable graph, including derive-generated code and including which enclosing
item rustc diagnoses first.** It is not predictable by reading your own call
sites. Add the item bare, build, and let rustc name the set — under
`warnings = "deny"` that costs one cycle and is never wrong.
