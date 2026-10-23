# Notes SL-285: Fork regenerates derived runtime

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Further review pass (after RV-425 integration, rev 28)

A further pass would probe what RV-425's repairs introduced, not what it
already covered: whether `plan_wire`'s dry-run arms report every path the live
arms write (the one claim the design defers to the implementer), whether the
`installed-version = "999.0.0"` lever reaches `regenerate` without an earlier
refusal, and whether the post-legs `git status` drift check false-reds on any
legitimately tracked write. All three are settled faster by the first phase's
tests than by another design pass, so none is recommended.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-10-23 · design reviewing rev 28 · RV-425 integrated

### Produced
- design run at reviewing; six sections drafted (commits 5cc8ce64b..c4064b1b5)
- RV-425 — independent pi design pass, F-1..F-8 disposed fix-now and integrated into sec-1..sec-6 (rev 28); awaiting user section review + disposition
- minted: CHR-181 — classify harness link dirs in the worktree tier map
- ISS-352 — client pi-harness fork evidence appended
- raw research incl. independent pi design + comparison: `research/raw/pi-*.md` (gitignored)

### Learned
- mem_01a1680243ea78e383f785c81fff6e07 — apply_wire dry-run reports no touched paths
- mem_01a1680244397e23b4c63de9a0223fad — linked worktrees read main's install marker; ignored hook files bake the absolute exec

### Open
- DEC-402 — regeneration legs + tracked-file guard; consequence corrected (RV-425 F-7)
- DEC-393 — harness set from source
- DEC-396 — provision composes copy and regenerate; amended to the ProvisionMode seam (F-5) and six callers (F-2)
- DEC-397 — creation compensates, repair preserves; creation list gains coordination + hook passthrough (F-2)
- DEC-399 — downgrade refusal
- DEC-401 — success contract, no marker
