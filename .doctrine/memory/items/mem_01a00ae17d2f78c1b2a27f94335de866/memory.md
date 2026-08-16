# Suspect your transcription before you suspect the tool

`grep`, `rg`, `sed` and `awk` are deterministic. An agent's re-rendering of their
output — into a finding, a design claim, a memory — is not. When those two
disagree, the tool is not the likely defendant.

## The case this was learned from

A high-trust, **critical**-severity memory
(`mem.fact.rtk.output-filter-rewrites-identifiers`, now superseded by this one)
recorded two sightings on 2026-08-16:

- an identifier **substituted** in `rg` output — `fn status_and_title_for`
  rendering as `fn status_and_n`, `tangle_baseline` as `ln`;
- the **same `grep -n`** over an unchanged file returning `1081/1086/1091` early
  in a session and `1094/1099/1104` later.

Both were blamed on an `rtk` output-filter proxy that the harness was said to
rewrite bare `git`/`rg` through.

**`rtk` was real, and was removed from this environment months before either
sighting.** Verified 2026-08-17: not on `PATH` in the jail; the global
`PreToolUse` hook array is empty; the project's only `Bash`-matched hook runs
`doctrine memory surface`, which injects context and does not rewrite commands.

The case was then re-run. `grep -n`, `rg -n` and an independent `awk NR` over
`src/design_run/tests.rs` all returned **1094/1099/1104** — the value the memory
itself recorded as the correct one. Every line number used to plan `SL-238`
PHASE-03 also matched the `Read` tool on both tools.

So the observation was probably an agent misreading its own output mid-session,
and the proxy was a ready-made culprit sitting in context.

## The failure mode worth naming

**A documented mechanism in context turns a misread into a diagnosis.** Once
"the proxy mangles output" is available, a surprising result stops being
investigated and starts being *explained*. The attribution then hardens into a
high-severity memory, and the next agent's transcription error is filed under
the same heading instead of caught. A corpus that absorbs errors this way gets
worse the more it is used.

The tell is an explanation that requires a mechanism nobody has checked is
present. Check that it is present before you name it.

## What to do

- **Confirm before asserting.** Any identifier, signature, or line number about
  to enter a finding, a design claim, or a memory gets confirmed with the `Read`
  tool or a single-line `sed -n 'Np'`. Locate with grep; quote from `Read`.
  Cheap, and sound regardless of what caused any given surprise.
- **Read the file for a range.** Do not reconstruct a multi-line span from
  `sed -n 'A,Bp'` output you are then going to quote.
- **A negative grep still needs a positive control** — a search that must return
  hits, so an empty result is a demonstrated absence rather than a broken query.
  That rule stands on its own footing:
  [[mem.pattern.harness.grep-negative-needs-positive-control]].
- **Do not blame a tool in a durable record without evidence it was involved.**
  Name what you ran, what you expected, and what you saw. "I may have misread"
  is a legitimate and usually correct entry.
