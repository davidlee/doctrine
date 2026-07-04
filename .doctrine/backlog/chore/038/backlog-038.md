# CHR-038: Sweep stale 'doctrine claude install' command refs

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Found during a `/doctrine:dreaming` pass (2026-07-04): `doctrine claude
install` no longer exists as a subcommand — `--help` shows only `doctrine
install` (flat namespace; `-a/--agent`, `-s/--skill`, `--only-memory` etc.
cover what the old `claude install` did). The rename isn't reflected
everywhere it's cited:

- `.doctrine/state/boot.md:383` (source: `mem.signpost.project.orientation`
  body, `mem_019ef1ae52c27ac2867b91db044f62a1`) — "then `doctrine claude
  install` to refresh installed skills"
- At least 14 memory files under `.doctrine/memory/items/` reference `claude
  install` (found via `grep -rl "claude install" .doctrine/memory memory`) —
  not all individually verified; some may already say `doctrine install`
  correctly, or use `claude install` in an unrelated sense. Needs a per-hit
  read before editing.

Fix: grep, confirm each hit is actually the stale command form, `memory edit`
+ md-body fix each, re-verify anchors, `doctrine memory sync -y` if any hit is
a shipped record.