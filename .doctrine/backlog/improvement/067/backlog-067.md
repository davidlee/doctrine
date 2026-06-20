# IMP-067: Corpus-wide entity id→path helper (entity::id_path over KINDS)

## Context

Widens the original narrow framing (delegate only the `dep_seq_for` SL arm to
the slice module). The real defect is corpus-wide: the entity id→path formula
`<dir>/<NNN>/<stem>-<NNN>.{toml,md}` is rebuilt inline ~24 times — across
`relation_graph.rs` (the SL **and** REV arms), `revision.rs` (7), `slice.rs` (8),
`review.rs`, and `main.rs` (6). No kind owns its own path mapping; each copy is an
independent chance to drift the layout and a place a new kind must be remembered.

This is the `kind-is-data-not-trait` pattern already blessed in
`relation_graph.rs` for `outbound_for` / `dep_seq_for` (dispatch over
`integrity::KINDS` as data). Path construction never got the same treatment — yet
the data already lives on the `KINDS` row (`dir` + `stem`).

## Approach

One data-driven helper — `entity::id_path(root, kind, id, Ext)` over
`integrity::KINDS` — closes all ~24 sites at once and removes the REV-arm
duplication the narrow framing didn't even list. Behaviour-preserving: the
existing suites stay green unchanged (the behaviour-preservation gate is the proof).

## Open (per loop proposal 0004)

- **slice-worthy vs quick-design** — ~24 mechanical replacements + one helper +
  behaviour-preservation gate reads slice-sized; leans slice.
- **helper signature** — `Ext` enum (`Toml` / `Md`) vs `&str`; relative vs
  root-joined path.

Supersedes the narrow `dep_seq_for` SL-arm scope. Ref: loop proposal 0004.
