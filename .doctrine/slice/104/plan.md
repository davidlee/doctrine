# Implementation Plan SL-104: Estimate hardening — NF-001 tripwire + confidence legitimization

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SPEC-020, REQ-275, IMP-112); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

SL-104 is a deliberately narrow hardening pass (design.md §11, cut-set confirmed).
Two real deliverables plus one cheap test, split into two phases:

- **PHASE-01 — NF-001 tripwire.** The substantive work: a dependency-free, two-tier
  structural proof of REQ-275's non-blocking guarantee (allowlist source-scan +
  `Gate` compile-guard).
- **PHASE-02 — cleanup + value test.** The trivial polish: re-cite the stale
  dead-code reasons to the confidence residue's real consumer, and pin FR-008's
  untested value-asymmetry contract.

**Not in any phase — confidence governance lands at reconcile.** Per design.md D2,
the confidence `REQ` and the SPEC-020 amendment route through a Revision folded into
SL-104's reconcile (mirroring REV-002). PHASE-02 only touches the *code-side* residue
(the `expect` reason strings); the spec/REQ authoring is reconcile work, not
execution. This keeps the governance change on its sanctioned axis (ADR-013) rather
than smuggling a spec edit into a phase.

## Sequencing & Rationale

**Why two phases, in this order.** The two units are independent in substance but
unequal in weight. PHASE-01 carries the only non-trivial design (the allowlist vs
denylist choice, the Tier-2 type-confinement, the documented residual gap); PHASE-02
is mechanical. Isolating them keeps the meaty structural proof reviewable on its own
and prevents the trivial polish from diluting its diff. The PHASE-02 entrance is a
**soft** sequence, not a hard dependency — the phases touch disjoint files
(PHASE-01: `tests/e2e_estimate_non_blocking.rs` + `src/slice.rs`; PHASE-02:
`src/estimate.rs` + `src/value.rs`), so they could parallelize, but serial keeps the
order legible for a one-agent run.

**TDD shape.** Both phases are test-first by construction:
- PHASE-01's red proof is *active*: plant a facet reference in a non-allowlist module
  (and a scratch facet field on `Gate`) to watch each tier fail, then remove the
  scratch — the tripwire is only trustworthy if its failure mode is demonstrated, not
  assumed.
- PHASE-02's value test is ordinary red/green; the `expect`-string edit is a
  comment-only change with no test (verified by grep, VA-1).

**Why the confidence code stays dead.** D3 defers all display wiring to IMP-112. The
`resolve_confidence` path and the display renderers remain `expect(dead_code)` — the
tripwire self-clears when IMP-112 wires them in. PHASE-02 corrects only the *reason*
strings so the dead code is honestly attributed while it waits.

## Notes

- **REQ-id sequencing (design F5).** PHASE-02's `expect` reasons cite IMP-112 + a
  descriptive phrase for the confidence requirement; the concrete `REQ-NNN` is
  substituted at reconcile when the REV allocates it.
- **CHR-014.** PHASE-01's source-scan test bakes `CARGO_MANIFEST_DIR`; under a shared
  target dir a stale worktree path can leak. Keep the scan tree-relative and tolerate
  the standard `cargo test` build (matches the existing `e2e_*` source-scan tests).
- **Audit hand-off.** The NF-001 residual gap (PHASE-01 VA-1) and the NF-002/NF-003
  "already green, cited not rebuilt" disposition (design §7) are the audit's to record;
  they are not phase work.
