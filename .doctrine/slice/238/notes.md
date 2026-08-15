# Notes SL-238: Cross-kind backlog ordering admission and override footer

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Design surface triage (exploring, 2026-08-15)

Discharges `explore.triage`. Evidence lives in `research/research.md`; this is the
shape of the design surface, not a restatement of it.

### Constraining governance

| authority | what it requires here |
|---|---|
| REQ-238 (SPEC-001 `FR-006`) | the per-kind status-class partition lives in policy — a per-kind terminality question routes through `priority::partition::status_class` |
| ADR-017 | `partition.rs` is the sole engine delta for gating; `QUE open` is `Gating` |
| ADR-009 | slice terminal is `{done, abandoned}` — what `--prune`'s `resolved`/`closed` probe contradicts |
| STD-001 | that inline probe is a duplicated terminal table |
| ADR-016 §2 | terminality is a projection, never a relation label |
| REQ-218 (SPEC-015 `FR-006`) | names `backlog_order` in the requirement — deleting it needs a REV; widening it does not |
| ADR-001 | `backlog_order → backlog` is an *accepted* violation, existing only because `ItemId` is `ItemKind`-backed |

Checked and not applicable: POL-001, POL-002, STD-002, ADR-004, ADR-010,
ADR-013/014, ADR-015 (except on the retire horn), ADR-019, ADR-020. Reasons in
`research.md` Thread 1.

### Shaping decisions

- **D-A. The fork — SETTLED, `DEC-231`.** Neither horn. `--by sequence` is a
  backlog-induced **work order**, not an actionability surface; cross-kind edges
  get no ordering effect on either axis and are **disclosed**, never used to
  order or withhold. `backlog_order.rs` is untouched. Supersedes `DEC-230`,
  which had ruled the opposite (see *Reversal* below).
- **D-B. Where the status probe lives — OPEN (`inq-5`).** Still live: the footer
  must say `QUE-219 (open)`, so the impure shell (`list_rows`) must resolve refs
  and classify via `partition::status_class`, then pass results into pure
  `project`/`render_overrides`. Targeted per-ref through
  `catalog::scan::status_and_title_for` (~16 reads, currently private) vs the
  full 24-kind `scan_entities` walk is the cost decision.
- **D-C. Footer leg direction — OPEN (`inq-6`).** The two legs state the same
  relation in opposite directions.
- **D-D. Reveal flag shape — OPEN (`inq-7`).** Name and stream.
- **D-E. Cross-kind clearing — OPEN (`inq-8`).**

### Inquiry map (design run `dr-01a00475`)

Resolved: `inq-1` → `DEC-230` (superseded), `inq-2` → `DEC-231`, `inq-3`
(non-durable, moot), `inq-4` (non-durable, ADR-001 paydown falls away with the
widen horn). Open: `inq-5`, `inq-6` (cursor), `inq-7`, `inq-8`.

### The reversal — read this before re-opening the fork

DEC-230 ruled `--by sequence` an actionability surface that must gate. **That was
wrong and DEC-231 supersedes it.** The reasoning that overturned it, so it is not
re-derived:

1. `--by sequence` does **not** withhold on a backlog-internal live `needs`
   today — `ISS-X needs ISS-Y`, both open, shows both with Y before X. Only
   *terminal* items leave the node set. So gating cross-kind dependents while
   ordering internal ones is asymmetric.
2. Origin intent settles it: SL-051 states *"ordering is a sort, not a view"*,
   keeps membership unchanged, and names blocked nodes as retained participants.
   The code says work order throughout (`src/backlog.rs:81`, `:1113`, `:1190`);
   `next` explicitly builds only the actionable set
   (`src/priority/surface.rs:384`, `:406`). Actionable `--by sequence` is
   retrofit, not recovered intent.
3. The pressure to make it gate came from `doctrine next` having **no kind
   filter** — so "the actionable backlog" was not expressible in one command.
   That is a gap in the actionability surface, misdiagnosed as a defect in the
   ordering one. Raised as **IMP-432**.
4. Phantom admission does not work either: cordage assigns longest-path
   **levels** before `NodeId` (`crates/cordage/src/resolve.rs:635`,
   `lib.rs:366`), so admitting a cross-kind node demotes its dependent behind
   *every* level-0 node rather than stating "IMP-390 comes after SL-251" — and
   there is no principled `exposure`/`created` for a node that has neither.

### Risks

- ~~**R1 — ordering divergence.**~~ Dissolved by `DEC-231`: no node admission, no
  comparator change, no adapter change. Row order is untouched.
- **R2 — golden churn.** Realised, not hypothetical: `74b773690` shipped
  suppression-by-default on one leg and locked it with two tests
  (`src/backlog.rs:5301`, `:5332`). They assert cross-kind drops are *silent*,
  and the settled design *discloses* them — so both must be superseded
  deliberately, not relaxed.
- **R3 — untested leg.** `run_after --prune` has no test coverage at all.
  Characterisation tests precede any probe change there.
- **R4 — soft-axis over-reach.** An `after` edge onto an unrecognised-status
  target must not withhold or order. The conservative blocker rule is
  implemented over `dep_overlay` only (`src/priority/channels.rs:58`, `:66`);
  treating status uncertainty as a gate would silently harden a preference into
  a blocker.

### Assumptions

- **A1** — `status_class` covers every kind reachable as a cross-kind dep target.
  Partially verified: the `PARTITION` table covers slice, ADR, policy, standard,
  PRD/SPEC, requirement, review, revision, backlog, and the knowledge kinds.
  Unrecognised is a defined fallback class, so the failure mode is conservative.
- **A2** — no non-backlog entity authors a `needs`/`after` edge whose *target* is
  a backlog item in a way this slice would newly order. **Still unverified.**
- **A3** — the corpus's cross-kind edges were authored through `doctrine needs`
  (validated by `kinds::ensure_ref_resolves`), not `backlog after`, which cannot
  express them. Consistent with `--remove`'s inability to clear them.

### Superseded by measurement

The scope's claim that harm is latent — *"all ten targets happen to be `done`"* —
is false as of 2026-08-15. Eleven cross-kind `needs` edges target `QUE` records,
two of which (`QUE-218`, `QUE-219`) are `open` and gate nine live items that
`backlog list --by sequence` currently renders as ungated while `next` and
`blockers` gate them correctly.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: <yyyy-mm-dd> · <PHASE-NN | stage> · <head-commit>

### Produced

### Learned

### Open
