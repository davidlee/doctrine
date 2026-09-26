Each shipped sub-corpus reaches a client through a **delivery step** that no gate
covers, so a skipped delivery is invisible:

- `memory/` masters → `cargo build` (RustEmbed) → `doctrine memory sync` →
  `.doctrine/memory/shipped/`
- `install/` assets → `cargo build` → read through `doctrine library show`
- `plugins/` skills → `cargo build` → `doctrine install -s <id>`

Edit a master, skip the delivery, and the *materialised* copy a client reads keeps
the old bytes while `doctrine doctor`, `doctrine check gate`,
`doctrine publication validate` and the e2e suites stay green — they read the
sources or the embedded assets, never the materialised corpus.

**Check it directly.** After any corpus edit:

    cargo build && doctrine memory sync --dry-run   # expect "0 new, 0 changed, N unchanged"

A nonzero `changed` count on a tree you believe is clean means the materialised
copy is stale (or unedited-but-never-synced). Then run the real
`doctrine memory sync -y` + `doctrine install` and re-read *through the render*
(`doctrine library show <address>`), never the source.

**Evidence.** `SL-267` closed with a stale delivery: PHASE-06's final commit
(`795317bdb`) repaired four shipped masters and never re-synced, so the shipped
copies still carried the pre-repair repo-private paths. Only re-running the sync
(`0 new, 9 changed`) exposed it. Audit `RV-395` `F-13`; the fix is filed as
`IMP-487`. Related: `mem.pattern.build.plugins-embed-auto-rebuild` (the
embed-vs-install split), `mem.pattern.build.rust-embed-no-rerun`.
