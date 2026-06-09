# Skill source-of-truth is plugins/, not the gitignored .doctrine/skills installed copy

Skills are **authored** under `plugins/doctrine/skills/<name>/SKILL.md` and
**installed** (copied) into `.doctrine/skills/`. The installed tree is
**gitignored** (`.gitignore:34` — `.doctrine/skills/*`), i.e. the derived tier.

**Why:** editing `.doctrine/skills/<name>/SKILL.md` changes only the local
installed copy — it is not tracked, will not ship, and is overwritten on the next
`doctrine install`. The change is silently lost.

**How to apply:** when a slice authors or edits a skill, the affected surface is
`plugins/doctrine/skills/...`, never `.doctrine/skills/...`. Treat
`.doctrine/skills/*` as derived-regenerable (like `memory/{index,embeddings,state}`)
— never copy it into a worktree fork, never hand-edit it. Confirmed during SL-029
design (codex review B1).

This is the skills-specific instance of the broader source-vs-installed split:
[[mem.pattern.install.authored-entity-wiring]] (authored entities need manifest +
gitignore-negation wiring) and [[mem.pattern.distribution.shipped-not-reachable]]
(a shipped doc is invisible unless pointed-at).
