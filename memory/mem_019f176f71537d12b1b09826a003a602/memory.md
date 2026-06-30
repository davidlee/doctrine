# Closure drift discharge via accept REC

`doctrine slice status <id> done` refuses — `undischarged residual drift on
requirement(s): …` — whenever a gate requirement's authored status diverges from
its scanned coverage composite (verdict `Divergent`/`Indeterminate`, not
`Coherent`). A slice that **modifies a requirement** (REV) or **records VA/VH
coverage** at reconcile will trip this at close.

Discharge each flagged requirement with an `accept` REC owned by the closing
slice. The predicate (`src/slice.rs` `rec_discharges`) demands ALL THREE:

- **(a)** `move = "accept"`.
- **(b)** a `[[status_delta]]` naming the requirement whose `to` equals the
  requirement's **current authored status** (guards a status edited away-and-back).
  If status did not change, record a same-value delta (e.g. `from="active"
  to="active"`) — `from` is unchecked; only `to == authored` matters.
- **(c)** the REC's `[[evidence_ref]]` set ⊇ **every** distinct coverage key
  feeding that requirement's composite — including cells from OTHER slices (e.g.
  the origin slice's original VA attestation), not just your own. Miss one and a
  "stale contradictory evidence" guard keeps it undischarged.

Recipe: `doctrine rec new --move accept --owning-slice SL-NNN --title "accept
REQ-NNN"`, then hand-author the `[[status_delta]]` + `[[evidence_ref]]` tables
into `rec-NNN.toml` (the CLI seeds only the skeleton; the file's own comments
sanction the reconcile writer appending them). One REC per requirement, mirroring
the origin slice's pattern. Find the keys: `grep -rl REQ-NNN
.doctrine/slice/*/coverage.toml`. Dedupe the hits — each coverage cell can surface
twice, once via its real directory and once via a slug-named symlink alias to the
same dir; a naive count double-walks them.

Discharge is the step that *follows* the integrate step at close: once the
admitted `close_target` has been landed on the trunk (`dispatch sync --integrate`),
discharge clears the residual drift the integration exposed.

Worked example — an illustration from Doctrine's own development (the ids below
are historical, not a live cross-reference into your repo): SL-165 modified
REQ-316 (via REV-014) and attested REQ-317; close needed REC-093 and REC-094,
each carrying a same-value `active→active` delta and two evidence_refs — a cell
from SL-064 plus the SL-165 cell.
