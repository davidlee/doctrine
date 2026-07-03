# IMP-244: Add concept entity and descriptor facet

Backlog these as two small, separate items:

## 1. Add `concept` as a knowledge kind

**Intent:** Provide a home for user-defined project concepts without introducing ontology machinery or changing the relation algebra.

**Problem:** `knowledge new` currently supports epistemic/governance kinds only: `assumption`, `decision`, `question`, `constraint`, `evidence`, `hypothesis`. There is no neutral, addressable record for local terminology, distinctions, models, or conceptual anchors.

**Change:**

* Add `concept` to `drn knowledge new <KIND>`.
* Generate a standard knowledge record with kind `concept`.
* Provide a concept-oriented markdown template:

  * Definition
  * Notes
  * Distinguish from
  * Examples
  * Related
* Keep concepts graph-addressable like other knowledge records.
* Do not add new relation labels as part of this item.

**Acceptance criteria:**

* `drn knowledge new concept "Attention burden"` succeeds.
* `concept` appears in help/validation output.
* Concept records can be listed, shown, searched, and referenced like existing knowledge records.
* No changes to `RelationLabel` semantics are required.

---

## 2. Add optional per-edge relational descriptors

**Intent:** Let broad, legal relation edges carry human-readable nuance without minting new edge kinds.

**Problem:** Existing relation labels cover the formal graph well, but concept-map/ontology-like usage needs lightweight descriptors such as “contrasts with story-point estimation” or “frames estimation as attention burden.” Adding a new formal relation for each nuance would bloat the closed relation vocabulary.

**Change:**

* Add optional `descriptor: String` metadata to selected relation entries.
* Descriptor is free text only: searchable/renderable, but not used for validation, inference, graph effects, or relation identity.
* Initially allow only on broad associative/contextual/epistemic relations, for example:

  * `contextualizes`
  * `related`
  * `interactions`
  * `references(role = concerns)`
  * future record→work “informs/bears-on” relation
* Explicitly disallow on structural/lifecycle edges such as `parent`, `descends_from`, `members`, `supersedes`, `fulfils`, `owning_slice`.

**Example:**

```toml
[[relations.references]]
role = "concerns"
target = "SL-128"
descriptor = "uses attention burden as the conceptual basis for bounded estimate fields"
```

**Acceptance criteria:**

* Relation entries can optionally store and round-trip `descriptor`.
* Invalid descriptor placement is rejected or ignored consistently by policy.
* Rendered relation views show descriptor text adjacent to the edge.
* Descriptor text is included in search/indexing.
* Existing relation validation, storage tier behavior, and graph effects remain unchanged.

These fit the existing relation taxonomy: it already separates closed semantic classes from legal runtime rules, and treats the taxonomy as descriptive rather than the source of runtime truth. 

