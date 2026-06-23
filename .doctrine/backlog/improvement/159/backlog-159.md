# IMP-159: boot-footer.md: inject user-authored footer at end of boot snapshot

A user-authored `.doctrine/boot-footer.md` injected at the end of the boot snapshot,
so agents see instructions (e.g. "retrieve these specific memories on session start")
without editing governance.md or modifying compiled-in assets.

## Motivation

The boot snapshot ends with `## Invoking doctrine` (the exec path) and the Memory
section passively lists signpost keys. There is no seam for project-specific
instructions like "before starting work, `doctrine memory retrieve mem.key.X`".

## Related

- IMP-155 (per-harness + per-model agent instruction injection) — broader scope;
  this is a simpler, project-level seam that could feed into that.
