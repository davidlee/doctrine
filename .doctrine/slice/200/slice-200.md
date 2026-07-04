# Memory wikilink resolver complains; corpus relinked

## Context

The shipped memory corpus (`memory/`, 31 memories) is **functionally unlinked**.
Bodies cite related memories with prefix-less wikilinks —
`[[signpost.doctrine.lifecycle-start]]`, `[[concept.doctrine.memory-model]]`,
`[[fact.doctrine.storage-tiers]]` — but the extractor regex
(`src/links.rs:16` `extract_wikilinks`) requires a `mem.`/`mem_` prefix:

```
\[\[(mem\.[…]|mem_[…])\]\]
```

So prefix-less links match **nothing** — not even flagged dangling. Empirical:

```
$ doctrine memory resolve-links mem.signpost.doctrine.overview
resolved: 0   dangling: 0   dangling_targets: []
```

The corpus *looks* linked (bracket prose) but has zero graph edges and zero
backlinks. This blocks human onboarding via the map explorer (SL-201): with no
memory→memory edges, there is nothing to browse. This slice makes the corpus
actually linked, and installs a guardrail so future prefix-less links are loud
rather than silent.

Design decision (owner: user): keep the strict `mem.` contract — do **not**
loosen the resolver to silently accept prefix-less targets. Instead **complain**:
teach the extractor to recognise a memory-*shaped* prefix-less link as a
malformed/dangling link so it surfaces in `resolve-links`/audit, then **clean up**
the shipped corpus to canonical `[[mem.…]]` form.

## Scope & Objectives

1. **Resolver complains.** Extend `extract_wikilinks`/`resolve_wikilink`
   (`src/links.rs`) so a prefix-less but memory-shaped `[[…]]` is recognised as a
   *malformed* memory link and resolves to dangling (surfaced by
   `memory resolve-links` and the backlinks index), instead of being dropped by
   the regex. Requires a discriminator (see OQ-1) to avoid false-positiving prose
   brackets (`[[TODO]]`) and TOML-ish noise.
2. **Corpus cleanup.** Rewrite every shipped-memory body link to canonical
   `[[mem.<type>.<domain>.<subject>]]` form so it resolves. Mechanical, ~35 files
   — dispatchable to a cheap agent once (1) makes the offenders enumerable via
   `resolve-links`.
3. **Result:** memory→memory links resolve → backlinks populate → the catalog
   graph carries memory edges → the corpus is browsable/linked in the map.

## Non-Goals

- The map command / `--focus` on memory refs / onboarding URL — that is **SL-201**.
- Loosening the `mem.` prefix contract (rejected: complain, don't silently
  accept).
- Auditing the *content* / correctness of which memories link to which (the user
  may do link-content review off-books); this slice fixes the *mechanism* and
  the *syntactic* form, not editorial link choices.
- New memory relations in TOML (`[[relation]]` typed rows) — out of scope.

## Affected Surface

- `src/links.rs` — extractor/resolver (the leaf; string-keyed, pure).
- `src/memory.rs` — consumers: `resolve-links`, `backlinks`, `show`
  (`extract_wikilinks`/`resolve_wikilink`/`backlinks_index` call sites).
- `src/catalog/hydrate.rs` — the **catalog/map graph** path uses its own
  `classify_target` + `mem_key_map` (line ~380), *not* `resolve_wikilink`.
  Whether memory body wikilinks flow into catalog edges at all is OQ-2.
- `memory/**` — shipped corpus bodies (`*.md`) to relink (cheap-agent pass).

## Risks / Assumptions / Open Questions

- **OQ-1 (design call).** Discriminator for "meant to be a memory link":
  dotted `type.domain.subject` shape, or match against the known memory-type
  prefixes (`signpost|concept|fact|pattern|…`). Must not fire on `[[TODO]]`,
  `[[row]]`, other bracket prose.
- **OQ-2.** Does the **catalog graph** (map edges) derive memory edges from body
  wikilinks, or only from typed TOML relations? `hydrate.rs` resolves via
  `mem_key_map`/`classify_target`, a separate path from `links::resolve_wikilink`.
  If body wikilinks are not scanned into catalog edges, Slice 1 must wire that
  too for the map payoff to land. Resolve in `/design`.
- **Behaviour-preservation:** `links.rs` and `memory.rs` suites are the proof —
  existing green tests for the `mem.`-prefixed happy path must stay green.
- **Assumption:** relinking is purely syntactic (add `mem.` prefix); targets
  already name real memory keys. Verify during cleanup — any target with no
  matching key is a genuine dangling link to fix editorially.

## Verification / Closure Intent

- `doctrine memory resolve-links` (corpus-wide) reports **0 dangling** after
  cleanup, with a non-zero resolved count for memories that cite others
  (e.g. `mem.signpost.doctrine.overview` resolves its ~10 quick-links).
- A regression test: a prefix-less memory-shaped `[[…]]` is reported dangling
  (the "complain" behaviour), while genuine prose brackets are not.
- `doctrine backlinks mem.signpost.doctrine.lifecycle-start` shows inbound links.
- Map graph carries memory→memory edges (contingent on OQ-2).
- Existing `links.rs` / `memory.rs` suites green, unchanged.

## Follow-Ups
