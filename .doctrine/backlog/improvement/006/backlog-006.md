# IMP-006: Uniform destructive + lifecycle-transition verbs across kinds (delete/archive, status-transition)

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Context

No kind has a uniform lifecycle-transition or destructive verb. `slice-nnn.toml`
`status` is hand-edited; `backlog edit` reimplements its own status transition;
adr/spec/standard/policy have no transition verb at all. Each kind that *does*
move status carries a bespoke implementation. The known-gap "no slice lifecycle
transition" (CLAUDE.md) is the slice-shaped face of this.

## The lifecycle FSM is trapped in slice.rs

The transition machinery — `classify`, `set_slice_status`, `is_terminal_status`,
`is_drifted`, `is_divergent`, `decorated_status`, `transition_label`
(`src/slice.rs:400-713`) — is real, well-tested, slice-only state-machine code.
`conduct.rs` holds the *other* half of ADR-009 (slice-lifecycle-fsm-conduct-axis):
the conduct **axis** (Actor / Autonomy — *who* gates a transition), and `slice`
already calls `conduct::resolve` to layer posture onto a status. But the FSM
itself never left `slice.rs`.

Delivering this item is the moment to **extract the lifecycle FSM into a shared
engine module** (sibling to `conduct.rs`, completing the ADR-009 pairing) so
slice / backlog / adr / spec transitions reuse one `classify` + terminal-set +
edit-preserving setter instead of re-growing bespoke copies.
`slice::is_terminal_status` was already left at module scope precisely so this
verb could reuse it — the handhold is in place.

## Scope sketch

- Shared lifecycle engine: transition classification, terminal-status set,
  edit-preserving authored-TOML status setter (the
  `mem.pattern.entity.edit-preserving-status-transition` shape, generalized).
- Per-kind transition vocab + seam rules as data the engine consumes (cf. the
  GovKind data-not-trait pattern, SL-033).
- Destructive verbs (delete / archive) as a separate but adjacent concern —
  share the claim/seam machinery in `entity.rs`.
- Reconciles the SL-009 status-vs-rollup divergence surfacing into an actual
  transition that can resolve it.

## Supersession — the transactional carve-out verb (from SL-048 OD-3)

A specific member of this item's lifecycle-transition family, split out of SL-048
and parked here to avoid a gov-only point solution (parallel implementation):

- **The verb** (e.g. `doctrine <gov> supersede <new> <old>`, ideally uniform
  across kinds that supersede — gov→gov, slice→slice) performs **one transaction**:
  write the forward `supersedes` edge on `<new>`, flip `<old>` to terminal
  `superseded`, and co-write the `superseded_by` carve-out on `<old>` (the only
  honest place a reader of the dead record finds its successor — ADR-004 §5, the
  predecessor file is rewritten for the status flip anyway, so zero marginal
  coupling).
- **Why it isn't SL-048's:** SL-048 deliberately walls `supersedes` off as
  `LinkPolicy::LifecycleOnly` — a plain `link` append would be half a transaction.
  ADR-010 D4 fixes the carve-out as *verb-written, never hand-authored*; that verb
  is this lifecycle axis, not SL-048's relation-capture axis.
- **Unblocks the SL-048 OD-3 exclusion:** once this verb exists, governance
  `supersedes` can migrate to the uniform `[[relation]]` block alongside the rest
  of tier-1 (SL-048 excluded it precisely because no transactional owner existed).
- SL-048 ships only the read-side guard: a `validate` cross-check that stored
  `superseded_by` agrees with the derived `in_edges` reciprocal (see corrected
  [[IMP-032]]).

Related: [[SL-048]] · [[IMP-032]] · ADR-010 D4 · ADR-004 §5.
