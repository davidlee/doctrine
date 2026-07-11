# IDE-037: Evergreen spec home for projection contract

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->
Projection contract P1–P15 (comparison tier-3 gauge/placement semantics) lives
in a closed slice's design doc (SL-213 design.md §3, amended by SL-216). Durable
contracts consumed by module docs (src/comparison/project.rs points readers
there) want an evergreen tech-spec home, not a slice artefact chain. Extract to
a tech spec when the comparison layer stabilises (post-RFC-019 Phase C).
