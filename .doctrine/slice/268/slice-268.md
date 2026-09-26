# Review ledger v2

## Context

RFC-032 (*Review ledger effectiveness*) programme, slice 1. Every later slice in
the programme (read surface, design-run binding, identity & locus) reads the
schema this slice fixes, so the hard-to-reverse choices land here. The decisions
are settled in RFC-032 `decision-frontier.md` (2026-09-26, user-settled forks in
§5); the evidence is RFC-032 `research.md`.

Preconditions met: step 0a (CHR-057, stale `ISS-314 needs IMP-392` edge
retracted) and step 0b (IMP-481, governance-coverage assessment: a new tech spec
owns the ledger mechanism, container level under SPEC-003, no new PRD).
Step 0c (the quick-win finding index, IMP-490) is independent and may land
first; if it does, its view struct moves with D4 rather than being rewritten.

Today: `src/review.rs` is 5 773 lines in the command tier; nothing below it can
ask an RV its status. `contest`/`verify` reasoning is ephemeral (`--note`), an
empty ledger derives `done` against ADR-007 D-C8 (`ISS-314`, `ISS-366`), a
corrupted enum silently coerces (`CHR-001`), and prose arguments pass through the
caller's shell (obs `019fc0bc`: live API keys spliced into a committed ledger).

## Scope & Objectives

Decision ids below are RFC-032 `decision-frontier.md`'s.

1. **D4: split `review.rs` along the tier line, golden first.** Write IMP-029's
   black-box e2e CLI golden *before* moving code. Then split into an engine-tier
   `review/ledger` module (schema, lenient reader, `derived_status`, transition
   graph, blocker predicates) and command modules (verbs, `with_turn`,
   baton/lock, prime, render). This refactor preserves behaviour, and the
   existing suites plus the golden are the gate.
2. **D1: an append-only turn journal per finding.** `[[finding.turn]]` rows
   (`act`, `role`, `note`) are written in the same CAS-guarded write that moves
   `status`, which stays stored as the current state. The ephemeral `--note`
   concept is retired. Two new acts are added: `amend` (responder,
   answered → answered) and `reopen` (raiser, verified → contested). There is no
   global sequence number. The `rounds`/`contests` counters become baseline plus
   count (`rounds_base` / `contests_base`). Legacy ledgers get no migration.
3. **D2: uniform `done`.** `done ⇔ all findings terminal ∧ concluded`, derived
   on every read and never latched. `conclude` takes a required `--basis`,
   stored as a review-level turn, and becomes the required last move of every
   pass: `raise` and `reopen` on a concluded ledger clear `concluded`
   (RV-396 `F-4`). Legacy ledgers are not backfilled. Closes `ISS-314` and `ISS-366`.
4. **D15: every closed vocabulary reads fail-safe.** An unknown `severity`
   gates as `blocker`. An unknown `status` reads as non-terminal. Both are
   disclosed where they are read (`review show`/`status`/`list` and the close
   gate warn, naming the RV, finding and raw value; STD-003, DEC-319). An unknown
   `disposition` renders verbatim. The baton-note write moves inside the
   lock/CAS (`CHR-001`).
5. **D8, write side.** `--as` stays closed `{raiser, responder}`, with the
   labels declared at `review new` accepted as aliases and `--help` naming the
   legal values (`IMP-336`). `disposition` is closed on write and open on read.
   An optional closed `route` field replaces the `route:` prose token.
   `@PHASE-NN` is accepted as a target spelling.
6. **D10: prose arguments accept `-` (stdin) and `@path`** on every review
   verb, through one shared arg type. The MCP tools get the same fields.
   Guidance routes writes to MCP, or to a quoted heredoc on the CLI.
7. **D11 (rides here), D-C10 amended to the selector model.** `prime` on a
   target with no path-set degrades and says so (STD-003, `IMP-259`). The
   literal-selector arm gets the non-file filter (`ISS-059`).
8. **D12: no reverse index.** Close `IMP-479` as not needed until measured.
9. **Governance.** Author the new "Review ledger" tech spec (container,
   `parent = SPEC-003`, no `descends_from` unless `/spec-tech` finds one, per
   IMP-481) describing v2, in a governance phase after the code lands so its
   anchors are live (DEC-321); design.md carries the target schema until then.
   Mint a REV amending ADR-007 D-C5 (turns, amend/reopen),
   D-C8 (uniform `done`, the conclude marker's role) and D-C10 (D11), and
   revising SPEC-003's container inventory to add the new spec (REV-035
   precedent; SPEC-029, already missing there, is added in the same REV;
   DEC-317). Keep DEC-233's RV `Unavailable` arm; update its comment and pinning
   test to say derivation is engine-tier and it retires with D14 (DEC-318).
10. **Guidance.** Update the skills that run a pass (`/audit`, `/code-review`,
    `/inquisition`, the design-run guidance) so `conclude --basis` is the last
    move and writes go through MCP. Refresh `install/review-ledger.md`
    (`CHR-079`) for the verbs and acts this slice changes.
11. **Bounded guidance review (DEC-320).** Review the installed guidance
    (install reference docs, shipped skills, MCP tool descriptions) for
    misleading CLI and MCP usage (verbs, flags, fields, argument shapes),
    starting with every surface this slice changes. Fix what is found in-slice
    or file it. The automated check waits on IMP-492.

## Non-Goals

- D5's read projection, census and JSON unification, and D14 (RV as a relation
  target): slice 2.
- D3 (section anchor) and D6 (design-run bind-before, `External` arm, the
  `Finding` deletion): slice 3. SPEC-029 is not revised here.
- D9 (clone-wide reservation) and D7 (locus tiers, merge backstop): slice 4.
  PRD-005 and SPEC-008 are not revised here.
- Secret scanning (D10 rejects it for the review kind; a repo-wide backlog item).
- Reconstructing pre-journal history, or backfilling `conclude` on legacy
  ledgers.
- Catalog scan callers that drop warning diagnostics (ISS-492, RV-396 `F-3`).
- Governing `doctor` (IMP-491), and new doctor checks: the install-doc CLI
  check (IMP-492) and the status/last-turn check (IMP-493) wait on it
  (DEC-320, DEC-322).

## Summary

Fix the RV ledger's authored schema and its derivation rules: turns carry the
reasoning, `done` means concluded, and bad vocabulary fails safe. Do it behind
a golden-guarded tier split, so the rest of the programme builds on an
engine-tier module with a spec that owns it.

### Affected surface

- `src/review.rs` becomes `src/review/` (ledger engine module plus command
  modules)
- review verbs in `src/commands/cli.rs`, and the review tools in
  `src/mcp_server/tools.rs`
- consumers of `derived_status` and the blocker predicates: `src/slice.rs`
  (close gate), `src/commands/design.rs`, `src/commands/guard.rs`,
  `src/relation.rs`, `src/priority/partition.rs`, `src/catalog/scan.rs`
- tests: new e2e golden (IMP-029); `tests/e2e_design_review.rs`; the
  `review.rs` unit suite
- `install/review-ledger.md`; `plugins/doctrine/skills/{audit,code-review,inquisition,reconcile,close}/SKILL.md`;
  other installed guidance the bounded review flags
- `src/kinds/mod.rs` `DERIVED_STATUS` comment and its pinning test (DEC-318)
- governance: new tech spec, a REV against ADR-007 and SPEC-003

### Risks

- **R1: behaviour drift in the split.** Mitigation: the golden lands first and
  unchanged suites are the gate (AGENTS.md behaviour-preservation gate).
- **R2: uniform `done` flips the visible status of many legacy ledgers** to
  `active`. This is accepted (frontier §5). Nothing gates on legacy status
  today, and the close gate counts only non-terminal blockers (D2).
- **R3: the MCP and CLI write surfaces diverge** on new fields (turn notes,
  basis, route). One shared arg type and one write function carry both.
- **R4: the `D-C10` amendment depends on D11 riding here.** If design pushes
  `prime` changes out, the REV drops D-C10.

### Assumptions

- The five documented disposition values are the closed write set (D8). The
  61 values found in the RFC-026 E1 corpus stay readable.
- The journal adds about one short table per transition. File growth is
  accepted (D1).

### Open questions

- **OQ-1**: *Resolved (DEC-320):* no automated check here; a bounded guidance
  review rides this slice (objective 11). The check is IMP-492.
- **OQ-2**: *Resolved (DEC-321):* authored at the end, describing what landed.
- **OQ-3**: *Resolved (research):* step 0c (IMP-490) landed at `694acaf43`,
  before D4. D4 moves its finding-index view struct.
- **OQ-4**: *Resolved (DEC-322, DEC-319):* no doctor checks here. Unknown
  vocabulary warns at the read site; the consistency check is IMP-493.

### Verification / closure intent

- The IMP-029 golden is green before and after the split. The existing review
  and design-review suites stay green without edits, except the D2 flips named
  in the frontier and research (`derived_status_empty_is_done_none`,
  `derived_status_total_over_enum`, `show_renders_empty_ledger_done_and_the_edge`,
  `list_renders_empty_ledger_done_and_the_edge`).
- Tests cover: a turn appended on every transition; `amend` and `reopen`;
  counters equal baseline plus count; `done` requires conclude; `--basis`
  required; fail-safe reads for each vocabulary, each with its read-site
  warning; `-`/`@path` on every prose
  argument; alias `--as`; and the closed disposition refusal naming the set.
- `derived_status` is reachable from the engine tier (no command-tier import).
- The tech spec exists with live anchors. The REV is applied at reconcile.
  `ISS-314`, `ISS-366`, `IMP-479` and `IMP-029` are closed.
- The skills and `review-ledger.md` name `conclude --basis` and the MCP write
  path. The bounded guidance review (VA) leaves a committed record of what it
  checked and what it fixed or filed. `doctrine check gate` is green.

## Follow-Ups

- Memory `mem_019f97fcab2e77a28902371f80743605` ("Verified RV findings are
  terminal — … not a reopened disposition") is contradicted by D1's `reopen`.
  Update it at close.
- Memory `mem_019fdfe379b67e53857735b88c394b52` ("A review reading `done` is not
  concluded") becomes false under D2. Update it at close.
- Repo-wide secret-scanning backlog item (D10). File it if not already captured.
