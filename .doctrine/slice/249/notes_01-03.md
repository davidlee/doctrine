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

T1's red was a compile failure, which proves the pins *run*, not that they
*catch*. Five defects injected one at a time, each reverted immediately
(`git restore --source=HEAD -- src/knowledge.rs`); `git diff --stat
src/knowledge.rs` empty at the end, so nothing survived.

| # | injected defect | I2 | I3 | I3b |
|---|---|---|---|---|
| a | `choice` moved from decision's row onto question's | green | **RED** (decision) | green |
| a′ | `choice` *added* to question's row, decision's left intact | green | **RED** (question) | green |
| b | `rationale` deleted from decision's row | **RED** | green | green |
| b′ | `confidence` deleted from evidence's row (assumption still owns it) | green | **RED** (evidence) | green |
| c | `context` duplicated inside decision's row | green | green | **RED** (8 vs 7) |

(a) fires on decision rather than question only because `assert_eq!` aborts at
the first kind in `RecordKind::ALL` order and decision precedes question; both
halves of the defect are real. (a′) is the pure form of RV-349 `F-3` round one —
a row handed a field its kind does not own, with the union unchanged so `I2`
cannot see it. It failed with `left: {answer, answered_by, answered_on,
question, why_matters}` / `right: {… choice …}`: the table claims `choice`,
`validate_facet` retains nothing of the sort. An inclusion pin would have passed.

(c) is the reason `I3b` exists: `I2` and `I3` both stayed **green** under a
duplicated row entry, exactly as `F-3` round two predicted, while every consumer
that iterates the row would see `context` twice.

**Divergence from the sheet, and why it is not a hole.** T3(b) is specified as
"`I3` must fail on decision AND `I2` must fail". `I2` failed; **`I3` did not**.
Cause: `I3`'s input is `populated_union()`, built from the table's own rows, so a
field deleted from *every* row is absent from the input as well — both sides of
the comparison shrink together and `I3` is structurally blind to it. That is not
a gap in the pin set, it is the division of labour the design's § 5.1 already
draws: totality is `P1`/`I2`'s job, placement is `P2`/`I3`'s. To confirm the set
is complete rather than merely assume it, (b′) was run as the complementary
case — a field dropped from *one* row while another still owns it, so the union
stays total and `I2` cannot see it. `I3` caught it. Between them the four defect
classes are covered:

| defect class | caught by |
|---|---|
| model field with no row anywhere | `I2` |
| row naming a key no model field carries | `I2` |
| field on a row whose kind does not own it | `I3` (a′) |
| field missing from one row, present in another | `I3` (b′) |
| field missing from every row | `I2` (b) |
| name repeated within one row | `I3b` (c) |

No STOP: the criteria that bind are `EX-2`/`EX-3`/`EX-4` and each holds. What
diverged is the planner's per-test attribution for one injected defect, recorded
here so the audit reads the measurement rather than the prediction.
