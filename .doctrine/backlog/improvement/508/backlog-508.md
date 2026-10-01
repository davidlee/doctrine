# IMP-508: Link refusal names target kinds but not the working label

## What

When `doctrine link` refuses a label/target combination, it names the legal target kinds
but not the label (and `--role`) that *would* work for the pair the agent asked for. The
legal label+role set is discoverable only by failing, typically two refusals per link.
Scaffold comments make it worse by advertising labels the CLI rejects.

## Fix sketch

On refusal, compute and print the admissible `label [--role …]` options for the given
source kind → target kind. Related: ISS-236 (duplicated role-bearing labels in the
error), IMP-480 (RV as a target).

## Evidence

8 observations, 2026-07-29 → 2026-09-27: `01a08e09` (SL→QUE), `01a09e65`, `01a0b88b`
(backlog→RFC), `01a0d906` (scaffold names a refused label), `01a0e262`.
