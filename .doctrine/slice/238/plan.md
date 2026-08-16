# Implementation Plan SL-238: Cross-kind dep/seq edges: disclose, check, and clear

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See glossary.md § reference forms. -->

## Overview

Eight phases, derived from `design.md` §8's file list and the design's own
decomposition — **not** from the slice title and **not** from the type
prototype's diff. `SL-238`'s scope grew five times during inquiry, and §8's R5
says plainly that the phase plan must be built against the file list rather than
against "make the footer honest". It is.

The design has one substrate and three surfaces standing on it:

- **the substrate** — a per-kind authored-status read that no surface may
  duplicate (§3). PHASE-01.
- **the report surface** — `doctor`, where refs that name nothing now go (§5).
  PHASE-03.
- **the listing surface** — the `boundary:` block, the reshaped `overrides:`
  block, the stderr signpost, and the record annotation on `inspect`/`show`
  (§2, §4). PHASE-04, PHASE-05.
- **the repair surface** — `needs --remove`, the source-only remove gate, the
  collapsed `--prune` probe, and the injection that lets `backlog`'s verbs run
  one implementation rather than a duplicate (§6). PHASE-06, PHASE-07,
  PHASE-08.

PHASE-02 sits between the substrate and the surfaces for one reason given in
§7: behaviour this slice deliberately changes must have its before-state pinned
first, or the change is invisible in the diff and indistinguishable from a test
that was always going to pass.

## Sequencing & Rationale

**PHASE-01 first because everything reads terminality through it.** The
`boundary:` block suppresses terminal targets, `--prune` drops them, and
`inspect` annotates them — three surfaces asking one question. `DEC-233`'s loud
rule is that no second terminal-status vocabulary may exist, and the only way to
hold that is to land the single reader before any consumer needs it. PHASE-01 is
also entirely file-disjoint from the rest of the slice (`kinds`, a new module,
`partition`, `catalog::scan`, `main.rs`), so it can run alongside PHASE-02
without contention.

**PHASE-02 before any behaviour it pins is changed.** `--prune` has no test
coverage at all, in either of its two copies (R3), and this slice changes it four
times over — the terminal vocabulary, the reason word, the silent keep on an
unreadable target, and a bare ref resolving instead of being deleted. `after
--remove`'s target gate is the fifth. The phase adds tests only; its diff must
contain no production statement. Its tests are then **deliberately superseded**
by PHASE-06 and PHASE-07, which is why `EX-4` requires each pin to name the
phase that will supersede it — a superseded pin and a regression look identical
six months later otherwise.

**PHASE-03 before PHASE-04, because the advisory has to agree with the check.**
§2's stderr signpost counts distinct `(dependent, axis, ref)` occurrences and
§5's check reports the same population, undeduplicated, over every item
including terminal ones. §2 states why that matters: *a signpost that points at
`doctor` for a subset of what `doctor` will say is worse than no signpost.*
Landing the check first gives PHASE-04's count a reference implementation to
agree with rather than a specification to re-derive. PHASE-03 is also the one
phase with **no prototype code whatsoever** — `DEC-242`'s third consequence
names it explicitly, and its planner and code author must treat it as greenfield
rather than as a diff against something.

**PHASE-04 is the largest phase and could not usefully be smaller.** The
projection change, the probe, the footer reshape and the deletion of
`classify_dangling` are one edit: `render_overrides` loses its `corpus`
parameter *because* the `Dangling` arm goes, `compose` loses its `cmap` build
*because* nothing reads it, and the repo's `dead_code` denials mean leaving any
of it half-done does not compile. Splitting it would mean landing an
intermediate state that the toolchain rejects.

**PHASE-05 is split out of PHASE-04 despite sharing a file.** `probe_item_refs`
is a second projection with different rules — it keeps what the footer drops —
and it lands in a different function (`run_show_inspect` / `format_metadata`,
already at seven positional arguments, four of them dead). Folding it into an
already-maximal phase buys nothing and costs the ability to bisect a footer
regression from an annotation regression.

**PHASE-06 before PHASE-07 before PHASE-08.** The remove seam (`RelRemove`, the
single IO wrapper) is what `--prune`'s collapse then rides, and the injected
`DepSeqOps` pointers must point at the *final* implementations — a pointer
installed at PHASE-06's intermediate `run_after_remove` would have to be
re-pointed twice. PHASE-08 last also puts the layering measurement at the end,
where it belongs: §6's whole injection argument exists because a
`backlog → commands` edge merges two SCCs, and `layering.toml:185-187` says to
measure rather than predict.

### File contention and parallelism

`src/backlog.rs` is touched by PHASE-03, PHASE-04, PHASE-05 and PHASE-08;
`src/commands/dep_seq.rs` by PHASE-02, PHASE-06, PHASE-07 and PHASE-08. The
serial order above is conflict-free. If phases are driven through `/dispatch`,
the only genuinely file-disjoint pair is PHASE-01 with PHASE-02; everything else
must run serially.

### Staffing: three seats, and one of them is blind

`DEC-242` governs how the type prototype (`proto/SL-238-types`, a disposable
uncommitted fork) may inform this work, and it binds every phase. Each phase is
staffed by three seats — **planner**, **test author**, **code author**. The
planner and the code author may read the prototype as an oracle to critically
evaluate; **the test author may not**, and writes the red suite from `design.md`
§7 and the design alone.

Two consequences the phases have to carry:

- Every output string a test asserts is quoted in `design.md`, not read out of
  the prototype. Where the design does not supply one, that is a design gap to
  escalate through `/consult` — not a licence to go and look.
- The code author is the only seat holding both the red tests and the skeleton.
  On any design/prototype conflict it escalates rather than picking a side.

The prototype is an **oracle, not a source**. Two rounds found four real defects
in `design.md`; round 2 found none and proved the fixes. Its `.doctrine/` is
stale, its `layering.toml` edit is fork-local, and it carries no tests, no lint,
and none of the `#[derive]`s §7's tests need on `RefState` / `BoundaryRow` /
`BoundaryProbe`.

### Test naming and the VT mandates

`design.md` §7 names roughly thirty assertions in prose. Those names are the
test names, normalised to snake_case — that is what makes the `VT` keyword
mandates in `plan.toml` checkable, and it is what lets an auditor walk §7 and
the suite side by side without a mapping table.

Two consequences for the `VT` keyword mandates in `plan.toml`, both of which
bit while drafting:

- **`slice verify-vt` runs slice-wide at every conclude, not per phase.** So a
  keyword may not name anything this slice later deletes. PHASE-02's mandates
  are therefore keyed on the production symbols that survive
  (`run_after_prune`, `run_after_remove`, `prune`) rather than on the strings
  its characterisation tests assert — the `/resolution` suffix, the
  `resolved`/`closed` literals — all of which PHASE-07 removes on purpose. The
  assertions themselves live in each row's `expects`. The mandates on that
  phase are correspondingly weak; `VA-1` (the diff is test-only) is what
  actually holds it.
- **A `VT` mandate is only attributable if the phase modifies the mandated
  file.** PHASE-08's layering assertion is a `VA`, not a `VT`, because the
  phase modifies no file carrying the evidence — the row for `authored_status`
  and the `command = 76` baseline are both PHASE-01's edit. A `test_file`
  mandate there would read `UNATTRIBUTABLE` forever and look like a gap.

### Per-phase hygiene

Project standing obligations, not restated as criteria on eight phases:
`doctrine check quick|commit` as the inner loop, `just gate` (clippy at zero
warnings plus `test-all`) before every commit, `cargo fmt`, and path-limited
commits scoped `feat(SL-238):` / `test(SL-238):`. `doctrine check gate` belongs
to close, where the build-before-validate order is what gives it a fresh
binary.

## Notes

### What the plan deliberately does not do

- **No node admission, no comparator change, no adapter change.** R1 is
  dissolved by non-goal, and `src/backlog_order.rs` and the `priority` modules
  are untouched. PHASE-04/EX-8 makes "the existing suites are green
  **unmodified**" an exit criterion rather than an aspiration: a suite edited to
  pass is a finding, not a pass.
- **No widening of the `doctor` check to other dep/seq-authoring kinds**, and no
  admissibility judgement inside it. Both are §9 follow-ups; folding either in
  would be this slice's sixth scope growth.
- **No fix for the unpadded-ref bound** (`SL-1` stored, `SL-001` sought).
  PHASE-06/EX-4 asserts the bound instead of the capability, so closing it later
  is a deliberate change with a red test.

### Open items the phases carry forward

- **The `Unavailable` fixture's second route.** §7 fixtures the `Unavailable`
  case twice — hand-authored, and CLI-authored via `doctrine backlog needs <ISS>
  <RV>`, which succeeds today — and says both are kept *after* §6 shuts the CLI
  route. A test that shells the live CLI for that fixture goes red the moment
  PHASE-08 lands. This plan reads §7 as requiring the CLI-authored fixture to be
  materialised as the stored form the CLI writes (so it survives the gate), with
  the live-CLI assertion becoming PHASE-08/VT-3's refusal test. If the test
  author reads §7 differently, that is a `/consult`, not a judgement call.
- **`format_metadata`'s parameter list** reaches eight positional arguments at
  PHASE-05, three of the existing seven already dead. The `ShowContext` collapse
  is the right cleanup and is explicitly out of this slice's blast radius (§9).
- **`RV-358` was waived, not conducted.** Ten findings sit on the ledger
  undisposed by design; five were never verified by the raiser. Four repairs in
  §6/§7 are self-authored and have never been adversarially read — the
  `DepSeqOps` fourth member, `project`'s second `AbsentDrop` case, `inspect`'s
  second annotation clause, and `--prune`'s resolver swap. The prototype
  exercised the first; the other three rest on their author's judgement. Each
  lands in PHASE-08, PHASE-04, PHASE-05 and PHASE-07 respectively, and a phase
  whose red suite cannot be written from the design is the signal that one of
  them is underspecified.
- **`ISS-368`** is the shipped half of `RV-358` `F-5` and is closed as a
  side-effect by PHASE-08/EX-4. **`CHR-068`** — the stale `command=120` comment
  at `tests/architecture_layering.rs:8,22`, against the real `command = 76` in
  `layering.toml:190` — is *not* folded in; it is unrelated to this slice's
  surfaces.
- **The corpus figures in §1 are a dated snapshot and drift daily.** §1 says so.
  A differing count is not a defect to fix without re-scanning.
