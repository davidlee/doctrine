---
seq: 0028
scope: codebase
target: `doctrine inspect` render (src/relation_graph.rs render_*, inspect_value)
confidence: med
reversible: yes (read-only analysis; nothing authored)
---
## What
`doctrine inspect <ID>` — the primary CLI view of an entity's graph neighbourhood —
renders **bare ids with no titles**, on *both* surfaces:
- human: `inbound:\n  governs: SL-058, SL-060, SL-065, SL-092, …` (verified on
  ADR-004).
- `--json`: `{"label":"governed_by","targets":["SL-058","SL-060",…]}` — target is a
  plain string id, no title.

So a reader inspecting ADR-004 sees that eight slices govern-link to it, but not
*what any of them are* — understanding the neighbourhood takes one follow-up
`show`/`inspect` per id. The titles **exist** and are already computed elsewhere: the
priority surface captures per-node titles (`NodeAttr`, used in `survey`/`next`
output), and the web map renders titled nodes. Only the `inspect` render drops them.

This is a consumption-surface gap (the 0014 thesis again): `inspect` is *the* way a
human or agent reads the graph from the CLI, and it's optimised for navigation (ids)
over comprehension (titles). For "indispensable to teams," the neighbourhood view
should be legible without N lookups — "governs: SL-095 (record-supersede migration),
SL-097 (…)" instead of "governs: SL-095, SL-097."

Likely-intentional constraint to respect: the relation render is **byte-identical /
determinism-gated** (run_inspect cites VT-4 byte-identical surfaces). Titles are
stable strings, so adding them stays deterministic — but it changes the output and
the `--json` shape, so it touches golden tests and any `--json` consumer that
expects string targets.

## Options
1. **Human render: `ID (title)`; `--json`: additive.** Show titles inline in the
   human view; in `--json`, either add a parallel `titles` map or make targets
   `{id, title}` objects. Tradeoff: legible human view; the json change is the only
   compatibility question (additive map = non-breaking; object targets = breaking).
2. **Human-only titles; leave `--json` bare.** Add titles to the human render only;
   keep json as string ids (machines resolve titles themselves). Tradeoff: zero json
   compatibility risk; smallest; humans get the win, machine consumers unaffected.
3. **Leave as-is.** Tradeoff: zero work; inspect stays id-only, comprehension stays a
   multi-lookup chore — at odds with the graph's whole value proposition.

## Recommendation
Option 2 (human render `ID (title)`, leave `--json` bare) as the safe default —
it delivers the comprehension win exactly where it matters (the human reading the
graph) with no `--json` compatibility risk, and titles are already computed so it's
a render-layer change. If a structured consumer wants titles, do the additive-map
half of Option 1 later (non-breaking). Update the VT-4 golden to the titled form;
determinism holds since titles are stable. This is the cheapest legibility upgrade to
the most-used graph surface.

Decisions deferred to YOU:
- (a) **human-only (2) vs human+json (1)** — is `--json` meant to stay id-only (let
  machines resolve), or carry titles (additive map vs breaking object change)?
- (b) the **VT-4 byte-identical** constraint — confirm titled output is acceptable
  (golden updated), or is there a reason the relation render must stay id-only?
- (c) title source — reuse the priority `NodeAttr` titles (already computed in the
  shared scan) to avoid a second read.

## Next doctrine move
```
# confirm both surfaces are id-only + titles exist elsewhere (read-only):
doctrine inspect ADR-004            # human: bare ids
doctrine inspect ADR-004 --json     # targets: string ids
grep -n 'title' src/priority/surface.rs   # NodeAttr titles already computed

# capture (NOT executed — fence forbids backlog transition):
doctrine backlog new improvement "inspect render: show target titles (ID (title)) \
  in the human view — reuse priority NodeAttr titles already computed in the shared \
  scan; --json additive or left bare. Legibility on the primary CLI graph surface" \
  --tag area:relations --tag area:cli
```
(Verbs described, NOT executed.)

## Illustration (optional)
None — a render-layer change reusing already-computed titles; the open question is
the `--json` contract (a), which a speculative diff would prejudge.
