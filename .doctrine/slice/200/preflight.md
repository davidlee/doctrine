# SL-200 preflight analysis

2026-07-04

## Scope confirmed

| Fact | Number |
|---|---|
| Unique memory-key wikilinks without `mem.` prefix | **32 targets** |
| Total occurrences across shipped corpus | **138** |
| Files with at least one prefix-less wikilink | **32** `.md` files in `memory/` |
| Targets that resolve to an existing `memory/` dir | **32/32** (100%) |
| Targets that do NOT resolve (editorial danglers) | **0** |
| False positives to skip | **2**: `[[status_delta]]`, `[[evidence_ref]]` |

All targets verified to exist as shipped memory directories — zero editorial
danglers.

## False positives

Both in `memory/mem_019f176f71537d12b1b09826a003a602/memory.md`:

- `[[status_delta]]` (lines 13, 23) — describes a TOML `[status_delta]` table in `rec-NNN.toml`
- `[[evidence_ref]]` (lines 17, 23) — describes a TOML `[[evidence_ref]]` array in `rec-NNN.toml`

Neither is a memory link. Both use `snake_case` single/double-word form, not the
dotted `type.domain.subject` shape.

## Discriminator (OQ-1 resolved)

**Dotted `type.domain.subject` form is a clean discriminator.** Every non-memory
bracket in the corpus (`status_delta`, `evidence_ref`) lacks the dotted form. A
regex matching `[[type.domain.subject]]` (the key shape minus the `mem.` prefix)
cleanly selects only memory targets. This holds across all 138 occurrences.

## Design-vs-reality deltas

| Design claim | Actual |
|---|---|
| "~258 prefix-less links" | 138 occurrences of 32 unique targets |
| "rec/rfc/concept-map" dangler | No such literal exists; `mem.signpost.doctrine.rec`/`.rfc`/`.concept-map` all ship |
| `[[concept.backlog.work-intake-membership]]` might be a dangler | Ships at `memory/mem.concept.backlog.work-intake-membership/` |

## TOML vs body wikilinks

`resolve-links` currently reports 326 resolved — these are TOML `[[relation]]`
entries (uid-form references), not body wikilinks. Zero body wikilinks resolve
because the extractor requires `mem.`/`mem_` prefix and all 138 shipped corpus
body wikilinks lack it.

After the fix, `resolve-links` will add 138 body-wikilink resolutions.

## Unique wikilink targets (all 32)

```
concept.backlog.work-intake-membership
concept.doctrine.boot-snapshot
concept.doctrine.entity-engine
concept.doctrine.memory-model
concept.doctrine.reading-entities
concept.doctrine.routing-gate
concept.doctrine.storage-model
fact.doctrine.cli-source-of-truth
fact.doctrine.storage-tiers
pattern.doctrine.conventions
pattern.doctrine.core-loop
pattern.doctrine.tdd-loop
signpost.doctrine.adrs
signpost.doctrine.audit
signpost.doctrine.backlog
signpost.doctrine.concept-map
signpost.doctrine.file-map
signpost.doctrine.install
signpost.doctrine.knowledge
signpost.doctrine.lifecycle-start
signpost.doctrine.overview
signpost.doctrine.policies-standards
signpost.doctrine.rec
signpost.doctrine.recording-memories
signpost.doctrine.reference-docs
signpost.doctrine.relating-entities
signpost.doctrine.requirements
signpost.doctrine.review
signpost.doctrine.revisions
signpost.doctrine.rfc
signpost.doctrine.skill-map
signpost.doctrine.specs
```

## Execution plan summary

1. Mechanical `mem.` prepend to all 138 occurrences across ~32 `.md` files
2. Skip-list guard for `[[status_delta]]` and `[[evidence_ref]]` — automatically
   excluded by the dotted-form discriminator
3. Post-edit: `cargo build` → `doctrine memory sync` → `doctrine memory
   resolve-links` → `doctrine check commit`

## Assumptions

- The 32 targets are the complete set (automated rg scan covered all `.md` in `memory/`)
- The dotted-form discriminator holds across the entire corpus (verified)
- Standard shipped-memory edit workflow applies (re-embed + sync round-trip)
