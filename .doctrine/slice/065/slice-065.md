# Product-level axis and PRD decomposition parent

## Context

Tech specs (`SPEC`) carry a `c4_level` axis (closed enum context/container/
component/code) and a single decomposition `parent` (`SPEC→SPEC`, reciprocal
derived per ADR-004). Product specs (`PRD`) have **no level axis** and their
`parent` is currently *rejected* by `spec validate` as a tech-only field
(`registry.rs::parent_findings`, REQ-083). So the product side of the spec model
is asymmetric: there is no vocabulary for product altitude and no queryable
decomposition lineage.

This slice adds the product analogue of the tech C4 ladder, and makes the
already-present-but-rejected product `parent` field a first-class decomposition
axis.

Product level ↔ C4 analogue:

| product_level | meaning                                             | C4 analogue |
| ------------- | --------------------------------------------------- | ----------- |
| `domain`      | broad product area / user-problem / strategic space | context     |
| `capability`  | durable ability the product should provide          | container   |
| `feature`     | coherent user/system-facing function in a capability| component   |
| `story`       | specific need/scenario/outcome motivating reqs      | code-ish    |

Design intent: `domain` decomposes into `capability` → `feature` → `story`.

## Scope & Objectives

1. **`ProductLevel` enum** — closed set `{domain, capability, feature, story}`,
   kebab serde, `as_str` render helper. Structural mirror of `C4Level`
   (`src/spec.rs`).
2. **`product_level: Option<ProductLevel>`** flat field on `Spec`, `#[serde
   (default)]`, optional, hand-authored into `spec-NNN.toml` (no `spec new`
   flag — mirror `c4_level`). Absent/ignored on tech specs.
3. **`spec show` render** — emit `product level: <level>` for product specs, and
   the outbound `parent: PRD-NNN` decomposition edge (product `show` currently
   renders neither). Keep tech render unchanged.
4. **Symmetric same-subtype `parent` validate rule** (`registry.rs::
   parent_findings`): replace the three tech-only special-cases with one rule —
   `parent` must resolve to a spec of the SAME subtype as the subject. Product→
   product valid; product→tech invalid-kind; missing dangling. Tech branch
   unchanged. Reciprocal "decomposes into" stays derived (ADR-004), never stored.
5. **Subtype-blind acyclicity** — drop the `on_product` exclusion from
   `self_parent` AND `parent_cycle` (REQ-087) so product decomposition gets the
   full single-parent-acyclic guarantee tech has (self-loop AND multi-hop cycle),
   by deletion not a parallel pass.
6. **Author two product requirements on PRD-002** — FR-005 (product_level
   taxonomy) and FR-006 (product decomposition, mirror of REQ-083). Product intent
   precedes the mechanism.

## Non-Goals

- **Level-adjacency enforcement** (a `feature`'s parent must be a `capability`,
  one rank up). Advisory in v1 — `product_level` is a tag, `parent` is any
  PRD→PRD. Tech does not enforce c4-adjacency on its `parent`; importing stricter
  ceremony than the project runs is out (route rule). Deferred as a `validate`
  tightening → Follow-Ups.
- **`descends_from` capability-retarget** — tightening a tech spec's
  `descends_from` to point specifically at a `capability`-level PRD. Couples two
  axes; not needed to ship the level. Stays tech-only, unchanged → Follow-Ups.
- **`spec new` flag for `product_level`** — hand-authored, mirroring `c4_level`.
- **Level-adjacency on `parent`** (a `feature`'s parent must be a `capability`).
  Advisory v1; deferred tightening (tech enforces no c4-adjacency).

## Affected surface

- `src/spec.rs` — `ProductLevel` enum; `Spec.product_level` field; product
  `show` render branch (level + parent).
- `src/registry.rs` — `parent_findings` symmetric same-subtype rule; drop the
  `on_product` exclusion from `self_parent` + `parent_cycle` (subtype-blind).
- PRD-002 — two new functional requirements (FR-005 product_level, FR-006 product
  decomposition) via `doctrine spec req add`.
- `install/templates/spec-product.toml` — no change (c4_level isn't templated
  either; hand-authored).

## Risks / Open Questions

- **OQ-1** (deferred → Follow-Up) `descends_from` once product has levels: natural
  target is a `capability` PRD. Record the intended future constraint.
- **OQ-2** (deferred → Follow-Up) level-adjacency: parent exactly one rank above
  child. Advisory-only v1.
- **ASM-1 (corrected)** Tech `parent` has FULL acyclicity (`self_parent` +
  `parent_cycle`, REQ-087), not self-loop-only. Product parity = subtype-blind
  reuse of both, by deletion. REQ-082/083 untouched (tech-only, correct).

## Verification / closure intent

- `ProductLevel` round-trips kebab serde; product `show` emits `product level:`
  and outbound `parent:` lines in deterministic order.
- `spec validate` accepts a product→product `parent`, rejects a product→tech
  parent (invalid kind), flags a dangling/self product parent.
- Tech-side behaviour (`c4_level`, tech `parent`/`descends_from` validate) is
  unchanged — the existing spec suites stay green (behaviour-preservation gate).

## Follow-Ups

- Level-adjacency `validate` tightening (parent exactly one rank up).
- `descends_from` → capability-level PRD constraint (OQ-2).
