# Notes SL-230: Memory body-write verbs and corpus-aware verify gate

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-07-26 · ready (design locked; plan approved, 0/6 phases; mode
dispatch — see § Execution constraints) · 4848a6a3. The Harvest body below is
fresh as of plan/68bf7f21; only Next and the header were carried forward by the
readiness pass. A full harvest is `/harvest`'s job, not that pass's.

### Produced

- **plan.toml / plan.md** — six phases, split by seam not by verb (`record` rides
  the scaffold fileset, `edit` rides the new `write_body`). Phase sheets
  materialised. Awaiting user approval before `ready`.
- **DEC-027** — the corpus-aware verify gate leaves SL-230 into **SL-232**. User
  ruling at RV-307 round 8. The governing record for the split boundary, the
  accepted R4 tradeoff, and the RV-307 disposition policy. **Its boundary table
  and finding sort were corrected** by the confirming pass (68bf7f21) — the table
  contradicted its own prose; the decision itself is untouched.
- **SL-232** — corpus-aware memory verify gate. Scope doc + inherited design
  (gate half of this slice's design.md, carried verbatim so eight rounds of
  review are not re-derived). Status `proposed`, design NOT locked, two open
  blockers (F-36, F-37).
- **ISS-247** — no verb removes a `needs` edge; re-pointing REV-034 needed a
  sanctioned TOML hand-edit.
- IMP-317 retitled (F-34/F-39) — the body was correct since round 7, the title
  still advertised the rejected shared-constructor model.
- RV-307 — round-7 raiser adjudication accepted F-19/F-20/F-27/F-29/F-30,
  returned F-6/F-22/F-24/F-25/F-26/F-28, and raised F-31–F-35. All five new
  charges reproduced by the responder. **All 35 findings now disposed
  `fix-now`; `await=raiser`.** Corrects a prior harvest: F-34/F-35 were recorded
  here as disposed when their *fixes* had landed but their ledger entries had
  not — they carried no disposition and no response until 8a625cb5.
- **DEC-020** — non-contribution classification leaves SL-230. User ruling at
  round 7. The governing record for the D10 shrink; read it before editing D10.
- REV-034 — SPEC-007 + REQ-147 amendment. Proposed, applied at close.
- IMP-317 — give `validate` and `retrieve` a history-stable scope dataflow.
- IMP-318 — persist attested coverage on the verification stamp.
- QUE-175 — retrieve-side scope drift question; body corrected per F-34.
- EVD-001, QUE-173 — from the pre-external design rounds.

### Learned

- F-31 — `git rev-list --all` is stable only over the current ref set, not over
  clones or branch deletion.
- F-32 — a textual wildcard-free prefix is not necessarily a resolvable path
  prefix; non-resolving outside pathspecs also abort the history probe.
- F-33 — a weak stamp contract and a strong scope contract cannot coexist as
  separate normative readings.
- F-34 — routing records are part of the integration sweep, not commentary
  outside the design.
- F-35 — pointer-only history must contain pointers only.
- **the dominant cost driver, named** — a property of a tool (stable, total,
  deterministic) is a *claim needing a falsifier*, not a premise. Measuring that
  a discriminator **works** is not evidence it is **stable**. Two of seven rounds
  went on successive locally-varying instruments for one class boundary
  (F-25 → F-31). Recorded in `case-notes.md`; a `/record-memory` candidate, since
  it is not memory-specific.
- **run the query** — D5's defect (below) survived eight rounds because § 10's
  live-data check read a count of 3 as confirmation the plumbing worked; the 3 was
  partly the stamp commit itself. F-17/F-23's defect, third instance, in the
  document that names it twice. When a design states a concrete query, execute it
  against the real corpus. Same `/record-memory` candidate as the entry above —
  both are one lesson at different altitudes.
- **mechanical narrowing damages joints, not blocks** — the reassembly's defects
  were all at seams: a retained paragraph whose justification was rewritten to be
  local and now cited a verb that had left; a routing notice naming a section that
  exists in both documents; an invariant pinned to the nearest-looking test.
  Cheapest instrument by far was `git show <narrowing-commit>^:<path>` — it settled
  the invented-id question, E10's non-existence, and step-numbering provenance in
  one read.
- **no kind has a prose-write verb, and `knowledge` has no `edit` at all** —
  correcting DEC-027 needed a raw-file write of `record-027.md`. This slice's own
  § 2 claim, paid for in the session that authored it. Bears on OQ-4: adoption for
  knowledge is a **new verb**, not a new flag, so D1's "adoption is a caller
  change" understates it there. In `case-notes.md`.

### Open

**This slice.** Design locked, plan authored, nothing blocking. Confirming pass
done (10 mechanical defects + 3 substantive on D5); RV-307 fully disposed.

- **Plan awaits user approval** before `slice status 230 ready` → `/phase-plan`
  PHASE-01. No code without an approved plan.
- **PHASE-03 and PHASE-04 must not be separated.** At PHASE-03's HEAD
  `edit --body` works and the attestation does not clear — strictly worse than
  today, since the footgun currently needs a forbidden raw-file write and after 03
  needs one supported command (§ 1). No intermediate HEAD between them is landable.
  Recorded in `plan.md`; the phase boundary reads innocuous and the consequence
  does not.
- **D5 = `memory.md`, not the item directory** (user ruling; I13, T42). The
  directory form reported drift on 30/30 anchored memories because `verify` stamps
  `verified_sha` and the stamp's own commit touches the directory being measured.
  Body-file form: 11/30. T42 is the falsifier — **a 30/30 result at any point means
  the pathspec regressed.**
- **DEC-027 is still status `proposed`** despite having been taken and executed.
  Left deliberately; settle it or leave it, but do not assume.
- **R4 runs unmitigated** until SL-232 lands. D4's affordability argument depended
  on the gate relaxation; the split removed that support. Accepted by DEC-027 —
  friction, not incorrectness — and the reason to sequence SL-232 next.
- New ids this slice: I10, I11, I12, I13, E14, OQ-7, T40, T41, T42. Moved ids are
  **not** reused. (**OQ-4 is not new** — it pre-dates the split verbatim, so it is
  reviewed text, not reassembly. Next free: I14, E15, T43.)
- OQ-4 — when do other kinds adopt `write_body`? Not here.

**Moved to SL-232 (do not track here).** OQ-2/3/5/6, D3/D9/D10/D11, I1-I4/I6-I9,
E2/E4-E9/E11-E13, R6/R7/R8, REV-034, QUE-173, QUE-175, IMP-317, IMP-318, and
RV-307 F-14/F-25/F-26/F-32/F-36/F-37/F-38 + F-39's first limb.

**F-14 added to that list by the confirming pass.** DEC-027 and this harvest both
counted it among the seven body-write findings, but it lands on I6 and T24 — both
SL-232's — so it was an orphan: named local by SL-230 and absent from SL-232's
inherited set. F-7 (→ R5) is the seventh local finding. Count unchanged.

### Next

1. ~~Readiness check over design / plan / selectors / phase gates~~ — **done**
   2026-07-26. One citation fixed (§ 5.4 `:1808-1810` → `:1827-1828`), T12b
   pinned with its own VT keyword, selector-doctor advisory refused → ISS-248.
2. ~~Plan approval → `slice status 230 ready`~~ — **done**; execution mode is
   **dispatch**, serial. Read `## Execution constraints (dispatch)` below BEFORE
   `dispatch setup`, then `/phase-plan` PHASE-01.
3. `/design` on SL-232, starting from F-37 and F-36 — **and F-14**, which this
   slice handed over. Enumerate-then-probe before asserting any replacement rule:
   that is § 8 R-A and the whole lesson of the eight rounds behind the inherited
   text.

## Execution constraints (dispatch)

Settled 2026-07-26 when the user chose dispatch as the execution mode. These are
constraints on the *orchestrator*, not on any worker — a worker cannot see them
and none is expressible as a phase criterion. Read before `dispatch setup`.

**Serial, not parallel — and know what that buys.** Five of six phases target
`src/memory.rs` (02, 03, 04, 06, and 05's VT-2). The chain is 01→03→04→05 with
04→06. The only file-disjoint pair is PHASE-01 (`src/entity.rs`) ∥ PHASE-02, worth
one phase of wall-clock out of six against a fork, an import, and a conflict
surface on the file PHASE-02 owns. So dispatch is being chosen for **context
isolation and the one-commit-per-phase funnel, not throughput**. Do not let a
later reader re-derive a parallelism benefit that is not there. Note
`dispatch plan-next`'s own `⚠ do not assume file-disjointness` warning is
load-bearing here: `compute_next_phases` (`src/dispatch.rs:3973`) reads status
and plan order ONLY and never consults entrance criteria, so a parallel spawn
over `next` would happily violate PHASE-03's EN-1.

**1. Do not `sync --integrate` between PHASE-03 and PHASE-04.** The inseparable
pair (see § Open) is a *trunk* hazard, not a coordination-branch one: intermediate
coord HEADs never reach trunk, so the whole exposure is integration **timing**.
Integrate no earlier than after PHASE-04 concludes; one integration at the end is
simplest. This is the one place dispatch can actually publish the 03-only state
that § 1 calls strictly worse than today.

**2. PHASE-06 VA-1 runs on the coord/landing tree, NOT in the worker fork.** It
re-measures the live corpus and expects ~11 of 30 anchored memories (30/30 means
the pathspec regressed to the item directory — the registered falsifier). The
measurement is `rev-list verified_sha..HEAD -- memory.md` against real git
history, and a worker fork has a different HEAD and flat topology, so a worker
can false-confirm or false-alarm the single most important check in the slice.
Same reasoning applies more weakly to PHASE-01 VA-1 (ADR-001 layering) and
PHASE-04 VA-1 (read the composed condition): all three are read-and-judge checks,
natural orchestrator work at audit. Workers deliver the VT half.

**3. Watch ISS-028** — worker-marker confinement refuses CLI writes in a stamped
fork, breaking tests that shell the doctrine CLI. Checked, not assumed, at
readiness: `src/memory.rs`'s memory tests are in-process (`GitScratch` + direct
calls), so 02/03/04/06 look clear, and `tests/e2e_mcp_server.rs` spawns the built
binary against a tempdir rather than the fork. Not a known blocker — the thing to
check on first worker return.

**4. Promote edge before setup.** `git fetch . edge:main` then
`dispatch setup --slice 230`, or the worktree forks without the design, plan, and
readiness fixes. Documented footgun; it bites silently.

**Arm.** Not pre-chosen: `/dispatch` routes on a claude↔env-marker agreement, and
the funnel cadence is identical either way (claude arm self-commits via
`worker_commit`; pi arm has the orchestrator import the working-tree diff).

**Arm taken 2026-07-27: claude.** No `[dispatch]` table in `doctrine.toml`, so
`claude-force-subprocess-dispatch` defaults false; `.claude/` present → claude
arm. Coordination worktree `.dispatch/SL-230` on `refs/heads/dispatch/230`.

## Phase trail

Per-phase facts the *next* phase's implementer needs. Distinct from Harvest
(which `/harvest` rewrites wholesale) and from Execution constraints (orchestrator
rules). Append one block per landed phase.

### PHASE-01 — Engine body seam · landed 2026-07-27

Coord `56520c04b → 74620ec5e` (import `290148469`, boundary `74620ec5e`). Worker
model **sonnet**; `src/entity.rs` +190/−0, six tests, no caller wired. Funnel
green: S1 baseline 0 failures at B, `regression diff` no new/changed; R-5 clean.

**`write_body` and `BodyMode` each carry
`#[cfg_attr(not(test), expect(dead_code, reason = "…"))]`, and PHASE-03 must
remove both.** `expect` (not `allow`) fails the build when the lint *stops*
firing, so the attributes self-retire by hard error the moment a caller exists —
deliberate, and the pre-existing project idiom (`src/install.rs:225-228` is the
identical `cfg_attr` form; `src/retrieve.rs:16` and `src/listing.rs:22` document
the same self-retiring phase-scoped pattern, verified live rather than taken from
the worker's word). **PHASE-02 is not the trigger** — it rides the `memory_scaffold`
fileset seam and never calls `write_body`, so the attributes must survive PHASE-02
untouched. PHASE-03 wires the first caller and owns their removal. An implementer
who meets this as a surprise build failure will reach for `allow` or delete the
wrong thing.

**The landed no-op guard is slightly broader than the design's wording.** § 5.2
says "returns false when the write was a no-op (Replace with identical content)";
the implementation compares composed content against existing for *both* modes, so
an `Append` of empty text onto a body already ending in a blank line also returns
false. A harmless superset — and the right shape for D4's `body_changed` signal,
which PHASE-03 reads. No criterion needs restating; noted so it does not read as
drift at audit.

**Reporting nit, no code defect.** The worker returned `Deviations: NONE` while
separately declaring two additions (the `Clone, Copy` derive on `BodyMode`, the
`expect(dead_code)` attributes). Both are sound and additive, and both were
verified here. `Deviations: NONE` is a funnel tripwire precisely because of this
pattern — the mandatory per-phase review it triggered is what confirmed the
`cfg_attr` idiom was real convention and not invention.
