# CHR-002: Reconcile SPEC-002/PRD-013 own requirements: all still pending despite engine shipped (SL-042/SL-044)

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Every SPEC-002/PRD-013 requirement (REQ-108..116) still reads authored `pending`
while the engine (SL-042 observe substrate / SL-044 reconcile writer + closure
gate) ships — the canonical dogfood divergence between authored intent and shipped
reality.

As of SL-045 (reconciliation audit RV-005) the drift is now **observable**: the
read surfaces `doctrine coverage SPEC-002` (verdicts per requirement) and `doctrine
spec req list SPEC-002` (authored roster) ship. SL-045 was a surfacing slice and
deliberately did not reconcile the drift itself. Next step: run `doctrine coverage
SPEC-002`, then reconcile each REQ's authored status so authored truth matches the
shipped coverage. Closes the loop SL-045 opened.
