# The .doctrine/agents ignore is a whitelist pair (ignore derived, re-include authored AGENTS.md), classified in DERIVED_RUNTIME — ISS-012 fixed

Never blanket-ignore an authored tree. `.doctrine/agents/` holds the **authored,
tracked** `AGENTS.md` alongside *derived* installed agents (`dispatch-worker.md`).
The ignore is a whitelist pair, in order (the `*` exclude before its negation, so
the re-include takes):

```
.doctrine/agents/*
!.doctrine/agents/AGENTS.md
```

and the non-negated `.doctrine/agents/*` glob MUST be registered in the worktree
provision classifier `DERIVED_RUNTIME` (`src/worktree.rs`), or the invariant test
`every_runtime_gitignore_glob_is_classified` (which enumerates every
`.doctrine/`-prefixed, non-negated gitignore glob) goes RED. The `!`-prefixed
negation is skipped by that test (starts with `!`, not `.doctrine/`).

## History (ISS-012, fixed)

Originally `doctrine claude install` appended the bare, too-broad
`.doctrine/agents/*` via `ensure_gitignored` — it swallowed the authored
`AGENTS.md` AND was unclassified, so every re-embed/skill-refresh re-RED'd the
classifier (symptom: `M .gitignore` (+`.doctrine/agents/*`), `just check` fails).
Fixed across both seams:

- `.gitignore` whitelist pair + `DERIVED_RUNTIME += ".doctrine/agents/*"`.
- `src/skills.rs` agents-install leg now emits the **pair** (two ordered
  `ensure_gitignored` calls), so re-install no longer reverts it.

Same lesson the install manifest documents for `memory/items/` — see
[[mem.pattern.install.authored-entity-wiring]] and the install gitignore
classification under [[mem.concept.dispatch.gitignored-tier-partition]].
