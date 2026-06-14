# SL-067 implementation notes

## Audit (RV-031)

Reconciliation audit complete. The implementation is design-conformant — all
four locked decisions (D1-D4), all five adversarial findings (F1-F5), and every
EN/EX/VT criterion hold. `cargo clippy` zero warnings; 1286 tests pass.

Single blocker (F-1, resolved): `review/067` is a dispatch worktree amalgam
that carries extraneous deletions of revision.rs, relation.rs Revises label,
main.rs Revision command, all pre-067 slices/specs/reviews, and 5 revision e2e
tests. The SL-067 code additions are correct; the branch must NOT be merged
whole. At `/close`, integrate only the SL-067 additions as conventional commits.

Minor finding (F-2, resolved → follow-up): dynamic column visibility lacks a
dedicated table-header assertion. The logic is simple and correct; captured as
a backlog follow-up.

Pattern recorded: `mem.pattern.dispatch.review-branch-extraneous-deletions`.

## Decisions carried from design

No decisions were revised during implementation.

## Risks carried forward

1. **PerToken coupling is by convention only.** `split` and `cell` both read
   `tags` but nothing in the type system couples them. The F1 guard test
   (`strip_ansi(coloured) == plain == cell(r)`) pins it.
2. **Intra-array comments not preserved.** The sorted-array replace rebuilds the
   `tags` array, dropping inline comments. Accepted (A5) — tags are machine-written
   from a seeded `[]`.
3. **Empty colon-segments tolerated.** `:x`, `a::b` render coloured colons with
   no segment text. Charset permits colon positionally.
