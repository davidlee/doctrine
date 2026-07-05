# Implementation Plan SL-206: Workflow-templated slice-driver

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Five phases. The deliverable is Rust-primary (three read-only doctrine tools +
a projection reader + a conformance-authority update) plus an install-templated
`/drive-slice` reference workflow. The design (`design.md`, RV-255-hardened) is
canon; this plan sequences it so each phase ends green and the risky premises
are proven before code depends on them.

## Sequencing & Rationale

**Why PHASE-01 first (empirical gate, not code).** Two premises the whole design
rests on are still unverified against *this* harness: SQ3 (a workflow's confined
fork mints at the armed base — RV-255 R4, leans on SL-199 prior art) and ISS-216
(a *changed* confined agent-def can be made live under `.claude/agents/` — R6).
Both are cheap to test with shipped SL-199 machinery and a scratch slice, and a
failure in either sends the work back to `/design`, not forward. Investing in the
receipt plumbing before this gate would be building on sand. No new code — pure
evidence.

**Why PHASE-02 is the reader core, alone.** The single genuinely-shared,
genuinely-testable seam is the per-phase projection over {plan, sheet, committed
boundaries}. It carries the behaviour-preservation risk (R5): `run_status` must
keep its slice-global aggregate output byte-identical while delegating only its
per-phase rows. Isolating that refactor in its own phase keeps the gate — the
existing `dispatch status` suite green *unchanged* — a clean pass/fail, unmixed
with new tool surface. `ReceiptStatus` lands here because the projection is what
derives it (Completed from the committed boundary, Blocked/Unknown fail-loud —
RV-255 F4).

**Why the tools (PHASE-03) precede the defs (PHASE-04) — a conformance ordering
proof.** Doctor check #9 is a *ceiling* ("extras rejected; tokens need not all be
present", `src/doctor_checks.rs`). Granting the orchestrator three new MCP tokens
means growing `allowed_mcp_tokens` FIRST: with the allowlist at six and the def
still listing three, `3 ⊆ 6` stays green. Then PHASE-04 adds the tokens to the def
(`6 ⊆ 6`, green). Reverse that order and the def would false-red against a stale
allowlist between phases (RV-255 F1). PHASE-03 also teaches the marker validator
the new `probe` role (it currently accepts only `worker|orchestrator`) — without
it, the read-only `dispatch-probe` def (RV-255 F5) is rejected as an invalid
marker.

**Why the read-only role is its own surface (F5).** The bootstrap planner and
closing divergence probe must be genuinely read-only; reusing `dispatch-orchestrator`
(which carries raw `Edit`/`Write`/`Bash`/`Agent`) would make "read-only" a lie
(CHR-039: containment is by policy, not MCP-unreachability). `dispatch-probe`
holds `Read`/`Grep`/`Glob` + the three reads only.

**Why PHASE-05 is a separate live acceptance.** PHASE-04's script is verified by
inspection + static conformance; but the load-bearing safety claims — halt-on-red
without auto-merge, a forbidden write refused against the *installed* def (F2's
runtime leg), the divergence advisory emitting without gating, the budget loop
pacing — only prove out in a real drive. This phase overlaps PHASE-01's SQ3 demo
but now exercises the full deliverable, not shipped machinery alone.

## Notes

- **Verification modes.** PHASE-01/05 are inherently empirical (VH/VA) — harness
  fork mechanics, live grant enforcement, and no-auto-merge are not unit-testable.
  PHASE-02/03 carry the structured VT floor (the testable core). PHASE-04 mixes
  a conformance VT with inspection VA.
- **OQ-1 (script home)** stays open into PHASE-04: `install/workflows/` does not
  exist yet; the exact authored home (install-templated vs harness-local) is
  settled there, informed by PHASE-01's ISS-216 finding.
- **ISS-216 dependency.** Out of scope to fix; SL-206 leans on the reseat/manual-
  placement procedure PHASE-01 characterises until ISS-216 lands (R6).
- **No auto-land (IMP-174).** No phase crosses the authored split-brain; landing
  stays `/audit → /reconcile → /close`. The driver reports; the divergence
  advisory is raw signal only.
