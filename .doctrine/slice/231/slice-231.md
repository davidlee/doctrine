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
  observation time, and concise symptom/detail. Ordinary provenance may remain
  implicit; exceptional attribution is an optional facet.
- Carry structured execution context needed by later analysis: role, harness,
  model, execution mode/arm, lifecycle stage, skill, command or product surface,
  and a session/run correlation id where available.
- Carry optional canonical context references such as slice, phase, backlog
  item, or change identity without requiring those references at capture time.
- Admit optional effort/usage counters only with explicit measurement source,
  scope, units, and completeness through a registered machine-source adapter.
  Agent estimates must not masquerade as measured token telemetry.
- Treat every user- or agent-authored string as hostile input at parse, render,
  agent-context, and MCP boundaries, with bounded record sizes and no silent
  truncation.

### 2. Merge-safe authored store

- Persist each capture independently under collision-free identity; no writer
  appends to a shared corpus file.
- Store one self-contained TOML record at a path derived only from its UUID,
  using a UUID-random-tail shard for bounded directory size.
- Make retries idempotent where the caller supplies an occurrence identity, and
  refuse clobbering a different occurrence.
- Keep raw occurrences authoritative and queried indexes derived. Corrections
  use per-control immutable supersession or retraction, never an in-place
  rewrite or component-wide rollback that obscures provenance.
- Reuse shared repository-root and safe-filesystem primitives while keeping the
  observation lifecycle separate from the numbered authored-entity engine.
- Default authoritative records to committed authored collection data while
  documenting repository-wide and local ignore choices and their durability
  tradeoffs.

### 3. Capture and basic read interface

- Provide CLI verbs to record, show, list/filter, and text-search occurrences.
- Provide a bounded MCP capture tool over the same engine functions for
  confined Claude workers. Trusted reads and correction remain CLI operations;
  subprocess-worker parity is a follow-up.
- Keep public CLI/MCP capture friction-only. Define the measurement schema now,
  but admit measurements only through registered trustworthy machine-source
  adapters.
- Auto-populate context that Doctrine can know reliably; accept explicit fields
  for context the caller alone knows. Render absent/unknown distinctly from an
  inferred value.
- Return machine-readable output suitable for later reporting without making
  any aggregate or prioritisation judgement in this slice.

### 4. Dogfood activation

- Replace RFC-011's shared-file append instruction with capability-aware
  guidance once behaviour is verified: primary-tree CLI, confined-Claude MCP,
  and orchestrator proxy capture where no worker broker exists.
- Preserve the existing case-note file as historical evidence; do not silently
  parse or rewrite it.
- Document the capture boundary: record an occurrence at friction time;
  promote consolidated reusable knowledge to memory and actionable
  consequences to backlog through their existing workflows.

### Affected surface

- New observation model/store under `src/observation/**` plus shared
  complete-content atomic no-clobber filesystem publication.
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
  UUID-sharded local collection contract.

## Risks, assumptions, and open questions

- **R1 — parallel substrate risk.** The collection shape resembles both named
  memory entities and comparison session ledgers. Design must identify and
  reuse their proven seams without inheriting the wrong lifecycle semantics.
- **R2 — capture-friction risk.** A schema rich enough for later analysis can
  make recording too expensive. Required fields must stay minimal; reliably
  auto-populated and optional context must remain distinct.
- **R3 — review-noise risk.** Authored observations may crowd pull-request
  review. Projects may ignore them, but then own the resulting loss of shared
  durability and analysis coverage.
- **A1 — collection-first sequencing.** Basic show/filter/search is sufficient
  to validate useful capture before aggregate interpretation is built.
- **D1 — evergreen home settled.** QUE-174 and DEC-043 place the capability in
  PRD-018 and SPEC-028; memory, comparisons, and a generic ledger abstraction
  remain intentionally separate.

## Summary

Ship a first-class, structured observation ledger whose friction occurrences
are cheap to capture, independent to merge, safe to inspect, and available
through the trusted CLI and a bounded MCP capture interface. Establish the
signal substrate only; leave interpretation to a follow-up.

## Follow-Ups

- Aggregate and cluster observations into recurring causes.
- Report frequency and wasted-effort trends by model, harness, role, stage,
  skill, and execution mode.
- Surface recurring causes without active backlog or slice coverage.
- Measure pre/post impact of fixes using normalized exposure denominators.
- Integrate authoritative harness/API token usage if a complete measurement
  boundary becomes available.
- Detect harness identity from individually named environment markers
  (IDE-005).
- Define compaction, retention, and external archival policy from observed
  corpus growth.
- Broker observation capture for subprocess workers (IMP-319).
- Add default-off `doctrine.toml` activation and conditional boot guidance
  (IMP-320).
