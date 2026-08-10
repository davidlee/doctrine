# ISS-334: Row B5 sibling tests are load-fragile

Four sibling tests around row B5 fail under heavy CPU contention. The mechanism
they test is correct; the *fixture window* is too narrow to survive load.

## Observed

Under 24 spinning cores the gate failed 2 of 2 runs — never on the test `F-34`
repaired, always on its siblings:

- `concurrent_capsules_cannot_signal_…`
- `concurrent_capsules_cannot_see_…`
- `control_with_the_pid_namespace_shared_…`
- `the_sweep_reaches_what_row_b5s_control_leaks`

All four with `Indeterminate { arm: Probe, detail: NoObservation }`.

Calibration: **unloaded 5/5 green · 8 spinners green · 24 spinners 0/2.** Latent,
not active.

## Diagnosis

That verdict is the harness being **honest**. `SUBJECT_LINGER_SECONDS = 3` — the
subject capsule closed before the observer capsule finished looking at it, so
there was genuinely nothing to observe. `Indeterminate` is the correct reading of
that state; the row declined to claim a result it had not measured.

## Why `SL-248` left it alone

Deliberately, and the cheap fixes are all worse:

- **Widening the linger** taxes every concurrent row and collides with
  `FIXTURE_TIMEOUT_SECONDS`, which `F-28` cut 120 → 30 for exactly this reason.
- **Retrying on `NoObservation`** folds fixture flakiness into the row algebra,
  which is where it must never live.

## The generalisation worth keeping

From the same phase (`notes.md` item 105): **where a test reads a live process, a
`/proc` entry, a window or a clock, the unit of evidence is a tally under load,
not an exit code.** Both this and `F-34` were invisible to one green run and
visible within minutes of repetition under contention. This one is still open
precisely because the cheap evidence never showed it.

## References

`SL-248` `PHASE-09` phase-sheet finding `F-35` · `notes.md` § *Owed* item 104 ·
`RV-352`
