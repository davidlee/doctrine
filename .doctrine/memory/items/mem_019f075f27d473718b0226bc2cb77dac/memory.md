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
.doctrine/slice/*/coverage.toml` (dedupe the dir + slug-symlink double-walk,
ISS-006).

Worked example: SL-165 modified REQ-316 (REV-014) and attested REQ-317; close
needed REC-093/REC-094, each with a same-value `active→active` delta and two
evidence_refs (SL-064 + SL-165 cells). Companion to the integrate step
(mem_019ec912f7fd746284bfaef00717443e — land the admitted close_target via
`sync --integrate --trunk main`); discharge is the step that follows it.
