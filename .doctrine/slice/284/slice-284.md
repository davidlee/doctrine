# Embedded skills for non-Claude harnesses

## Context

`doctrine install` gives Claude its skills from the binary's embed: canon copies
under `.doctrine/skills/<id>`, linked into `.claude/skills/`. Every other
registered harness (`pi`, `codex`, `universal` in `install/manifest.toml`'s
`[harness.*]` registry) is routed `skills_via = "universal"`, which shells out to
`npx skills add davidlee/doctrine --agent universal`. That path:

- clones the **published** GitHub repo, not the embed — installed skills lag the
  running binary (see `mem.fact.doctrine.agents-skill-mirror-is-published-source`;
  IMP-351 records a stale cached skill teaching a retired taxonomy);
- makes Node / `npx` / network / GitHub reachability hard install requirements;
- writes a root `skills-lock.json` into the client project.

Separately, an explicit codex `boot install` also writes pi's
`.pi/extensions/doctrine/*` files, because pi rides the codex boot adapter
(ISS-479).

```
today   claude          embed → .doctrine/skills/<id> ← .claude/skills/<id>  (link)
        pi/codex/univ.  npx skills add davidlee/doctrine → .agents/skills/<id> (copy, GitHub)

after   claude          embed → .doctrine/skills/<id> ← .claude/skills/<id>
        pi/codex/univ.  embed → .doctrine/skills/<id> ← .agents/skills/<id>  (link, same canon)
```

**Settled input (user, 2026-10-19 — IMP-406 `OQ-1`):** pi, codex, and most
non-Claude harnesses read `.agents/skills/`. Pi warns when skills are present in
both `.pi/skills/` and `.agents/skills/`, so `.agents/skills/` is the sole
non-Claude target; doctrine must not also link into `.pi/skills/`.

## Scope & Objectives

1. **Embed-sourced `.agents/skills/`.** Route pi, codex and `universal` to the
   existing direct-link mechanism with `skills = ".agents/skills"` (SL-281's
   registry route). Canon copies, links, dead-link pruning and gitignore follow
   from the existing convergence — a new target, not a new mechanism.
2. **Migrate npx-written state.** Existing projects hold *real directories*
   (npx copies) at `.agents/skills/<id>`, which proven-ownership reconcile
   classifies foreign and keeps. Define how install converges them (and the
   stale `skills-lock.json`) without clobbering user-owned skills.
3. **Decide the npx fallback (IMP-406 `OQ-2`).** Keep it for unregistered
   `--agent` names / bespoke layouts, or remove it with `install_config.rs`'s
   repo-slug setting and the `npx`/`bunx` runner resolution.
4. **Settle `skills-lock.json` (IMP-406 `OQ-3`)** — no equivalent record vs. an
   owed one.
5. ~~Fix the hardcoded `agent claude (direct):` header (RV-350 `F-7`).~~
   Already resolved by SL-281's rewrite — `install_skills_direct` no longer
   exists (research.md cross-thread finding 1).
6. **Separate pi's boot-install selection from codex (ISS-479).** A codex-only
   selection leaves pi files untouched; pi gets its own selection/detection, or
   an explicit, disclosed combined selection. Keep per-file foreign-skip and
   regenerate-on-change.
7. **Spec change through a REV.** SPEC-010 `D2` / `REQ-175` ("Claude direct, all
   others delegated") and its responsibilities line are amended via a Revision,
   not edited in place.

## Non-Goals

- Codex agent defs into `.codex/agents` and the global bake (SL-283, IMP-515,
  ISS-520). Same files; sequence, don't merge.
- Cursor as a harness (IMP-245) — downstream beneficiary only.
- Skill-content governance under taxonomy change (IMP-351's main gap); this
  slice removes only its publication-lag aggravator.
- Shipped-corpus delivery-freshness gates (IMP-487).
- Claude plugin channel retirement (IMP-400 / SL-250 territory).

## Affected surface

- `install/manifest.toml` — `[harness.*]` registry entries.
- `src/install.rs` — skills routing, `install_skills_direct`, npx delegate
  (`delegate_argv`, runner resolution), install plan output.
- `src/install_ownership.rs` — reconcile / migration of real-dir copies.
- `src/install_config.rs` — repo-slug setting (if npx goes).
- `src/boot.rs` — `install_refresh` harness routing (ISS-479).
- `.doctrine/spec/tech/010/` — via REV only.
- Install docs / skills / memories that teach the npx path.

## Risks & assumptions

- **R1 — foreign real dirs.** Migration that is too eager deletes a user's
  hand-authored skill; too timid leaves stale copies shadowing nothing (a link
  can't be written over a dir). Needs a provable "this was npx's" test —
  `skills-lock.json` membership is the obvious candidate.
- **R2 — pi double-load.** Any leftover `.pi/skills/` doctrine content produces
  pi warnings; confirm doctrine never writes there.
- **R3 — SL-283 collision.** Both slices edit `[harness.codex]` and
  `src/install.rs`; land in sequence.
- **A1** — symlinked skill dirs under `.agents/skills/` are followed by pi and
  codex (the user's confirmation covers the location; link-following is to be
  verified in research).

## Verification intent

- Tests: a pi/codex/universal install with no `npx` on `PATH` and no network
  produces `.agents/skills/<id>` links into canon matching the embed.
- Tests: migration over a fixture holding npx-style copies + `skills-lock.json`;
  foreign user skills survive.
- Tests: codex-only boot install writes no `.pi/` files.
- VH: pi session in a migrated project loads skills with no duplicate warning.
- SPEC-010 REV approved and applied; `doctrine check gate` green.

## Summary

## Follow-Ups
