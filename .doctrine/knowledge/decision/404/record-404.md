# DEC-404: Never bake a binary inside the installed project

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Amended (codex review, 2026-10-23)

User assent: "yes, agreed" to the SL-286 codex-review dispositions.

F2: comparison pinned to canonicalised paths and whole components. The accepted gap stays: a dev build of a *different* checkout passes the guard and is baked; the report names the baked path so it is visible. DEC-410 narrowed to this testable rule.

## Amended (RV-427 F-8, 2026-10-24)

The project-root prefix check missed the routine case: `install` in a fork under `<coord>/.worktrees/` run with the coord's `./target/debug/doctrine` (outside the fork root) baked a dev build machine-wide. "Any git checkout" was rejected because the user's home is itself a git checkout (`~/.git`), which holds the installed binary. The same-repository rule catches the main checkout and every worktree without encoding cargo layout (POL-002). The accepted gap stays: a build from an unrelated clone passes, and the report prints its path. User reply: "F-8: my ~/.codex is inside a (sparse) git checkout".