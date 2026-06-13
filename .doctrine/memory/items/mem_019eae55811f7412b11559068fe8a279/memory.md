# Skill content refresh = doctrine claude install + touch src/skills.rs to re-embed

> **SL-056 PHASE-11 rename:** `doctrine skills install` → **`doctrine claude
> install`** (the renamed primary — it also installs the dispatch-worker agent def
> + the SubagentStart hook). `skills install` survives as a **hidden deprecated
> alias** dispatching the identical handler, so old invocations still work — prefer
> `claude install` going forward.

After editing a `plugins/<domain>/skills/<id>/SKILL.md` (or a sibling like
`NOTICE.md`), getting that change into the in-session installed copy is a
**two-gotcha** sequence:

- **`doctrine install` does NOT refresh skill content.** It only ensures the
  `.doctrine/skills/` *dir* exists; every skill prints `skip … (exists)`. The verb
  that rewrites content and (re)links `.claude/skills/<id> → ../../.doctrine/skills/<id>`
  is `doctrine claude install -y` (alias `skills install`; filter with `-s <id>` / `-d <domain>`).
- **A lone `plugins/` edit does NOT re-embed on `cargo build`** — RustEmbed only
  re-reads when the embedding crate (`src/skills.rs`, `#[folder = "plugins/"]`)
  recompiles. A plain `cargo build` finishes in <1s as a no-op and the stale bytes
  ship. See [[mem.pattern.build.rust-embed-no-rerun]] / [[mem.pattern.embed.rustembed-recompile-and-symlinks]].

Working sequence:

```bash
touch src/skills.rs            # force the embedding crate to recompile
cargo build                    # now re-embeds the edited plugins/ files
./target/debug/doctrine claude install -y   # refresh .doctrine/skills/* + relink .claude/skills/* (alias: skills install)
```

Sibling files (e.g. `NOTICE.md`) ride the dir-copy automatically — `discover()`
collects every file under a skill dir. Author under `plugins/`, never the
gitignored `.doctrine/skills/` copy ([[mem.pattern.distribution.skills-source-vs-installed]]).
