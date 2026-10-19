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
2. **No migration code (DEC-387).** Existing npx copies at a link site are kept
   and disclosed by the existing `Shadows` notice; the maintainer's checkouts get
   a one-off cleanup at close.
3. **Remove the npx delegation (DEC-388)**, with the `[install] repo` setting and
   the `npx`/`bunx` runner; an unknown `--agent` is refused.
4. **`skills-lock.json`:** no equivalent record; doctrine writes none, and the
   stale one is removed in the one-off cleanup.
5. ~~Fix the hardcoded `agent claude (direct):` header (RV-350 `F-7`).~~
   Already resolved by SL-281's rewrite — `install_skills_direct` no longer
   exists (research.md cross-thread finding 1).
6. **Pi boot adapter (ISS-479, DEC-389).** A codex-only selection leaves pi
   files untouched and vice versa. Keep per-file foreign-skip and
   regenerate-on-change.
6a. **Pi extensions out of the client tree (ISS-525, DEC-390):** generated into
   `.doctrine/state/pi/extensions/`, registered via `.pi/settings.json`.
7. **Spec change through a REV.** SPEC-010, SPEC-009 and SPEC-011 are amended via one Revision
   (design sec-6), not edited in place.

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
- `src/install.rs` — skills routing, npx delegate removal, harness refusal.
- `src/install_config.rs` — `[install] repo` removal.
- `src/boot.rs` — pi adapter, `install_refresh` arms, extension dir,
  `.pi/settings.json` merge.
- `README.md` and comments teaching npx.
- `.doctrine/spec/tech/{009,010,011}/` — via REV only.

## Risks & assumptions

- **R1 — foreign real dirs** (retired by DEC-387): copies are kept and disclosed,
  never deleted by product code.
- **R2 — pi double-load.** Leftover `.pi/skills/` or `.pi/extensions/doctrine/`
  content duplicates in pi; doctrine never writes there after this slice, and
  the one-off cleanup removes the maintainer's copies.
- **R3 — SL-283 collision.** Both slices edit `[harness.codex]` and
  `src/install.rs`; SL-284 lands first.
- **R4 — disposable extension dir.** Wiping `.doctrine/state/` removes the pi
  extensions until `doctrine boot install` re-runs (design sec-5).
- **A1** — pi and codex follow symlinked skill dirs under `.agents/skills/`
  (VH at close).

## Verification intent

See design sec-7: VT over routing, shared-dir collapse, per-link gitignore,
`Shadows` retention, unknown-agent refusal, codex/pi boot isolation, and the
`.pi/settings.json` planner; VH for A1 after the one-off cleanup; REV applied;
`doctrine check gate` green.

## Summary

## Follow-Ups
