# ISS-467: Facets withholds the Argument tier with no disclosure

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Surfaced at `SL-246`'s reconciliation audit as `RV-372` `F-6`, and it is the
mechanism behind the human's `VH-1` complaint that *"the omitted facets defy
understanding"* (`RV-372` `F-5`).

## The defect

`DEC-150` splits each knowledge record's facet into a **Deciding** tier (what
rules, and what would change whether the ruling still stands) and an
**Argument** tier (how the ruling was reached). `--knowledge facets` renders the
Deciding tier and drops the Argument tier.

It drops it **silently**. Nothing in the rendering names the tier, says a
projection happened, or distinguishes:

| the reader sees | could mean |
|---|---|
| a `DEC` with `context` / `choice` / `rationale` and nothing else | `alternatives` and `consequences` are populated and were withheld |
| the same | `alternatives` and `consequences` are empty |
| the same | this kind has no Argument-tier fields at all |

Three different states, one rendering. A reader cannot tell which they are
looking at without re-reading at `full`, which is the cost the level exists to
avoid.

## Why this is a conformance defect and not just an ergonomic one

`STD-003` (*No silent skip — a degraded read is disclosed*) governs degraded
reads generally, not only erroring ones. A deliberate projection that does not
announce itself is the failure mode the standard names.

`SL-246` demonstrably knew this: it built an explicit three-marker vocabulary
for the *empty* cases — by-design, unfilled, unreadable (`DEC-149`, `X5`, `I6`)
— and got all three onto both arms at both levels, which was the hardest thing
in the slice (`RV-370` `F-23`). The **withheld** case has no marker and was
never modelled. The gap is an omission in the design, not a bug in the build.

## Shape of a fix

Not prescribed, but the cheap end exists: the tier is already a column on the
authored `FacetFieldRow` table (`src/knowledge.rs`), so the set of withheld keys
is derivable at the render site without a second per-kind match — the same
property that made `C4` achievable. A one-line trailer naming the withheld keys
(text arm) plus a sibling key (JSON arm) would satisfy `D1`'s both-arms binding.

Note the interaction with `ISS-469`'s sibling concern: whatever is added must not
re-open `C2` (`knowledge show` stays byte-identical, its golden unedited).

Related: `RV-372` `F-5`, `F-6`; `SL-246`; `DEC-149`, `DEC-150`; `STD-003`;
`IMP-465` (the wider product pass); `IMP-403` (unfilled facets).
