# Review RV-392 — code-review of SL-266

Adversarial-review ledger. Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

<!-- Pre-reading + lines of attack: what this review is probing, the invariants
     it must hold the subject to, and where the bodies are likely buried. Seeded
     at `review new`; the reviewer fills it before raising findings. -->

Per-phase review of SL-266 PHASE-03, commit `3cdceba18` (command surface and
run resolution). Tripwire that made it mandatory: edits outside the design's
declared selectors (`src/knowledge.rs`, `src/slice.rs`, `src/commands/guard.rs`,
test fixtures).

Governing: `design.md` sec-4 (DEC-304, DEC-305); `plan.toml` PHASE-03 EX-1..4,
VT-1, VT-2, VA-1; STD-001, STD-003; ADR-001 (layering).

Lines of attack:

1. **One read (DEC-305).** Is the snapshot projected exactly the one whose stage
   and slice status `select_run` judged, with no second read?
2. **`select_run` purity and rules.** Locked and done/abandoned excluded; newest
   mtime; tie goes to the higher slice; count covers open runs only.
3. **Disclosure (STD-003).** Does every failure to read land in `skipped` with its
   cause, and is anything silently dropped that should not be? Check non-numeric
   dirs, a missing `design.toml`, and an unreadable slice record (a phase
   decision not stated in sec-4).
4. **Titles at Full.** `project()` now reads titles for *every* Full projection,
   so `prompt --full` and `json --full` too. Is that a correct reading of sec-2, or
   scope creep? Are errors carried, not blanked?
5. **Cohesion / coupling.** `knowledge::record_title` and `slice::status` placement;
   `design.rs` growth; is `RunCandidate<T>` over-generalised?
6. **Parity.** Are `design tree SL-N` and `show --format tree` identical by
   construction, not merely by test?
7. **STD-001.** `TREE_COMMAND` reuse; no new spelling of the state path.
8. **Tests.** Do they catch the risks (ordering, tie, terminal) or are they
   theatre? Is there a gap in the e2e for locked runs?

## Synthesis

- **Overall:** acceptable → solid after fixes.
- **Synopsis:** PHASE-03 wires `design show --format tree` and `design tree
  [SLICE]` onto one renderer (`tree_lines`), so parity is by construction. It
  also adds the no-slice scan and the pure `select_run`. The reviewer (codex,
  `gpt-6-sol`, high) raised three verified findings, all in the scan and title
  reads and all fixed in `6d9bf2db0` with red-first tests. F-1: status was judged
  by directory, not by the snapshot's own slice. F-2: a numeric alias directory
  double-counted a run. F-3: title-only records passed as found. Checked sound:
  colour parity, locked/terminal filtering, mtime tie-break, one read per
  snapshot, ADR-001 layering. Accepted tradeoffs: titles are read at every
  `Full` projection, not only the tree's (the sec-2 reading, recorded in notes);
  an unreadable slice record skips its run rather than counting it open. No e2e
  covers a locked run, since the fixture cannot lock cheaply; the unit table
  covers it. The raiser's verifications were recorded by the implementer: the
  reviewer runs one-shot and does not take a second turn.
- **Haiku:** *Two doors, one number — / the scan now asks the snapshot / whose
  name it carries.*
