# DEC-022: Observation is the sole v1 public interface

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Decision

V1 exposes the reusable Observation primitive directly through one generic,
kind-dispatched CLI surface:

```text
doctrine observation record <kind> <summary>
doctrine observation show <uuid>
doctrine observation list [--kind <kind>]
doctrine observation supersede <uuid> ...
doctrine observation retract <uuid>
```

There is no `doctrine friction ...` alias in SL-231. The MCP interface mirrors the
same primitive and operations; an agent-facing tool may default the observation
kind to `friction` where its tool contract is purpose-specific.

`record` is an action verb here, consistent with existing CLI surfaces. Observation
is the noun and capability identity; this decision does not add another entity
family called Record.

## Rationale

Observation now has a stable core, registered typed facets, typed kind payloads,
and a durable UUID identity. Exposing that reusable primitive avoids duplicating
capture, read, and correction interfaces for each future observation kind.

Adding a friction-specific alias before real usage demonstrates a need would create
two public shapes to maintain without adding capability.

## Consequences

- Observation-kind registration owns payload and facet validation.
- Generic commands reject unknown kinds and invalid supplied structured values.
- Human and agent callers share the same underlying operations.
- Convenience aliases remain a compatible later addition if invocation evidence
  shows the generic surface is too costly.
