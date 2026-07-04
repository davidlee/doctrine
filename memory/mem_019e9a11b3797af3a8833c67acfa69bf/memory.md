# Doctrine overview

Retrieve this single memory after boot for a complete orientation map.
Doctrine governs intentional change in a repo through four pillars backed by a
thin Rust shell (the `doctrine` CLI).

## Pillars

1. **Slice lifecycle** — intentional change: create a slice, design it, plan it,
   execute phases, audit, reconcile, close.
   See [[mem.signpost.doctrine.lifecycle-start]].
2. **Governance** — ADRs, policies, standards, and revisions record *why* and
   *what rules*. See [[mem.signpost.doctrine.adrs]],
   [[mem.signpost.doctrine.policies-standards]].
3. **Memory** — durable agent knowledge: record, find, retrieve, show memories.
   See [[mem.concept.doctrine.memory-model]].
4. **Entity engine** — TOML+MD tiers, relations, read via `doctrine <kind> show <ID>`.
   See [[mem.concept.doctrine.entity-engine]].

## Mental model

- **Two-tier storage**: structured data in `.toml`, prose in `.md`. Read via
  `doctrine <kind> show`, never raw files. See
  [[mem.concept.doctrine.reading-entities]].
- **Three tiers**: authored (committed, diffable), runtime state (gitignored,
  `rm -rf`-able), derived (regenerated). See [[mem.fact.doctrine.storage-tiers]].
- **The CLI is the source of truth**: `doctrine --help` for command shapes,
  never guess. See [[mem.fact.doctrine.cli-source-of-truth]].
- **Storage rule**: never write queried/derived data in prose; progress in
  runtime state, not authored files. See [[mem.concept.doctrine.storage-model]].

## When to retrieve what

| When you need to... | Retrieve... |
|---|---|
| Start the core workflow | `mem.signpost.doctrine.lifecycle-start` |
| Choose the right skill for a task | `mem.signpost.doctrine.skill-map` |
| Understand where files live | `mem.signpost.doctrine.file-map` |
| Understand the routing gate | `mem.concept.doctrine.routing-gate` |
| Understand the entity engine | `mem.concept.doctrine.entity-engine` |
| Understand the memory model | `mem.concept.doctrine.memory-model` |
| Understand the storage model | `mem.concept.doctrine.storage-model` |
| Understand storage tiers | `mem.fact.doctrine.storage-tiers` |
| Understand the boot snapshot | `mem.concept.doctrine.boot-snapshot` |
| Read entities correctly | `mem.concept.doctrine.reading-entities` |
| Follow conventions | `mem.pattern.doctrine.conventions` |
| Follow the core loop | `mem.pattern.doctrine.core-loop` |
| Follow TDD practice | `mem.pattern.doctrine.tdd-loop` |
| Use the CLI as source of truth | `mem.fact.doctrine.cli-source-of-truth` |
| Record a durable memory | `mem.signpost.doctrine.recording-memories` |
| Author an ADR | `mem.signpost.doctrine.adrs` |
| Work with policies and standards | `mem.signpost.doctrine.policies-standards` |
| Create a backlog item | `mem.signpost.doctrine.backlog` |
| Test backlog membership rules | `mem.concept.backlog.work-intake-membership` |
| Create an RFC | `mem.signpost.doctrine.rfc` |
| Author a spec | `mem.signpost.doctrine.specs` |
| Manage requirements and reconciliation | `mem.signpost.doctrine.requirements` |
| Create a revision | `mem.signpost.doctrine.revisions` |
| Relate entities to each other | `mem.signpost.doctrine.relating-entities` |
| Run an audit phase | `mem.signpost.doctrine.audit` |
| Conduct an adversarial review | `mem.signpost.doctrine.review` |
| Work with reconciliation records | `mem.signpost.doctrine.rec` |
| Capture knowledge records | `mem.signpost.doctrine.knowledge` |
| Create a concept map | `mem.signpost.doctrine.concept-map` |
| Read the reference docs | `mem.signpost.doctrine.reference-docs` |
| Install doctrine in a project | `mem.signpost.doctrine.install` |

## Conventions

- Cite entities by durable prefixed id (e.g. `SL-NNN`, `ADR-NNN`, `REQ-NNN`) —
  never by mobile labels (FR-/NF-).
- Read via `doctrine <kind> show <ID>`, never raw TOML/MD files.
- The CLI is the source of truth: `doctrine --help`, never guess.
- Storage rule: authored (committed), runtime (gitignored), derived (regenerated).
- Commit often with conventional commits; lint as you go.
- See [[mem.pattern.doctrine.conventions]] for full detail.

## Quick-links

Start here:
- [[mem.signpost.doctrine.file-map]] — where everything lives
- [[mem.signpost.doctrine.skill-map]] — which skill for which task
- [[mem.signpost.doctrine.lifecycle-start]] — the core workflow
- [[mem.signpost.doctrine.reference-docs]] — glossary and usage guide
- [[mem.signpost.doctrine.install]] — install doctrine
