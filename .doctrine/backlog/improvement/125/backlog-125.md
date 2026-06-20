# IMP-125: Consolidate per-kind id parse_ref onto a shared listing helper

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Context

The **parse** mirror of SL-114 (which consolidated the id-**format** side onto
`listing::canonical_id`). SL-114's §Non-Goals explicitly carved this out as a
follow-up. The id **parse** side — case-insensitive prefix strip + bare-id
fallback + digit parse — is copy-pasted across ~6 kinds with no shared helper.

## Evidence — the scattered `parse_ref` family

Near-identical bodies (`strip_prefix("XX-").or_else(strip_prefix("xx-"))` + digit
parse), one per kind:

- `rec.rs:381`, `revision.rs:517`, `review.rs:1024`, `slice.rs:1024`,
  `concept_map.rs:148` — same shape, different prefix literal.
- `governance.rs:196` — already partly generic (`parse_ref(g: &GovKind, ref)`),
  the closest existing prior art for what a shared helper looks like.
- `backlog.rs:1134` — `parse_ref -> (ItemKind, u32)`, the multi-prefix variant.
- `requirement.rs:267` — `id_from_fk` (the FK-canonicalize cousin).

Each hard-codes its prefix literal twice (upper + lower); a format/prefix change
touches N sites — the same DRY smell SL-114 fixed on the format side.

## Proposed shape

A shared `listing::parse_ref(prefix: &str, reference: &str) -> Result<u32>`
(symmetric to `listing::canonical_id`), case-insensitive on the prefix with the
bare-id fallback, that each kind delegates to with its `Kind.prefix`. `governance`'s
generic form and `backlog`'s `(Kind, u32)` multi-prefix variant are the design
inputs for whether one helper covers both, or a small family does.

## Non-Goals / boundaries

- Unrelated `strip_prefix` sites are NOT in scope: `skills.rs`, `fsutil.rs`,
  `worktree.rs`, `dispatch.rs`/`state.rs` (`PHASE-` parsing), `coverage.rs`,
  `lazyspec.rs` — these parse non-canonical-ref strings.
- Behaviour-preserving: every existing `parse_ref_*` test stays green unchanged.

## Links

- Mirror of [SL-114] (id-format consolidation) — its §Follow-Ups names this.
