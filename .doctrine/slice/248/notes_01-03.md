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
