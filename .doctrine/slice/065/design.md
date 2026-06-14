# Design SL-065: Product-level axis + PRD decomposition parent

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-065, ADR-004, REQ-083); doc-local refs bare — OQ-1 (§8), D1 (§7). -->

Scopes the product analogue of the tech-spec C4 ladder. Adds a `product_level`
taxonomy to product specs and promotes the already-present-but-rejected product
`parent` field to a first-class intra-family decomposition axis. Four locked
decisions frame this design: requirements home **PRD-002** (two new FRs, not a new
PRD); the **symmetric same-subtype parent** validate rule (replacing three
tech-only special-cases); **subtype-blind** acyclicity; **advisory** levels (no
rank-adjacency enforcement in v1).

## 1. Design Problem

The spec model is asymmetric. Tech specs (`SPEC`) carry a `c4_level` altitude
(closed enum context/container/component/code) and a single decomposition
`parent` (`SPEC→SPEC`, single-parent-acyclic, reciprocal derived per ADR-004,
REQ-083). Product specs (`PRD`) have:

1. **no altitude vocabulary** — nothing names whether a PRD is a broad domain or a
   narrow story; and
2. **a `parent` field that exists on the shared `Spec` struct but is hard-rejected
   by `spec validate`** as a tech-only field at three sites
   (`registry.rs::parent_findings`, `::self_parent`, `::parent_cycle` all gate on
   `e.on_product`), and is suppressed in `spec show` render with an explicit "do
   not legitimise a hard-invalid field" comment.

So a PRD hierarchy can only be described in prose, never recorded as a queryable
relation. This slice closes both gaps with a structural mirror of the tech side.

Product level ↔ C4 analogue:

| product_level | meaning                                              | C4 analogue |
| ------------- | ---------------------------------------------------- | ----------- |
| `domain`      | broad product area / user-problem / strategic space  | context     |
| `capability`  | durable ability the product should provide           | container   |
| `feature`     | coherent user/system-facing function in a capability | component   |
| `story`       | specific need/scenario/outcome motivating reqs       | code-ish    |

Design intent: `domain` decomposes into `capability` → `feature` → `story` — but
the ladder is **advisory** in v1 (§7 D2): the level is a tag, `parent` is any
same-subtype edge, no rank-adjacency check.

## 2. Current → target behaviour

| surface | current | target |
| --- | --- | --- |
| product altitude | none | `product_level ∈ {domain,capability,feature,story}`, optional, hand-authored |
| product `parent` validate | invalid-kind (tech-only field) | valid `PRD→PRD`; parent must be same subtype |
| cycle detection | tech-only (`on_product` excluded from self/cycle) | subtype-blind (chain is a chain) |
| product `show` | no level, no spine line | `product level:` + `parent:` lines |
| tech behaviour | — | **byte-identical** (behaviour-preservation gate) |

## 3. Data model — `src/spec.rs`

New closed enum, structural mirror of `C4Level` (same derives, kebab serde,
`as_str` render helper):

```rust
/// The product altitude of a product spec. Closed set, kebab serde; product-only,
/// optional. Mirror of C4Level (domain≈context, capability≈container,
/// feature≈component, story≈code).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ProductLevel { Domain, Capability, Feature, Story }

impl ProductLevel {
    const fn as_str(self) -> &'static str {
        match self {
            ProductLevel::Domain => "domain",
            ProductLevel::Capability => "capability",
            ProductLevel::Feature => "feature",
            ProductLevel::Story => "story",
        }
    }
}
```

New flat field on `Spec`, placed immediately after `c4_level`:

```rust
#[serde(default)]
pub(crate) product_level: Option<ProductLevel>,
```

**Decision D1 — two enums, not one generic `Level`.** `ProductLevel` and `C4Level`
are structurally identical but semantically distinct vocabularies. A shared generic
would obscure the domain meaning and force a parameterisation the rest of the code
does not want; the duplication is four match arms and a derive line. Mirror, don't
abstract. (No-parallel-implementation still holds: this is the *same pattern*
applied to a sibling axis, not a duplicated mechanism — the mechanism is `serde` +
`as_str`, unchanged.)

## 4. Validate model — `src/registry.rs`

Replace the three tech-only special-cases with one symmetric rule.

**`parent_findings` — same-subtype rule.** `parent` must resolve to a spec of the
**same subtype** as the subject:

| subject | parent | verdict |
| --- | --- | --- |
| product | product | valid (new) |
| product | tech | `invalid parent: … is a tech spec (must be product)` |
| product | missing | `dangling parent: …` |
| tech | tech | valid (unchanged) |
| tech | product | `invalid parent: … is a product spec (must be tech)` (unchanged) |
| tech | missing | dangling (unchanged) |

`ParentEdge.on_product` becomes the discriminator that selects *which* subtype the
parent must match, rather than a flag that rejects outright. Self-edges remain
excluded here (owned by `self_parent`).

**`self_parent` + `parent_cycle` — drop the `on_product` filter.** Acyclicity is
subtype-independent: a `child→parent` chain is a chain regardless of family. Remove
`&& !e.on_product` (self_parent) and the `e.on_product ||` guard (parent_cycle) so
both subtypes get the 1-cycle and multi-hop cycle guarantees REQ-083/REQ-087 give
tech. This is a **net deletion**, not a parallel pass. The ephemeral child→parent
map (storage rule — never persisted) is unchanged in shape.

Cross-subtype edges (already invalid-kind via `parent_findings`) may appear in the
unified cycle map; this is harmless — they are already reported as errors and
cannot manufacture a spurious *additional* cycle finding that matters. In practice
decomposition chains are within-family.

`descent_findings` (descends_from) is **untouched** — descends_from stays tech-only
(deferred, §8 OQ-2).

## 5. Render — `src/spec.rs::show`

Today render is: `…category` → `c4 level:` (Some) → tech-gated spine
(`descends from:`, `parent:`). Restructure the post-category block to branch on
subtype so tech output stays byte-identical and product gains its own lines:

- **tech subject**: `c4 level:` (Some) → `descends from:` (Some) → `parent:` (Some)
  — same lines, same order as today.
- **product subject**: `product level:` (Some) → `parent:` (Some).

Example strings: `product level: capability`, `parent: PRD-003`. Children are
**never** rendered — reciprocal "decomposes into" is derived (ADR-004 §3,
outbound-only). `responsibilities`/`sources` blocks are unchanged (empty on a
product spec, so no output).

## 6. Spec-product authoring — two requirements on PRD-002

Product intent must precede the mechanism, so the slice authors two functional
requirements via `doctrine spec req add PRD-002 --kind functional "<title>"`
(reserves the REQ, appends as the next `FR-`):

- **FR-005 — "Label a product spec with its product level"**
  - A product spec can record a single product level from the closed set
    `domain | capability | feature | story`.
  - The level is optional; an unlabelled product spec is valid.
  - The level is shown in the spec's rendered identity.

- **FR-006 — "Decompose a product spec into a single-parent acyclic hierarchy"**
  (mirror of REQ-083)
  - A product spec can record a single parent product spec, marking it a child in
    the decomposition.
  - Containment is stored once, outbound on the child; the parent's children are
    derived, never stored.
  - A product spec has at most one parent, and no chain of parents forms a cycle.
  - A parent must be a product spec; a tech parent is invalid.

REQ-082/083 (PRD-012, tech-only) are **not** touched — they are correct as-is. The
`parent_findings` doc-comment will cite the new product REQ alongside REQ-083.

## 7. Decisions

- **D1 — two enums, not a generic `Level`** (§3). Clarity over nominal DRY.
- **D2 — advisory levels.** No rank-adjacency enforcement (a `feature`'s parent need
  not be a `capability`). Tech does not enforce c4-adjacency on its `parent`;
  importing stricter ceremony than the project runs is out of scope (route rule).
  Deferred as a `validate` tightening (§8).
- **D3 — symmetric same-subtype parent + subtype-blind cycles** (§4). Replaces three
  divergent tech-only branches; gives product full acyclicity by deletion.
- **D4 — requirements home PRD-002, two FRs** (§6). No parallel "Product
  Specifications" PRD — the product-only surface (two fields) does not justify one;
  PRD-002 already holds cross-cutting spec requirements. Reversible.

## 8. Open questions / deferred

- **OQ-1 (deferred, Follow-Up):** level-adjacency `validate` tightening — parent
  exactly one rank above child. Advisory-only in v1 (D2).
- **OQ-2 (deferred, Follow-Up):** `descends_from` → capability-level PRD constraint.
  Now that product has levels, a tech spec's descent target is naturally a
  `capability` PRD; couples two axes, not needed to ship. descends_from stays
  tech-only and unconstrained on level.

## 9. Verification

- `ProductLevel` kebab round-trips the identity toml; an unknown variant is rejected
  at parse.
- product `show` emits `product level:` + `parent:` in deterministic order; **tech
  `show` goldens unchanged** (byte-identical).
- `spec validate`: product→product parent accepted; product→tech rejected
  (invalid-kind); dangling product parent flagged; product self-loop
  (`self_parent`) and product `A→B→A` cycle (`parent_cycle`) caught.
- existing spec + registry suites stay green unchanged (behaviour-preservation gate
  on the shared engine).
