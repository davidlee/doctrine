# Doctrine requirements and reconciliation

Requirements are verifiable statements that descend from a specification.
Doctrine tracks their coverage — which code or tests satisfy them — and
reconciles their status against observed evidence.

## CLI

The CLI is the source of truth: `doctrine coverage --help` and
`doctrine reconcile --help`.

## What's shipped

- **Coverage store** — the observed tier: record evidence (a test function, a
  code region) that satisfies a requirement; verify attests it; forget removes
  stale evidence. The show subcommand displays the drift view (REQ status vs
  observed coverage).
- **Reconcile** — the sole author of reconciled requirement status:
  `doctrine reconcile <REQ> --slice <SLICE> --move <MOVE> [--to <STATUS>]`.
  Writes exactly one move and emits an atomic REC record.

## What's not yet widely dogfooded

VT verification (formal verification-test authoring) is shipped and stable,
but not yet widely applied across the project's own requirements.
The `doctrine coverage` surface is operational; the project's REQ corpus is
still being backfilled. Be honest about this when assessing coverage in your
own project.

See [[signpost.doctrine.specs]] for the spec hierarchy,
[[signpost.doctrine.audit]] for the audit phase where coverage is reviewed,
and [[fact.doctrine.cli-source-of-truth]] for the CLI.
