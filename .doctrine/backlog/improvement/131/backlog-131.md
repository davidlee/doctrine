# IMP-131: Consolidate 4 parallel id-to-toml-path resolvers

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Four near-identical "id → entity TOML path" resolvers exist, today adjacent in
`main.rs`:

- `resolve_link_path`
- `resolve_supersede_path` (its comment: "Mirrors `resolve_link_path` — the same
  `KindRef` `(dir, stem)` path map")
- `resolve_dep_seq_src_path`
- `resolve_entity_path_and_canonical`

All four walk the same `KindRef (dir, stem)` map to turn a canonical id into the
entity's `*.toml` path. A parallel implementation (CLAUDE.md "no parallel
implementation"); the candidate single home is `integrity.rs` (already owns
`parse_canonical_ref` / `ensure_ref_resolves`).

**Surfaced by SL-115** (main.rs decomposition). SL-115 relocates each resolver
into its verb's `commands/` shell (`relation.rs`, `supersede.rs`, `dep_seq.rs`,
`facet.rs`) — scattering them across four files and burying this dedup target.
SL-115 deliberately does **not** consolidate (it is scoped behaviour-preserving /
mechanical; a resolver merge is behaviour-changing). This item captures the debt
at its most visible moment, before the scatter.

Each relocated resolver carries a one-line breadcrumb comment pointing here.
