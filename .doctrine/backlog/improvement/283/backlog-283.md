# IMP-283: elicit.rs: bundle rank_map+depth into one impact-band context

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Deferred at SL-217 PHASE-02 gate (recorded in notes.md): `rank_map` + `depth`
thread through four assembler fns in `src/priority/elicit.rs` as one implicit
impact-band context. Bundle into a single context struct — behaviour-neutral
reshape of four call sites, skipped at the finish line to keep the phase
diff clean.
