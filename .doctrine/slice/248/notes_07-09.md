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
