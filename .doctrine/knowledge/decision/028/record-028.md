# DEC-028: Confined Claude workers receive only observation_record

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Decision

The confined Claude worker toolset receives exactly one writable observation
capability: the purpose-built, create-only `observation_record` MCP tool.

The Doctrine MCP server may expose observation show, list, search, supersede, and
retract operations to trusted or orchestrator contexts. Those tools are not granted
to confined workers merely because they share the same server.

## Rationale

Claude MCP calls execute outside the worker's additional bwrap wall. That is the
delivery seam needed to reach the repository-wide observation sink, but it is also
the escape described by RSK-225. Security therefore depends on the worker tool
allowlist and the server operation both being narrow.

Workers need to preserve a raw signal; they do not need correction authority or a
general writable Doctrine surface. Create-only UUID semantics bound the granted
operation to one new immutable file or an idempotent replay.

## Consequences

- Worker agent definitions explicitly name `observation_record`; they do not inherit
  broad writable Doctrine MCP access.
- The tool validates kind, payload, facets, UUID replay, and the resolved primary
  sink before writing.
- The tool cannot select an arbitrary filesystem destination.
- Supersession and retraction are performed by a trusted caller.
- SL-231 verification includes a worker-tool allowlist assertion and a negative
  test that the tool cannot write outside the observation corpus.
