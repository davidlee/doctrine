# Scaffold a new freestanding skill plugin

A skill ships as a plugin under `plugins/<name>/`. To add one freestanding
(its own plugin, like `handover`), three files:

1. `plugins/<name>/.claude-plugin/plugin.json` — manifest:
   ```json
   { "name": "<name>", "version": "0.1.0", "description": "<one line>" }
   ```
2. `plugins/<name>/skills/<name>/SKILL.md` — the skill. YAML frontmatter
   `name:` + `description:` then the body. The `description` is the
   auto-trigger surface — write it as the real trigger, not a placeholder
   (see [[mem.pattern.skill.description-is-the-trigger]]).
3. Register in the root marketplace: append a `{name, source, description}`
   entry to the `plugins` array of `.claude-plugin/marketplace.json`
   (`"source": "./plugins/<name>"`).

Validate the two JSON files (`python3 -m json.tool <f>`).

Source-of-truth is `plugins/` only. NOT installed/embedded — a shipped skill is
invisible until installed + re-embedded
(see [[mem.pattern.distribution.skills-source-vs-installed]],
[[mem.pattern.distribution.skill-refresh-command]]).
