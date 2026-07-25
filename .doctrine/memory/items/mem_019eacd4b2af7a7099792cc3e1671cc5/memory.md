# Skill source-of-truth is plugins/; the harness copy is a release away

Skills are **authored** under `plugins/doctrine/skills/<name>/SKILL.md`. That is
the only surface a slice ever edits. Everything downstream is derived.

> **Corrected 2026-07-25 (SL-229 audit, RV-306 F-1).** This memory previously
> said the installed copy is a local `.doctrine/skills/` tree. That mechanism is
> **dead**: SL-227 (minimal projection, ADR-019) removed local skills projection
> entirely — `install --dry-run` emits no skill-file rows, and `.doctrine/skills/`
> does not exist in this repo (the stale `.gitignore` line survives at :38, not
> the :34 this memory used to cite). The headline claim is unchanged; the route
> from master to agent is not.

**Why it still matters, and more than before.** The route is now
`plugins/` master → RustEmbed → **release tag** → `origin/main` (what
`.claude-plugin/marketplace.json` sources) → `claude plugin update` → the
harness cache at `~/.claude/plugins/cache/doctrine/doctrine/<version>/skills/`.
Other harnesses delegate to `npx`. A correct, committed, embedded master is
**still invisible to every agent** until a release carries it. `doctrine install`
cannot close that gap, and neither can `touch src/install.rs && cargo build` —
that re-embeds, which is necessary but no longer sufficient.

**How to apply:**

- Edit `plugins/doctrine/skills/...`. Never hand-edit a cache copy; it is
  overwritten on the next `plugin update` and ships nothing.
- Re-embed after any `plugins/`-only edit: `touch src/install.rs && cargo build`
  (a bare `cargo build` is a silent no-op — [[mem.pattern.build.rust-embed-no-rerun]]).
- **To claim a skill edit is live, check the cache, not the tree.** grep the
  edited string in
  `~/.claude/plugins/cache/doctrine/doctrine/<version>/skills/<name>/SKILL.md`.
  If it is absent, the work is authored but undelivered — that is a release
  obligation, not a code defect.
- Verifying a commit reached the harness: `git merge-base --is-ancestor <sha>
  <tag>` and `git branch -a --contains <sha>` against `origin/main`.

Confirmed during SL-029 design (codex review B1); mechanism corrected at the
SL-229 audit, where PHASE-03's four consumption hooks were found authored,
committed, embedded, gate-green — and absent from every harness (CHR-048).

This is the skills-specific instance of the broader source-vs-installed split:
[[mem.pattern.install.authored-entity-wiring]] (authored entities need manifest +
gitignore-negation wiring) and [[mem.pattern.distribution.shipped-not-reachable]]
(a shipped doc is invisible unless pointed-at). See also
[[mem.pattern.build.jail-binary-for-skill-install]] — run `install`/`boot` from
the freshly built in-tree binary, corrected the same day for a related reason.
