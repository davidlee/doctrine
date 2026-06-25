# Skill content refresh = doctrine install -s <id> -y + touch src/skills.rs to re-embed

> **SL-088 consolidation (supersedes the SL-056 `claude install` rename):** the
> installer is now ONE verb — **`doctrine install`** (flags `-s <id>` / `-d <domain>`
> / `-g` / `-y`). `doctrine claude install` is **gone** (`error: unrecognized
> subcommand 'claude'`); `doctrine skills` survives only as a hidden deprecated
> alias exposing `skills list`. Use `doctrine install` everywhere. Verified live
> 2026-06-25 (SL-152 PHASE-05).

After editing a `plugins/<domain>/skills/<id>/SKILL.md` (or a sibling like
`NOTICE.md`), getting that change into the in-session installed copy is a
**two-gotcha** sequence:

- **A lone `plugins/` edit does NOT re-embed on `cargo build`** — RustEmbed only
  re-reads when the embedding crate (`src/skills.rs`, `#[folder = "plugins/"]`)
  recompiles. A plain `cargo build` finishes in <1s as a no-op and the stale bytes
  ship. See [[mem.pattern.build.rust-embed-no-rerun]] / [[mem.pattern.embed.rustembed-recompile-and-symlinks]].
- **Run the install from the re-embedded binary, not PATH.** `doctrine install
  -s <id> -y` rewrites `.doctrine/skills/<id>/` content and (re)links
  `.claude/skills/<id>` from the **running binary's** embedded assets — so it must
  be the jail-built binary after the rebuild, never the stale PATH/`./target` copy.
  See [[mem.pattern.build.jail-binary-for-skill-install]].

Working sequence:

```bash
touch src/skills.rs            # force the embedding crate to recompile
cargo build                    # now re-embeds the edited plugins/ files
TARGET_DIR=$(cargo metadata --format-version=1 | jq -r '.target_directory')
$TARGET_DIR/debug/doctrine install -s <id> -y   # refresh .doctrine/skills/* + relink .claude/skills/*
```

Sibling files (e.g. `NOTICE.md`) ride the dir-copy automatically — `discover()`
collects every file under a skill dir. Author under `plugins/`, never the
gitignored `.doctrine/skills/` copy ([[mem.pattern.distribution.skills-source-vs-installed]]).
