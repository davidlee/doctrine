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

### T4 / T5 — the template pin and its control

**T4 green on first run**, as A-5 predicted and as the design intends: R5 is a
*standing* pin over future template edits, not a red/green step. Its input is
`render_record_toml_seed`, whose own `match` owns the kind → template-path
mapping, so the test restates no path list.

**T5's control fired.** Deleting `decided_on = ""` from
`install/templates/knowledge-decision.toml` turned T4 **RED** naming decision:

    assertion `left == right` failed: decision: the shipped template must seed
    exactly its table row
      left:  {alternatives, choice, consequences, context, decided_by, rationale}
      right: {alternatives, choice, consequences, context, decided_by,
              decided_on, rationale}

Restored with `git restore --source=HEAD -- install/templates/…` (explicit
pathspec), rebuilt, **PASS**. `git status --porcelain` shows the template clean.
STOP-4 did not fire.

### R2 re-probed — the re-embed footgun does not reproduce

R2 and `mem_019e98a783ea7471ac4bfcefdc04ae5e` both say a lone `install/` edit
leaves the binary carrying the old asset until you `touch` the embedding file.
**Not true in this tree today.** From a fully built worktree:

    cargo build   # Finished, 0.05s — nothing to do
    <delete one key from install/templates/knowledge-decision.toml>
    cargo build   # Compiling doctrine … — rebuilt, with NO touch

and the test read the new asset on the first run without a touch anywhere.
rust-embed 8 registers its `#[folder]` files as build dependencies, so cargo's
staleness check already covers them. The touch is a harmless escape hatch, not a
precondition — and if reached for, the file is `src/asset_source.rs:19`, since
SL-223 moved the embed off `src/install.rs`.

The memory was amended in place rather than left to misdirect: title, summary
and body now carry the re-probe, keep the still-live residue (never trust
`Finished`; verify through the render; the nix/crane `cleanCargoSource` strip is
a *separate* and still-real trap), and bound the claim to the roots actually
probed. The planner had already flagged two of its three stale claims; the third
— the core one — needed the probe.

### T6 / T7 — the tripwire, and a sixth site the sheet did not name

**Red observed:** `inert_facet_key_findings` and `Category::InertFacetKey`
absent. Four VT-3 cases: a populated inert `confidence` on a decision reported
once naming `DEC-001` and **both** honouring kinds (D7 — `confidence` chosen
deliberately, it is the one field two kinds own); a `frobnicate` key no kind
honours reported as *"no record kind honours it"* (D6); a clean corpus plus a
seeded-but-**empty** inert key reporting nothing (D5); and
`knowledge::run_list` returning `Ok` on the damaged corpus (EX-7).

**R5's double-report probe, run before wiring #12.** `doctrine doctor --json` on
this corpus returns `{Raw Label: 952, Prose Citation: 82, Lifecycle: 5, Coord
Hook: 1}` — **zero `TOML Parse` rows**. No knowledge record produces a `TomlParse`
finding today, so D8's skip rule is sufficient and there is no #7/#12 overlap to
report. R5 discharged.

**The new `Finding` category cost SIX sites, not five.** The sheet named the
enum, `severity`, `ordinal`, `display_name` + its const, and
`CATEGORIES_BY_ORDINAL`, plus the hand-enumerated `test_severity_mapping`. There
is a sixth, and it is the dangerous one: `render_findings`
(`src/finding.rs:225`) carried its own `let mut by_category: [Vec<&Finding>; 11]`
bucket array. Its bucketing is `by_category.get_mut(idx)` — so a category whose
ordinal exceeds the hand-written length is **silently dropped from the render and
from the total**, with no panic and no lint. A doctor check that reports nothing
looks exactly like a clean corpus.

`test_render_all_categories` (`:343`) caught it, as R6 said it would — it is the
canary, and it earned its keep. Fixed by **deriving** the length rather than
bumping it:

    let mut by_category: [Vec<&Finding>; CATEGORIES_BY_ORDINAL.len()] =
        [const { Vec::new() }; CATEGORIES_BY_ORDINAL.len()];

which retires the site permanently instead of leaving a seventh trap for
category #13. Same STD-001 argument as D4's.

### T8 — the staged expects retire, all fourteen

`cargo build` after T7 reported **`this lint expectation is unfulfilled` for
every one of the 14** staged `cfg_attr(not(test), expect(dead_code, …))`. All
removed; build and suite green.

**R1's prediction was wrong in the other direction too.** It predicted
`FacetField::shape` and `FieldShape` would *stay* dead until PHASE-04's shape
dispatch, since no production code reads the shape column. They did not:
`#[derive(Debug, Clone, Copy, PartialEq, Eq)]` generates code that reads every
field, so a derive alone satisfies `dead_code` for a field its production callers
never touch. Combined with the T2 finding (a never-constructed struct subsumes
its fields' deadness), the rule is sharper than
`mem.pattern.lint.dead-code-staged-ahead-cfg-test` states: **deadness is a
property of the item's whole reachable graph including derive-generated code, so
never predict which items need staging — build, read rustc, and iterate.** Two
predictions, two misses, in opposite directions.

### T8 — VA-1

Read over the whole phase range (`c1940d7c8^..` + working tree). Every quoted
facet field name in an added line is a `name: "…"` row inside `facet_fields`'
seven per-kind consts — **31 of them, one table**. The only other hit is
`finding.message.contains("confidence")`, a substring assertion on a message.
Both consumers added here call `crate::knowledge::facet_fields`
(`doctor_checks.rs:168` for the owned set, `:222` in `honouring_kinds`); neither
carries a field list. Positive control: the same grep returns 5 hits for
`"(claim|confidence|basis)"` over the diff, so "no consumer restates it" is a
reading and not a broken command.

### T9 — EX-9 / I1, and the A1 canaries

Per **item**, not per file (`src/knowledge.rs` necessarily changed). Each item
extracted from `c1940d7c8^` and from the working tree by brace-matching and
compared byte-for-byte:

| item | bytes | verdict |
|---|---|---|
| `populated_fixture` | 2167 | unmodified |
| `populated_record_round_trips_byte_stable_per_kind` | 778 | unmodified |
| `populated_record_round_trips_into_shared_meta` | 509 | unmodified |
| `render_escapes_hostile_facet_values` | 927 | unmodified |
| `render_record_toml` | 728 | unmodified |
| `opt_text_line` / `list_line` / `render_facet` / `render_evidence` | 124 / 104 / 3305 / 244 | unmodified |
| `validate_facet` (A1) | 2459 | unmodified |
| `RecordFacet` + all six variant structs (A1) | — | unmodified |

**EX-9 holds on the criterion as written — passing *unedited*, not passing.**
A1 holds too, so STOP-1 and STOP-5 did not fire. Positive control: the same
extractor *does* find this phase's own additions in the same file (they are
absent at base and present at head), so "unmodified" is a reading.

The phase's whole production delta is: the table + `Serialize` on `RawFacet` +
the four `KNOWN` visibility promotions (`src/knowledge.rs`), the tripwire
(`src/doctor_checks.rs`), the category and the derived bucket array
(`src/finding.rs`), and check #12's wiring (`src/commands/doctor.rs`).
`git diff --stat c1940d7c8^ -- src/` = 4 files, 693 insertions, 27 deletions.
