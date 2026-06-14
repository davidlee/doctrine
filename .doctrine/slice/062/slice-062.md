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

This slice extracts the lifecycle FSM into a shared engine module (sibling to
`conduct.rs`, completing the ADR-009 pairing) and exposes uniform
status-transition and destructive verbs that every kind consumes as data — the
GovKind data-not-trait pattern (SL-033), not per-kind copies.

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
- **Destructive verbs (delete / archive)** as an adjacent concern sharing the
  claim/seam machinery in `entity.rs`. Define what each means per tier (authored
  files are committed + diffable — "delete" semantics need care vs git).
- **The transactional supersession verb** (SL-048 OD-3 carve-out): one
  transaction that writes the forward `supersedes` edge on `<new>`, flips
  `<old>` to terminal `superseded`, and co-writes the `superseded_by` carve-out
  on `<old>` (ADR-004 §5 / ADR-010 D4: verb-written, never hand-authored). This
  unblocks migrating governance `supersedes` into the uniform `[[relation]]`
  block (the SL-048 OD-3 exclusion).
- Reconcile the SL-009 status-vs-rollup divergence-*surfacing* into a transition
  that can actually *resolve* it (slice already does via SL-040; preserve under
  the extracted engine).

## Non-Goals

- Not re-opening relation *capture* (the `[[relation]]` write seam, SL-048 /
  ADR-010) — this is the lifecycle/destructive axis, orthogonal. Supersession
  touches both but is owned here per ADR-010 D4.
- Not the cross-kind needs/after capture axis (SL-060).
- Not a new conduct/gating policy — reuse `conduct::resolve` as-is.
- Not deleting/migrating existing authored files; no corpus migration beyond the
  governance `supersedes` carve-out the supersession verb enables.

## Related work

- **SL-060** is the structural template. It lifts a backlog-welded seam
  (`needs`/`after`) into a shared leaf + per-kind dispatch (`dep_seq_for`
  mirroring `outbound_for` / `status_and_title_for`). The FSM extraction here is
  the same "lift once, dispatch per kind" shape — reuse that idiom, no parallel
  implementation. Note SL-060's non-destructive refuse-message lesson: an
  edit-preserving seam must never fall back to "regenerate via `<kind> new`"
  (it would nuke an authored entity) — load-bearing for the destructive verbs / R2.
- SL-060 reinstates a typed `[relationships].needs/after` table on slices,
  ordered **before** the `[[relation]]` rows (F-1 hazard). This slice's
  edit-preserving status setter and destructive verbs touch the same files and
  must preserve **both** blocks. ADR-010 D1 keeps `needs`/`after` off the
  relation vocab, so SL-060's capture seam and this slice's supersession
  `[[relation]]` edge are orthogonal — no overlap.

## Affected surface (provisional — design confirms)

- `src/slice.rs` — FSM extraction donor; transitions re-homed onto the engine.
- new shared module (sibling to `src/conduct.rs`) — the lifecycle engine.
- `src/backlog.rs` — bespoke status transition retired onto the engine.
- `src/entity.rs` — destructive-verb claim/seam sharing.
- `src/governance.rs` — supersession verb target; `superseded_by` carve-out.
- CLI command wiring for the new verbs.

## Risks / Assumptions / Open Questions

- **R1** — behaviour-preservation: extraction must not perturb slice transition
  output (decorated status, labels, classify edges). Existing slice suites are
  the proof; they stay green unchanged.
- **R2** — "delete" of an authored, committed entity is semantically loaded
  (git history vs working tree). Design must pin tier-aware semantics before any
  destructive verb ships; archive may be the only honest authored-tier verb.
- **OQ-1** — vocab coverage: which kinds get transition verbs in *this* slice vs
  deferred? (slice + backlog are load-bearing; gov/spec/standard/policy may be
  vocab-only.)
- **OQ-2** — does supersession generalize gov→gov *and* slice→slice in one verb,
  or ship gov-first? (IMP-006 sketches "ideally uniform".)
- **OQ-3** — destructive vs lifecycle: one slice or split? IMP-006 frames them
  adjacent but separable; design may carve destructive out to a follow-up if the
  FSM extraction + supersession already fill a shippable unit.
- **A1** — `slice::is_terminal_status` was left at module scope precisely for
  this reuse; the extraction handhold is in place.

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
