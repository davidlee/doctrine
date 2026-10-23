# DEC-404: Never bake a binary inside the installed project

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Amended (codex review, 2026-10-23)

User assent: "yes, agreed" to the SL-286 codex-review dispositions.

F2: comparison pinned to canonicalised paths and whole components. The accepted gap stays: a dev build of a *different* checkout passes the guard and is baked; the report names the baked path so it is visible. DEC-410 narrowed to this testable rule.

## Amended (RV-427 F-8, 2026-10-24)

The project-root prefix check missed the routine case: `install` in a fork under `<coord>/.worktrees/` run with the coord's `./target/debug/doctrine` (outside the fork root) baked a dev build machine-wide. "Any git checkout" was rejected because the user's home is itself a git checkout (`~/.git`), which holds the installed binary. The same-repository rule catches the main checkout and every worktree without encoding cargo layout (POL-002). The accepted gap stays: a build from an unrelated clone passes, and the report prints its path. User reply: "F-8: my ~/.codex is inside a (sparse) git checkout".

## Amended (RV-427 F-14/F-15, 2026-10-24)

Re-review found the plain common-directory comparison refused the installed binary for any project under a git-managed home that is not its own repo (both resolve to `~/.git`), and left the non-git project case undefined. The repository check now applies only when the project root is a checkout or worktree top; the inside-the-project check remains for every project.

## Amended (RV-427 F-17/F-18, 2026-10-24)

The toplevel comparison uses the canonicalised project root (F-17). A same-repository build installing a project that is a subdirectory of that repository passes, as a second accepted gap (F-18, tolerated): only doctrine's own repository builds doctrine, its doctrine project is its checkout top, and the consent prompt shows the binary path before any write.

## Replaced (RV-427 F-18, 2026-10-24)

User ruling: "path rule". Path-shaped guards each produced a false refusal or a gap (see alternatives). Identity with the PATH `doctrine` tests the intent directly ("is this the doctrine the user installed?"), needs no git, cargo-layout or `.doctrine/` knowledge (POL-002), closes the earlier accepted gaps, and baking the PATH entry rather than its target keeps a nix profile path stable across upgrades (RSK-234 mitigation for this entry). Cost: a user with no `doctrine` on PATH always gets the printed command instead of a write — fail safe.