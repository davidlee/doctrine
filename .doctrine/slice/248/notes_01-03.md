# SL-248 — execution record, PHASE-01…PHASE-03

Per `LOOP.md` § *Notes, sharded*: what was done, what diverged, what was
measured. Durable-across-phases material belongs in `notes.md`, not here.

PHASE-01 and PHASE-02 predate the shard; their record is in `notes.md`.

---

## PHASE-03 — Host facts, the `[capsule]` table, and advisory capacity

### T1 — Baseline

- `git status --porcelain` on arrival: `M .claude/settings.json` only (the
  user's file — untouched, never committed).
- Branch `sl-248`, HEAD `ee562c9b9`.
- `just check` **green** on arrival (exit 0), including the
  `doctrine-control` unittest target (`running 0 tests`, which is the point:
  the crate is in the checked set and this phase gives it something to run).
- `cargo tree -p doctrine-control --depth 1` on arrival — the A2 baseline:

  ```
  doctrine-control v0.1.0 (/workspace/doctrine-SL-248/crates/doctrine-control)
  └── doctrine v0.37.0 (/workspace/doctrine-SL-248)
  ```

  One edge, as PHASE-01 `EX-6` left it. A2 predicts the three edges this phase
  adds bring **zero new packages** into `Cargo.lock`; re-measured at T13.

### T2 — `host.rs`: the four-method contract, `SystemHost`, the fixture

- `crates/doctrine-control/src/host.rs` new; `main.rs` gains `mod host;`;
  manifest gains `rustix = { version = "1", default-features = false,
  features = ["fs", "std"] }`.
- `D1` applied: `HostFacts::available_bytes -> Result<u64, CapacityUnknown>`.
  The `ByteCount` conversion is the caller's, which keeps `host` out=0.
- `D2` applied: `CapacityUnknown` is defined in `host.rs`.
- `rustix 1.1.4`'s `StatVfs` fields are all `u64`
  (`backend/linux_raw/fs/types.rs:815-827`), so `f_bavail × f_frsize` is a
  plain `u64::checked_mul` — no `as`, no `TryFrom`. `Errno::raw_os_error()`
  gives the `i32` `ProbeFailed { errno }` carries.
- The fixture is `#[cfg(test)] pub(crate) mod fixture` with a `FixtureHost`
  builder (`with_env` / `with_available` / `with_resolution`). `BTreeMap` +
  `BTreeSet`, never the hashed pair (`clippy.toml` `disallowed-types`).
  Its **empty** default is the interesting one: no environment at all is the
  case root resolution must refuse rather than guess.
- `SystemHost` gets a deliberately weak smoke test — it constructs the type and
  calls all four. `sec-5`'s two Table C rows (statvfs agreement within one
  allocation unit; the cross-filesystem claim) are **PHASE-08's**, and this test
  does not reach for either. It exists because `SystemHost` is otherwise never
  constructed under `cfg(test)` and `dead_code` would fire.
- `D5` suppression: module-level
  `#![cfg_attr(not(test), expect(dead_code, reason = …))]`. First spelling held
  — no `unfulfilled_lint_expectations`.
- Measured: `cargo test -p doctrine-control` 3/3 green; `cargo clippy -p
  doctrine-control` clean; `cargo test --test architecture_layering` 25/25
  green **before** the `layering.toml` rows land (see T12 — that is the `F-5`
  gap, not a licence to omit the rows).
- **A2 confirmed for `rustix`**: `Cargo.lock`'s only change is `"rustix"` added
  to the existing `doctrine-control` package's dependency list. No new
  `[[package]]` stanza. `S3` does not fire.

### T3–T7 — `config.rs`

- Vocabulary as `sec-5` sketches it, with `D4` applied throughout: every item is
  `pub(crate)`, never `pub`. The crate is bin-only, nothing crosses its
  boundary, and `pub` would have cost `unreachable_pub` (PHASE-01's measured
  trap) for nothing.
- `Argv` lands here (`F-2` — the design uses it and never defines it).
- `ConfigRefusal` carries the ten variants `EX-11` names, with the exact
  payloads it gives. Three of the four fieldless variants nonetheless have to
  *name* keys, because `VT-4`'s titles say so
  (`closure_roots_without_a_resolver_refuses_naming_both_keys`,
  `neither_variable_usable_refuses_naming_the_config_key`). Resolved with
  `ConfigRefusal::keys() -> &[&'static str]` — a **structured** accessor
  returning the `KEY_*` constants a refusal is about, not a formatted sentence.
  This keeps `EX-11`'s variant shapes exactly as written while giving the
  "naming" tests something real to assert. `MalformedTable` returns none: serde
  has already named the offending key inside `detail`, and this reader does not
  re-parse its own error text.
- Serde shape is `src/reserve.rs:78-103`'s: tolerant outer `CapsuleDoc`
  projecting one table, inner `RawCapsule` with
  `#[serde(rename_all = "kebab-case", deny_unknown_fields, default)]`. No
  `#[serde(flatten)]` anywhere. Container-level `default` is what makes the
  `Vec` fields optional; `Option<T>` fields default to `None` without it.
  Tolerance is asserted inside
  `absent_optional_keys_take_the_named_default_constants`, which feeds a
  `[dispatch]` table alongside `[capsule]` — one test, two properties, no extra
  title outside the `VT` set.
- Validation order, which the tests depend on: configured-root absoluteness →
  readable lists → resolver rules → `execution-timeout-seconds` →
  `file-size-cap-mib` → `execution-kill-grace-seconds` →
  `expected-capsule-size-mib` → multiplier floor. Root first is what lets
  `relative_configured_root_refuses` be a one-key fixture.
- `seconds_bound` / `mib_bound` share one convention: `default: None` **is**
  "required", and zero refuses whether the value was configured or defaulted.
  Two four-line functions, each used twice, instead of four.
- `D6` observed: no `/` on integers anywhere. The test-side conversion ceiling
  is `u64::MAX >> BYTES_PER_MIB.trailing_zeros()` — derived from the constant
  rather than restated, exact because a mebibyte is a power of two, and it
  needs **no** `#[expect(clippy::integer_division)]` at all, so `D6`'s escape
  hatch went unused.
- Root resolution runs entirely through the fixture — no process env mutation.
  `no_resolution_path_yields_a_relative_root` is the property over all 16
  combinations and additionally pins the split: **7 resolve, 9 refuse**
  (an absolute value in either variable resolves: 4 + 4 − 1).
- Divergence in *form*, not substance, from `EX-17`: `serde` and `toml` are
  declared `{ workspace = true }` rather than spelled out. `[workspace.dependencies]`
  already carries `serde = { version = "1", features = ["derive"] }` and
  `toml = "0.8"`, so the resolved dependency is exactly what `EX-17` names, and
  inheritance is the root package's own convention (`toml.workspace = true`).
  `rustix` is spelled literally because it is not a workspace dependency — the
  root package spells it inline too.

### T8–T10 — `capacity.rs`

- `assess_capacity(probe, &CapacityPolicy) -> CapacityVerdict` is pure and
  **rootless**, exactly as `EX-14` fixes it.
- `D3` applied: `CapacityReport` is a four-variant enum minted by
  `CapacityReport::of(verdict, capsule_root)`. The variants carry `EX-16`'s five
  field names *literally* — `available_bytes`, `expected_bytes`,
  `threshold_bytes`, `capsule_root`, `key` — rather than a struct of `Option`s,
  so `low_warns_with_named_fields_and_provisioning_continues` asserts the named
  fields and not a rendering. `refuses()` is true for `Refuse` alone: an
  unusable probe is reported and provisioning continues.
- The key on every report is `KEY_EXPECTED_CAPSULE_SIZE_MIB` — a warning and a
  refusal about the same condition name the same lever, which is `EX-16`'s
  "differ in severity and not in vocabulary".
- Saturation vs refusal both asserted, and they are genuinely different code
  paths: `ByteCount::saturating_mul_u32` for the threshold, `from_mib`'s
  `checked_mul` for the conversion.
- `as` casts are banned (`clippy::as_conversions`), and `u64::from` is not
  const-stable, so the test's `EXPECTED × MULTIPLIER` is a small `fn`, not a
  `const`.

### T11 — layering rows and module declarations

- `[doctrine_control_tiers]` gains `host` / `config` / `capacity`, all `leaf`,
  in the section's existing comment style. Appended to the **second** section
  (`layering.toml:257`), never the root `[tiers]`.
- `main.rs` declares all three modules. `F-4` stands: the plan's file-ownership
  table does not list `main.rs` for PHASE-03, but without these three lines
  nothing compiles and `EN-1`'s ≈33 tests never run.
- **Measured: 33 tests, all green** — `EN-1` predicted ≈33.
- `cargo clippy -p doctrine-control` clean. One pedantic hit on the way:
  `doc_markdown` on the bare word `snake_case` in a module doc comment.
- **A2 fully confirmed**: `Cargo.lock`'s entire diff for this phase is three
  lines — `rustix`, `serde`, `toml` added to the existing `doctrine-control`
  package's dependency list. **Zero** new `[[package]]` stanzas. `S3` does not
  fire.

### Harvest — the `git checkout` footgun, and a corpus fix

T12's delete-and-observe probe restored `layering.toml` with `git checkout --`,
which resets to **HEAD** — and the three rows under test were uncommitted, so
the first iteration destroyed all three and iterations 2 and 3 silently measured
a file with no rows at all (6 `Unclassified` instead of a per-row signal).

This repo **already knew**: `mem.pattern.git.revert-control-by-edit-not-checkout`
(SL-244 PHASE-06, trust high) is exactly this pattern, observed twice there.
It did not surface here because its `scope.paths` was `src/** install/**
publication/**` and the file at issue is `.doctrine/adr/001/layering.toml`.

- Widened that memory's `scope.paths` to include `crates/**` and `.doctrine/**`.
- The duplicate I recorded before finding it
  (`mem.pattern.git.probe-restore-not-checkout`) is marked **superseded by**
  the incumbent rather than left as corpus noise.
- Friction observation recorded (`019fe0f6-6964-7bc0-a150-2300e5ac4512`).
- Method for the T12 redo: snapshot to the scratchpad, restore with `cp`.

### T12 — the three rows bite (`EN-3`, `F-5`)

Method: snapshot `layering.toml` to the scratchpad, delete one row, run
`cargo test --test architecture_layering`, restore with `cp`. **Not**
`git checkout` — see § *Harvest* above for why the first attempt lost the rows.

| row deleted | verdict | violations raised |
|---|---|---|
| `host` | RED — `the_layering_gate_runs_over_both_source_trees` FAILED | `Unclassified("host")` ×2 |
| `config` | RED — same test FAILED | `Unclassified("config")` ×2 |
| `capacity` | RED — same test FAILED | `Unclassified("capacity")` ×2 |

Restored: 25/25 green, `git diff` on the file empty.

**Each row reds twice, and the doubling is `F-5`'s mechanism made visible.** The
completeness assertion is keyed on *edge endpoints*, not on units: `host`
appears in `config → host` and `capacity → host`; `config` in `config → host`
and `capacity → config`; `capacity` in `capacity → config` and
`capacity → host`. Two edges each, two violations each.

So `EN-3`'s "an unclassified unit fails the gate, so the appends are
self-enforcing" holds **for these three units** and for the reason `F-5` gives —
they all sit in edges — not for the reason the criterion states. Measured
directly at T2, before the rows landed: the gate was **green** over a tree
containing an edge-free `host` unit with no row. A unit with out=0 that nothing
imports is still invisible to this gate. That is `ISS-326` reaching this phase,
exactly as `F-5` predicted, and it is not a licence to omit a row.

### T13 — the manifest derived from `use` statements (`VA-2`, `VA-5`, `EX-17`)

Derivation performed by grepping the three new files for their own imports and
inline paths, **not** by copying `sec-6`'s table.

| file | non-`std`, non-`crate` paths |
|---|---|
| `host.rs` | `rustix::fs::statvfs` (`:93`) |
| `config.rs` | `serde::Deserialize` (`:47`), `toml::from_str` (`:404`) |
| `capacity.rs` | none — `std` + `crate::{config,host}` only |
| `main.rs` | none |

**Derived list: `rustix` (features `fs`, `std`), `serde` (derive), `toml`.**
No `doctrine::` path is named by any of the three — this phase reaches nothing
in the root package, so PHASE-01's export set is untouched here.

- **Agrees with `sec-6`'s table** on the three edges. One difference in *form*
  only (`F-9`): `serde`/`toml` are `{ workspace = true }` because the workspace
  already declares exactly the versions/features `EX-17` names; `rustix` is
  spelled inline because it is not a workspace dependency.
- `capacity.rs` needs no `rustix` even though `sec-6`'s table attributes the
  `statvfs` call to `capacity`. It does not disagree about the *edge* — `D1`/`D2`
  put the probe in `host`, which the table already lists — so this is a note,
  not a staleness report.
- **`rustix::process` absent**: `grep -rn 'rustix::process|rustix::thread|Uid|Gid|getpid|kill('` over the whole crate returns NONE. It is behind an
  undeclared feature, so a call would be `E0433` by design (`RV-346` `F-29`).
- `Cargo.lock`: **zero** new `[[package]]` stanzas across the whole phase
  (`git diff Cargo.lock` = three names added to the member's dependency list).
  A2 holds; `S3` does not fire.

### T14 — `.doctrine/doctrine.toml` gains `[capsule]` (`EX-19`, `VH-1`)

`VH-1` is **DISCHARGED**, not owed: the slice owner ruled the figures on
2026-08-08 and the sheet's `F-7` carries the ruling. `T14`'s
"mark `VH-1` owed" clause is superseded by `F-7` and was not executed.

Written exactly as `F-7` gives them:

```toml
execution-timeout-seconds    = 900
file-size-cap-mib            = 512
expected-capsule-size-mib    = 8192
capacity-warn-multiplier     = 2
execution-kill-grace-seconds = 5
```

plus a provisional `readable-roots = ["/bin/sh", "/usr/bin/env"]` with a comment
naming PHASE-06 as its first consumer, and a comment recording the owner's
scoping of 900 to the **build/verification** workload — the separate
agent-execution bound is `notes.md` § *Owed* item 16 and **not** a key this
phase may add (`EX-19` fixes the key set; adding one would be `S4`).

Validation method: **a scratch test, not a reading.** A temporary
`#[test] fn scratch_the_repository_capsule_table_round_trips` in `config.rs`
pulled the file in with `include_str!("../../../.doctrine/doctrine.toml")`, ran
it through `parse_capsule_config`, and asserted all five figures plus the
readable list, the empty closure list and the absent resolver. **Passed.** The
test was then deleted; the suite is back to 33.

### T15 — the inspection criteria (`VA-1`, `VA-3`)

**`VA-1` — no clock in `host.rs`.** The sheet's suggested grep
(`grep -nE 'now|Instant|SystemTime|crate::clock' host.rs`) **cannot** be empty:
the bare alternative `now` is a substring of `CapacityUnknown`, which `EX-14`
requires and `D2` places in this file. Tightened to word boundaries and with
comment lines stripped:

```
grep -vE '^\s*(//|/\*|\*)' host.rs |
  grep -nE '\bnow\b|now\(|\bInstant\b|\bSystemTime\b|crate::clock|std::time|Duration'
→ EMPTY
```

The only match before stripping comments is `host.rs:13`, a doc line saying a
`now()` here *would be a second one*. `host.rs` imports no `std::time` at all.
`VA-1` holds. (Recorded as `F-10` — the criterion is satisfiable as written; the
task's suggested method is not, and the next reader will hit the same false
positive.)

**`VA-3` — `REQ-461`'s four negatives are structural.** Confirmed by inspection,
each separately:

1. *No reserved figure on `CapacityPolicy`* — the struct is two fields,
   `expected_capsule_size: ByteCount` and `warn_multiplier: u32`. Nothing else.
2. *`assess_capacity` admits no capsule count or queue* — its signature is
   `(probe: Result<ByteCount, CapacityUnknown>, policy: &CapacityPolicy) ->
   CapacityVerdict`. Neither parameter can express how many capsules exist, and
   the function has no other input.
3. *No removal call anywhere in the phase's files* —
   `grep -E 'remove_file|remove_dir|remove_dir_all|std::fs::(copy|rename|write)|unlink|truncate'`
   over `host.rs`, `config.rs`, `capacity.rs` with comment lines stripped
   returns NONE. There is no delete capability to misuse.
4. *Nothing copied or compressed on any refusal path* — same grep covers
   `copy`/`rename`/`tar`/`zip`/`flate`/`compress`: NONE. Every refusal path in
   this phase returns a typed value and performs no IO whatsoever.

The only surviving occurrences of "reserved" in the crate are two doc comments:
`host.rs:87` (`f_bfree` includes the reserved blocks a capsule cannot have) and
`capacity.rs:115` (nothing is reserved at `Low`).

### T16 — green, and the VT mapping

- `doctrine check gate` — **green, exit 0**. (`check`/`gate` build before they
  validate, which is what gives the corpus check a fresh binary.) 33
  `doctrine-control` tests among them.
- `doctrine slice verify-vt 248` — PHASE-03's **eight `VT` keyword mandates are
  all satisfied**: every mandated title is present in its mandated file. All
  eight report `UNATTRIBUTABLE`, not `PASS`, with the same reason — *"keyword
  present but `<file>` not modified by this slice"*. That is the slice's
  recorded source-delta boundary, not a test problem: `record-delta` is the
  **orchestrator's** surface (`LOOP.md` § *Writer map*) and has not been run for
  this phase's commits yet. Flagged in the hand-back.
- The `FAIL` rows in the same output are later phases' — `backend.rs`,
  `backend/bubblewrap.rs`, `provision.rs` do not exist yet. Expected, not ours.
- PHASE-01/02's `VT`s all still `PASS`: nothing this phase did moved them.

#### Divergences from the sheet, collected

1. **`ConfigRefusal::keys()`** — an accessor `EX-11` does not name, added
   because three `VT-4`/`VT-7` titles require refusals to *name keys* that
   `EX-11` gives no field to carry. Variant shapes unchanged. Sheet `F-8`.
2. **`serde`/`toml` declared `{ workspace = true }`** rather than spelled out as
   `EX-17` writes them; the resolved dependency is identical. Sheet `F-9`.
3. **`T15`'s suggested `VA-1` grep is broken** — bare `now` matches
   `CapacityUnknown`. The criterion holds; the method needed word boundaries.
   Sheet `F-10`.
4. **`D6`'s `#[expect(clippy::integer_division)]` escape hatch went unused** —
   the ceiling is expressible as a shift derived from `BYTES_PER_MIB`.
5. **`T14`'s "mark `VH-1` owed" clause not executed** — superseded by `F-7`,
   which discharges `VH-1`. Deliberate, per the brief.

Nothing in this list is an `S1`–`S5` stop. None of the twenty `EX` and none of
the eight `VT` was adjusted.
