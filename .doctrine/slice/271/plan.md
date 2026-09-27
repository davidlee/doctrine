# Implementation Plan SL-271: Codex MCP registration during install

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See glossary.md § reference forms. -->

## Overview

Three code phases deliver the locked design of run `dr-01a0e17c` (revision 44):
the shared decision core (`DEC-328`), the codex registration leg
(`DEC-323`, `DEC-325`, `DEC-332`), and the hooks probe plus disclosure folding
(`DEC-329`). No phase cites the `SPEC-011` Revision: it is raised at reconcile,
after the phases, and its member ids do not exist yet.

```text
PHASE-01 shared core ──► PHASE-02 codex leg ──► PHASE-03 probe + disclosure ──►
 McpEntryClass,            constants, planner,     CaptureRunner, parse,
 mcp_action, plan_mcp      toml_edit write, arm,   codex_hooks_state,
 refactor, payload =       report seam, README     Option<HooksState>,
 full invocation           sh declaration          trust caveat
                    ──► audit ─► reconcile (two-member SPEC-011 Revision) ─► close
```

The slice traces to `IMP-111` and the design (`SL-271` design sec-1..sec-9).
`IMP-111` is the source item and is resolved only at close.

## Sequencing & Rationale

- **Shared core first, because both arms call it.** `mcp_action` and
  `McpEntryClass` are the one element the codex arm and the Claude arm share
  (`DEC-328`). Landing them with the `plan_mcp` refactor makes the existing
  `plan_mcp_*` suite the behaviour-preservation proof, exactly as the design
  requires (sec-9): the suite, not a new one, shows the refactor changed no
  planner outcome. The codex arm cannot start before this seam exists.
- **Payload change travels with the message change.** Moving the
  `Wired`/`Refreshed` payload to the full invocation and dropping the
  appended ` serve --mcp` in `wire()` are two halves of one edit; splitting
  them across phases would print the arguments twice between phases.
- **The whole leg lands together, because half a leg is dead code.** The codex
  planner and writer are meaningless without the arm that calls them, and the
  arm is meaningless without the report seam that names the codex file. A
  planner-only phase would leave `install_codex_mcp` uncalled in a non-test
  build — a `dead_code` warning at the zero-warning gate. So PHASE-02 carries
  the planner, the writer, the arm wiring, the report seam, its unit matrix and
  its e2e cases in one coherent unit.
- **The probe is keyed to the hook write, not the MCP write** (`DEC-329`), and
  needs the MCP outcome only for the trust caveat. That makes it a natural
  third phase: PHASE-02 supplies the `report.mcp` signal, PHASE-03 adds the
  capture seam and folds the probe into the disclosure.
- **Docs travel with the code that makes them true.** The `sh` declaration
  (`POL-002` facet 3) is an `EX` in PHASE-02, beside the wrapper it declares,
  rather than a separate documentation phase.
- **Verification ids are local and immutable.** They restart at `VT-1` per
  phase; nothing cross-references a bare `VT-n` without its phase qualifier.

## Routed findings (RV-399)

None. Every one of `RV-399`'s 44 findings is disposed `fix-now` and verified;
the ledger carries no `route:demonstrate`, `route:probe` or `route:control`
finding, so there is no routed obligation to transcribe into a criterion. The
design (revision 44) already carries every decision the ledger produced; the
ledger is history, not plan input.

## Notes

- **Research advisory.** `doctrine slice research 271` reports drift in
  `design.md` only — the artefact was baselined before design existed, so a
  change there is expected, not a stale code premise. Every concrete design
  reference (paths, symbols, constants, line numbers) was re-grepped against
  the tree at plan time and resolves; `src/boot.rs` and `src/install.rs` are
  unchanged since the design commit `e569620cd`. No refresh is warranted. The
  baseline is deliberately NOT restamped, so the advisory keeps telling the
  truth about when research was taken.
- **Selectors.** Drafted from the research Thread 2 hotspot map and the design
  impact table: `src/boot.rs`, `src/install.rs`, `tests/e2e_codex_install.rs`,
  `README.md` and `install/**` are `design-target`; `tests/**` is
  `scope-relevant`. The redundant `install/manifest.toml` selector was removed
  (subsumed by `install/**`). `tests/e2e_codex_install.rs` reads `unmatched`
  until PHASE-02 creates it — expected, not drift.
- **Version-delta assumptions.** Design sec-5.5 rests on a codex 0.155.1 probe
  and is recorded as a `POL-003` facet 2 version delta. No phase re-probes
  codex, and no criterion asserts a live harness behaviour. The post-write
  check (`codex mcp get doctrine`) and the seam re-verification are captured as
  `IMP-497` (this slice, `references`), deliberately outside the phases; the
  install report claims only what doctrine wrote.
- **`ISS-495` is out of scope.** `install_mcp`'s blanket `.ok()` on the Claude
  read is left as-is; PHASE-02 applies the `NotFound`-vs-error split to the
  codex read only, and `ISS-495` tracks the Claude fix.
- **Spec Revision at reconcile.** One `SPEC-011` Revision introduces two
  members (retro-covering the shipped Claude `.mcp.json` arm; the codex
  `mcp_servers` leg). It is created and applied at reconcile; no phase cites
  its `REQ` ids. Close requires it landed or a recorded waiver
  (`SL-250`/`RV-350` precedent).
- **No new module edge.** The capture seam rides the existing `boot` ↔
  `install` command edge: `boot` already reaches `install`
  (`install::asset_text`), and `install` already reaches `boot`
  (`boot::wire`), so `install::CaptureRunner` used from `boot::wire` adds no
  new module pair and does not grow the command tangle ratchet
  (`ADR-001`; `tests/architecture_layering.rs`). Both are `command` tier.
- **Fixture discipline.** Every ownership fixture is seeded literally, never
  through a real installer, and every absence assertion carries a positive
  control in the same test (`mem.pattern.boot.test-exec-is-not-doctrine-owned`,
  `mem.pattern.testing.absence-assertion-over-an-unwritten-file`).
- **No new runtime state.** The only state is the config file; the planner is a
  pure function of its bytes. Nothing here writes runtime state, so no VA
  criterion reads gitignored state.
