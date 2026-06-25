# IMP-170: UX review of relation-authoring CLI surfaces (coverage + consistency)

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Why

Spawned by SL-153 (CLI verbs for the last hand-edit-only spec-internal edges).
SL-153 closes `descends_from`/`parent`/`interactions` but surfaced a wider gap:
the relation-authoring CLI surface is not uniformly modelled.

Concrete known instance: product `parent` (PRD→PRD) is authorable (SL-065 added
`Spec.parent` + render + `build_registry` `on_product` validation) and SL-153 lets
`spec edit --parent` set it, but `RELATION_RULES` declares **no** PRD-parent row and
the product template carries no `parent` example. The table under-declares reality.

## Scope

UX/consistency review across **all** CLI surfaces where a relation edge could or
should be authored:

- Audit `RELATION_RULES` against what the code actually accepts/emits/validates
  (table honesty) — add the missing PRD-parent row + the VT-1 golden-order update.
- Check every relation label has a coherent author/remove verb and that flag/arg
  shapes are consistent across `link`, `spec edit`, `spec interactions`,
  `spec req`, `review`, `rec`, `revision`, `concept-map`.
- Surface any remaining hand-edit-only or inconsistent edge and close it.

## Preflight Findings (2026-06-26)

### Coverage — all 23 edges have CLI verbs

SL-153 closed the last 3 hand-edit-only edges (`descends_from`, `parent`,
`interactions`). Every `RELATION_RULES` row now has a verb:

| Label | Source(s) | Verb |
|---|---|---|
| references (implements/scoped_from/concerns) | SL, RFC, backlog | `link --role` |
| supersedes | SL | `link supersedes` |
| supersedes | GOV | `supersede` |
| supersedes | RECORD | (LifecycleOnly, no CLI verb needed) |
| descends_from | SPEC | `spec edit --descends-from` |
| parent | SPEC | `spec edit --parent` |
| members | PRD, SPEC | `spec req add` |
| interactions | SPEC | `spec interactions add/remove` |
| contextualizes | CM | `concept-map add` |
| shapes | RECORD | `link shapes` |
| spawns | RECORD | `link spawns` |
| governed_by | 13 kinds | `link governed_by` |
| consumes | PRD | `link consumes` |
| slices | backlog | `link slices` |
| related | GOV / work | `link related` |
| reviews | RV | `review new --target` |
| owning_slice | REC | `rec new --owning-slice` |
| drift | backlog | `link drift` |
| decision_ref | REC | `rec new --decision` |
| revises | REV | `revision change add --target` |
| originates_from | REV | `revision new --originates-from` |

**Verdict: no remaining hand-edit-only edges.**

### Consistency gaps (actionable)

**C1 — `RELATION_RULES` parent row missing `PRD` source**
(`src/relation.rs` ~line 410)

```rust
RelationRule {
    sources: &[SPEC],        // ← missing PRD
    label: RelationLabel::Parent,
```

`spec edit --parent` accepts PRD→PRD (via inline fallback in `run_edit`),
`registry.rs` validates it as a `ParentEdge`, and `Spec.parent` has no subtype
restriction. The table under-declares reality. Fix: add `PRD` to `sources`.
(This is the original R2 from SL-153 §8.)

**C2 — Template staleness (3 items)**

1. `install/templates/spec-tech.toml` line 19: parent comment says "tech-only"
   — now subtype-aware (SL-153).
2. `install/templates/spec-product.toml`: no `parent` example at all.
3. `install/templates/interactions.toml` line 3: "hand-authored in v1 (no verb)"
   — stale; verbs exist since SL-153.

**C3 — Stale doc comment in `src/spec.rs`** (~line 482)

```rust
/// Single decomposition parent (`SPEC-NNN`). Tech-only, single-valued
/// outbound; the reciprocal children view is derived, never stored (§5.2).
#[serde(default)]
pub(crate) parent: Option<String>,
```

"Tech-only" is wrong — product specs use it too (SL-065, SL-153).

### Argument shape survey — no issues

All verb shapes are justified by their entity models:

- `link` positional SOURCE LABEL TARGET — generic tier-1 edge authoring.
- `spec edit` positional spec_ref + flags (`--parent`, `--descends-from`) —
  multi-field set/clear in one pass; flags prevent positional ambiguity.
- `spec interactions add` positional spec_ref target + `--type` flag —
  interactions use free-text edge kinds, not declared labels; `--type` as
  flag matches the `Interaction` struct shape.
- `spec req add` positional SPEC_REF + `--kind`/`--label` flags — label
  is auto-derived, so a flag is correct.
- `review new` `--target` flag, `rec new` `--owning-slice`/`--decision` flags,
  `revision change add` positional REFERENCE + `--target` flag — creation-time
  verbs where the edge is one of several orthogonal options.
- `supersede` positional NEW OLD — specialised ADR-only transaction.
- `concept-map add` positional ID SOURCE REL TARGET — graph-DSL shape.

Verdict: the diversity is deliberate and coherent, not accidental drift.

### Recommended actions from preflight

1. Add `PRD` to `RELATION_RULES` parent row `sources` (C1).
2. Refresh 3 template comments (C2).
3. Fix `Spec.parent` doc comment (C3).

## UX Surface Audit — where relations exist but aren’t shown (2026-06-26)

Audited both surfaces per entity kind: `show` (per-kind render) and `inspect`
(cross-kind, outbound + inbound). The `inspect` surface is generally complete;
`show` has systematic gaps.

### G1 — `rfc show` drops `references(concerns)` edges

RFCs author `references(concerns)` via tier-1 `[[relation]]` rows (dozens of
them across RFC-001 through RFC-005). `inspect RFC-004` correctly shows
`references(concerns): IMP-012, IMP-152, ADR-007, …`. But `rfc show RFC-004`
shows only `tags` under "relationships" — all `references` edges are silent.

Root cause: `governance::run_show` reads ALL tier-1 edges via `tier1_edges()`
but then **only extracts `Supersedes` and `Related`** before passing to
`format_show`. RFC-authored `references` edges are read then discarded.
`format_show` itself only knows about `supersedes/superseded_by/related/tags`.

Fix: `run_show` must pass `references` edges through; `format_show` needs a
`references` axis (with role annotation).

### G2 — `revision show` drops `revises` and `originates_from`

`revision show REV-001` shows `status=done · approval=approved` and the body —
that’s it. No `revises` edges (the whole point of a REV), no `originates_from`.
`inspect REV-001` shows `revises: SPEC-020, SPEC-020, SPEC-020, SPEC-020`
(duplicates — see G4).

Root cause: `revision::format_show` only renders id/title/status/approval/body.
It never reads `[[change]]` rows or tier-1 `[[relation]]` rows.

Fix: add a relationship block showing `revises` (deduplicated) and
`originates_from`.

### G3 — `revision::relation_edges` drops `originates_from` (affects `inspect` too)

`revision::relation_edges` only iterates `[[change]]` rows for `revises` edges.
It never reads tier-1 `[[relation]]` rows, so `originates_from` — authored by
`revision new --originates-from RFC-NNN` as a `[[relation]]` row — never
appears in `inspect REV-NNN` either.

Fix: append `tier1_edges()` results like other kinds do.

### G4 — `inspect REV-NNN` shows duplicate `revises` targets

Each `[[change]]` row targeting the same spec produces a separate edge.
`REV-001` has 4 rows targeting `SPEC-020` → `inspect` shows `revises: SPEC-020,
SPEC-020, SPEC-020, SPEC-020`. The `spec.rs` members array preserves authored
order within a label, so duplicates are intentional at the storage level, but
the render should deduplicate (a REV revises each target once).

### G5 — Governance `supersedes` invisible: no edges authored, template wrong

No governance entity in the corpus has a `[[relation]] label = "supersedes"`
row. The `supersede` verb exists but was never used to record supersession
edges. Both ADR-004 and ADR-012 have empty `superseded_by = []`.

Compounding: the ADR template comment instructs users to use `doctrine link`
for supersedes — but governance `supersedes` has `LinkPolicy::LifecycleOnly`,
so `link` **refuses** it. The template directs users to a command that fails.

Fix: correct the template comment to reference `doctrine supersede`, then run
`supersede ADR-012 ADR-004` to author the canonical edge.

### G6 — `spec show` drops tier-1 edges (governed_by, consumes)

`spec show SPEC-002` shows descends_from/parent in the header and
members/interactions in the body, but governed_by/consumes/related (authored
via `link` as `[[relation]]` rows) never appear. The `render` function includes
them only as prose in the markdown body, not as structured relationship lines.

Fix: the header should include an optional `governed_by`/`consumes` line when
populated (like the `slice show` output).

### G7 — `revision::relation_edges` doesn’t deduplicate targets

Same root cause as G4. The accessor emits one edge per `[[change]]` row,
regardless of target identity. Since `inspect` groups by `(label, role)` and
concatenates targets, duplicates survive.

Fix: deduplicate in `relation_edges` or in the `render_outbound` grouping.

### Surface summary table

| Entity | `show` surface | `inspect` surface |
|---|---|---|
| Slice | governed_by, references(role), related ✓ | full outbound+inbound ✓ |
| Spec (tech) | descends_from, parent, members, interactions ✓; **governed_by/consumes ✗** (G6) | full outbound+inbound ✓ |
| Spec (product) | parent, members ✓; **governed_by/consumes ✗** (G6) | full outbound+inbound ✓ |
| ADR/POL/STD | supersedes, superseded_by, related, tags ✓ | full outbound+inbound ✓ |
| RFC | tags ✓; **references(concerns) ✗** (G1) | full outbound+inbound ✓ |
| Revision | id/title/status/body ✓; **revises, originates_from ✗** (G2) | revises ✓ (duplicated G4); **originates_from ✗** (G3) |
| Review | reviews(→target), findings ✓ | full outbound+inbound ✓ |
| REC | owning_slice ✓; **decision_ref ✗** (not yet authored) | full outbound+inbound ✓ |
| Backlog | slices, governed_by, references(role), related, drift ✓ | full outbound+inbound ✓ |
| Knowledge record | tier-1 edges ✓ | full outbound+inbound ✓ |

## List View Audit — what list columns omit (2026-06-26)

List views are the scanning surface. Most omit relation edges entirely. A few
surface one derived column. The pattern is inconsistent.

### L1 — `revision list` doesn't exist

`doctrine revision` has `new`, `show`, `status`, `change`, `approve`, `apply`,
`paths` — but no `list`. Every other entity kind has one. No way to survey
revisions at a glance; you must know their IDs.

### L2 — `slice list` shows no relation columns

`slice list` shows `id | status | phases | title`. No spec (`implements`), no
governance (`governed_by`), no backlog origin (`scoped_from`). A reader
scanning the slice list can't see which spec a slice targets.

### L3 — `spec list` has `#members` but no other relation columns

`spec list` shows `id | status | title | #members`. The members count is a
derived column (surfaced from `members.toml`). No `descends_from`, `parent`,
`#interactions`, `governed_by`. Product specs don't show `consumes`.

### L4 — `adr list` / `policy list` / `rfc list` show no relation columns

Pure `id | status | title`. No `supersedes`, `superseded_by`, `related`,
`#governs`, `references`. An ADR reader can't see which ADR supersedes which
from the list.

### L5 — `backlog list` shows `tags` but no relation columns

`backlog list` shows `id | kind | status | tags | title`. Tags are free-text
classification, not relations. No `slices`, `governed_by`, `references(concerns)`
— a reader can't see which slice works on a backlog item.

### L6 — `review list` shows `target` but from typed field, not tier-1

`review list` shows `id | status | facet | target | title`. The `target` column
comes from the typed `[review].target` field, NOT from `[[relation]]`. Works
correctly, but by coincidence rather than design.

### L7 — `rec list` shows `owning` but not `decision_ref`

`rec list` shows `id | move | owning | title`. The `owning_slice` is surfaced,
but `decision_ref` is not.

### List coverage matrix

| Kind | List columns with relations | Missing |
|---|---|---|
| Slice | — | implements, governed_by, scoped_from |
| Spec | #members ✓ | descends_from, parent, #interactions, governed_by, consumes |
| ADR/POL/STD | — | supersedes, superseded_by, #governs, related |
| RFC | — | references(concerns), related |
| Revision | **no list verb** | all |
| Review | target ✓ (typed) | — |
| REC | owning ✓ | decision_ref |
| Backlog | tags (not relations) | slices, governed_by, references |

## Links

- Spawned from SL-153 design (§8 R2 follow-up).
