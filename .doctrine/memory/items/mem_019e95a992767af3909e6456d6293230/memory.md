# User-customizable governance surface (the `doctrine.md` analog)

**Scoped into SL-011** (cache-friendly session boot context) — folded in as a boot
input + `/canon` reference. Retire/realize there. Surfaced while porting the `/canon`
skill from spec-driver's `doctrine` skill.

## The idea

spec-driver had a single governance hook — `.spec-driver/hooks/doctrine.md` — a
user-editable file the `doctrine`/`/canon` skill loaded to learn project-local
"articles of truth". Doctrine has no equivalent: `/canon` currently points at a fixed
list (`CLAUDE.md`, `AGENTS.md`, `.doctrine/adr/`, `doc/*`, the slice `design.md`).

Want a customizable surface that:

- is **user-editable** — projects add their own conventions / posture / pointers;
- **survives `doctrine install`** — never overwritten from the embed (unlike the
  derived/gitignored `.doctrine/skills` canonical tree);
- **loads at defined workflow points** — at minimum `/canon`, plausibly also
  `/route`, `/preflight`, `/execute` (mirrors spec-driver's load-on-route).

## Open questions

- Location + tier. Authored (committed, user owns) vs a seeded-once file install
  refuses to clobber (like the SL-010 override hatch). Probably authored under
  `.doctrine/` with install seeding a template only when absent.
- One file vs per-point files (a `canon.md` vs `route.md`/`execute.md` hooks).
- How `/canon` references it without an `@` force-load (CSO: avoid force-loading).
- Relationship to ADRs/`doc/*` — pointer layer, not a competing truth.
