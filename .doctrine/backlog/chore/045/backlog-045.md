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

## Reassessed at SL-250 close (2026-08-08) — explicitly **retained**

SL-250 (*Retire the Claude plugin delivery channel*) `sec-6` flagged this item as
**no longer moot** and asked close to resolve or explicitly retain it. Retained,
unchanged.

The reasoning SL-250 supplies cuts both ways and nets to zero:

- **Why it is not moot.** SL-250 retires the plugin *delivery path for doctrine's
  own activation only*. `.claude-plugin/marketplace.json` stays published as the
  escape hatch for anyone who prefers the plugin (IMP-400 `OQ-1`, settled by the
  user 2026-08-05), and `plugins/` stays the canonical skill source for every
  channel. A stale published plugin therefore remains a real defect with real
  consumers — it is no longer *doctrine's* activation path, but it is still
  somebody's.
- **Why the residual still does not earn a gate.** SL-250 changes nothing about
  the residual's shape. The coupling is still to releases rather than to the
  skill set, `just sync-plugin-versions` inside `just release` still derives all
  five manifests from the Cargo version, and the undone part is still the
  release-checklist / lint hook the CHR-048 close (2026-07-25) deliberately
  declined to build. No stale-cache incident has recurred since, which was the
  stated revisit trigger.

Net: the item survives its own moot-ness test and fails its build-it test, which
is exactly the state "retain, YAGNI-tagged" describes. Recorded here so the next
reader does not re-derive the question a third time.
