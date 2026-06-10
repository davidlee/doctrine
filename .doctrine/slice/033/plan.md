# Implementation Plan SL-033: Standard (STD) governance kind

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Two phases. PHASE-01 ships STD as a working capability — the thin data module,
templates, CLI, and install wiring — riding the SL-030 spine with no spine edit.
PHASE-02 pays back the boot debt SL-030 named: it collapses boot's per-kind
projection to one data-driven variant and lights up the Active Standards section.

The split is along a hard seam: PHASE-01 is **additive surface** (new kind, no
existing behaviour touched); PHASE-02 is a **behaviour-preserving refactor** of
shared machinery (boot) gated by existing tests. Keeping them apart means the
refactor lands against a tree where STD already exists and is tested, so the new
Active Standards row has a real kind to bind and a real projection to assert.

## Sequencing & Rationale

**Why the rider first.** STD is a near-verbatim mirror of `policy.rs` over an
unchanged spine, so PHASE-01 carries almost no design risk — it is pattern
application (the authored-entity-wiring + thin-data-module patterns). Building it
first gives PHASE-02 a concrete `STD_KIND` to bind in `boot_sequence` and a
populated standard tree to drive the new projection test. The phase is end-to-end
(CLI reachable, tree writable) so there is no module-built-ahead-of-consumers
dead-code window to suppress — the forwarders are wired the moment they exist.

**Why boot is its own phase.** The boot collapse is the only change that touches
shared machinery, so it carries the behaviour-preservation gate: the ADR and POL
section bytes must not move. Isolating it makes that gate legible — the diff is
boot.rs only, and the pre-existing boot suites are the proof, run unchanged. STD
forces the generalization the scope under-specified: it is the first kind with a
**two-element** in-force set (`default` + `required`), so the collapsed variant
must carry a status *set*, not a single literal. PHASE-02's new
in-force-set projection test is the only coverage that exercises that breadth;
the single-element ADR/POL sets are the byte-identity control.

**Ordering within boot.** Active Standards sits after Active Policies and before
Memory — governance kinds grouped, build-volatile ExecPath kept last (the
cache-warm-prefix rule the existing sequence already follows).

## Notes

- The boot variant name avoids `Governance`, which already binds the
  `governance.md` disk reader; the design uses `GovRows`.
- Inherited shared gaps (boot error≡empty marker collapse, supersession⇏status,
  inert `--tag`) are out of scope — STD inherits POL's parity, fixes none.
- At close: a fourth governance kind would now be pure data + a one-line
  `boot_sequence` addition (no code-shape change). Worth a durable memory.
