# SL-065 — implementation notes

## PHASE-01 — author product intent (FR-005, FR-006 on PRD-002)

- FR-005 → REQ-259 "Label a product spec with its product level".
- FR-006 → REQ-260 "Decompose a product spec into a single-parent acyclic
  hierarchy" (mirror of REQ-083).
- `doctrine spec req add` reserves identity + label + kind + title only.
  `description` + `acceptance_criteria[]` are **hand-authored** into
  `requirement-NNN.toml` afterwards (edit-preserving). Criteria text lifted from
  design §6 verbatim.
- Pure entity authoring, no source change. `spec validate PRD-002` clean.
  REQ-082/083 (PRD-012, tech-only) untouched.
- Done in solo worktree fork `sl-065-p01`; landed `--no-ff` (e057760) onto main
  (merge 2bf8254), fork gc'd.

## PHASE-02 — data model + render (src/spec.rs)

- `ProductLevel { Domain, Capability, Feature, Story }` — closed enum, kebab
  serde, const `as_str`; structural mirror of `C4Level` (D1). `Spec.product_level:
  Option<ProductLevel>` `#[serde(default)]`, immediately after `c4_level`.
- `spec show` post-category block restructured from a Some-gated `c4 level:` + a
  Tech-gated spine into a single `match spec.kind`:
  - Tech → `c4 level:` → `descends from:` → `parent:` (same lines/order — tech
    render byte-identical, behaviour-preservation gate holds).
  - Product → `product level:` → `parent:`.
- Consequence (design §5 F1): a product spec illegitimately hand-carrying
  `c4_level` no longer renders it — it falls outside the tech branch. Intended,
  more correct, NOT a tech regression. Symmetric for a tech spec's `product_level`.
- Test fallout: the old `render_omits_descent_and_parent_when_none_and_for_product`
  product half asserted product omits `parent:` — that behaviour intentionally
  changes, so its product case was reworked to `parent: None` (still proves the
  product arm never renders `descends_from`). Tech goldens untouched.
- Only ONE full `Spec` literal in tests needed the new field — `tech_spec(id)`
  helper; all others use `..tech_spec(n)` spread.
- `just gate` clean. Solo fork `sl-065-p02`; landed `--no-ff` (6399fa8) onto main
  (merge 65228ca), fork gc'd.
