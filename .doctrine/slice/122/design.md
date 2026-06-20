# SL-122 Design — RFC kind: first-class discussion artifact

Scope: `slice-122.md`. This doc is canon for design intent. Sections lock
incrementally; unresolved questions are named explicitly, not buried.

## §1 Relation model (OQ-2, OQ-3 — LOCKED)

### Grounding

The relation engine is **label-first, not kind-first**. The whole vocabulary is
one flat const table, `RELATION_RULES` (`src/relation.rs:256`); each row is a
5-axis tuple:

```
RelationRule { sources: &[kind_prefix], label, inbound_name, target: TargetSpec, tier, link }
```

A *label* is kind-agnostic; a *rule* binds a label to a set of authoring kinds +
a target constraint. Same label may recur across rows with different source sets
/ target specs (e.g. `related`: governance→`SameKind`, work-items→`AnyNumbered`).
Validation is `lookup(source_kind, label)` → row, then `check_target_kind`
against the row's `TargetSpec` (`relation.rs:897`). Edge logic never lives inside
a Kind — ADR-010's "unify the contract, keep storage bespoke."

"Link to anything" is therefore an **additive const edit, not an engine fork**
(retires R2). Three target modes already exist:
- `TargetSpec::AnyNumbered` — link to any *entity* that exists (`related`,
  `reviews`). Full graph citizen: overlay allocated, reciprocity derived.
- `TargetSpec::Unvalidated` — free-text, no kind check, **no overlay, permanently
  dangling**, preserved for visibility only (`drift`, `decision_ref` — the "DEC
  pattern").
- `TargetSpec::Kinds(&[…])` — closed target set (e.g. `governed_by` → ADR/POL/STD).

### Decision 1 — RFC's own outbound edges (OQ-2)

RFC's general "cite whatever is worth citing" need is served by **`AnyNumbered`**,
not free-text. RFC links to any *entity*; those links are real graph edges
(navigable, reciprocal, rendered by `inspect`). Free-text `Unvalidated` is **not**
adopted as RFC's primary edge — dangling refs that never resolve or reciprocate
defeat the point of a citable discussion artifact. (Borrow `Unvalidated` later
only if a concrete non-entity-citation need appears; out of scope until then.)

RFC-as-source slots in with **no engine special case** — confirmed against both
gates:
- `check_target_kind` (`relation.rs:916`): `AnyNumbered` is a bare no-op arm —
  accepts any target prefix, never reads `source_kind`. RFC→RFC same-kind and
  RFC→other both pass.
- overlay allocation (`relation_graph.rs:162`): table-derived, one overlay per
  distinct non-`Unvalidated` label, auto-allocated; `CyclePolicy::Reject,
  Arity::Unbounded`. A new resolvable label joins the graph for free.

Mechanics: add `RFC` to `integrity::KINDS` (numbered kind), add `RFC` to the
`related` rule's `sources` set (the `AnyNumbered` row). No new label required for
RFC's own context edges.

### Decision 2 — the RFC↔REV precursor edge (OQ-3)

Two hard constraints collapse the option space:
1. **Temporal order** — RFC is born first; REV is born later (when a proposal is
   enacted).
2. **ADR-004 outbound-only** — the edge author needs the target to exist *at
   write time*.

Only **REV → RFC** satisfies both without a §5 carve-out: the RFC exists when the
REV is created, REV authors one outbound edge, RFC's inbound derives. This also
matches who REV already is — the active, later, change-axis edge-author (it
already carries `revises`). The mirror (RFC→REV) would need the ADR-004 §5
carve-out *and* an RFC lifecycle flip to justify rewriting the RFC file — more
machinery, no gain.

`supersedes` is the exact structural template (active/later entity holds outbound;
passive/earlier renders derived inbound). New rule row:

| axis | value |
|---|---|
| `sources` | `&[REV]` |
| `label` | `enacts` — reads "REV-005 enacts RFC-003" |
| `target` | `TargetSpec::Kinds(&[RFC])` — REV enacts RFCs only |
| `inbound_name` | `"enacted by"` — `inspect RFC-003` shows "enacted by: REV-005" |
| `tier` | `Tier::One` (writable) |
| `link` | `LinkPolicy::Writable` — `doctrine link rev-005 enacts rfc-003` |

Arity unbounded both ways (a REV may enact several RFCs; an RFC may be enacted
across REVs). Verb `enacts` over realizes/implements/fulfils (governance register;
"precursor → enactment" is the scope's own framing). **Not** `spawns` (taken:
record→backlog).

**Tier-1 over tier-2**: `enacts` is a bare citation with *no payload*. `revises`
is `TypedVerbOnly` only because its `[[change]]` rows carry payload; `enacts`
carries none, so typing it buys a bespoke verb for nothing. Tier-1 writable via
generic `link`.

Consequences:
- RFC needs **no reverse-edge storage**; "enacted by" is pure derived render
  (ADR-004 clean).
- The precursor link is **not** authored by RFC → it puts RFC in no `sources`
  set. Independent of Decision 1's `related`/`AnyNumbered` edge set.

## §2 Remaining open questions (driving next)

- OQ-4: **Lifecycle states** — status machine vs stateless. Bears on the TOML
  schema and on whether `enacts` flips RFC status.
- OQ-1: **Naming** — `.doctrine/rfc/` (singular, peer convention) vs user's
  `.doctrine/rfcs/`.
- OQ-5: **ADR scope** — confirm the governing ADR + exactly what it asserts.
- OQ-6: **Catalog / boot visibility** — surface in catalog, absent from
  governance sections, no leaked governance position.
