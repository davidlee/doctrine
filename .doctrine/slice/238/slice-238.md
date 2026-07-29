# Cross-kind backlog ordering admission and override footer

## Context

`doctrine backlog list --by sequence` (the default) trails an `overrides:`
honest-record footer that today reports ten lines of the form:

```
  ISS-028 → SL-182 dropped (dangling: SL-182 absent)
```

Every one of those claims is **false**. All ten slices exist; all ten are `done`.

The lines come from the `AbsentDrop` leg of `render_overrides`
(`src/backlog.rs:2285`), not the adapter-override leg. `backlog::project`
(`src/backlog.rs:745`) resolves each `needs`/`after` reference with `parse_ref`
(`src/backlog.rs:1346`), which knows only the five backlog prefixes
(`ISS/IMP/CHR/RSK/IDE`). Any other prefix falls into `AbsentDrop`, and the footer
hardcodes the word `absent`.

Three consequences, escalating past cosmetics:

1. **The footer lies.** `absent` means "not a backlog item", rendered as "does not
   exist". The author cannot judge staleness from it.
2. **The edge is silently discarded.** A cross-kind reference contributes no
   ordering edge at all — so `IMP-321 needs SL-228` would fail to gate IMP-321 in
   `list --by sequence` even with SL-228 live. Cross-kind `needs` is *legal*
   authored data: `run_needs` validates via `kinds::ensure_ref_resolves`, and
   `run_needs_accepts_cross_kind_slice_prereq` (`src/backlog.rs:4667`) locks that
   in. No visible harm in today's corpus only because all ten targets happen to be
   `done`.
3. **Two disagreeing classifiers.** `backlog after --prune`
   (`src/backlog.rs:2013-2031`) already resolves cross-kind refs via
   `kinds::parse_canonical_ref` and reads the target's status — but tests
   `resolved`/`closed`, which is *backlog* vocabulary. Slice terminal is
   `done`/`abandoned` (ADR-009). So `--prune` calls SL-182 live and declines to
   clear it, while `list` calls it absent. The edge is unclearable by either verb:
   `--remove` rejects the `SL` prefix outright.

This slice fulfils **IMP-099** (triaged), which named all three shortfalls at
SL-105 reconcile (RV-084) and deferred them as needing their own design. It also
subsumes **IDE-019** (open) — the `--verbose`/`--explain` gate for footer noise —
because the truth fix changes *what belongs in* the footer, so gating and content
cannot be settled independently on one output surface.

### What already exists (ride these seams, do not rebuild)

- **`src/priority/partition.rs::status_class(kind, status)`** — the kind-aware
  terminal/workable/gating classifier, already covering slice, ADR, spec,
  requirement, backlog, review, revision, and knowledge vocabularies. This is the
  classifier `classify_dangling` and `--prune`'s probe should consume. No new
  terminal-status table.
- **`src/priority/graph.rs`** — already admits cross-kind `needs`/`after` onto the
  dep/seq overlays (`slice_needs_lands_on_dep_overlay_cross_kind`,
  `slice_after_lands_on_seq_overlay_with_rank_and_array_index_age`). The
  actionability graph is correct; `backlog_order.rs` is the holdout.
- **`src/priority/order.rs`** — the extracted ordering primitives
  (`surviving_seq_predecessors`, `frontier_order`) shared by `next` and the
  interestingness detectors.

So the repo carries **two cordage consumers**: `src/backlog_order.rs` (backlog-only
vocabulary, feeds `list --by sequence`, SL-039/SL-051) and `src/priority/graph.rs`
(cross-kind, feeds `next`/`blockers`/`survey`/`explain`, SL-060/IMP-033). The
backlog-only one predates cross-kind admission. Whether this slice *widens* that
adapter or *retires* it in favour of the graph is the central open question below —
and per POL-002/DRY it is the question worth resolving properly, not routing
around.

## Scope & Objectives

One coherent change: make `backlog list --by sequence` tell the truth about
cross-kind dep/seq edges, and make those edges clearable.

1. **Admission.** A `needs`/`after` reference to a non-backlog entity resolves
   against the real corpus and is classified by *its own* kind's status
   vocabulary via `priority::partition::status_class` — not shunted into
   `AbsentDrop`. A live cross-kind prerequisite orders; a terminal one is a
   satisfied prerequisite; a genuinely non-resolving ref is the only thing that
   earns the word `absent`.
2. **Honest footer.** The `overrides:` block reports cross-kind drops with the
   target's real status/resolution (the shape `classify_dangling` already produces
   for backlog targets: `closed/wont-do`), and collapses duplicate
   `(from, to, reason)` lines. `IMP-172` carries `SL-154` in *both* `needs` and
   `after` (`backlog-172.toml:19-20`) — two real edges, but one line's worth of
   information for the reader.
3. **Default quiet, opt-in loud (IDE-019).** Terminal-target drops are suppressed
   by default; an explicit flag reveals the full record. This extends the
   suppression already present for adapter-level `Dangling`s with a terminal
   from-endpoint (`src/backlog.rs:2294-2304`, which cites IDE-019) to the leg that
   never got it.
4. **Clearable.** `backlog after --prune`'s terminality probe consumes the same
   kind-aware classifier instead of its inline `resolved`/`closed` string test, and
   `--remove` accepts a cross-kind target ref. The `IMP-095 → SL-095` edge that
   IMP-099 names as the standing instance becomes clearable — deliberately
   retained, but by choice rather than by tooling inability.

## Non-Goals

- **No change to relation vocabulary.** No new `RelationLabel`, no new dep/seq
  axis, no widening of the dep/seq *source* gate (records still cannot author
  dep/seq — ADR-017, SL-158 D2).
- **No change to `src/priority/graph.rs`'s cross-kind semantics.** It is already
  correct; this slice makes the backlog ordering view agree with it, not the
  reverse.
- **No corpus edit as the fix.** The ten authored cross-kind refs are legal data.
  Rewriting them to silence the footer would hide the defect. Clearing individual
  spent edges *after* the tooling can do so honestly is a separate judgement call.
- **No new terminal-status table.** `partition.rs` is the single source; a second
  one would reproduce exactly the bug being fixed.
- **Not the slice/spec ordering *product* model.** IMP-099 notes non-backlog
  entities "do not reuse item→item `after` semantics verbatim". Where that bites
  beyond making `list` honest, it goes to a follow-up.
- **No `doctor` / integrity check for cross-kind dep refs.** Adjacent, separable.

## Affected surface

- `src/backlog.rs` — `project`, `AbsentDrop`, `render_overrides`,
  `classify_dangling`, `compose`, `run_after`'s `--prune`/`--remove` legs,
  `parse_ref`'s callers, `list` flag surface.
- `src/backlog_order.rs` — the adapter's `ItemId` vocabulary is backlog-only;
  cross-kind admission touches it, or retires it (open question OQ-1).
- `src/priority/partition.rs` — read-only consumer seam; expected unchanged.
- `src/kinds.rs` — `parse_canonical_ref` / `ensure_ref_resolves`, the existing
  cross-kind resolution primitives.
- Footer goldens / `backlog list` test fixtures in `src/backlog.rs`'s test module.

## Risks & assumptions

- **R1 — ordering-semantics divergence.** `backlog_order.rs` composes with a
  `created`/`exposure` comparator; `priority::order::frontier_order` is
  score-aware. If OQ-1 resolves toward retiring the adapter, `list --by sequence`
  row order changes for reasons unrelated to this slice's intent. Behaviour
  preservation on the existing suites is the gate (AGENTS.md).
- **R2 — golden churn.** The footer is asserted by name in several tests
  (`src/backlog.rs:5094`, `:5115`, `:5224`, `:5262`). Suppression-by-default flips
  the sense of "no drops, no footer" assertions; the risk is a test relaxed to fit
  the new behaviour rather than re-expressing intent.
- **A1** — `status_class` covers every kind reachable as a cross-kind dep target.
  Its per-kind vocabulary tests suggest yes; verify, don't assume.
- **A2** — no non-backlog entity currently authors a `needs`/`after` edge whose
  *target* is a backlog item in a way this slice would newly order. Unverified.

## Open questions

- **OQ-1 (the fork).** Widen `backlog_order.rs` to cross-kind ids, or retire it and
  compose `list --by sequence` from `priority/graph.rs` + `priority/order.rs`?
  Retiring kills a parallel implementation (the stronger DRY answer) but changes
  row order (R1) and pulls scoring into a view that has none today. Widening is
  contained but keeps two cordage consumers alive. `/design` decides.
- **OQ-2.** Flag naming and shape for the reveal: `--explain` vs `--verbose`
  (IDE-019 leaves it open), and whether the footer belongs on stdout with the table
  or on stderr as an advisory (the cycle warning already goes to stderr —
  `ListOutput`, `src/backlog.rs:1177`).
- **OQ-3.** Is a *terminal* cross-kind prerequisite silently satisfied (no edge, no
  line by default), or is a spent edge worth one suppressed-by-default line? Bears
  directly on whether `IMP-095 → SL-095` still reads as the reminder IMP-099
  intends it to be.
- **OQ-4.** Does `--remove` gain cross-kind target resolution, or does a cross-kind
  edge route through a kind-neutral `doctrine unlink`-shaped verb? IMP-099 offers
  both.

## Verification / closure intent

- **VT** — a cross-kind `needs` on a *live* slice orders the dependent item;
  the same on a `done` slice does not, and neither emits the word `absent`.
- **VT** — a genuinely unresolvable ref (`SL-9999`) still reports `absent`; the
  word retains its meaning.
- **VT** — the default footer is empty for today's corpus shape; the reveal flag
  prints one line per drop, deduplicated across the `needs`/`after` pair.
- **VT** — `after --prune` clears a cross-kind edge whose target is terminal under
  *its own* vocabulary (`done`), and declines a live one.
- **VA** — no second terminal-status vocabulary is introduced; `grep` shows the
  inline `resolved`/`closed` probe in `run_after` gone, and `status_class` the sole
  classifier on both legs.
- **VA** — existing `backlog`, `backlog_order`, and `priority` suites green
  unchanged where behaviour is preserved; every intentional golden change named in
  the reconciliation brief with its reason.

## Summary

## Follow-Ups
