# IMP-061: Fold knowledge::set_record_status onto the SL-062 set_authored_status seam

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

SL-062 PHASE-02 lifted the byte-duplicated edit-preserving TOML write-core into one
seam (`set_authored_status` + the `apply_status`/`apply_string_append` pure cores,
in `src/dep_seq.rs`) and retired the gov/slice/backlog/requirement setters onto it.

A **fifth** setter — `knowledge::set_record_status` (`src/knowledge.rs:1283`) — is
byte-identical to the donor recipe but was out of SL-062's declared scope, so it was
left untouched. Folding it onto `set_authored_status` (status-only managed key-set,
its own gate kept in the shell) completes the DRY collapse. Its golden
`tests/e2e_knowledge_cli_golden.rs` may need the non-destructive F-1 hint rewording
(EX-4 parity) if the message is pinned there.
