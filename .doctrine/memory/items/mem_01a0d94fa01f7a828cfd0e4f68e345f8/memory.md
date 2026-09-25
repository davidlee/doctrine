Refreshing a `plugins/**` skill master:
`cargo build` re-embeds on its own — **no `touch src/install.rs` needed** — then
`doctrine install -s <id> -y` materialises `.doctrine/skills/<id>/SKILL.md`.

Probed SL-267 PHASE-03 (2026-09-26): edited `plugins/doctrine/skills/elicit/SKILL.md`,
ran a plain `cargo build` + `install -a claude -s elicit -y`, and the installed copy
carried the edit. `diff -r` of a pre-phase snapshot against the live
`.doctrine/skills/` returned exactly the 14 edited skills — nothing stale, nothing
extra.

This **narrows two earlier memories**, neither of which is wrong about what it
covered:
- `mem.pattern.build.rust-embed-no-rerun` proved auto-rebuild for the
  `install/templates` root and explicitly listed `memory/`+`plugins/` as **not
  re-probed**. They are now probed: `plugins/` (this memory) and `memory/`
  (which re-materialises through `doctrine memory sync`, a separate path from the
  cargo embed — see `mem.pattern.doctrine.shipped-master-body-scrub`).
- `mem_019eae55811f7412b11559068fe8a279` prescribes `touch src/install.rs` as step
  1 of the refresh loop. Read that as belt-and-suspenders, not a precondition.

The touch remains harmless, so keep it if you like. What is *not* optional is the
final step: `install -s <id>`. The re-embed makes the **binary** current; only
`install` makes the **installed tree** current. And never trust `Finished` —
verify through the render (`diff` the installed copy, or
`grep -a -c '<string>' target/debug/doctrine`).
