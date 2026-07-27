# Implementation Plan SL-233: CLI-managed design runs and inquiry maps

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Nine phases, serial. The serialism is forced rather than chosen: phases 2–7
nearly all route through `src/design_run/` or `src/commands/design.rs`, so there
is no honest file-disjoint pair to parallelise. Two phases are coordinator-only
by construction — a dispatch worker never writes authored `.doctrine/` state, so
the governance descent (PHASE-01) and the evaluation kit (PHASE-09) cannot be
dispatched at all.

The middle five phases build a vertical: a pure model, its persistence, its
command surface, its entity effects, and its authored output. Each is usable
evidence that the one before it was right, which is the argument for the order.

## Sequencing & Rationale

### Two constraints the design fixes, which this plan honours verbatim

**Governance descends first.** PHASE-01 allocates the exact specification
entities and appends their exact paths as `design-target` selectors *before*
editing any body. This is the narrow bootstrap the design chose in place of a
corpus-wide spec wildcard, and its exit is a green `slice conformance`. The
phase is unforgiving in both directions: allocating too few entities blocks
later phases behind undeclared spec edits, while allocating too many grants
authority over empty shells. Selectors and phase ids are immutable append-only,
so neither error is undoable within this slice.

**Guidance never names a command the binary rejects.** RV-315 F-15 was raised
because the core-process sentence in `install/routing-process.md` was rewritten
seven hours before the command it taught existed — and that file is the first
section of every boot snapshot, `@`-imported into every agent's context in every
session. The rewrite therefore lands in PHASE-08.

That placement is an **interpretation of the locked design, and is recorded as
one.** Design §5.5 binds the rewrite to "the implementation phase that ships the
executable `doctrine design` family", with the two changes landing "together".
Read narrowly, that could mean PHASE-04, where the commands are first wired.
The reason it is placed at PHASE-08 instead: the same paragraph requires that
phase to prove *every command named by the generated core process is accepted by
the just-built binary*, and at PHASE-04 `materialise` does not yet exist. A
core-process sentence describing the managed-design loop would name a command
the binary rejects — the precise failure F-15 punished. PHASE-08 is where the
family is executable end to end, so it is where the constraint is satisfiable
rather than merely assertable. The design also grants a freedom it does not
advertise: guidance may land no *earlier* than executability, but later is
strictly safe. There is a lower bound and no upper one.

### The three design gates

Design-in-the-small is not evenly distributed across the nine phases. PHASE-02,
03, 05 and 07 are heavily prescribed by design.md — ride the review ledger's
`can()` table, ride `wire.rs`, ride `write_body`, one enum selecting one file —
and PHASE-03 in particular acquired three explicit rules and a seven-assertion
matrix through RV-315 F-19 and F-20. Those phases can go straight to
`/phase-plan`.

Three phases carry genuine undesigned surface, and each takes a design gate as
an entrance criterion: a written sketch plus an adversarial pass whose findings
are dispositioned before implementation starts.

- **PHASE-04 — projections and token bounds.** §9.3 asserts that named limits
  bound the envelope against a large-run fixture, but the algorithm behind them
  does not exist: what is dropped, how the nearby frontier is chosen, how the
  material-change delta is computed. R1 — that protocol ceremony exceeds its
  value — is won or lost here, and it is the risk the whole slice is judged on.
- **PHASE-06 — the marker grammar.** Re-adoption correctness rests entirely on
  it, and the design specifies only that markers are "unobtrusive" plus a list
  of refusals. Refusals are only as good as the grammar they refuse against;
  syntax, escaping, collision behaviour, and survival of a Markdown formatter
  are all open.
- **PHASE-08 — the thin adapter.** RV-315 F-17 established that *no
  specification owns a skill body*: PRD-003 disclaims what a skill says, and
  SPEC-010 treats it as opaque payload. Rewriting an eight-state workflow
  machine into a thin adapter is therefore a real design act with no governing
  artifact over it. The adversarial pass is the only gate it gets.

The gates are entrance criteria rather than separate slices deliberately. These
three surfaces are *inside* the implementation — a child slice for the marker
grammar could not compile without PHASE-02 and PHASE-03, and its closure
evidence would be a fragment. That is ceremony without isolation. An EN
criterion buys the same scrutiny for one line each.

### Why the evaluation kit stayed in

Lifting PHASE-09 into its own slice was considered and rejected; the reasoning
is recorded on the abandoned SL-235 rather than repeated here. In short:
CHR-049 already carries the live human-in-the-loop exercise post-close, against
genuinely installed bytes, with entry checks that refuse authored-only presence.
The separation that mattered already existed, and buying it again would have
cost a reopened lock, a superseded DEC-079, and a weakened closure gate.

## Notes

**First-use risk in PHASE-07.** `install/hymns/stage/` is empty and
`[hymns].seal` holds exactly one entry (`preamble/core`), though the `stage`
band is registered in `install/hymns/README.md` and `KNOWN_STAGE_LABELS`
validates its labels. `stage/design` will be the first stage-band hymn ever
authored. The band's happy path has never run, so PHASE-07 exercises it before
relying on it rather than discovering the gap through a failing install test.

**The prompt-pack fork is bounded by test, not by intention.** RFC-021 says to
reuse and generalise the existing cascade; this slice forks a closed,
design-specific store instead. That was argued and accepted, but research rates
the precedent risk high, so PHASE-07's exit requires a test proving
`install/design-prompts/` has no consumer outside the design run and that
`name@digest` appears nowhere outside `design show`/`design resume`. If the fork
later generalises, those tests are what make the blast radius knowable.

**Editing an `install/` asset is not self-delivering.** Three artifacts carry
the same sentence — the embed, the projection, and the boot snapshot — and each
needs a different verb to move: `cargo build` re-embeds, `doctrine boot`
regenerates, `doctrine boot --check` confirms, `doctrine install` refreshes the
projection. Any phase touching `install/**` inherits that tail or it ships a
lie. PHASE-08 carries it explicitly.

**The research packet is gitignored.** `.doctrine/slice/233/research/` exists in
the primary working tree only. It will not travel to a fresh clone or a dispatch
worktree, and `rm -rf` of the runtime tier destroys it. The plan load-bears on
its Round 2 threads; the durable residue is in `notes.md` § Harvest.

**Worktree state is not shared.** A fork carries its own `.doctrine/state`
directory rather than the parent's — verified at plan time against the live
coordination trees. This is why PHASE-09 is coordinator work and why no
dispatched phase can exercise a live design run.
