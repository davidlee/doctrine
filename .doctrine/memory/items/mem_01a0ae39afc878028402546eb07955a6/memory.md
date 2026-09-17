`tests/e2e_design_legacy_corpus.rs::every_authored_design_in_this_repo_reads_losslessly`
runs every authored `design.md` through the legacy reader. The reader refuses
**non-blank bytes before the first heading** with
`Refusal::UnheadedPreamble { line }` (`src/design_run/legacy.rs`).

The `<!-- doctrine:section sec-1 -->` marker comment is fine above the heading —
comments are skipped. Prose is not.

This bites at `/reconcile`, where the natural instinct is to head the document
with a banner ("this design was locked before a scope cut; read sec-5 as intent,
not as shipped"). Put the banner **under** the sec-1 `##` heading instead. It
still reads as the first thing after the section title, and the gate stays green.

Cheap to hit and cheap to fix, but it fails in `just test-all` / `doctrine check
gate` rather than at write time, so it costs a full gate cycle if you only find
it at the end.


## Related

Why a reconcile banner is a direct edit in the first place, and why the design
run's fingerprints are *expected* to diverge from the file afterwards:
[[mem.pattern.reconcile.edit-design-out-of-band]].
