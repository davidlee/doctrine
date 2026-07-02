# Implementation Plan SL-187: Prompt cascade: per-harness delivery & boot integration

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Four phases deliver SL-186's inert `prompt resolve` engine to live agents,
splitting content by cache property (design D1). The plan is cut on **two
independent seams**: the *dependency* seam (what needs SL-186's verb) and the
*blast-radius* seam (which live surface each phase mutates).

- **PHASE-01** Boot universal sector — expose the snapshot generator + add the
  universal-hymns section. `src/boot.rs`. SL-186-independent.
- **PHASE-02** Onboarding memories inlined — tag + select (key-else-uid) +
  footer→inline. `src/boot.rs`, `src/memory.rs`, `install/memory/**`.
  SL-186-independent.
- **PHASE-03** `doctrine_onboard` contract change — model-band self-ID guidance +
  drop the memory load. `src/mcp_server/tools.rs`, `tests/e2e_mcp_server.rs`.
  SL-186-independent (references the verb *names* in guidance text; never calls
  them).
- **PHASE-04** `prompt resolve` delivery + per-harness wiring — disk regen +
  stdout + the pi/hook command swap. `src/boot.rs` + SL-186's `prompt` command.
  **HARD-GATED on SL-186.**

## Sequencing & Rationale

**Why this order.** PHASE-01 lands the reusable generator seam PHASE-04 depends
on, so it must come first. PHASE-02 mutates the same disk-snapshot assembly
(`src/boot.rs`) as PHASE-01 — sequencing them serially avoids a self-conflict on
the section table. PHASE-03 can only shed the two-memory load *after* PHASE-02
has moved those bodies into the always-present cached sector (EN-1), else onboard
agents lose the memories outright.

**PHASE-04 is last by external necessity, not preference.** At plan-time SL-186
was `started`; it has since **closed and merged to edge** — `doctrine prompt` is
now a subcommand (`src/commands/prompt.rs`, `src/hymns.rs`) and `prompt resolve
--role <ROLE> [--harness --model --arm --stage --band]` is live, with the loader
at `install::load_full_corpus`. PHASE-04 *extends* that verb (adds disk-regen +
stdout delivery) and *targets* it (the hook/pi swap), so its EN-1 hard external
gate — the `after: SL-186` sequence relation made executable — **is now
satisfied**. Design §3 frames this as a *contract* dependency, not a *build*
dependency: PHASES 01–03 built in full against the locked contract; only
PHASE-04's green depended on SL-186 landing, which it now has.

**The behaviour-preservation gate is scoped, not blanket** (RV-210 F-2/F-4).
Boot's *entity-derived* section goldens and the dispatch suite stay green
unchanged; three deltas change **by intent** and are budgeted in the exit
criteria: the boot Onboarding golden (footer→inline, PHASE-02), a new
universal-hymns golden (PHASE-01), and the onboard e2e assertion (PHASE-03,
`tests/e2e_mcp_server.rs:1083`). The gate is "no *unintended* delta beyond these
three."

## Notes

- **RV-210 inquisition** (codex, design facet) reconciled all 7 charges into
  `design.md` before this plan; the plan inherits the corrected §5.2/§9 and
  decisions D6–D8. See design §10.
- **Parallel-dispatch caution.** PHASE-02 and PHASE-01 both edit `src/boot.rs`
  section assembly — NOT file-disjoint, so they run serial, not parallel.
  PHASE-03 (`tools.rs`) is file-disjoint from 01/02 and could dispatch in
  parallel once its EN-1 (PHASE-02 done) holds.
- **Open at plan-time:** OQ-2 (single vs combined hook emit) is a PHASE-04
  shaping detail, not a fork — settled during phase-plan. OQ-1 resolved to D6
  (soft `doctrine check` warning, no hard cap).
