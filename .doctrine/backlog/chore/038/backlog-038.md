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

## Resolution (2026-07-04)

Per-hit read of every `claude install` grep match. True positives fixed
(local, no rebuild needed): `mem.signpost.project.orientation`,
`mem.signpost.doctrine.skill-masters` (toml `commands` scope), a routing-row
pattern, a jail-binary pattern, shipped-memory-authoring, and
shipped-skill-platform-independence — 7 items total. `boot.md:383` fixed
itself via `doctrine boot` regen off the corrected orientation body.

Left untouched (false positives / out of scope):
- `mem.pattern.distribution.skill-refresh-command` — already documents the
  SL-088 rename correctly; the phrase appears only in its own historical
  changelog note.
- `mem.pattern.distribution.hookspec-merge-core-generalized-event-matcher` —
  `claude install` there names the SL-056 feature that wired
  `install_claude_hook`, not a runnable command.
- `mem.pattern.distribution.skill-frontmatter-yaml-no-colon-no-quote` — status
  `archived` + `superseded_by`; left as a frozen historical record.

Widened scope while touching linked shipped signposts (user-approved,
2026-07-04): found `mem.signpost.doctrine.skill-map` and
`mem.signpost.doctrine.file-map` (both shipped via `memory/`, sent to every
client) carrying doctrine's-own-dev-repo paths as if universal — a POL-002
violation, same class as `mem.pattern.doctrine.shipped-skill-platform-independence`.
skill-map's scope pointed at `plugins/doctrine/skills/` (only exists in this
repo); file-map's body claimed internals live in `doc/*` (repo-local, and
even the repo's own dir is `docs/` not `doc/`) and named `src/git.rs` /
`src/boot.rs` as if every client had them. Trimmed rather than renamed.
Rebuilt (`touch src/corpus.rs && cargo build`), `doctrine memory sync -y`
(2 changed), `doctrine boot` regen. Local memories re-verified
(`--allow-dirty`, tree noise from an unrelated `skills-lock.json` drift not
part of this chore); shipped-memory `verify` is a known gap (IMP-148,
resolve_show/verify only look in `items/`).

Commits: `0807fb5b` (content fixes), `5a94bb90` (re-verification stamps).