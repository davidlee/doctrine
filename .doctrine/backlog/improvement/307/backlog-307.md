# IMP-307: Lint SKILL.md frontmatter description hazards

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Problem

SKILL.md frontmatter is parsed as YAML (`src/skills.rs` `parse_meta`);
`description:` values are plain unquoted scalars. Three character sequences are
hazards, and none is caught at authoring time:

- `: ` (colon-space) — loud parse error, but only surfaces in
  `install::tests_skills`, downstream of authoring.
- embedded `"` — loud parse error, same discovery lag.
- ` #` (space-hash) — starts a YAML comment: the description is **silently
  truncated**. Legal YAML, tests stay green; damage visible only in the
  rendered skill list.

IMP-306 hit two of these (`: ` in handover, ` ## Harvest` in harvest), costing
a fix cycle each. See `mem.pattern.skills.yaml-frontmatter-colons` for the full
pattern and sweep command.

## Desired shape

An authoring-time gate — a check in `install::tests_skills` (or a
`doctrine check` leg) that walks every embedded `SKILL.md` and rejects `: `,
`"`, and ` #` in the `name`/`description` scalar values, with a message naming
the file and the offending sequence. The silent-truncation variant is the
important one: it is the only one no existing signal catches.
