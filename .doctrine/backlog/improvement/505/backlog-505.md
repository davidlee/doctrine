# IMP-505: Second restate pass over shipped skills

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

SL-273's audit re-pass (design §6.4, RV-414 F-4) ran a fresh restate pass
over `plugins/doctrine/skills/**`, blind to the first inventory. It found
71 blocks (27 owned-concept, 26 flag-shape, 18 ownerless); 58 have no
counterpart in `.doctrine/slice/273/inventory.toml`. The rows, each with
`owner_carries` and the overlapping first-inventory row, are in
`.doctrine/slice/273/inventory-audit.toml`.

Most are a summary paragraph beside an existing `lib:` citation. That is a
stricter bar than PHASE-03 applied, and some are sanctioned —
`lib:reference/review-ledger.md` lets each review skill restate the trigger
in its own voice. So the list is leads, not confirmed misses.

Work: settle the bar (DEC-345 says cut every flag shape and owned concept;
does "summary + citation" in a skill count?), adjudicate the 71 rows against
it, apply cuts, and log ownerless rows to IMP-500. Reuse SL-273's method —
inventory, adjudication, per-occurrence verifier (`.doctrine/slice/273/verify-sweep.nu`).
The three owner contradictions the pass surfaced are ISS-505.
