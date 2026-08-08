# SL-248 — execution record, PHASE-07 … PHASE-09

Shard per `LOOP.md` § *Notes, sharded*. What was done, what diverged from the
phase sheet, what was measured. Cross-phase items and anything owed to the
reconciliation brief stay in `notes.md`.

## PHASE-07 — conformance classification and verdict algebra

One new file, `crates/doctrine-control/src/conformance.rs` (1,978 lines), plus
the two append-only edits the sheet permits (`S1`): `mod conformance;` in
`crates/doctrine-control/src/main.rs` and one `[doctrine_control_tiers]` row in
`.doctrine/adr/001/layering.toml`. `backend.rs` untouched — `git diff` empty,
which is `VA-1`'s first half in full.

Result: **137 tests green** in `doctrine-control` (114 on arrival, 23 new).
`doctrine check gate` exit 0. All nineteen mandated `VT` keywords present across
`VT-1`/`VT-2`/`VT-3`.

### The one-compile wall (`C5`, `R1`) — smaller than PHASE-04's

PHASE-04 measured 69 errors before its first green. This phase's first
`cargo test` produced **three**, and they were all signal rather than staging
noise:

1. two `unpredictable_function_pointer_comparisons` (see divergence 1);
2. one unfulfilled `expect(dead_code)` on `Fixture`, which turned out **not** to
   be dead — a type named in `Delta::Widened`'s payload signature and in a test
   helper's parameter is read, so `D4`'s forward declaration needed no
   suppression at all.

The whole production surface plus the whole test module were written in one pass
and compiled together, per `C5`. Nothing was gained by sequencing and nothing was
lost by not.

### What diverged from the sheet

1. **`PidProbe`, `ArmShape`, `Delta` and `Row` do not derive `PartialEq`/`Eq`.**
   `PidProbe.argv` is a `fn(HostPid) -> Argv` — `D3`'s pid-rendered payload — and
   deriving `PartialEq` over a function pointer is
   `unpredictable_function_pointer_comparisons`, denied under `-D warnings`,
   because fn addresses are not unique across codegen units. The three types that
   contain it lose the derive transitively. Nothing needs to compare two
   payloads; what the tests compare is the `ArmResult` a payload produced, so no
   assertion was weakened. The cost is entirely in dead-code accounting — see the
   residual below.

2. **`verify_over` takes the auxiliary list as a parameter**, which is one field
   wider than `D2`'s `verify_over(backend, host, today, &tables(), &run_row)`.
   Forced by `M13`/`M14`: those two mutations let a `Failed` and a `Skipped`
   auxiliary outcome reach the admission, and a test cannot red them unless it
   can *inject* an auxiliary outcome. `verify`'s own three-parameter signature
   (`EX-2`) is untouched, `admission(rows)` still takes the row list alone — so
   *table C cannot reach admission* stays structural rather than promised — and
   `tables()`/`auxiliary_claims()` remain private. Recorded as `notes.md` item 52.

3. **The battery was run with `cargo test -p doctrine-control`, not
   `doctrine check gate`.** The sheet says the gate; `C10` says the gate. The
   battery's signal is *which tests red*, and a mutation that happens to trip a
   lint (an unused import, a now-unreachable branch) fails the build before any
   test runs and reports nothing. Fifteen full gates would also have cost roughly
   fifteen minutes of workspace clippy and `validate` for no added signal. The
   final green was taken with `doctrine check gate` (exit 0), which is what `C10`
   is actually protecting.

4. **`no_liveness_marker_is_indeterminate_not_failed` carries a trailing
   fixture-sanity block.** Beyond the three negative fixtures the sheet mandates,
   it asserts that each of the three, *with* the marker restored, reads its own
   positive answer. This is the "assert the discriminating property itself"
   follow-through from `mem.pattern.tests.mutation-needs-a-discriminating-fixture`.
   It is also why `M4` reds this test as well as its own — see the battery notes.

### The dead-code residual (`T10`), measured

Five suppressions in the file. One is the module header `D5`'s convention
requires; four are item-level and every one self-clears at a named phase:

| site | what | cleared by |
|---|---|---|
| module `#![cfg_attr(not(test), expect(dead_code, …))]` | the whole unit is dead under `cfg(not(test))` until PHASE-10's `backend verify` calls `verify` | PHASE-10 |
| `enum ArmShape` | the payloads are executed by the harness; nothing runs them yet | PHASE-08 |
| `enum Delta` | a delta's payload is *applied* by the harness — the removal reaches `execute_weakened`, the widening is called with the fixture | PHASE-08 |
| `struct Row` | `run_row` reads `shape` and `delta` | PHASE-08 |
| `enum RowId` | `RowId::Property` is uninhabited while `Property` is empty (`EX-14`) | PHASE-09 |

PHASE-06 cleared five files of these; PHASE-08 deletes three of the four above
and PHASE-09 the fourth. `Fixture`, `Property`, and every function and constant
needed none.

**The altitude lesson, which cost a cycle.** All four were first written on the
*field* or the *variant*. That fails: rustc reports dead code at the outermost
dead item and never descends, so under `cfg(not(test))` — where the whole struct
is dead — the lint fires at the struct and the field-level expectations are
unfulfilled, which `unfulfilled_lint_expectations` makes a hard error. Written at
item level, one attribute is fulfilled in **both** compilation units, for
opposite reasons: the field lints under `cfg(test)`, the item lint under
`cfg(not(test))`. Recorded as `mem.pattern.lint.expect-dead-code-at-item-level`.

A second cycle went to a memory that was wrong.
`mem.pattern.lint.dead-code-derives-count-as-reads` said "the derived `Clone` and
`PartialEq` impls read every field"; rustc's own note says
*"has derived impls for the traits `Clone` and `Debug`, but these are
intentionally ignored during dead code analysis"*. Only `PartialEq` counted, and
divergence 1 had just removed it. Corrected in place, and captured as a friction
observation — the memory generalised past its measurement and was retrieved at
exactly the moment it was load-bearing.

### The mutation battery (`T11`) — 15 of 15

Applied to green source, restored **by copy** from a scratch snapshot and
`diff`-verified after every run (`C6`); `git checkout` never used. Driver kept at
`/tmp/sl248-battery/battery.py`; each mutation's anchor text was asserted unique
before substitution, so a silently-missed edit could not read as "reds nothing".

| id | reds | verdict |
|---|---|---|
| `M1` | `no_liveness_marker…`, `an_arm_specific_breakage…`, `an_indeterminate_arm_carries…` | must-red ✓, must-not-red (`both_tokens…`) ✓ |
| `M2` | `both_tokens_present_is_ambiguous_not_held` | exact ✓ |
| `M3` | `a_missing_value_line…` | exact ✓; the wrong-value sibling stayed green ✓ |
| `M4` | `a_termination_observation…`, `no_liveness_marker…` | must-red ✓, the `Token` tests green ✓ |
| `M5` | `an_observed_execution_that_never_calls_back…` | exact ✓; `a_subject_that_exited…` green ✓ |
| `M6` | `a_subject_that_exited_before_the_observer_ran…` | exact ✓; `an_observed_execution…` green ✓ |
| `M7` | `held_control_is_unproven…`, `a_backend_ignoring_its_removal…` | exact ✓ |
| `M8` | `failed_probe_is_violated…`, `an_indeterminate_arm_is_never_proven` | must-red ✓, `held_probe_and_failed_control…` green ✓ |
| `M9` | `an_indeterminate_arm_is_never_proven`, `an_arm_specific_breakage…` | exact ✓ |
| `M10` | `admitted_requires_every_row_proven`, `auxiliary_outcomes_do_not_reach…` | must-red ✓, `an_unavailable_backend…` green ✓ |
| `M11` | `an_unavailable_backend_is_not_admitted_and_runs_no_row` | exact ✓ |
| `M12` | `every_row_id_is_covered_by_exactly_one_table` | exact ✓, everything else green ✓ |
| `M13` | `auxiliary_outcomes_do_not_reach_the_admission` | exact ✓; `admitted_requires_every_row_proven` green ✓ |
| `M14` | `a_skipped_auxiliary_claim_leaves_the_verdict_admitted` | exact ✓; `auxiliary_outcomes…` green ✓ |
| `M15` | `a_host_without_a_usable_shell_is_unavailable_not_violated` | exact ✓ |

**Nothing redded nothing.** First phase in four where no mandated test was inert
— see `notes.md` item 55 for why that is probably the sheet's doing rather than
the worker's.

**The four sole-evidence rows all discriminated.**

- `M1` — the three fixtures each carry a *positive* observation (the failed
  token, a wrong value line, the expected termination) with the marker absent, so
  deleting the liveness gate moves each to a different answer. A fixture with
  nothing to read would classify `Indeterminate` under both rules.
- `M8` — probe `Failed` **and** control `Failed`. With a `Held` probe, reading
  the control first gives the same answer and the mutation is invisible.
- `M11` — the real tables are empty, so this is vacuous against `verify`. Driven
  through `verify_over` with a four-row hand-built set and a call-counting
  runner; the assertion is `calls == 0`, with a positive control asserting the
  same runner is called `rows.len()` times when the backend is available.
- `M13`/`M14` — the two auxiliary tests are kept **disjoint by outcome kind**
  (one carries `Failed` + `Passed`, the other only `Skipped`) so that neither
  mutation can red the other's test. `R3`'s trap, one level up.

**`R3` is closed by measurement.** The two concurrent-classification tests were
the phase's most likely inert pair: both classify `Indeterminate`, so an
assertion of the *verdict* passes under both `M5` and `M6`. `D5`'s distinct
reasons are what make them separable, and the tests assert the **reason**
(`NoLiveness` vs `NoObservation`) plus `assert_ne!(arm, Held)`, each with a
positive control that would read `Held` if its branch were deleted. `M5` and `M6`
red exactly their own test and neither reds the other's. Appended to
`mem.pattern.tests.mutation-needs-a-discriminating-fixture`.

**Four rows red a superset of their column.** None is entanglement between
*rules*; each extra red is a test of the same rule under a different fixture:

- `M1` additionally reds `an_arm_specific_breakage…` (whose control arm is a
  no-marker observation) and `an_indeterminate_arm_carries…` (whose entire
  fixture is one). Both are liveness-gate tests.
- `M4` additionally reds `no_liveness_marker…`, whose third fixture is a
  `Termination` observation on a killed run — the very reading `M4` breaks.
- `M8` additionally reds `an_indeterminate_arm_is_never_proven`: reading the
  control first genuinely does let a `Failed` control outrank an indeterminate
  probe. That is arm precedence, which is what `M8` mutates.
- `M10` additionally reds `auxiliary_outcomes_do_not_reach_the_admission`, whose
  second half drives a `Violated` row through the same `all`.

Recorded rather than fixed: narrowing any of these would trade real coverage for
a tidier battery table.

### The `VA` walks (`T12`)

- **`VA-1` — invariant 6.** `git diff` on `crates/doctrine-control/src/backend.rs`
  and `backend/`: **empty**. `CapsuleBackend` gains no weakening and no
  observation surface; `ConformanceBackend` is a second trait declared in
  `conformance.rs` and implemented in this phase only by a `#[cfg(test)]` stub.
  `grep '^\s*pub [^(]'` over `conformance.rs`: **no hits** —
  `ConformanceBackend`, `PropertyRemoval`, `AuthorityGrant` and `HostPid` are all
  `pub(crate)`, as is every other item in the unit. **Verdict: holds.**
- **`VA-2` — no claim that the code checks the design document.** Grepped
  `design|document|table A` and read all eleven hits adversarially. Three sit in
  `Property`'s doc block and *are* the disclaimer: the compiler's one check is
  that `RowId::Property` keys the verdict, and membership, ordering and
  completeness against the document are named as reader discipline. One sits on
  the covering test and says so explicitly — "This says nothing about any
  document: it is a property of the two functions above it." The remainder are
  `design-faithful` (about `rustix`), `documented fallback`, `Table A. Empty
  until PHASE-09`, and one `documented` inside an unrelated doc comment. No hit
  claims a machine check that does not exist. **Verdict: holds.** `RV-346`
  `F-28`'s shape is not reproduced.
- **`VA-3` — the sealing assumption, and no `pub` escape.** `EX-17`'s assumption
  is written in the module header under its own subheading, at the vocabulary's
  declaration site, and states the condition (every backend lives in this crate)
  and what would falsify it (a backend shipping from outside, which makes the
  vocabulary public API and needs a newtype seal). `mod conformance;` is private
  in `main.rs`, the crate is bin-only, and the `pub ` grep above returns nothing,
  so no row, delta or payload landed here is reachable from outside the crate.
  **Verdict: holds.**

### Notes for PHASE-08

- The four item-level `dead_code` expectations are **hard errors** once you land
  a reader — `unfulfilled_lint_expectations` is denied. Delete `ArmShape`'s,
  `Delta`'s and `Row`'s as `run_row` starts reading them; do not batch it.
- `run_row` currently returns `Indeterminate { arm: Which::Probe, detail:
  BackendError("the executing harness lands in SL-248 PHASE-08") }` with
  underscore parameters. It is unreachable because `tables()` is empty.
- `Fixture` is a bare unit struct with no fields, constructor or `Drop`, per
  `D4`. It is already *live* (named in `Delta::Widened`'s signature), so it needs
  no suppression to be given a body.
- The battery driver at `/tmp/sl248-battery/battery.py` is session-local and will
  not survive; the mutation texts are in the table above.
