# IMP-351: Skill content is ungoverned under taxonomy change

## Observed

IMP-350 retired `story` from the product altitude ladder. The value was carried
in prose by `plugins/doctrine/skills/spec-coverage-assessment/SKILL.md` step 5
("Product altitude (domain | capability | feature | story)"). Nothing caught it:
it was found by hand-sweeping, and only because that skill happened to be the one
being invoked next.

## The gap

PRD-003 (Skills) and SPEC-010 (Skills distribution) govern how skills are
**curated, delivered, and installed** — the shipping channel. Neither carries any
obligation about skill *content* staying true to the model the skill describes.

So a taxonomy or vocabulary change has no mechanical relationship to the skill
prose that teaches it. Three skills carry taxonomy guidance today
(`spec-product`, `spec-tech`, `spec-coverage-assessment`); a fourth could be
added without anyone knowing it needs sweeping on the next change.

## Sharpened by the publication lag

Skills reach `.agents/` via `npx skills add davidlee/doctrine`, which fetches the
**published** repo — not the working tree. So even after the master is corrected,
the installed copy carries the stale text until release. During IMP-350 the
invoked skill was the 0.33.2 cache, still teaching the retired four-rung ladder
while the binary rejected it. See observation
`019faba8-898d-7231-96f8-6f75a77c2f61`.

That makes the window between a model change and its skill-prose correction
longer than a working-tree edit suggests, and invisible from inside the session
doing the change.

## Not yet a proposal

Deliberately capturing the gap, not the fix. Candidate directions, none assessed:

- a `[[source]]`-style anchor from a spec to the skill prose that teaches it, so
  `validate` can at least report the coupling;
- a doc-vocabulary check in `doctrine check` over a declared closed set;
- an authoring-time convention: closed vocabularies are named in one place and
  skills reference rather than restate them (the STD-001 argument applied to
  prose);
- accept it and rely on sweeping, but record the sweep list somewhere durable.

The last is the cheapest and may be sufficient — the set of skills carrying
taxonomy vocabulary is small and slow-moving.

## Provenance

Surfaced by the RFC-024 / IMP-350 / REV-042 sequence and recorded in RFC-024's
§ Parked state as the one process gap with no home. Coverage census at
`.doctrine/state/spec-coverage-taxonomy.md` gap 4.
