# CHR-045: Bump doctrine plugin.json version when the skill set changes

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Original context (SL-214 audit, RV-280) — historical

`plugins/doctrine/.claude-plugin/plugin.json` was `0.1.0` while skills had been
added (e.g. `knowledge`, SL-214). The Claude marketplace cache
(`~/.claude/plugins/cache/doctrine/doctrine/0.1.0/`) keys on that version and so
did not pick up the new skill — `/knowledge` was routed in boot but not
invocable in a Claude session until the plugin cache refreshed.

## What is already fixed

`just sync-plugin-versions`, invoked inside `just release`, derives all five
plugin manifests (`plugins/marketplace.json`, `.claude-plugin/marketplace.json`,
and the three `plugins/*/.claude-plugin/plugin.json`) from the Cargo version. The
manual drift this item was filed against is structurally gone: CHR-048's release
propagated 0.31.0 → 0.31.1 across all five with no manual step.

## Residual — tagged YAGNI

The coupling is to **releases**, not to the **skill set**. Add or edit a skill
without cutting a release and the cache key never moves, so harness caches keep
serving the old prose. The card's original parenthetical — "consider a
release-checklist or lint hook" — is the undone part: nothing forces a bump when
`plugins/*/skills/**` changes.

Deliberately **not** building that gate (decision at CHR-048 close, 2026-07-25).
Releases are frequent enough that the window is short, and a lint hook coupling
skill edits to a version bump would fire on every in-progress skill edit. Revisit
only if a stale-cache incident actually recurs — CHR-048 was a missed *push*, not
a missed bump, so it is not evidence for this gate.
