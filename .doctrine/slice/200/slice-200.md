# Memory onboarding corpus: correct wikilink keys

## Context

Doctrine ships an agent-memory corpus (`memory/`) as human onboarding material,
but it is not browsable as a linked graph. The onboarding memories cross-
reference each other **only** via body wikilinks in prose —
`See [[signpost.doctrine.lifecycle-start]]` — and those links are **broken**:

- The extractor (`src/links.rs` `extract_wikilinks`) requires a `mem.`/`mem_`
  prefix. **0 of 276** body wikilinks in the shipped corpus carry it — every
  one is prefix-less. So none resolve: `resolve-links`/`backlinks`/`show`
  report nothing, and the corpus reads as linked prose but is functionally
  unlinked.

This slice fixes the **corpus** — the syntactic form of the links — so they
resolve. It is the curation half of a three-part onboarding goal; the two code
halves are split out as backlog issues (see Dependencies):

- **ISS-213** — `build_memory_key_map` ignores shipped memories (map-side
  shipped-key resolution).
- **ISS-214** — catalog/web-map never reads memory `.md` bodies, so body
  wikilinks produce no map edges.

**Path decision (owner: user): Path 2 — single link store.** Keep the body
prose wikilink as the *one* expression of a memory's associations; make every
surface read it (CLI now via this slice; the web map once ISS-213 + ISS-214
land). Rejected: authoring duplicate TOML `[[relation]]` rows (Path 1) — a
second, drift-prone link store beside the prose.

## Scope & Objectives

1. **Correct the shipped wikilink keys.** Rewrite prefix-less body wikilinks to
   canonical `[[mem.<type>.<domain>.<subject>]]` form across `memory/` so the
   extractor matches and they resolve. ~258 are a mechanical prepend.
2. **Skip false positives.** `[[status_delta]]`, `[[evidence_ref]]` (and any
   like them) are prose references to TOML table/field names, **not** memory
   links. They must **not** be rewritten. A blind prepend corrupts them —
   the cleanup needs a discriminator (memory-shaped `type.domain.subject`
   dotted key, or match against known memory-type prefixes).
3. **Resolve editorial danglers.** `[[signpost.doctrine.rec/rfc/concept-map]]`,
   `[[concept.backlog.work-intake-membership]]` don't currently resolve as
   shipped keys — verify the targets ship (some may be broken shipped symlinks
   / absent uid dirs) and fix or retarget.
4. **Verify CLI resolution.** After the fix, `memory resolve-links` over the
   shipped corpus reports the cross-references resolved, 0 spurious dangling;
   `backlinks` populate.

The map payoff (onboarding memories browsable as a linked graph) is delivered
when ISS-213 + ISS-214 land on top of this slice's corrected keys.

## Non-Goals

- The map/catalog code changes to ingest body wikilinks — **ISS-214**.
- Shipped-key resolution in `build_memory_key_map` — **ISS-213**.
- The map `--focus` on memory refs and the onboarding command/URL — **SL-201**.
- Authoring TOML `[[relation]]` rows on memories (Path 1, rejected).
- A resolver "complain" guardrail against future prefix-less links — candidate
  follow-up (fold into ISS-214's extract path or a new hygiene item), not this
  slice.
- Editorial review of *which* memories *should* link to which (link content) —
  this slice fixes the syntactic form of existing links, not their curation.

## Affected Surface

- `memory/**` — shipped-corpus `.md` bodies (the key rewrite).
- No `src/` changes in this slice — the enabling code is ISS-213 / ISS-214.
- Post-edit: `cargo build` (re-embed RustEmbed), `doctrine memory sync` to
  materialise, per the shipped-memory edit workflow.

## Dependencies

- **ISS-214 `after` SL-200** — body-wikilink map edges need this slice's
  corrected keys to resolve.
- **ISS-213** — shipped-key resolution; pairs with ISS-214 for the map payoff.
  Independent of this slice's CLI outcome.
- **SL-201** (`after` SL-200 already) — the onboarding command/URL; unaffected
  by the path choice.

## Risks / Assumptions / Open Questions

- **OQ-1 (discriminator).** The cleanup must distinguish memory-link brackets
  from prose brackets (`[[status_delta]]`). Manual judgement for ~18 exceptions,
  or a scripted discriminator (dotted `type.domain.subject` shape). Since this
  slice touches only the corpus (no resolver change), a careful edit pass with
  an explicit skip-list is sufficient; the durable discriminator belongs to
  ISS-214's extractor if wanted.
- **Assumption:** the 258 prefix-less links are syntactically fixable by
  prepending `mem.`; their targets are real shipped keys. Verified by
  resolution count after the edit.
- **Risk:** shipped-memory edits require the re-embed + sync round-trip; a
  forgotten rebuild ships stale embedded bytes (AGENTS.md nix embed note).
- **Danglers:** rec/rfc/concept-map/work-intake targets need verification —
  they may indicate onboarding memories referenced-but-not-shipped.

## Verification / Closure Intent

- `doctrine memory resolve-links` over the shipped corpus: cross-references
  resolve; remaining dangling count is only genuine editorial danglers, 0 from
  the prefix bug.
- `doctrine backlinks mem.signpost.doctrine.lifecycle-start` shows inbound links
  from the memories that cite it.
- `[[status_delta]]` / `[[evidence_ref]]` are unchanged (not corrupted).
- Corpus re-embedded + synced; no stale-embed drift.
- (Downstream, post ISS-213/214) the onboarding memories draw edges in the map.

## Follow-Ups

- Resolver "complain" guardrail for future prefix-less memory-shaped wikilinks
  (regression fence) — fold into ISS-214 or a new hygiene item.
