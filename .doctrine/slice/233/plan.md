# Implementation Plan SL-233: CLI-managed design runs and inquiry maps

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Sixteen phases, serial. Execution order is the `plan.toml` array order:

    01 → 02 → 03 → 04 → 05 → 06 → 13 → 14 → 11 → 12 → 10 → 07 → 16 → 08 → 09

Phase ids are immutable and append-only, so the three phases added by the
2026-07-28 revision carry `PHASE-10`…`PHASE-12` while sitting mid-sequence, the
2026-07-29 split added `PHASE-13`/`PHASE-14`, and the 2026-07-31 design
conversation added `PHASE-16` between `PHASE-07` and `PHASE-08`.
`src/plan.rs` reads the ordered phase array and never sorts by id, so the array
is the authority; the ids are names, not positions. Appending a phase and then
*placing* it is therefore the supported move — a renumber is not.

`PHASE-15` is the one phase whose array position does not state its execution
position: it was appended at the tail and executed thirteenth. It is
`completed`, so it is left where it is rather than churned; every phase that
remains sits in execution order.

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
  **Discharged, and now inherited by three phases.** RV-323 reached terminal at
  round 30 with 6 of 6 findings verified against rev 5 of the sketch. The
  2026-07-29 split (below) left the gate stated at PHASE-06 `EN-2` and inherited
  verbatim by PHASE-13 `EN-2` and PHASE-14 `EN-3`. Splitting a gated phase does
  **not** re-arm its gate: one sketch, one adversarial ledger, three consumers.
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

### PHASE-16 — why a sixteenth phase, and why it runs before PHASE-08

Added 2026-07-31, out of the design conversation the owner ordered in place of
PHASE-08's six-question `EN-2` procedure. Design: `sketches/runbook-runner.md`;
decisions: DEC-101 (the runner) and DEC-102 (the customization line).

The conversation was driven by two D2 models of the design workflow as the skill
prose describes it against the instruction-emitting surface as built. They do not
match, and the pattern is sharper than the mismatch count: **Doctrine took the
state and the invariants; it did not take the checklists or the craft.** Applying
the owner's test — instruction should arrive as "do this now", and an invariant is
by definition not a trigger — no part of the built surface is in the "do this now"
register at all. PHASE-16 adds it: an ordered-obligation primitive whose steps are
TOML asset data and whose optional verifiers are arbitrary executables.

The mechanical form of the gap is sharper than the register argument. The
`Exploring → Inquiring` edge already exists and already carries a guard —
`GoverningContextRecorded` and `InitialConcernsRecorded` — but **both are
caller-claimed**, specified in nine words at `design.md:268`, named in no shipped
guidance whatsoever, and bound to an arbitrary draft section's fingerprint. The
edge exists, the guard exists, and the guard is two booleans an agent declares.
The runbook is the first real guard that edge has ever had.

**It runs before PHASE-08 because PHASE-08 cannot honestly happen first.**
PHASE-08 `EX-1` claims the workflow machine's deterministic mechanism now lives in
Doctrine. Without a primitive to move the checklists *into*, that rewrite can only
drop them — losing them silently — or keep them, staying fat and failing `VA-4`'s
anti-theatre size comparison against the incumbent's 214 lines. The runner is what
lets PHASE-08's rewrite be true rather than merely shorter.

**The split between them is deliberate.** PHASE-16 ships the mechanism plus ONE
runbook — `exploring`, the sharpest instance: the state with the most explicit
ordered checklist and no fragment of its own (`Fragment::for_stage` maps
`Exploring | Inquiring → Fragment::Inquiry`, and `inquiry.md` is entirely about
nodes). PHASE-08 does the remaining prose work as part of the body rewrite it was
already scoped as. Prose is cheaper than mechanism, and splitting them this way
balances the two phases rather than loading one.

> **Amended 2026-08-01** (owner ruling; RV-325 round 7, F-11 second contest).
> This paragraph continued: *"PHASE-08 converts the remaining two prose
> checklists … **and lands `set` mode with them** — deferred out of PHASE-16 at
> the second review, because a mode with no shipped user and no reachable
> end-to-end test is speculative generality, and PHASE-08 will have two real
> coverage-set instances to design it against."* **Every clause of that has
> lapsed.** D14 reclassified both named checklists as stage framing, so PHASE-08
> converts neither; F-5's settlement then produced five order-independent steps
> under `sequence`, so the *real instances* precondition is met by different
> instances; and *no shipped user* is **withdrawn as false wherever it appears**.
> `set` is deferred to **IMP-373, not to PHASE-08** — what defers it is its
> **render**, a cursorless runbook under `EX-14`'s token bound, unsketched and
> outside PHASE-08 `EN-2`'s scope. DEC-101 carries the governing amendment in
> both tiers; PHASE-16 `EX-7`/`EX-14` are annotated with their discharged
> obligations preserved. The no-shipped-user argument was true **of PHASE-16**
> and is not a warrant for deferring past it.

This phase takes no new adversarial gate. `EN-2` is the sketch plus the two
accepted decisions — the owner replaced the sketch-as-driver procedure on
2026-07-30 ("design conversation first, paperwork back-solved"), and this phase is
that conversation's product rather than another artefact to attack. PHASE-08's own
`EN-2` gate over `sketches/thin-adapter.md` is untouched and still owed.

It did take one **external adversarial prose review** — deliberately not an RV
ledger, because the owner declined a fifteen-round disposition cycle for what is a
cheap pre-implementation catch. It paid: nine of ten findings stood on independent
verification, two of them changing the design rather than its wording. The
criteria above were therefore amended once, **before the phase started and before
any evidence was recorded against it**; ids are unchanged and `EX-16`/`EX-17` are
appended rather than renumbered. The alternative — appending criteria that
contradict the ones they correct — would have produced a self-contradictory set,
which is worse than an amendment made in the same session that authored it. The
two out-of-scope residuals are IMP-372 and ISS-284.

**It then took a second one, and that is a pattern worth naming.** A fresh codex
session reviewed revision 2, because revision 2 had rewritten precisely the parts
revision 1 got wrong and those repairs had been checked by nobody. Three
structural findings stood on verification: the runbook had **no selector**
(`Fragment` is a closed enum with no exploring variant), the `Condition`
vocabulary **cannot express** the gate it had been given (payload-free,
existential, and its refusal has no room for a step id), and the
verifier-results-as-payload path **does not exist** — the obvious form of it is
refused by an existing engine invariant. So the criteria were amended a second
time, again pre-start and pre-evidence, with `EX-18`/`EX-19` appended.

Two amendments in two passes is disclosed rather than absorbed. Each review found
the previous round's repairs resting on false claims about the code, which is
evidence about the *method* rather than about any one author: a design conversation
cannot hold enough of this surface in view at once to be right by inspection. The
owner's ruling (2026-07-31) is therefore to stop seeking a third pre-implementation
pass, ship the mechanism, and correct it from operation — spending the remaining
design attention only on the handful of decisions that harden into contract at the
moment of shipping. Sketch §0 records that sort; §12 carries both disposition
tables.

### PHASE-08 — `VA-4` amended, `VH-1` appended (2026-08-01)

Same pattern as PHASE-16's two rounds, same conditions, disclosed the same way:
**before the phase started and before any evidence was recorded against it**, ids
unchanged, the withdrawal recorded in the criterion that lost it rather than left
to be inferred.

`VA-4` asked an agent to compare the rewritten `SKILL.md` against the incumbent
baseline and judge whether the body *substantially shrank*. Pre-design target-state
work (`sketches/target-machine.d2`) showed the criterion was **defeatable by
construction**: under DEC-101 the checklists move into runbook TOML, a different
file, so shrinkage is satisfiable by relocation alone and a byte count cannot
distinguish relocation from mechanism. That is the same defect class `VA-5` exists
to catch on `EX-9` — a check that reads like evidence and is not — sitting inside
the criterion written to prevent it.

The split follows the `VT-2` precedent set in this same plan: substance relocated,
id retained rather than renumbered, coverage unchanged in total. `VA-4` keeps the
**measurement** and is strengthened into a disposition table that must account for
every part of the incumbent body and state runbook bytes separately, which is what
closes the loophole. `VH-1` takes the **judgement**, because `substantially` is a
judgement term, an agent grading its own rewrite against it is marking its own
homework, and the only reader who can tell craft from ceremony is the one who has
to live with the skill.

`VH-1` is bounded rather than open: it names the scope decisions that make residue
legitimate — `set` mode deferred (IMP-373), the Locked exit retained in the adapter,
selector recording left where the incumbent prose puts it. Residue those decisions
account for is fine; residue they do not is prose that was merely not moved.

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

### The phases that were doing too many jobs

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

#### PHASE-06 again, on 2026-07-29 — the same argument, one round later

Removing conservative import was not enough. What was left still carried **ten
separable deliverables and eighteen named tests** behind one design gate, in a
single phase intended for a single dispatch worker holding a single commit. The
2026-07-28 note above gives the test — *an undesigned surface plus a bolted-on
objective* — and applying it honestly to the residue splits it again, by
**dependency**, not by test file:

- **PHASE-06** keeps the **wire contract**: the `record`/`adopt_record` collapse
  (`EX-12`), the derived title at the declare arm (`EX-13(b)`), and
  `deny_unknown_fields` (`EX-14`). It touches neither markers nor materialise,
  and it goes **first** because `EX-14` is the precondition that makes both wire
  removals falsifiable at all — without it serde accepts and discards a removed
  key, and every removal test passes for the wrong reason. This is also where all
  three wrong-acceptance ordering constraints (`VA-NC2`, `VA-NC3`) now live
  together, which is what they were always describing: one ordered sequence
  through one file.
- **PHASE-13** takes the **write side**: the marker grammar, byte-exact framing
  (`EX-13(a)`), the `DESIGN_ID_BYTES` refusal, `seq` and document-order emit
  (`EX-11(a)(b)`), and the generated round-trip property. It stops deliberately
  short of parsing a document a *human* edited — which lets its round trip be
  proved against bytes Doctrine itself wrote before the same grammar is asked to
  survive a hand edit.
- **PHASE-14** takes the **read side**: DEC-092 rule 2, the five §5.5 refusals,
  re-adopt renumbering (`EX-11(c)`), `DerivedInput` carrying prose (`EX-13(c)`),
  and the deprecated shim.

Two placements are worth stating because they read like deferral and are not.
**`EX-13(b)`'s adoption arm goes to PHASE-14**, not PHASE-06: the hand-edited
document that arm guards against cannot *reach* adoption until PHASE-14 opens
that door, so the guard travels with the door. And **`EX-13` itself stays whole
in PHASE-06** as the argument of record — five revisions of RV-323 reasoning
that no reader should meet in three pieces. Its successors cite it by id rather
than copying it, and its text now carries a partition notice naming where each
of (a), (b) and (c) is discharged.

One obligation changed **shape**, not home, and it is the only one: PHASE-06
`EX-8` said `tests/e2e_design_materialise.rs` contains *exactly* five named
tests. PHASE-13 `EX-8` creates that file and states a **floor** of two, because
an *exactly* there would forbid PHASE-14 from completing a file it is required
to complete; PHASE-14 `EX-8` then restores the *exactly* over the finished file
at **eight** — the original five, plus `EX-9`'s round trip, plus `EX-11`'s two.
No name was dropped, renamed, or invented.

`RV-323`'s round-7 carriage-return obligation, which had no home in the plan at
all and lived only in `notes.md`, lands as PHASE-13 `EX-4`.

Cost of the split: three phase ids where there was one, `VT-1`/`VT-2`/`VT-3`
waived-with-reason in PHASE-06 against live mandates at their new homes, and
`VT-5` narrowed to the one pattern that is still declare-arm evidence. Benefit:
no phase now asks one worker for more than eight named tests, and every
negative control sits in the phase that can actually observe its red.

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
`src/commands/`. Design §5.5's implementation-home list omits it. The shim work
edits it — **PHASE-14's** since the 2026-07-29 split — so it is now a declared
design-target selector; without that the phase would exit into a conformance
failure.

**The stall sweep.** A phase stalls if it must edit a path with no design-target
selector: the work lands, then `slice conformance` refuses it at close, and the
selector cannot be added by the worker because authored `.doctrine/` state is
coordinator-only. Ten more such paths were found by walking each phase's actual
edit set against head, and all are now declared: `.doctrine/routing-process.md`
(the tracked *projected* copy, which carries the same `slice design`
core-process sentence as the embed and so must move with it);
`.doctrine/spec/tech/019/**` and `.doctrine/spec/tech/023/**` (both amendment
targets are tech specs — now known exactly, so PHASE-01 only has two ids left to
append); `src/state.rs` (`STATE_SLICE_DIR` is a private const there, and STD-001
forbids a second literal, so the snapshot path helper must live in that module);
`tests/common/**` (shared e2e helpers); `tests/e2e_help_families_golden.rs`,
`tests/e2e_subcommand_help.rs` and `tests/e2e_boot_map_golden.rs` (all shift when
a top-level command family is added); `install/hymns/README.md` (the band
registry); and `Cargo.toml`. `sha2` is already a workspace dependency, so
fingerprinting adds none.

**What PHASE-08 must NOT converge.** A repo-wide `git grep 'slice design'` hits
ADR-005, ADR-014, DEC-075, QUE-190, a memory item, three revisions, and REQ-089 /
REQ-384. Every hit is either a historical record correctly naming the verb of its
own era, or a noun-phrase false positive ("the boundary against slice design",
"its concrete home … are slice design"). None mandates the verb, so PHASE-01's
four-entity descent stands and only the two routing-process files move. PHASE-08
VA-6 fences this explicitly, because "converge shipped guidance" is exactly the
instruction an over-eager implementer answers by editing accepted ADRs.

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

## Verification under an untrusted implementer

The implementation is being written by a model that must be assumed lazy,
literal-minded, and willing to make a gate green without making the code right.
The plan is therefore constructed so that **no criterion can be satisfied by a
claim** — every one resolves to a command with an exit code, a grep result, or a
recorded failure output. The relevant facts:

**A VT mandate proves less than it looks like.** `src/vtgate.rs::check_vt` never
runs a test. It checks that a slice-modified file contains each keyword as a raw
substring — over unstripped source, so a comment satisfies it — and that each
`patterns` regex matches at least one line. Four of the first draft's mandates
could not have passed a correct implementation at all (lowercase stage names
against PascalCase variants, a `DesignTurnEnvelope` the design calls
`TurnEnvelope`, a `"slice design"` CLI invocation that appears in test source as
`["slice", "design"]`, and the layering-test misfire above) — and every one of
them, once fixed, would still have been satisfiable by a comment.

Four devices carry the real weight. Every code phase uses all four:

1. **`patterns` anchored at line start on a declaration shape** —
   `^\s*fn name\(`, `^\s*pub(\(crate\))? enum Stage\b`,
   `^\s*(mut\s+)?on_reserved:`. A comment line begins with `/`, so it cannot
   match. Each shape was positive-controlled against existing code before being
   written into the plan; a pattern that can never match is worse than no
   pattern.
2. **Exit criteria that enumerate the exact test function names.** Sixty-odd
   names across the phases. The names are the contract, the patterns anchor on
   them, and a paraphrased or merged test fails the gate. Where a mandate
   depends on a name that does not yet exist — `Stage`, `TurnEnvelope`,
   `on_reserved`, `design_snapshot_path`, a `DEPRECAT*` constant — the exit
   criterion fixes that name and says so, and renaming it is a plan revision
   rather than a local decision. Anchoring on a name that already exists
   (`run_design`, `claim_fresh_id` alone) would pass vacuously, which is why
   those anchors moved.
3. **A negative control per phase (`VA-NC`).** Every named test was observed RED
   before it was made green, with the failing output recorded in the phase
   sheet. This is the only device that catches a test which passes because it
   asserts nothing, and several phases constrain what the red must be: for the
   PHASE-12 lock refusals it must be a wrong *admission*, for the PHASE-06 wire
   removals and the PHASE-14 §5.5 refusals a wrong *acceptance*, for the
   PHASE-13 round trip a byte *inequality* against the incumbent framing, and
   for the PHASE-03 rule-3 tests a behavioural failure rather than a compile
   error. A test never seen red is not evidence.

   **On the dispatch arm, a worker cannot discharge this alone.** A worker fork
   carries its own gitignored `.doctrine/state`, so red output written into a
   fork's phase sheet dies with the fork — it looks like success and loses the
   evidence. The worker hands the transcripts back; the *orchestrator* writes
   them into the coord phase sheet. PHASE-13 and PHASE-14 `VA-NC` say so in the
   criterion rather than leaving it to be rediscovered.
4. **An escape sweep per phase (`VA-ES`).** No `#[ignore]`, `todo!()`,
   `unimplemented!()` or `assert!(true)` under the phase's paths, and a test
   census that never shrinks — the four cheapest ways to turn a red suite green.

Beyond those, each phase carries **anti-theatre criteria** aimed at its own
specific way of being faked: a bounds test whose fixture never reaches the
bound; five distinct refusals collapsing into one `is_err()`; a shim-equivalence
test comparing exit codes instead of bytes; a lock-gate test that omits three
components and proves nothing about which one the gate read; a similarity
function that makes `text similarity never merges` false regardless of what any
test asserts; a core-process test with a hardcoded command list wearing a
parser's name. These are written as instructions to the *coordinator*, who runs
the VA rows — the worker does not self-certify.

The two design gates that were "a written sketch and an adversarial pass" now
enumerate the questions the sketch must answer — seven for projection bounds,
seven for the marker grammar, six for the thin adapter — and require the RV
reviewer to be someone other than the sketch's author. A sketch missing any
listed answer is not a sketch, which is what stops the gate degrading into a
paragraph and a rubber stamp.

**The research packet is gitignored.** `.doctrine/slice/233/research/` exists in
the primary working tree only. It will not travel to a fresh clone or a dispatch
worktree, and `rm -rf` of the runtime tier destroys it. The plan load-bears on
its Round 2 threads; the durable residue is in `notes.md` § Harvest.

**Worktree state is not shared.** A fork carries its own `.doctrine/state`
directory rather than the parent's — verified at plan time against the live
coordination trees. This is why PHASE-09 is coordinator work and why no
dispatched phase can exercise a live design run.
