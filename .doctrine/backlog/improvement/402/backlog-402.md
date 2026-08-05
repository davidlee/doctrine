# IMP-402: Inquiry fragment omits the map's edge kinds

## The gap

`install/design-prompts/inquiry.md` is delivered every turn of **both**
`exploring` and `inquiring` — it is the fragment an agent building the inquiry
map actually reads. It never mentions `parent`, `needs`, decomposition,
dependency, or the graph. Its nearest bullet polices *what deserves a node* ("a
question worth a decision, not a task to tick off") and is silent on *how nodes
relate*.

The model underneath is a graph and has been since DEC-061:

- `InquiryNode.parent: Option<DesignId>` — the primary-parent tree, the
  readable decomposition (`src/design_run/inquiry.rs:210`).
- `InquiryNode.needs: BTreeSet<DesignId>` — sparse non-tree dependency edges
  (`inquiry.rs:211`).
- Both acyclic, both checked — `Refusal::CyclicEdge` (`refusal.rs:183`).
- `blocked` derived and never stored (DEC-060) — `InquiryMap::is_blocked`
  answers from the `needs` edges and the lifecycle of their targets
  (`inquiry.rs:437`).
- Already rendered: `needs_in_degree` on frontier rows and blockers, and the
  `Kinship` proximity ladder (`parent` / `needs-neighbour` /
  `grandparent-or-nibling`) ranking what a turn sees
  (`render/envelope.rs:168,183,466-493`).

So the machine ranks by kinship and refuses cycles, and the prose that teaches
the agent describes none of it. The predictable outcome is a sea of individual
nodes where a tree with a few dependency edges was available — which costs most
when the map is rendered for a human to understand.

## Compounding: the worked example teaches half of it

`DECLARATION_EXAMPLE` (`render/envelope.rs:67-71`) is the copyable one-liner in
the no-drop set of every envelope. It carries `"parent":"inq-1"` and **no**
`needs`. An agent working by imitation therefore builds a tree and never learns
the second edge kind exists.

## Two unrelated things share the word "blocking"

Worth stating in whatever prose lands, because the collision is live:

- **needs-derived** `is_blocked` — structural, computed, above.
- **`AgentAct::BlockingSetDeclared { blocking }`** — an agent-declared list of
  node ids validated only against the covered node set
  (`admission.rs:169-182`), which is what clears `initial-concerns-recorded` at
  the `exploring → inquiring` edge.

The fragment's existing "Disposition blocking inquiries explicitly" is the
second sense. A reader has no way to tell.

## Proposed shape

Prose only. Three additions in the asset's own voice:

1. In the loop or Craft — a map is a decomposition, not a list: a parent
   wherever one exists, and a `needs` edge only where one question genuinely
   cannot be answered before another. Both sparse on purpose.
2. In "What the machine will reject" — both edge kinds are acyclic and checked;
   an edge closing a cycle is refused, not silently dropped.
3. Also there — blocked is derived from `needs`, never declared: a node is
   blocked while anything it needs is open or deferred, and resolving that node
   unblocks it with no second act.

Optionally `needs` into `DECLARATION_EXAMPLE`, which costs bytes against a
compile-time-asserted bound (`ENVELOPE_DECLARATION_EXAMPLE_BYTES`) — measure
before assuming it fits.

## Resolution — done 2026-08-05

All four landed. Two things the measurement changed:

**The byte caveat above was unfounded.** The bound is 1024 and the example was
288; with `needs` it is 306. There was never a budget question.

**The example change is not optional — it is the load-bearing half.** A caller
that declares the fragment digest has the fragment body *elided*, which is the
fragment receipt working as designed. `DECLARATION_EXAMPLE` is in the no-drop
set and is never elided. So on any warm turn the prose is gone and the example
is the only surface naming the map's shape: showing `parent` alone taught a warm
run that the map is a tree. That reasoning is now in the constant's doc comment,
so the 18 bytes cannot be trimmed back out as a saving by someone reading only
the byte budget.

Proved end to end in a throwaway installed project: the three prose additions
ship from the embed, and the shipped example copied **verbatim** is accepted on
the wire and immediately renders the derived consequence —
`blockers / inq-2 — needs inq-1`. Prose teaches it, the example shows it, the
wire takes it, the envelope shows what it did.

**Not touched:** `.doctrine/backlog/chore/049/exercise/moderator-sheet.md` quotes
the pre-change digest `inquiry@98fffa6b…`. That is a transcript of what an
exercise run observed at the time, not an expectation to match — updating it
would falsify the record. Leave it stale.

## Cost and constraints

Cheap. `design-prompts/inquiry.md` is `customization = "fixed"` in
`publication/manifest.toml:229-236`; no test golden pins the fragment digest, so
the edit is prose plus a re-embed. The asset is SL-233's, not SL-244's —
deliberately kept out of SL-244 PHASE-06, which was one human attestation from
done when this surfaced.

Raised by the user while reading the SL-244 PHASE-06 `VH-1` acceptance artefact.
