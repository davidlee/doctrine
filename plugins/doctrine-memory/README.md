# doctrine-memory

The two memory skills from the `doctrine` plugin — `record-memory` and
`retrieve-memory` — packaged on their own, for projects that want
doctrine's scope-anchored memory layer without the full process skill set.

Both skills drive the `doctrine memory …` CLI (`record` / `find` / `retrieve` /
`verify`). The CLI is the dependency; this plugin only ships the skills.

## Install one, not both

In this repository the subset's skills are **symlinks** into the canonical
source — there is one source of truth, so they cannot drift:

- **Canonical source:** `plugins/doctrine/skills/{record-memory,retrieve-memory}/`
- `plugins/doctrine-memory/skills/<id>` → `../../doctrine/skills/<id>` (symlink)

Claude Code plugins are self-contained at distribution time, so each symlink is
followed and ships as a real copy in the published artifact. That means a
consumer who installs **both** `doctrine` and `doctrine-memory` gets two skills
of each name, invokable only by their namespaced form. Install `doctrine` **or**
`doctrine-memory`, not both.
