`Sparse<T>` (`src/design_run/submission.rs`) has three states on the wire —
`Omitted` (persist), `Null` (clear), `Value` (replace) — but its `Serialize`
writes both `Null` and `Omitted` as none. TOML cannot hold a none, so the key
is dropped, and on the next read `#[serde(default)]` supplies `Omitted`. A
`null` that meant *clear* comes back meaning *persist*, and nothing reports it.

This bites any path that **stores** a `Declaration` rather than applying it at
once. The known instance is a delegated proposal: stored at `Propose`, replayed
at `Accept` (`RV-389` `F-16`). SL-264 closed it by refusing any `null` at
`Propose` (`Refusal::ProposalCannotClear`, via `Declaration::nulled_keys`) and
by rehearsing the proposal through the direct path's rules. Restoring
null-clearing through storage is `IMP-483`.

**How to apply:** before storing a `Declaration` (or any struct carrying
`Sparse`) in a snapshot, either refuse `Null` up front or give it an encoding
that distinguishes `Null` from `Omitted` — and keep stored snapshots readable
(`mem.fact.design-run.snapshot-outlives-the-binary`). A JSON round-trip test
does not catch this; test the TOML round-trip.
