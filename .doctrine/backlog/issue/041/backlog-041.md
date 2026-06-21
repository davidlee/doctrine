# ISS-041: Concept-map contextualizes edges writable but invisible

## Source

IMP-133 UX review, second pass (RF-1). See `.doctrine/backlog/improvement/133/ux-review-findings.md`.

## Problem

`RELATION_RULES` (src/relation.rs) declares `contextualizes` as
`LinkPolicy::Writable`, `Tier::One` for source kind `CM` (concept map):

```rust
RelationRule {
    sources: &[CM],
    label: RelationLabel::Contextualizes,
    inbound_name: "contextualized_by",
    target: TargetSpec::Unvalidated,
    tier: Tier::One,
    link: LinkPolicy::Writable,
},
```

This means `doctrine link CM-001 contextualizes SL-047` succeeds and writes
to `[[relation]]` in `concept-map-001.toml`. Verified: the write is
confirmed, the TOML block is populated.

But `outbound_for` in `src/catalog/scan.rs` explicitly returns empty for CM:

```rust
"REQ" | "CM" => Ok(Vec::new()),
```

Consequence: every read path (`inspect CM-001`, `catalog graph`,
`validate_relations`) is blind to these edges. They exist on disk but are
invisible to the entire read surface.

Additionally, concept-map's own `add`/`remove` commands write edges to the
DSL string field (`dsl = "..."`), not to `[[relation]]`. These two storage
paths are disconnected — no bridge, no migration.

## Evidence

```
$ doctrine link CM-001 contextualizes SL-047
linked: CM-001 contextualizes SL-047

$ doctrine inspect CM-001
CM-001 — relations
(no relations)

$ cat .doctrine/concept-map/001/concept-map-001.toml | grep -A2 '\[\[relation\]\]'
[[relation]]
label = "contextualizes"
target = "SL-047"
```

Catalog graph shows zero CM-sourced edges (live census: 1004 edges, 0 from CM).

## Options

1. **Remove from RELATION_RULES** — if concept maps are intended to use only
   their DSL for edges, the rule entry is incorrect. Remove the
   `contextualizes` rule row for CM. `link CM-001 contextualizes ...`
   would then be refused (no legal label for CM).

2. **Wire the read path** — add a `concept_map::relation_edges` accessor
   that reads `[[relation]]` rows from concept-map TOMLs. Migrate existing
   DSL edges to `[[relation]]` (or build a bridge that projects DSL edges
   into the relation graph at scan time).

3. **Hybrid** — read DSL edges as `contextualizes` relations at scan time
   without changing storage (the `outbound_for` dispatch reads the DSL,
   projects edges). `link`-authored `[[relation]]` rows co-exist; both
   paths feed into the graph. Simplest bridge, no migration needed.

## Decision needed

The `RELATION_RULES` entry and the `outbound_for` dispatch must agree.
Either the rule is wrong (remove it) or the scan is incomplete (wire it).
