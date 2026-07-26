# Friction observation ledger and capture interface

## Context

RFC-011 currently asks every agent to append incidental friction to one
project-local Markdown file. The practice has produced useful evidence, but the
storage shape has become the limiting factor: observations are unstructured,
concurrent appends conflict, the live file grows without bound, and neither
search nor stable classification is available. The current corpus is already
large enough that agents cannot consume it economically as one document.

These observations are occurrence evidence, not backlog work and not yet
durable reusable knowledge. A raw occurrence may later inform a memory, a
knowledge record, or a backlog item, but storing every occurrence directly in
one of those sinks conflates their lifecycles and makes consolidation harder.

The change introduces a dedicated collection primitive and its capture/read
interface. It deliberately stops before interpreting the corpus: collection
must first become structured, concurrent, and reliable; reporting and
aggregation can then consume a proven wire model in a follow-up.

## Scope & Objectives

### 1. Friction-observation occurrence model

- Define one typed, immutable occurrence carrying collision-free identity,
  observation time, concise symptom/detail, and explicit provenance.
- Carry structured execution context needed by later analysis: role, harness,
  model, execution mode/arm, lifecycle stage, skill, command or product surface,
  and a session/run correlation id where available.
- Carry optional canonical context references such as slice, phase, backlog
  item, or change identity without requiring those references at capture time.
- Admit optional effort/usage counters only with explicit measurement source,
  boundary, and completeness. Agent estimates must not masquerade as measured
  token telemetry.
- Treat every user- or agent-authored string as hostile input at parse, render,
  and MCP boundaries.

### 2. Merge-safe authored store

- Persist each capture independently under collision-free identity; no writer
  appends to a shared corpus file.
- Partition the store so the active write/query surface does not become another
  ever-growing monolith. The design must settle the exact fileset and partition
  contract.
- Make retries idempotent where the caller supplies an occurrence identity, and
  refuse clobbering a different occurrence.
- Keep raw occurrences authoritative and queried indexes derived. Corrections
  use an explicit immutable mechanism such as supersession or tombstoning,
  never an in-place rewrite that obscures provenance.
- Reuse the shared entity/storage and pure/imperative seams rather than
  introducing parallel identity, TOML rendering, or disk-write machinery.

### 3. Capture and basic read interface

- Provide CLI verbs to record, show, list/filter, and text-search occurrences.
- Provide MCP capture and read parity over the same engine functions so agent
  execution mode does not determine whether friction is recorded.
- Auto-populate context that Doctrine can know reliably; accept explicit fields
  for context the caller alone knows. Render absent/unknown distinctly from an
  inferred value.
- Return machine-readable output suitable for later reporting without making
  any aggregate or prioritisation judgement in this slice.

### 4. Dogfood activation

- Replace RFC-011's shared-file append instruction with the new capture
  interface once its behaviour is verified.
- Preserve the existing case-note file as historical evidence; do not silently
  parse or rewrite it.
- Document the capture boundary: record an occurrence at friction time;
  promote consolidated reusable knowledge to memory and actionable
  consequences to backlog through their existing workflows.

### Affected surface

- New occurrence model/store and shared-engine integration under `src/**`.
- CLI and MCP command adapters under their existing command/interface homes.
- Focused unit, CLI, MCP, concurrency, and hostile-input tests under `src/**`
  and `tests/**`.
- Project-local RFC-011 instrumentation wording for dogfood activation.

## Non-Goals

- Frequency, trend, clustering, deduplication, prioritisation, or impact
  reporting.
- Derived backlog-coverage reports or automatic creation of backlog items.
- Automatic consolidation into memory or knowledge records.
- Automatic harness/API token telemetry, cost accounting, or claims that the
  captured counters form a valid benchmark.
- Cross-model efficiency comparison or workload-normalisation policy.
- Migration, parsing, or retrospective classification of the existing
  `case-notes.md` corpus.
- A UI or hosted telemetry service.
- Destructive retention, compaction, or remote archival policy beyond the
  partitioned local collection contract.

## Summary

Ship a first-class, structured friction-observation ledger whose occurrences
are cheap to capture, independent to merge, safe to inspect, and available
through both CLI and MCP interfaces. Establish the evidence substrate only;
leave interpretation to a follow-up.

## Follow-Ups

- Aggregate and cluster observations into recurring causes.
- Report frequency and wasted-effort trends by model, harness, role, stage,
  skill, and execution mode.
- Surface recurring causes without active backlog or slice coverage.
- Measure pre/post impact of fixes using normalized exposure denominators.
- Integrate authoritative harness/API token usage if a complete measurement
  boundary becomes available.
- Define compaction, retention, and external archival policy from observed
  corpus growth.
