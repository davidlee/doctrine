# Notes SL-264: Inquiry-map growth without re-attestation

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Triage (2026-09-25)

Recorded to discharge the `explore.triage` runbook step. Detail lives in
`research/research.md` (gitignored) and the scope's own risks section — this is
the index, not a copy.

### Open questions

- **OQ-1 — the fix family.** Narrow the two attested rows' binding, or mint a
  derived "added since acceptance" condition? `R4` makes narrowing alone unsafe.
  *Load-bearing.*
- **OQ-2 — two staleness causes, not one.** `initial-concerns-recorded` goes
  stale via `CoverageStale` (coverage) and `ConfirmationStale` (the `confirms`
  link); only the first is a coverage question.
- **OQ-3 — does the human act survive?** Whether a shape change voids the
  sufficiency acceptance. `RFC-031`'s recorded position (2026-09-25): it
  survives.
- **OQ-4 — blocking, or warning?** The strength of the "map moved ⇒ re-declare"
  obligation. `DEC-101` supplies the warning precedent, `DEC-120` the case
  against silence.
- **OQ-5 — the no-op's rows.** Does an edge-free `needs: null` emit nothing?
  `REQ-478` does not settle whether a no-op is a "recorded mutation".
- **OQ-6 — stored snapshots.** Does the narrowed reading apply uniformly over
  the seventeen live runs?

### Risks

- **R1** loosening invalidation is a truthfulness change — the `RFC-031` T1 class
  inverted: a condition that should have invalidated and did not is a silent lie.
- **R2** seventeen live snapshots change semantics underneath; `DEC-059` prefers a
  reading-side change.
- **R4** the derived guard is coupled to the attested rows' staleness. The crux.

### Assumptions carried

- **A1** growth is permitted, unrefused and exercised; the disincentive is cost.
- **A2** `ISS-481` is wiring only — `apply_collection` already clears on `Null`
  (`submission.rs:66-71`), and `SPEC-029` Responsibilities[13] plus `DEC-063`
  already require it.

### Shaping decisions (so far)

- The map's growth obligation belongs in the `inquiry.md` lens, not the
  `inquiring` runbook (`DEC-104`) — landed as `IMP-470`.
- `SL-264` carries `IMP-469` + `ISS-481`; `IMP-386` (backward cascade) and
  `ISS-450` (the create branch) are fenced out deliberately.
- The `DEC-062` reconciliation is a new or superseding `DEC`: `revises` targets
  `&[SPEC, PRD, REQ, ADR, POL, STD]` and cannot reach a `DEC`.

### Constraining governance

See `research/research.md` § Thread 1. Load-bearing: `SPEC-029`
Responsibilities[13] (already requires the sparse fix), `DEC-062` (the conflict),
`DEC-063`, `DEC-120` (the principle this slice creates an exception to),
`DEC-101` (stale-warns precedent), `DEC-121`, `DEC-126`, `DEC-067`; `ADR-001`;
`ADR-019` + `DEC-127` (the shipped stage table is generated); `STD-001`.
Verified: `REQ-427` does **not** reach gate attestations, so no REV is owed.

## Review passes

Written after the first pass (2026-09-25, revision 24) to discharge `review.passes`.
A further pass, if one is wanted, would probe:

- **The `DEC-300` refinement.** The decision says *derived*; the design implements the
  growth obligation as an **attested-by-agent** condition (`blocking-set-current`),
  because the act already carries coverage and `DEC-126`'s ledger prefers attested
  where an act exists. It avoids a new `EngineSource` and a new derivation rule. The
  substance is unchanged, but this is a deviation from a recorded decision's wording
  and is the single most arguable choice in the design.
- **The comparison's API shape.** Whether the joiners-blind comparison belongs as a new
  method beside `ContentCoverage::diff` or as a parameter of it — the design says "a new
  method" without arguing against the alternative.
- **`submission.rs`'s involvement.** `sec-5` hedges ("where one is owed"). A second pass
  should establish whether the new row needs a key-home/state entry at all.
- **The new row's reach.** `Cumulative` means it re-derives at every edge above
  `inquiring→drafting`. That is what keeps the guard alive, but it also means a map
  addition during `drafting` blocks the next edge. Deliberate, worth a second look.
- **Test placement.** Whether the flipped pin in `src/design_run/tests.rs` plus new unit
  rows is sufficient, or an e2e is owed (no e2e file carries the gate table today).
- **`IDE-057`.** Whether a traversal-only apply owing no change row interacts with
  `REQ-478`'s "every mutation the run records" once `blocking-set-current` makes
  traversal-shaped state load-bearing.

A second pass would not re-probe: the mechanism mapping in `research/research.md`
(`material()`, `materials()`, the digest's binding, the two staleness causes) — all
verified against source during design.

## PHASE-01 — `needs: null` clears the set (completed 2026-09-25)

- Landed `37d26a5ff`. `declare_node`'s needs arm now matches all three `Sparse` states
  through one `Option<BTreeSet<DesignId>>` binding that feeds the **existing** difference
  loops — no second row loop, no new `ChangeEvent`. Closes `ISS-481`.
- **Finding for `/audit` (durable).** The plan's `VT-2` is RED-impossible in its plain
  form: an edge-free `needs: null` is the identity under *both* the buggy and the fixed
  code (before, `Null` reads as omission; after, it clears an already-empty set), so a
  test asserting only "no rows on an edge-free node" passes before the fix — a weak red.
  The worker strengthened it with a control clause (the same spelling *does* record
  removals where an edge exists) per `mem.pattern.tests.mutate-the-data-not-just-delete-it`
  and `mem.pattern.harness.grep-negative-needs-positive-control`; in that form it is RED
  before the fix. `VT-2` is therefore discharged by a control-bearing test, not by a
  direct no-op discrimination — read it that way at audit.
- Binding renamed to `declared_needs`: clippy `shadow_unrelated` is denied at zero warnings.
- Pre-existing and unrelated: `reserve::tests::vt3_auto_degradation_is_fail_closed_with_explicit_optin`
  fails in this jail via `DOCTRINE_RESERVATION_FALLBACK=1` (`ISS-483`); `src/reserve.rs` untouched.

## PHASE-02 — `blocking` is a node attribute (completed 2026-09-25)

- `InquiryNode`/`NodeMaterial` carry `blocking: Option<bool>`; the wire key `blocking`
  has two homes (`Finding` create-only, unchanged; `Inquiry` either state, required at
  creation, `null` refused); a flip emits one `NodeBlockingChanged`; import seeds
  `blocking: true`. The gate derivation is untouched — the set still derives from the
  stored legacy declaration until PHASE-04.
- **Plan deviation (for `/audit`).** The plan's `EX` file list did not name
  `src/design_run/refusal.rs`, but the refusals are unavoidable: the wire now needs
  `BlockingJudgementMissing` (creation omitted the judgement) and
  `BlockingJudgementWithdrawn` (`blocking: null`), built on the existing per-key
  pattern (`FindingSummaryMissing` et al.). Also touched beyond the list: the
  `InquiryNode::open` call sites in `fixture.rs`/`delegation.rs` and the contract
  render in `render/envelope.rs`.
- **New vocabulary, not just a row.** `Presence` gains a fourth variant,
  `RequiredAtCreation` — "required where the subject is created, optional on update,
  `null` refused". This is the gap `RV-385`'s verification pass named ("none of the
  three expresses the inquiry home"); the plan's `EX-4` said the compiler would force
  it, and it did. `KeyContract` gained `home: KeyHome` as the single source of the
  per-home render.
- **Refusal honesty.** `InertKey.honoured_by` widened from `IdKind` to `Vec<IdKind>` —
  with two homes, naming one would have been a lie by omission. `contract_check`'s
  admitted-key list deduped.
- **Sizing finding (durable).** This phase ran past one worker's 30-minute budget and
  was cut off *during* its final verification run; the work was complete, compiled and
  green, and the orchestrator finished verification. Six production modules, a new
  vocabulary variant, a regenerated golden and ~500 lines of test/fixture churn is more
  than one worker should charter. Size later phases to roughly half this, or split.
- Pre-existing and unrelated: `ISS-483`'s `reserve::tests::vt3_auto_degradation_...`
  remains the only red in the unit suite (the jail's `DOCTRINE_RESERVATION_FALLBACK=1`).

## PHASE-03 — legacy act kinds, retired (completed 2026-09-25)

- `ActKind::is_legacy()` names `BlockingSetDeclared` as the single source of the class;
  the invariant is restated (non-legacy exactly one contract row, legacy none);
  `admit_and_record`'s `None` arm refuses `Refusal::RetiredAct { kind }` instead of
  storing unchecked (`RV-386` F-16); `live_acts` excludes legacy kinds in both sets by
  *stated* rule (`STD-003`); the `blocking-set-declared` key left the contract model;
  the stored `confirms` digest is now a carried-driven, frozen read. Both goldens
  regenerated (`install/design-run-stages.md` moved with the rule's remedy line).
- **The A/B split was wrong, and the plan with it.** Dropping the `BlockingSetDeclared`
  `ActRequirement` from `initial-concerns-recorded` is entailed by *"legacy kinds have no
  rule"*, so the rule removal belongs with the class, not with the key's retirement.
  Doing it in one pass also let the submitting tests be flipped **once**. The plan's
  `EX-7`/Part-B split ("the class and the retirement land together") is satisfied; the
  intermediate cut was the error.
- **`EX-7`/B3 was unnecessary.** `stale_conjunct_does_not_satisfy`'s setup writes the
  store directly (`run.declarations.record`), never through admission, so the retired
  key could not break it. Its assertion is **unchanged and green** — the PHASE-04 flip has
  its pin intact, and the sheet's re-base task is dropped.
- **CRITICAL for PHASE-04 — a coverage hole the phase opens.** With no writable blocking
  set and the derived set not yet landed, a submission-produced run reports **no open
  blockers**: `blocking-inquiries-dispositioned` is trivially satisfied until PHASE-04
  derives the set from `blocking: true` nodes. `e2e_design_projection`'s
  `large_run_still_renders` (300 open blockers) was adapted in place with a comment.
  **PHASE-04 must restore that coverage from node attributes**, and `/audit` should treat
  the range PHASE-03→PHASE-04 as holding an intentionally trivially-met guard.
- `ActRequirement.confirms` is now always `None`; the field and its two readers
  (`confirmation_fault`, `confirmation`) are inert but retained — the design retires the
  *link*, not the field. `ContentCoverage::covers` was removed as unused.
- `VT-1` is discharged on the **admission** path; because the key left the contract, no
  wire submission reaches that arm, so `admit_and_record_refuses_a_legacy_kind_with_no_rule`
  pins it directly. `VT-4`'s test name does not match the `retired` cargo filter.

## PHASE-04 — the narrowed predicate and the derived set (completed 2026-09-25)

- **The slice's central behaviour change, landed.** `Coverage::ReviewedGraph` exists and
  `CoveredSet::moved` now takes the `Coverage` its act's rule names, so `coverage_moved`
  and `live_acts` cannot disagree about which nodes block. `initial-concerns-recorded`
  takes `ReviewedGraph` (carried material **plus** the full-set effective-blocking
  comparison, whatever the lifecycle); `user-accepts-sufficiency` keeps `InquiryMap` over
  carried keys. **`user-accepts-sufficiency` is now inert for a pure addition** — the
  design's intent (`sec-2` *Invisible*) and the loosening the slice exists to make.
- **Both projections were shown to still fire, not assumed.** Material moved on a covered
  node, a leaver among carried keys, and a new blocking node all still stale; a new
  non-blocking node and a resolved covered blocker do not; add-then-resolve still stales
  (`RV-386` F-15's reversal). This matters because loosening invalidation is a truthfulness
  change — `RFC-031` T1 inverted — so a condition that should still invalidate and no longer
  does would be a silent lie.
- **Two of the plan's criteria are guards, not red-to-green transitions.**
  `adding_a_blocking_node_stales_the_graph_review` and
  `adding_then_resolving_a_blocker_leaves_it_stale` did **not** fail before the change: the
  old union walk already staled on a joiner. They pin that `ReviewedGraph`'s full-set
  comparison still has teeth once the joiner arm is removed. `/audit` should read them as
  pins, not as evidence of a fixed defect.
- **`stale_conjunct_does_not_satisfy` was replaced, not deleted, and its pre-flip green was
  recorded verbatim** (the plan's VA-1). Seven criteria now stand where it did, plus new
  leaver and covered-flip criteria the plan did not name.
- **A test had to change because the assertion encoded the superseded behaviour.**
  `e2e_design_forward`'s `large_run_still_renders` listed `user-accepts-sufficiency` as an
  over-cap cause; under the narrowing it is current for a pure addition. The entry was
  dropped and a **positive** assertion added (sufficiency stays current on pure additions) so
  the narrowing is pinned at e2e altitude rather than merely unasserted.
- **Sheet/plan VT numbering differs**: the sheet called the pin `VT-3`; plan.toml's `VT-3` is
  the move criterion. Follow plan.toml — done.
- Residual: two mark-iterations (over `InquiryNode` and over `NodeMaterial`) share one
  judgement (`judged_blocking`); a reviewer may want them fused. `live_acts` is no longer
  rule-free — it looks each act's coverage up via `requirement_for`.

## PHASE-05 — mirrors, integration and the fitness re-measure (completed 2026-09-25)

- **The prompt prose corrected.** `install/design-prompts/conditions/initial-concerns-recorded.md`
  no longer teaches the retired two-act shape ("declare the blocking set you showed and record
  their review of it in one submission"); it now says the tree and its marks are the whole of
  what the user confirms, so there is no separate set to declare. Framing and voice untouched;
  no `blocking-set-current` prompt exists (nine condition prompts, the designed-and-dropped
  tenth absent).
- **The measurement is a test, not a claim.**
  `tests/e2e_design_forward.rs::map_growth_after_sufficiency_re_faces_only_for_a_blocking_addition`
  drives the built binary: a non-blocking addition after `user-accepts-sufficiency` re-faces
  nothing; a blocking addition re-faces `initial-concerns-recorded` and **only** that row
  (`user-accepts-sufficiency` is `InquiryMap` over carried keys). Measured **2 nodes and 1
  edge** added after the acceptance without voiding it — `RFC-031`'s quantity. Taken at the
  `reviewing` edge rather than inquiring→drafting; equivalent because both conditions are
  cumulative.
- **Selector amended: `src/commands/design.rs` (design-target).** The design's own decision
  that `KeyContract` gains the home it applies at forces the extern-contract construction site
  (`home: None`) in the command layer, and the node-declaring payload fixtures live there too.
  The slice's selector list omitted the file; the change was not out of bounds, the declaration
  was incomplete.
- **FINDING for `/audit` → `/reconcile`: `design.md` sec-5's code-impact table under-declares
  its own surface.** The diff touches six paths it never names: `src/commands/design.rs` (the
  escape above) and five `src/design_run/` modules — `contract_check.rs`, `delegation.rs`,
  `fixture.rs`, `refusal.rs`, `render/envelope.rs`. All five sit inside `src/design_run/**` so
  no selector escaped, but the table is the design's own account of what it changes, and its
  closing sentence commits the design-target selectors to that under-declared set. The first
  four were known from PHASE-02's notes; `contract_check.rs` (PHASE-03's legacy enum-token
  exception) was flagged nowhere until this sweep.
- **Nit for `/audit`.** `src/commands/design.rs:3549`'s unknown-key fixture carries a
  **duplicated** `blocking` key (`"blocking":false,"blocking":false`). Harmless — serde reads
  last-wins and the test targets the unknown `cursror` — but a duplicated key in a fixture is
  misleading and worth a ledger row.
- `doctrine check gate` green through fmt, clippy (zero warnings), eslint, build and validate;
  `test-all`'s only red is the pre-existing `ISS-483` reserve test. `architecture_layering` 25
  passed; all twelve `e2e_design_*` binaries green. `doctrine slice selector doctor 264`
  reports four **pre-existing** redundant selectors subsumed by `src/design_run/**` (advisory).

## Wrinkle for `/audit` — PHASE-01 has no boundary row

`.doctrine/state/slice/264/boundaries.toml` holds rows for PHASE-02..05 only. While
finishing PHASE-05 I accidentally re-issued `slice phase 264 PHASE-01 --status planned`
and then restored it to `completed`; the runtime status write cleared PHASE-01's
`code_start_oid`, and re-completing it reported *"phase-binding capture skipped for
PHASE-01: no code_start_oid stamped"*. The row was **not** hand-written back: the
boundaries ledger is machine-written disposable state, and a fabricated row would be
indistinguishable from a real one to a later reader.

PHASE-01's evidence is therefore the commit log, not the ledger: `37d26a5ff` (the
`needs: null` change) and `e6d0ee9c8` (its notes), i.e. the range
`6564b2bcc..e6d0ee9c8`. `/audit` should attribute PHASE-01 from those commits.

## Deviation for `/audit` — the `RFC-031` re-measure was synthetic (`RV-389` F-11)

Design sec-6 `VA` and PHASE-05 `EX-4`/`VA-2` ask for the re-measure **on a real run**, with
the run named. PHASE-05 substituted
`tests/e2e_design_forward.rs::map_growth_after_sufficiency_re_faces_only_for_a_blocking_addition`,
a synthetic e2e run, and presented it as the measurement rather than as a substitution.
That run crosses every edge below `reviewing` over an **empty** map, so the covered map at
`user-accepts-sufficiency` is empty: every later node is uncovered by construction and
`RV-386` F-1's covered-node scoping (the part a real run exercises) is never in play. It
is good regression evidence for the uncovered-addition half, not the `VA` the design asked
for. Disclosed here as a deviation for `/audit` to rule on — accept the substitution
(waive), or require a re-measure on a live run with a non-empty covered map at acceptance.
The covered-node scoping itself is evidenced at unit level (`a_move_re_faces_for_a_covered_node_only`,
and the lifecycle cases of `the_change_log_agrees_with_the_gate_on_every_case`), not on a run.

## `RV-389` repairs (2026-09-26)

Fourteen `fix-now` findings; F-5 is follow-up (`IMP-474`), untouched. Red-first evidence
is the mutation or regression each new test was shown to fail against, then reverted.

- **F-8 + F-13** — `fixture.rs::LEGACY_SNAPSHOT`, a frozen full-TOML stored snapshot (nodes
  unjudged, a `blocking-set-declared` act naming `inq-1`/`inq-4`, the legacy act's
  `act_recorded` row and a two-term `node_created` row), and
  `tests.rs::a_stored_legacy_snapshot_parses_and_keeps_its_pre_change_verdicts`, read through
  `snapshot::parse`. Asserts the effective set equals the stored act's, the mixed regime after
  judging another node, no row for the legacy act on that edit, and every condition's verdict
  against the **pre-change binary's**: taken by parsing the same text with the tree at
  `37d26a5ff^` (a `git archive` export, built out of tree) and asking `satisfied` of all nine
  conditions. They differ in exactly one place — `user-accepts-sufficiency`, stale by the
  `inq-3` addition alone, now current (the slice's intended change); `initial-concerns-recorded`
  was stale by the addition and a `ConfirmationStale` and stays stale on the second alone.
  In-test negative control: the act token renamed does not parse. Red: the review's
  mutant (`snapshot::parse` refusing text naming `blocking-set-declared`) fails it at the parse.
- **F-10 + F-14** — `tests.rs::the_change_log_agrees_with_the_gate_on_every_case` now asserts the
  full `ActInvalidated` subject set equals the gate-stale set over **both** attested acts
  (`cpa-graph`, `cpa-suff`), across six cases all driven through `apply`: non-blocking add,
  blocking add, covered re-word, covered flip, add-then-resolve (via a checkpoint disposition),
  and resolving a covered blocker. Red: `live_acts` excluding `SufficiencyAccepted` fails it on
  the covered re-word.
- **F-12** — `tests.rs::needs_empty_and_needs_null_are_one_clearing`: both spellings over the
  same edge-bearing and edge-free priors give equal rows and an equal stored node (helper
  `run_with_an_edge_free_node` extracted). Red: `needs: null` read as an omission fails it.
- **F-15** — the tautological unchanged-set assertion dropped; the doc says `apply`'s
  `&DesignSnapshot` makes it a type guarantee. Renamed `submitted_legacy_act_is_refused`, and
  `plan.toml` PHASE-03 `EX-6`/`VT-1` keywords updated to the new name.
- **F-9** — `change_log.rs`: `PayloadKey::Blocking` added; `NodeCreated`'s `payload_terms` gain
  `(Blocking, Label)`, rendered through the existing `judgement_label` vocabulary.
  `run.rs::created_prior` pushes the term. Budget: `node_created` saturates at 93 B, well
  inside the 145 B widest (`stage_moved`); stored two-term rows still read (the frozen
  fixture holds one). Red: `blocking_flip_emits_one_node_blocking_changed_row` now asserts
  the creation row's `blocking=non-blocking` label and failed with the key added but no term
  pushed. No `install/*.md` enumerates the terms. **For `/reconcile`:** `SL-233`'s
  `sketches/projection-bounds.md` §(d) table still lists `node_created` as *parent id,
  provenance* — the code calls that table its source; left for reconcile rather than edited
  in a closed slice.
- **F-1** — `run.rs::declare_node` rides `Sparse::apply_collection` for `needs`; the three-arm
  match and the `Option` wrapper are gone (an omission diffs empty).
- **F-2** — `gate.rs::Coverage::carried_shape` is the one Coverage→stored-shape statement
  (`attestation.rs::CoveredShape`, `CoveredSet::shape`); `admission.rs::coverage_fault` and
  `run.rs::covered_in` read it. `CoveredSet::moved` now takes `Option<&CoveredSet>`, holds the
  only fail-closed fallback, and gives `InquiryMap`/`ReviewedGraph` separate arms;
  `gate.rs::coverage_moved` shrank to a call. The fault still names a `Nodes` shape
  `inquiry-map` (`CoveredShape::incumbent`), so refusal text is unchanged.
- **F-3** — `run.rs::admit_and_record` branches on `ActKind::is_legacy` (a `None` rule for a
  non-legacy kind fails closed on the same refusal — panics are denied, so no `unreachable!`);
  `live_acts` computes each kind once behind one guard. Docs corrected (`gate.rs::requirement_for`).
- **F-4** — one set-builder, `inquiry.rs::blocking_marks` over materials beside
  `judged_blocking`; `attestation.rs`'s private copy and `InquiryMap::blocking_marks` removed;
  `open_blockers` filters on `effective_blocking` directly; the gate doc points at the builder.
- **F-6** — `Declaration::blocking()` → `finding_blocks()`.
- **F-7** — `run.rs::resolve_blocking`, pure, carrying the four-case doc; unit test
  `a_blocking_judgement_resolves_per_case`.
- **F-11** — the deviation above.
- **F-16** (round 2, option 1) — `run.rs::rehearse_proposal`, called from `delegate`'s
  `Propose` arm before the proposal is stored: rehearses the declarations over a scratch
  snapshot through the direct path's own `Batch::validate` + `declare_node` (so `blocking`
  missing at creation / `null` refuses exactly as a direct apply does), then refuses any
  remaining sparse `null` with new `Refusal::ProposalCannotClear { subject, key }` (names
  IMP-483). `Declaration::nulled_keys` + `Sparse::is_null`; pinned to the contract's sparse
  key set by `tests.rs::every_sparse_key_is_reported_when_null`. Only `inq-` subjects are
  rehearsed through `declare` — `cp-` needs a shell-minted record and `sec-` a body digest,
  so rehearsing them would refuse lawful proposals; `accept` still runs the full path. Stored
  encoding untouched (IMP-483). Red: e2e `a_proposal_carrying_null_is_refused_at_propose`
  (`blocking`/`question`/`needs`/`parent` null; revision unchanged, no proposal stored, edge
  kept) and `a_proposal_creating_an_unjudged_inquiry_is_refused_at_propose` both failed on
  `propose` succeeding. `install/design-prompts/delegation.md` gains one bullet; the payload
  contract states nothing about proposal nulls, so it is unchanged.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-09-26 · RV-389 code review done · pending-audit

### Produced
- RV-385 — re-homed RV-386's seven live findings (F-1..F-7) onto the run's pass; commits f491f490e, 3e5b85e00
- 01a0d83e — friction record: SL-264 as an ISS-322 recurrence after ISS-476
- RV-389 — pre-audit code review, 18 findings terminal; repairs 00c747b7d, c793fbfc3, 166f69a9f, e58a53f34

### Learned
- mem_01a0d8fea6887683af86b6aaf2e4b4f2 — Sparse null does not survive a TOML round-trip
- mem_01a0d17f827772b096e836f95a2887c4 — pass_stale is a lamp, not a gate; raise on the run's pass RV, never a second

### Open
- IMP-474 — KeyContract home duplicates WIRE_KEYS (RV-389 F-5)
- IMP-483 — stored proposals preserve Sparse null (RV-389 F-16 follow-up)
- ISS-322 — a run-minted pass cannot bind an externally conducted RV
