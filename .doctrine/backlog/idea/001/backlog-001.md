# IDE-001: Ship a scaffold-skill skill + user-owned skills install repo config

Scaffolding a new skill is a thing a *user* might want to do to customise their
doctrine install — not just a build-repo maintainer task. Today the recipe is
captured only as memory [[mem.pattern.distribution.scaffold-new-skill]]
(three files: `plugin.json`, `skills/<name>/SKILL.md`, marketplace entry).

Idea: promote that recipe into a shipped skill (e.g. `/scaffold-skill` or
`doctrine skill new`) so a user can author a custom skill against their own
install without reading internals.

## The catch — config surface gap

A user-authored skill cannot live in *our* canonical `plugins/` source tree.
For the feature to be real, the user needs to point the skills install at
**their own** repo / marketplace, not doctrine's canonical npm one. We have no
config surface for that today (no "skills source override"). So this likely
splits:

- a config/install improvement (point install at a user-owned skills repo /
  marketplace) — the actual blocker; and
- the scaffold skill itself, which is cheap once the source is relocatable.

Until the config surface exists, scaffolding only lands skills into doctrine's
own tree (the memory recipe), which is a maintainer flow, not a user flow.

## Pointers

- [[mem.pattern.distribution.scaffold-new-skill]] — the manual recipe.
- [[mem.pattern.distribution.skills-source-vs-installed]] — source vs installed copy.
- [[mem.pattern.distribution.skill-refresh-command]] — re-embed after install.
