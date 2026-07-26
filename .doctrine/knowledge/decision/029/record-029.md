# DEC-029: Observation queries default to resolved active records

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Decision

Observation list and search project the resolved active view by default:

- retracted observations are omitted;
- a supersession chain contributes only its terminal replacement; and
- observation-control records are omitted.

`--history` includes original, superseded, and retracted observations plus their
control records.

`show <uuid>` is identity-faithful. It renders the exact record named by that UUID
and annotates its resolution state, successor chain, or retraction. It never
silently redirects a citation. An explicit resolved projection may follow the
chain and render the current terminal observation.

## Rationale

Most collection reads need the clean current signal set, not correction machinery.
But UUIDs are durable citations: silently replacing the object shown for a cited
UUID would make historical references ambiguous and conceal why the current view
differs.

Separating exact identity from resolved projection preserves both everyday
ergonomics and auditability.

## Consequences

- Resolution is a deterministic pure fold over immutable observation and control
  documents.
- Cycles, dangling targets, conflicting terminal controls, or multiple active
  successors are corpus errors surfaced by validation rather than resolved by
  timestamp precedence.
- CLI and MCP use the same resolved-view rules.
- Search and filters apply after resolution by default and over the raw historical
  set only when explicitly requested.
