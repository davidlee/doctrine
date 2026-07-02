# Implementation Plan SL-181: OQ-D reframe: confinement is the close (REV-only)

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

One phase. The slice's sole deliverable is a **Revision** against ADR-012 (and the
ADR-006 D2a/D2b notes) that makes the OQ-D framing honest: it retracts "the
positive coordination marker is the real close" and records the genuine close as
**shipped confinement** (SL-182/183/185). There is **no code** (design §3) — the
guard was retired (design §1; RV-199 F-1 upheld). So the plan is not build-work; it
is a single authoring act against the governance corpus.

## Sequencing & Rationale

**Why one phase.** The deliverable is atomic: a single REV whose change rows and
rationale specify two prose amendments (ADR-012, ADR-006). There is no code to
stage, no test to grow, no dependency ordering — splitting would manufacture
phases without work in them.

**Where the phase boundary sits — author/approve here, apply at reconcile.** The
reconcile skill is the sanctioned governance write surface ("REV for
governance/spec"). So PHASE-01 stops at an **approved-but-unapplied** REV: `revision
new` → `revision change add` (the `revises` rows for ADR-012 + ADR-006) → author the
rationale → `revision approve`. `revision apply` — which edits the ADR prose — is a
**reconcile** action, alongside the two closure effects the REV entails (close
IMP-065 as reframed/superseded; downgrade RSK-014 to "enforcement-closed on confined
arms; bounded residual"). This honours design D3 (REV authored at execute) and D4
(IMP-065/RSK-014 dispositions are the REV's effects, landed at reconcile) without
smearing the governance write across execute.

**First REV in the corpus.** `revision list` is empty — this is the corpus's first
Revision. The flow is exercised end-to-end here; watch for skeleton/change-row shape
surprises and note anything durable to memory.

**Verification is VA/VH, not VT.** The deliverable is governance prose; no unit test
can judge "the retraction is honest and complete." VA-1/VA-2 have an agent inspect
the authored REV (`revision show`) and confirm target set, approval state, and the
empty `src/` diff. VH-1 records the human acceptance the owner-locked ADR-006 D2a/D2b
amendment requires (VH) — the honesty gate before reconcile applies it.

## Notes

- The one load-bearing premise the REV asserts — "where a confinement backend runs,
  worker ref-mutation is blocked at the ro-`.git` floor" — was re-confirmed at plan
  time (SL-182/183/185 `done`; `pretooluse-wrap.sh` fail-closed present). If a fresh
  inquisition is wanted, that assertion is the one to push on.
- The retired guard design is preserved in git history (`244d7bc4`) and summarised
  in design §6 — do not resurrect it without defeating the §1 argument.
