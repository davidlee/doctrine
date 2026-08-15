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
- **D-B. Where the status probe lives — SETTLED, `DEC-233`.** Both horns the
  node posed are dead: the full `scan_entities` walk is a ~20x regression on a
  0.19s command, and `catalog::scan` is command-tier and *reaches* `backlog`, so
  calling into it closes an ADR-001 cycle. The probe instead composes two
  downward seams — `kinds` (leaf) for resolution, a stat with no file read; and
  `meta::read_meta` (engine) for status, ~15 distinct targets ≈ 8ms. No new
  module; the footer shell and the doctor check each call both directly. Shell
  returns a three-way (`Some` / `None` / `Unavailable`), pure layer classifies
  via `partition::status_class`.
- **D-C. Footer content, direction, dedup — SETTLED, `DEC-232`.** The footer's
  contract is *only what is needed to understand the rendered content*.
  Cross-kind edges with a live target go to a separate `boundary:` block
  (`ISS-327 needs QUE-219 (open)`, dependent-first, relation word, no arrow);
  terminal-target edges are reveal-only; **every ref-integrity failure leaves
  the footer for `doctrine doctor`**, which takes the project-level `AbsentDrop`
  leg with it — so the direction clash dissolves by deletion, not by flipping.
  The doctor check is **folded into this slice**.
- **D-D. Reveal flag shape — SETTLED, `DEC-234`.** No flag. `-a/--all` is
  already taken (row hide-set) and a footer flag would exist to defeat the rule
  `DEC-232` just gave the footer. `backlog inspect <ID>` already prints the full
  record on both axes; it gains only a target status annotation from `DEC-233`'s
  probe. Footer shape is invocation-independent.
- **D-E. Cross-kind clearing — SETTLED, `DEC-235`.** Neither horn. The
  kind-neutral verb already exists — top-level `doctrine after <SRC> <TGT>
  --remove` clears a cross-kind edge today; `backlog after` is a backlog-only
  duplicate that refuses what its twin accepts. So: route `backlog after`'s
  remove/prune to the kind-neutral shell, add `doctrine needs --remove` (new
  `dep_seq::remove_needs` leaf), and collapse **four** copies of the terminality
  probe onto `DEC-233`'s + `status_class`. `parse_ref` untouched.

### Inquiry map (design run `dr-01a00475`)

Resolved: `inq-1` → `DEC-230` (superseded), `inq-2` → `DEC-231`, `inq-3`
(non-durable, moot), `inq-4` (non-durable, ADR-001 paydown falls away with the
widen horn), `inq-6` → `DEC-232`, `inq-5` → `DEC-233`, `inq-7` → `DEC-234`,
`inq-8` → `DEC-235`. **Open: none — 8 of 8 resolved at revision 21.**

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
  deliberately, not relaxed. `DEC-232` supersedes `:5301` on a **second** count:
  its `not-a-ref` leg asserts a malformed ref is named *in the footer*, which
  now moves to `doctor`.
- **R5 — surface growth.** `DEC-232` folds a `doctor` ref-integrity check into
  this slice. Deliberate (the footer removal is only honest with somewhere for
  the errors to go), but it is real added surface: a check, its wiring, and its
  tests. Guard against it pulling further scope — the check is
  `ensure_ref_resolves` over each item's `needs`/`after` under the existing
  `RelationIntegrity` category, nothing wider.
- **R3 — untested leg.** `run_after --prune` has no test coverage at all, in
  **either** copy. Characterisation tests precede any probe change there. And
  the prune fix is a deliberate *behaviour* change (an edge onto a `done` slice
  becomes prunable), not a refactor — it must be tested as one, never smuggled
  through the duplication cleanup (`DEC-235`).
- **R4 — soft-axis over-reach.** An `after` edge onto an unrecognised-status
  target must not withhold or order. The conservative blocker rule is
  implemented over `dep_overlay` only (`src/priority/channels.rs:58`, `:66`);
  treating status uncertainty as a gate would silently harden a preference into
  a blocker.

### Assumptions

- ~~**A1**~~ — *resolved by `DEC-233`.* `status_class` covers the kinds, but the
  gap is upstream of it: `RV`'s status is **derived** at command tier
  (`review::derived_status_string`), so no engine-side reader can obtain it.
  Handled as an explicit `Unavailable` arm keyed off a pinned kind set, never a
  silent Terminal. `REC` is not a gap — `status_class(kind, None)` defines it as
  Terminal. Follow-up: `IMP-433`.
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
fresh-as-of: 2026-08-15 · design/exploring (run `dr-01a00475`, rev 21) · 20ecb2296

### Produced

- `DEC-235` — clearing rides the existing kind-neutral verb; `needs --remove`
  added; four probe copies collapse.
- `DEC-234` — no reveal flag; the full record is `backlog inspect`.
- `DEC-233` — the status probe's seams, its cost, and the loudness rules on its
  one degradation.
- `IMP-433` — lift `RV`'s derived status to a tier engine-side readers reach;
  the follow-up that lets `DEC-233`'s `Unavailable` arm retire.
- `DEC-232` — the footer's contract, and the doctor counterpart it requires.
- `DEC-231` — the settled fork. Supersedes `DEC-230`.
- `DEC-230` — superseded; retained because the reversal is instructive.
- `IMP-432` — `next` lacks kind/tag/status filters; the complement that makes
  the split whole.
- `research/research.md` (runtime tier, gitignored) — two threads plus corpus
  measurement and the ✓/✗ verification pass.

### Learned

- Records cannot author dep/seq, so a `QUE` node has zero *incoming* dep/seq
  edges and can never sit mid-chain (`src/commands/dep_seq.rs:50-56`).
- Cordage assigns longest-path **levels** before `NodeId`
  (`crates/cordage/src/resolve.rs:635`), so a phantom cross-kind node imposes a
  broad level demotion, not a local `after` statement. No neutral fallback
  attribute exists.
- `REQ-218` names `backlog_order` in the requirement *title* and carries no
  statement body and no acceptance criteria — so retiring the adapter needs a
  SPEC-015 REV, but widening it would falsify nothing.
- SPEC-015's "a grouping, never a priority claim" attaches to the `ordinal`
  grouping (`--by id`), **not** to `--by sequence`, which it separately calls
  "priority order". The governance research conflated them.
- `run_after --prune` has no test coverage at all, and its disk-read-and-parse
  block is duplicated within one function (`src/backlog.rs:2014-2060`) — **and
  the whole function is duplicated again** in `src/commands/dep_seq.rs:196+`.
  Four copies of one terminality probe, all hardcoding `resolved`/`closed`.
- **The kind-neutral clearing verb already exists.** Proved: `doctrine after
  IMP-172 SL-154 --remove` succeeds; `doctrine backlog after IMP-172 SL-154
  --remove` fails with `unknown backlog prefix SL`. The top-level dep/seq shell
  gates through `kinds::parse_resolvable_ref`; `backlog after` is the duplicate.
- **The `needs` axis is append-only for every kind.** `unlink` is
  tier-1-`[[relation]]`-only and refuses `needs`; `dep_seq::remove` is
  `remove_after` only. No verb anywhere clears a `needs` edge.
- **`--prune` misses `done`.** Proved: `SL-154` is `done`, the edge is present,
  `after IMP-172 --prune` reports `nothing to prune` — the ADR-009 vocabulary
  bug, live.
- `backlog inspect <ID>` already prints both dep/seq axes undeduped; it lacks
  only the target's status.
- `doctrine explain <ID>` already prints `blocked by:` for a **live** cross-kind
  prerequisite (`channels::blocked_by` filters to non-terminal,
  `src/priority/surface.rs:319`), so only the *satisfied* case is unshown.
- **Nothing re-checks `[relationships] needs`/`after` after authoring time.**
  `doctrine validate` is id-integrity only; `relation_graph::validate_relations`
  consumes `Catalog.edges`, built from `[[relation]]` rows. Probed: a fixture
  carrying `needs = ["not-a-ref", "ISS-999", "QUE-219", "SL-9999"]` gets
  `doctor: corpus clean` on all four, while `list --by sequence` renders two of
  them with opposite arrows in one block.
- **Measured cost, this corpus (~4,400 numbered-entity tomls):**
  `backlog list --by sequence` 0.19s · `doctrine validate` (id scan alone) 3.6s ·
  `doctrine doctor` 10.8s. Any full-corpus scan on the listing path is a ~20x
  regression. The last two are slow enough to deserve their own item — noted,
  not raised, out of this slice's scope.
- `catalog::scan` is **command**-tier and *reaches* `backlog`
  (`.doctrine/adr/001/layering.toml:128`), so `backlog → catalog::scan` closes a
  cycle. Any "just make it `pub(crate)`" reach into the cross-kind scanner from
  a kind module is refused by the ratchet, not merely untidy.
- `kinds::parse_resolvable_ref` on a *canonical* ref is a **directory stat, no
  file read** (`src/kinds/resolve.rs:68-78`) — so ref-resolution and
  status-reading have genuinely different costs and can be sited separately.
- `Override::from()` is documented as **uniformly the predecessor** across all
  three adapter reasons (`src/backlog_order.rs:111-116`); the evicted arms take
  it from `evicted.edge().src()` (`:312`) and the `Dangling` arms push
  `from: *dep` (`:228`, `:248`). Only `AbsentDrop` (`:702`) is dependent-first.
- Withholding needs no new machinery — `pos.get(…).unwrap_or(usize::MAX)`
  (`src/backlog.rs:1241`) already tails unplaced rows. Not used, but the seam is
  known.

### Open

- **Inquiry map is fully dispositioned** (8/8 at revision 21). What remains is
  the stage gate, not a question.
- **`IDE-019` divergences (2), for reconcile.** It asked for the absent-ref case
  to be *surfaced in the footer* (`DEC-232` routes it to `doctor`) and for a
  `--verbose`/`--explain` flag on `backlog list` (`DEC-234` declines the flag and
  sites the record on `inspect`). Both deliver its intent; neither its mechanism.
  `IDE-019` must close against what was built.
- **Scope grew three times**, each owner-accepted: the `doctor` check
  (`DEC-232`), the probe's loudness rules (`DEC-233`), and `needs --remove` plus
  the duplicate-clearing-path collapse (`DEC-235`).
- `inq-8` — cross-kind clearing (`--prune` / `--remove`).
- **A2 unverified** — whether any non-backlog entity authors a `needs`/`after`
  edge whose *target* is a backlog item.
- Slice scope (`slice-238.md`) is now **stale against `DEC-231`**: its objective
  1 (admission) is withdrawn, and its `OQ-1` still presents the widen/retire
  fork. Reconcile at close, or sooner if it misleads.
