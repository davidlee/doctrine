# Migrate record Supersedes storage to match the governance pattern

SL-097 ships record supersession in typed `[relationships]` arrays (consistent with
ADR's current storage). Once SL-095 migrates governance `supersedes` to `[[relation]]`
+ typed `superseded_by` carve-out, records will be the only supersession edges left in
typed arrays — an inconsistency to resolve.

This IMP tracks migrating the four record kinds to match: `[[relation]] label="supersedes"`
on NEW, typed `superseded_by` on OLD (the ADR-004 §5 carve-out). The verb
already targets typed arrays; the change is in the write path and the reader.

Gated on SL-095 landing — needs the proven `[[relation]]` + typed-carve-out split
before records adopt it.
