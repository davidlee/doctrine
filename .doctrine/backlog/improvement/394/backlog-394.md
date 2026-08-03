# IMP-394: Bootstrap context for non-indoctrinated collaborator agents

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## The gap

Doctrine's context assets are built for agents **inside** the workflow. The boot
snapshot is a governance digest for an agent about to route; the hymns cascade is
keyed on role (`worker` / `orchestrator`), harness, model and stage; the skills
assume the reader is executing a lifecycle stage.

A **collaborator** agent is none of those. An external reviewer (the project's
default is codex MCP / GPT-5.5), a research agent, a second-opinion sub-agent —
each arrives with a bounded task, no lifecycle role, and no reason to load the
routing table. What it needs is orthogonal to what every existing asset ships:

- the reference forms it will meet in the prose — what `SL-`, `DEC-`, `REQ-`,
  `ADR-`, `RV-` mean, that ids are durable and slugs are not, that `OQ-1` is
  doc-local and meaningless outside its artefact;
- the read verbs — `doctrine <kind> show <ID>`, and that raw TOML/MD is the wrong
  way in because prose and queried data are separate tiers;
- the two-tier storage rule, because otherwise an empty `.md` body reads as a
  defect rather than as a design;
- how to find things — `doctrine search`, `doctrine library tree`;
- what it must **not** do — it is not a writer; it does not route; it does not
  advance anything.

Today a collaborator gets whatever the spawning agent pasted into its prompt,
which is unbounded, ad hoc, and different every time.

## Shape of the want

A bootstrap asset for the collaborator band — published, not projected, on the
`library` seam — that a spawning agent can hand over by reference rather than by
paraphrase. Candidate shapes, not yet chosen:

- a `reference/collaborator.md` published doc, cited by path, read on demand;
- a `--band collaborator` in the hymns cascade, resolved and pasted;
- a `doctrine prompt` role beside `worker` / `orchestrator`.

The third is the most honest fit — the cascade already models *who the reader
is* — but it is also the biggest change, and the role vocabulary is currently
closed at two with real semantics attached (`prompt resolve --role` refuses
anything else). Worth checking whether a collaborator is a role or a band before
choosing.

Constraint: it must stay **short**. The failure mode is reproducing the boot
snapshot for an audience whose whole advantage is not carrying it. It should be
readable in one pass and cost less than the review it enables.

## Origin

Raised during `SL-244`'s design run, from the question of how a design is handed
to a reviewer that may not understand doctrine conventions. Siblings: `IMP-393`
(the reader-facing design render the collaborator would read), `IMP-395` (the
skills and prompts that hand it the invocation). This is the **orientation**
piece.
