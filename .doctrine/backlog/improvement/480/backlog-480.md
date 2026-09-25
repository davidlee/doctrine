# IMP-480: Allow an RV as a cross-kind relation target (finding-born provenance)

`doctrine link ISS-NNN references RV-NNN --role originates_from` is refused:
*"references target must be one of [ISS, IMP, CHR, RSK, IDE, SL]"*. An adversarial
review is a first-class kind and a legitimate origin for work intake, so a
finding-born backlog item cannot record the review that raised it; provenance
survives only as prose (obs `019feebf`, 2026-09-15; obs `/code-review` same class).

Fix: admit `RV` (and `REC`) as relation targets, or state why not. `IMP-433` is the
target-side sibling. See RFC-032 `research.md` F9.
