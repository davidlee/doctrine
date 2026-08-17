# EVD-026: SL-258 map exercises both edge kinds

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## What was measured

`SL-258`'s design run (`dr-01a00e5b`, revision 5, stage `exploring`), read off the
runtime snapshot at `.doctrine/state/slice/258/design.toml` on 2026-08-17:

| measure | SL-258 | RFC-026 E8.3 (SL-243) |
|---|---|---|
| nodes | 14 | 9 |
| `parent` edges | 10 | **0** |
| `needs` edges | 25, over 13 of 14 nodes | **0** |
| `needs_in_degree` | non-zero throughout | 0 throughout |
| revisions | 5 | 17 |
| lifecycles | 12 open, 2 pruned | — |

Counted over the live map only (`[map.inquiry.nodes.inq-*]`). The raw file yields
double the parent count because `[declarations.declaration.covered.nodes.covered.*]`
persists a `ContentCoverage<NodeMaterial>` copy of every node — a naive
`grep -c parent` over the whole file over-reports by ~2x.

## Why it bears on QUE-218

`QUE-218` (*Does the inquiry map earn a semantic tier?*) rests on E8.3's zero: both
edge kinds shipped with full render support and a competent agent used neither
across 17 revisions. E8.3's own update block (2026-08-15) already weakened the
inference — `ISS-360` means the instrument cannot separate *did not use* from
*could not use by the obvious route* — and `CHR-065` carries the re-measurement.

This is that re-measurement, unsolicited. The edges are not a star on the root:
`inq-9` needs `inq-1/2/5/7/8`, `inq-6` needs `inq-5/7/8`, `inq-4` needs
`inq-2/5/8` — multi-target, non-root, with diamonds. So the topology is not the
shape `ISS-360`'s workaround would produce by accident.

It was built **with `ISS-360` still open**, meaning the per-level revision cost was
paid deliberately. That makes the evidence stronger than unobstructed use would
have been: the structure was worth paying for.

## The counter-finding in the same run

The map is *used* and not *consumed*. At this edge density:

    frontier                    (empty)
    open_outside_frontier = 12
    blocked = 13

`InquiryMap::is_blocked` excludes blocked nodes from frontier eligibility, so at
realistic density almost everything is ineligible and the frontier empties. Twelve
open questions and no derived answer to *which one next*.

This relocates the fitness problem. It is not that the semantic tier is unused —
it is that the traversal/reporting side does not survive the tier being used. That
promotes `IMP-389` (derive the next traversal candidate) and `IMP-390` (envelope
reports state, not what to do next) above the map-deepening batch
(`IMP-386`/`387`/`388`) that `QUE-218` gates.

## Limits

- **n=1**, single agent, single slice — the same single-arm limit RFC-026 caveat 11
  states for E8 itself. Two n=1 runs disagreeing is not a trend.
- Edge **presence** was measured, not edge **quality**. Whether these 25 `needs`
  edges encode real dependency or defensive over-linking is unassessed, and the
  empty frontier is consistent with either.
- The comparison inherits E8.3's confound: different slice, different subject
  matter, different agent instance.
