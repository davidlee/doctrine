# ISS-483: reserve VT-3 test is not env-isolated: the jail's DOCTRINE_RESERVATION_FALLBACK=1 turns check gate red

**Correction (2026-09-25, same day).** The ambient trigger was removed by the
operator: `flake.nix` no longer exports `DOCTRINE_RESERVATION_FALLBACK` (commit
`67df42ec8`). In this repo the variable was never load-bearing anyway — the
committed `.doctrine/doctrine.toml` sets `[reservation] reach = "local"` with
`allow-local-fallback = true`, so id-reserving verbs never contact a remote;
probed by creating a backlog item with the variable unset (succeeded). The gate
is therefore green in a fresh jail. **What remains** is the hermeticity defect
below: the suite reddens whenever the variable *is* set, and that variable is the
documented opt-in for `auto` plus a configured-but-unreachable remote. Impact is
downgraded from "red for every jailed agent" to "red for an operator who sets
the opt-in".

Observed while auditing SL-265 (`doctrine check gate` on the admitted candidate
surface, 2026-09-25): the suite is 4608 passed / 1 failed. The sole failure is
`reserve::tests::vt3_auto_degradation_is_fail_closed_with_explicit_optin`,
panicking at `src/reserve.rs:800` with *"auto + failing configured remote
hard-errors when fallback declined"*.

Root cause is ambient-env leakage, not the exercising slice. The test asserts
fail-closed **without** opt-in, while the NixOS/bubblewrap jail exports
`DOCTRINE_RESERVATION_FALLBACK=1` (required for entity creation in the jail —
`mem_019ef7d0cbb87061ac828c2c92978c5b`).

Evidence:

- `git diff main..edge -- src/reserve.rs` is empty — the file predates the slice,
  so the failure reproduces identically at the base `B = 2464bbfd6`.
- With the variable unset, `cargo test -p doctrine --bin doctrine reserve::tests`
  is **19/19 green** in the same tree.

Consequence: `doctrine check gate` — the documented close ritual — is red for
every agent working in the jail, and the dispatch verify beat's `check regression
diff` false-halts on it because the failure signature keeps the panic's **thread
pid**, so the same failure reads `changed` rather than `persistent` (observation
`01a0d82d-21cd-7b50-9b95-a3fe859ab25a`).

The same failure was recorded on RV-154 `F-4` (SL-150), where it was disposed
`aligned` as out-of-slice. Precedent for the fix: ISS-220 and ISS-260 were both
resolved by making the env-sensitive test hermetic.

Candidate fix: the `reserve` suite scrubs `DOCTRINE_RESERVATION_FALLBACK` (and any
sibling reservation env) for the duration of the test — an explicit env guard in
its `Substrate` — so the suite is independent of the ambient jail environment.
