# Backlog slug aliases are tracked symlinks — trailing-slash rm follows them

Every `.doctrine/backlog/<kind>/NNN-<slug>` entry is a **tracked symlink** →
the bare numeric data dir `NNN/` (where `backlog-NNN.{toml,md}` live). Same
convention noted for entity dirs by mem_019ebaa76fa4707383006befdd9580a7
(corpus walks must skip the alias).

**Footgun:** `rm -rf .doctrine/backlog/<kind>/NNN-<slug>/` — *with a trailing
slash* — dereferences the symlink and deletes the **target dir's contents**
(`backlog-NNN.{toml,md}`), not the link. Observed CHR-015 (2026-06-19): wiped
the tracked data files; recovered via `git restore`.

**Apply:** never `rm` a slug alias with a trailing slash. To remove an alias,
operate on the link itself (`rm <path>`, no slash). To inspect the data, target
`NNN/`. Treat the alias as a pointer, never a directory.
