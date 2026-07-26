# DEC-048: Keep measurement schema while closing generic writers

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

SL-231 will define and validate the typed measurement observation and usage
facet wire schemas, but it will not expose a generic measurement writer.

The public `observation record` CLI path and the confined-worker
`observation_record` MCP tool accept friction observations only. In particular,
neither path accepts caller-asserted counters merely because the caller supplies
source metadata.

A measurement may be created only through a registered machine-source adapter
that supplies the structured request to the observation service. Registration
is an explicit trust boundary: the adapter must identify the harness-owned
counter source and provide source, scope, units, completeness, and supported
counters. Agent estimates and free-form operator assertions are not registered
measurement sources.

QUE-176 governs which real harness adapters may be registered and the boundary
and completeness each can truthfully claim. Until one is settled and
implemented, the production registry is empty and no measurement observation
can be created. SL-231 may exercise the service boundary with an injected fake
registered adapter in tests; that test seam does not become a public capture
interface.

Keeping the schema in the initial ledger establishes compatibility and validates
the collection primitive without prematurely asserting that any current harness
has trustworthy token instrumentation. EVD-002 identifies `claude -p` as the
leading first adapter candidate; registration remains contingent on verifying
its exact metrics and completeness under QUE-176.
