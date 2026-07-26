# DEC-039: CLI and MCP share one atomic observation create contract

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Decision

CLI and MCP recording are adapters over one observation-create service.

The friction CLI fast path is:

`doctrine observation record friction <summary>`

The generic command dispatches payload requirements by kind. It accepts optional
detail where the kind permits it, caller-stable UUID, registry-validated
`--set <facet.field>=<value>` assignments, structured request file/stdin input,
and an opt-out from best-effort enrichment. `--set` cannot create unknown facet
or field keys.

The confined MCP surface exposes only:

`observation_record({ uid?, kind, payload, facets?, enrich? })`

Both paths:

1. collect allowlisted automatic candidates;
2. overlay explicit values;
3. validate the kind, payload, facets, and origins;
4. inject UUIDv7 and UTC time when absent;
5. deterministically serialize the complete record; and
6. perform one atomic create-new write in the primary repository corpus.

The receipt contains UUID, repository-relative path, `created` or `replayed`,
and enrichment warnings. It performs no staging, commit, push, or index update.

A repeated caller UUID with semantically identical record input returns the
existing receipt. The same UUID with conflicting content fails and never
overwrites.

## Rationale

One service prevents CLI and MCP semantics from drifting while allowing the MCP
adapter to cross the Claude worktree jail without granting broad filesystem or
command authority. The minimal friction path stays cheap, while structured
input supports measurement and advanced callers.

Atomic create-new persistence and UUID replay provide retry safety without
content deduplication or hidden Git side effects.

## Consequences

- The pure layer receives clock, UUID, automatic candidates, and request data as
  inputs; filesystem and environment discovery remain in thin shells.
- Invalid explicit input creates no file. Automatic enrichment failures produce
  warnings and do not block an otherwise valid record.
- The MCP tool returns a repository-relative path and does not expose list,
  search, show, correction, or general command execution to confined workers.
- Trusted contexts may use the broader CLI read and correction surface.
- Structured stdin/request input must have the same validation and precedence as
  flags and MCP input.
