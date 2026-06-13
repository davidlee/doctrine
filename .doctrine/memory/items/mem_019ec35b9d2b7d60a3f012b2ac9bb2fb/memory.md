# SKILL.md frontmatter description must avoid colon-space and double-quotes (YAML scalar)

SKILL.md frontmatter is parsed as **YAML** (`serde_yaml::from_str`, `src/skills.rs`
`parse_meta`). `description:` / `name:` are **plain YAML scalars**, so two characters
break the embed at `cargo build` time (`Failed to parse SKILL.md frontmatter`):

- **`: ` (colon-space)** inside the value — YAML reads it as a nested mapping. Bit
  `/dispatch-agent` (`subagent_type: dispatch-worker` in the description); bites any
  `key: value` / time-like token.
- **embedded double-quotes** (`"`) — e.g. `env -C "$D"` in `/dispatch-subprocess`.

Fix: reword into prose — "the dispatch-worker subagent type", "via `env -C`".
Backticks, em-dashes, `→`, `↔`, parens, `=` are all fine; only `: ` and `"` are
unsafe in an unquoted scalar. (Quoting the whole value would work too, but no shipped
skill does — keep descriptions colon/quote-free.)

The failure surfaces only when the embedding crate recompiles
([[mem.pattern.distribution.skill-refresh-command]]); a lone `plugins/` edit + plain
`cargo build` is a no-op that ships stale bytes.
