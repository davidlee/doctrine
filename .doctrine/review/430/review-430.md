# Review RV-430 — code-review of IMP-526

Adversarial-review ledger. Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

<!-- Pre-reading + lines of attack: what this review is probing, the invariants
     it must hold the subject to, and where the bodies are likely buried. Seeded
     at `review new`; the reviewer fills it before raising findings. -->

Subject: commit `aea84237e` (IMP-526), implementing DEC-421 (graph-reviewed binds
carried-keys material only; inquiry growth surfaced by relay in both
`map_delivery` modes). Lines of attack:

- Gate / change-log agreement: `coverage_moved` (gate) and `live_acts` (change
  log) must still read one predicate, `CoveredSet::moved`, under the rule's
  `Coverage`.
- Residue of the removed `Coverage::ReviewedGraph` / blocking-marks comparison in
  code, comments, tests, shipped docs (`install/`), generated artefacts, specs,
  and accepted decision records.
- Relay line: emitted on every map-changing write (apply, start --from-design;
  adopt never touches the map), in relay and sidecar modes; names added ids and
  the consultation rule; absent when the map is unchanged.
- Test adequacy for the flipped behaviour and the new line.
- Stale comments.
