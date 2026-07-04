# IMP-260: slice phase show — read-only per-phase criteria view (EX/VT)

## Problem

There is no verb to display a single phase's authored criteria (EX-nn, VT-nn,
VA-nn, VH-nn). Agents at every lifecycle stage are forced to grep raw
`plan.toml` for the phase's block — violating the "read via show, not raw files"
guardrail and risking mis-parses when criteria strings contain phase ids from
other phases.

Witnessed 3+ times:
- SL-191-plan-a: verifying just-authored plan.toml required Reading the raw file
  because `slice show` has no plan projection
- SL-195-P03-a: locating one phase's EX/VT before execute forced `grep plan.toml`
- SL-198-drive-01: `sed -n '/PHASE-01/,/PHASE-02/p'` badly mis-parsed when
  PHASE-01's objective text contained "PHASE-02" — stitched PHASE-02 criteria
  under PHASE-01's heading

The raw-file parse risk is not academic: `plan.toml` criteria strings can
contain phase references, and sed range patterns don't understand TOML
structure.

## Fix direction

- **`doctrine slice phase show <ID> <PHASE-NN>`**: reads plan.toml, extracts
  the named phase's authored criteria block, renders it as a table (objective,
  EN-nn, EX-nn, VT-nn, VA-nn, VH-nn with their structured fields).
- **Or extend `slice show`**: add a `--plan` flag that renders the authored
  plan alongside the scope. Similar to how `slice show` synthesizes scope +
  metadata. The per-phase view is also useful on its own for phase-planning and
  execution contexts.
- Pure read-only; does not touch runtime phase sheets.

## Related

- RFC-011 case-notes: SL-191-plan-a, SL-195-P03-a, SL-198-drive-01
- IMP-209 (structured VT mandates — the plan-side fix that gives criteria
  parseable shape)
