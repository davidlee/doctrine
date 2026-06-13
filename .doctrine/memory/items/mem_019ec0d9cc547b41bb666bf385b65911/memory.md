# Structural relations are [[relation]] rows (SL-048 'the cut'); writer and read-side display are unwired as of SL-057

Since SL-048 PHASE-04 ("the cut") tier-1 entity relations are uniform
`[[relation]]` rows (`label` + `target`), NOT the legacy typed `[relationships]`
table. Author them structurally — do not default to prose ("no structural surface
in v1" is the pre-SL-048 world and is now false).

Legal slice labels (source = slice): `specs` (→ SPEC-NNN **or** PRD-NNN),
`requirements` (→ REQ-NNN), `supersedes` (→ SL-NNN), `governed_by` (→ ADR/POL/STD).
The legal `(source, label)` set lives in `RELATION_RULES` (`src/relation.rs`),
read by `read_block`. Row form:

```toml
[[relation]]
label = "specs"
target = "SPEC-002"
```

Tooling is HALF-WIRED end to end as of SL-057 (each gap has a backlog item):
- **No `link`/`unlink` CLI verb** — the writer exists in `src/relation.rs` but is
  not surfaced; you must hand-author the TOML rows. (IMP-048)
- **Read-side shows nothing** — `inspect` (`outbound: []`) and `slice show`
  (`relationships: { specs: [] … }`) do NOT render authored outbound tier-1
  relations, table or json. Confirmed on the done slice SL-047. Relations are
  write-only in practice. (ISS-010)
- **`slice new` scaffolder emits the stale `[relationships]` comment**, not the
  `[[relation]]` idiom — misleads toward prose. (ISS-009)
- **No agent guidance** on how/when to relate. (IMP-049)

Verification that hand-authored rows are correct: `doctrine validate` reads them
and reports `corpus clean` (read-tolerant) even though no display surface shows
them. Validate is the only confirmation until the read side is wired.

Related: [[mem.pattern.review.superseded-by-is-adr004-carveout]],
[[mem.pattern.entity.numbered-kind-identity-table]].
