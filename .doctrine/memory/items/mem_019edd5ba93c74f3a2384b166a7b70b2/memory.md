# YAML frontmatter hazards in SKILL.md descriptions: colon-space, quotes, space-hash

SKILL.md frontmatter is parsed as **YAML** (`serde_yaml::from_str`, `src/skills.rs`
`parse_meta`). `description:` / `name:` are **plain unquoted scalars**, so three
character sequences in the value are hazards:

- **`: ` (colon-space)** — YAML reads a nested mapping; hard parse error
  (`Failed to parse SKILL.md frontmatter`, fails `install::tests_skills`).
  Bit `/dispatch-agent` and `/handover` (IMP-306).
- **embedded double-quotes** (`"`) — e.g. `env -C "$D"`; hard parse error.
- **` #` (space-hash)** — starts a YAML **comment**: the rest of the line is
  **silently truncated**. Legal YAML, so tests stay green; the damage shows only
  in the rendered skill list (trigger text cut mid-sentence). Bit `/harvest`
  (` ## Harvest` in the description, IMP-306).

Fix: rephrase (em dash instead of colon), or use a block scalar (`>-`) / quoted
string. Sweep check for the silent variant: scan each description *value* for
`: ` and ` #` — e.g.
`awk '/^description:/{sub(/^description: */,""); print FILENAME": "$0}' plugins/*/skills/*/SKILL.md | grep -E ': | #'`.
