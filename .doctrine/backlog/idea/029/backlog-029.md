# IDE-029: Lifecycle-stage hymn seams for project customisation

## Idea

Give projects a first-class, DRY seam to inject **project-custom lifecycle
guidance** into skills — via the existing **hymns cascade** (ADR-011, SPEC-023),
not a proliferation of bespoke per-skill hook files.

## Origin

Carved out of SL-144 (ADR-005 docs-IA). That slice's scope speculated a new
`install/reconcile-rules.md` — a user-owned doc the reconcile/close loop would
consult for project-custom drift handling. On inspection that would be a
**parallel implementation**: the hymns cascade already has **`stage` and
`project` bands** (preamble/harness/model/role/stage/project) resolved by
`doctrine prompt resolve`. A one-off `reconcile-rules.md` reinvents, per-skill,
what the stage/project bands deliver generally. So SL-144 drops the narrow
deliverable and files this general form instead.

## Shape (not yet designed)

1. **Take stock of the skill set as a whole** — enumerate every lifecycle skill
   (route → slice → design → plan → execute → audit → reconcile → close, plus
   the conduct postures) and identify where a project would legitimately want to
   insert its own guidance (e.g. reconcile drift rules, close checklists,
   execute conventions).
2. **Design the seams** — which skills resolve which hymn `stage` bands, at what
   altitude, with what precedence vs the shipped skill body. A skill would pull
   its stage band the way the worker def bakes `--role worker` today.
3. **One mechanism, many stages** — reconcile is just the first instance; the
   payoff is a uniform customisation surface across the whole lifecycle.

## Why eventually worth executing

- **DRY / no parallel implementation** — one cascade seam vs N bespoke hook docs.
- **Distinct access pattern from `governance.md`** — governance.md is boot-PUSH
  (resident every session); lifecycle guidance is PULL, stage-scoped (loaded
  only when that skill runs), so it never bloats the resident boot prefix.
- Rides machinery that already exists (SL-186/187 resolver, SL-191 worker
  contract), rather than inventing a new one.

## Dependencies / relations

- Rides the hymns cascade — ADR-011, SPEC-023, SL-186/187 (resolver + delivery),
  SL-191 (worker-contract hymns, authoring conventions + coverage lint).
- Best sequenced **after** SL-191 lands, so the hymns authoring conventions
  (`traits:` frontmatter, band layout, dual-site coverage lint) are stable to
  build on.

## Not this

- Not a per-skill bespoke hook-file family (`reconcile-rules.md`,
  `close-rules.md`, …) — that is the anti-pattern this replaces.
