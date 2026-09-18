# IMP-459: Corpus scan fully parses every knowledge record and discards all but its edges

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## What happens

`catalog::scan::scan_entities` calls `outbound_for` once per entity
(`src/catalog/scan.rs:192`). For a knowledge-record prefix that dispatches to
`knowledge::relation_edges` (`:65`), which delegates to `read_record`
(`src/knowledge.rs:1637`).

`read_record` does the full job: reads `record-NNN.toml`, parses it, validates
it into a `KnowledgeRecord`, collects its tier-1 edges — and then reads
`record-NNN.md` **unconditionally**, erroring if the file is absent.
`relation_edges` returns `record.tier1` and drops the rest on the floor.

So every corpus scan reads and parses **360 records**, including roughly
**964 KB of prose bodies**, to obtain their relation edges. The facet is parsed
and discarded; the body is read and discarded.

Every command that scans the corpus pays it: `inspect`, `validate`, `survey`,
the priority graph, `status`, `search`.

## What it actually costs

Less than the numbers suggest, which is why this is an improvement and not an
issue. `doctrine inspect SL-244` — a full corpus scan plus a relation render
plus the priority actionability block — runs in about 220 ms end to end. The
discarded prose is a fraction of that.

Measured before acting. The point is that the waste is unnecessary, not that it
is currently painful.

## The fix, and why it is already half-designed

`read_record`'s all-or-nothing shape is the cause: there is one read path and it
always takes the body. A caller that wants only edges, or only the facet, has no
way to say so.

`SL-246` needs exactly that seam for a different reason — its `facets`
verbosity level renders a record's deciding fields and never touches the prose,
so paying for the body there would be waste it can see. `DEC-146` flagged the
same sharp edge in its consequences and deferred it as a design detail.

So the accessor `SL-246` adds is the one this item would ride. Whoever takes
this should read that design first rather than inventing a second read path —
`AGENTS.md` forbids the parallel implementation, and a second one here is how
the two drift.

Not folded into `SL-246` because fixing the scan is not that slice's job and
would widen a behaviour-preservation gate it deliberately keeps narrow.

## Related

- `SL-246` — adds the per-id accessor this would use; `DEC-146` is where the
  sharp edge was first named.
- `STD-003` — whatever replaces the unconditional `.md` read must keep the
  missing-file case disclosed, not laundered into an empty body.
