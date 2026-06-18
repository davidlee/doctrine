# Spec composition seam — requirement peer entity, membership labels, edges

How a spec composes its requirements and cross-spec edges, as shipped in SL-015.
Distinct from `mem.system.engine.identity-claim-seam` (that is `entity.rs`'s
numeric/named materialiser); this is the spec-side composition layer in
`src/spec.rs` / `src/requirement.rs` / `src/registry.rs`.

## 1. A requirement is a reserved numeric PEER entity, not a facet row

`REQ-NNN` under `requirement/NNN/` (own tree, own reservation namespace), reserved
through the same `entity.rs` `Fresh` materialiser as specs/slices. Identity is
**immutable**. It is **spec-mediated** — no standalone CLI; `spec req add` is the
only producer. This overturned the old canon's compound-key facet row
`SPEC-110.FR-001` (see SPEC-004, the entity engine spec).

## 2. Membership is a spec-side edge row carrying FK + a MOBILE label

A spec's `members.toml` holds `[[member]] requirement = "REQ-007"  label = "FR-001"
order = 1`:
- `requirement` — the **durable canonical-string FK** (registry stores canonical
  strings, not numerics; checks are direct `BTreeSet::contains`).
- `label` — a **sticky per-spec display label**, `FR-` functional / `NF-` quality,
  auto-assigned next-free-by-kind (`spec req add --label` overrides). It lives on
  the EDGE, so it is mobile: the same requirement membered by two specs can carry a
  different label in each; identity never moves.
- `order` — advisory stable-sort key (gaps/dups cosmetic, not validated).

The append is **edit-preserving** (`toml_edit` array-of-tables `push`, NOT serde
reserialize). `spec req add` is a two-tree write (reserve `REQ-NNN` + append row),
NOT transactional: reserve-OK-append-fails leaves an **orphan** requirement
(uncommitted dir) that `spec validate` flags hard.

## 3. Edges: members (both subtypes) + interactions (tech only); collaborators GONE

`interactions.toml` `[[edge]] target = "SPEC-NNN"` is the tech-only spec→spec edge
(absent on product). `collaborators.toml` is **dissolved** — cross-spec requirement
reuse is the deferred `spec req link` verb (a second `members.toml` row), not a
table.

## 4. `spec validate` is the FK gate (registry seed)

`src/registry.rs` scans the trees into canonical-string id sets + an edge list and
hard-checks: dangling member FK, dangling tech-only interaction target, duplicate
label within a spec, orphan requirement (corpus-only). Scoped run checks one spec's
outbound FKs + label uniqueness; orphan is whole-corpus only. The tech-only
interaction rule falls out free — a `PRD-*` target is simply absent from the
`tech_specs` set. Cache + cycle detection deferred to the feature DAG.

## 5. `spec show` is the one-way ephemeral reader (D8/D9)

Pure stdout reassembly: identity + flat fields + `spec-NNN.md` verbatim +
Requirements section (members in `order`, each requirement read by FK; the
requirement's structured `description` is its statement line — NOT its `.md` prose,
per the storage rule) + outbound interactions (tech). No write, no mutation, cannot
go stale. Materialised `*.rendered.md` is deferred derived-tier.
