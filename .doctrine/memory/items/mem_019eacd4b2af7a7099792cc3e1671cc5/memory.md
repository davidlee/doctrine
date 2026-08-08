# Skill source-of-truth is plugins/; the harness copy is a release away

Skills are **authored** under `plugins/doctrine/skills/<name>/SKILL.md`. That is
the only surface a slice ever edits. Everything downstream is derived.

> **Re-corrected 2026-08-08 (SL-250 audit, RV-350 F-2).** The route flipped
> twice. SL-227 (minimal projection, ADR-019) removed local skills projection,
> and a 2026-07-25 correction here (SL-229 audit, RV-306 F-1) recorded the
> `.doctrine/skills/` mechanism as **dead**. **SL-250 restored it.** For claude,
> `install` materialises a canonical `.doctrine/skills/<id>` tree
> (`skills_canonical_dir`) and reconciles a relative `.claude/skills/<id>`
> symlink into it by proven ownership (`install_skills_direct` →
> `reconcile_link`), per SPEC-010 responsibilities 3–6 — the plugin/marketplace
> activation path is retired. The headline claim below is unchanged across all
> three states; only the route is.

**Why it still matters — but the route is short again for claude.** After
SL-250 the claude route is `plugins/` master → RustEmbed → `doctrine install`
→ `.doctrine/skills/<id>` → `.claude/skills/<id>` symlink. So `touch
src/install.rs && cargo build && doctrine install` **does** make a skill edit
live in a new session; no release tag, no `claude plugin update`.

Other harnesses still delegate to `npx skills add <repo>`, and that route is
unchanged and still long: master → release tag → `origin/main` → `npx`. A
correct, committed, embedded master is **still invisible** there until a release
carries it. The published plugin cache
(`~/.claude/plugins/cache/doctrine/doctrine/<version>/skills/`) survives only as
the managed-policy escape hatch, and anyone on it is on the long route too.

**How to apply:**

- Edit `plugins/doctrine/skills/...`. Never hand-edit a derived copy — a
  `.doctrine/skills/<id>` tree is overwritten by the next `install`, a cache
  copy by the next `plugin update`, and neither ships anything.
- Re-embed after any `plugins/`-only edit: `touch src/install.rs && cargo build`
  (a bare `cargo build` is a silent no-op — [[mem.pattern.build.rust-embed-no-rerun]]).
- **To claim a skill edit is live, check the derived copy, not the master** —
  but which derived copy depends on the channel, and after SL-250 claude and
  everyone else differ:
  - **claude (direct, post-SL-250):** re-embed, then `doctrine install`, then
    grep the edited string in `.doctrine/skills/<id>/SKILL.md` and confirm
    `.claude/skills/<id>` is a symlink pointing at it. No release needed.
  - **npx / the published plugin:** grep
    `~/.claude/plugins/cache/doctrine/doctrine/<version>/skills/<name>/SKILL.md`.
    If it is absent, the work is authored but undelivered — a release
    obligation, not a code defect.
- Verifying a commit reached a release-route harness: `git merge-base
  --is-ancestor <sha> <tag>` and `git branch -a --contains <sha>` against
  `origin/main`.

Confirmed during SL-029 design (codex review B1); mechanism corrected at the
SL-229 audit, where PHASE-03's four consumption hooks were found authored,
committed, embedded, gate-green — and absent from every harness (CHR-048).

This is the skills-specific instance of the broader source-vs-installed split:
[[mem.pattern.install.authored-entity-wiring]] (authored entities need manifest +
gitignore-negation wiring) and [[mem.pattern.distribution.shipped-not-reachable]]
(a shipped doc is invisible unless pointed-at). See also
[[mem.pattern.build.jail-binary-for-skill-install]] — run `install`/`boot` from
the freshly built in-tree binary, corrected the same day for a related reason.
