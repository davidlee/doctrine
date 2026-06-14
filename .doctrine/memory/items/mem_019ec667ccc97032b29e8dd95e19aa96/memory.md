# dispatch worktree review branch carries extraneous deletions — filter at integration

When a dispatch worker produces code on an isolated worktree (merge-base far
behind main HEAD), the review branch amalgamating the phase commits will diff
against main as massive deletions of everything not in the worktree's partial
tree. At `/close`, integrate ONLY the slice's additions as conventional commits —
**never merge the review branch whole.**

The SL-067 audit (RV-031) is the precedent: the `review/067` branch (built from
`phase/067-01` + `phase/067-02`) had correct SL-067 code but deleted:

- `src/revision.rs` (entire REV kind, 1478 lines)
- `src/relation.rs` Revises label + REV kind constant
- `src/main.rs` Revision command variants (~215 lines)
- 5 revision e2e test files
- All pre-067 slices, specs, reviews, requirements, ADRs, backlog items, memory items
- `install/templates/revision.{md,toml}`

The correct integration path: cherry-pick each SL-067 concern as a conventional
commit onto main, filtering extraneous deletions. The review branch is for audit
inspection only, never for merge.
