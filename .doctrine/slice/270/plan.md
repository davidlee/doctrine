# Implementation Plan SL-270: Routing trial two

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See glossary.md § reference forms. -->

## Overview

Two code phases deliver design sec-2 (the six-route set, `DEC-330`, `DEC-333`)
and sec-3 (the route-presence check at the design-run lock, `DEC-326`). The
protocol (`DEC-334`) is already authored and frozen; the window itself is
`CHR-082`. `REV-065` is applied at reconciliation, not in a phase.

```text
PHASE-01 six-route set ──► PHASE-02 lock check ──► audit ─► reconcile (REV-065) ─► close
 vocab, CLI/MCP text,       unrouted_severe,
 goldens, reviewing.md,     Cause, remedy,
 review-ledger.md, memory   fixtures, closing para
```

## Sequencing & Rationale

- **Route set first.** The lock check reports `owner-fix` as an unknown route;
  its legacy case (PHASE-02 `VT-2`) only holds once PHASE-01 has removed
  `OwnerFix`. The reverse order would test against a vocabulary about to change.
- **Docs travel with the code that makes them true.** The route table and
  counts change in PHASE-01 with the vocabulary; the closing paragraph that
  names the lock check as the route's one reader lands in PHASE-02 with the
  check.
- **Verification ids keep the design's numbers** (sec-5): PHASE-01 holds `VT-1`,
  `VT-6`, `VA-1`; PHASE-02 holds `VT-2`..`VT-5`. The gaps are deliberate, so a
  criterion reads the same in the design and the plan.

## Routed findings (RV-400)

The design review `RV-400` carries two `control`-routed findings (the route is
stated in each response; the ledger predates the `route` field's use here):

- `RV-400` `F-7` — the predicate must not list a contested route-less blocker in
  both the blocker list and the route list. Transcribed into PHASE-02 `VT-2`.
- `RV-400` `F-5` — the build-floor check must reject an MCP server served by a
  stale five-route binary. Its host is `CHR-082`'s window-open step, not a
  phase here.

`RV-400` `F-1` (open majors, superseded passes and post-lock dispositions escape
the check) is an owner follow-up at audit.

## Notes

- **Fixture fallout (PHASE-02 `EX-4`).** Any e2e fixture that locks over a
  disposed severe finding with no route will start refusing. Known case:
  `defect_warning_printed_gate_unchanged` seeds a verified finding of unknown
  severity `crit` with no route; an unknown severity counts as severe, so the
  seed needs a route to keep isolating the warning. Sweep the others with a
  full `doctrine check gate` run rather than predicting them.
- **Reads stay open.** `Route` is parsed only on write; a legacy `owner-fix`
  ledger still reads and renders. No migration.
- **Install at close.** `DEC-334`'s build floor needs the landed binary,
  MCP server and projections current; `doctrine install` runs at close, on the
  landed tree, not in a phase.
- **Research advisory.** `doctrine slice research 270` reports drift only in
  `slice-270.md` and the added `design.md` — both authored after research, no
  code premise moved. The design's code references were re-grepped against the
  tree at plan time and all resolve.
