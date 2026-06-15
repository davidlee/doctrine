# Doctrine CLI command map

The top-level verb surface — a navigational signpost, NOT a flag reference. For
exact shapes, flags, ids, and subcommands, ask the CLI: `doctrine <verb>
--help`. Never guess. See [[fact.doctrine.cli-source-of-truth]].

`doctrine` verbs:

- **install** — install doctrine files into a project. See [[signpost.doctrine.install]].
- **claude** — manage the agent harness surface (skills, agents, hooks).
- **slice** — create/list slices, the unit of intentional change:
  `new · design · plan · phases · notes · phase · list · status`. Drives the
  lifecycle in [[signpost.doctrine.lifecycle-start]].
- **memory** — record and query durable memory:
  `record · show · verify · list · find · retrieve · sync`. The shipped global
  corpus arrives via `memory sync`. See [[concept.doctrine.memory-model]] and
  [[signpost.doctrine.recording-memories]].
- **review** — the adversarial review ledger (RV kind, ADR-007):
  `new · raise · dispose · verify · contest · withdraw · prime · show · list`.
  See [[signpost.doctrine.audit]].
- **rec** — reconciliation records (REC kind): `new · show · list`.
- **revision** — the change-axis for governance (REV kind, ADR-013):
  `new · show · list · status`. See [[signpost.doctrine.revisions]].
- **reconcile** — reconcile a requirement against observed coverage. The sole
  author of reconciled status.
- **coverage** — requirement coverage: `show` (drift view), `record`/`verify`/
  `forget` (observed-tier write). See [[signpost.doctrine.requirements]].
- **inspect** — read-only cross-kind relation view of one entity (authored
  outbound + derived inbound). See [[signpost.doctrine.relating-entities]].
- **survey** — cross-kind importance survey: every eligible entity in importance
  order. Advisory, never writes.
- **next** — advisory worklist: actionable entities (eligible + unblocked), in
  dependency/sequence order. Mutates nothing.
- **blockers** — blocker view of one entity: its blocked-by prerequisites and
  items it blocks. `--transitive` walks both chains.
- **explain** — structured explanation of one entity's priority: eligibility
  reason, transitive blocker chain, order-key contributors, consequence.
- **adr** — architecture decision records: `new · list · show · status`.
  See [[signpost.doctrine.adrs]].
- **policy** — governance policies (standing rules): `new · list · show · status`.
  See [[signpost.doctrine.policies-standards]].
- **standard** — governance standards (conventions of practice):
  `new · list · show · status`. See [[signpost.doctrine.policies-standards]].
- **spec** — product / technical specifications:
  `new · list · show · req`. See [[signpost.doctrine.specs]].
- **backlog** — capture and survey work-intake items (issue · improvement ·
  chore · risk · idea): `new · list · show · edit`. See [[signpost.doctrine.backlog]].
- **knowledge** — durable knowledge records (assumption · decision · question ·
  constraint): `new · list · show · status`.
- **boot** — regenerate the governance snapshot (`.doctrine/state/boot.md`);
  `boot install` wires the `@`-import + session hook; `boot --check` is the disk
  sentry. See [[concept.doctrine.boot-snapshot]].
- **worktree** — isolated forks for `/execute`/`/dispatch`:
  `fork · coordinate · import · land · gc · branch-point-check · verify-worker ·
  marker · provision · stamp · status`.
- **dispatch** — orchestrate phase execution through sub-agent workers:
  `sync · record-boundary · candidate`. Orchestrator-classed.
- **validate** — scan every entity kind for id-integrity violations (dir
  basename == toml id, no intra-kind duplicates, alias target equality).
  Exits non-zero on any violation.
- **reseat** — renumber an entity's canonical id, moving it to the next free
  trunk-aware id or `--to <NNN>`. Reports inbound prose citations as danglers.
- **link** — author a tier-1 `[[relation]]` edge (idempotent).
  See [[signpost.doctrine.relating-entities]].
- **unlink** — remove a tier-1 `[[relation]]` edge (idempotent).
- **needs** — append a hard prerequisite (blocking dependency). Idempotent.
- **after** — append a soft-sequence edge (ordering hint). Idempotent.
- **supersede** — record that a NEW ADR supersedes an OLD one. Idempotent.

These verbs are what the skills wrap ([[signpost.doctrine.skill-map]]); the
files they read and write are mapped in [[signpost.doctrine.file-map]].
