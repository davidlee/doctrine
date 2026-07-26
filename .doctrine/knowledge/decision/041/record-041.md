# DEC-041: Observation engine architecture and verification boundary

SL-231 introduces a dedicated `observation` engine rather than extending the
authored-entity engine. Observations have distinct UUID identity, single-file
storage, append-only correction, and tolerant-read semantics. The engine reuses
shared repository-root and safe-filesystem primitives without inheriting the
numbered TOML-plus-Markdown entity lifecycle.

The engine is split by responsibility:

- `wire` owns typed envelopes, payloads, facets, origins, control records,
  schema dispatch, and strict validation without disk, clock, UUID generation,
  Git, or harness dependencies;
- `resolve` purely derives active and historical projections plus deterministic
  diagnostics from supersession and retraction controls;
- `query` purely filters, orders, searches, and paginates loaded records;
- `store` is the sole observation filesystem seam, owning partition discovery,
  tolerant loading, atomic create-new, and UUID replay checks; and
- the module façade accepts injected time, identity, enrichment, and repository
  root and exposes the common service used by thin CLI and MCP adapters.

The CLI owns rendering, clock and UUID generation, and the full create, read,
search, supersede, and retract surface. The confined MCP tool exposes the same
create contract only for primary signal kinds (`friction` and `measurement`);
it cannot create supersession or retraction controls or address arbitrary
paths. Confinement grants and conformance checks must name this capability
explicitly.

Verification covers typed round trips and invalid inputs; atomic concurrent
creation, replay, collision, containment, and non-torn writes; deterministic
resolution and malformed-control diagnostics; resolved and historical query
semantics; CLI/MCP parity and enrichment precedence; MCP refusal of control
kinds; ADR-001 layering; worker grant conformance; and unchanged existing
entity, memory, comparison, dispatch, and MCP suites.

IMP-320 separately owns configuration-driven boot guidance. SL-231 supplies the
capture interface that guidance can later activate.
