# Skill content refresh = doctrine install -s <id> -y + touch src/install.rs to re-embed

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
  re-reads when the embedding crate (`src/install.rs`, `#[folder = "plugins/"]`, since IMP-226 removed `src/skills.rs`)
  recompiles. A plain `cargo build` finishes in <1s as a no-op and the stale bytes
  ship. See [[mem.pattern.build.rust-embed-no-rerun]] / [[mem.pattern.embed.rustembed-recompile-and-symlinks]].
- **Run the install from the re-embedded binary, not PATH** — it reads the
  **running binary's** embedded assets. Use the in-tree
  `./target/debug/doctrine` after the rebuild; the PATH copy is stale from the
  last release. (Corrected 2026-07-25: this bullet used to prescribe the
  "jail-built binary … never the stale PATH/`./target` copy" — the in-tree
  target *is* the live one, no redirect since SL-156 / ADR-008 D-B1.)
  See [[mem.pattern.build.jail-binary-for-skill-install]].

Working sequence:

```bash
touch src/install.rs                          # force the embedding crate to recompile
cargo build                                   # now re-embeds the edited plugins/ files
./target/debug/doctrine install -s <id> -y
grep -a -c "<string you added>" target/debug/doctrine   # prove the re-embed took (-a!)
```

**Where the refreshed copy lands changed at SL-227** (minimal projection,
ADR-019). `install` no longer projects a local skills mirror: for claude it
registers the marketplace and installs the plugin (so the harness-visible copy
is `~/.claude/plugins/cache/doctrine/doctrine/<version>/skills/`, refreshed by a
release tag + `claude plugin update` — *not* by `install -s`); other harnesses
delegate to `npx skills add`. There is no `.doctrine/skills/` and nothing
relinks `.claude/skills/<id>`. Consequence: after editing a master, the re-embed
makes the *binary* current, but the slash-invocable skill in your session only
moves on a plugin update.

Sibling files (e.g. `NOTICE.md`) still ride the dir grouping — `discover()`
collects every file under a skill dir. Author under `plugins/`, never a derived
installed copy ([[mem.pattern.distribution.skills-source-vs-installed]] — whose
`.doctrine/skills/` mechanism prose is itself stale post-SL-227).
