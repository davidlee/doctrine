# SL-248 — execution record, PHASE-04 … PHASE-06

Shard per `LOOP.md` § *Notes, sharded*. What was done, what diverged from the
phase sheet, what was measured. Cross-phase items and anything owed to the
reconciliation brief stay in `notes.md`.

## PHASE-04 — the backend contract and placement validation

One file of production code, `crates/doctrine-control/src/backend.rs`, plus the
two shared/append-only edits the sheet permits: `mod backend;` in
`crates/doctrine-control/src/main.rs` and one `[doctrine_control_tiers]` row in
`.doctrine/adr/001/layering.toml`. No dependency added (`S4` clear); nothing
imported from `crate::host` in production code (`D2` held).

Result: **63 tests green** in `doctrine-control` (33 on arrival, 30 new — the 28
mandated titles plus `overlaps`' own test and the trait witness). Clippy clean.

### What diverged from the sheet

1. **The per-task red/green sequencing (T2–T10) was not available**, and this is
   a property of the lint configuration rather than a shortcut taken. The module
   carries PHASE-03 `D5`'s `#![cfg_attr(not(test), expect(dead_code, …))]`
   header, which `cfg(test)` strips; `unused = "deny"` then makes every item
   without a *test* reader a hard error. After the task that landed the type
   vocabulary, `cargo test -p doctrine-control` produced **69 errors**, none of
   them signal — including constants that *are* referenced, by a `const` slice
   that is itself dead, so the whole chain reads dead. There is no partial-green
   state between the first task and the last. The first successful compile came
   with all thirty tests present, and they all passed at once.

   Everything passing on the first compile is not evidence, so the red phase was
   reconstructed as a **mutation battery** (below). Recorded as
   `mem.pattern.lint.staged-module-unused-deny-collapses-tdd` and as a friction
   observation against `/execute`.

2. **`try_new`'s filesystem-root rule is tested per declared entry, before the
   general overlap rule, rather than as a trailing stage.** `D5`'s arrow reads
   *inner-path rules → carve-outs → blanket scope overlap → filesystem root*;
   its operative "Concretely" sentence rules only on carve-outs-before-blanket
   and is silent on the filesystem root. Ordering the specific rule last makes it
   **unreachable**: `/` is an ancestor of every scope, so a general overlap test
   answers first and `PlacementRefusal::FilesystemRoot` could never be
   constructed — which
   `mem.pattern.rust.unconstructible-variant-evades-dead-code` says is invisible
   to the compiler, and which would make
   `declared_root_that_resolves_to_the_filesystem_root_refuses` green whether the
   rule existed or not. The stage order `D5` rules on is unchanged; the
   specific-before-general choice sits inside the last stage. Mutation `M5`
   measures that the test now discriminates. See § Findings note (i).

3. **The transaction-root carve-out admits an entry *equal to* the root**, not
   only a strict descendant. The design says "a descendant of *this placement's*
   `root`"; the root is this placement's own writable state, so refusing it is
   the over-denial direction `R2`/`F-25` warns about, and no criterion or title
   covers equality. A *sibling* is what the rule denies, and `M7` confirms the
   cluster reds without the carve-out.

4. **The source must *strictly* descend from `<capsule_root>/export/`** — the
   export directory itself is refused, since the contracted export is
   `<capsule_root>/export/<base-oid>`. Asserted in
   `a_source_outside_the_export_directory_refuses`.

### The mutation battery (the reconstructed red phase)

Green file snapshotted to scratch, one rule broken at a time, restored **by
copy** each time (`mem.pattern.git.revert-control-by-edit-not-checkout`).

| # | mutation | reds |
|---|---|---|
| `M1` | the pre-round-4 blanket rule: `source` subjected to the forbidden-scope walk | **20 of 24** placement tests, incl. `the_lawful_source_export_for_this_base_is_admitted` |
| `M2` | `overlaps` reduced to equal-or-ancestor (`F-10`'s defect) | 11 tests — every descendant mutant, plus the inner-collision pair |
| `M3` | the transaction-root licence extended to readable entries | exactly `a_readable_entry_under_this_placements_own_transaction_root_refuses` |
| `M4` | reserved-destination tested by `overlaps` instead of equality | exactly `a_readable_entry_beneath_a_profile_owned_mount_is_admitted` |
| `M5` | the filesystem-root rule deleted from the declared-entry walk | exactly `declared_root_that_resolves_to_the_filesystem_root_refuses` |
| `M6` | the source's carried base identity no longer compared | exactly `a_source_export_of_a_different_base_refuses` |
| `M7` | the transaction-root carve-out removed entirely | exactly `R3`'s transaction cluster (4 tests) |

`M1` is `T6`'s mandated demonstration and it measured **more** than the sheet
predicted. The sheet expects the one positive control to red; in fact the
pre-round-4 rule reds twenty of the twenty-four placement tests, because the
lawful source export is a precondition of every other placement fixture. That is
`RV-346` `F-25`'s own claim — "every conformance row would have failed before
running, for a reason that has nothing to do with any property under test" —
reproduced as a measurement rather than an argument.

`M3`/`M4`/`M5`/`M6` each red **exactly one** test: the pairing discipline
(`EX-9`) produced tests that discriminate their own rule and nothing else.

### T12 — the layering row bites (`ISS-326` does not reach this phase)

`layering.toml` snapshotted, the `backend = "leaf"` row deleted, `cargo test
--test architecture_layering` run: **red**, at
`tests/architecture_layering.rs:1304` —

```
GATE FAILED over crates/doctrine-control/src (section [doctrine_control_tiers]):
        "backend",
```

Restored by copy; green again (25 passed). So `ISS-326`'s narrowness — the gate
demands a tier only for a unit appearing in an *edge* — does not bite here, as
the sheet predicted: `backend → config` is a real edge.

### T13 — the inspection criteria

**`VA-1` — `working_directory` has no inherit value.** Recorded verbatim:

```rust
pub(crate) struct InnerPath(PathBuf);
pub(crate) fn try_new(path: PathBuf) -> Option<Self> {
    path.is_absolute().then_some(Self(path))
}
```

A one-field newtype over `PathBuf`, one fallible constructor, no sentinel and no
`Option` *inside* the type. `PlacementParts.working_directory` and
`CapsulePlacement.working_directory` are both plain `InnerPath` — grep for
`Option<InnerPath>` and for a sentinel returns nothing. The constructor's
`Option` return is the refusal channel, not a stored inherit value. The empty
path is not absolute, so non-emptiness follows from the one condition.

**`VA-2` — no pure test stands in for an executed-only invariant.** Method:
strip trailing comments (`sed -E 's://.*::'`) so doc prose cannot produce a hit,
then word-boundary search the remaining code for `unshare|clearenv|cloexec|
O_CLOEXEC|RawFd|AsRawFd|FromRawFd|no_new_privs|uid|gid`, and separately for any
`"--…"` long-flag literal. **No hits in code, both searches.** Positive control
(ledger 19's lesson: a clean sweep proves nothing without one): the same stripped
stream matches `\bstarts_with\b` four times, so the pipeline reaches code.

Two prose hits, both `--clearenv`, at `backend.rs:547` and `:1466`. Neither is an
assertion: `:547` is `CapsuleEnv`'s doc recording that such a flag stops
*inheritance only*, and `:1466` is
`no_public_route_carries_caller_supplied_environment_text`'s mandated *what this
does not cover* disclaimer naming `sec-7` row 11 as the executed home. They are
`EX-16` being complied with in writing, not violated.

**`VA-3` — no property-count numeral.** Method: a proximity search, both
directions, for a digit or count word within ~40 characters of
`propert(y|ies)|clauses?|invariants?|channels?|rows?`. One hit,
`backend.rs:654`: *"[`Argv`] itself refuses one (invariant 9)"* — "one" is a
pronoun and "invariant 9" is a citation, which is the design's own reference
form, not a count. Positive control: the same pattern hits *"the fourteen
properties are listed in sec-7"*. A second sweep for `<count> <noun>` prose found
only `two carve-outs`, `one variant` ×3 — all statements about this unit's own
types, none restating `sec-7` Table A.

### Carried forward

- **PHASE-06 must delete four dead-code headers**, not three: `host.rs`,
  `config.rs`, `capacity.rs` and now `backend.rs`. All four carry the identical
  `reason` string, so they are greppable.
- **PHASE-05 imports the inner layout, never restates it** (`D9`):
  `INNER_PROC`, `INNER_DEV`, `INNER_TMP`, `INNER_SOURCE`, `INNER_CAPSULE`,
  `INNER_AGENT` and `RESERVED_INNER_DESTINATIONS` are `pub(crate)` in
  `backend.rs`. `EXPORT_DIRECTORY_LEAF` likewise.
- **PHASE-05 owns `CapsuleEnvVar::Path`'s value.** `fixed_value()` returns
  `None` for `Path` by design (`D6`) — the inner `PATH` is composed from the
  bound paths at execute time, in `bubblewrap.rs`. Every other variant's value
  is a constant here.
- **PHASE-05 will add one line to `backend.rs`** (`mod bubblewrap;`). The module
  doc's layering sentence is worded so that does not falsify it (`F-3`): it says
  a profile lands as a submodule that the gate maps to `backend` itself.
- **`AcceptedBase` lives in `backend.rs`** (`D1`/`F-4`). PHASE-06 `VT-6`'s
  keyword mandate over `transaction.rs` is satisfied by
  `use crate::backend::AcceptedBase;` plus the field.

## PHASE-05 — the bubblewrap backend

`crates/doctrine-control/src/backend/bubblewrap.rs`, ~2050 lines, 34 tests.
Suite 63 → **97**, gate exit 0, `Cargo.lock` unmoved.

### What diverged from the sheet

- **`S1` was discharged before work started.** The `F-1/R` ruling landed on the
  sheet (option i): `unsafe_code` `forbid` → `deny`, `"process"` on `rustix`,
  exactly two `#[expect(unsafe_code, reason = …)]` sites, and a mechanised check
  holding the count. Both manifest edits made; `Cargo.lock` md5 identical before
  and after, `cargo metadata` clean — the plan-time reasoning confirmed by
  measurement rather than assumed.
- **An eighth `ProfileRefusal` variant.** `D6` lists seven; `EmptyReadableSet` is
  the fail-closed floor for *everything declared, nothing produced* — reachable
  when a resolver returns no members, which `ConfigRefusal::NoReadableInputs`
  cannot catch because it runs before expansion. It is also what makes the
  sheet's own `the_readable_set_is_never_empty` reachable.
- **`D4`'s pure seam could not perform `D5` step 4.** Existence and resolution of
  each resolver-returned path needs `HostFacts`; `closure_members` is pure by
  construction. Step 4 moved to `expand_closure_root`, which has the host.
  `D4`'s actual constraint — refusals in production code, not fixtures — is met.
- **Three profile-owned binds the design's assembly list does not mention.**
  `/source`, `/capsule` and `/agent` cannot arrive through the declared vectors
  (`RESERVED_INNER_DESTINATIONS` refuses them), so the backend derives them from
  the placement's typed fields and emits them first in their block.
- **The status channel is a file, not a pipe.** `std::io::pipe` is 1.87; the
  workspace pins `rust-version = "1.85"` and `clippy::incompatible_msrv` is
  denied. `rustix`'s `pipe` feature was outside the ruling's widened file set.
  A regular file under the transaction root, outside every bound path.
- **`Execution` carries no kill grace**, so `timeout -k` had no configured
  figure. `BubblewrapBackend` holds one with a `with_kill_grace` builder for
  PHASE-06; the clean fix is a field on `Execution` and is not this phase's.

### The mutation battery (the reconstructed red phase)

All ten mutations applied from a green snapshot and restored by copy; `diff`
clean at the end. Nine redded exactly their named test on the first run. Two
results are worth carrying:

- **`M10` redded nothing** — and the rule was fine. The fixture's host `PATH`
  had host order and lexical order *coinciding*, so `sort()` was a no-op on that
  input and the assertion held under either rule. Fixture changed so the two
  orders disagree, plus an `assert_ne!` on the sorted copy so the discriminating
  property is itself asserted. The criterion was never touched. This is the
  vacuous-criterion class caught by the method the sheet mandated — the battery
  paid for itself here alone.
- **An `M7` run produced an unexplained extra red**, once in ~20. Not the
  mutation: the descriptor sweep skipped a failed `fcntl_getfd` but *propagated*
  a failed `fcntl_setfd`, so another thread closing a descriptor between the
  listing and the mark failed the whole sweep. Its doc comment already claimed
  the skip. Fixed to skip on `Errno::BADF` and only that errno. An intermittent
  found by an expectation, not by chance.

`M8` behaved exactly as the sheet predicted: the floor mutation left
`every_descriptor_above_two_…` **green**. `RV-346` `F-30`'s lesson reproduced as
a measurement instead of quoted.

### `VA-1` ran the argv the code assembles

A throwaway `#[test]` printed `confinement_argv`'s real output; the fixture-only
paths were swapped for real ones and *that* vector went to `bwrap` 0.11.2. Probe
removed by restoring the snapshot. Four things fell out of one run: `F-3`
reproduced (the `/tmp`-resident bind works, `EX-3` lawful, `S2` did not fire);
`F-4`'s reversed-order control reproduced (`/tmp` empty, input gone, exit 0 —
silent); `F-5` reproduced (`uid=1000 gid=1000 groups=1000,65534` — the overflow
gid survives, so invariant 15 is still PHASE-10's); and `D11` confirmed —
`{ "exit-code": 0 }` in the status file, and `/proc/self/fd` inside the capsule
showing only `0 1 2 3`, the status descriptor not crossing.

### Carried forward

- **PHASE-06 deletes five dead-code headers**, not four: `host.rs`, `config.rs`,
  `capacity.rs`, `backend.rs`, and now `backend/bubblewrap.rs`. Identical
  `reason` string in all five, so they are greppable.
- **`backend.rs:50-53` is now stale** and `S5` blocked fixing it: it says the
  `backend` unit's out-edges are `{config}` and that a profile adds "no row and
  no edge". With `bubblewrap.rs` importing `crate::host` the edges are
  `{config, host}`. The *row* claim still holds — `VA-4` green, 25 passed.
- **`"bwrap"` is a literal in `backend.rs`'s test fixture** and now duplicates
  `BWRAP_EXECUTABLE`. `STD-001` wants the constant; `S5` blocked it here.
- **`CapsuleEnvVar` has no `name()`**, so the seven variable *names* live in
  `bubblewrap.rs` and their *values* in `backend.rs`. One `name()` would reunite
  them.
- **The `forbid` → `deny` flip wants an ADR** (recommendation, not a ruling —
  the sheet's `F-17`). The mechanised two-site budget is what makes `deny`
  acceptable, and a test can be deleted with no governance trace.

## PHASE-06 — `provision`, the thirteen steps, and the five headers

Landed: `transaction.rs` and `provision.rs` (new), the `provision` verb in
`main.rs`, two `engine` rows in `[doctrine_control_tiers]`, one free function in
`src/interpretation.rs` under `F-1/R`, and the five `dead_code` headers deleted.
**114 tests pass** in `doctrine-control` (up from 90 at PHASE-05's close).

### Measured

- **`T10` residual: 2 error groups / 6 items**, against the plan-time baseline of
  **164 errors** taken with the headers stripped and no `provision` present. So
  ~96% of the baseline evaporated transitively once `main()` was wired into the
  chain, which is what the staging predicted. The six:
  `BackendId::as_str` (1) and `UnrootedCapsuleConfig`'s five accessors.
- **Ladder rungs used: 1 and 2. Rung 3 was not reached.** `BackendId::as_str`
  went to rung 2 — the `provision` verb's success line names the backend, because
  an observation that cannot be attributed to a mechanism cannot be attributed to
  an admission verdict either. The five `UnrootedCapsuleConfig` accessors went to
  rung 1: every consumer is `config.rs`'s own test module, so the `impl` block
  carries `#[cfg(test)]`. **Zero item-level `#[expect(dead_code)]` was added**, so
  `S3` did not fire and no finding is owed for `T10`.
- **`ProvisionRefusal::paths()`/`keys()` and, transitively, `ConfigRefusal::keys`,
  `PlacementRefusal::paths`, `ProfileRefusal::paths`/`keys` reached their first
  real consumer** — `render_refusal` in `main.rs`. That is what rung 2 means here:
  the accessors were never dead, they were unwired.
- **The `unsafe` budget is untouched.** Two `#[expect]` sites, both in
  `bubblewrap.rs`, and `the_unsafe_budget_is_exactly_two_sites` stays green.

### The mutation battery — 15 mandated rows, 2 supplementary, all green-restored

Applied to green source, one at a time, restored by copy from a scratch snapshot
and `diff`-verified after each (never `git checkout`). Reds, as measured:

| id | reds | verdict |
|---|---|---|
| `M1` | `export_that_is_a_symlink_refuses` | exact |
| `M2` | `export_with_an_alternates_file_refuses_rather_than_being_adopted` | exact |
| `M3` | `export_holds_the_contracted_history_and_no_other_ref` | exact |
| `M4` | `concurrent_publication_converges_on_one_export_and_the_loser_adopts_it` | **under-red** — see below |
| `M5` | `concurrent_publication_converges_on_one_export_and_the_loser_adopts_it` | **under-red** — see below |
| `M6` | `provision_onto_an_existing_temporary_export_directory_refuses_and_removes_nothing` | exact |
| `M7` | that plus `failed_provision_removes_only_the_root_it_created` | over by one, within `VT-3` |
| `M8` | `provision_onto_an_existing_transaction_root_…`, `failed_provision_removes_only_the_root_it_created` | exact |
| `M9` | `placement_pairing_a_base_with_another_bases_export_refuses` | exact |
| `M10` | `resolver_whose_basename_is_forbidden_by_the_policy_refuses` | exact |
| `M11` | `a_failing_execution_in_the_three_step_clone_refuses_at_that_step`, `clone_inside_leaves_no_working_tree_trusted_side` | exact after a fixture fix |
| `M12` | `capsule_identity_persists_into_the_clone_config` | **redded nothing first** |
| `M13` | `capsule_identity_persists_into_the_clone_config` | **redded nothing first** |
| `M14` | `a_transaction_id_carrying_a_path_separator_refuses_at_construction` | exact |
| `M15` | `clone_inside_leaves_no_working_tree_trusted_side` | exact |
| `M16` | `failed_provision_leaves_an_existing_export_intact` | supplementary, added because `M4`/`M5` under-redded |
| `M17` | `export_is_adopted_across_transactions_on_the_same_base` | supplementary, added because nothing redded it |

Every one of the 15 mandated `VT` test titles now reds under at least one
mutation, and every mutation reds at least one test.

**`M4`/`M5` under-red for one structural reason, and it is not a defect.** Both
target the publish path, and `failed_provision_leaves_an_existing_export_intact`
never reaches it: step 8.1 adopts the already-published export and returns before
any rename. The prediction assumed a route the code does not take. `M16` — a
transaction-failure rollback that also removes the export — is what gives that
test its evidence, and it reds exactly it.

**`M7`'s extra red is not entanglement.** Both `VT-3` titles assert the same rule
at step 9 (a root this call did not create must survive), which the sheet's own
`VT-3` row lists them under together. `failed_provision_removes_only_the_root_it_created`'s
second half *is* the collision case.

**`M11` first redded four, and two of those were fixture coupling.** The scripted
backend's positions bound every test that merely needed a *successful*
provisioning to the length of the clone sequence. `clone_succeeds()` is now
`WitnessBackend::always(…)`, which succeeds at any length; `scripted` survives for
the one test that must vary per call. How many executions there are is
`clone_inside_leaves_no_working_tree_trusted_side`'s claim and no other test's.

**`M12`, `M13` and `M17` are `F-8`** — three mandated tests with no evidence,
fixed in the tests and never in the rules. Detail in the sheet.

### `VA` sweeps

**`VA-1` — invariant 6 at every `ProvisionRefusal` construction site.** 22 sites,
walked one by one. Four classes, no exceptions:

1. *Before any keyed create* (13 sites): `Config`, `ConfigUnreadable`, `Capacity`,
   `BasePolicy`, `RefinementPolicy`, `Restriction`, `BaseDocumentAbsent`,
   `BaseDocumentUnreadable`, `RefinementUnreadable`, `ForbiddenResolver`,
   `Profile`, and `Export`/`DirectoryNotExclusivelyCreated` on their step-8.1
   routes. Steps 1–7 create nothing this call owns, so there is nothing to roll
   back. ✔
2. *The create itself failed* (`create_exclusively`, both call sites): no token is
   minted and nothing was created at that path. This is why the two collision
   tests can assert that a pre-existing directory's contents survive. ✔
3. *Under the export temporary's token* (`Export` at 8.3/8.5, `ExportBuildFailed`
   from `build_export` and from the general rename error): every one of these
   arms calls `roll_back(token)` before returning, including 8.5's, where the
   rollback runs whether or not the adoption validated. ✔
4. *Under the transaction root's token* (`Placement`,
   `InnerDestinationNotAbsolute`, `EmptyExecutionArgv`, `Backend`, `CloneFailed`,
   `IdentityNotPersisted`, and `DirectoryNotExclusivelyCreated` from `finish`'s
   layout containers): all are reached from `finish`, whose single caller rolls
   the root back on any `Err`. ✔

One honest residual, by design: the **shared containers** — the capsule root,
`export/`, `tx/` — are created by `ensure_container` and are never rolled back.
They belong to no transaction and removing one would race every other. Invariant
6 is "removes nothing it did not create", not "creates nothing when it refuses",
and the second was never claimed.

**`VA-2` — no delete capability leaks.** One delete exists in `provision.rs`:
`std::fs::remove_dir_all(&token.path)` inside `roll_back`, which is a private
`fn`, takes `CreationToken` **by value**, and has no `pub(crate)` caller.
`CreationToken` is a private struct constructed only by `create_exclusively`,
also private. `provision.rs`'s whole `pub(crate)` surface is `provision`,
`host_capsule_config`, `ProvisionRefusal::paths`/`keys` and the vocabulary types
— none removes anything. `main.rs`'s verb exposes no removal and no flag that
reaches one. The crate's only other production delete is
`bubblewrap.rs:341`, PHASE-05's removal of the status file it just created and
read. ✔ (`DEC-156`'s hazard is a tidiness primitive a later slice reaches for;
there is nothing here to reach for.)

**`VA-3` — steps 1, 4, 5, 6 pure given `HostFacts`.** Structural, as `T6` kept
the four as free functions: `capsule_config(&str, &dyn HostFacts)`,
`resolved_policy(&str, Option<&str>)`, `admit_resolver(&InterpretationPolicy,
Option<&Argv>)`. The last two take no host at all. `HostFacts` has exactly four
methods — `available_bytes`, `resolve`, `path_exists`, `env_var` — none a clock,
none an entropy source, so it is clock-free at the call site. The disk reads live
in the thin shell: `host_capsule_config`, `read_working_tree_document`,
`read_base_document`, `read_refinement_document`. The one clock in the crate is
`SystemTime::now()` in `main.rs`'s `mint_transaction_id`, which is where `EX-7`
puts it — outside `provision`, so a test can hand the same id twice, and both
collision tests do. ✔

### Diverged from the sheet

- **`PhaseIdentity` is `{ slice: String, phase: u32 }`**, not `sec-3`'s
  `SliceId`/`PhaseNumber` — those types do not exist and `EN-3` forbids widening
  the root export set to add them. `F-7`.
- **Steps 2 and 7 are one call** (`readable_set`), so there is no statement
  numbered 2. `EX-9`'s ordering still holds by construction. `F-4`.
- **`ForbiddenScopes`' credentials list is empty** — no configuration source
  exists at this altitude. `F-5`.
- **A non-refusing capacity report goes to stderr**, because `EX-6`'s three
  parameters and `EX-1`'s nine fields leave it no return channel. `F-6`.
- **`AcceptedBase::as_str()` was added** to `backend.rs` (already in the widened
  set, `C1`): the export path, the fetch refspec and the detach all name the base
  and none could read it.
- **`backend.rs`'s `mod bubblewrap;` became `pub(crate)`** as `T7c` directed, and
  `profile_owned_host_path` was promoted to `pub(crate)` rather than
  re-deriving `<root>/capsule` in `provision.rs`.
- **Two supplementary battery rows** (`M16`, `M17`) beyond the mandated fifteen.

### `T11` — the three `backend.rs` debts (`notes.md` item 34)

- **(a) closed.** `backend.rs`'s module doc now says out-edges `{config, host}`
  and that a profile adds no **row** but does add that edge, citing
  `bubblewrap.rs`'s `F-2`. Item 27 closes with it.
- **(b) closed.** `CapsuleEnvVar::name()` now single-sources the seven names
  beside their values; `bubblewrap.rs`'s `env_var_name` is deleted and
  `HOST_PATH_VARIABLE` is defined as `CapsuleEnvVar::Path.name()`, so `PATH`'s
  second role — the host variable read through `HostFacts::env_var` — keeps one
  spelling.
- **(c) closed as no-change** (`D7`, adjudicated at plan time). `backend.rs`'s
  bare `"bwrap"` sits inside `WitnessBackend`, a test double describing an
  *unavailable fake* backend. It is descriptive text about a fiction, not a
  second spelling of the executable the profile execs, and importing
  `BWRAP_EXECUTABLE` into it would couple a backend-agnostic witness to one
  profile. Item 34(c) does not roll forward.

### Still owed

- **Item 35 stays open by decision, not by omission** (`D3`). `Execution` still
  carries no kill grace; the verb threads the configured figure through
  `BubblewrapBackend::with_kill_grace` instead. Moving the field would change a
  type `EX-10` enumerates, for no observable difference. Recorded here so the
  temporary default does not become permanent by silence.
