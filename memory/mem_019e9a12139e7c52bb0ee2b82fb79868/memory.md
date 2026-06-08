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

These verbs are what the skills wrap ([[signpost.doctrine.skill-map]]); the
files they read and write are mapped in [[signpost.doctrine.file-map]].
