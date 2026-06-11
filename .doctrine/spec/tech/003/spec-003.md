# SPEC-003: Doctrine

<!-- Reference forms: entity ids padded (SPEC-007, ADR-004); doc-local refs bare
     (D1 decision, OQ-1 open question). See .doctrine/glossary.md § reference forms. -->

## Overview

Doctrine is a single-binary CLI that governs a codebase's durable artefacts —
slices, specs, ADRs, backlog items, memory, and the governance kinds — as
versioned, addressable entities living beside the code they describe. This is the
whole-system **context** spec (C4 level 1): it names the containers Doctrine is
composed of and how they decompose under one system, and holds the principles that
hold across all of them. It is a synthesis, not a description of any one
container's mechanism — each container has its own spec, and this root defers to
them for the *how*.

The system decomposes into these containers:

- **Entity engine** (SPEC-004) — the kind-agnostic scaffolding and identity
  substrate every authored kind is materialised through.
- **Spec composition** — the spec family, its requirement peers, membership edges,
  reassembly, and corpus validation.
- **Memory** — the scope-aware durable-knowledge store, recorded and retrieved
  out-of-band of any one task.
- **Id lifecycle** — next-id allocation, corpus-wide integrity, and reseat repair.
- **Install & distribution** — the embedded sources, manifest, and templates the
  installer lays into a target repo.
- **Skills distribution** — the routing skills shipped from `plugins/` into the
  installed skill tree.
- **Boot snapshot** — the cache-friendly governance projection assembled for
  session start.
- **Dispatch & worktree** — the isolation and orchestrator-sole-writer machinery
  for concurrent work.
- **Priority engine** (SPEC-001) — the derived, explainable "what next" view over
  the entity graph.
- **Reconciliation** (SPEC-002) — the two-tier authored-status-vs-observed-coverage
  machinery and its closure gate.
- **CLI surface** — the uniform command grammar and listing model that fronts every
  container.

The root *contains* these containers by C4 decomposition; it does not peer with
them. Containment (`parent`) and peering (interaction edges) are different
relations, kept distinct so the architecture's shape is not lost.

## Responsibilities

Mirrors the structured `responsibilities` list: name the parts and their
composition; carry the system-wide cross-cutting principles; and state the quality
invariants the whole corpus is checked against. The bar this spec holds is
**altitude** — it names what the containers are and how they fit, never restating
what any one of them does internally.

## Concerns

The cross-cutting principles that hold system-wide (each enforced inside the
containers, named here only as shared discipline):

- **The storage rule — three tiers of truth.** Every artefact is one of three
  kinds: *authored* (committed, diffable TOML for structured data plus Markdown for
  prose — never queried data in prose), *runtime state* (gitignored, disposable
  progress), or *derived* (regenerable indexes and caches). Which tier a thing
  belongs to is decided once and held everywhere.
- **Outbound-only relations (ADR-004).** Relationships are authored on one side
  only; the reverse direction is always derived, never stored twice. No inbound
  field is authored to mirror an outbound edge.
- **The pure/imperative split.** The pure layer takes no clock, RNG, git, or disk —
  those are passed in as inputs (the date/uid injection pattern); impurity lives in
  a thin shell. This is a system-wide principle, enacted by the date/uid seam, not a
  container of its own.

## Hypotheses

- **The container set is the right decomposition.** Doctrine's architecture is
  legible as one context with these containers; capabilities that share a substrate
  (the kinds over the entity engine) are components within their container rather
  than peer containers, so the tree reflects true containment.
- **The three principles are genuinely system-wide.** Storage tiering,
  outbound-only relations, and the pure/imperative split apply to every container,
  which is why they live at the root rather than being restated per container.

## Decisions

- **D1 — the root is a synthesis, never a lift of one subsystem.** This context
  spec names the containers and their composition and holds only the cross-cutting
  principles. Any container's internal mechanism — including the entity engine's —
  is owned by that container's spec and is never restated here. Authored anchor-free
  (no `[[source]]`): a context-level synthesis governs no single code surface
  (REQ-085 admits anchor-free context specs).
- **D2 — every container parents to this root.** The corpus is one tree:
  containment is asserted only where it is true (the root decomposes into
  containers; a container into its components), and peering is left to interaction
  edges. False containment — parenting a peer relation as a `parent` edge — is
  prohibited.
