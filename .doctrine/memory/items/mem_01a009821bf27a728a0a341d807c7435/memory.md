`doctrine coverage record --slice N --requirement REQ-NNN --change N --mode VT`
with **no** `--alias`, `--command`, `--extra-args`, `--matcher-source` or
`--matcher-pattern` does **not** record a VT check. It records a `VA`/`VH`-shaped
attestation that merely *stores* the string `VT` in its mode field.

The mechanism, in `src/coverage_store.rs`:

- `CoverageRecordArgs::has_check` (`:289-300`) is false when every check field is
  absent, so `run_record` passes `check = None` (`:363-370`).
- `record` derives `is_vt` from `check.is_some()` (`:132-139`) — **not** from
  `--mode`. With no check it takes the attestation branch: the default
  `CoverageStatus::Verified` and today's date.
- `coverage_verify` (`:125-135`) then treats a check-less entry as backfill and
  never runs anything.

Net effect: a cell that reads as **verified VT with no test bound to it**. The
mode flag alone is a claim, not evidence.

Always supply a runnable check plus a matcher, and make the matcher a **positive
control** — one that fails if the tests never ran. A `test result: ok` pattern
does not qualify: it also matches a run in which the new checks were never
compiled. Name a specific test instead:

```
--command cargo --command test --command --test --command <target> \
--matcher-source stdout --matcher-pattern '<test_name> \.\.\. ok' --regex
```

Found by an external adversarial reviewer on `RV-360` (`F-3`) against `SL-256`'s
design, where the bad recipe had been written into both the design and the
slice's closure intent. See [[mem.signpost.doctrine.requirements]].
