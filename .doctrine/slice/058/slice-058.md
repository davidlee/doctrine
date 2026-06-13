# Finish the relation surface: fix stale scaffold templates, migrate their entity fallout, add agent guidance

## Context

SL-048 ("the cut") migrated tier-1 relations to the uniform `[[relation]]` row
idiom and shipped the machinery end to end: `RELATION_RULES` (the legal
`(source, label)` table, single source of truth — SPEC-018, ADR-010), the
`read_block` parser + `tier1_edges` reader, the `append_edge`/`remove_edge`
writer, the `link`/`unlink` CLI verbs, and the `inspect` / `slice show` render.
**All of that works in source** — verified on SL-057/SL-058 with a fresh dev
build.

The "relations are write-only / half-wired" symptoms that originally motivated
this slice were an artifact of the **stale read-only installed
`~/.cargo/bin/doctrine`** in the jail (predates SL-048: no `link` verb, renders
no relations). On that false premise two items were filed and are now closed:

- **ISS-010** (reader empty) → closed `obsolete` — works in source.
- **IMP-048** (link verb unwired) → closed `done` — shipped by SL-048.

The corrected memory: `mem.pattern.relation.authored-rows-tooling-half-wired`.

What is genuinely unfinished: **the scaffold templates were never updated for the
cut.** All three still emit the migrated tier-1 axes as typed `[relationships]`
slots, so every entity scaffolded since SL-048 is born malformed:

- `install/templates/slice.toml` — emits a `[relationships]` table (the whole
  table migrated away for slices). (**ISS-009**, originally filed.)
- `install/templates/adr.toml` — emits `related = []` (governance `related`
  migrated to `[[relation]]`). This gave **ADR-011** its stale key, which tripped
  `e2e_relation_migration_storage` and blocked the binary update; the entity was
  repaired in `138038c`.
- `install/templates/backlog.toml` — emits `slices`/`specs`/`drift` (migrated to
  `[[relation]]` for backlog). Entities `ISS-009/ISS-010/IMP-045..049`,
  `IDE-005` carry the stale keys; latent (their `[relationships]` header's inline
  comment slips past the migration test's exact-match parser, so they are not
  red — but they are malformed).

## Scope & Objectives

Finish the surface by fixing the source of new malformation and cleaning up the
entities already malformed.

1. **Templates** — bring `slice.toml`, `adr.toml`, `backlog.toml` to the
   post-cut shape: migrated tier-1 axes leave the typed `[relationships]` table
   (slice: table removed entirely; adr: drop `related`; backlog: drop
   `slices`/`specs`/`drift`), kept typed axes stay (adr:
   `supersedes`/`superseded_by`/`tags`; backlog: `needs`/`after`/`triggers`).
   Add a `[[relation]]` guidance comment in place. Re-embed (RustEmbed recompile
   footgun — touch the embedding crate).
2. **Entity fallout** — migrate the already-scaffolded malformed entities to the
   correct shape: backlog `ISS-009`/`ISS-010`/`IMP-045..049`/`IDE-005` (drop the
   typed migrated keys; none carry real edges, so no `[[relation]]` rows needed).
   ADR-011 already done.
3. **Guidance (IMP-049)** — agent-facing support: how/when to relate, the legal
   vocabulary pointer (`RELATION_RULES`), the `link`/`unlink` verb, the
   dev-binary-vs-stale-installed trap. Skill / memory / docs — surface TBD at
   design. May split out if it sprawls.
4. **Test hardening (open question)** — the migration-storage test's `view()`
   parser exact-matches `line == "[relationships]"`, so an inline-comment header
   evades it (why the backlog fallout is latent). Decide at design whether to
   harden the parser as part of this slice.

## Non-Goals

- Reshaping the relation model — fixed by ADR-004 / ADR-010. This is conformance
  cleanup, not redesign.
- Touching the read/write/render machinery (`read_block`, `tier1_edges`, `link`,
  `format_show`) — all proven correct.
- Cross-corpus prose-only relation gaps (IMP-016, IMP-035) — separate items.
- Cleaning the leftover `.worktrees/` (false-RED confound) — orthogonal hygiene,
  user-owned.
- SL-057 (formal VT verification) — parked; resumes after this slice.

## Affected Surface

- `install/templates/slice.toml`, `install/templates/adr.toml`,
  `install/templates/backlog.toml` (+ `backlog-risk.toml`? check) — the fix.
- The embedding crate (`src/install.rs` / `src/slice.rs` `asset_text`) — touch
  to re-embed (`mem.pattern.embed.rustembed-recompile-and-symlinks`,
  `mem.pattern.build.rust-embed-no-rerun`).
- `.doctrine/backlog/{issue,improvement,idea}/…` — the malformed entity files.
- `tests/e2e_relation_migration_storage.rs` — possible parser hardening + new
  scaffold-output assertions.
- IMP-049 surface: skill (`plugins/…` per
  `mem.pattern.distribution.skills-source-vs-installed`) / memory / `doc/*`.

## Risks, Assumptions, Open Questions

- **Behaviour-preservation gate**: SL-048 / SL-046 + relation / cordage suites
  stay green unchanged.
- The malformed entities carry only EMPTY migrated keys (no real edges observed)
  — confirm during execution before deleting any key; a populated key would need
  a real `[[relation]]` migration, not a deletion.
- OQ: harden the migration test parser in this slice or file separately?
- OQ: IMP-049 surface — skill vs memory vs `doc/*` (or all three)?
- OQ: does the slice scaffold drop `[relationships]` entirely, or keep an empty
  commented stub? (slice has no kept typed axes — likely full removal.)
- Assumption: the stale installed binary is the user's to reinstall (out-of-jail,
  read-only here).

## Verification / Closure Intent

- `slice new` / `adr new` / `backlog new` scaffold output contains the
  `[[relation]]` idiom and **no** migrated typed key — black-box CLI assertion.
- `e2e_relation_migration_storage` passes over the whole corpus, including the
  previously-latent backlog items (after migration + any parser hardening).
- A freshly scaffolded entity then `link`-ed renders its relation in
  `slice show` / `inspect` (round-trip) — the end-to-end fixture.
- Behaviour-preservation suites green unchanged.
- Agent guidance landed per the IMP-049 disposition.

## Follow-Ups

- SL-057 resumes (`/design`) after this slice.
- Possible IMP-049 split-out (decided at `/design`).
- Leftover-worktree cleanup (separate, user-owned).
