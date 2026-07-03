# Implementation Plan SL-195: Installer dual-mode — `--dev` marketplace source + `.mcp.json` POL-002 fix

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Three phases realise the locked design (`design.md`): a POL-002 portability fix
on the committed MCP surface, the `--dev` marketplace-source axis, and the
reinstall source-refresh that satisfies the dev invariant. The design collapsed
to a small, well-verified surface, so the plan is deliberately thin.

## Sequencing & Rationale

**PHASE-01 first — and file-disjoint.** The MCP-command POL-002 fix lives
entirely in `boot.rs` (`desired_mcp_entry`, `is_doctrine_mcp_entry`, `plan_mcp`),
touching none of the flag surface. It is the governance-critical fix (CHR-013's
committed abspath), isolated, and sits behind an already-well-tested pure seam
(`plan_mcp_*`). Landing it first banks the POL-002 win independently of the
installer work. Because it shares no file with PHASE-02/03 (`install.rs`,
`cli.rs`), it is the one genuinely parallelisable phase if dispatched.

**PHASE-02 — the actual new capability.** The `--dev` flag and marketplace-source
selection. It rides the existing claude-arm shell-out (marketplace add + install
`--scope project`) and only swaps the source argument, so the diff is small: a
clap bool (mirroring `dry_run`), a source selector, a manifest-name read, the
`doctrine@doctrine` qualification, and the precondition error. Sequenced after
PHASE-01 only for a clean serial history — there is no logical dependency.

**PHASE-03 last — depends on PHASE-02 and on an empirical probe.** The reinstall
source-refresh needs PHASE-02's source selection to know the *intended* source,
and needs a live probe (R4) to choose the refresh verb before code is written.
Isolating it keeps the flag work (PHASE-02) unblocked by that probe. Directory
sources are live-loaded, so this phase is narrowly about the registered *path*
going stale (repo move / slug change), not content.

## Notes

- **Behaviour-preservation gate.** PHASE-01 must leave `generate_mcp_extension`
  and its bake test untouched (invariant: baked ⟺ gitignored; design D2/F1). The
  migration arm of `is_doctrine_mcp_entry` (legacy abs + new env) is the subtle
  bit — a legacy abs entry must still read as *ours* so `plan_mcp` refreshes it,
  never double-registers (R1). This is the phase's primary test focus.

- **Impl-time probes (carried, not blockers).** OQ-4 (does the env-form
  `.mcp.json` connect under `/mcp` — PHASE-01 VH-1) and the R4 refresh verb
  (PHASE-03 EN-1/VH-1) are live checks. Both have documented expected answers
  (mcp.md:384; plugin-marketplaces.md:969-1001) — the phase sheets record the
  observed result before the dependent code locks.

- **GPT inquisition deferred.** Per the handoff (`/next`), a fresh agent raises a
  codex/GPT-5.5 adversarial pass on the locked design + this plan *before*
  execution, integrating findings via `notes.md`. F3 (bare `--dev` vs explicit
  `--marketplace-source`) is the most likely reopen.

- **CLI file correction.** The `--dev` wiring is in `src/commands/cli.rs` (Install
  clap struct + `InstallArgs` build), not `src/main.rs` as the design first named
  — selector corrected at plan-time.
