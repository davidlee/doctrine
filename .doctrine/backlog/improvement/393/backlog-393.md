# IMP-393: Reader-facing design render for review

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## The gap: `show` does not mean here what it means everywhere else

`show` is the codebase's settled convention for **one** thing — render an entity
for a reader, reuniting the two storage tiers. It is stated as a standing
guardrail: *read entities via `doctrine <kind> show <ID>`, not raw files;
structured data lives in `*.toml`, prose in `*.md`, and `show` synthesizes both*.
Every kind honours it. `slice show`, `adr show`, `knowledge show`, `memory show`,
`review show`, `backlog inspect`/`show` — all render for a reader.

`design show` is the exception. It renders the **writer's turn envelope**: active
path, nearby frontier, blockers, counts, change rows since a known revision, and
a bounded projection that truncates. That is a legitimate and necessary artefact
— it is just not what `show` names anywhere else in the system, and it occupies
the slot a reader would reach for first.

So the defect is not only a missing capability, it is a **naming violation** that
mislabels the missing capability as present. An agent or reviewer that applies
the documented rule to a design gets a turn envelope and has no reason to suspect
it is looking at the wrong thing.

`doctrine design` has five verbs — `start`, `show`, `apply`, `resume`,
`materialise`. None of them renders the design *for a reader*.

- `design show <SLICE>` is the **writer's turn envelope**: active path, nearby
  frontier, blockers, counts, change rows since a known revision, and a bounded
  projection that truncates. It answers *what do I do next*, not *what has been
  designed*.
- `design resume` is the same register, narrower — a cold-start re-entry
  projection for the agent holding the run.
- `design materialise` writes `design.md` to disk and returns nothing
  reader-facing.
- `slice show <ID>` renders the **scope** document and slice metadata. Not the
  design.

So a reviewer handed this slice gets one of two bad artefacts: raw `design.md`
prose with no metadata whatever, or a turn envelope that is not a review
artefact in any sense.

## Why it bites hardest for a reviewer

The design's substance is **not all in `design.md`**. On `SL-244` the seven
settled decisions are knowledge records — `DEC-120` … `DEC-126` — bound to the
run through checkpoint dispositions. A reviewer reading `design.md` alone sees
prose that cites ids and cannot resolve them without already knowing that
`doctrine knowledge show DEC-120` is the move. The prose tier and the queried
tier are split by the storage rule, which is correct, and `show` is the verb that
is supposed to reunite them — the guardrail every other kind already honours
("read entities via `doctrine <kind> show <ID>`, not raw files"). The design run
is the one kind where that verb does not exist.

## Shape of the want

A render that reunites the two tiers for one design, at minimum:

- the authored prose sections, in document order, with their titles;
- run metadata: stage, revision, whether materialisation is current, the
  authored watermark's standing;
- the decision set — every record the run created or adopted, with its id, kind,
  title and the inquiry node that produced it;
- outstanding state a reviewer must judge against: open inquiries, blockers,
  deferred nodes, gate conditions not yet cleared;
- section attestation and finding standing, once `reviewing` is in play;
- the navigable neighbourhood — governing entities, the slice, research
  artefacts — as **resolvable references**, so a reviewer knows what else to read
  and how to read it.

## Resolving the name

Because this is a convention violation, the fix is not "add a sixth verb and
leave `show` meaning something else". Two candidates:

1. **`show` reverts to the convention.** `design show <SLICE>` renders the design
   for a reader; the turn envelope keeps its own name. Note the envelope's own
   `--format` slot already spells it — `[default: prompt]`, with `prompt` / `json`
   / `status` — so the writer rendering is *already called* `prompt`. Making the
   reader rendering the default and leaving `--format prompt` for the envelope is
   the smallest change that restores the convention. Cost: every existing caller
   that relies on the bare-`show` default gets a different artefact, so the skills
   and the run's own guidance move with it.
2. **The envelope moves to its own verb** (`design turn` / `design envelope`),
   sibling to `resume`, which is the same register. Cleaner conceptually — a turn
   projection is not a rendering of an entity — and it frees `show` completely.
   Cost: a new verb plus the same caller migration.

Either way the migration is the same size, so the choice is about which name
tells the truth. Worth checking whether any other kind has a writer-facing
projection wearing a reader's verb before deciding — if `design` is the only one,
it is a local fix; if not, it is a convention sweep.

## Origin

Raised during `SL-244`'s design run, from the question of how a design is handed
to a reviewer agent that may not be kitted out with doctrine conventions. Sibling
items: `IMP-394` (bootstrap context for such an agent), `IMP-395` (the skills and
prompts handing the collaborator its exact invocation). This item is the
**artefact**; `IMP-394` is the **orientation**; `IMP-395` is the **handoff**. All
three are needed for the reviewer to succeed; each is separately useful.
