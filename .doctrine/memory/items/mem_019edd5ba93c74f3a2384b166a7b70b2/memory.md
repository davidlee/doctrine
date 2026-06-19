# YAML frontmatter colons break skill SKILL.md parsing

Colons in YAML frontmatter description fields cause parse errors in skill discovery (14 tests fail). Use YAML block scalar (>-) or quote the value.
