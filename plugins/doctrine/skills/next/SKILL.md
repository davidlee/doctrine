---
name: next
description: Use when everything is written down and in order for a fresh agent, and the user wants to continue with a new context — print a concise continuation prompt for the next agent.
---

# Next

Print:

```
/route
(path to the governing slice and the next obvious task)
```

If the next task is implementation-bound, include the specific artefacts to read
(`design.md`, `plan.toml`, the active runtime phase sheet) and name any unresolved
assumptions or design questions the next agent must assess before declaring
readiness. Have the continuation prompt **cite the open ids** from the governing
slice's `## Harvest` — the open DEC / QUE / ASM (decisions / questions /
assumptions) — so the next agent inherits the live open items, not a stale story.

Before printing, confirm the governing slice's `## Harvest` is fresh: its
`fresh-as-of` stamp matches the current lifecycle position (phase / stage + head
commit). If stale, run `/notes` first to bring it current — do not re-survey.
Phase status accurate and work committed (or its uncommitted state noted) remain
adjacent truth checks.
