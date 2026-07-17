# CHR-045: Bump doctrine plugin.json version when the skill set changes

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Context (SL-214 audit, RV-280)

`plugins/doctrine/.claude-plugin/plugin.json` is still `0.1.0` while skills
have been added (e.g. `knowledge`, SL-214). The Claude marketplace cache
(`~/.claude/plugins/cache/doctrine/doctrine/0.1.0/`) keys on that version and
did not pick up the new skill — `/knowledge` is routed in boot but not
invocable in a Claude session until the plugin cache refreshes. Bump the
plugin version whenever the skill set changes (consider a release-checklist or
lint hook) so caches invalidate.
