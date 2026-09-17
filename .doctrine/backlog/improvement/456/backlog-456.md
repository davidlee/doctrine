# IMP-456: CatalogNode fixture helper — one optional field touched 29 constructions

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Adding one optional field (`memory_key`) to `CatalogNode` at IMP-454 broke 29
hand-rolled struct constructions across `src/catalog/dot.rs`,
`src/catalog/graph.rs`, and `src/map_server/routes.rs` — every one a test
fixture spelling out every field.

The field was backfilled mechanically, and a `graph_of` helper now covers the
`CatalogGraph` wrapper in `dot.rs` tests, but the node literals remain.

A fixture builder (or `Default` plus struct-update syntax) would make the type
evolvable: adding a field would touch the builder, not every call site. The
same shape of duplication exists for `CatalogEntity` in `hydrate.rs` tests.
