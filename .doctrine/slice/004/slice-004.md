# Implementation-plan and phase siblings

## Context

slice-003 shipped the entity-scaffold engine (`Kind` descriptor, fileset-as-function,
`acquire` seam, non-reserved sub-artefact path) and proved it across two shapes:
a top-level reserved slice and a non-reserved prose-only design doc. It was built
*fit to host* richer siblings but never exercised one — the design doc is a single
prose file with no relational data and no mutable state ([slice-003.md](../003/slice-003.md)
Non-Goals; [003/audit.md](../003/audit.md) §1).

The roadmap's next two change-side siblings are the first to break that simplicity,
and they break it along **two different axes at once**:

- **Implementation plan (IP).** Carries the project's first **relational authored
  block** — `plan.overview`: ordered phases, spec/requirement links
  (spec-driver-schemas `plan.overview@v1`). This is design-data: authored once,
  queried, fixed schema → it lives in TOML under the storage rule
  ([entity-model.md](../../../doc/entity-model.md) § The storage rule), the first
  facet doctrine actually reads rather than scaffolds-and-forgets.

- **Phase tracking.** Carries the project's first **mutable runtime state** —
  `phase.tracking`: status, start/complete timestamps, task counts, a timestamped
  progress log, per-task done/blocked flags (spec-driver-schemas `phase.tracking@v1`).
  *That list is the **source** schema being adapted; doctrine's v1 trims it to phase
  status + progress log and graduates the rest on demand (design.md D5/§5.6, Scope
  below).* This is *not* authored design schema. The entity model draws a hard boundary
  around exactly this churn ([entity-model.md](../../../doc/entity-model.md)
  § Runtime state; [spec-entity-spec](../../../doc/spec-entity-spec.md)
  § Design-data vs runtime-state): runtime state lives under a separate
  `.doctrine/state/` tree, mutable and likely gitignored, **not** in the
  authored-entity taxonomy beside specs/slices/audits.

So this slice is where two deferred boundaries first become real code: the
**relational-facet** storage rule (read/queried TOML, edit-preserving append) and
the **design-data ÷ runtime-state** split. It is also where the engine takes its
first **multi-file sub-artefact fileset** — which inherits a writer debt
slice-003's audit explicitly deferred here.

## Scope & Objectives

- **Implementation-plan sibling.** `doctrine slice plan <id>` (verb TBD) scaffolds
  an IP sub-artefact inside `.doctrine/slice/<id>/` carrying `plan.overview` as an
  **authored relational facet** (TOML row data + prose companion per the storage
  rule), not an embedded fenced block. Spec/requirement link fields are present in
  the schema but empty in v1 (no registry yet — same posture as slice
  `[relationships]`). It is a non-reserved sub-artefact; its id derives from the
  parent slice.

- **The design-data ÷ runtime-state boundary, built.** `phase.tracking` churn lives
  in a **separate `.doctrine/state/` tree that mirrors the slice path** — canonical
  `.doctrine/state/slice/<id>/phases/` — never inside the authored slice dir. This
  keeps the authored tree (`.doctrine/slice/<id>/`) pure: authored `toml`+`md` only,
  reviewable and git-tracked, while the whole `.doctrine/state/` subtree is
  gitignored and disposable (`rm -rf`-able, matching the runtime-state ethos —
  [entity-model.md](../../../doc/entity-model.md) § Runtime state). The state tree is
  **not** written by the scaffold engine: design.md D3 homes it in a separate
  `src/state.rs` mutable-runtime module (the engine stays write-once/refuse-clobber),
  sharing only the `fsutil` path-safety primitives. Future runtime facets (session,
  locks, review-index) sit as siblings under `.doctrine/state/slice/<id>/`.

- **Phase access is symlink-blind; id is identity.** A convenience symlink
  `.doctrine/slice/<id>/phases → ../../state/slice/<id>/phases` is created on write
  for humans browsing, but the **tool resolves the canonical state path from the id,
  never by following the symlink** — same rule as the slug symlink already in
  slices-spec (convenience alias, not authority). The symlink is **also gitignored**
  (`.doctrine/slice/*/phases`), so it never dangles into the ignored state tree on a
  fresh clone; the tool recreates it on demand. Read-locality (the one cost of the
  split tree) is recovered by a CLI reassembler, not by a tracked symlink.

- **Phase materialisation from the plan.** Phases are **declared in `plan.toml`** (the
  ordered list, id-bearing — the single durable source of each phase's objective,
  criteria, verification expectations, links), so phase sheets are scaffolded *from the
  authored plan*, not independently id-reserved — no second-level reservation primitive.
  Each phase materialises a `phase-NN.toml` (runtime status + progress log) **and a
  `phase-NN.md`** (disposable narrative: assumptions, risks, decisions, findings,
  task-details), both in the state tree. Phase content sorts on two axes — *durability*
  (definition→durable plan; status→disposable tracking) × *structure* (tool-queried→TOML;
  narrative→md) — with the discriminator **"does the tool read it," not "has a checkbox."**
  Per-criterion / verification / task *status* ships as a markdown checklist and
  **graduates to TOML rows when a consumer lands** (first: exit-criteria, at the done-gate);
  graduation is free because the graduating content is *status* — disposable runtime
  (loss-accepted, gitignored), not durable data that would migrate.

- **Two slice-folder file conventions.** `notes.md` — durable, tracked, scaffolded
  on demand from a simple template (`doctrine slice notes <id>`, the `design.md`
  single-file pattern); a per-slice scratchpad. `handover.md` — disposable, gitignored,
  **no template and no verb**: an agent convention for surviving a session boundary.
  The matched pair sits on the same durable/disposable axis as plan vs phase sheet.

- **Per-fileset transactionality (inherited debt).** The IP is the first
  `CreateInExistingEntity` kind writing **more than one file** under a parent it
  does not own. slice-003's `create_in_existing` writer has no partial-write
  cleanup, so a mid-fileset failure leaves leftovers ([003/audit.md](../003/audit.md)
  §2 `[M]` — *"the first `>1`-file `CreateInExistingEntity` kind needs per-fileset
  transactionality … it inherits this writer"*). This slice discharges that:
  stage-and-rename or track-and-unlink, with a test for mid-write failure.

- **First relational-facet writer.** The IP facet is read and round-tripped, so its
  mutating path must be **edit-preserving** (`toml_edit`, not serde reserialize) to
  honour the comment/unknown-key preservation the notes promise
  ([entity-model.md](../../../doc/entity-model.md) § Rust implementation model;
  [spec-entity-spec](../../../doc/spec-entity-spec.md) § Known risks). Whether v1
  needs any *mutation* verb or only scaffold+read is a design-doc decision.

End state: a slice can grow an authored implementation plan (relational facet,
queryable) and a runtime phase-tracking surface that lives under `.doctrine/state/`,
cleanly separated from the authored tree. The engine has now hosted a multi-file
fan-out fileset with partial-failure cleanup, and the storage rule's read/query
edge is exercised for the first time — so the spec family (next) inherits a proven
relational-facet writer rather than a theoretical one.

## Non-Goals

- **Spec / requirement registry and FK validation.** `plan.overview` spec/req link
  fields stay empty (no registry to point at). Coverage gates, `doctrine validate`,
  and cross-entity FK resolution are the spec-family slice, later.

- **Multi-agent / resumable run orchestration.** Phase tracking is a *record* of
  progress, not an execution engine. Session state, heartbeat/lease, and
  review-orchestration runtime are deferred ([entity-model.md](../../../doc/entity-model.md)
  § Runtime state) — `.doctrine/state/` is established here but only for phase
  tracking; the broader runtime surface is not built.

- **General table-mutation verb surface** (`req add`-style row appends to arbitrary
  facets). The IP facet is authored/scaffolded; the full mutate-existing-table
  machinery is the spec family's concern ([003/audit.md](../003/audit.md) appendix
  M-nonreserved: file-creating sub-artefacts ≠ row-appending sub-artefacts).

- **`git-ref` reservation backend.** Unchanged — phases derive ids from the plan and
  do not reserve; the `acquire` seam is untouched.

- **No `PHASE-` reserved entity.** The location decision is pinned (slice-scoped
  runtime state under `.doctrine/state/slice/<id>/`), which makes phase tracking a
  **slice-scoped runtime facet, not a reserved authored entity** — no `PHASE-` kind,
  no second-level reservation. Phases are materialised from the authored
  `plan.overview` list. (A future `phase`/run entity is only revisited if resumable
  multi-agent execution demands it — [entity-model.md](../../../doc/entity-model.md)
  § Entity vs facet taxonomy; out of scope here.)

## Summary

Add two change-side siblings that split along the design-data ÷ runtime-state seam:
an **implementation plan** (`plan.overview` as the first authored relational facet,
read and edit-preserved) and **phase tracking** (`phase.tracking`, the first mutable
runtime state, homed in a separate gitignored `.doctrine/state/slice/<id>/` tree
mirroring the slice path — the authored slice dir stays pure; a gitignored
convenience symlink aliases it but the tool resolves by id, never follows it). Phase
sheets materialise from the authored plan (no new reservation). The IP is the engine's first multi-file
sub-artefact fileset, so this slice discharges the per-fileset transactionality debt
slice-003's audit deferred here. Behaviour-preserving for slices/design docs — the
existing suites gate every step.

Approach, the IP facet schema, the phase-materialisation flow, and the
transactional-writer design live in the design doc (to be authored:
`doctrine slice design 004`, then adversarial review per the slice-002/003 rhythm).
The runtime-state home is decided here (split `.doctrine/state/` tree, symlink-blind
tool); the design doc implements it.

## Follow-Ups

- **Spec family** — the relational-facet writer and storage rule proven here become
  the spec decomposition's foundation (`requirements.toml`, `coverage.toml`, …);
  still registry-gated ([spec-entity-spec](../../../doc/spec-entity-spec.md)). The
  coverage join (requirement ↔ change side) may want optional **`feature`** (functional)
  / **`quality`** (NFR) join granularity, reaching *phase* level, not just slice —
  endpoints don't exist yet; this slice's `[[phase]].requirements` field is the
  anticipating hook. Premature to model before requirements land.
- **`doctrine slice validate`** — design-doc/IP presence is workflow-significant but
  unobservable (slice-003 audit §3 / M5); the queryable marker lands when the
  validate verb does.
- **Slice close-out audit** — the durability backstop for the disposable phase sheet
  (audit finding 3) and the GC for `handover.md` (finding 7). When a slice closes, the
  close-out audit **harvests** the phase sheets' execution-time risks/decisions/findings
  into the tracked audit artefact (the durable home those notes otherwise lack), and
  **GCs** the gitignored `handover.md`. Until it lands, phase-sheet findings are working
  notes and `handover.md` is hand-deleted.
- **Broader runtime state under `.doctrine/state/`** — session/handoff/review-index
  caches join phase tracking once multi-agent execution needs them
  ([entity-model.md](../../../doc/entity-model.md) § Runtime state).
- **Agent memory** — parallel research track ([memory-spec / memory-contract] local
  notes); a runtime-state-shaped, externally-substrated subsystem whose home is
  informed by the `.doctrine/state/` boundary this slice establishes. Separate
  go/no-go, not sequenced here.
