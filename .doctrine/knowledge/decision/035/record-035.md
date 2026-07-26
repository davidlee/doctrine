# DEC-035: V1 registers friction and measurement observation kinds

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Decision

The v1 kind registry contains two public observation kinds:

- `friction`, whose payload requires the accepted non-empty summary and may
  include detail; and
- `measurement`, whose schema requires at least one registered typed measurement
  facet and represents a raw measurement rather than an analytical conclusion.

Supersession and retraction use reserved internal control kinds. They are part of
the resolution protocol, not public domain kinds.

Future capabilities, including the planned drift ledger, extend the registry
with deliberately authored kind schemas. V1 does not accept ad hoc runtime kinds
or schemaless custom payloads.

## Rationale

`friction` proves the cheap human/agent signal path. `measurement` is materially
different: it supports trustworthy late usage records without pretending that a
measurement is friction or requiring mutation of an earlier observation.

Two registered kinds exercise the reusable substrate while keeping extension
governed and typed. A runtime custom-kind mechanism would either require dynamic
schema governance now or recreate the rejected metadata-bag design.

## Consequences

- Kind registration owns payload validation and the set of compatible or
  required facets.
- The generic CLI and MCP interfaces dispatch through the registry rather than
  hard-coding a friction-only storage path.
- Public callers cannot directly create reserved control kinds; correction
  commands construct them through the resolution API.
- New kinds can reuse the stable core, storage, querying, and correction
  machinery without changing observation identity.
