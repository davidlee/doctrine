# Inquiry-map growth without re-attestation

## Context

The design run's inquiry map is meant to be **live**: `DEC-061` permits nodes to be
added as conditional discoveries become concrete, `DEC-063` makes redeclaration the
edit verb, and the engine applies no stage gate at all on `declare` — it is
admissible even at `locked`. `RFC-030` describes the map as a *mutable graph*.

Two facts about it are wrong today, and they compound.

**1. Discovery is punished.** Two conditions bound to the map are `attested`,
`binding = inquiry-map`, `reach = cumulative` (`install/design-run-stages.md`):
`initial-concerns-recorded` and `user-accepts-sufficiency`. Cumulative reach means
every edge above their own re-derives them against **current content**, so any
*shape* change — adding a node, adding a `needs` or `parent` edge, re-wording a
question — voids both and re-faces the human gates at `inquiring→drafting`,
`drafting→reviewing` and `reviewing→locked`.

Progress is already exempt: `NodeMaterial` excludes lifecycle and disposition,
pinned by `node_material_ignores_progress_and_observes_shape`
(`src/design_run/tests.rs`). Shape is not. So the cheapest strategy is to
front-load the whole map during `exploring`/`inquiring`, get sufficiency accepted
once, and then never touch its shape again — which is what runs do. `DEC-062` says
the opposite of the policy in force: "Ordinary inquiry-map maintenance—adding,
moving, pinning, deferring, or pruning nodes—does not require human approval."

**2. A published sparse key lies.** `doctrine design contract --format prompt`
publishes `needs … sparse (omit persists · null clears)`. `needs: null` changes
nothing **and reports success**: `declare_node` matches only `Sparse::Value`
(`src/design_run/run.rs:1449`). `question` goes through `Sparse::apply`, which
handles `Null`, and `parent` has explicit `Value`/`Null` arms — `needs` is the one
field whose `null` arm was never wired. This is `SL-259`'s disease (*success for
input it ignored*) arriving in the one place `SL-259` did not look.

Both are parked in `RFC-031` T4 as `IMP-469` and `ISS-481`.

## Scope & Objectives

Two objectives, one substantive and one mechanical:

1. **A shape change must not void an attested judgement about the map.** Derived
   coverage must still react to it: `blocking-inquiries-dispositioned` is
   `derived` and cumulative, and must keep blocking advance when a genuinely new
   blocking question appears. The two *attested* conditions should not.
2. **`needs: null` clears**, emitting one `NeedsRemoved` per removed edge —
   exactly what `needs: []` already does, so the rows machinery needs nothing new.

The first is the work of the slice, and its *shape* is an open design decision
rather than a foregone conclusion (see `OQ-1`).

## Non-Goals

- **The map's semantic tier and edge kinds** (`QUE-218`). This slice changes what
  a *change* to the map costs, not what the map can express.
- **Backward cascade** (`IMP-386`) — a changed answer reopening or flagging its
  dependents. Deliberately not bundled: `IMP-386` wants *more* propagation
  (backwards), this wants *less* (forwards), and one design must not do both by
  accident. `IMP-386` is gated on `QUE-218` in any case.
- **Surfacing the map** (`ISS-299`), the derived traversal candidate (`IMP-389`),
  and the runbook growth obligation (`IMP-470`) — T4 neighbours, separate units.
- **`ISS-450`** (a `needs` edge declared at node *creation* emitted no row). It is
  the create branch; this is the update branch's `null` arm. Two distinct tests,
  and keeping them distinct is the point.
- **Migrating or re-attesting existing runs.** Seventeen live snapshots keep
  whatever their receipts already say.

## Affected surface

- `src/design_run/gate.rs` — requirement coverage: the `Coverage::InquiryMap`
  rows and `coverage_moved`, the comparison that decides invalidation
- `src/design_run/inquiry.rs` — `NodeMaterial`, `materials()`, the shape/progress
  split
- `src/design_run/run.rs` — `declare_node`'s `needs` block
- `src/design_run/submission.rs` — the key-home / state axes, if a new key is
  minted
- `install/design-run-stages.md` — the shipped edge/condition/reach table, if a
  binding or reach changes
- design-run unit and e2e suites

## Risks, assumptions, open questions

- **OQ-1 — which fix family.** (a) Narrow the binding of the two attested rows: a
  set-identity that ignores additions, or a reach conditional on the *accepted*
  subset. (b) Mint a derived "added since acceptance" row that blocks advance
  without voiding judgement. (b) reads closer to intent — it blocks without
  retracting — but adds a ninth condition to a table of eight. `/design` decides
  and records why.
- **OQ-2 — `initial-concerns-recorded` names a set.** It requires the current
  `blocking-set-declared` to be named, so growth re-faces it even under a narrowed
  binding. Any policy must distinguish *the named set is still a subset of the map*
  from *the map moved*.
- **OQ-3 — does the human act survive, or get re-asked with the delta?** The
  counter-argument to `OQ-1` is that "enough has been asked" is a judgement over a
  set that has since changed, so re-asking is honest. The position recorded in
  `RFC-031` (2026-09-25, at the operator's direction) is that it survives.
  `/design` either adopts that or records why not — it may not leave it implicit.
- **OQ-4 — blocking, or warning?** If the design keeps a "map moved ⇒ the
  blocking set is re-declared" obligation (`R4`), is that obligation blocking or
  a warning? `DEC-101` supplies the precedent for a stale discharge that *warns
  without blocking*; `DEC-120` supplies the case against the judgement surviving
  silently.
- **R1 — loosening invalidation is a truthfulness change.** `RFC-031` T1 was about
  the exit signal telling the truth; making a condition stop re-deriving is the
  same class of change in the other direction. A condition that *should* have
  invalidated and did not is a silent lie, so the derived half must be shown to
  still fire (see `VT-1`).
- **R2 — seventeen live snapshots change semantics underneath.** Bound by
  `mem.fact.design-run.snapshot-outlives-the-binary`. This must not make a stored
  snapshot unreadable; a reading-side change is preferred to a stored-shape change.
- **R3 — the `needs: null` fix is mechanical and low-risk.** No stored snapshot can
  hold a `Null` that mattered, because the no-op means none was ever honoured. The
  test is the work, not the fix.
- **A1 — the corpus supports the diagnosis.** Sixteen of seventeen runs report
  `cursor unset`, and the map's largest, most-connected instances are the runs that
  kept adding to it during inquiry. Growth is permitted and exercised; the
  disincentive is cost, not refusal.
- **R4 — the guard is coupled** *(found in the pre-design research round, and it
  narrows `OQ-1`)*. `blocking-inquiries-dispositioned` quantifies over the
  **declared** blocking set, and its own doc comment says a declaration whose map
  has moved *"goes stale there rather than being silently re-read here"*
  (`src/design_run/gate.rs`, above `blocking_inquiries_open`). So the derived
  row's correctness **depends on** the attested rows' map-moved staleness.
  Narrowing the binding without preserving a "map moved ⇒ the set is re-declared"
  path would let a newly-added blocking node go unblocked — the defect `VT-1`
  exists to catch. This is why `OQ-1(A)` alone is unsafe and `OQ-1(B)` is the
  likely answer. Evidence: `research/research.md` cross-thread finding 1.

## Verification / closure intent

- **VT-1**: a declaration adding a node (a shape change) after
  `user-accepts-sufficiency` is attested does **not** void it — while
  `blocking-inquiries-dispositioned` still reports unsatisfied for every
  **declared** blocking node, and a newly-added *blocking* node still cannot pass
  unblocked (`R4`). The derived half must be demonstrably *not* exempted.
- **VT-2**: a declaration supplying `needs: null` clears the set and emits one
  `NeedsRemoved` per removed edge; `null` on a node with no edges is a no-op *with
  no rows* — absence of change is not failure to report.
- **VT-3**: the pre-change behaviour is pinned first. The suite asserting that a
  shape change voids the cumulative conditions stays green until the change is
  deliberate, then flips against a named criterion rather than being deleted.
- **VA**: on a real run, a node added late does not re-face the human gates —
  re-measured against `RFC-031`'s new fitness measure (nodes/edges added after
  `user-accepts-sufficiency`).
- Closes `IMP-469` and `ISS-481`; records the outcome and the `DEC-062`
  reconciliation on `RFC-031` T4.

## Summary

## Follow-Ups
