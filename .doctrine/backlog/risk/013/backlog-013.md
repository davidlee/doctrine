# RSK-013: scan_coverage silently skips malformed/unreadable coverage.toml — closure gate needs a strict scan mode

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Surfaced by the SL-179 external adversarial pass (codex GPT-5.5), deferred as
out-of-scope for RSK-008.

`scan_coverage` skips unreadable or malformed coverage files
(`src/coverage_scan.rs`, ~176). The closure gate consumes `scan_coverage`, so a
foreign `coverage.toml` that is corrupt or unreadable — yet contains a live
`Failed`/`Blocked` cell feeding a gate-set requirement — silently drops out of the
composite. SL-179's forget-refusal (D2) closes the *CLI erasure* path but not the
*malformed-file* path: a contradiction can disappear from the gate's view without a
recorded act.

Proposed: a **strict scan mode** for closure-time use — any unreadable/malformed
coverage file that could contribute to a gate-set requirement refuses close (fail
closed), distinct from the lenient read-view scan that tolerates partial corpora.

Origin: RSK-008 / SL-179 design. Refs: SPEC-002 D8, `coverage_scan.rs`.
