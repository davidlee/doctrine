# Implementation Plan SL-262: Envelope names the next move

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See glossary.md § reference forms. -->

## Overview

Four phases. A behaviour-preserving refactor opens the seam; the envelope
change lands whole; an end-to-end and bounds phase pins it through the binary;
a small guidance phase closes. REV and backlog bookkeeping (design sec-6) are
reconcile/close work, not phases.

```
PHASE-01 GateFacts + forward_unmet + watermark move   (refactor, suites unchanged)
   │
PHASE-02 Forward on the envelope, next_obligation out, v2, golden   (the change)
   │
   ├── PHASE-03 e2e forward suite + growing-run bound
   └── PHASE-04 hymn + design skill wording
```

## Sequencing & Rationale

- **PHASE-01 first, alone.** `GateFacts` (`DEC-292`) and the extracted
  `forward_unmet` are what make a read able to evaluate the gate at all. Doing
  it as a pure refactor lets the existing gate/apply/e2e suites prove behaviour
  preservation before any new behaviour exists. The watermark move rides here
  because it is the same kind of change (shell → core, no behaviour) and
  PHASE-02 needs it in the core.
- **PHASE-02 is one phase on purpose.** Adding `forward`, deleting
  `next_obligation`, the version bump and the byte-exact golden all move the
  same bytes (design sec-9 golden-churn risk). Splitting them would regenerate
  the golden twice and leave an intermediate version-1 envelope carrying both
  keys. The cause cap lives here, not in PHASE-01, because it has no consumer
  until `forward` exists (dead code under clippy). Unit tests for derivation,
  rendering, capping and the maximal-forward bound land with the code.
- **PHASE-03 after PHASE-02.** The e2e list in design sec-8 exercises the
  whole stack through the binary; it may expose disagreements in PHASE-02 code,
  which it fixes. It also carries the two probes left open by `RV-382` (the
  codex design review): batched submissions and the cause-cap constant's
  derivation against measured row sizes.
- **PHASE-04 last.** Guidance should name a field that exists. Independent of
  PHASE-03; either order works.

## Notes

- `RV-382` has no routed (`route:*`) findings — all five were fixed in the
  design, so no criteria are transcribed from it.
- Plan-time re-grep (2026-09-24): every path and symbol in design sec-2..sec-7
  resolves; line numbers have drifted slightly (e.g. `Runbook::section` is at
  `runbook.rs:503`, `forward_unmet`'s source loop at `gate.rs:1677`). No stale
  premise.
- PHASE-02 is the largest phase. If its phase sheet shows it will not fit one
  worker context, split the rendering (EX-5) from the derivation (EX-1..EX-3)
  by appending a phase, keeping the golden regeneration with whichever lands
  the key change.
