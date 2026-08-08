# SL-249 — execution record, PHASE-01 … PHASE-03

Shard per LOOP.md § Notes, sharded. Append-only; PHASE-01/02's record lives in
the slice's own history, this shard opens at PHASE-03.

## PHASE-03 — The facet field table and the standing scan

### T1 / T2 — the table and its three pins

**Red observed** (`cargo test --bin doctrine`): four items absent —
`FacetField`, `FieldShape` (masked), `facet_fields`, `RawFacet: Serialize`. A
compile failure, so it proves the pins *run*, not that they *catch*; T3 is the
compensating control.

**Divergence from R1's prediction — recorded because R1 asked for it.** R1
predicted that `FacetField::shape` and `FieldShape` would need staging and that
`name` / `facet_fields` / the `KNOWN` consts would go live at T7. What rustc
actually said on the non-test build was **14 dead-code errors**, and the shape of
the set was not the predicted one:

- the four `KNOWN` consts, the `FacetField` struct, the seven per-kind row
  consts, and `facet_fields` — thirteen items, each needing its own
  `cfg_attr(not(test), expect(dead_code, …))`. The seven row consts were not in
  the prediction at all.
- `FacetField::shape`'s own expect came back **unfulfilled** (`error: this lint
  expectation is unfulfilled`, `-D unfulfilled-lint-expectations`). Cause: a
  struct that is never *constructed* hides its fields' deadness — rustc reports
  the struct and stops, so a per-field expect has nothing to fulfil. The expect
  had to be removed, not added.

That last one is the generalisable part and is new relative to
`mem.pattern.lint.dead-code-staged-ahead-cfg-test`, which says every item in a
staged chain carries its own expect. Not every item: a field of a dead struct
must *not*, because its deadness is subsumed. Recorded as a memory amendment
candidate at T10.

**Name collision, pre-existing, worth a ruling before PHASE-04.** rustc's own
"consider importing this enum: `use crate::facet_write::FacetField`" revealed
that `src/facet_write.rs:56` already declares a `FacetField` — an unrelated enum
carrying a *value to write* (`Str { key, value }` / `Arr { key, values }`). The
new `knowledge::FacetField` carries a field's *declaration* (`name`, `shape`).
Different modules, so it compiles, but PHASE-04's `plan_facet_edits` builds the
second from the first and will import both. The design (§5.1) names the type
verbatim, so this phase implements it as written rather than renaming
unilaterally. See Findings for the minted id.

### T3 — the injected-defect control

(recorded below as it is run)
