# IMP-418: Reconcile slice owed items against phase sheet findings

A mechanical check that every phase-sheet finding is accounted for in the slice's
tracked § *Owed* ledger **before** the sheets are discarded.

## The gap

Phase sheets are runtime state and gitignored. `notes.md` § *Owed to the
reconciliation brief* is the tracked record that has to survive them. Keeping the
two in step is currently a manual orchestrator sweep with **no mechanical
backstop**.

`SL-248` is the evidence. Its § *Owed* ledger ran to 176 items and drifted three
times by its own account — items 142, 143 and 176 each record a drift caught by a
later manual sweep. Item 176 is the sharpest: `PHASE-10`'s `T13`/`T14` findings
`F-64`–`F-67` map to § *Owed* items 172–175 only via cross-references written
*in the phase sheet* (`phase-10.md` lines 785 and 842). At merge those pointers
go with the gitignored file and the ids become unresolvable from the tracked
record alone.

Every drift was caught. That is the argument for the check, not against it: three
catches in one slice is a rate, and the catches were luck-adjacent — a sweep that
happened to run.

## Shape

Advisory, not blocking, and run at audit or close while the sheets still exist:

- read each phase sheet's § *Findings* (`F-NN` ids and titles);
- read `notes.md` § *Owed*'s item bodies;
- report findings with no citation in any owed item, and owed items citing a
  finding id that no sheet carries.

Both directions matter. The first is the drift `SL-248` hit; the second catches
an owed item whose provenance has gone stale.

Prose parsing is the obvious objection. It is tolerable here because the output
is advisory and the failure mode is a false positive a human dismisses — as
opposed to a gate that blocks on a regex.

## References

`SL-248` `notes.md` § *Owed* items 142, 143, 176 · `RV-352` `F-5` · `LOOP.md`
§ *Notes, sharded* (never put anything load-bearing only in gitignored state)
