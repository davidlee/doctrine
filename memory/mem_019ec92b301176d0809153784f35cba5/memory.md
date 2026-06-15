# Doctrine requirements and reconciliation

Requirements are verifiable statements that descend from a specification.
Doctrine tracks their coverage — which code or tests satisfy them — and
reconciles their status against observed evidence.

## What's shipped (as of 2026-06)

- **Coverage store** — the observed tier: `doctrine coverage record` captures
  evidence (a test function, a code region) that satisfies a requirement.
  `doctrine coverage verify` attests it. `doctrine coverage forget` removes
  stale evidence. The read-only `coverage show` displays the drift view
  (REQ status vs observed coverage).
- **Reconcile** — the sole author of reconciled requirement status:
  `doctrine reconcile REQ-NNN --to accept|revise|redesign`. Writes exactly
  one move and emits an atomic REC record.

## What's not yet widely dogfooded

VT verification (SL-057 — formal verification-test authoring) is shipped and
stable, but not yet widely applied across the project's own requirements.
The `doctrine coverage` surface is operational; the project's REQ corpus is
still being backfilled. Be honest about this when assessing coverage in your
own project.

See [[signpost.doctrine.specs]] for the spec hierarchy,
[[signpost.doctrine.audit]] for the audit phase where coverage is reviewed,
and [[fact.doctrine.cli-source-of-truth]] for the CLI.
