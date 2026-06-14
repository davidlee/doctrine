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
4. **Flip product `parent` validate rule** (`registry.rs::parent_findings`,
   REQ-083): on a *product* subject, `parent` is now VALID and must resolve to a
   *product* spec — a tech parent is invalid-kind, an absent target is dangling,
   the self case is excluded (mirror the tech rule, inverted kind). Reciprocal
   "decomposes into" view stays derived (ADR-004), never stored.
5. **Product self-parent / cycle guard** — extend the `self_parent` 1-cycle
   finding (SL-022 PHASE-03) to product subjects so `PRD-A parent PRD-A` is
   caught.

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
- **Multi-hop product decomposition cycle detection** beyond the self-loop, if
  the tech side has none — match tech's depth, no more.

## Affected surface

- `src/spec.rs` — `ProductLevel` enum; `Spec.product_level` field; product
  `show` render branch (level + parent).
- `src/registry.rs` — `parent_findings` product branch (REQ-083 flip);
  `self_parent` product extension; possibly `ParentEdge.on_product` consumers.
- `install/templates/spec-product.toml` — optional commented `product_level`
  hint (decide in design; `c4_level` is not templated, so likely none).
- Requirement touch: **REQ-083** currently states `parent` is tech-only. Flipping
  the product rule changes its normative content — design must decide whether
  this is a requirement revision (IDE-003 revision-vehicle territory) or a
  scoped amendment.

## Risks / Open Questions

- **OQ-1** REQ-083 semantics change — is product-parent a revision of REQ-083 or
  a new sibling requirement? (storage-rule / spec-product call in `/design`.)
- **OQ-2** `descends_from` once product has levels: the natural target is a
  `capability` PRD. Defer (Non-Goal) but record the intended future constraint.
- **OQ-3** Should `validate` warn (not reject) when a non-`story` PRD has no
  children, or a `story` has children? Advisory hygiene — likely defer.
- **ASM-1** Tech `parent` cycle protection is self-loop-only; product mirrors
  that depth. Verify in design.

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
