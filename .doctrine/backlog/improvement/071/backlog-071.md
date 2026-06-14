# IMP-071: Wire dispatch record-orthogonal verb when the OQ-B orthogonal classifier lands (SL-064 deferred funnel-time writer)

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

SL-064 PHASE-06 wired the funnel-time `dispatch record-boundary` verb (the
claude-arm phase-cut input) but **left `record-orthogonal` unexposed**. The ledger
helper `ledger::record_orthogonal` + `orthogonal.toml` model exist and are tested,
but there is no CLI verb — its driver is the **OQ-B orthogonal classifier**, a
deferred plan-gate (what marks an entity slice-orthogonal vs impl-bundle). Until
that classifier exists, an empty `orthogonal.toml` is the correct conservative
`review/<slice>` EXCLUDE fallback: nothing is excluded ⇒ everything is reviewed once
(design §4.2 "verified, not merely listed; reviewed once, never lost").

When OQ-B lands: expose `doctrine dispatch record-orthogonal --slice --entity --path
[--status]` (mirror `record-boundary`), remove the `expect(dead_code)` on
`ledger::record_orthogonal`, and align the dispatch skill's funnel step to mark
ahead-projected entities. See `src/ledger.rs` (record_orthogonal), `src/dispatch.rs`
(run_record_boundary as the template), SL-064 design §4.2.
