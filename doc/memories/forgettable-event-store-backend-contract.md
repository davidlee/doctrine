# Memory: forgettable is the event-store backend (DEC-005-C constraint)

The deferred `event-store` backend ([memory-spec § Backend abstraction](../memory-spec.md))
is **forgettable**. It accepted the client role in forgettable `ADR-005`
(*generic append-only event substrate; clients ride opaque payloads*, dated
2026-06-04). Recorded here because the one genuinely **new obligation** it adds is
easy to lose before the adapter slice exists (no adapter slice yet; this is
downstream of SL-005's deferred ledger seam).

## The hard constraint — DEC-005-C

The doctrine adapter **MUST NOT depend on forgettable's first-party `/memory/*`
domain**. Every memory and every lifecycle event is written and rebuilt through
the **generic event store** (streams / events / append / read), so lifecycle
fidelity never relies on forgettable's own git-scoped memory feature. Calling
`/memory/*` is an optional side-channel for explicit product interop only — never
the integration path, never required. Two memory models, one substrate, separate
callers — not a shared schema.

## Stream shape forgettable binds (for the adapter slice)

One append-only stream per memory, created on append by
`(workspace_id, stream_type, natural_key)`:

- `stream_type = "doctrine.memory"`
- `natural_key  = memory_uid`   (the v7 uid SL-005 mints; § Identity)
- `source_uri   = "doctrine://memory/v1"`   (already in memory-spec § Identity)
- `event_id`    = client-minted deterministic `uuid5` (the deferred ledger seam)
- lifecycle ledger = the stream's event sequence, **folded client-side** into the
  same projection the native `local-text` backend reads as files (conformance
  contract: both fold the same sequence → same projection).

## Read surfaces forgettable commits to — DEC-005-D

forgettable un-defers its "global commit-safe cursor" against doctrine as first
caller, providing what rebuild/export needs:

- **stream catalog** — discovery; enumerate streams in authorized scope,
  filterable by `stream_type` / `source_uri`, opaque cursor. NOT ordering or
  completeness.
- **workspace event feed** — authoritative rebuild/export; ordered resumable read
  over the **committed safe prefix**, same authz boundary as per-stream reads,
  `stream_type` / `source_uri` filterable, opaque cursor only (never `row_id`,
  hash-chain, checkpoints, authz internals). Storage-order safe prefix, not causal
  truth.

## No impact on SL-005

ADR-005 lives entirely in SL-005's deferred ledger/backend seam. It **confirms**
SL-005's identity calls (uid v7 = `natural_key`; `event_id` uuid5 deferred;
`workspace` carried always; opaque payloads) rather than changing them. See
[engine-identity-and-claim-seam](engine-identity-and-claim-seam.md) for the SL-005
engine work itself.

## Locating forgettable

- agent jails (most agents): `/workspace/forgettable`
- the ADR: `<forgettable>/.spec-driver/decisions/ADR-005-*.md`
