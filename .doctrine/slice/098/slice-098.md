# SL-098: Requirements discovery and home-finding

Skill-level integration: make implied-requirement discovery and canonical
requirement placement a natural part of every pass through the design→plan→audit→reconcile→close loop.

No new code — the entity machinery already supports the operations. This slice
edits the affected skills (§8 of the design): `/design`, `/plan`, `/audit`,
`/reconcile`, `/close`. The `/spec-product` and `/spec-tech` skills are upstream
of the reconcile loop and are deferred to IMP-096.
