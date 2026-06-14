# Doctrine CLI command map

The top-level verb surface — a navigational signpost, NOT a flag reference. For
exact shapes, flags, ids, and subcommands, ask the CLI: `doctrine <verb>
--help`. Never guess. See [[fact.doctrine.cli-source-of-truth]].

`doctrine` verbs:

- **install** — install doctrine files into a project.
- **skills** — manage agent skills.
- **slice** — create/list slices, the unit of intentional change:
  `new · design · plan · phases · notes · phase · list`. Drives the lifecycle in
  [[signpost.doctrine.lifecycle-start]].
- **memory** — record and query durable memory:
  `record · show · verify · list · find · retrieve · sync`. The shipped global
  corpus arrives via `memory sync`. See [[concept.doctrine.memory-model]].
- **adr** — architecture decision records: `new · list · status`.
- **spec** — product / technical specifications:
  `new · list · show · validate · req`.
- **backlog** — capture and survey work-intake items (issue · improvement ·
  chore · risk · idea): `new · list · show · edit`. The intake surface upstream
  of a slice.
- **boot** — regenerate the governance snapshot (`.doctrine/state/boot.md`);
  `boot install` wires the `@`-import + session hook; `boot --check` is the disk
  sentry.
- **worktree** — isolated forks for `/execute`/`/dispatch`:
  `fork · coordinate · import · land · gc · branch-point-check · verify-worker ·
  marker`. The orchestrator-sole-writer dispatch seam.
- **dispatch** — drive a slice's phases through worktree workers and project the
  result: `sync` (stage-1 `--prepare-review`, stage-2 `--integrate`) ·
  `record-boundary` · `candidate {create · status · admit}`. The candidate verbs
  are the review/repair/land surface over the immutable `review/*`/`phase/*`
  evidence refs (admission pins an immutable OID `/close` integrates).
- **review** — the adversarial review ledger (RV kind, ADR-007):
  `new · raise · dispose · verify · contest · withdraw · prime · show · list`.
  The structured audit substrate `/audit` runs on.

These verbs are what the skills wrap ([[signpost.doctrine.skill-map]]); the
files they read and write are mapped in [[signpost.doctrine.file-map]].
