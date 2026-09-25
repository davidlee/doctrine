# IMP-477: Add a corpus census surface over review findings

RFC-026 **E1** needed base rates over the review corpus (findings per ledger by
facet, severity mix, disposition mix, convergence). There is no CLI surface; it
took four hand-written Python passes over 312 raw `review-NNN.toml` files, with a
correctness trap — two reviews named `RV-323` exist in different trees, so any
census must dedupe on `(id, title)` (RFC-026 E1/E7.4; obs `019fbd4c`).

`doctrine findings` is unrelated (interestingness over the priority graph).

Fix: a read-only census verb over review findings. Blocked in spirit by `ISS-279`
(id collisions) and `IMP-433` (derived-status reach). See RFC-032 `research.md` F10.
