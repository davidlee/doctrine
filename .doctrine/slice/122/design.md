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
| `label` | `originates_from` — reads "REV-005 originates from RFC-003" |
| `target` | `TargetSpec::Kinds(&[RFC])` — a REV originates from RFCs only |
| `inbound_name` | `"precursor of"` — `inspect RFC-003` shows "precursor of: REV-005" |
| `tier` | `Tier::One` (writable) |
| `link` | `LinkPolicy::Writable` — `doctrine link rev-005 originates_from rfc-003` |

Arity unbounded both ways (a REV may draw on several RFCs; an RFC may precede
several REVs).

**Verb is outcome-neutral by design.** The edge means "this revision *arose from*
this discussion" — true whether the RFC concluded yes or no. An earlier candidate
`enacts` / "enacted by" was rejected: it smuggles a yes-outcome into the label,
so an RFC that resolved *against* a proposal (and was formalised by a REV
recording the "no") would falsely render "enacted by". That is the same
governance-position leak banned from `status` (see §2), sneaking in via the
relation. `originates_from` / "precursor of" stays honest under rejection and
matches the scope's own word ("the precursor discussion a REV may later
formalise"). Outcome (yes/no) lives in the RFC **body prose** and in what the REV
records — never structurally asserted by the edge. Not `spawns` (taken:
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

## §2 Lifecycle (OQ-4 — LOCKED)

**Minimal status machine**, governance-neutral terminals:

```
open (default) → resolved | withdrawn
```

- `open` — deliberation live.
- `resolved` — discussion concluded. **Outcome-blind**: means "concluded," not
  "concluded yes". A discussion may resolve *against*; the yes/no outcome lives in
  the body prose, never in the status.
- `withdrawn` — abandoned without conclusion.

Rejected alternatives:
- **Stateless** (REC-path, `meta::read_id`): leanest, but no edge expresses
  *withdrawn* or *resolved-without-a-REV*; those are pure status facts. The
  live/concluded/dropped distinction is real catalog signal no relation captures.
- **`accepted` / `declined` terminals**: forbidden — they make the RFC assert a
  governance position. `resolved` stays outcome-blind precisely to avoid this.
- **`draft` / `superseded`**: speculative; omit. Supersession, if ever needed, is
  an edge not a state.

Reader path: an authored `status` field ⇒ RFC does **not** use REC's status-less
`meta::read_id` scan; it follows the status-bearing read path (cf. ADR/slice).
Transition verb `doctrine rfc status <id> <state>` (status moves via CLI, not
hand-edit — boot rule).

**Status ⊥ `originates_from`.** A REV-authored edge never flips RFC status (that
would reintroduce the target-mutation avoided in §1 Decision 2). An RFC may be
`resolved` then later preceded by a REV, or precede a REV while still `open`. The
two axes are independent.

## §3 Remaining open questions (driving next)

- OQ-1: **Naming** — `.doctrine/rfc/` (singular, peer convention) vs user's
  `.doctrine/rfcs/`.
- OQ-5: **ADR scope** — confirm the governing ADR + exactly what it asserts.
- OQ-6: **Catalog / boot visibility** — surface in catalog, absent from
  governance sections, no leaked governance position.
