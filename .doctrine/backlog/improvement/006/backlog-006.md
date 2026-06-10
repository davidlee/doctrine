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
