# DEC-040: Observation reads and corrections use resolved projections and append-only controls

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Decision

V1 exposes:

- `observation show <uuid> [--resolved]`;
- `observation list [filters] [--history]`;
- `observation search <text> [filters] [--history]`;
- `observation supersede <old-uuid> <replacement-uuid> [--reason]`; and
- `observation retract <uuid> [--reason]`.

`show` renders exact identity by default. `--resolved` explicitly follows the
successor chain. List and search use the resolved active projection by default;
`--history` includes superseded and retracted observations and control records.

Queries implement the accepted structured and lexical semantics, deterministic
newest-first/UUID ordering, and explicit compatibility diagnostics.

Correction writes one control record per command. A supersession replacement
must already exist, be a public observation, and be kind-compatible with its
target. Retraction targets exact identity; retracting a non-terminal superseded
node does not retract its terminal successor, and the CLI warns and identifies
that successor.

## Rationale

Exact `show` preserves UUID citations while resolved collection reads serve the
common current-signal use case. Requiring an existing replacement keeps
supersession to a single atomic create-new write and avoids pretending that two
filesystem writes can be transactional.

Exact-target retraction avoids hidden chain-wide side effects. Users can inspect
or explicitly target the resolved successor when that is their intent.

## Consequences

- Resolution is a pure deterministic fold over public and control observations.
- Cycles, dangling targets, multiple successors, incompatible replacements, and
  conflicting controls are validation errors rather than timestamp disputes.
- Correction commands never edit original files or perform Git actions.
- The authored corpus is the source of truth; derived indexes cannot change
  results.
- Confined worker MCP access remains create-only even though trusted adapters
  may expose the same read semantics elsewhere.
