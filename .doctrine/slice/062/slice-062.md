# Uniform lifecycle-transition + destructive verbs across kinds

> Scopes IMP-006. See that backlog item for the originating survey and the
> SL-048 OD-3 supersession carve-out detail.

## Context

No kind has a uniform lifecycle-transition or destructive verb. The transition
machinery that *does* exist is trapped in `slice.rs` — `classify`,
`set_slice_status`, `is_terminal_status`, `is_drifted`, `is_divergent`,
`decorated_status`, `transition_label` — real, well-tested, but slice-only.
`conduct.rs` already holds the *other* half of ADR-009 (the conduct axis: Actor /
Autonomy — *who* gates a transition), and `slice` calls `conduct::resolve` to
layer posture onto a status. The FSM itself never left `slice.rs`.

Meanwhile `backlog edit` reimplements its own status transition, and
adr / spec / standard / policy have no transition verb at all — each kind that
moves status carries a bespoke implementation or none. The "no slice lifecycle
transition" known-gap (now SHIPPED for slice via SL-040) was the slice-shaped
face of this broader absence.

This slice extracts the **pure FSM primitives** (`classify`/predicates/edge table)
into a shared leaf (sibling to `conduct.rs`, completing the ADR-009 pairing) and
the **edit-preserving write-core** into one shared IO seam every status setter
delegates to — NOT a single uniform cross-kind transition engine (codex C8:
slice's `run_status` still composes classify with its RV close-gate, drift, and
conduct posture in the shell; those stay per-kind). Plus the transactional
supersession verb. The reuse pattern is GovKind data-not-trait (SL-033) for the
write seam, not per-kind copies. Destructive verbs are carved out (F1).

## Scope & Objectives

- **Extract a shared lifecycle engine** from `slice.rs`: transition
  classification (`classify`), terminal-status set, and the edit-preserving
  authored-TOML status setter (the
  `mem.pattern.entity.edit-preserving-status-transition` shape, generalized).
  Slice transitions must reuse the extracted engine — behaviour-preserving, the
  existing slice suites stay green unchanged (the behaviour-preservation gate).
- **Per-kind transition vocab + seam rules as data** the engine consumes
  (legal states, ordered transitions, terminal set, ordering/closure guards) —
  kind-is-data-not-trait. Wire slice (re-homed) + backlog onto it first; adr /
  spec / standard / policy as the vocab table is populated.
- **The transactional supersession verb** (SL-048 OD-3 carve-out): one
  transaction that writes the forward `supersedes` edge on `<new>`, flips
  `<old>` to terminal `superseded`, and co-writes the `superseded_by` carve-out
  on `<old>` (ADR-004 §5 / ADR-010 D4: verb-written, never hand-authored).
  **ADR-first** (only ADR has a `superseded` status today; POL/STD/slice lack it
  — design §6 D4 / F-C). Unblocks migrating governance `supersedes` into the
  uniform `[[relation]]` block (the SL-048 OD-3 exclusion).
- Reconcile the SL-009 status-vs-rollup divergence-*surfacing* into a transition
  that can actually *resolve* it (slice already does via SL-040; preserve under
  the extracted engine).

## Non-Goals

- **Destructive verbs (delete / archive)** — carved out to follow-up F1 (design
  §9). The file-level destruction semantics for committed authored entities
  (archive-status vs git-rm vs tombstone) are unsettled and adjacent, not core;
  they share only the `entity.rs` claim seam, no FSM.
- **Supersession for POL / STD / slice** — deferred to follow-up F2 (they lack a
  `superseded` status). This slice ships ADR supersession only.
- Not re-opening relation *capture* (the `[[relation]]` write seam, SL-048 /
  ADR-010) — this is the lifecycle axis, orthogonal. Supersession touches the
  typed `supersedes`/`superseded_by` fields but is owned here per ADR-010 D4.
- Not the cross-kind needs/after capture axis (SL-060) — but this slice
  *generalizes* `dep_seq::append` (SL-060's seam), keeping its suites green.
- Not a new conduct/gating policy — reuse `conduct::resolve` as-is.
- Not the SL-048 OD-3 `supersedes` typed→`[[relation]]` storage migration (F3) —
  this verb *unblocks* it; the migration is downstream.

## Related work

- **SL-060** is the structural template. It lifts a backlog-welded seam
  (`needs`/`after`) into a shared leaf + per-kind dispatch (`dep_seq_for`
  mirroring `outbound_for` / `status_and_title_for`). The FSM extraction here is
  the same "lift once, dispatch per kind" shape — reuse that idiom, no parallel
  implementation — this slice *generalizes* `dep_seq::append` rather than
  re-rolling array-append. Note SL-060's non-destructive refuse-message lesson:
  an edit-preserving seam must never fall back to "regenerate via `<kind> new`"
  (it would nuke an authored entity) — pinned as a `set_authored_status` test.
- SL-060 reinstates a typed `[relationships].needs/after` table on slices,
  ordered **before** the `[[relation]]` rows (F-1 hazard). This slice's
  shared `set_authored_status` + `append_string_array` touch the same
  files and must preserve **both** blocks. The supersession verb writes the
  typed `supersedes`/`superseded_by` fields (not `[[relation]]`), so SL-060's
  capture seam and this slice are storage-orthogonal — no overlap.

## Affected surface (design §5.1 confirms)

- `src/lifecycle.rs` (NEW, pure leaf, beside `conduct.rs`) — the re-homed FSM
  (`classify`/`Transition`/predicates/edge table). Completes the ADR-009 pairing.
- the authored-TOML-mutation seam (grow `dep_seq.rs`, OQ-3) — shared
  `set_authored_status` (scalar) + `append_string_array` (reuses `dep_seq::append`'s
  string-array path, parametrized by field — NOT the `after` struct path; codex C2).
  The IO half; kept OUT of the pure `lifecycle.rs` (D1).
- `src/slice.rs` — FSM donor; `run_status` keeps the gate + RV close-gate,
  delegates the write. `SLICE_STATUSES`/`SliceStatus` stay (read-filter + CLI vocab).
- `src/governance.rs`, `src/backlog.rs`, `src/requirement.rs` — bespoke setters
  retired onto the shared `set_authored_status`, each keeping its own gate.
  (`requirement::set_status` is status-**only**, no `updated`; there is **no** spec
  status setter — `spec req status` delegates to requirement. Design §1/§2, codex C1.)
- `src/adr.rs` / the new top-level `supersede` handler — the supersession verb +
  `supersede_policy` (ADR `Some`, others `None`).
- CLI command wiring for `doctrine supersede`.

## Risks / Assumptions / Open Questions

- **R1** — behaviour-preservation: the FSM extraction + setter rewire must not
  perturb slice/gov/requirement/backlog transition output (no spec setter exists —
  codex C1). Existing suites are the proof (assertions unchanged; import paths shift
  on the module move).
- **R3** — reusing `dep_seq::append`'s string-array path must not regress SL-060's
  needs/after
  suites (they stay green unchanged).
- **R-atomicity** — the supersession verb writes two files non-atomically;
  mitigated by pre-flight + not-already-superseded guard + idempotent re-run +
  the shipped SL-048 PHASE-05 `validate` detectability (design §5.4 R1).
- **A1** — `slice::is_terminal_status` was left at module scope precisely for
  this reuse; the extraction handhold is in place.

Resolved at design (were OQ-1/2/3): altitude = **C** (re-home FSM + share setter);
supersession = **ADR-first**; destructive verbs = **carved to F1**. Remaining
execution-detail OQs (test home, hint wording, seam module name) live in design §9.

## Verification / Closure Intent

- Shared engine exists; slice + backlog transitions both route through it.
- Slice behaviour-preservation suites green unchanged.
- New engine has its own unit suite (classify, terminal-set, edit-preserving
  setter) driven through the write seam, not just pure helpers.
- Supersession verb proven as one transaction (forward edge + status flip +
  carve-out) with a test driving `run()`; governance `supersedes` carve-out
  migration demonstrated or explicitly deferred with a follow-up.
- `just gate` green; zero clippy warnings.

## Follow-Ups

- Populate transition vocab for any kinds deferred out of this slice.
- SL-048 OD-3: migrate governance `supersedes` to the uniform `[[relation]]`
  block once the supersession verb lands (if not done here).
