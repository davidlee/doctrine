# IMP-398: Knowledge record discoverability

## Motivation

The `/knowledge` skill creates knowledge records (assumptions, decisions,
questions, constraints, evidence, hypotheses, concepts) as part of the design
workflow. These records link to slices, RFCs, ADRs, specs, and each other — but
they are hard to *find* after creation. An agent asking "what do we assume about
this slice?" or "what decisions shaped this spec?" has no direct CLI path; they
must know the ids or grep raw files.

The aim is to make knowledge records as legible as the entities they annotate.

## Status: partially sliced (2026-08-05)

The largest gap — **reading an entity together with the content of the knowledge
records that shape it** — is now **`SL-246`** (*Entity reads carry their
knowledge records*). Its scope: a relation-keyed, one-hop composed read with a
`skip | facets | full` verbosity dial, motivated by `SL-244`'s 3,456-line
`design.md` citing 20 `DEC` records whose content nothing renders alongside it.

The **prose-cited-but-unlinked** problem discovered while shaping that slice is
now a fourth lint canary on **`IDE-009`** — `SL-244`'s design cites ~10 `DEC`
records it holds no edge to, so a relation-keyed read misses them. `SL-246`
deliberately reads edges and does not parse prose; `IDE-009` raises the
divergence instead.

What remains on this card is below, shaped in the same 2026-08-05 pass.

## Remaining candidates

### A. `knowledge list` has no kind filter

Rows carry `record_kind`; there is no flag. You cannot list only decisions or
only assumptions.

**Shape — settled, small.** `backlog list --kind <KIND>` is the exact structural
precedent: a cross-kind survey verb over a kind family with a value-enum flag.
`knowledge list` is the same shape over `RecordKind` and simply never got it. The
seam exists — `knowledge.rs:1644` already runs a per-item pre-pass before the
shared `listing::retain` *because* "the status-keyed `retain` closure cannot see
the kind" — so the predicate has the kind in hand. `RecordKind::ALL` supplies the
vocabulary and `knowledge new <KIND>` already exposes it as a value enum
(`assumption, decision, question, constraint, evidence, hypothesis, concept`).

Arg + predicate + tests. No new machinery.

**Open:** flag name. `--kind` matches `backlog list` and the `record_kind` field;
`--type` was this card's original wording. Note `memory retrieve --type` uses the
other word for a different concept (memory *type*, not a kind family). Recommend
`--kind` for CLI-grammar consistency (`SPEC-013`).

**Note:** `IDE-009`'s C3 leg proposes a kind-aware `listing::retain` closure that
would kill the `knowledge.rs:1644` duplication. If both land, land C3 first or
this filter will be written against a seam that is about to move.

### B. No inbound reciprocity on `show` (with `ISS-306`)

**Confirmed corpus-wide, not knowledge-specific.** `using-doctrine.md` states
reciprocity is derived and rendered both directions; no kind does it.
`doctrine slice show SL-244` renders `governed_by` and `references(*)` — all
outbound — while sixteen knowledge records pointing *at* it appear nowhere.
`ISS-306` scopes this to `knowledge show/inspect`; the identical hole is on
`spec` / `slice` / `adr` / `rfc` `show`. One fix at the shared render seam.

**Shape — the derivation is done; the curation is the work.** `doctrine relation
list --target <ref>` already computes inbound correctly. It lives in a third
command family reachable from neither the entity nor the record, so nothing
routes you to it from where the question arises.

The real problem is **noise**. `SL-244` carries 28 inbound edges, of which **11
are `references(originates_from)` from backlog items** — harvest exhaust, not
content. An uncurated inbound render makes the common read worse, not better. So
this needs a label allow-list, a grouping, or an opt-in flag before it is an
improvement at all.

`SL-246` will settle a version of this question for its own render (its `OQ-3`);
check what it concluded before designing this one.

### C. `knowledge list --related-to <ref>`

No way to ask "what knowledge records are attached to `SL-226`?" as a *list*.

**Shape.** The list-shaped sibling of **B**, sharing one reverse-scan derivation
(relations are outbound-only — `ADR-004` — so this cannot walk the ref's own
edges; the earlier sketch on this card said otherwise and was wrong). Cheap once
B exists; do not build the derivation twice. Its own open question — one-hop or
transitive — is really **D**'s question.

### D. Recursive knowledge view from a non-knowledge entity

From a `SPEC` / `RFC` / `ADR` / `SL`, show all knowledge records reachable
*without another non-knowledge entity separating them*. Speculative; wanted
later.

**Shape — this is a typed traversal, and it reframes the graph gaps.** The bound
is a type predicate ("halt at any non-knowledge node"), not a hop count. Walk out
from the focus, continue through knowledge→knowledge edges (`supersedes`,
`supports`, `refines`), stop at the first non-knowledge node. Neither `--depth`
(a hop count) nor `--kind` (a global node filter) can express it.

That is why the three graph gaps below are **enablers of D**, not independent
asks — each is something D needs and none is separately motivated. Recorded here
so a later reader does not re-derive them as standalone work:

- **`KN` collective kind prefix.** Including all seven knowledge subtypes needs
  seven `--kind` flags. ⚠ Lands in a known minefield:
  `mem.pattern.doctrine.record-kind-touch-sites` records ~17 sites hardcoding
  record-kind prefix literals instead of reading `kinds::RECORD`, one a
  `debug_assert!` that panics every debug-build corpus scan if a kind is added
  without a matching scan arm. **No drift canary — only grep finds them.**
- **`graph --label` takes exactly one label.** Cannot include `shapes`,
  `governed_by`, `references(concerns)`, and `related` in one graph. Making it
  repeatable is trivial in itself.
- **`graph --kind` filters the focus out.** `doctrine graph SL-226 --kind ASM
  --kind DEC` fails with *"SL-226 excluded by `--kind`"*. The pipeline
  (`src/commands/graph.rs:114-126`) applies `filter_kinds` to the whole graph
  *before* `neighbourhood(focus, depth)`, so the focus is dropped and the
  neighbourhood loses its anchor; the check at `:94-112` exists to fail loudly
  rather than print an empty graph. ⚠ **Changing this is a spec change, not a bug
  fix** — `SPEC-027` lists that error as intended behaviour ("Report focus
  exclusion as a named, self-describing error — focus absent, focus *filtered out
  by `--kind`* …"). Exempting the focus makes that clause wrong rather than
  merely superseded, so it needs a **`REV` against `SPEC-027`**.

Also note **D and `SL-246` are the same capability at different reach** — both
are "the knowledge closure of this entity", one emitted as content and one as a
graph. `SL-246`'s objective 3 keeps its inbound derivation factored so D extends
it rather than replaces it. Check that seam before starting D.

### E. Also consider (unshaped)

- `doctrine search` — should knowledge records surface in results?
- `doctrine slice plan` / phase sheets — should linked knowledge be visible
  during planning?
- `/design` skill — should it auto-surface existing knowledge when resuming a
  design run?

## Worked instance (2026-08-05)

`QUE-206` (*is the interview substrate separable from slice design*) was captured
with `shapes` edges onto `PRD-019` and `SL-244`. What each surface shows:

| view | result |
|---|---|
| `knowledge show QUE-206` | `shapes: [PRD-019, SL-244]` — correct |
| `spec show PRD-019` | **nothing** — no sign `QUE-206` exists |
| `relation list --target PRD-019` | both inbound edges, correctly |

The capability exists; the discoverability does not. The entity is where the
question gets asked ("what is unresolved about `PRD-019`?") and it is the side
with no answer; the side that renders correctly is the side you already had the
id for.

## Scope

The **discoverability** surface — filtering, listing, and graph projection — for
knowledge records in relation to the entities they annotate, across all entity
kinds that carry knowledge relationships. The *composed content read* half has
left this card for `SL-246`.

## Related

- `SL-246`: the sliced half — entity reads carry their knowledge records
- `ISS-306`: `knowledge show/inspect` render no inbound reciprocity — same defect
  as **B**, scoped one entity-kind narrower; fix together
- `IDE-009`: knowledge lint verb — now also holds the prose-cited-but-unlinked
  canary, and a C3 leg that moves the seam **A** would be written against
- `IMP-120`: transitive impact query on relation graph — likely shares **D**'s
  traversal
- `QUE-206`: the worked instance above
