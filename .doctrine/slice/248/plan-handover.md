# SL-248 — planning handover

Written 2026-08-08, at design lock. For the agent (or agents) authoring the
implementation plan. Committed rather than left in `.doctrine/state/` because it
is meant to survive several sessions and the state tree is disposable.

---

## Where things stand

The **design run is locked**: `dr-019fd432-6e5e-7282-b425-19f07630a627`,
revision 85, `design.md` materialised and watermark current. Nothing in the run
is open — 10 of 10 inquiry nodes resolved, no blockers, all nine sections
attested, both checkpoint acts recorded.

`RV-346` — the adversarial pass — is **done, concluded**: nine rounds, 38
findings, every one at a terminal disposition. It is the review of record for
this design. `RV-347`, the run-minted pass, has zero findings and was never
used; the run's pass was disposed `Waived` with a reason pointing at `RV-346`.
That is not a weak review, it is `ISS-322` — read it before you draw any
conclusion from the word "waived".

Slice lifecycle is still `design`. The next transition is
`doctrine slice status 248 plan`.

**Do not reopen the design.** It has been attacked for nine rounds. If planning
genuinely surfaces a design defect, that is a back-edge through `/consult`, not
a quiet edit — and `design.md` is watermarked, so a hand-edit costs a recovery
cycle (`ISS-320`).

---

## The next action

```bash
doctrine slice status 248 plan       # lifecycle advance
doctrine slice plan 248              # scaffolds plan.toml + plan.md
```

Then author the **spine** before anything else (below). `doctrine slice phases
248` materialises runtime tracking from the plan and comes *after* the plan
declares its phases — not before.

`plan.md` is an ordinary authored file. It carries **no watermark** and none of
`design.md`'s adoption machinery, so edit it normally.

---

## Recommended shape: three stages, so this survives 2–3 sessions

The plan is large (see the seam table below) and will not fit one comfortable
session. The failure mode of splitting it is incoherence — phase 7 written in a
context that has forgotten what phase 3 promised. What follows is structured
specifically against that.

The load-bearing constraint: **`PHASE-NN` ids are immutable and edits append,
never renumber.** So a phase list that turns out wrong in session 2 cannot be
repaired by renumbering — only by appending a phase that then sits out of
logical order forever. The phase list is therefore the one artefact that must
be right the first time, and it is what stage 1 exists to get right.

### Stage 1 — the spine (one session, whole design in context)

Author the **complete phase list and nothing else**: for each phase, its
`PHASE-NN` id, a one-line objective, the files/surfaces it owns, its dependency
order, and — importantly — **which `design.md` sections it is expanded from**.

Explicitly *not* in stage 1: `EN-`/`EX-`/`VT-` criteria. Those are local, they
are the bulk of the writing, and deferring them is what makes stage 1 cheap
enough to do in one sitting with the entire design in view. Sizing judgement is
the deliverable here, not detail.

The section-provenance column is the anti-drift device. It means a later session
reads only its own phases' sections rather than all ~4900 lines, and the
integration pass can check design coverage mechanically instead of trusting
anyone's memory.

### Stage 2 — criteria expansion (one or more sessions, disjoint phase sets)

Each session takes a contiguous, disjoint set of phases and writes their
`EN`/`EX`/`VT`. Coherence holds because the spine is frozen, file ownership is
declared so two phases cannot both claim `backend.rs`, and each session's input
is *design.md's relevant sections plus the spine* — never the previous session's
prose. Do not chain sessions through narrative.

### Stage 3 — integration pass (one session)

Check the seams, which is the only thing no stage-2 session can see:

- every phase's `EN` is discharged by some earlier phase's `EX`, or by the
  slice's entry state — no phase entering on a condition nobody establishes;
- no `VT` duplicated across phases, and the union covers sec-8's evidence table;
- the checked-set change is in the first crate-creating phase (see the trap);
- the section-provenance column covers all nine sections — anything unclaimed is
  either out of scope or a missing phase.

### One refinement on "vertical slices"

Worth being deliberate about the word. A vertical slice normally means thin and
end-to-end, but this design's own seam — `sec-8`'s evidence table — is cut by
**surface/module**, and it was sized that way deliberately. Forcing verticality
onto it would cut across the test groupings the design already costed, and the
phase sizing would stop matching its stated rationale.

So: cut the *planning sessions* on the design's existing seam, because that is
what makes them disjoint. Reserve the vertical property for the *phases* — each
should leave the tree green and self-contained. The design already guarantees
this is possible and says so: **"every slice but the last lands as tested
machinery sitting beside the incumbent worktree arms, unused, with no flag
day."** There is no cutover in SL-248, so no phase needs a big-bang landing.

---

## The phase seam, from `sec-8`

`sec-8` is the phase-sizing authority and has done much of this work already.
Its evidence table:

| surface | tests | lands in | kind |
|---|---|---|---|
| `sec-2` argv, closure, vocabulary, placement validation | ≈39 | `backend.rs`, `backend/bubblewrap.rs` | pure |
| `sec-3` export, clone, rollback, ownership | ≈22 | `provision.rs`, `transaction.rs` | pure + trusted-side |
| `sec-4` parse, normalization, hash, restriction | ≈42 | `src/interpretation.rs`, root package | pure |
| `sec-5` capacity, configuration, root resolution | ≈33 | `config.rs`, `capacity.rs`, `host.rs` | pure, fixture `HostFacts` |
| `sec-6` export-set + two-tree gate | 7 | `tests/architecture_layering.rs` | integration, root package |
| `sec-7` classification + verdict algebra | ≈20 | `conformance.rs` | pure |
| `sec-7` tables A (14 rows), B (5), C (4) | — | `conformance.rs` | executed, one fixture per run |

Roughly 160 pure tests plus 23 executed rows, over a new workspace member
(`doctrine-control`) and a new root lib target. `sec-8` states the counts are
indicative for phase sizing and that **each section's own list is the
authority** — and that two titles are named by two sections and belong to one
test, so the totals are not additive.

---

## The sequencing trap — put it in phase 1

`sec-8` records this and it is the single most expensive thing to get wrong:

> These tests live in the new crate, which `just check` would not build by
> default — it is root-package only, and **a suite that is never built is green
> by never running.**

If the crate lands before the checked set changes, every subsequent phase ships
vacuously-green tests and no gate notices. The checked-set change is **not** a
tidy-up at the end; it belongs in the first phase that creates the crate.

Two edges bite at the same moment:

1. **`cargo test` starts building the library's own test targets.** The export
   set must be transitively closed over `crate::` paths in `#[cfg(test)]` code
   as well as production code — `git.rs`'s test module already reaches
   `crate::kinds`. `sec-8` notes this surfaces only when the library's tests
   first compile.
2. **The nix `srcWithDist` graft.** `craneLib.cleanCargoSource` filters to
   `.rs`/`.toml`/`.lock` only, so a dropped embed root ships a hollow binary
   with **no compile error**. Scope `R5` ties this to the binstall asset
   contract: they move together with the crate. Validate with `just nix-build`
   (host only — nix is absent in the jail).

Also live: `R4` — `bwrap_core_argv` is asserted byte-equivalent to
`scripts/pi-spawn-confined.sh` (`jail.rs:1215`). Any change to it fails a test
whose entire purpose is to notice. Plan around it rather than through it.

---

## Constraints and gotchas

- **`PHASE-NN` / `EN-` / `EX-` / `VT-` ids are immutable.** Edits append.
- **Another agent is live in this repo.** `SL-249`/`SL-250`, `src/review.rs`,
  `src/commands/guard.rs`, `src/mcp_server/tools.rs`,
  `.doctrine/observations/records/43`, and an untracked `.doctrine/rfc/027/`
  tree are theirs. **Path-limit every commit** — `git commit <paths> -F -`,
  never pathless, because a shared index means their staged files ride yours.
- `flake.lock` is dirty and was already dirty. Leave it.
- Primary worktree stays on `edge`. Never `git checkout <ref> --`.
- Shell backticks mangle `--response` / `--detail` text. Write to a file and
  pass `"$(cat file)"`.
- `doctrine validate` after authored edits; `doctrine check quick|commit|gate`
  for code. Documentation-only changes need `doctrine validate`, not a build.

---

## What not to redo

- The design. Locked, and reviewed nine times.
- `RV-346`. Closed and concluded; its findings are all terminal.
- Any part of the credential-row split, row 10's or row 12's mutants, or the
  `sec-7` executed-test list. These were the last three rounds' subject matter
  and they are settled.
- Reading this session's review history. `notes.md` § Harvest carries what
  survived; the rest was working state.

---

## Continuation prompt

```
/route

Plan SL-248. The design is locked (design run dr-019fd432, revision 85) and
the slice is at lifecycle `design`.

Read `.doctrine/slice/248/plan-handover.md` first — it carries the recommended
three-stage split (spine, then criteria expansion, then integration), the
phase seam from sec-8, and the checked-set trap that must land in phase 1.

Then: `doctrine slice status 248 plan` and `doctrine slice plan 248`.

This session's job is stage 1 only — the complete phase list with objectives,
file ownership, dependency order and design-section provenance. No EN/EX/VT.
Phase ids are immutable, so the list is the artefact that must be right first
time; the criteria are local and can be expanded in later sessions.
```
