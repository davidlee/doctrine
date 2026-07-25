# IMP-314: Research artefact has no harvest pointer at close

Surfaced by the SL-229 audit (RV-306 F-5).

## Problem

SL-229 D1 justifies storing the pre-design research artefact gitignored-in-place
at `.doctrine/slice/NNN/research/` partly on the ground that "end-of-slice
harvest is explicit". `/research` states the same ("Runtime tier: disposable
working evidence; durable findings are harvested at slice close").

Nothing implements it. The only skill masters that mention research at all are
`research`, `slice`, `design`, `plan`, `phase-plan` — the five SL-229 touched.
`/close`, `/harvest`, and `install/harvest.md` say nothing about it. The
artefact is invisible to `git status` (`.gitignore:48`), so it evaporates at
slice close without any signal.

Bounded, not catastrophic: `research.md`'s conclusions should already be cited
into `design.md`, so what is lost is the audit trail and the `raw/` thread
output, not the design rationale.

## Sketch

A one-to-three line advisory in `/harvest` (its sweep already enumerates
sinks) or `/close`: if `.doctrine/slice/NNN/research/` exists, sweep durable
findings into `notes.md` / memories before the artefact is discarded. Same D6
advisory phrasing as SL-229's four hooks. Note that shipping it needs the same
distribution step as RV-306 F-1.
