# IMP-202: Close drift-discharge legibility — recipe + richer slice-status-done error

**Source:** SL-165 PIR S1,S2,S6 (close complexity cascade, HIGH). **Home:** RFC-011.

`slice status done` refuses with a one-line `undischarged residual drift on
requirement(s): …` and zero guidance. The accept-REC recipe — the 3-clause
`rec_discharges` predicate (`move=accept` + `status_delta` matching *current
authored* status + `evidence_ref` ⊇ ALL residual coverage keys, including other
slices' cells) — had to be reverse-engineered from `src/slice.rs`. ~4 round-trips
per governed close.

**Fix direction:** (a) richer error naming the flagged REQs, the accept-REC
pattern, and a pointer; (b) `/close` skill documents the recipe; (c) surface the
existing memory `mem_019f075f`.

Related: RFC-011; IMP-192 (L0 close/orientation friction cluster).
