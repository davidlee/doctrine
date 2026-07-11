
Projection contract P1–P15 (comparison tier-3 gauge/placement semantics) lives
in a closed slice's design doc (SL-213 design.md §3, amended by SL-216). Durable
contracts consumed by module docs (src/comparison/project.rs points readers
there) want an evergreen tech-spec home, not a slice artefact chain. Extract to
a tech spec when the comparison layer stabilises (post-RFC-019 Phase C).
