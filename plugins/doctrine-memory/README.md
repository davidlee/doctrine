# doctrine-memory

The two memory skills from the `doctrine` plugin — `record-memory` and
`retrieve-memory` — packaged on their own, for projects that want
doctrine's scope-anchored memory layer without the full process skill set.

Both skills drive the `doctrine memory …` CLI (`record` / `find` / `retrieve` /
`verify`). The CLI is the dependency; this plugin only ships the skills.

## Install one, not both

These skill files are **duplicated** from `plugins/doctrine/skills/` (Claude Code
plugins are self-contained; there is no shared-source mechanism that survives
distribution). The copies are byte-identical to the canonical originals.

- **Canonical source:** `plugins/doctrine/skills/{record-memory,retrieve-memory}/`
- Install `doctrine` **or** `doctrine-memory`, not both — installing both yields
  two skills of the same name, invokable only by their namespaced form.
- If you edit a memory skill, update both copies (or re-copy from canonical) so
  they do not drift.
