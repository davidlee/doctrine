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

**Where the refreshed copy lands changed twice — SL-227, then back at SL-250.**
SL-227 (minimal projection, ADR-019) stopped `install` projecting a local skills
mirror for claude, leaving the plugin cache
(`~/.claude/plugins/cache/doctrine/doctrine/<version>/skills/`) as the
harness-visible copy, refreshed only by a release tag + `claude plugin update`.

**SL-250 retired that channel and restored the direct write.** For claude,
`install` now materialises a canonical `.doctrine/skills/<id>` tree
(`skills_canonical_dir`, `src/install.rs`) and reconciles a relative
`.claude/skills/<id>` symlink into it by proven ownership
(`install_skills_direct` → `reconcile_link`) — SPEC-010 responsibilities 3–6.
So `install -s <id>` **does** move the slash-invocable skill again, and no
plugin update is involved. Other harnesses still delegate to `npx skills add`.
The `plugins/` tree remains the canonical source for every channel, and the
published plugin manifest survives only as the managed-policy escape hatch.

Consequence for the loop above: after editing a master, the re-embed makes the
*binary* current and the `install -s` that follows makes the session current.

Sibling files (e.g. `NOTICE.md`) still ride the dir grouping — `discover()`
collects every file under a skill dir. Author under `plugins/`, never a derived
installed copy ([[mem.pattern.distribution.skills-source-vs-installed]]).
