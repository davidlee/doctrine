# Dispatch trunk promotion and close-time edge gate

## Context

Backlog: **IMP-267** (close transition: sync edge with main after dispatch
integration) and **IMP-308 half B** (reliable `edge → main` promotion). They are
the two directions of one seam, with one shared root cause.

**The seam.** In this repo the primary worktree stays on `edge`; `main` is the
dispatch landing zone. `git::trunk_ladder` (`src/git.rs:1609`) resolves trunk
from the candidate set `["origin/HEAD", "main", "master"]` — **`edge` is not a
candidate**, so SL-127's ancestor-dominant selection can never pick it. A
dispatch therefore forks `main`, and any authored content living only on `edge`
is absent from `trunk_base_B`. Symmetrically, after integration nothing checks
that `edge` came to carry the landed code.

**The root cause, in one line** (named verbatim in
`mem.pattern.dispatch.close-split-lineage-reconcile-on-edge`): *the dispatch base
(`main`) was never re-promoted from `edge` after audit.* From there the drive ends
in a 3-way split — the **candidate** has the code (based on old `main`), **`edge`**
has the reconcile/migration authored truth, **`main`** has neither. The naive
`dispatch status` "next" hint then projects the code-only candidate onto `main`,
strands `edge`, and still reaches `done`. Recovering that by hand is the
expensive close documented at SL-224 and mechanised as SL-211's recovery path.

Both directions are ungoverned convention, not pinned invariant:

- `git fetch . edge:main` is a **workflow convention** (AGENTS.md + IMP-129
  prose), not spec-pinned — so hardening it does not relax the governed
  integrate invariant. A fresher `main` simply yields a fresher `trunk_base_B`.
- Advancing `edge` at integrate is already **possible** —
  `dispatch sync --integrate --edge <ref>` advances a standing aggregate ref to
  the `review/<slice>` bundle (`src/dispatch.rs`). IMP-267's gap is that it is
  *optional and unverified at close*, i.e. a missing gate on existing machinery,
  not missing machinery.

**The DRY seam for the gate half is SL-126** (done). It added a *third*
close-gate in `slice::run_status`, alongside the blocker scan and the drift gate,
answering: *is this slice's journaled **trunk**-row OID an ancestor of trunk?* —
three-state **integrated / not-integrated / not-dispatched** (no journal ⇒
silent, so solo slices are unaffected). IMP-267 is the **edge-row analogue of
that same question**: same journal read, same `git::is_ancestor`, same
`reconcile → done` crossing, same one-way `slice` shell → `dispatch` query
direction (ADR-001, no cycle). This slice generalises that gate over the row's
target ref rather than adding a parallel one.

**A hard constraint on the shape of the fix.** IMP-267's card floats
"auto-merge `main` → `edge` as part of close" as an option. That is unsafe as
written, and this slice does not do it:
`mem.fact.dispatch.edge-advance-leg-not-ff-gated` (verified, high trust) —
`plan_edge_row` / `plan_candidate_edge_row` are explicitly commented *"Not
ff-gated"*, and `advance_pure_ref`'s `git::update_ref_cas` guards concurrency
(`current == expected_old`) but **not ancestry**. So the `--edge` leg can already
advance `edge` to a non-descendant tree; only SL-166's g3 corpus-clobber gate
stands between that and a content regression. A gate that *refuses and instructs*
is safe; an auto-advance at close is not. This also honours SL-126's own
Non-Goal ("auto-integrating at close") and ADR-006's orchestrator-sole-writer
posture.

## Scope & Objectives

Two axes, one seam. Each is independently shippable; both ride existing
machinery.

1. **Promotion verb (IMP-308 half B) — `edge → main` before the fork.**
   Promote the `git fetch . edge:main` ritual to a tested CLI verb: advance the
   trunk ref to the aggregate ref, **fast-forward-only + CAS**, refusing rather
   than merging when `edge` does not strictly dominate `main`. Per ADR-011 the
   mechanism lands as a verb with tests; skills only route to it. The verb must
   be framework-neutral (ADR-006, POL-002) — no hardcoded `edge`/`main`; source
   and target come from the existing config/ladder surfaces
   (`[dispatch] deliver_to`, SL-128) so a project not using an edge/main split is
   unaffected.

2. **Close-time edge gate (IMP-267).** Generalise SL-126's third close-gate to
   also answer the **edge-row** question: when the slice's journal carries an
   edge/aggregate row, is its `planned_new_oid` an ancestor of that row's target
   ref? Refuse `reconcile → done` with a named, actionable token when it is not,
   naming the integrate invocation that fixes it. Three-state as SL-126:
   not-dispatched or no edge row ⇒ **silent** (a project that never asked for an
   aggregate ref must not be gated). Composes with the three existing gates —
   any one refuses independently.

3. **Firing-window decision (design-time).** Objective 1's open call: *when* may
   promotion fire? Promoting mid-flight moves `main` under a base a live dispatch
   already pinned — the exact "foreign commit on trunk" hazard SPEC-022's
   moved-trunk **refusal** guards. Candidate answers include pre-`setup` only,
   or refuse-when-a-live-coordination-branch-pins-the-current-trunk. Whichever
   lands must be enforced by the verb, not by prose.

4. **Skill routing.** Wire the promotion verb into the pre-dispatch ritual
   (AGENTS.md + the `/dispatch` setup path) so the documented ritual and the
   enforced mechanism are the same thing.

## Non-Goals

- **IMP-308 half A — relaxing the "no trunk until close" invariant.** Pinned in
  three *accepted* entities: SPEC-022 (post-audit gate; pinned fork-point
  `trunk_base_B = merge-base(dispatch, trunk)`, RV-030 F-1), SPEC-021 (the same
  two-stage audit-gated projection), ADR-012 D5 ("integrate only after audit
  passes", "no close-time merge", SL-068). Relaxing it is a **governance
  revision** (REV against SPEC-021/022 + ADR-012), not a claim tweak. Stays
  resident on IMP-308, or is absorbed into RFC-016 (zero-rescue dispatch).
  Nothing in this slice touches the integrate invariant — a fresher `main` only
  yields a fresher `trunk_base_B`.
- **Auto-merging or auto-advancing `edge` at close.** Refused above on the
  not-FF-gated evidence; the gate instructs, it does not act.
- **FF-gating the `--edge` advance leg itself.** A real latent correctness gap
  (`plan_edge_row` "Not ff-gated") but a change to integrate's own semantics —
  SL-121/SL-166 territory, and it would need its own design call on whether a
  standing aggregate of local work is *allowed* to be non-FF by intent. Filed as
  **RSK-230** (OQ-3, answered at scope time).
- **A trunk advisory lock.** IMP-308's stated prerequisite for the general
  relaxation ("I'm closing, don't touch trunk"). Out of scope; the interim
  operating rule stands — at most one audit-or-close at a time repo-wide.
- **Adding `edge` to the `trunk_ladder` candidate set.** Would make trunk
  selection host-convention-dependent, against POL-002 and ADR-006 D3.
- **The split-lineage *recovery* path.** SL-211 / `sync --record-integration`
  already handle recovery after the fact. This slice reduces how often it is
  needed; it does not re-cut it.
- **Other close crossings.** As SL-126: `reconcile → done` only.

## Affected Surface

Coarse, scope-fresh fence — the exact touch-set is `/design`'s job.

- `src/dispatch.rs` — journal rows (`plan_edge_row`, `advance_row`), the
  integration query SL-126 shares with `--show-journal-trunk-oid`; home of the
  new promotion verb's engine half.
- `src/slice.rs` — `run_status`, where the existing three close-gates live.
- `src/git.rs` — `is_ancestor`, `update_ref_cas`, `trunk_ladder` /
  `freshest_descendant` (read-only reuse).
- `AGENTS.md`, `.agents/skills/dispatch*` — routing for objective 4.

## Risks & Assumptions

- **R1 — gate false-positives on solo slices.** A gate that fires without an
  edge row would block every non-dispatch close. Mitigated by SL-126's
  three-state shape; the absent-row case must be silent, and that is a test.
- **R2 — firing-window hazard.** Promotion that moves `main` under a live pinned
  base trades one failure for another (OQ-1 / objective 3).
- **R3 — POL-002 leakage.** `edge`/`main` are *this project's* convention.
  Baking either name into the engine is a policy violation; the verb takes refs.
- **A1** — the journal's edge row carries a `planned_new_oid` and a target ref
  readable by the same path `--show-journal-trunk-oid` uses for the trunk row.
  **To verify in `/research`** — if the edge row's shape differs, objective 2's
  "generalise over the target ref" framing changes.
- **A2** — no accepted spec/ADR pins the *promotion* direction (only the
  integrate direction). Taken from IMP-308's preflight; re-check in `/research`.

## Open Questions

- **OQ-1** — Firing window for promotion: pre-`setup` only, or allowed whenever
  no live coordination branch pins the current trunk? (objective 3)
- **OQ-2** — Does the close gate **refuse** or **warn**? SL-126 refuses on the
  trunk row because unintegrated code is a correctness failure. An unsynced
  `edge` is hygiene — the code *is* landed on trunk (IMP-267's own note
  downgrades it: "not a missing-code emergency"). Refusing may be
  disproportionate; warning may be ignorable. Decide at design-lock.
- ~~**OQ-3** — Is the not-FF-gated `--edge` leg already tracked as backlog?~~
  **Answered at scope time:** it was not (nearest prior art is RFC-006, resolved,
  and it concerns the *trunk* leg). Filed as **RSK-230**.
- **OQ-4** — Does the promotion verb subsume or compose with `refresh-base`?
  `refresh-base` advances *the dispatch branch* to trunk; promotion advances
  *trunk* to the aggregate. Adjacent, opposite, possibly one verb family.

## Verification / Closure Intent

- **VT** — the close gate's three states (integrated / not-integrated /
  no-edge-row⇒silent) each covered by a test at the `run_status` seam, mirroring
  SL-126's gate tests.
- **VT** — the promotion verb: FF-only success; refusal when the source does not
  dominate the target; CAS refusal on a moved target; firing-window refusal per
  OQ-1's resolution.
- **VT** — behaviour-preservation: SL-126's and SL-121's existing suites stay
  green **unchanged** (shared machinery gate, AGENTS.md).
- **VA** — a dry-run walk of the SL-224 close scenario shows the gate would have
  fired where the hand recovery was needed.
- **VH** — `AGENTS.md`'s pre-dispatch ritual and the verb agree.

## Summary

## Follow-Ups
