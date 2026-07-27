# Implementation Plan SL-233: CLI-managed design runs and inquiry maps

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Twelve phases, serial. Execution order is the `plan.toml` array order:

    01 → 02 → 03 → 04 → 05 → 06 → 11 → 12 → 10 → 07 → 08 → 09

Phase ids are immutable and append-only, so the three phases added by the
2026-07-28 revision carry `PHASE-10`…`PHASE-12` while sitting mid-sequence.
`src/plan.rs` reads the ordered phase array and never sorts by id, so the array
is the authority; the ids are names, not positions.

The serialism is forced rather than chosen: phases 02–07 nearly all route
through `src/design_run/` or `src/commands/design.rs`, so there is no honest
file-disjoint pair to parallelise. **Three** phases are coordinator-only by
construction — a dispatch worker never writes authored `.doctrine/` state — so
the governance descent (PHASE-01), the pure run model (PHASE-02, which must add
a tier line to `.doctrine/adr/001/layering.toml`), and the evaluation kit
(PHASE-09) cannot be dispatched at all.

The middle vertical builds a stack: a pure model, its persistence, its command
surface, its entity effects, and its authored output. Each is usable evidence
that the one before it was right, which is the argument for the order. Three
seams then ride that stack — entry from authored prose (PHASE-11), the reviewing
stage (PHASE-12), and delegation (PHASE-10) — before assets, adapter, and
evaluation close the slice.

## Sequencing & Rationale

### Two constraints the design fixes, which this plan honours verbatim

**Governance descends first.** PHASE-01 allocates the exact specification
entities and appends their exact paths as `design-target` selectors *before*
editing any body. This is the narrow bootstrap the design chose in place of a
corpus-wide spec wildcard, and its exit is a green `slice conformance`. The
phase is unforgiving in both directions: allocating too few entities blocks
later phases behind undeclared spec edits, while allocating too many grants
authority over empty shells. Selectors and phase ids are immutable append-only,
so neither error is undoable within this slice. EX-7 now requires the selector
append to land as its own commit — otherwise EX-5's *before, not after* is
unfalsifiable, since a four-entity descent's natural shape is a single commit.

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

The *runtime* boot snapshot is a separate matter and moves earlier — see
§ Notes, "Boot diverges at PHASE-04".

### The three design gates

Design-in-the-small is not evenly distributed across the phases. PHASE-03, 05
and 07 are heavily prescribed by design.md — ride `wire.rs`, ride the existing
reservation backend, one enum selecting one file — and PHASE-03 in particular
acquired three explicit rules and a seven-assertion matrix through RV-315 F-19
and F-20. Those phases can go straight to `/phase-plan`.

Three phases carry genuine undesigned surface, and each takes a design gate as
an entrance criterion. **A gate is two artifacts, and both are authored.** The
first draft left the gates homeless — "a written sketch exists and has had an
adversarial pass" with no location, no ledger, and no disposition mechanism,
which would have put three real post-lock design acts in the gitignored,
`rm -rf`-able runtime tier, against the design's own R3 and the storage rule.
Each gate is therefore:

1. a committed sketch under `.doctrine/slice/233/sketches/` — durable, diffable,
   surviving `rm -rf` of the runtime tier and travelling to a fresh clone; and
2. a dedicated RV ledger raised against that sketch, driven terminal with every
   finding dispositioned, exactly as RV-315 gated `design.md`.

`design.md` is locked; these three surfaces are the design work that remains,
and they get the same instrument the locked design got.

- **PHASE-02 — projections and token bounds**
  (`sketches/projection-bounds.md`). §9.3 asserts that named limits bound the
  envelope against a large-run fixture, but the algorithm behind them does not
  exist: what is dropped, how the nearby frontier is chosen, how the
  material-change delta is computed. R1 — that protocol ceremony exceeds its
  value — is won or lost here, and it is the risk the whole slice is judged on.
  The gate sits at PHASE-02 rather than at PHASE-04 where the bounding is coded,
  because the envelope *type* is defined in PHASE-02 and it is the thing being
  bounded. The circularity is real and acknowledged: the sketch bounds a type
  that does not exist yet, and the type is shaped by a budget the sketch sets.
  Something has to go first, and rework of the pure model is the more expensive
  direction because everything above it inherits the change. PHASE-04 EN-2 is a
  re-read of the sketch against the model as built, not a mandatory second
  review round — "no drift" is the expected finding.
- **PHASE-06 — the marker grammar** (`sketches/marker-grammar.md`).
  Re-adoption correctness rests entirely on it, and the design specifies only
  that markers are "unobtrusive" plus a list of refusals. Refusals are only as
  good as the grammar they refuse against; syntax, escaping, collision
  behaviour, and survival of a Markdown formatter are all open.
- **PHASE-08 — the thin adapter** (`sketches/thin-adapter.md`). RV-315 F-17
  established that *no specification owns a skill body*: PRD-003 disclaims what
  a skill says, and SPEC-010 treats it as opaque payload. Rewriting an
  eight-state workflow machine into a thin adapter is therefore a real design
  act with no governing artifact over it. The adversarial pass is the only gate
  it gets.

The gates are entrance criteria rather than child slices deliberately. These
three surfaces are *inside* the implementation — a child slice for the marker
grammar could not compile without PHASE-02 and PHASE-03, and its closure
evidence would be a fragment. That is ceremony without isolation. A sketch plus
an RV buys the same scrutiny without the lifecycle overhead.

### Why the evaluation kit stayed in

Lifting PHASE-09 into its own slice was considered and rejected; the reasoning
is recorded on the abandoned SL-235 rather than repeated here. In short:
CHR-049 already carries the live human-in-the-loop exercise post-close, against
genuinely installed bytes, with entry checks that refuse authored-only presence.
The separation that mattered already existed, and buying it again would have
cost a reopened lock, a superseded DEC-079, and a weakened closure gate.

### Scope objectives the phase list dropped

Two passes over the plan against the scope found scope objectives with no owning
phase. They are recorded here because the omission is instructive: each is a
*seam* rather than a component, and seams are what a phase list built from an
architecture diagram tends to lose. The first pass (2026-07-27) swept scope §4
only; the second (2026-07-28) swept §2, §3, §5 and design §5.2/§5.4 and found
six more.

Assigned in the first pass:

- **Bounded delegation** (DEC-058, DEC-068) had three incidental mentions and no
  exit criterion — the fragment file was planned, the protocol was not.
- **Conservative import** (DEC-084, DEC-085) had none. `--from-design` is the
  only entry an existing authored design has.
- **The handover short-circuit** (DEC-058) had none, despite
  `plugins/doctrine/skills/handover/SKILL.md` being a declared design-target
  selector no phase touched. It lands in PHASE-08 as EX-7 alongside the other
  adapter convergence.

Assigned in the second pass:

- **The entire `reviewing` stage** (scope §5, DEC-073, DEC-074, design §5.4).
  The word `review` appeared in the plan exactly twice: inside PHASE-02's
  "separate types" list, and as `src/review.rs`'s `can()` table to copy.
  Section attestations, the human-default/adversarial-opt-in choreography,
  finding disposition, and the `reviewing → locked` predicate — including the
  user-acceptance attestation that makes the lock legitimate — had no owner.
  This is now PHASE-12, and it is a phase rather than a criterion because it is
  a stage's worth of machinery.
- **Traversal direction** (scope §2). "Permit immediate user pin, defer, prune,
  breadth, and depth direction during the run" had no criterion. It lands in
  PHASE-04 as EX-8, with node `provenance` as a first-class field in PHASE-02
  EX-7 so that design R4 (authority laundering) is a type-level property rather
  than a convention.
- **Recovery after runtime loss** (scope §1's headline, DEC-057). `resume` was
  one bare word in a command list. Its projection contents are now PHASE-04
  EX-9; semantic reconstruction after snapshot loss is PHASE-11 EX-6, paired
  with import because both are "enter a run from authored artifacts alone".
- **The two non-record dispositions** (scope §3). PHASE-05 implemented create
  and adopt; *retain-explicitly-unresolved* and *mark-intentionally-non-durable*
  had no owner, making a resolved node that produces no record unrepresentable.
  Now PHASE-05 EX-7.
- **The sparse mutation contract** (DEC-063). Omitted-persists, `null` clears a
  scalar, `[]` clears a collection, stable-ID partial update, unordered batch,
  duplicate-subject refusal, no deletion — the actual semantics of `apply` — had
  one criterion covering only whole-candidate validation. Now PHASE-04 EX-7.
- **Direct reasoned regression** (DEC-067) had no criterion. Now PHASE-02 EX-6,
  where it belongs: §9.1 lists it as pure-engine behaviour.

### The two phases that were doing three jobs

PHASE-04 and PHASE-06 each ended the first revision carrying an undesigned
surface behind a design gate *plus* a bolted-on orphan objective. Both are now
split. Splitting cost non-monotonic ids; leaving them would have cost the two
phases most likely to be abandoned mid-flight.

- **PHASE-04** kept the command surface, the canonical envelope, and bounded
  projections. The delegation protocol moved to **PHASE-10**, placed after the
  reviewing stage so that the four prompt fragments PHASE-07 authors — inquiry,
  drafting, reviewing, delegation — all name obligations that exist.
- **PHASE-06** kept materialise, the marker grammar, re-adoption, and the
  two-armed shim. Conservative import moved to **PHASE-11**, which rides the
  same marker machinery but is a separate act: entering a run from prose that
  Doctrine never wrote. Its old `EX-7` is retained as a superseded marker rather
  than renumbered, per the immutability rule.

## Notes

**PHASE-02 is coordinator work.** ADR-001's tier map at
`.doctrine/adr/001/layering.toml` is the authoritative classification the
layering gate loads (`tests/architecture_layering.rs` auto-discovers units by
scanning `src/` and panics on an unclassified one). Registering `design_run` is
therefore an authored `.doctrine/` edit, not a test edit — the first draft had
the VT mandate pointed at the test file, where the string would never appear.
The path is now a declared design-target selector, and the phase joins PHASE-01
and PHASE-09 as undispatchable.

**The incumbent `slice design` lives in `src/slice.rs`.** `SliceCommand::Design`
at `src/slice.rs:174`, dispatched at `:449`, `run_design` at `:734` — not in
`src/commands/`. Design §5.5's implementation-home list omits it. PHASE-06's
shim work edits it, so it is now a declared design-target selector; without that
the phase would exit into a conformance failure.

**A new top-level command is not free.** `FAMILIES` in `src/commands/cli.rs` is
the only hand-maintained classification of the command surface, and
`families_partition_the_visible_command_tree` panics on any visible command not
in it, then asserts a hard-coded census count. PHASE-04 VA-3 carries both.

**Boot diverges at PHASE-04.** The boot snapshot's command map is generated from
the compiled clap tree through FAMILIES/SPINE (`src/boot.rs`), so
`.doctrine/state/boot.md` goes stale the moment the family is wired — six phases
before PHASE-08 rewrites the authored guidance. That is a *runtime* tier
divergence with no gate behind it (`doctor` has no boot leg), but leaving it
means phases 05–07 run against a snapshot that omits the family they are
building. PHASE-04 VA-4 re-runs `doctrine boot`; no authored guidance moves
until PHASE-08.

**First-use risk in PHASE-07.** `install/hymns/stage/` does not exist at all —
the directory must be created — and `[hymns].seal` holds exactly one entry
(`preamble/core`), though the `stage` band is registered in
`install/hymns/README.md` and `design` is already a valid `KNOWN_STAGE_LABELS`
member. `stage/design` will be the first stage-band hymn ever authored. The
band's happy path has never run, so PHASE-07 exercises it before relying on it
rather than discovering the gap through a failing install test.

**PHASE-07 must NOT touch `src/install.rs`.** The first revision asked it to
correct the `KNOWN_STAGE_LABELS` comment so that code and design.md §7 agree.
At head the comment already carries both facts design §7 rests on — that the set
is provisional, and that `check` flags any `stage`-band label outside it — and
`design` is already a member. EX-6 is now the negative obligation: leave the
constant alone, and do not read §7's rejection argument as licence to extend it.

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

**`skills-lock.json` drifts when a SKILL.md changes.** It is tracked and holds a
`computedHash` per skill. PHASE-08 rewrites two bodies, so PHASE-08 EX-8 commits
the regenerated lock — otherwise the drift contaminates the next agent's
`git status`, which is exactly the shared-index hazard AGENTS.md warns about.

**VT mandates match raw, case-sensitive substrings.** `src/vtgate.rs` runs
`source.contains(kw)` over the unmodified file and additionally requires the
`test_file` to have been modified by the slice. Four of the first draft's
mandates could not have passed a correct implementation: lowercase stage names
against PascalCase Rust variants, a `DesignTurnEnvelope` the design calls
`TurnEnvelope`, a `"slice design"` CLI invocation that appears in test source as
`["slice", "design"]`, and the layering-test misfire above. Where a mandate now
depends on a name, the corresponding exit criterion fixes that name, so the
gate is self-enforcing rather than a guess about how the implementer will write
it.

**The research packet is gitignored.** `.doctrine/slice/233/research/` exists in
the primary working tree only. It will not travel to a fresh clone or a dispatch
worktree, and `rm -rf` of the runtime tier destroys it. The plan load-bears on
its Round 2 threads; the durable residue is in `notes.md` § Harvest.

**Worktree state is not shared.** A fork carries its own `.doctrine/state`
directory rather than the parent's — verified at plan time against the live
coordination trees. This is why PHASE-09 is coordinator work and why no
dispatched phase can exercise a live design run.
