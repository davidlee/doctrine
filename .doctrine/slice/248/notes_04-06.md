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
