# ISS-508: check quick reprints the full standing warning set with no delta

## What

`doctrine check quick` reprints the repo's entire standing warning set on every run
(one capture: 45 findings + 1085 raw-label warnings) and exits 0. The one finding an
agent's change introduced is buried; there is no way to see only the delta. `check gate`
output likewise buries the real error and its exit status.

## Fix sketch

Baseline the standing set (or diff against the last run / merge-base) and print new
findings first, with the standing count summarised in one line. Pairs with IMP-396
(verdict line must survive a tail).

## Evidence

7 observations, 2026-07-31 → 2026-09-27: `019fb70a` (one clippy error under ~700
warnings), `01a0b9ce`, `01a0d78a`, `01a0dd28`, `01a0e2d0`, `01a0e4d6`.
