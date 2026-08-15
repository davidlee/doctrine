# Cross-kind dep/seq edges: disclose, check, and clear

> **Rewritten 2026-08-15** at the `inquire.scope` gate, against the five
> decisions this design accepted (`DEC-231`…`DEC-235`). The original scope
> centred on *admission* — making a cross-kind edge order the backlog work
> order. `DEC-231` withdrew that objective and moved the slice's centre of
> gravity to *disclosure*. What follows is the current scope; the superseded
> framing survives in `DEC-230` (superseded) and in this file's git history.

## Context

`doctrine backlog list --by sequence` (the default) trails an `overrides:`
honest-record footer. Until `74b773690` it reported lines of the form:

```
  ISS-028 → SL-182 dropped (dangling: SL-182 absent)
```

Every one of those claims was **false**. All the slices named exist.

The lines came from the `AbsentDrop` leg of `render_overrides`
(`src/backlog.rs:2299`), not the adapter-override leg. `backlog::project`
(`src/backlog.rs:745`) resolves each `needs`/`after` reference with `parse_ref`
(`src/backlog.rs:1346`), which knows only the five backlog prefixes
(`ISS/IMP/CHR/RSK/IDE`). Any other prefix fell into `AbsentDrop`, and the footer
hardcoded the word `absent`. `74b773690` silenced those lines as an interim —
the footer is now empty repo-wide — which removed the lie without supplying the
truth.

Measured on the corpus, 2026-08-15: **30 authored cross-kind `needs`/`after`
edges**. Five hang off already-terminal dependents, which `project` never admits
as nodes, so **25 reach the footer** — 10 with a non-terminal target, 15 with a
terminal one. Twenty-one of the thirty are on the `needs` axis.

Four defects, of which the first is the one that survives `DEC-231`:

1. **The edge is invisible.** `ISS-327 needs QUE-219` and `QUE-219` is `open`,
   but nothing on the `--by sequence` screen says a prerequisite exists — where
   `ISS-X needs ISS-Y` at least renders Y above X. `next` and `blockers` gate on
   it correctly through `priority/graph.rs`; only this view says nothing.
2. **Nothing re-checks the refs after authoring.** `doctrine validate` is
   id-integrity only, and `relation_graph::validate_relations` consumes
   `[[relation]]` rows, not `[relationships] needs`/`after`. Probed: a fixture
   carrying `needs = ["not-a-ref", "ISS-999", "QUE-219", "SL-9999"]` yields
   `doctor: corpus clean` on all four.
3. **The footer states one relation in two directions.** `AbsentDrop.from` is the
   *authoring* item (`src/backlog.rs:702`) while `Override::from` is documented
   as uniformly the *predecessor* (`src/backlog_order.rs:111-116`), so the same
   fact renders `ISS-001 → not-a-ref` beside `ISS-999 → ISS-001`.
4. **The terminality probe is wrong, in four copies.** `src/backlog.rs:2013-2064`
   and `src/commands/dep_seq.rs:196+` are near-verbatim duplicates of each other,
   and each doubles its own read-parse-status block internally. All four hardcode
   `resolved`/`closed` — *backlog* vocabulary. Slice terminal is `done` (ADR-009).
   Probed: `SL-154` is `done`, the edge is present, and `after IMP-172 --prune`
   reports `nothing to prune`.

And one gap that is not cross-kind-specific at all: **the `needs` axis has no
removal verb.** `doctrine unlink` operates on tier-1 `[[relation]]` rows;
`dep_seq::remove` is `remove_after` only. A `needs` edge is append-only for every
kind.

This slice fulfils **IMP-099** (triaged), which named the footer, admission, and
clearing shortfalls at SL-105 reconcile (RV-084). It also subsumes **IDE-019**
(open) — the `--verbose`/`--explain` gate for footer noise — because the truth
fix changes *what belongs in* the footer, so gating and content cannot be settled
independently on one output surface. **Two divergences from IDE-019's proposal
must be recorded at reconcile** (see Follow-Ups).

### What already exists (ride these seams, do not rebuild)

- **`src/priority/partition.rs::status_class(kind, status)`** — the kind-aware
  terminal/workable/gating classifier. The sole classifier for every terminality
  question this slice touches. No new terminal-status table.
- **`src/kinds/resolve.rs`** — `parse_canonical_ref` / `ensure_ref_resolves`.
  Leaf, `out=0`; for a canonical ref, resolution is a **directory stat with no
  file read** (`:68-78`).
- **`src/meta.rs::read_meta`** — engine tier, one parse per entity: the status
  half of the probe.
- **`src/commands/dep_seq.rs`** — the **kind-neutral** dep/seq shell, gated by
  `resolve_dep_seq_src` over `parse_resolvable_ref`. `doctrine after <SRC> <TGT>
  --remove` already clears a cross-kind edge today; `backlog after` is its
  backlog-only duplicate.
- **`backlog inspect <ID>`** — already prints both dep/seq axes in full and
  undeduped; it lacks only the target's status.

## Scope & Objectives

One coherent change: make the backlog work order **disclose** the cross-kind
edges it cannot order, give ref-integrity failures a home that can report them,
and make the edges clearable on both axes.

1. **Disclosure, not admission** (`DEC-231`, `DEC-232`). A cross-kind
   `needs`/`after` edge contributes **no ordering effect on either axis** and is
   never used to withhold a dependent. `backlog_order.rs`, `ItemId`, and the
   cordage adapter are untouched. Where the target is **non-terminal**, the edge
   is disclosed in a new `boundary:` block naming the dependent, the relation
   word, the target and its status — `ISS-327 needs QUE-219 (open)` —
   dependent-first, no arrow, deduplicated per `(dependent, target)` with the
   axes joined. Where the target is terminal, nothing is printed.
2. **The footer's contract** (`DEC-232`). The footer states **only what is needed
   to understand the rendered content**. `overrides:` keeps evicted and
   contradicted `after` edges — authored edges that could have ordered and did
   not. `boundary:` carries edges that were never in this order's universe. The
   project-level `AbsentDrop` leg is **removed from the footer**, which dissolves
   defect 3 by deletion rather than by choosing a direction.
3. **Ref integrity moves to `doctor`** (`DEC-232`). A malformed ref, a dangling
   edge onto an absent backlog id, and a well-formed cross-kind ref that does not
   resolve all become `doctor` findings under the existing `RelationIntegrity`
   (Error) category, routed through `kinds::ensure_ref_resolves` — the same
   function `run_needs` uses at authoring time, so authoring and health agree by
   construction.
4. **The probe** (`DEC-233`). Resolution is leaf (`kinds`, a stat); status is
   engine (`meta::read_meta`, one parse per *distinct* target — ~15 in the live
   corpus, ~8ms against a 190ms baseline). `catalog::scan` is **not** reached:
   it is command-tier and reaches `backlog`, so the back-edge closes an ADR-001
   cycle. The shell returns `Some(status)` / `None` / `Unavailable`; the pure
   layer classifies through `status_class`.
5. **No reveal flag** (`DEC-234`). The footer's shape is invocation-independent.
   `-a/--all` keeps its single row-hide-set meaning. The full authored record is
   reached through `backlog inspect <ID>`, which gains a status annotation on each
   cross-kind target from the same probe.
6. **Clearable, on both axes** (`DEC-235`). `backlog after`'s `--remove` and
   `--prune` stop being a second implementation and route to the kind-neutral
   shell; `doctrine needs <SRC> <TGT> --remove` is added, backed by a new
   `dep_seq::remove_needs` leaf beside `remove_after`; and all four copies of the
   terminality probe collapse onto objective 4's, so `done` and `answered` are
   recognised as terminal. `backlog::parse_ref` is **not** widened — its five
   other hard-failing callers keep their contract.

## Non-Goals

- **No admission.** Withdrawn by `DEC-231`. No phantom node, no `ItemId`
  widening, no withheld-partition machinery, no change to membership or to
  backlog-internal dependency order.
- **No change to relation vocabulary.** No new `RelationLabel`, no new dep/seq
  axis, no widening of the dep/seq *source* gate (records still cannot author
  dep/seq — ADR-017, SL-158 D2). `doctrine unlink` stays tier-1-relation-only.
- **No change to `src/priority/graph.rs`'s cross-kind semantics**, and no new
  `channels` accessor. It is already correct.
- **No `needs --prune`.** A satisfied *hard* prerequisite is meaningful history;
  auto-dropping it is a judgement the tool should not make unasked (`DEC-235`).
- **No corpus edit as the fix.** The authored cross-kind refs are legal data.
  Clearing individual spent edges *after* the tooling can do so honestly is a
  separate judgement call.
- **No new terminal-status table.** `partition.rs` is the single source.
- **No fix for `RV`'s unreachable derived status.** Handled as a loud, pinned
  degradation; the real fix is `IMP-433`.
- **Not the slice/spec ordering *product* model.** Where IMP-099's "non-backlog
  entities do not reuse item→item `after` semantics verbatim" bites beyond this,
  it goes to a follow-up.

## Affected surface

- `src/backlog.rs` — `render_overrides` (the `AbsentDrop` leg removed, the
  `boundary:` block added), `classify_dangling` (becomes a classifier, stops
  returning a display string), `list_rows` (the probe call site), `run_after`'s
  `--prune`/`--remove` legs (routed to the kind-neutral shell), `inspect`'s
  relationship rendering.
- `src/commands/dep_seq.rs` — `run_after_prune`'s probe replaced; `needs
  --remove` shell added.
- `src/dep_seq.rs` — new `remove_needs` leaf beside `remove_after`.
- `src/relation_graph.rs` / `src/commands/doctor.rs` — the ref-integrity check
  under `RelationIntegrity`.
- `src/cli.rs` — `--remove` on the `needs` verb.
- `src/kinds/resolve.rs`, `src/meta.rs`, `src/priority/partition.rs` — read-only
  consumer seams; expected unchanged.
- Footer goldens and `backlog list` fixtures in `src/backlog.rs`'s test module.
- **Not** `src/backlog_order.rs`, and **not** `src/priority/`.

## Risks & assumptions

- **R1 — ordering divergence. Dissolved** by `DEC-231`: no node admission, no
  comparator change, no adapter change. Row order is untouched, so the
  behaviour-preservation gate on the `backlog_order` and `priority` suites should
  hold with no intentional golden change.
- **R2 — golden churn. Realised, not hypothetical.** `74b773690` shipped
  suppression-by-default and locked it with two tests (`src/backlog.rs:5301`,
  `:5332`) asserting cross-kind drops are *silent*. Both must be **superseded
  deliberately, not relaxed** — and `:5301` on a second count, since it asserts a
  malformed ref is named *in the footer*, which `DEC-232` moves to `doctor`.
- **R3 — untested leg.** `--prune` has **no test coverage at all, in either
  copy**. Characterisation tests precede the probe change. And the prune fix is a
  deliberate *behaviour* change (an edge onto a `done` slice becomes prunable),
  which must be tested as one rather than smuggled through the de-duplication.
- **R4 — soft-axis over-reach.** An `after` edge onto an unrecognised-status
  target must not withhold or order. The conservative blocker rule is implemented
  over `dep_overlay` only (`src/priority/channels.rs:58`, `:66`); treating status
  uncertainty as a gate would silently harden a preference into a blocker.
- **R5 — surface growth.** Scope grew three times, each owner-accepted: the
  `doctor` check, the probe's loudness rules, and `needs --remove` plus the
  duplicate-path collapse. Each is a counterpart the previous decision required,
  but the phase plan is larger than "make the footer honest" implies.
- **A1 — resolved** (`DEC-233`). The gap is upstream of `status_class`: `RV`'s
  status is derived at command tier, so no engine-side reader can obtain it.
  Handled as an explicit `Unavailable` arm off a pinned kind set. `REC` is not a
  gap — `status_class(kind, None)` defines it as Terminal.
- **A2 — still unverified.** Whether any non-backlog entity authors a
  `needs`/`after` edge whose *target* is a backlog item. Nothing in this scope
  depends on the answer, since no ordering effect is added either way.

## Open questions

All four are closed. Retained with their resolutions so an inbound reference
finds the answer rather than a dead end.

- **OQ-1 (the fork) — closed by `DEC-231`.** Neither widen nor retire
  `backlog_order.rs`: `--by sequence` is a backlog-induced **work order**, not an
  actionability surface, so a cross-kind edge is diagnosed and never ordered. The
  adapter is untouched, and the parallel-implementation debt against
  `priority/graph.rs` stays on the books rather than being cleared here.
- **OQ-2 (flag naming and stream) — closed by `DEC-234`.** No flag. `-a/--all` is
  already taken, and a footer flag would exist only to defeat the contract
  `DEC-232` gives the footer. The record lives on `backlog inspect`.
- **OQ-3 (is a spent edge worth a suppressed line?) — closed by `DEC-232`.** No,
  not on this surface: a satisfied prerequisite explains nothing about where a row
  landed. The default-quiet rule is now a consequence of the footer's contract
  rather than a noise heuristic. `IMP-095 → SL-095` remains readable as IMP-099's
  reminder via `inspect`.
- **OQ-4 (`--remove` vs a kind-neutral verb) — closed by `DEC-235`.** Neither.
  The kind-neutral verb already exists; `backlog after` is its duplicate and is
  routed to it. The genuinely missing capability was `needs --remove`.

## Verification / closure intent

- **VT** — a cross-kind `needs` on a **non-terminal** target emits one
  `boundary:` line naming the target and its status; on a terminal target it
  emits nothing; neither emits the word `absent`.
- **VT** — a `(dependent, target)` pair carried on **both** axes emits one
  `boundary:` line, not two.
- **VT** — a malformed ref, an absent backlog id, and an unresolvable cross-kind
  ref each raise a `doctor` `RelationIntegrity` finding and appear **nowhere** in
  `backlog list` output.
- **VT** — an `RV` target renders a named `Unavailable` token, is never
  suppressed, and never classifies Terminal; the derived-status kind set is
  pinned so a new derived-status kind fails a test.
- **VT** — `after --prune` clears an edge whose target is terminal under *its
  own* vocabulary (`done`, `answered`) and declines a live one; `backlog after
  --remove` accepts a cross-kind target; `needs --remove` clears a `needs` edge.
- **VT** — characterisation tests for `--prune`'s current behaviour land **before**
  the probe is replaced (R3).
- **VA** — no second terminal-status vocabulary survives: `grep` shows every
  inline `resolved`/`closed` probe gone from `src/backlog.rs` and
  `src/commands/dep_seq.rs`, with `status_class` the sole classifier.
- **VA** — `backlog_order` and `priority` suites green **unchanged**; every
  intentional golden change named in the reconciliation brief with its reason,
  including the two `74b773690` tests superseded rather than relaxed.

## Summary

## Follow-Ups

- **`IMP-432`** — `doctrine next` lacks kind/tag/status filters. The complement
  that makes `DEC-231`'s three-surface split whole; the pressure to make `--by
  sequence` gate came from "the actionable backlog" not being expressible.
- **`IMP-433`** — lift `RV`'s derived status to a tier engine-side readers can
  reach, retiring `DEC-233`'s `Unavailable` arm.
- **`IDE-019` divergences, for reconcile.** It asked for the absent-ref case to be
  *surfaced in the footer* (`DEC-232` routes it to `doctor`) and for a
  `--verbose`/`--explain` flag on `backlog list` (`DEC-234` declines the flag and
  sites the record on `inspect`). Both deliver its intent; neither its mechanism.
  `IDE-019` must close against what was built.
- **Parallel-implementation debt.** `backlog_order.rs` survives as a second
  cordage consumer beside `priority/graph.rs`. `DEC-231` declined to clear it
  here (removal needs a SPEC-015 REV — `REQ-218` names `backlog_order` in the
  requirement itself). Recorded, not quietly dropped.
- **Measured aside, out of scope.** `doctor` at 10.8s and `validate` at 3.6s on a
  ~4,400-entity corpus are slow enough to deserve their own item.
