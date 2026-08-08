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
