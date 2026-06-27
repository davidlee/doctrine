# CHR-028: Delete dead orphan src/commands/superserde.rs (duplicate of supersede.rs)

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`src/commands/superserde.rs` is a near-byte duplicate of `src/commands/supersede.rs`
(only rustfmt line-wrap differs). It is **dead**:

- not declared as a module in `src/commands/mod.rs` (only `pub(crate) mod supersede;`);
- referenced nowhere in the tree (`grep superserde` finds only the file itself);
- pre-existing — introduced in commit `98c75027` ("wip"), present on `main`.

Because it is not compiled, it silently drifts from the real `supersede.rs`. A
future grep/rename/refactor hits a phantom and can edit the wrong file. SL-159's
`four → six` comment edit landed on it too (the inert spray that surfaced it during
the SL-159 audit, RV-172 F-3).

**Fix:** `git rm src/commands/superserde.rs`; confirm `just gate` stays green.

Surfaced by: RV-172 F-3 (SL-159 audit). Out of SL-159 design scope (pre-existing).
