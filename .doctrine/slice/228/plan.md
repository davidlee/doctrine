# Implementation Plan SL-228: Zero-rescue dispatch funnel

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Seven phases deliver design.md (post-RV-305, converged): move E first (the
read surface, then the write guard — the live ISS-234 hazard closes at
PHASE-02), then move A in three slices (the pure machine + sole writer, the
fork-binding protocol, the verb gates + verify + receipt matrix), then the
oracle + skill rewrite, then the OQ-5 memory-blind benchmark as terminal
acceptance.

## Sequencing & Rationale

**Why move E leads.** Design §12's ordering constraint: gates must never
refuse toward a verb that doesn't exist. Verify's forward-sync (PHASE-05)
leans on `tree_clean_untracked` (PHASE-01), and the guard (PHASE-02) is
self-contained — landing it early retires the checkout-import idioms and
shuts ISS-234 before any funnel machinery moves.

**Why the §12 sketch's "machine + record" phase is split in two
(PHASE-03/PHASE-04).** The fork-binding protocol (claim → bind → act, the
claim lock, gc's lock-gated sweep, worker_commit's retry-signature legs) was
the most contested surface across all three review rounds — three RV-304
contests plus RV-305's blocker with two more upheld contests. It gets its own
phase boundary so its crash-window tests (VT-1/VT-3/VT-4) land and stabilise
in isolation rather than riding a phase that also births the machine.
PHASE-03 also pulls the `run_suite` extraction forward (design §5 step 3):
PHASE-04's belt-gated self-record leg needs a status-returning gate runner,
so the extraction cannot wait for the verify phase.

**Why gates + verify + receipt share PHASE-05.** The conclude gate demands
verify evidence; landing gates without the verify verb would wedge every
in-flight dispatch at `imported`. Same-phase delivery keeps the "refusal
prescribes an existing verb" invariant inside move A itself. The receipt
matrix rides along because D2's `Option<Position>` third input is only
testable once positions land (R1's byte-identical `None` arm is pinned by
the existing suites staying green unchanged).

**Order within the tail.** The oracle (PHASE-06) is the read projection of
everything before it — `expected_next` exists from PHASE-03, but `next` can
only prescribe verbs that gate, so it waits for PHASE-05. The skill rewrite
rides the oracle phase because the prose it deletes is replaced *by* the
`next`-loop. The benchmark (PHASE-07) is terminal acceptance by scope
definition and needs the full verb-driven funnel.

**Parallelisation.** PHASE-01 and PHASE-02 are file-disjoint from PHASE-03
(git.rs/hook vs funnel_machine.rs/record) except for `src/dispatch.rs` CLI
wiring and the PHASE-03 dependency on nothing from move E — PHASE-03 may run
in parallel with PHASE-01/02 under dispatch if worker file-sets are kept
disjoint; PHASE-04+ are strictly ordered. Default remains serial.

## Notes

- **R1 (behaviour preservation)** is enforced as EX-4/VT-3 of PHASE-05: the
  `position = None` arm must leave existing suites green *unchanged*.
- **R2 (embedded-asset strip)** is EX-3/VA-1 of PHASE-02 at authoring time;
  `just nix-build` remains a host-side close gate, not per-commit.
- **Deferred by design** (§14, unchanged): subprocess-arm full gating
  (REQ-387 may stay `pending` at close), MCP mirror of `dispatch commit`,
  OQ-3/OQ-4/move-D tail, the ship-time sibling REV for the four active-REQ
  modifies. The OQ-6 retirement list rides PHASE-07.
- **VT keyword floors** are deliberately conservative where the design fixes
  vocabulary (STD-001 reason tokens, type names from §2/§6) and minimal
  where implementation naming is free (PHASE-04 VT-1 "claim", VT-4 "lock") —
  the phase-plan expands them once file shapes exist; expanding a mandate is
  an append, not a renumber.
