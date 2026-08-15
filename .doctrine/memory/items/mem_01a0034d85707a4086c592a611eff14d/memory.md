# Design-run cluster: entry point

Start here for work on the **managed design run** — the engine behind
`doctrine design start|apply|resume|materialise`, governed by **SPEC-029**
(*Design run engine*) and built by **SL-233**.

This is a map, not a status board. Live state is always queryable:

```
doctrine backlog list -t cluster:design-run --by sequence
doctrine next                       # gated items are absent by design
doctrine blockers <ID>              # names the gate
```

## Where the evidence lives, and where it does not

**RFC-026** (*Design review response effectiveness*) is the **evidence home** and
should stay so. Its items **E8** (the shipped engine measured by using it), **E8.5**
(capability delta against `jkaloger/hydra`), **E8.6** (the asymmetry) and **E8.7**
(0.63 source reads per engine call, 15 of 33 being payload-shape lookups) are the
only measurement of this engine anywhere in the corpus.

RFC-026 is **not** the plan home, and a plan should not be added to it. Its subject
is the review/response *process*; the design run appears as its **instrument**. Its
own open questions OQ-1, OQ-2 and OQ-3 all propose experiments *in* the managed
design run — so engine fitness is **upstream** of RFC-026, not a section inside it.
Assessed 2026-08-15: no new RFC is warranted either. Roughly 22 of the cluster's
27 items are conformance work against SPEC-029 and want slices, not discussion; the
two genuinely open questions became records instead (below).

## The five batches

`cluster:design-run`, ~27 items. `area:design-run` means the **engine**;
`area:design` means the **stage/skill** — they were used interchangeably until
2026-08-15 and are now separate.

| batch | items | state |
|---|---|---|
| **Contract legibility** — what may I send, what does the run need next | `IMP-390`, `ISS-320`, `ISS-282`, `ISS-298`, `IMP-393`, `ISS-299` | open; **SL-251** in flight on the payload-contract face |
| **Submission strictness** — the silent-absorption class | `ISS-290`, `ISS-327`, `ISS-328`, `ISS-333`, `ISS-346` | **gated on QUE-219** |
| **Inquiry-map semantics** — the meaning tier | `IMP-386`, `IMP-387`, `IMP-388`, `IMP-389` | **gated on QUE-218** |
| **Durability / lifecycle** | `ISS-315` (live breakage), `IMP-412`, `IMP-366` | open, ungated |
| **Engine quality / test seams** | `IMP-399`, `IMP-364`, `IMP-357`, `IMP-373` | open, ungated |

`ISS-300` and `ISS-303` sit beside the map batch and are deliberately **ungated** —
correctness defects in moves that already exist. `ISS-303`'s fix (retain the prior
disposition across a reopen) is the `prior` capability `IMP-386` would build on.

## The two gates

Both are `QUE` records, so the block is real — `next` and `blockers` honour them
(ADR-017).

- **QUE-218** — *Does the inquiry map earn a semantic tier?* Gates the map batch.
  Settled by **CHR-065** (`needs ISS-299`): surface the map, then re-measure on a
  real design run against E8.3's baseline of 9 nodes / 0 edges / 17 revisions.
  The trap this gate exists to prevent: building cascade and cauterisation onto a
  structure nobody has been shown using.
- **QUE-219** — *Submission strictness against snapshot forward-compatibility.*
  Gates the strictness batch. One tradeoff on four surfaces, bound by
  `mem.fact.design-run.snapshot-outlives-the-binary`. No separate experiment needed
  — it is naturally the first decision of whatever slice takes that batch.
  `ISS-315` is the read-path horn already live as a breakage and is **ungated**;
  its fix is half the answer.

## In flight and adjacent

- **SL-251** (*Acts carry their own payload contract*, `design`) takes one of
  `IMP-390`'s four candidates — making the payload contract fetchable rather than
  exemplified. The other three stay with `IMP-390`, which is sequenced `after SL-251`.
- **SL-238** (*Cross-kind backlog ordering*, `design`) owns admitting cross-kind
  `needs` into `backlog list --by sequence`. Not a blocker: the gates work in
  `next`/`blockers`; only that one view could not represent the target, and its
  false `absent` lines were suppressed 2026-08-15.

## Open decisions nobody has ruled on

Three duplicate pairs, confirmed 2026-08-15, **not** merged — the later filer in
each case wrote a full independent record, so folding is authoring work:

| later | earlier (keep) | what the later one adds |
|---|---|---|
| `ISS-346` | `ISS-333` | the refuse-the-no-op-submission candidate; the SL-253 sighting |
| `ISS-348` | `ISS-320` | refusal-enumerates-ids; the *is the section map adding information at all?* question |
| `IMP-357` | `IMP-364` | why the obvious fixes fail; the `[lib]` target direction |

All three were re-filed because the earlier item sat outside `cluster:design-run`
and could not be found. The cluster tag is the dedup instrument — check
`doctrine backlog list -t cluster:design-run` before filing here.

## Related

- [[mem.fact.design-run.snapshot-outlives-the-binary]] — the constraint behind QUE-219
- [[mem.signpost.doctrine.backlog]] — backlog verbs
- [[mem.signpost.doctrine.knowledge]] — knowledge-record verbs
