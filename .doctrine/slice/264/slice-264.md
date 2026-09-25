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

Three objectives, two substantive and one mechanical:

1. **A shape change must not void an attested judgement about the map.**
   Additions must stop re-facing the human, and the guard that removal takes away
   must be replaced: `blocking-inquiries-dispositioned` is `derived` and
   cumulative, and must keep blocking advance when a genuinely new blocking
   question appears.
2. **The blocking set is a property of the node** (`DEC-302`). `blocking` becomes
   a node attribute, required when a node is created, and the set derives from it
   — one representation instead of the free-standing `blocking-set-declared` act.
   The user's `graph-reviewed` coverage compares blocking membership over the
   full set, so a node declared blocking moves the user's act and reaches them,
   and `R4` closes against the human rather than only the agent.
3. **`needs: null` clears**, emitting one `NeedsRemoved` per removed edge —
   exactly what `needs: []` already does, so the rows machinery needs nothing new.

Objectives 1 and 2 are one change: the narrowing is only safe alongside a
derived set, and the derived set is what makes the narrowing's guarantee whole.

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
- **Migrating or re-attesting existing runs.** Stored snapshots are not rewritten;
  they are read per node through the legacy fallback (design `sec-3`), and keep
  whatever their receipts already say except where the slice intends otherwise.

## Affected surface

Authoritative list: design `sec-5`. In summary:

- `src/design_run/gate.rs` — the `InquiryMap` / new `ReviewedGraph` coverages,
  `initial-concerns-recorded` as a single act, `blocking_inquiries_open` over
  effective node judgements
- `src/design_run/attestation.rs` — `CoveredSet::moved` parameterised by
  `Coverage`; the legacy act variants kept read-only
- `src/design_run/inquiry.rs` — `blocking` on the node and in `NodeMaterial`
- `src/design_run/run.rs` — `declare_node`'s `needs` and `blocking` arms, import
  seeding, `live_acts`
- `src/design_run/submission.rs` — the two-home `blocking` key; the retired
  `blocking-set-declared` key
- `src/design_run/admission.rs`, `change_log.rs`, `payload_contract.rs`
- `install/design-payload-contract.md`, `install/design-run-stages.md`,
  `install/design-prompts/conditions/initial-concerns-recorded.md`
- design-run unit and e2e suites

## Risks, assumptions, open questions

- **OQ-1 — which fix family.** (a) Narrow the binding of the two attested rows: a
  set-identity that ignores additions, or a reach conditional on the *accepted*
  subset. (b) Mint a derived "added since acceptance" row that blocks advance
  without voiding judgement. (b) reads closer to intent — it blocks without
  retracting — but adds a tenth condition to a table of nine *(corrected under
  `RV-386` `F-18`)*. `/design` decides
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

**Settled in the design run, 2026-09-25** — by `DEC-300` and `DEC-301`; **revised
under `RV-386`** (`DEC-302`, below):

- `OQ-1` → **both halves, not either.** The two attested rows' map comparison
  narrows to ignore *additions*, **and** a new derived cumulative condition
  requires the map's additions to be re-declared before the next edge
  (`DEC-300`). Narrowing alone was rejected: the derived blocking check quantifies
  over the *declared* set and relies on the map-moved staleness, so a newly-added
  blocking question could pass.
- `OQ-2` → the two mechanisms stay distinct. `CoverageStale` is what narrows.
  *(Revised under `RV-386`: with the set derived from the nodes there is no
  declaration left to confirm, so `ConfirmationStale` survives only as a frozen
  read of links stored before the change.)*
- `OQ-3` → the acceptance **survives** a change of shape. Recorded in `DEC-300`,
  not left implicit.
- `OQ-4` → **blocking, not a warning.** A warning nobody is made to see is worth
  nothing (`ISS-299`).
- `R4` → closed by the derived condition *(mechanism superseded — see below)*; the
  guard is preserved deliberately rather than as a side effect.
- **Governance delta:** `DEC-062`'s *"moving"* is read narrowly — an addition is
  not a change to what was seen, but a move or a re-word is (`DEC-301`), **scoped
to the nodes the accepting act covered** (`RV-386` `F-1`). A `Revision` cannot
  reach a `DEC`; `REQ-427` was verified not to reach gate attestations, so no REV
  is owed.

**Revised in adversarial review (`RV-386`, 2026-09-25)** — the shape changed, the
intent did not:

- `RV-386` `F-2` showed that the derived re-declaration condition does not close
  `R4`: an unchanged re-declaration over a larger map is free, so a question the
  agent judges blocking can still be omitted. `DEC-300`'s condition is
  **superseded** (`DEC-302`); the narrowing it also chose stands.
- The blocking set stops being a free-standing agent act and becomes a **derived
  projection of a per-node `blocking` attribute**, required when a node is
  created. The user's `graph-reviewed` coverage compares blocking membership over
  the full set, so a node declared blocking moves the user's act and reaches them;
  a node declared non-blocking is free. `R4` closes by construction, against the
  human.
- `blocking-set-declared` can no longer be written. *(Second pass: its enum
  variants and `Cause::ConfirmationStale` stay as a legacy read-only class so
  stored snapshots parse and keep their verdicts — see below.)* A node with no
  judgement reads the stored set, per node (read-side only, `DEC-059`).
- `RV-386` `F-1` scopes `DEC-301`'s move rule to **covered** nodes: a node added
  after the accepting act and later moved was never shown, so it does not re-face.
  Recorded as an amendment to `DEC-301` itself and pinned by `VT-4`.
- **Second pass (`RV-386` `F-6`–`F-14`).** The two attested rows split coverage:
  sufficiency compares covered material only; initial concerns (`ReviewedGraph`)
  also compares the full blocking set. The legacy act stays in its enums so
  stored snapshots parse, and the fallback is per node, so a partly-judged run
  keeps its unjudged blockers. `blocking` is one key with two homes; import seeds
  `blocking: true`; the change log reads the gate's predicate. *(Third pass,
  `F-15`–`F-18`: the review compares blocking marks regardless of lifecycle;
  legacy act kinds are a named read-only class, refused at write and excluded
  from the change log; `blocking` parses as `Sparse<bool>` and the contract
  renders it per home; the condition table keeps nine rows.)* `DEC-121`'s
  two-act letter is amended explicitly — both actors' judgements remain.
- **R1 — loosening invalidation is a truthfulness change.** `RFC-031` T1 was about
  the exit signal telling the truth; making a condition stop re-deriving is the
  same class of change in the other direction. A condition that *should* have
  invalidated and did not is a silent lie, so the derived half must be shown to
  still fire (see `VT-1`).
- **R2 — live snapshots change semantics underneath.** Bound by
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
  narrows `OQ-1`; historical — the declared set it describes is retired by
  `DEC-302`, and `R4` is now closed by the derived node judgements)*. `blocking-inquiries-dispositioned` quantifies over the
  **declared** blocking set, and its own doc comment says a declaration whose map
  has moved *"goes stale there rather than being silently re-read here"*
  (`src/design_run/gate.rs`, above `blocking_inquiries_open`). So the derived
  row's correctness **depends on** the attested rows' map-moved staleness.
  Narrowing the binding without preserving a "map moved ⇒ the set is re-declared"
  path would let a newly-added blocking node go unblocked — the defect `VT-1`
  exists to catch. This is why `OQ-1(A)` alone is unsafe and `OQ-1(B)` is the
  likely answer. Evidence: `research/research.md` cross-thread finding 1.

## Verification / closure intent

The criteria are design `sec-6` `VT-1`–`VT-7`; the slice holds their intent:

- **VT-1**: a declaration adding a node after `user-accepts-sufficiency` is
  attested does **not** void it, blocking or not; a node judged `blocking: true`
  stales `initial-concerns-recorded` and re-faces the user; and
  `blocking-inquiries-dispositioned` still reports unsatisfied for every open node
  whose effective judgement is blocking (`R4`). The derived half must be
  demonstrably *not* exempted, and the change log must agree with the gate.
- **VT-2**: a declaration supplying `needs: null` clears the set and emits one
  `NeedsRemoved` per removed edge; `null` on a node with no edges is a no-op *with
  no rows* — absence of change is not failure to report.
- **VT-3**: the pre-change behaviour is pinned first. The suite asserting that a
  shape change voids the cumulative conditions stays green until the change is
  deliberate, then flips against a named criterion rather than being deleted.
- **VT-4**: a move re-faces for a covered node and not for a node added after
  the act.
- **VT-5**: `blocking` is required at creation, `null` is refused, a flip emits
  one `NodeBlockingChanged`, and the key keeps its finding home unchanged.
- **VT-6**: a stored snapshot holding the legacy act parses, and its unjudged
  blockers survive a partly-judged map (with a negative control).
- **VT-7**: every creation path, import included, records a judgement.
- **VA**: on a real run, a node added late does not re-face the human gates —
  re-measured against `RFC-031`'s new fitness measure (nodes/edges added after
  `user-accepts-sufficiency`).
- Closes `IMP-469` and `ISS-481`; records the outcome and the `DEC-062`
  reconciliation on `RFC-031` T4.

## Summary

## Follow-Ups
