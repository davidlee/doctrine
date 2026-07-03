# Review RV-236 — design of SPEC-003

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Target: SPEC-003, the active whole-system context technical specification for
Doctrine.

Protocol note: `doctrine review prime RV-236` refused because the review targets
`SPEC-003`, not a slice, and the cache primer currently requires slice selectors.
The tribunal proceeds on the ledger; the warm-cache absence is a tooling
limitation, not an absolution.

Lines of interrogation:

- Hold SPEC-003 to PRD-012 and SPEC-017: tech specs must keep clear C4 altitude,
  descent/decomposition discipline, and context-level anchor exceptions without
  smuggling a container mechanism into the root.
- Hold SPEC-003 to the relation canon: ADR-004 is superseded; ADR-010 and
  SPEC-018 now carry the active relation contract. Any live root citation to
  dead authority is suspect.
- Hold SPEC-003 to STD-002 and the boot reference discipline: cite durable ids
  when naming governed containers and requirements; do not rely on slugs,
  mobile labels, or naked prose names where an entity id exists.
- Check whether the prose container list matches the structured parent tree and
  whether every claimed child container has an explicit canonical home.

## Synthesis

Judgement: SPEC-003 passes the mechanical FK tribunal (`doctrine spec validate
SPEC-003` is clean), but it does not leave the chamber unscorched. Three
heresies were proven and entered as verified findings:

- F-1 (major): the root relation principle still names superseded ADR-004 as
  live authority, while the active relation contract is ADR-010 / SPEC-018.
- F-2 (minor): two child containers are named in prose without their durable
  ids, even though SPEC-012 and SPEC-013 already parent to SPEC-003.
- F-3 (major): SPEC-003 is active with `descends_from = null`; if this is a
  lawful root-context exception, canon must say so instead of leaving silence
  to masquerade as doctrine.

Sentence: IMP-237 is the assigned penance. It must repair the relation-canon
citation, add the missing container ids, and settle the root descent exception
explicitly. Until then, SPEC-003 remains structurally valid but doctrinally
tainted: not condemned by the parser, but marked by the ledger.

Standing risk: `review prime` cannot warm a cache for a non-slice target, so
spec/ADR/design inquisitions currently proceed without selector-backed staleness
state. That tooling limitation was recorded in the brief and did not block this
review, but it remains a sharp edge for future tribunals.
