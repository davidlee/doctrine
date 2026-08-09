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

---

## PHASE-08 — the executed harness (execution record)

Written as the phase runs, per the sheet's *harvest as you go*. Each entry is
finished work; a task with no entry here is not ticked.

### Measurements taken before any code (`R7`, `S7`)

`bwrap` 0.11.2 on this host. Both untried argv shapes hold, so `S7` does not
fire — the detail is in the sheet's § *Findings (execution)* `F-7`.

| shape | measured |
|---|---|
| `--cap-add ALL` | accepted **and** effective: `CapEff` `0000000000000000` → `000001ffffffffff` |
| `--unshare-user-try --unshare-ipc --unshare-net --unshare-uts --unshare-cgroup-try` | accepted; `/proc` entry count 20 against 4 under `--unshare-all` |
| the enumerated set **plus** `--share-net` | accepted, and the payload sees the host's interface count — so `D5`'s "combines only with `--unshare-all`" is not what 0.11.2 enforces (`F-8`). `D5`'s omit-`--unshare-net` route is still the one taken |
| `--die-with-parent` vs the pid namespace | `--die-with-parent` is what reaps an escaping detached grandchild; removing `ProcessVisibility` alone does not let one escape (`F-9`) |

PHASE-09 inherits these rather than re-running them.

### `T1` — `TempRoot`, and `EX-2` made checkable

`conformance.rs`, new section *The fixture's root: real disk, never tmpfs*.

- `TempRoot::new(&dyn HostFacts)` walks `D1`'s three candidates —
  `$XDG_DATA_HOME`, `$HOME/.local/share`, then the enclosing repository's `.git`
  — each suffixed `doctrine-conformance/`, and creates a `<pid>-<nonce>`
  directory in the first that serves. `FixtureFault::NoRealDiskRoot` carries the
  candidates actually tried.
- **`EX-2` is a check, not a comment**: `prepare_root` refuses a base whose
  `rustix::fs::statfs` `f_type` is `TMPFS_MAGIC` (`0x0102_1994`), spelled locally
  because `rustix` exports `StatFs::f_type` but no magic constants and `libc` is
  not a dependency (`S4`). `FsWord` is the field's own type, so no cast — the
  `as_conversions` deny is untouched.
- A failed `statfs` reads as **not** real disk. Answering "fine" to an
  unanswerable probe is precisely the `M20` failure.
- Three rejection reasons are deliberately not distinguished (base uncreatable,
  base on tmpfs, run root uncreatable). All three mean *next candidate*, and the
  third doubles as the writability check — `create_dir_all` on an existing
  unwritable directory succeeds, so only creating something proves usability.
- Nonce is `std::process::id()` plus a process-static `AtomicU32`. **No clock**:
  `HostFacts` carries none by design, and two fixtures built in the same
  millisecond would collide on one.
- `enclosing_git_directory()` reads `std::env::current_dir()` directly rather
  than widening `HostFacts` — four methods is the contract (`sec-5`), and moving
  a production trait for a scratch-path fallback is the wrong trade.
- `Drop` is `remove_dir_all`, best-effort and silent (a failing `Drop` would have
  to `panic`, which is denied). Nothing public on the type removes anything —
  invariant 8 holds and this phase mints no capsule-delete capability.
- Tests: `the_fixture_root_is_on_a_non_tmpfs_filesystem` (independent `statfs` on
  the chosen root; plus, guarded on this host's `/tmp` actually being tmpfs, that
  `prepare_root` **refuses** `std::env::temp_dir()`; plus two roots in one
  process differ) and `the_fixture_root_is_removed_when_the_fixture_is_dropped`
  (root populated with a nested directory **and** a file first, so a
  non-recursive removal reds here instead of passing on an empty directory).

### `T2` — the fixture: a self-contained control plane

Thirteen fields minus one: exactly the twelve `design.md:4056-4093` names, in
that order, with the design's doc comments carried over verbatim. A
`readable_roots` cache was written and then removed — the sheet says *exactly*
those fields, and a delta that wants the declared roots can re-read the
document the fixture already wrote.

- **Layout.** `root/{project,capsules,decoys}`. `own_export` is
  `capsule_root/export` via `backend::EXPORT_DIRECTORY_LEAF`, so the fixture and
  `CapsulePlacement::check_source_export` agree on the leaf by construction
  rather than by a matching literal.
- **`scopes` names `project_root`, its `.doctrine/`, and `capsule_root`** — the
  fixture's own repository stands in for the operator's, which is named by no
  arm. `decoy_credential` and `decoy_repository` are deliberately *not* members,
  and a test asserts that: if they were, row 3's control arm would be an
  unlawful widening and the row unrunnable.
- **`readable-roots` are derived from the host, never hardcoded.** One rule: the
  **top-level** ancestor of the resolved `/bin/sh` and of every resolved `PATH`
  entry — `/nix/store/…/bin` yields `/nix`. The top level and not the entry,
  because a dynamically linked executable needs its loader and libraries, which
  on a store-based host live under sibling directories of the same top level; a
  measurement (pre-`T2`) showed `git` runs under `--ro-bind /nix/store` and
  cannot under its own `bin` alone.
- Any candidate whose top level **contains operator state** is dropped: the
  fixture root, `$HOME`, and the cwd. That is what keeps `/home` — the top level
  of a `PATH` entry on most hosts — from binding the operator's credentials into
  a capsule (invariant 7, `VA-2`). The discriminating fixture is a `PATH`
  spanning *both* a lawful top level and the operator's; a `PATH` of store paths
  alone passes under an implementation with no exclusion at all.
- Top-level ancestors are pairwise non-overlapping by construction, which
  satisfies `check_inner_destinations`' collision rule with no deduplication
  pass beyond `contains`.
- **The synthesized document is proved by parsing it with
  `config::parse_capsule_config`**, not by eyeballing the format string. Shape
  copied from `provision.rs`'s own test `document()` helper, including the
  `[interpretation]` block `resolved_policy` reads back from the base blob. No
  `closure-roots`: declaring them would need a resolver, and the resolver is
  itself admitted against the policy — a second moving part in a fixture whose
  job is to be boring.
- **The repository holds an object the base cannot reach, deliberately.** Commit
  `base` on the initial branch; `switch -c unreachable-from-base`; a second
  commit; `switch --detach <base oid>`. Without it
  `the_clones_object_set_is_exactly_the_exports` is vacuous — it would pass
  under a clone that copied everything. Asserted by comparing
  `rev-list --objects <base>` against `rev-list --objects --all`, and by
  checking `HEAD == base` so `provision`'s working-tree read of `[capsule]` sees
  the document the base commits.
- Git identity is pinned via `git config`, for the reason `provision` pins the
  capsule's: an unset identity makes Git resolve the hostname, and inside an
  unshared UTS namespace that is a multi-second DNS stall.
- `GIT` is spelled locally: `provision.rs`'s constant is private to that module
  and `provision.rs` is not this phase's file (`S1`).
- `std::fs::write` is banned by `clippy.toml`, so `write_file` is
  `File::create` + `write_all`.

**`second_filesystem` — three conditions, and the mount table is a parameter.**

- Selected by parsing `/proc/self/mountinfo` (field 4, octal-escaped), never by
  naming `/tmp` — hardcoding a mount is `M18`'s mutation.
- Three conditions, each ruling out a different way of picking wrong: a
  different `st_dev`; a non-zero available figure (`/proc`, `/sys` report
  nothing); and a figure that **differs from the capsule root's**, so *the two
  figures differ* is established at selection rather than asserted hopefully at
  read time.
- Every probe is a raw `rustix::fs::statvfs`, never `HostFacts::available_bytes`
  — the selection must not route through the function `M16`/`M17` mutate.
- The escape decoder is its own tested function. A mount under a directory with
  a space is the discriminating case: splitting on whitespace without decoding
  truncates it to a **prefix that still `stat`s**, so the wrong filesystem is
  selected silently.
- **The mount table is an argument, not a read.** `A2` says this host always
  takes the `Some` branch, so the `None` branch — the one that makes Table C's
  conditional row report *skipped* — would ship untested. Two forced tables now
  cover it: empty (one filesystem) and pseudo-filesystems only.

**tmpfs is deliberately not excluded from the second-filesystem selection.**
Excluding it was written, measured, and reverted: inside this jail it flips the
answer from `Some("/")` to `None`, which would skip the conditional capacity row
on the very host the suite is developed against. `DEC-156` bans tmpfs for the
*fixture root*, where a resource observation would measure the mount instead of
the disk; this row's claim is only that two paths on two filesystems yield two
figures, for which a tmpfs mount is a perfectly good second filesystem. Recorded
because it reads like an oversight and is not.

- On this host the selection answers `/`. The test says so out loud
  (`eprintln`) on both branches, so a vacuous pass is visible in the run output
  rather than indistinguishable from a real one.

**`FixtureFault` widened** from `T1`'s single `NoRealDiskRoot` to add `Io`,
`Git` and `NoReadableRoots` — a refusal rather than a panic throughout, because
`verify` is a production path (`backend verify`, PHASE-10) and a host that
cannot supply a Git or a loopback socket is a fact about the host, reported the
way `NotAdmitted::Unavailable` reports a missing shell.

**Owed to `T9`.** `EX-5`'s skip branch is now reachable in the *selection*; the
row-level skip (Table C reporting `AuxOutcome::Skipped` naming the reason) still
needs its own forcing at `T9` — build the row over an explicit `None` rather
than over whatever this host happens to mount.

Tests (10): `a_mount_point_is_decoded_rather_than_truncated`,
`a_malformed_mount_escape_is_passed_through`,
`a_readable_root_is_the_top_level_ancestor_of_an_entry`,
`no_readable_root_contains_the_operators_home`,
`no_readable_root_contains_the_fixture_root`,
`the_synthesized_table_is_the_one_provision_parses`,
`every_artefact_the_fixture_builds_lies_beneath_its_own_root`,
`the_second_filesystem_is_on_another_device_or_absent`,
`a_host_with_no_second_filesystem_selects_none`,
`the_fixture_repository_holds_an_object_unreachable_from_the_base`.

### `T3` — the weakening seam in `bubblewrap.rs`

**An enum, not a bag of eleven booleans.** `Weakening` has one variant per axis
(ten removals plus `AllCapabilities`), so a weakened run differs from the
confining profile along *exactly one* axis **structurally** — a type that cannot
express two at once beats a comment saying it should not. There is no
`Weakening::None`: absence is `Option`'s job, and `WeakenedProfile::confining()`
is the profile with nothing selected.

- `WeakenedProfile { weakening: Option<Weakening>, observer: Option<&dyn Fn(i32)> }`.
  The observer is **orthogonal** to the weakening — a probe arm observes an
  otherwise fully confining run, so it cannot be a twelfth variant.
- `CapsuleBackend::execute` is now literally
  `self.run(placement, execution, &WeakenedProfile::confining())`. One assembly
  of the profile in the tree; `conformance.rs` maps its own `PropertyRemoval`
  onto `Weakening` and the ADR-001 edge stays conformance→backend (`D2`).
- `Weakening` names **no `conformance` type** and carries only primitives and
  `OwnedFd`s. `StdioOwned` is the only variant with a payload (`D4`) — three
  `OwnedFd`s, duplicated with `try_clone` at spawn rather than consumed, since
  the caller keeps the other end and the profile may run more than once.

**`D5`'s network trap is handled inside `confinement_argv`, not by the caller.**
`--share-net` is bubblewrap's only re-share flag and pairs with `--unshare-all`.
Under `ProcessVisibility`'s enumerated set there is no `--unshare-all` to
re-share against, so a permitted network is expressed by **filtering
`--unshare-net` out of the set** and emitting no `--share-net` at all. Emitting
both is an argv error — a control the mechanism refuses to build, which is `S7`
in miniature. `the_enumerated_unshare_set_expresses_a_permitted_network_by_omission`
holds it, and the discriminating fixture is the **pair**: the permitted
placement alone passes under an implementation that drops `--unshare-net`
unconditionally, the denied one alone passes under one that never drops it.

**Order is unchanged under every axis.** The confining assembly was restructured
from one `vec![…]` literal into conditional pushes, and
`argv_is_assembled_in_the_declared_order` — which asserts the whole token vector
byte for byte — still passes unedited. That test is the behaviour-preservation
proof for the restructure, and it was already there.

- The descriptor sweep still runs **before** the status fd is cleared of
  `CLOEXEC`, including under `Weakening::Descriptors`, which skips the sweep but
  not the clear. The sheet calls that order load-bearing and it does not move.
- `--setenv` stays byte-identical under `EnvironmentCleared`, which drops only
  `--clearenv` (`VA-4`).
- `--cap-add ALL` is appended after the environment and before the payload argv,
  so every other flag keeps its position.

**The observer seam is plumbed but its pid is `T5`'s.** `spawn_and_wait` is
`Command::output()` when no observer is set, and `spawn` + callback +
`wait_with_output` when one is — `wait_with_output` is what `output()` does
internally, so the piped stdout stays drained rather than deadlocking against a
capsule that fills the pipe. **The pid handed over today is the immediate
child's**, which under the wall bound is `timeout(1)`, not the capsule's
top-level process. `T5` replaces it with the capsule's own; the seam is here,
the `/proc` descent is not. Said out loud in the function's doc comment so it
cannot be mistaken for finished.

**Two staged `#[expect(dead_code)]` sites added, both removed at `T4`.** The
seam is in `backend` and its only consumer is the mapping in `conformance`, and
`D2` forbids the edge running the other way — so the two land one task apart and
`unused = deny` collapses the gap into a hard error. Item level, never field
level (`mem.pattern.lint.expect-dead-code-at-item-level`). `R6`'s count is now
four staged sites carried in from PHASE-07 **plus** these two; all six must be
gone by `T12`.

## PHASE-08 `T4` — `impl ConformanceBackend for BubblewrapBackend`

**Where it lives, and why.** `conformance.rs`, per `D2` and ADR-001: `backend`
is a leaf and `conformance` is engine with an out-edge to it, so the mapping
from the property-shaped vocabulary onto the flag-shaped one may only sit on the
`conformance` side. Every method is `BubblewrapBackend::run` under a different
`WeakenedProfile` — the same code the production path runs — which is the whole
of what `D2` buys: there is no second implementation of the confinement profile
to drift from this one. `run` and `confinement_argv` became `pub(crate)` for it;
nothing else in `bubblewrap.rs` widened.

**The mapping is one exhaustive `match`.** `weakening_for(removal, stdio)`, nine
arms, ten removals — `ResourceBound(Bound::FileSize|Wall)` is where the tenth
lives. Widening `PropertyRemoval` fails to compile here, which is the reason the
vocabulary is an enum and not a list of names. Two name changes to bridge:
`DescriptorsClosed` → `Descriptors`, `EnvCleared` → `EnvironmentCleared`.

**`EX-18`'s split is recorded above the mapping, and honestly.** Three deltas
measured — `Teardown` by `EVD-013`, `MappedIdentity` and
`Granted(AllCapabilities)` by `EVD-014`. `EVD-013`'s adjacent fact that
bubblewrap has no `--share-pid` tells us how `ProcessVisibility` must be
*expressed*, which is not the same as having seen it produce its row's control
failure. **Every other delta is reasoned and cites no measurement.** The comment
says exactly that and no more; a caption claiming all ten were measured is what
`EX-18` exists to correct.

**The grant needed a third trait method** (`F-13`). `design.md:2960`'s sketch has
two, and `Delta::Granted` has no way to run under either — folding it into
`PropertyRemoval` is what `sec-7`'s ruling and `F-31` forbid. `execute_granted`
mirrors `execute_weakened`; `weakening_granting` is its one-arm mapping, named
so the test and the run read the same function. Its cost is a second fails-closed
case: `a_backend_ignoring_its_grant_yields_unproven`, because a backend honest in
`execute_weakened` and lazy in `execute_granted` passes the removal test and
proves nothing about row 14.

**Row 12's descriptors are two socket pairs, not a file** (`F-14`).
`execute_weakened` sees no fixture and `S5` forbids both channels that would
carry one, so `OwnedStdio::opened()` builds them: descriptor 0 is the read end of
a pair the trusted side wrote a decoy body into and closed (so the capsule reads
real bytes and then sees EOF — a capsule blocking forever on descriptor 0 would
be a containment failure of this function's own making); descriptors 1 and 2 are
the write end of a second pair. `UnixStream::pair`, not `std::io::pipe`, which is
1.87 against an MSRV of 1.85 (`C7`).

**The capture must be drained after the profile is dropped.** Under
`StdioOwned` the child's stdout is not piped, so `Command::output()` returns an
empty `stdout` and `Observation.stdout` has to be filled from this side's end.
A socket pair reports end-of-file only when its peer is *fully* closed, and
`Weakening::StdioOwned` owns the write ends — so `drop(profile)` before
`read_to_end`, or the read blocks forever. Written as an explicit `drop` with the
reason at the site rather than left to scope order.

**`SpawnOptions` is the phase's one design improvement** (`F-16`). Four of the
eleven axes change no argv word at all — the two resource bounds, the descriptor
sweep, owned stdio — so an argv diff reads them as "changed nothing", which is
indistinguishable from a delta nobody implemented. `SpawnOptions::under` names
the four as data; `run` reads it in place of three open-coded `matches!` guards,
and `each_removal_changes_exactly_its_own_flags` compares it. One table, two
readers, no drift.

**`each_removal_changes_exactly_its_own_flags` is a multiset diff.** Eleven
cases, each stating the words its axis removes and adds against the confining
baseline plus the spawn options it switches off. Multiset rather than positional
because `argv_is_assembled_in_the_declared_order` already holds the order byte
for byte and restating it eleven times would make every case brittle to an
unrelated insertion. Two fixture decisions that carry the discrimination:

- the environment passed to `confinement_argv` is **non-empty**. `M5` predicts
  that dropping the `--setenv` list reds the environment case; against an empty
  list, dropping it is a no-op and `M5` would red nothing.
- a sibling test, `every_axis_leaves_the_setenv_list_byte_identical`, walks all
  eleven axes and compares the `(name, value)` pairs in order — the multiset
  cannot see a *reordering*, and `VA-4` names the `--setenv` list specifically.

**`M1` is predicted not to red, in advance** (`F-15`). `execute_weakened`
returns an `Observation` and nothing else, so no static test can observe which
profile it chose; a backend that ignores its removal is caught by the
fails-closed pair instead, which is where `EX-7` puts it. `M2`…`M10` all bite on
the flags test, because each is a change to a delta rather than a bypass of the
mapping. Recorded now so `T11` records a prediction rather than a discovery.

**`R6` is back to four.** Both `bubblewrap.rs` staged suppressions are gone —
`Weakening` and the `WeakenedProfile` impl both have live consumers now. The
four carried in from PHASE-07 (`ArmShape`, `Delta`, `Row`, `RowId`) remain and
must all be gone by `T12`.

**Still owed by `T5`.** `execute_observed` is plumbed and calls back exactly
once, but with the immediate child's pid — `timeout(1)` under the wall bound.
The `/proc` descent to the capsule's top-level process and the session id (`D3`)
are `T5`'s, and the obligation is written at the call site so it cannot be
mistaken for finished.

## `T5` — `execute_observed`, the descent, and the session id (`EX-7`, `D3`)

**The measurement first.** Under the confining profile the tree the trusted side
faces is three deep, and only the last one is the subject:

```
reader sid 1310932
1310936 ppid=1310932 pgrp=1310936 sid=1310932   timeout -k 2 10 bwrap …
1310938 ppid=1310936 pgrp=1310936 sid=1310932   bwrap …
1310939 ppid=1310938 pgrp=1310939 sid=1310939   bwrap …   ← the capsule
```

`spawn_and_wait` hands over the **first** of those, which is `timeout(1)`. So the
seam PHASE-07 left is a wrapper pid, and every row that reasons about the
subject's liveness would have been reasoning about the wall-bound wrapper.

**The rule that names the subject: nearest session leader below the child.**
`--new-session` calls `setsid` in exactly the top-level sandbox process, and a
host-namespace reader sees that session as the leader's **host** pid — so
`session == pid` identifies the subject, and the number it yields is
simultaneously the pid `EX-7` wants and the sid `EX-12`'s sweep wants. `D3`
assumed the pid seam could supply the sid; the measurement is why it can, and
the reason is stronger than the assumption — it is the same integer.

The rule is a property, not a depth, which is what makes it survive the
weakenings: `Weakening::WallBound` deletes the `timeout` layer and
`Weakening::ProcessVisibility` deletes the pid namespace, and neither moves the
answer. A wrapper count would have broken on both.

**Three ways to get it wrong, each with a fixture.**

- *First match rather than nearest.* Row 7's payload deliberately `setsid`s a
  descendant, so a later arm has two leaders under the same child and the deeper
  one is the escapee. `/proc`'s readdir order is neither numeric nor stable, so
  `an_escaping_orphan_is_not_mistaken_for_the_capsule` lists the escapee
  **first** — otherwise a first-match implementation passes by luck. Ties break
  on the lower pid so the answer is a function of the table, not of readdir.
- *Member rather than leader.* Once the top-level process exits, its children
  keep its sid and none of them is the leader.
  `a_survivor_of_a_departed_leader_is_not_the_capsule` pins the honest answer:
  `None`. Naming a survivor would hand the row a pid whose liveness says nothing
  about the subject.
- *Field index off by one.* `comm` is the payload's own `argv[0]` basename,
  unescaped, and may hold spaces and parentheses — so the split is on the **last**
  `)`, never on whitespace from the left. After it, field 1 is ppid, field 2 is
  **pgrp**, field 3 is sid. `stat_is_read_past_the_last_paren_of_a_hostile_comm`
  uses a `comm` of `evil ) 9 9 9 9` and three distinct numbers, so both mistakes
  red separately. `SessionId` is a newtype for the same reason: `M13`'s
  process-group-instead-of-session mutation needs the two to be distinguishable
  at the point where confusing them is cheap.

**The guard that could never fire, removed** (`F-18`). The first draft filtered
`session != own_session()` — "a foreign session". Probed with an ad-hoc mutation
before `T11` rather than after: it redded **nothing**, and the reason is
structural, not a fixture gap. A process leading the harness's own session
predates the child the harness just spawned, so it can never be that child's
descendant; the descent already excludes it. Deleted, with the reasoning at the
site so it is not reinstated as an obvious omission. `own_session()` stays — it
is `F-3`'s "record what you *can* record beforehand", and `EX-12`'s sweep is its
real caller.

**Six mutations run by hand at green, before `T11`:** first-match instead of
nearest (reds the escaping-orphan case); drop the subtree restriction (reds the
foreign-session case — its pid is deliberately **lower** than the capsule's so
the depth tie-break cannot rescue it); drop the leader predicate (reds six);
weaken the leader predicate to membership (reds the departed-leader case);
`STAT_SESSION_FIELD` 3 → 2 (reds the hostile-`comm` case); own-session exclusion
(reds nothing — see above). Restored by copy each time.

**Silence is the honest failure.** `observed_capsule_process` polls for half a
second (250 × 2ms) and, failing, does **not** call `observer`.
`classify_concurrent` reads that as `Indeterminacy::NoLiveness`. Calling back
with the wrapper's pid would be worse than silence: it is a pid that is always
alive, so every liveness probe would pass and the concurrency row would be a
rubber stamp.

`depth_from` is bounded by the table's own length. `/proc` is read entry by
entry and can tear between them, so a parent chain that closes on itself is
reachable; `a_torn_parent_cycle_terminates` is that fixture.

**Live coverage is owed, and where.** Everything above is unit-tested against
process tables as data, which is the only way to get the shapes this host will
not produce on demand (`A2`'s lesson generalised). The live half — a real
capsule, a real callback, a real sid — arrives with `T6`'s arms and `T10`'s
`the_orphan_left_by_a_teardown_or_visibility_control_is_reaped_by_the_harness`,
which needs both halves of the same machinery. Recorded here so `T5`'s tick is
not read as "the descent has run against a real tree".

Durable: `mem.fact.linux.session-leader-identifies-the-sandbox-subject`.

## T6 — the three arm shapes, and what an arm is read from (EX-6, EX-7, S8)

**The unit of reading is the arm, not the capsule.** `Under` — `Confining` /
`Removing(PropertyRemoval)` / `Granting(AuthorityGrant)` — applies to *one*
capsule in every shape: the sole capsule of `Single`, the **reader** of
`Sequential`, the **observer** of `Concurrent`. The other capsule always runs
confined, because a control that changed the scene rather than the reading
would not be a control (invariant 3).

That is the answer to what looked at PHASE-07 like a trait gap. Row B5's
control removes process visibility from the *observer*, which is an ordinary
weakened capsule reached through `execute_weakened`. The subject is the same
confined capsule on both arms. So `execute_observed` needs no removal
parameter, and PHASE-07's three-method seam is complete as shipped (`F-19`).

**The choreography deviates from the sheet, and is stronger for it.** The sheet
prescribed a thread: subject on its own thread through `execute_observed`, pid
published to the main thread, observer run from there. The trait already
defines its callback as the interval between the subject's top-level process
existing and the trusted side waiting on it — so the observer capsule runs
*inside* the callback. No threads, no `Send`/`Sync` bound on
`ConformanceBackend`, and the window is guaranteed by construction rather than
raced for. Cost, recorded so it is not rediscovered: the subject's stdout is
unread for the duration of the callback (`wait_with_output` follows), so a
subject payload that filled the 64 KiB pipe would block there. It would block
*alive*, which widens the window rather than closing it, and every table-B
subject prints one marker line.

**The window closes when the observer finishes, not when it starts.**
`subject_live_when_observer_ran` is sampled after the observer capsule returns.
Sampling it before would credit an observer that outlived its subject — one
that looked at a process which was there at launch and gone by the time it
looked. `the_window_closes_when_the_observer_finishes_not_when_it_starts` is
the only test that reds when the two lines are swapped, and it needs a `live`
closure that changes its answer across the observer's run, which is why `live`
is injected on `Arm` rather than being `capsule_still_running` directly.

**`S8` is answered: the two concurrent indeterminacies are distinct, and both
are reachable from a test.** The subject exiting before the observer ran is
`NoObservation`; the backend returning without ever calling back is
`NoLiveness`. Different repairs — a fixture whose subject is too short-lived
versus a backend that did not implement the seam — so collapsing them would
send the reader to the wrong half of the harness. A third case shares
`NoLiveness`: a subject that never printed its liveness marker at all. That one
is the dangerous one, because an observer that finds no live process is exactly
what row B5's *probe* arm expects to see, so without the early return a subject
that never ran would read as a **held** probe. `M?`-class mutation confirmed:
deleting the early return reds
`a_concurrent_arm_whose_subject_never_ran_reports_no_liveness` and nothing else.

**A writer that did not write establishes nothing.** `Sequential`'s writer
failing is `Indeterminate{NoObservation}`, never `Failed`, and the reader does
**not** run — its observation would be of a state nobody staged. Two mutations
red `a_sequential_arm_whose_writer_did_not_write_establishes_nothing`: mapping
the writer's failure to `Failed`, and falling through to the reader anyway.

**`EX-6` is asserted by counting, not by inspection.** The test `Stub` grew a
`reached: RefCell<Vec<Under>>` entry-point recorder, and the arm's capsule
closure is a counter. `a_sequential_arm_provisions_one_transaction_per_capsule`
asserts two provisionings and two executions; the weakening tests assert the
exact sequence — `[Confining, Removing]` for `Sequential`, and
`[Removing, Confining]` for `Concurrent`, observer first because it runs inside
the subject's callback.

**Eight mutations run by hand at green, before `T11`,** restored by copy each
time: single arm loses its weakening (reds the grant test); sequential reader
loses its weakening, and sequential writer gains one (both red the sequential
control test); writer-failure → `Failed`, and writer-failure → run the reader
anyway (both red the establishes-nothing test); observer runs confined (reds
the concurrent control test); the `NoLiveness` early return deleted (reds the
never-ran test); liveness sampled before the observer (reds the window test);
the observed pid never recorded (reds four). None redded nothing.

**Fixtures read in execute order, which is not arm order.** For a `Concurrent`
arm the observer's scripted observation is consumed *first*. Anyone extending
these tests who scripts them subject-first will get a green test measuring the
wrong capsule.

**`ArmShape`'s staged `dead_code` suppression is gone** — its own reason said it
self-clears when the harness reads a payload, and it did. `R6`'s count is now
three, all in `conformance.rs`: `Delta`, `Row`, `RowId`, all owed to `T7`.

## T7 — delta application and the real `run_row` (EX-10)

**The fixture is built lazily, and the laziness is invariant 1's ordering, not
an optimisation.** `verify_over` reports availability and the missing shell
*before* any row runs. A fixture built eagerly in `verify` would put a
git-and-disk build ahead of both, so a host with no `bwrap` would be told about
its disk. A `OnceCell` in `verify` builds it when the first row asks; a host
that cannot build one gets every row `Indeterminate` naming the fault, which is
`EX-11`'s rule for a mechanism that failed rather than a property that did.
`verify_over`'s signature is untouched, so the algebra tests still run against
hand-built rows and a counting runner.

**`run_row` gained the fixture and the host as parameters.** `D6` already
prescribed `auxiliary_claims(fixture, backend)`, so the fixture as a parameter
is the sheet's own shape; the alternative — a fixture per row — would rebuild
`sec-3`'s export for every one of them and contradicts the sheet's own cost
note.

**Trait upcasting is not available at this MSRV.** `provision` takes `&dyn
CapsuleBackend`; `verify` holds `&dyn ConformanceBackend`. Coercing one to the
other is *trait upcasting*, stable from Rust 1.86, and the workspace floor is
1.85. `clippy::incompatible_msrv` catches std **APIs** below the floor, not
language features — so the upcast would compile on the developer's 1.98
toolchain and fail only on the oldest toolchain the crate claims. Hence
`ConformanceBackend::as_capsule_backend`, three lines per impl and two impls
exist. This generalises: an MSRV floor is only enforced for library calls, and
language-feature regressions are silent.

**Both arms run unconditionally, probe first.** `row_verdict` needs both
readings, and short-circuiting on a failed probe would make `Violated` cheaper
to reach than `Proven` — the wrong asymmetry for a suite whose green path must
be the expensive one.

**The probe arm never rebuilds.** Its capsule closure returns
`transaction.placement` and nothing else touches it (invariant 4). The control
arm's closure is the only one that calls `placed_under`.

**`SharedRoot` keeps the root, not the transaction.** A `RefCell<Option<
TransactionRoot>>` spans the control arm's two capsules; the second placement is
rebuilt on the first's root. Both transactions are provisioned normally — a
second provision *into* the first's root is what `sec-3` step 9 refuses, and a
control the system refuses to build proves nothing. Nothing in the crate removes
a transaction root, so the transactions themselves are dropped and the fixture's
own `Drop` reclaims the whole capsule root (invariant 8).

**Every rebuild goes back through `CapsulePlacement::try_new`, and that is
evidence.** The widened control passes the same validating constructor the
probe's placement passed, so a row that proves a property cannot be dismissed as
having proved that its control was malformed. A refusal is reported as the
mechanism failing.

**`accepted_base` cannot be read back off a placement** — `try_new` checks it
against the source export and discards it — so `parts_of` takes it from the
fixture. Every transaction in a run contracts the same base, so this is exact
rather than approximate, but a future fixture with two bases would break it
silently. Written here because the type gives no warning.

**Transaction ids are pid plus a monotonic counter, no clock.** Two capsules in
one arm are two transactions and step 9 creates each root exclusively, so a
repeated id refuses the second capsule of every two-capsule row — and it would
read as a mechanism failure on a perfectly good backend.
`every_transaction_is_allocated_its_own_id` is the guard.

**What `T7` did *not* test, and where it is owed.** What each delta does to a
placement needs two real provisioned transactions to discriminate, and the sheet
already mandates exactly those two tests at `T10`:
`a_probe_arm_placement_is_byte_identical_to_what_provision_returned` and
`the_shared_root_delta_repoints_only_the_second_placement`. `T7` tested the
routing (`only_the_two_backend_side_deltas_reach_the_profile`), the identity
allocation, and the bound sourcing. Recorded so `T7`'s tick is not read as
"`placed_under` has been exercised".

**Two more staged suppressions discharged** — `Delta` and `Row`, both of whose
reasons said they self-clear when the harness applies a delta and reads a row.
`R6`'s count is now **one**: `RowId`, which needs PHASE-09's `Property` variants
to inhabit it.

## T8a — the session sweep (EX-12, D3)

**By session, never by process group, and the reason is the payload.** `RV-346`
`F-27` strengthened row 7's payload to a descendant that leaves the original
process group precisely so a process-group-only backend cannot pass. A
process-group kill in the harness therefore could not reap the survivor its own
control arm creates. The general rule, worth carrying past this slice: **when a
row's payload is strengthened, its containment is part of the payload.**

**The fixture records its own sid at build; the capsule's arrives later.**
`EX-12` asks the fixture to record the capsule's sid *before the arm runs* and
it cannot — the session does not exist until bwrap creates it inside the child
(`F-3`). What it can record beforehand is its own, and that is what makes a
foreign session identifiable afterwards.

**The own-session refusal is at record time, not at signal time.** Skipping the
harness's own session when signalling would be enough to be *safe*; refusing it
when recording is what makes the refusal *observable*, because a sweep that
signalled nothing and a sweep that recorded nothing look identical from outside.
`the_harness_never_records_its_own_session_as_a_capsules` reads the harness's own
session leader pid and asserts the swept set stays empty.

**`own` is a parameter of `kill_session`, not a global it reads.** There is no
route to the signalling loop that has not already had to name the session it
must not touch. A `None` own-session — a host whose `/proc` did not answer —
sweeps nothing at all: a sweep that cannot tell its own session from a capsule's
is a sweep that must not fire.

**Sweeping drains.** It is called after every row *and* from `Fixture::drop`,
and the second call must not signal a pid the kernel has since recycled. The
drain makes the second call a no-op by construction rather than by a flag.

**Testing a sweep without killing the machine.** The dedup-and-drain test drives
a session id **above every live one** (`max(sid) + 1`), so the sweep runs its
whole real path — enumerate `/proc`, match, signal — and finds no member. A test
that named a session with members would be a test that `SIGKILL`s this machine.
The recording rule was split out as `note_session(SessionId)` for exactly this:
the pid→session read needs a live process, the refusal-and-dedup rule needs a
session with no members, and one function could not be driven by both.

**`Arm.noticed` is a second sink, not a widened `live`.** They answer different
questions at different times: `live` is read once, after the observer has run,
and decides whether row B5's window held; `noticed` fires the moment the pid
exists and decides what the sweep must reach on the way out. Folding them would
tie containment to a row *shape* — and the arm whose containment matters most,
row 7's, is a `Single`.

**`Fixture::drop` sweeps before `TempRoot` removes the tree**, because field
drops follow the type's own `Drop`. It kills processes and removes nothing:
invariant 8's no-delete rule is about capsules on disk, and cleanup there is
still `TempRoot`'s alone.

## T8b — row 10's inheritable decoys (`EX-13`)

**A removal is only a removal if there is something to remove.**
`PropertyRemoval::DescriptorsClosed` skips the backend's parent-side sweep. A
sweep that had nothing to close is a removal that changes nothing, so the row
needs descriptors that are *already* inheritable in the trusted process when the
arm spawns. Rust opens its own files `O_CLOEXEC` — the trap PHASE-05 `VT-4`
records — so `File::open` would have handed row 10 three descriptors the sweep
never had to touch, and the row would have passed with the mechanism inert.
Hence `rustix::fs::open` **without** `OFlags::CLOEXEC` for the two files, and
`make_inheritable` (the inverse of the backend's sweep) for the socket end
`UnixStream::pair` opens closed.

**The decoy set is per-arm state, not fixture state** (`F-26`). The backend's
sweep mutates the *parent's* descriptor flags, and that change is permanent. Row
10's confining probe arm therefore closes whatever set was open when it ran; a
set held once per fixture would leave the control arm nothing to leak. So
`Fixture::inheritable_decoys()` returns a fresh, owned set per call and the arm
holds it for exactly its own run. The property is a test —
`a_decoy_set_opened_after_a_sweep_is_inheritable_again` — because nothing else
would notice a memoising refactor until row 10 quietly stopped discriminating.

**`O_TMPFILE`, not create-then-unlink.** The write-only decoy must be
"unreachable by name and dies with the descriptor". `O_TMPFILE` names the
*directory* the blocks live in — the fixture's own root — and hands back the
only handle there will ever be. So invariant 7 still holds for a file with no
name, and the phase introduces no unlink, which keeps `VA-3`'s hit list empty of
anything outside `TempRoot::drop`. Create-then-`remove_file` would have reached
the same state and put a delete primitive in the crate to get there.

**The namelessness is asserted two ways, and the write is asserted at all.**
`/proc/self/fd/<n>` readlinks to `<fixture root>/#<inode> (deleted)`, which
proves both the origin and the missing link; and the fixture root's entry
listing is identical before and after. Without the write assertion the test
passes against a decoy that could not be written, which is the default outcome
of a dozen ways to get the open flags wrong.

**"No descriptor the trusted side holds for real is ever made inheritable"** is
an assertion, not a comment: the socket pair's retained far end *and* row 5's
listener are both checked to be close-on-exec in the same test that checks the
three decoys are not. The three decoys are invariant 12's only exception, and
that is the shape that says so.

## T9 — table C's four claims (`EX-14`, `EX-15`, `EX-16`)

**The claims are lazy, and that is what keeps the two early returns honest.**
`verify_over` takes `&dyn Fn() -> Vec<(Claim, AuxOutcome)>` rather than the
vector. Table C's claims provision real capsules; an unavailable backend and a
missing shell both return before any row runs, and an eagerly-built claim list
would have run two capsules to populate a report that says the mechanism is
absent. The two early returns carry `Vec::new()`; the closure is called once,
after the shell check, and a fixture fault inside it degrades to
`claims_skipped(...)` rather than to a lost report.

**Widening happened at `auxiliary_claims`, never at `admission`.** `admission`
still takes the row list alone. That is the structural reason a table C claim
may skip where `DEC-156` forbids one in an admission: a claim has no path to the
verdict in either direction — a failed one cannot block it and a skipped one
cannot grant it.

**The read-once claim is vacuous without its read-back.** `sec-4`'s claim is
that a capsule rewriting its own `doctrine.toml` does not move the bound policy.
A capsule that could not write at all satisfies that trivially, and "could not
write" is the default outcome of a dozen ways to get the mount wrong. So
`rewrite_policy_inside` reads the rewritten document back **trusted-side**,
through `profile_owned_host_path`, and fails the claim if the write never
landed. The substitution target is a shared constant with
`capsule_config_document` (`EMPTY_FORBIDDEN_EXECUTABLES`), so the textual
replacement cannot silently miss, and the test asserts the fixture's document
actually contains it — a rewrite with nothing to replace would "pass" against
itself.

**Set equality in both directions, over a non-empty set.** ⊇ alone (`M15`)
passes a clone that dragged extra objects in, which is the whole of `sec-3`'s
claim; and two empty sets are equal, which is what a capsule whose `git` never
ran produces. `object_sets_agree` is driven directly by the test with a strict
superset, a strict subset and two empty sets, so the comparison's *shape* is
under test rather than this host's luck. `--batch-check` is not optional: bare
`--batch-all-objects` is a fatal error.

**`F-28` — a single independent `statvfs` cannot check a live filesystem.** The
sheet prescribes the probe agreeing with an independent reading "within one
allocation unit (`f_frsize`)". Written that way the row passes in isolation and
reds under `cargo test`'s parallel harness: sibling tests in this same suite
build git fixtures on that filesystem, and the free-space figure moved 38
allocation units (152 KiB) *between two adjacent syscalls*. The fix is not a
widened tolerance — it is a **bracket**. Two independent readings, one either
side of the probe, bound what the truth can have been while the probe ran, and
the figure must land within one allocation unit of that interval. The tolerance
is unchanged; the measurement's own noise is removed. Discrimination is
untouched and was measured, not assumed: `M16`'s wrong quantity
(`f_bfree × f_bsize`, counting the reserved blocks) is ~92 GiB out on this host,
four orders of magnitude beyond any interval two adjacent readings can span —
run as a hand mutation of `host.rs`, and it redded both capacity rows.

**One statvfs arithmetic site in the crate.** `available_bytes_of` (the
mount-table selection) and `agreed_capacity` (the claim) both read through
`independent_capacity`, which is the only place `f_bavail × f_frsize` is
computed outside the probe under test. Two independent sites would have been two
places for the *independent* side to drift toward the thing it is checking.

**The skip carries the absence, and the absence is forced.** `A2` says this host
always takes the `Some` branch, so `capacity_filesystem_claim`'s `None` branch
ships untested unless a test drives it with `None` directly.
`a_missing_second_filesystem_reports_skipped_naming_the_reason` asserts the
reason names both the absence (`NO_SECOND_FILESYSTEM`) and the capsule root
there was nothing to tell it apart from. A skip with an empty reason is a silent
pass wearing a label.

**These are the crate's first tests that run a real capsule.** Every earlier
test — provision included — runs against `WitnessBackend`. The two executed
claims drive `BubblewrapBackend` over `SystemHost` and assert
`availability() == Available` first, so a host without `bwrap` fails with the
precondition named rather than with a confusing claim failure.

## T10 — the three harness invariants (`VT-2`)

**`F-29` — `Delta::SharedRoot` is a rebase, not a field swap.** Re-pointing the
second placement's `root` and leaving its entries where they were does not
produce a lawful placement: `check_declared_entry` licenses a writable entry
only when it sits under *this placement's own* transaction root, so the second
capsule's own `capsule/tmp` becomes a `ForbiddenScopeOverlap` the instant the
root moves out from under it. The delta therefore moves the entries with the
root — `rebase_onto` re-writes every writable and readable host path that was
under the old root to the same relative path under the shared one, and leaves
anything outside it alone. That is also what the row is *about*: two capsules
sharing a root means the second capsule's writable state lands in the first's
tree, which is exactly what the rebased entries express and a bare field swap
never did. The first placement of a `SharedRoot` arm passes through untouched —
it is the one whose root is shared — which the test pins alongside the rebase so
"only the second placement moves" is asserted rather than assumed.

**The widened control cannot use the project root.** `widens_the_undeclared_decoy`
mounts the fixture's *undeclared decoy*, not `project_root()`: the latter is the
canonical-repository scope, and widening onto it is refused by placement
validation before the control can exist. A control that cannot be built is not a
control.

**`F-30` — the escape is the visibility control's, not the teardown control's.**
`F-9` is right that `--die-with-parent` is what reaps a detached descendant, and
under a `Teardown` arm the grandchild does outlive its parents. It outlives them
*inside the pid namespace*, whose init process holds the harness's captured
descriptors until the namespace empties — so the arm cannot return while the
escapee is alive, and (a) has nothing to observe. Measured both ways on this
host: with the namespace, end-of-file waits out the escapee (a `sleep 120`
payload took the arm to the 120s wall bound, where `timeout` killed the whole
tree and left nothing to sweep); with `ProcessVisibility` removed, the namespace
is gone, the grandchild is an ordinary host process, and end-of-file arrives in
25ms with the escapee still there. The test's own title admits either control;
the visibility one is the one that can be observed.

**A payload that echoes and exits is gone before the descent can see it.** The
trusted side's descent to the capsule's own top-level process polls `/proc` for
half a second, and it is looking for a *session leader* — once the leader exits,
its surviving orphan keeps the session id but leads nothing, and the descent
correctly answers `None`. So the payload lingers a second after spawning the
escapee. Without the linger the arm reports no liveness, which is the honest
answer to a question asked too late, and the test reds for a reason that has
nothing to do with containment.

**`SIGKILL` is asynchronous; the `/proc` entry is not the process.** Re-reading
the table immediately after the sweep finds the survivor it just killed, still
listed until whoever inherited it waits on it. `drained_of` polls the same half
second the descent uses. A session that has not emptied by then was not swept —
which is the failure the assertion is for, and the poll does not weaken it.

**`F-31` — the parent-side descriptor window is not thread-safe.** This is the
phase's most consequential finding and it is *not* about the tests.
`BubblewrapBackend::run` mutates **process-wide** descriptor flags either side of
its spawn — `mark_inherited_descriptors_close_on_exec` marks every inherited
descriptor `CLOEXEC`, then `clear_close_on_exec` un-marks the status file — and
what reads those flags is `fork`. Two runs in flight at once corrupt each
other's handover. Measured, not theorised: a capsule was spawned with
`--json-status-fd 4` holding **no descriptor 4** (a concurrent run's sweep had
re-marked it `CLOEXEC` before this one forked) and a *different* transaction's
`bwrap-status.json` at descriptor 6 (this one forked inside that run's cleared
window). That capsule then blocked at bubblewrap's user-namespace handshake for
ever — its outer process gone, `read` on the sync eventfd never satisfied —
while holding the harness's capture pipe, so the arm reading that pipe to
end-of-file never returned. Observed as a suite that hung for 14 minutes with a
different test stuck each run.

The hazard belongs to the mechanism, not to `cargo test`: any caller that runs
two capsules at once meets it, and `ArmShape::Concurrent` (row B5) is exactly
that caller. The fix is to narrow the window to the fork itself, in production,
and it is deliberately **not** made here — a phase's evidence should be about the
phase. What is here is a `#[cfg(test)]` mutex around that window in `run`, which
keeps today's only multi-threaded caller off a defect it did not introduce. Held
for the whole of `run` rather than just the fork, so a future real concurrent arm
would serialise rather than deadlock; narrowing it is part of the production fix.
The suite went from a 14-minute hang to 197 tests in 1.58s.

## `T11` — the mutation battery (21 of 21)

Applied to green source, restored **by copy** from a scratch snapshot before and
after every run (`C6`); `git checkout` never used, and `git status` was clean of
`crates/` at the end of every batch. Each mutation's anchor text was asserted
**unique** before substitution, so a silently-missed edit cannot read as "reds
nothing".

**Method note.** The per-mutation gate is `cargo test -p doctrine-control --bin
doctrine-control`, not `doctrine check gate`. The test leg is the only leg a
semantic mutation can red, and it is ~1.6s against minutes — 24 runs at gate
price is most of a phase. The gate is run once, on restored green source, as the
phase's evidence.

| id | reds | verdict |
|---|---|---|
| `M1` | *nothing* | predicted in advance by `F-15`; `a_backend_ignoring_its_removal_yields_unproven_for_every_row` stayed green ✓ |
| `M2` | the chdir case | exact ✓ |
| `M3` | the teardown case | exact ✓ (the escape half did not red — `F-30`, below) |
| `M4` | the process-visibility case, `the_enumerated_unshare_set_expresses_a_permitted_network_by_omission`, `the_orphan…_is_reaped_by_the_harness` | must-red ✓, teardown case green ✓ |
| `M5` | the env case, `every_axis_leaves_the_setenv_list_byte_identical` | must-red ✓, stdio case green ✓ |
| `M6` | the stdio case | exact ✓; the descriptor case green ✓ |
| `M7` | the descriptor case | exact ✓; the env case green ✓ |
| `M8` | the identity case | exact ✓; the capability case green ✓ |
| `M9` | the inputs case | exact ✓ |
| `M10` | the capability case | exact ✓; the identity case green ✓ |
| `M11b` | `the_shared_root_delta_repoints_only_the_second_placement` | exact ✓ (`M11` as worded does not compile) |
| `M12` | `the_probe_arm_of_a_row_is_not_passed_through_the_delta` | redded **nothing** as worded; the fix was in the test — see below |
| `M13` | `the_orphan…_is_reaped_by_the_harness`, `stat_is_read_past_the_last_paren_of_a_hostile_comm` | must-red ✓ |
| `M14` | `rewriting_doctrine_toml_inside_a_capsule_does_not_change_the_bound_policy` | exact ✓; the object-set test green ✓ |
| `M15` | `the_clones_object_set_is_exactly_the_exports` | exact ✓ |
| `M16` | both capacity rows | must-red ✓; read-once and object-set green ✓ |
| `M17` | `the_capacity_probe_reads_the_filesystem_the_capsule_root_is_on` **only** | the phase's key discrimination ✓ |
| `M18` | the same row, plus `the_second_filesystem_is_on_another_device_or_absent` | must-red ✓; the unconditional capacity row green ✓ |
| `M19` | `a_missing_second_filesystem_reports_skipped_naming_the_reason` | exact ✓ |
| `M20a` | 21 tests — every fixture-building test | the guard refuses, so it discriminates nothing |
| `M20b` | `the_fixture_root_is_on_a_non_tmpfs_filesystem` **only** | exact ✓ |
| `M21b` | `the_fixture_root_is_removed_when_the_fixture_is_dropped` | exact ✓ (`M21` as worded does not compile) |

### `F-32` — three of the sheet's mutations are not compilable as worded

The battery is a source edit, and this crate's deny list is part of the source.
Three rows had to be re-expressed to be *runnable at all*, and the re-expression
is recorded here so the next reader does not read a rewritten row as a dodged
one:

- **`M11`** — "rebuild the whole second placement rather than re-point its root"
  deletes `rebase_onto`'s only call, and `dead_code` is denied: the crate does
  not compile, so no test can red. `M11b` is the same defect expressed so it
  compiles — the root *is* shared and the entries beneath it are **not** moved
  with it (`rebase_onto(&mut …, &own, &own)`). It reds the right test, and for
  the right reason: `ForbiddenScopeOverlap`, which is `F-29`'s trap seen from the
  other side.
- **`M21`** — `drop(&self.path)` is refused by clippy's `drop_ref`. `M21b`
  substitutes `drop(std::fs::metadata(&self.path))`: still touches the path,
  still removes nothing.
- **`M20`** — `TempRoot::new` on `std::env::temp_dir()` **keeps** the
  `on_real_disk` guard, so the fixture refuses to build and 21 tests red on the
  refusal, not on the property. Split: `M20a` is the sheet's mutation as worded
  (recorded, and it discriminates nothing); `M20b` removes the guard too, and
  reds `the_fixture_root_is_on_a_non_tmpfs_filesystem` alone. Only `M20b` is
  evidence that the tmpfs assertion is doing work.

The general shape — a deny list that makes a *plausible defect* uncompilable —
is worth carrying: it means the lint, not the test, is that row's guard. Fine as
long as it is named. It is not fine when it is mistaken for a green battery row.

### `M12` redded nothing, and the fix was in the test

Invariant 4 — the probe arm hands the backend exactly what `provision` returned
— had one test, and that test builds its own `Arm` and calls `run_arm`. So it
establishes that **`run_arm` does not rewrite what it is handed** and says
nothing about `run_row`, which is the only place a delta could reach the probe.
Passing `run_row`'s probe closure through `placed_under` therefore redded
nothing at all: the sheet's `M12` was aimed at a seam no test was watching.

Closed with `the_probe_arm_of_a_row_is_not_passed_through_the_delta`: one real
row (`Delta::Widened`, `ArmShape::Single`), both arms provisioned and run for
real, and the widened entry asserted present in exactly one of the two
placements that reach the backend. `M12` now reds it and nothing else.

The test needed a backend `Stub` could not be: `run_row` provisions *and*
executes through one backend, and `WitnessBackend` cannot provision. `Recording`
is the answer — a `ConformanceBackend` whose `as_capsule_backend` hands out the
**real mechanism**, so the clone's executions go straight to it unrecorded, and
whose `execute_noticing` records and delegates. What is left in `placements()` is
exactly the two arms. Worth reaching for again whenever a seam is only reachable
through a real provision.

### The two mutations whose prediction moved

- **`M1`** was predicted to red nothing *before* the battery ran (`F-15`): a
  backend that ignores its removal is permitted by the type system and reports
  `Unproven`, which is `EX-7`'s intent, so the property lives in
  `a_backend_ignoring_its_removal_yields_unproven_for_every_row` and not in an
  argv diff. The battery confirms both halves — nothing red, and that test green.
- **`M3`** was predicted to red the orphan test's escape half. It does not, and
  `F-30` is why: under `Teardown` the escape is real but *unobservable* — the
  pid namespace's init holds the harness's capture until the namespace empties,
  so the arm cannot return while the escapee lives — and the test therefore runs
  the `ProcessVisibility` control, which has no pid namespace. `M4` — which breaks exactly that control —
  reds it in `M3`'s place. The coverage moved with the arm; it was not lost.

### Extra reds, all of them the same rule under another fixture

`M4` additionally reds a `bubblewrap.rs` unit test asserting the enumerated set
*replaces* `--unshare-all` rather than joining it, and the orphan test whose
control arm is that set. `M5` additionally reds the `--setenv`-list invariant,
which is the same claim stated once for every axis. `M13` additionally reds the
`/proc` stat-parsing unit test, because field 3 is the field it parses. `M16`
and `M18` red both capacity rows because both call the one probe. None is
entanglement between *rules*, and narrowing any of them would trade real
coverage for a tidier table.

## `T12` — the `VA` walks

### `VA-1` — nothing survives a run, and nothing is written beside the fixture

Measured on a **cleaned** baseline: both fixture bases emptied first
(`~/.local/share/doctrine-conformance` and `.git/doctrine-conformance`), a
listing and a `ps -eo pid,sid,comm` taken, the full suite run, then both taken
again.

- **Write half.** Both bases are *empty* after 199 tests. Every `TempRoot`
  reclaimed its own root, and no sibling was left beside them — so no arm wrote
  outside the root it was given. The mechanical half of the claim (every host
  path a placement names lies under the fixture root or a system readable root)
  is now a test, `no_placement_an_arm_builds_names_an_operator_path`, not a walk.
- **Process half.** The before/after process tables differ by three pids, all
  three the measurement shell's own (`bash`, `ps`, `sort`). No capsule, no
  `bwrap`, no `git`, no `sleep` from the run survives it.

**But the baseline was not clean when I found it, and that is the finding.** 26
fixture roots were sitting under the share base and two under the git-dir base,
left by runs that were *killed* — the mutation battery's `M21b` (which disables
`TempRoot::drop` on purpose) and, before it, the runs killed during the `F-31`
hang. `Drop` does not run for a process that takes `SIGKILL`. The leak is
bounded and self-identifying — the base is a dedicated directory and every root
is pid-prefixed — so the remedy is to sweep the base, not to add a delete
primitive (invariant 8, `VA-3`).

**One orphaned capsule was still alive**, from the `F-31` investigation two days
before: pid 277938, session 276883, parented to init, `S` in
`do_wait_intr_irq`. It is `F-31`'s evidence standing in the process table —
`--json-status-fd 4` with **no descriptor 4 open**, and descriptor 6 holding a
*different transaction's* `bwrap-status.json` (`276899-19`'s, in a capsule
belonging to `276899-5`), now `(deleted)`. Killed after recording. Nothing this
run started was among the survivors, and this one was in the *before* table too.

### `VA-2` — no operator path is named by any placement

Written as a test rather than walked:
`no_placement_an_arm_builds_names_an_operator_path`. One provision plus every
`Delta` variant, and for each placement: `root()`, `source().host()` and every
`writable()` host path under the fixture root; every `readable()` host path
under the fixture root **or** under a system readable root the fixture derived
(`/nix`, `/bin` — the shell has to come from somewhere, so a
fixture-root-only assertion is false, not strict). Two clauses keep it from
holding vacuously: at least one system root must exist, and at least one
readable entry must be outside the fixture root.

Discrimination measured, not assumed: pointing the widening at
`std::env::current_dir()` — the operator's repository, the most plausible wrong
answer — reds it. Restored by copy.

It joins `every_artefact_the_fixture_builds_lies_beneath_its_own_root` (what the
fixture *builds*) and `no_readable_root_contains_the_operators_home` /
`no_readable_root_contains_the_fixture_root` (what the root derivation
*excludes*). The three cover build, derive and hand-off.

### `VA-3` — the phase introduced no capsule-delete capability

`git diff 3a1e85dba..HEAD -- crates/` for `remove_dir_all` / `remove_file` /
`std::fs::remove`: **one** added line in the whole phase —

    +        drop(std::fs::remove_dir_all(&self.path));

which is `TempRoot::drop`, over a root the fixture created. The other four hits
in the crate all pre-date the phase: `provision.rs`'s `roll_back` (private,
token-guarded, no `pub(crate)` caller), `bubblewrap.rs`'s removal of its own
`bwrap-status.json`, and two `#[cfg(test)]` cleanups. Nothing public removes
anything, and no capsule root is reachable by a delete from outside the fixture.

### `VA-4` — `execute_weakened` changes exactly its own axis

The mapping is a **total, one-to-one** match arm: `weakening_for` sends each of
the ten `PropertyRemoval`s to exactly one `Weakening` (the eleventh, the grant,
through `weakening_granting`), with one documented fallback —
`StdioOwned` with no endpoints falls back to `Descriptors`, the *confining*
descriptors, so a control that was never built reports `Unproven` rather than
weakening some other axis. Downstream each `Weakening` gates exactly one thing:
one `if !matches!(…)` per flag in `confinement_argv`, one boolean per axis in
`SpawnOptions::under`.

The three clauses the sheet singles out, each with its mechanical backing:

- *`EnvCleared` leaves the explicit `--setenv` list unchanged* — the `--setenv`
  loop is outside the `--clearenv` guard. `every_axis_leaves_the_setenv_list_byte_identical`,
  and `M5` reds it.
- *`StdioOwned` touches nothing above descriptor 2* — it replaces
  `stdin`/`stdout`/`stderr` and leaves `descriptors_closed` true, so the sweep
  above 2 still runs. `M6` — which makes it drop the sweep too — reds the stdio
  case and nothing else.
- *`MappedIdentity` touches no namespace* — it drops `--uid`/`--gid` only, and
  `M8`, which also drops a `--unshare-*`, reds.

`each_removal_changes_exactly_its_own_flags` is the standing form of the whole
inspection: eleven cases, each diffing argv and spawn options against the
confining baseline and asserting the difference is exactly its own. `M2`…`M10`
all bite on it.

### `F-36` — the session sweep's floor, and why it was written too late

The `EX-12` sweep kills by session and refused exactly one: the suite's own.
The harness is not in the suite's session. Measured in this jail — `bwrap` is
**pid 1 in session 0**, the agent process is **pid 2, also session 0**, and the
suite runs in a session of its own. Session 0 was therefore foreign to the
guard and killable, and killing it takes the sandbox and the agent with it: no
error, no shutdown, the operator back at their shell. That happened twice; the
second time the jail's pid 1 was sixteen seconds old, so the whole sandbox had
been rebuilt rather than just the agent restarted.

The causal claim is deliberately bounded: no `dmesg` in the jail, OOM presents
identically, and the first death's draft has been built over. Mechanism,
topology and symptom match; the recording path was never caught in the act.

Floored at both ends in `3860a948c` — a `LOWEST_SIGNALLABLE = 2` refused at
record time, at sweep time, and per-pid so pid 1 is never signalled whatever
session it reports; `own_session: None` fails closed; a discriminating test
each. 201 green in 2.5s afterwards, against a ~14 minute hang before.

**The transferable part is the sequencing.** The floor was scheduled as `T12`
work to avoid changing code under test while the 21-row battery ran — a
defensible reason that produced an indefensible ordering. The battery completed;
the teardown then happened *during* `T12`, before the guard inside it was
committed. An instrument whose failure mode is killing processes it did not
start needs its floor in its first commit, ahead of every row that exercises it.
The rule the next phase should carry: **a destructive test instrument is
floored before it is aimed, never after.**

## PHASE-09 — the measured rows (execution record)

### `T1` — the descriptor window, narrowed to the fork (`F-31`)

`BubblewrapBackend::run` mutated **process-wide** descriptor flags —
`mark_inherited_descriptors_close_on_exec` over everything above 2, then
`clear_close_on_exec` on the status file so the child inherits it — and held
that mutated state across the whole spawn-and-wait. `fork` reads those flags at
the instant it runs, so two concurrent runs interleave: the measured symptom is
a capsule told `--json-status-fd 4` finding fd 4 absent, or fd 6 holding
*another* transaction's status file. PHASE-08 shipped a `#[cfg(test)]` mutex
around the whole of `run` as a stopgap and owed the production fix upward.

**The fix.** A production `static DESCRIPTOR_WINDOW: Mutex<()>` and one new
private function, `fork_within_the_descriptor_window(command,
sweep_descriptors, status_file)`, holding the guard across *mark, clear, spawn,
re-mark* and nothing else. `spawn_and_wait` becomes `observe_and_wait(child,
observer)` and runs outside the guard. No type, trait or signature above
`bubblewrap.rs` moved, so `S1` did not fire; the change is `run`'s body, one new
function, one rename, one new helper (`set_close_on_exec`) and two import lines.

Dropping `Command::output()`'s fast path was checked, not assumed: `output()` is
`spawn` + `wait_with_output` internally, and all three stdio endpoints are set
explicitly upstream, so the two paths are equivalent here.

**`D2` resolved — the window includes the restore.** The narrow reading would
end the guard at `spawn`. It ends one `fcntl` later, after `CLOEXEC` is put back
on the status file. The cost is a syscall; what it buys is the stronger and much
more checkable invariant that *outside the window, no descriptor of this process
lacks `CLOEXEC` because of a capsule run*. Without it the status descriptor
stays inheritable until the next run's sweep, and a subsequent `DescriptorsClosed`
control arm — whose whole point is that the sweep already happened — would
inherit it. Spawn and restore are both attempted before either is reported: a
failed spawn must not leave the descriptor inheritable, and a failed restore must
not mask the spawn error, which is the one the caller can act on.

Post-fork marking inside `pre_exec` was rejected: it would spend the third
`unsafe` `#[expect]` site (`S7`) to buy nothing the guard does not already give.

**Re-entrancy is by construction, and now measured.** The guard is released at
the fork, before `observe_and_wait` calls the observer — which is the callback
that re-enters `run` for `execute_observed`'s descent. A reentrant lock would
have been the wrong repair: it would make the deadlock disappear without making
the interleaving safe.

### Two mutants, and why the first one redded nothing at first

`M-T1a` — delete the `DESCRIPTOR_WINDOW.lock()` line. It would not compile:
`-D unused` reds the now-dead `static` and the `PoisonError` import, so the
mutant needs an `#[expect(dead_code)]` and a narrowed import to even build.
Same class as PHASE-08's `F-32` — a deny list can make a plausible mutation
uncompilable as worded.

Built, it passed `two_concurrent_runs_each_observe_their_own_status_handover`
(two threads, four rounds of real capsules). That is a fixture that cannot
discriminate, not an untested rule (`R7`): the clear→fork window is a few
hundred microseconds against a **~70 ms** per-capsule period, so the threads
essentially never overlap inside it.

The repair was a denser test at mechanism altitude, not row altitude:
`concurrent_forks_each_hand_over_only_their_own_status_descriptor` — 4 threads ×
60 rounds of `/bin/sh -c 'echo ok >&N'` straight through
`fork_within_the_descriptor_window`, each round asserting its own status file
reads back `ok`. Period drops to ~2 ms. Against `M-T1a` it reds at **thread 3
round 1** with `left: "" right: "ok"` — the handover went to the wrong
descriptor. The constants are chosen by measurement and the doc comment says so,
because a later reader will otherwise trim them.

`M-T1b` — move the guard back up to `run`, i.e. the PHASE-08 stopgap's shape.
It **hangs** `a_capsule_run_inside_the_observer_callback_returns`; killed under
`timeout 90`. This converts the plan-time finding `F-1` (that the stopgap
deadlocks row B5 rather than serialising it) from a traced reading into a
measurement, and it is the reason `T11` can now be written at all.

Both mutants reversed textually against `/tmp/sl248-t1/bubblewrap.rs.orig` and
verified by `diff` (0 deletions) plus `grep -c MUTANT` = 0, never by
`git checkout` (`C11`).

**`F-35` bit again.** The `M-T1b` kill left `~/.local/share/doctrine-conformance/63730-0`
behind — `Drop` does not run for a killed process. Hand-swept with `rm -rf` per
`C12`/`R8`; no delete primitive was added to the harness. Process table checked
first: only pid 1 `bwrap`, the jail itself.

**Where the evidence lives.** Two tests sit in `bubblewrap.rs` (the mechanism —
the restore invariant and the fork stress). Two sit in `conformance.rs`, because
running real capsules concurrently needs the fixture and `provision`, which are
that unit's. Both of the latter fail *by hanging* against the old mechanism,
so each was run alone under a shell `timeout` before joining the suite (`R1`).

Suite 201 → 205, 1.38 s, all pre-existing tests green unchanged (`C10`).

### `T2` — `VA-1`: `EVD-013`'s 2×2 re-run against the session-escaping payload

Instrument: `.doctrine/slice/248/spike-teardown-2x2.sh`, output
`spike-teardown-2x2-output.txt`. Both committed with this task, and the sweep
inside the script is floored **above** the cells that need it, not below them
(`F-36`): sid ≥ 2, sid ≠ our own, pid ≥ 3. The pid floor is one higher than
`conformance.rs`'s because this script runs *beside* the agent rather than
under the suite — pid 2 is the process reading its output.

**Method, and the two false negatives it had to route around.** `setsid` is
still absent from this jail's PATH — `EVD-013`'s own recorded trap, still live.
`python3` is present and `os.setsid()` is the same syscall. The first draft had
the escapee `execv` into `sleep` with the search marker as argv[0]; coreutils
ships `sleep` as a **multi-call symlink that dispatches on argv[0]**, so the
escapee died instantly with "unknown program" and every cell reported a clean
false negative. Caught by the positive control, which is the whole reason
`EVD-013` insists on one. The escapee now sleeps in python with the marker in
its source.

Each cell carries its own marker so a survivor is attributed to the arm that
spawned it, and every cell records the arm's **wall duration**, which is what
makes the result legible at all.

| # | pid ns | `--die-with-parent` | arm returns after | descendant survives |
|---|---|---|---|---|
| P | — (no bwrap) | — | 0s | **yes**, in a session of its own |
| C1 | present | absent | **8s** (= the escapee's whole life) | no |
| C2 | absent | absent | 0s | **yes** |
| C3 | present | present | 0s | no |
| C4 | absent | present | 0s | **yes** |

`EVD-013`'s table stands against the strengthened payload: **both** mechanisms
are required and `--die-with-parent` is what reaps. `S2` does not fire. A
descendant that leaves its session is no harder to reap than a plain orphan,
because `setsid` moves a session and the reap works through the namespace.

**C1 is not a reap, and the duration is what says so.** Eight seconds is
exactly `ESCAPE_SECONDS`: the arm waited the escapee out. Read without the
duration, C1 says "no survivor" and looks like C3.

### `C1b` — what hides the escapee is the capture pipe, not the namespace

`$(bwrap …)` waits on two things at once — the process exiting and the capture
reaching end-of-file — and the harness's `wait_with_output` conflates them the
same way. C1 re-run with the arm's stdout on a **file** instead of a pipe:

| arm's stdout | bwrap exits after | escapee visible afterwards |
|---|---|---|
| pipe read to EOF | the escapee's whole lifetime | no |
| file | **0s** | **yes** — two processes, the namespace init and the escapee, in a session of their own |

`bwrap` returns promptly either way. What waits is end-of-file, because the
namespace's init inherits the capture descriptor and holds it until the
namespace empties. This sharpens `F-30`: the thing that makes a teardown
escapee unobservable is the **capture**, which is a choice, not the namespace,
which is not.

`S4` does **not** fire on this: the delta plainly produces its row's control
failure — C1b shows the escaped descendant alive and host-visible. What is in
question is the *observation*, and that is `T9`'s to solve. See `F-6`.

### `T3` — `VA-3`: the four unmeasured deltas, each shown to fire

Instrument `.doctrine/slice/248/spike-deltas.sh`, output
`spike-deltas-output.txt`. Confining form and weakened form for each axis, one
observable difference apiece. `Teardown` is `T2`'s and is not repeated.

| axis | confining | weakened | observable difference |
|---|---|---|---|
| `WorkingDirectory` | `--chdir /capsule` | omitted | `pwd` = `/capsule` → `/` |
| `FileSizeBound` | `RLIMIT_FSIZE` 1 MiB | unset | 2 MiB write truncated to 1 MiB; as the top-level payload, **exit 153** (128+SIGXFSZ) → exit 0 |
| `WallBound` | `timeout -k 5 2` outside `bwrap` | no wrapper | **exit 124** at 2 s → exit 0 at 6 s with the payload's output |
| `ProcessVisibility` | `--unshare-all` | `NON_PID_UNSHARE_SET` | capsule sees own pid 4, host pid **not** visible, 5 `/proc` entries → own pid = the host pid, the harness's pid **visible**, 31 entries |

**No `S4`.** Every one of the four produces its row's control failure, cleanly
and at the altitude the row reads at.

Three things worth carrying forward:

- **`ProcessVisibility` is now row-shaped, not mechanism-shaped.** `F-7`
  measured the `/proc` entry count — 20 vs 4 there, 31 vs 5 here, the number
  moves with what else is running and is not a claim about anything. The row's
  question is whether the capsule can see a *named* host process, so the probe
  asks for the harness's own pid by path. Under confinement: absent. Under the
  weakened set: present, and the capsule reads its own pid as the **host** pid.
  That is the row's control failing, and it is the reading `T3` owed.
- **`FileSizeBound` needs the write to *be* the payload.** With the write inside
  a shell (`dd …; echo …`), `dd` dies of `SIGXFSZ` and the shell carries on, so
  the arm exits 0 and only the truncated byte count betrays the cap. `exec`ing
  the write makes the capsule's top-level process the one that dies, and the arm
  exits 153 — the reading `classify_termination` turns into
  `Termination::FileSizeExceeded`. Both are recorded; row 8's payload must be
  the second shape or the row reads a successful run.
- **Bash's `ulimit -f` is in 1024-byte units, not the POSIX 512.** The first run
  was labelled 512 KiB and wrote 1 MiB. Harmless here because the difference is
  what is measured, but it is the sort of unit error that makes a cap look
  ineffective.

### Where PHASE-09 stands, and the shape of what is left

`T1`, `T2` and `T3` are done, harvested and committed. `T4` onward is not
started; nothing of it is on disk.

The thing the next worker should read first is the sheet's `F-7`. `T4` is not
separable from `T5`–`T11`: `dead_code` is denied, an enum variant that is never
*constructed* trips it, and `table_a()`'s rows are the only construction site
for a `Property`. So the eight variants, the eight rows and the retirement of
`RowId`'s `#[expect(dead_code)]` are one compile. `T4` + `T5` together is the
natural first landing — the spine plus the `Sequential` family, five of the
thirteen rows — after which `T6`…`T11` each append their own rows and their own
row-level test.

Two things checked while working that out, so they are not re-derived:

- a `Row` is only `{id, shape, delta}`; every closure (capsule, execution,
  `live`, `noticed`) belongs to `run_row`, so the table bodies need each row's
  payload argv, its expected observation and its delta, and nothing else;
- populating the tables does not make the existing stub-backend `verify` tests
  run real capsules — those tests inject `FixtureHost`, which has no environment
  by construction, so `Fixture::new` refuses and every row reports
  `Indeterminate`, leaving their three assertions (backend, host, date) intact.

### `T4` + `T5` — the thirteen rows, and the five storage rows executed

Landed as one commit, per `F-7`: denied `dead_code` welds the eight `Property`
variants, `table_a`'s eight rows, `table_b`'s five and the retirement of
`RowId`'s `#[expect(dead_code)]` into a single compile. There is no green
intermediate state to land, and there was no need to fight for one.

**Every payload is a constant.** `Probe.argv` is an `Argv`, and `tables()` is
built eagerly inside `verify` *before* the fixture exists (invariant 1's
ordering). So no payload can be parameterised by the fixture. Two consequences,
both structural rather than stylistic:

- the `Widened` deltas mount their decoys at **fixed inner destinations**
  (`/widened/executable`, `/widened/credential`, `/widened/repository`,
  `/widened/undeclared`), so rows 2–4's payloads can name them literally;
- row 5's listener port is ephemeral and cannot be. `Fixture::new` now binds the
  listener **before** `initialise_project` and writes the port to
  `listener-port` in the project root, so `git add .` commits it and the payload
  reads it from `/capsule/repo/listener-port`. Never `/source` — that is a bare
  export with no working tree.

**`Argv::new` (production, `config.rs`).** `shell_argv` assembles `sh -c
<script>` and knows it is non-empty, but `try_new` returns a `Result` and the
crate denies `unwrap`/`expect`/`panic`. The choice was an invented unreachable
error case, a lint expectation, or putting the totality where the invariant
already lives. The third: `Argv::new(head, tail)` is non-empty by construction,
and `shell_argv` became total, which de-`?`d its two existing call sites.

**`run_row` split into four.** `run_probe_arm` and `run_control_arm` now sit
either side of a shared `arm_over`. This is not tidying — `VT-4`'s discriminator
needs to *read one arm*. A row verdict cannot report it: `Proven` says only that
*the* control failed, and all five storage rows share one `Delta::SharedRoot`,
so without the seam the five `Proven` assertions rest on a control nobody has
seen fail. The alternative was a hand-built copy of the arm in the tests, which
`F-33` forbids for good reason.

**`D3` settled — diagnosable, not just failed.** `reads_sentinels` prints
`SHARED-<path>` per hit and classifies on the whole line, so the diagnostics
cannot themselves be mistaken for the token. A control failure names *which*
storage axis leaked, which is the difference between a red that reports and a
red that only accuses.

`VT-4`'s five titles plus `fresh_mutable_state_is_proven` are green; each was run
alone under a shell `timeout` before joining the suite (`R1`), and the suite is
green five runs out of five, 212 tests.

#### `F-38` — a test that swept process-wide descriptor flags outside the guard

`check gate` redded once, in `bubblewrap.rs`'s
`concurrent_forks_each_hand_over_only_their_own_status_descriptor` — the child
received an empty status file. Both that test and the suspect pass alone; only
the whole suite reds, and only sometimes.

`every_descriptor_above_two_is_marked_close_on_exec_before_the_exec` called
`mark_inherited_descriptors_close_on_exec()` **without taking
`DESCRIPTOR_WINDOW`**. That sweep is a process-wide mutation, so fired from one
test thread it re-marks another thread's just-cleared status descriptor between
its clear and its fork — the exact corruption the guard exists to exclude, and
the exact failure `F-31`'s message predicts. The old doc comment on that test
said the sweep was benign because "nothing in this crate's suite passes a
descriptor to a child expecting it to survive… It stops being benign the moment
one does." `T1` wrote the test that does.

Fixed at the test, by taking the guard. The lesson generalises past this file: a
guard whose contract is over *process-wide* state is not satisfied by every
production caller taking it — a test that mutates the same state is the second
writer, and calling it a test does not exempt it. Recorded as
`mem.pattern.tests.process-wide-state-needs-the-production-guard`.

#### Rows landed knowingly incomplete (each is a later task's, not a defect)

| row | what is there | what the later task owes |
|---|---|---|
| 2 (bounded input set) | execs `/bin/sh` and `git` | `T6`: "each bound path", not two of them |
| 4 (bounded fs visibility) | reads the undeclared decoy | `T6`: the `/`-enumeration half, whose expected set is host-dependent |
| 5 (network posture) | the TCP leg only | `T7`: the abstract-unix leg (`VA-2`, watch `S3`) |
| 7 (teardown) | prints **neither** token, so the row reads `NoObservation` | `T9`: the trusted-side observation — per `F-6` the capture pipe, not the namespace, is what hides the survivor |

Row 8's payload carries `T3`'s measurement directly: `exec head -c <2 × cap>
/dev/zero`, `exec`'d so the capsule's **top-level** process takes the `SIGXFSZ`
and the arm exits 153. Bounded at twice the cap on purpose — the control arm
removes the file-size bound, and an unbounded `cat /dev/zero` there would fill
the operator's disk.

Table B's `Repository` axis cannot carry an oid across capsules, so its reader
recomputes the blob name with `git hash-object --stdin` (no `-w`) rather than
having the writer's oid baked into a constant it cannot know.

### `T6` — rows 2, 3 and 4, the `Widened` family

Three rows, three payload rewrites, one fixture addition, and one more instance
of `F-12`'s class.

**Row 2 — "a binary from each bound path", where *bound path* is the mount.**
The payload derives its own root set from the capsule's `PATH` (first component
of each entry) and requires every root to have contributed a binary that **ran**;
short of that it prints neither token and the row is honestly indeterminate. The
alternative reading — one binary per `PATH` *entry* — is false on a Nix-style
host, where entries are per-package `bin` directories and most hold no member of
any fixed candidate list. Roots are exact rather than approximate here because
readable entries are identity-mapped, so a `PATH` entry's first component *is* a
readable root.

Candidates are `sh`, `cat`, `head`, `env`, `true` — chosen because a bare
invocation with `stdin` closed exits 0 and does nothing. The payload runs no
binary it has not named; picking "the first executable in the directory" would
eventually pick `reboot`.

**Row 3 — the decoy repository needed a secret to deny.** It was `git init
--quiet` and nothing else, so "read the decoy repository's secret blob" had
nothing to read. `Fixture::new` now writes and **commits** one; committed rather
than written, because the property is about history the capsule was not given and
an uncommitted file is just a file next to a `.git`. That produced
`commit_everything`, which also absorbed `initialise_project`'s two
config-add-commit runs — the ident has to be set per *repository*, and the decoy
is a second repository (`mem.pattern.sandbox.git-ident-unset-dns-stall`).

Row 3 reads three paths now: the credential, the secret, and the repository's
`.git/HEAD`. A repository whose history is unreachable but whose ref tip is
readable has still handed over canonical state.

**Row 4 — `/` enumerated against a derived permitted set.** This was the half
deferred at `T4` as "host-dependent expected set". It is not, once you notice the
payload can read its **own** `PATH`: the permitted set is the profile's inner
destinations (`/capsule`, `/agent`, `/source`, `/proc`, `/dev`, `/tmp` — all
constants) plus the first component of each `PATH` entry, which enumerates the
readable roots exactly. Anything else at `/` is the host's root showing through
an unshared mount namespace, and each is printed as `UNEXPECTED-<leaf>` so a
violation says what leaked.

Mutation-checked rather than assumed: replacing `capsule` in the permitted set
with a literal that matches nothing turned the row from `Proven` to
`Indeterminate{Probe, NoObservation}` — the enumeration runs and is load-bearing.

`rows_three_and_four_read_disjoint_paths` is pure and guards `EX-4`: the two rows
deny by one mechanism and stay independent only while they read different things.

#### `F-12`'s class recurred, twice more, in the other direction

`check gate` redded again after `T6`, this time on
`a_decoy_set_opened_after_a_sweep_is_inheritable_again`. Same root cause, opposite
direction: that test (and `the_row_ten_decoys_are_the_only_inheritable_descriptors`)
opens descriptors deliberately inheritable and asserts they *stay* so — while
every capsule run in the suite performs the parent-side sweep that marks every
descriptor above 2 close-on-exec. `T6` added five capsule-running tests, which is
what made a latent race land.

A reader of process-wide state is as much a party to the guard as a writer. The
repair is a named seam: `hold_descriptor_window()` — `pub(crate)`, in
`bubblewrap.rs`, taken by `fork_within_the_descriptor_window` and by the three
tests that touch inheritability. A guard nobody outside the module can take is a
guard everyone outside the module will skip. Memory updated with both directions.

Gate exit 0, three consecutive runs, 217 tests.

### `T7` — row 5, both legs, and `S3` measured rather than assumed

**No `S3`.** Under `--unshare-all` both legs are refused; under the permitted
posture both connect. The verdict is `Proven`, which is only reachable when the
two legs *agree* — the payload prints its hold token when both were refused, its
failure token when both connected, and **neither** when they disagree. So `S3`
would have arrived as `Indeterminate` and named itself, rather than being
averaged into a pass. Same mechanism (the network namespace) takes both legs, as
the design reasoned; the difference is that it is now executed.

**The abstract name travels the way the port does.** Per-run, so it cannot be a
constant, so it goes into the project repository before the base commit and the
payload reads it at `/capsule/repo/listener-abstract`. It is derived from the run
root's path, which already carries pid and nonce — `two_fixtures_bind_distinct_abstract_names`
pins that, because every executed test in this suite builds its own fixture and a
fixed name would `EADDRINUSE` on the second.

**Both legs are dialled with `socat`, and the TCP leg stopped using `/dev/tcp`.**
`/dev/tcp` is a bash extension; `SHELL` is contracted only to be POSIX. On a
`dash` host the redirect fails, the leg reports a refusal it never observed, and
*both* arms hold — which reads `Unproven` and looks exactly like a delta that did
not fire. `socat` is checked for at runtime with `command -v`; absent, the
payload prints neither token, which is an honest indeterminate.

`row_five_dials_both_the_tcp_and_the_abstract_listener` is pure and guards what
the verdict cannot see: a row 5 that *lost* its abstract leg would still read
`Proven`, since one refusal out of one is still "all of them". `VA-2`'s
obligation is that both legs execute, so the count is asserted where it cannot
silently drop.

### `T8` — row 6, and `D1` settled on route 3 (a child process, not a shared cwd)

**`D1`: route 3. Route 2 does not exist without moving a production type.** The
sheet recommended trying `Command::current_dir` on the spawn first. It is the
right *idea* — what bubblewrap inherits is its spawner's cwd — but the `Command`
is built inside `BubblewrapBackend::run`, five call frames below anything the
harness holds, and nothing on the way down carries a working directory:
`Execution`, `CapsulePlacement`, `SpawnOptions` and the `ConformanceBackend`
trait would each have to gain one. That is a production type widened on the
security boundary to serve a test, which is `S1`'s shape and `S5`'s stop.

So the cwd is varied one level up, and route 3 turns out to be route 2 aimed at
a different process: the test re-executes **its own test binary** with
`Command::current_dir(cwd)`, and the child — a whole process whose cwd is ours
to choose — runs the arms. `current_dir` is per-spawn and thread-safe; the child
has its own `DESCRIPTOR_WINDOW`, its own fixture, and no shared mutable state
with the parent suite at all.

Route 1 stays rejected, and the reason is sharper than "process-wide":
`operator_regions` (`conformance.rs:1510`) reads `std::env::current_dir()` to
decide which roots a fixture may bind. Mutating the process cwd would silently
change what *every other concurrently running test's* capsule can see. That is
not a race against a hypothetical reader; it is a race against this file.

**The instrument.** `row_six_arms_from_this_processes_cwd` is `#[ignore]`d — it
is a measurement, not a claim, and it is selected by `--exact` + `--ignored`
from `row_six_measured_from`. Its name is a constant (`ROW_SIX_HELPER`) because
a rename that forgets it selects *zero* tests, and a child that runs nothing
prints nothing; the parse then fails loudly rather than measuring nothing
quietly. Three prefixed lines cross the boundary (`ROW6-CWD=`, `ROW6-VERDICT=`,
`ROW6-TRACKS=`), read out of the child's stdout by prefix.

**Two readings per cwd, and the second is what gives the first force.** One cwd
cannot distinguish *the working directory is fixed at `/capsule`* from *it is
merely not equal to this one directory* — the design's measured table
(`design.md:3906–3915`) separates them only because it has two rows without
`--chdir`. So each child reports the shipped row's verdict **and** the arm
result of a row differing from it in one place only: `Observed::Exactly` naming
the child's own cwd instead of `/capsule`, on the same payload, on the
`WorkingDirectory`-removed arm. Measured, both cells:

```
--chdir /capsule, trusted cwd /      →  pwd = /capsule   (Proven, tracks Held)
--chdir /capsule, trusted cwd /tmp   →  pwd = /capsule   (Proven, tracks Held)
```

`tracks: Held` is the un-`--chdir`ed capsule landing on the trusted side's own
directory, so `/capsule` on the probe arm is a directory the profile *chose*
rather than one it happened to inherit.

**Why `/` and `/tmp`.** Both exist on the host and inside the capsule at the
same path — `/` is the capsule's root, `/tmp` its tmpfs. A host-only directory
would leave bubblewrap's fallback `chdir` to fail and the capsule wherever
bubblewrap left it, and the measurement would be about that fallback rather than
about inheritance. The suite's own cwd is deliberately not one of the two: it is
whatever `cargo test` chose, so it is not a *varied* input.

**Mutation-checked.** Disabling the `--chdir` push in `confinement_argv` turned
`deterministic_working_directory_is_proven` from `Proven` to `Violated` and
redded the two-cwd test with `verdict: "Violated", tracks: "Held"` — the two
readings move independently, which is what makes the pair worth running.

Cost: two child processes, ~1.5 s each, three capsules apiece. Gate exit 0, 222
tests (one ignored, which is the instrument).

### `T9` — row 7 stops on `S8`: the mechanism discriminates, the vocabulary cannot say so

`T9` is **blocked**, not abandoned, and the block is a design gap rather than a
measurement problem. Everything below was measured in this jail, not reasoned.

**The first thing measured killed the plan.** Coming into `T9` the reasoned route
was `Observed::Termination(Termination::Exited { code: 0 })` with the suite's wall
bound pushed below the escape, so the control arm would read `TimedOut` where the
probe read a clean exit. A throwaway `#[ignore]`d spike running both arms of row 7
through `run_arm` with an injected short-bound `Execution`, printing the whole
`Observation` and the elapsed time, says otherwise:

```
payload                       arm                  termination      stdout      elapsed
escapee(20s), streams to null Confining            Exited{code:0}   "LIVE\n"     1.03 s
escapee(20s), streams to null Removing(Teardown)   Exited{code:0}   "LIVE\n"    20.03 s
sleep 10 (wall bound 3 s)     Confining            TimedOut         "LIVE\n"     3.00 s
sleep 10 (wall bound 3 s)     Removing(Teardown)   TimedOut         "LIVE\n"    10.02 s
```

`classify` reads exactly two things — `termination` and `stdout` — and row 7's two
arms agree on both. The arms differ **only in elapsed wall time**, which nothing in
the observation vocabulary can see. `C1b` said the capture pipe hides the survivor;
the sharper statement is that the capture pipe hides *everything*, so row 7 is
unreadable through `classify` as it stands.

Rows 1–2 also retire the `TimedOut` idea on its own terms: with the escaping
payload the wall bound never fires, because `timeout(1)`'s child exits 0 at ~1 s
while the capture is held open by a process `timeout` is not waiting on.

**Then the mechanism turned out to be easy.** Leave the escapee's stdout
*inherited* rather than redirected to `/dev/null`, and have it speak only after its
parent is gone — the payload states the held token before exiting, the survivor
states the failed token later:

```
sh -c "python3 -c '<fork; setsid; sleep 4; print REACHED>' </dev/null 2>/dev/null;
       echo LIVE; echo DENIED; sleep 1"
Observed::Token { held: "DENIED", failed: "REACHED" }

Confining          -> Held                       stdout "LIVE\nDENIED\n"          1.03 s
Removing(Teardown) -> Indeterminate{Ambiguous}   stdout "LIVE\nDENIED\nREACHED\n" 4.03 s
```

Teardown kills the escapee before it can speak; removing teardown lets it speak.
That is row 7's property, in about five seconds of wall clock, with no production
type touched — and it settles `S2` in the negative on measured ground:
`--die-with-parent` **is** required for the session-escaping payload, so row 7's
control stays `Removed(Teardown)` and is not switched to `ProcessVisibility`
(`EX-8`).

**What blocks it is one cell of a truth table.** `Observed::Token` maps *(held
present, failed present)* to `AmbiguousObservation`, so the control arm reports
`Indeterminate` and the row reports `RowVerdict::Indeterminate` where it must
report `Violated`. That cell guards a real defect — one payload printing both
tokens is broken — but here the second token arrives from a **different process,
after the capsule exited**, which is not the ambiguity it guards. And invariant 5
(an absence is not a hold) is exactly why the held token must be *stated* by the
payload rather than inferred from the escapee's silence, which is what puts both
tokens on the wire in the first place.

Three resolutions exist and all three widen the observation vocabulary the design
fixed: a new `Observed` variant (held iff `stated` present and `forbidden` absent);
re-reading `Token`'s both-present cell globally, which weakens every other row's
guard; or a trusted-side observation through the `Arm.live` / `noticed` seam, which
`classify` never consults for `ArmShape::Single`. That is `S3`'s "not yours to
widen" one axis over, so the choice goes upward. The moment the observation is
named, the rest of `T9` — payload, direct escape assertion, the process-group-only
reaper mutant, and its signalling floor — is unblocked.

### Two facts `T9` paid for that outlive it

**Removing teardown also neuters the wall bound.** `timeout -k` wraps `bwrap` from
outside (`EX-15`). Under `Removing(Teardown)` the wrapper still fires on schedule
and still reports `TimedOut`, but killing `bwrap` no longer kills what `bwrap`
started: the payload survives, keeps the harness's capture pipe, and
`wait_with_output` returns only when *it* finishes. Rows 3 and 4 of the table
above: identical bound, identical verdict, **3.00 s against 10.02 s**.
`--die-with-parent` is what makes the wall bound an actual bound. So any row whose
control removes teardown pays its payload's full natural runtime rather than its
bound, and `TimedOut` on such an arm does not mean the arm was bounded — only that
the wrapper fired.

**`setsid(1)` is absent from this jail, and row 7's shipped payload uses it.**
`escapes_its_own_session` spawns its escapee with `setsid`; `command -v setsid` is
empty here (`perl` too; `python3` is present). A missing `setsid` makes the escape
a no-op that still prints the liveness marker — the clean false negative `T2`
already paid for once. Every measurement above used `python3 -c` with `os.fork()`
plus `os.setsid()`. Whoever unblocks the row must either change the shipped
payload's escape mechanism or accept that row 7 is unmeasurable in the jail. The
task's own rule — assert the escape directly, so a payload that stops escaping
fails rather than passes — is what catches this, provided the assertion is written
before the payload is trusted.

### `T10` — row 8, and the constant that turned out to be a suite budget

Five payloads, one per `Termination` variant. Two of them carry a control and are
read as rows; three carry none and are read as arms. That split is the row's
substance rather than an omission (`EX-9`): `Exited`, `Signalled` and
`NotExecutable` are observations of what the OS reported, and no confinement
property's removal would change them, so there is nothing for `run_row` to apply.
The enforcement half is the two bounded payloads — file-size (the shipped row) and
wall (built locally, because the design's table A has one row 8 and the row count
belongs to the design, not to the suite).

**The wall bound made `FIXTURE_TIMEOUT_SECONDS` a runtime budget.** The only
payload that can prove a wall bound is applied is one that overruns it. So the row
costs the bound on its probe arm, and on its control arm — where the bound is
removed — it costs whatever the payload sleeps, which `EX-10` requires to be a
small fixed multiple. Three times the bound, every run, irreducibly. At the
inherited 120 s that is six minutes for one row. Lowered to **30 s**, which keeps
an order of magnitude over every payload the suite actually runs (the longest is
row B5's subject at three seconds), and the constant's own doc comment already
described it as small and as containment rather than budget.

Measured after the change: **227 tests, 0 failures, 90.73 s**, of which 90 s is the
wall row. The suite's wall clock is now `3 × FIXTURE_TIMEOUT_SECONDS` plus noise —
everything else runs in parallel underneath it. That is `R4`'s number, and it is a
lever rather than a fact: PHASE-10 can trade the bound against the suite directly.

`ESCAPE_SECONDS = 23` now sits *under* the bound rather than well under it, and
its doc comment justified the gap by claiming the wall bound would otherwise reap
row 7's escapee. `T9` measured that false on both arms — the probe's escapee dies
with teardown about a second in, and on the control arm `timeout(1)`'s own child
exits cleanly at that same moment, so the bound never fires. The comment now says
the relation is decorative, because it is.

**`NotExecutable` versus `Exited { code: 127 }` is not a contest between two exit
codes.** The design's pairing reads that way; the mechanism is not. When the
capsule's exec fails, `bwrap` exits **1**. `classify_termination` calls that
`NotExecutable` only because bubblewrap's `--json-status-fd` never reported an
`exit-code` key — which is what `child_ran` carries — and the `127` in the pair
comes from the other payload entirely, a shell that ran, looked and could not
exec. POSIX's number, not bubblewrap's.

That is worth a mutation rather than an assertion, and it is the discriminating
one `R7` asks for: forcing the `child_ran` branch (`if true || child_ran`) reds
both `not_executable_*` tests with `left: Exited { code: 1 }, right:
NotExecutable`. The `1` in that message is the whole finding.

Two consequences for anyone extending the row. An argv-refusal payload must be
asserted on the *outcome*, never on a code. And the `NotExecutable` observation
prints no liveness marker of its own — it never ran — so its liveness has to come
from an execution preceding it **in the same placement**: same mounts, same
profile, asserted to have exited zero and printed the marker. Without that
ordering the observation passes identically on a host where the capsule cannot
execute anything at all, which looks exactly like success.

**Distinction without a cross-product.** `every_termination_variant_is_distinguished`
reads each payload against its *own* expected termination and requires `Held`, then
asserts the expectations pairwise distinct. `Observed::Termination` classifies by
equality, so holding against your own variant is failing against every other one.
Five capsule runs instead of twenty-five, and the same claim.

### `T11` — row B5, and `D4` settled by looking rather than by inheriting

**One token cannot carry two claims.** `EX-12` asks B5 to assert two things
positively — the subject is absent from the observer's `/proc`, and the observer
cannot signal it — and the shipped observer folds both into one token
(`[ -e /proc/N ] || kill -0 N`). That is right for the row: a verdict is binary
and `Observed::Token` is a pair. It is not enough for `VT-5`, because a control
that restored only one of the two would still read as restoring "the" reading.

So the row is built twice, once per half — same subject, same
`Delta::Removed(ProcessVisibility)`, same `ArmShape::Concurrent`, differing only in
the observer's condition — and both go through `run_row`. Both `Proven`. The
control is then read as an **arm** across both halves rather than as a verdict,
because `RowVerdict::Proven` says *the* control failed and the claim is that
**both** became possible under the one delta. Four arms at `SUBJECT_LINGER_SECONDS`
apiece, about six seconds a test.

**The pid-provenance test needed a subject that lies, and a capsule that would
believe it.** Invariant 10 says the observed pid is the one the trusted side saw,
never one the subject reported about itself. To assert that, the wrong answer has
to be plausible: the subject prints `SUBJECT-PID=1`, because `1` is what `$$`
reports inside a fresh pid namespace and `/proc/1` **exists** inside the observer's
namespace. A harness that read the subject's claim would hand the observer a pid it
can see, the observer would print `REACHED`, and the arm would fail. The arm
holding *is* the assertion. Two checks ride with it: the pid actually handed over
is not the decoy, and it resolves in the host's process table, which a
capsule-namespace pid could not.

One mutation covers the whole seam: substituting `HostPid(1)` for the reported pid
inside `observe_concurrently` reds all three tests at once — both split rows to
`Violated`, the provenance arm to `Failed`.

**`D4`: the sweep is sufficient, and it is now asserted rather than presumed.**
`the_sweep_reaches_what_row_b5s_control_leaks` runs B5's real control arm — shipped
subject, shipped delta — then reads the sweep: every session the arm noticed is
swept, none is the harness's own, and `drained_of` finds nothing alive in any of
them afterwards. Taken from B5's own arm, not inherited from row 7's bespoke
payload, which is what `D4` asked for.

The mechanism is also why B5 leaks where row 7 blocks. B5's control removes
`ProcessVisibility`, so the survivor is an ordinary host process in a foreign
session and the session sweep reaches it. Row 7's control removes teardown, and its
survivor stays **inside** the pid namespace — which is what makes that arm block on
a capture pipe rather than leak a process (`T9`). Same containment machinery, two
quite different shapes of leak.

**A trap paid for on the way.** A concurrent arm runs two capsules (`EX-6`), and a
directly-built `Arm` that hands both of them one cloned `CapsulePlacement` fails
with `MechanismFailed("No such file or directory")`: the two runs race the single
status file in the shared transaction root. `run_probe_arm` provisions per call for
exactly this reason, and anything assembling an `Arm` by hand has to do the same.

### `T12` — `VA-4`: the thirteen-row walk, per row rather than in aggregate

Two invariants, confirmed one row at a time because a verdict cannot distinguish
them. `RowVerdict::Proven` is *probe `Held` and control `Failed`*, so reading it
back to prove "the control was seen to fail" is circular; and a row whose arms
differed by two things would still read `Proven`.

**Invariant 2 — the control was seen to fail.** Executed, not argued:
`every_shipped_rows_control_is_seen_to_fail` walks `tables()`, calls
`run_control_arm` on each row, and asserts `ArmResult::Failed` per row, sweeping
between rows. Twelve of thirteen; 7.25 s. The one exclusion is **named in the
source** as `UNWALKED` rather than filtered silently — an exclusion nobody can
see is how a walk starts lying. Mutation-checked by moving `UNWALKED` to another
row: the walk then reds on row 7 with
`Indeterminate { reason: NoObservation, termination: Exited { code: 0 }, stdout: "LIVE\n" }`,
which is `T9`'s blockage reproduced from the walk's own side. So the test is not
vacuous and the exclusion is not cosmetic.

**Invariant 3 — one typed `Delta`, one `ArmShape`.** This one is carried by the
types, not by discipline: a `Row` holds **one** `shape` and **one** `delta`, and
`run_row` hands `row.shape` to both arms, so a row whose arms differ in two
places or run different shapes cannot be spelled. What a test can still add is
that the shipped tables are the tables the design describes —
`the_shipped_tables_are_thirteen_distinctly_identified_rows` pins the count at 13
and the ids as distinct, so a row silently duplicated or dropped cannot pass as
the walk having covered it.

**The walk.** `EX-13`'s two payload rules read per row: is the payload the
*hardest* instance of the property's negation, and does the row observe the
property rather than a convenient consequence of it?

| row | shape | delta | control seen to fail | hardest negation | observes the property |
|---|---|---|---|---|---|
| A1 `FreshMutableState` | `Sequential` | `SharedRoot` | yes — walk + `control_second_transaction_in_the_first_root_does_share_each_storage_axis` | writes a sentinel to **every** declared writable location, then a second transaction reads them all | reads back what a *prior transaction* wrote, not merely that the dir is empty |
| A2 `BoundedInputSet` | `Single` | `Widened(decoy executable)` | yes — walk | execs from **every mount root its own `PATH` names**, not one probe binary (`F-14`) | the decoy is reached by exec, not by `stat` — the property is what can *run* |
| A3 `DeniedCanonical…` | `Single` | `Widened(credential + repo decoys)` | yes — walk | three paths, incl. the decoy's `.git/HEAD`: unreachable history with a readable ref tip is still canonical state handed over (`F-15`) | reads the secrets themselves; counted, not short-circuited, so a partial reach prints neither token |
| A4 `BoundedFilesystemVisibility` | `Single` | `Widened(undeclared decoy)` | yes — walk | both halves: the undeclared decoy *and* a full `/` enumeration against a derived permitted set (`F-16`) | an unexpected leaf is named `UNEXPECTED-<leaf>`, so the row sees the host showing through, not just one path |
| A5 `ExplicitNetworkPosture` | `Single` | `NetworkPermitted` | yes — walk | **both** legs — TCP and abstract-unix — through `socat`, to trusted-side listeners that exist (`F-18`, `F-19`) | holds only if both legs were refused, fails only if both connected; a disagreement is `Indeterminate`, never averaged into a pass |
| A6 `DeterministicWorkingDirectory` | `Single` | `Removed(WorkingDirectory)` | yes — walk | the trusted-side cwd is *varied* across a re-executed child, so "fixed at `/capsule`" is separated from "unequal to one directory" (`F-21`, `F-22`) | `Observed::Exactly(/capsule)` — the directory itself, not a proxy for it |
| A7 `ProcessTreeTeardown` | `Single` | `Removed(Teardown)` | **no — `UNWALKED`** | payload escapes its own session; but `setsid(1)` is absent in this jail (`F-27`) | **blocked at `T9`/`S8`**: both arms yield a byte-identical `Observation`, so `classify` cannot read this row at all (`F-24`, `F-25`) |
| A8 `TrustedTerminationObservation` | `Single` | `Removed(ResourceBound(FileSize))` | yes — walk | writes `OVERSIZE_MULTIPLE ×` the cap; the wall half overruns its bound by `WALL_OVERRUN_MULTIPLE` (`F-28`) | the observation is the `Termination` the trusted side read; the five variants are asserted pairwise distinct, so holding against one *is* failing against the rest (`F-30`) |
| B1 `Checkout` | `Sequential` | `SharedRoot` | yes — walk + the storage test | sentinel written into the capsule's own checkout, read by a second transaction | the axis's own path, not the transaction root |
| B2 `Repository` | `Sequential` | `SharedRoot` | yes — walk + the storage test | writes an **object and a ref** — a repo that lost its objects but kept its refs is not fresh | reads both back; either surviving is a failure |
| B3 `Runtime` | `Sequential` | `SharedRoot` | yes — walk + the storage test | sentinel in `INNER_AGENT` | ditto |
| B4 `TemporaryState` | `Sequential` | `SharedRoot` | yes — walk + the storage test | sentinel in the capsule's **retained** tmp leaf, not a scratch one | ditto |
| B5 `Process` | `Concurrent` | `Removed(ProcessVisibility)` | yes — walk + `control_with_the_pid_namespace_shared_both_become_possible` | a **live** subject observed while it lingers, whose pid comes from the trusted side and which lies about its own (`F-32`) | both readings — absent from `/proc` *and* unsignallable — split into two rows so a control restoring one cannot read as restoring both (`F-31`) |

Twelve rows walked, one named and excluded. The exclusion is `T9`'s, not `T12`'s:
row 7 is unreadable through the shipped `Observed` vocabulary, and the choice of
how to widen it is the orchestrator's (`F-25`).

## PHASE-09 close-time defect — the pid-provenance test's unsound corroboration (`F-34`…`F-36`)

Found by the orchestrator at close, after the phase's tasks were ticked:
`doctrine check gate` red in **2 of 3** runs on
`the_observed_pid_is_the_one_the_parent_reported_not_one_the_subject_printed`
(`HostPid(<n>) does not resolve in this host's process table, so it is not a
host pid`), against **3 of 3** green on bare `cargo test -p doctrine-control`.
The gate builds immediately before testing, so the machine is loaded — that is
the discriminator, and it is what says *race*, not *flake*.

### `F-34` — the read was taken after the subject had been waited on

The test asserts two things about the pid the trusted side handed the observer.
`assert_ne!(subject, HostPid(DECOY_PID))` — the subject printed `SUBJECT-PID=1`
and nothing believed it — is timing-independent and carried the real claim. The
second, `session_of(subject).is_some()`, ran **after `run_arm` returned**. By
then `execute_observed` has waited on the subject; a reaped process has no
`/proc` entry, so the read was asking after a pid that no longer existed. It
answered `Some` only when the reap lagged the assertion. Not a flaky test — an
unsound one that a quiet machine kept rescuing.

### The repair — resolve where liveness is *certified*, not where it is assumed

Deleting the corroboration was the cheap fix and the wrong one: a pid that
merely differs from the decoy could still be garbage, which is the failure the
assertion exists to exclude. The resolution now rides the `Arm.live` seam:

```rust
let resolved: Cell<Option<SessionId>> = Cell::new(None);
let live = |pid: HostPid| {
    resolved.set(session_of(pid));
    capsule_still_running(pid)
};
```

and the post-arm assertion reads `resolved.get().is_some()`.

**Why `live` and not `noticed`.** `noticed` fires earlier — the instant
`observed_capsule_process` establishes the pid — and the subject is live there
too, but only by an *argument* about `/proc` persistence before the reap; the
test would assert nothing about it. `live` is the one point liveness is
**certified**: `classify_concurrent` yields `Indeterminacy::NoObservation`
unless this closure returned true, so the `assert_eq!(result, ArmResult::Held)`
already in the test *is* the proof that the recorded resolution was taken while
the subject ran. The seam that decides row B5's window is made to also witness
the pid's provenance, and one assertion now backs both. Reads are ordered
resolution-then-liveness so the weaker read is the earlier one. The claim is
unchanged; only its footing is.

**Empirical control, not just a green run.** The old assertion was temporarily
restored beside the new one and the pair run 6× under 24-way CPU saturation:
**5 pass / 1 fail**, the failure being the old form while the new form was green
in the same process. That is the mutation-style evidence this slice asks for
elsewhere (`R7`) — the repair is seen to matter, not assumed to.

**Tally.** `doctrine check gate` **5 / 5 green unloaded**; green under an
8-spinner load; the repaired test green in **every** run, including two runs
under 24-spinner saturation that redded four of its neighbours (`F-35`).

### `F-35` — four sibling row-B5 tests are load-fragile in the same class, **unfixed, owed upward as an `ISS-`**

Under 24 spinning cores on a 32-core host, `doctrine check gate` failed 2 of 2 —
never on the repaired test, always on its neighbours:
`concurrent_capsules_cannot_signal_each_others_processes` (both runs),
`concurrent_capsules_cannot_see_each_others_processes`,
`control_with_the_pid_namespace_shared_both_become_possible`, and
`the_sweep_reaches_what_row_b5s_control_leaks`. The symptom is
`Indeterminate { arm: Probe, detail: NoObservation }`.

That indeterminacy is the harness being **honest**: the subject's
`SUBJECT_LINGER_SECONDS = 3` window closed before the observer capsule — a full
bwrap provision and run — completed, so `subject_live_when_observer_ran` was
false and the row correctly refused to report a verdict. The mechanism is right;
the *fixture window* is too narrow to survive contention, and these tests assert
a definite verdict unconditionally.

Left unrepaired on purpose. The candidates are a tradeoff, not a fix: widening
`SUBJECT_LINGER_SECONDS` taxes every concurrent row and interacts with
`FIXTURE_TIMEOUT_SECONDS`, which `F-28` already cut 120 → 30 *because* row 8
makes it a suite-runtime constant; a bounded retry on `NoObservation` folds
fixture flakiness into the row algebra. Calibration for whoever takes it:
unloaded 5/5 green, 8 spinners green, 24 spinners 0/2 — latent under today's
gate, not active, but the gate's own build is real contention.

### `F-36` — one green run is not verification of a timing-dependent suite

`F-34` did not reach the orchestrator as a suspicion. It reached it as a phase
claimed **"gate exit 0"** — a statement that was true and worthless, because one
green run of a suite containing a race says only that the race was won once.
This suite is unusually exposed: rows B5 and 7 assert on live processes, `/proc`
entries, reaps and windows, so a large share of its assertions are statements
about *when*, and `cargo test`'s parallelism plus the gate's preceding build
make load a hidden variable in every run.

The rule this phase paid for: **where a test reads a live process, a `/proc`
entry, a window or a clock, the unit of evidence is a tally under load, not an
exit code.** Both `F-34` and `F-35` were invisible to a single run and visible
within minutes of running the suite repeatedly and under contention — and
`F-35` is still open precisely because the cheap evidence never showed it.

## PHASE-10 `T1` — `VA-2`: the four reasoned deltas, measured

**Shard note.** `phase-10.md` nominates `notes_10-12.md` as PHASE-10's shard and
forbids creating one silently. No such file existed at this sitting, so PHASE-10
appends here, per the sheet's own fallback.

`weakening_for` (`conformance.rs:1736`) carries `InputsWritable`,
`DescriptorsClosed`, `EnvCleared` and `StdioOwned` as **reasoned** (`EX-18`).
Rows 9–12 each stake a control arm on one. `VA-2` requires each shown to fire
*before* the row depends on it, and `S3` says a delta that does not produce its
row's control failure means the row is wrong.

Measured through the shipped seam — `provision_capsule` → `harness_execution` →
`ConformanceBackend::execute_noticing` under `Under::Confining` versus
`Under::Removing(…)` — so the argv exercised is the one `weakening_for` actually
emits, not a shell approximation of it. **All four discriminate.** `S3` is not
triggered and no row is withdrawn.

### The tally (`C14`, `F-36`): 5 runs per delta per arm, 32 spinners on 32 cores

| delta | probe reading | control reading | *n*/5 |
|---|---|---|---|
| `InputsWritable` | `WROTE-NOT /source` (`Read-only file system`), `NOT-WRITABLE /bin`, `NOT-WRITABLE /nix` | `WROTE /source`, `WRITABLE /bin`, `WRITABLE /nix` | 5/5 both arms |
| `DescriptorsClosed` | `/proc/self/fd` = `0 1 2 3` (3 is the enumeration's own handle) | `0 1 2 3` **plus** `5 → decoys/descriptor`, `6 → #NNNNNN (deleted)`, `7 → socket:[…]` | 5/5 both arms |
| `EnvCleared` | 10 variables — exactly `CapsuleEnv` | 42 variables — the whole trusted-side environment | 5/5 both arms |
| `StdioOwned` | `STDIN[]` | `STDIN[TRUSTED-SIDE-DECOY-INPUT]` | 5/5 both arms |

Every arm exited `Exited { code: 0 }`; no run went indeterminate, and no 4/5.
Wall clock per spike run under load: 0.85–1.86 s.

### The exact argv delta each produced

Measured off `confinement_argv` (`bubblewrap.rs:1106`), confining versus
weakened, over one real placement — plus `SpawnOptions::under`, which is where
the two parent-side axes live.

| delta | argv words | change |
|---|---|---|
| `InputsWritable` | 40 → 40 | three `--ro-bind` → `--bind`, at positions 14, 17, 20. **No word added, none removed, no path changed.** |
| `DescriptorsClosed` | 40 → 40 | **none — the argv is byte-identical.** `SpawnOptions.descriptors_closed` `true` → `false`. |
| `EnvCleared` | 40 → 39 | `--clearenv` removed. The `--setenv` list is untouched. |
| `StdioOwned` | 40 → 40 | **none — the argv is byte-identical.** `SpawnOptions.parent_owned_stdio` `true` → `false`. |

Two of the four change **no argv byte at all**. That is not a weakness: it is
what makes `the_descriptor_control_changes_no_mount_no_env_and_no_argv_byte`
(`VT-3`) and `the_stdio_control_changes_nothing_above_descriptor_two` (`VT-5`)
assertions about a measured fact rather than restatements of an intention. And
`InputsWritable`'s 40→40-with-three-substitutions is
`the_writable_inputs_delta_changes_no_mount_and_no_path` (`VT-2`) measured: the
mount set, its count, its inner destinations and its host paths are all
identical across the arms, and only the attachment moves.

### `F-9` — the descriptor decoys must be opened **after provisioning**, not merely per arm

`F-6` predicted a vacuous row 10 and named the probe arm's sweep as the cause.
The measured cause is one layer broader and it matters for `D1`.

Opening the decoy set *before* `provision_capsule` and holding it across both
arms — which is what "per fixture" and a naive "per arm" both amount to — reads:

    STALE  probe   = 0 1 2 3
    STALE  control = 0 1 2 3

The control sees **nothing**. Opening a fresh set *after* provisioning and
before the spawn reads:

    probe   = 0 1 2 3
    control = 0 1 2 3 5 6 7

Provisioning itself runs capsules, and every one of those runs goes through
`fork_within_the_descriptor_window` with the sweep enabled, marking every
descriptor above 2 close-on-exec process-wide. So a decoy opened before
`(Arm.capsule)()` is already `CLOEXEC` by the time the arm forks, whichever arm
it is. **A per-arm hook is necessary and not sufficient; the hook has to sit
between the capsule closure and the spawn.** That is the placement `T2` adopts.

### `F-10` — `bwrap` does not close inherited descriptors, so the mechanism is the parent's sweep alone

Checked directly, because the alternative explanation for `F-9`'s stale reading
was that bubblewrap closes what it inherits and the delta could never fire:

    exec 9</tmp/decoy; bwrap --unshare-all … /bin/sh -c 'ls /proc/self/fd'
    → 0 1 2 3 9=>/tmp/decoy

Descriptor 9 survives into the capsule. Bubblewrap 0.11.2 passes inherited
descriptors through untouched, so the whole of row 10's confinement is
`mark_inherited_descriptors_close_on_exec` on the trusted side — and the whole of
its control is skipping that call. Worth knowing before anyone reads a
non-discriminating row 10 as a bubblewrap fact.

### `F-11` — `InputsWritable`'s control arm makes the **system readable roots** writable, outside the fixture

The delta is *"`--ro-bind` becomes `--bind` for `/source` **and every declared
readable entry**"*. On this host the declared readable entries are
`system_readable_roots`' output: `/bin` and `/nix`. Under the control arm a
payload that writes through them writes onto the operator's filesystem, outside
the fixture's `TempRoot`, where nothing reclaims it — `Fixture::Drop` reaches
only its own root and this slice adds no unlink (invariant 8).

Measured, not reasoned: the first run of this spike wrote through every readable
mount and left `/nix/va2-write` and `/bin/va2-write` on the host, owned by the
operator. **Swept by hand**, and the spike was then narrowed to write only into
`/source` — this run's own export, inside the fixture root — and to probe the
other entries non-mutatingly with `[ -w ]`, which discriminates just as cleanly.

This is `T3`'s problem, and it is exactly what
`the_writable_inputs_control_writes_only_to_this_runs_own_export` (`VT-2`) is
for. Note the tension it names: a `Row` carries **one** `ArmShape`, so the probe
and the control run the *same* payload. `a_write_through_every_readable_mount_fails`
therefore cannot be satisfied by a payload that literally writes through every
readable mount — that payload, on the control arm, writes into `/nix`. Row 9's
payload has to write only into the source export and establish the other entries'
read-onlyness by a non-mutating read. Recorded before `T3` starts so it is a
design constraint rather than a discovery made by an operator finding files in
their nix store.

### `F-12` — `EnvCleared`'s control arm hands the capsule live operator credentials

Probe: 10 variables, exactly `CapsuleEnv`'s set. Control: 42, the trusted side's
whole environment. On this host that set includes several third-party API keys
belonging to the operator, in full, readable by the capsule.

No action is owed on the *delta* — leaking is precisely what the control exists
to demonstrate, and it discriminates beautifully. What is owed is on the
**payload**: row 11's probe is set equality against `CapsuleEnv`, and its
reporting must be **names and counts, never values**. A payload that printed
`env` verbatim on failure would put the operator's credentials into the test log
and into CI output, which is a worse disclosure than the one the row exists to
prevent. The spike was rewritten to print names only after the first run proved
the point.

### `F-13` — reopening `/dev/fd/1` does not test whether the capsule can read descriptor 1

The first stdio spike asked `head -c 1 /dev/fd/1` and read `FD1-READABLE` under
**both** arms, which would have looked like row 12's second leg failing to
discriminate. It is an artefact twice over: opening `/proc/self/fd/N` on a pipe
*reopens the pipe* rather than duplicating the descriptor, so a read end is
obtainable even where the descriptor held is write-only; and the payload had
already written its own output into that pipe, so the read returned the payload's
own bytes.

`F-30`'s claim — that what a socket pair confers and a capture pipe's write end
does not is the capsule's ability to **read descriptor 1** — is a claim about the
inherited descriptor, so `T6` must read descriptor 1 *itself* (`<&1`), never
`/dev/fd/1`, and must bound the read: under the control arm descriptor 1 is a
socket whose peer the trusted side holds open across the run, so an unbounded
read blocks until the wall bound rather than returning (`R1`). The stdin leg is
unaffected and was measured clean; this bears only on the descriptor-1 leg.

## PHASE-10 `T2` — the per-arm trusted-side setup seam (`D1`)

*Same shard-fallback note as `T1`: `notes_10-12.md` does not exist and the sheet
forbids minting one silently, so `T2`'s record lands here.*

### `D1`, settled — the setup rides the capsule closure, keyed on the delta

`arm_over` composes `trusted_side_setup(&row.delta, fixture)` onto the capsule
closure it hands to `Arm`, holding the result in a `RefCell<Option<
InheritableDecoys>>` scoped to its own frame. `Arm`, `run_arm`, `Row`, `Delta`
and `PropertyRemoval` are all untouched; `run_row` is still the only route to a
backend.

Three things decided it, and the first is the one that matters:

1. **Placement is the load-bearing constraint, not per-arm-ness** (`F-9`).
   Provisioning runs capsules, and every capsule fork sweeps this process's
   descriptors close-on-exec — so the state has to open *after* the arm's
   capsule closure returns. Composing onto that closure puts the ordering in the
   structure rather than in a comment above it.
2. **The delta is the right key.** What a control arm needs in order to be able
   to fail belongs to the thing being removed, not to the row carrying it. Both
   arms therefore get a set from the same call, and the probe's non-vacuity
   (`EX-3`) is a consequence rather than a convention.
3. **Scope gives the lifetime, so there is no teardown call to forget.**
   Replaced per capsule; the last set drops when `arm_over` returns.

Rejected: a sixth field on `Arm` — identical ordering, reads more explicitly,
but widens a struct four hand-built test arms construct, so `C10` would be paid
in edits to tests `A4` says establish nothing (this is the migration if a later
phase wants the hook visible). A row-side hook keyed off `RowId` — wrong key per
(2), and no live key today. A `SetUp`/`TearDown` pair — converts a guarantee
scope already gives into a call that can be forgotten or mis-ordered.

### `F-15` — the `EBADF` floor is met by construction, and the floor as written cannot be built

The rule the sheet cites governs a `/proc/self/fd` **walk** — flag mutation over
descriptors the caller does not own. The seam has none and cannot have one:
`BorrowedFd::borrow_raw` is the only route from a raw number to an `AsFd`, it is
`unsafe`, and both budgeted sites are spent (`S7`). Every descriptor the seam
touches is owned, so teardown is `close(2)` and there is no `fcntl` for `EBADF`
to race. A `BADF`-tolerant restore here would be a guard that cannot fire.

The floor that *is* live, and landed in the same commit ahead of `T4`:
`trusted_side_setup` holds `hold_descriptor_window()` across the open, because
inheritability is process-wide state and this is code that opens an inheritable
descriptor and needs it to stay inheritable. It releases before returning —
`fork_within_the_descriptor_window` takes the same lock, so holding it across
the spawn deadlocks the arm at its own fork.

### `F-16` — the one residual race, measured rather than reasoned

Between the window's release and the arm's own fork, another thread's capsule
run can sweep the decoys and leave the control arm nothing to inherit. Closing
it needs a re-entrant window, which production rejects for a better reason than
this one. Narrowed, not closed, and tallied: **5/5** alone under 32 spinners on
32 cores, **8/8** inside the whole conformance suite at `--test-threads=16`
under the same load. `T4` should re-tally when row 10 carries it for real.

### `F-17` — `A7`'s inherited `F-35` set confirmed and named

Three of five whole-suite multi-threaded runs were red, and every red was one of
PHASE-09 `F-35`'s four row-B5 tests — nothing else failed in any run. The seam
cannot be the cause: `trusted_side_setup` returns `None` for every delta a
shipped row carries, so no shipped row's behaviour changed, and the
single-threaded suite is 236/236 green unchanged.

## PHASE-10 `T4` — row 10: `ClosedDescriptorSet` / `DescriptorsClosed`

Shard note (`C9`): the sheet nominates `notes_10-12.md` and forbids minting one
silently. Still no such shard at this sitting, so this record joins `T1`'s and
`T2`'s here under the sheet's own fallback — the third time that instruction has
been overridden, which is itself worth correcting.

### The payload — resolution, not counting; presence, not readability

`ls -1 /proc/self/fd` in a clean capsule reports `0 1 2 3`. The fourth entry is
the enumeration's **own** directory handle, so *no descriptor above 2 appears* is
literally unsatisfiable, and the round-4 row failed before any decoy was ever
inherited (`RV-346` `F-36`). Row 10 therefore resolves rather than counts, and
excludes exactly one descriptor **by identity**: the handle whose target is
`/proc/*/fd`. Everything else above 2 is printed as `FD-RESOLVED-<target>` and
the row says `DESCRIPTOR-INHERITED`.

Three things had to be true at once and only one shape gets all three:

- **One process, not two.** A `readlink` per entry resolves *`readlink`'s* own
  descriptor table, not the capsule's — `F-13`'s trap one level up. A glob over
  `/proc/self/fd/*` lists a directory handle that is already closed by the time
  the shell reads it. `ls -l` enumerates and resolves in the same process, in one
  read.
- **Exactly one own handle, or no answer.** The payload requires `own -eq 1` and
  otherwise prints neither token, so a host whose `/proc` answered strangely
  reads `NoObservation` → `Indeterminate` rather than passing.
- **Named diagnostics.** A failure names what crossed the `exec`; a count would
  only say the table was longer than someone expected.

The shell was validated standalone before any Rust was written: clean → `LIVE` /
`NO-DESCRIPTOR-ABOVE-TWO`; one readable decoy at fd 9 → `LIVE` /
`FD-RESOLVED-…` / `DESCRIPTOR-INHERITED`; unchanged under a pipe.

### Isolation — the decision `F-19` left open, and it was taken

`F-19` closed `F-16` for the seam's own discriminator by measuring it in a child,
and recorded that the residual stays open for **any shipped row carrying the
descriptor delta**. Row 10 is that row. Every executed row-10 claim here is
measured in a child process — `--exact <helper> --ignored --nocapture`, the
parent requiring the marker line positively because a selector matching nothing
exits 0 having run nothing.

The alternatives were all settled upstream and none was reopened: a re-entrant
descriptor window (rejected by `fork_within_the_descriptor_window`'s own
contract), carrying an already-held window into the fork (`S6`, a signature
change), a `#[cfg(test)]` mutex (a known-bad shape that deadlocks row B5), an
idempotent re-check (laundering), and weakening the assertion (the vacuous pass
`EX-3` forbids).

**The residual is the multi-threaded test binary's, not `verify`'s.** `verify`
runs the rows sequentially in one thread, so nothing of its own can sweep this
process between a row's setup and its fork. The window that made `F-18` a 1-in-3
is a property of the runner, and removing the runner removes it.

### The three authority modes

A test-local row (id borrowed, as `descriptor_shaped_row`'s is) that holds when
**none** of the three kinds is present, fails when **all three** are, and prints
nothing in between — so a partial set reads `Indeterminate` and two-of-three can
never be mistaken for *the modes are exercised*, which is the failure a `-ge 1`
threshold would have had. Each mode is recognised by what the descriptor
resolves to: the readable decoy by its path under the decoy directory, the
write-only one by the kernel's ` (deleted)` suffix, the socket end by `socket:`.
Row 10 itself is deliberately blind to kind — it must fail on any of them — so
this is the complementary claim the shipped row cannot make about itself.

### The write-only decoy, both directions

`F-31`'s lesson stated as a test. The decoy is `O_TMPFILE` and write-only: a
capsule that inherits it reads zero bytes through it, exactly as it would
through a closed one, which is how the round-4 row passed a real leak. So the
trusted side reads the file's **size**, never its contents, and reads it in both
directions — `[Some(0), Some(23)]` against arms `Held` / `Failed`. One direction
alone proves nothing: *mutated after the control* would pass for a harness that
wrote on every arm, and *unmodified after the probe* for one that never wrote.

Two things this cost, both recorded as findings. The reading has to happen inside
`arm_over`, because the set is per-arm state dropped there and the file is named
by nothing (`F-26`). And the writing payload is the phase's one destructive
instrument, so its containment landed ahead of the aim (`C13`): the capsule
writes only to a descriptor whose target it has already resolved to an unlinked
file, never to a number chosen blind; there is exactly one redirection; and where
that write lands was already established by
`the_write_only_decoy_is_reachable_by_no_name`.

### The mutants

`F-26`'s (one readable descriptor) and `F-31`'s (one write-only descriptor), both
`Violated` — a probe arm that *fails* convicts regardless of the control, so a
backend leaking on both arms cannot launder the leak into *the removal changed
nothing*.

Their stdout is **observed, not predicted**: row 10's shipped script is run under
`/bin/sh` with exactly one decoy left inheritable and the other two swept by
hand, and what it really prints is what the stub answers with. A discriminating
leg runs the same pipeline with nothing leaked and requires a verdict that is not
`Violated`. Two limits, both findings: no stub can reach `run_row` at all
(`F-24`), and a *real* partial-leak backend is not constructible without a
production visibility change or a third `unsafe` site (`F-25`).

### Tally

Five sequential full runs of `doctrine check gate` — the channel the hazard runs
on, per `F-36`; a green `cargo test` or a CPU spinner would not have seen `F-16`
either. All five `exit=0`, all five `248 passed; 0 failed; 6 ignored`, wall
91.45–91.55s. **5/5.**

That number is only worth stating against its predecessor: the same channel
measured `F-16`'s window at roughly 1 in 3 before the isolation, so five clean
runs is evidence the arms no longer race a sweep rather than a lucky streak.
The suite moved 236 → 248 passing with 6 ignored, the growth being the
`#[ignore]`d child instruments; a log reading `236 passed; 2 ignored` is from a
previous session and counts for nothing here.

