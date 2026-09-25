# IMP-484: Shipped corpus does not convey what a fresh client agent needs to run doctrine

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## The gap

ISS-309 is about shipped prose being *wrong for the reader*. This is about it
being *absent for the reader*. The shipped corpus is what a fresh agent in a
client repo has to orient by; every CLI surface with no shipped orientation is a
surface that agent either rediscovers from `--help` or never learns exists.

Measured by mapping the shipped orientation against the CLI spine
(`doctrine --help`, verified 2026-09-26). Note the shape of a gap here: it is
never an error. Nothing fails. The agent simply runs doctrine worse-at-best, or
not-at-all-at-worst, and no signal distinguishes the two.

## Confirmed gaps

**The corpus-health surface — `doctor`, `validate`, `check`, `publication`.**
No shipped signpost or reference doc. `doctor` is described by the CLI itself as
"full corpus health scan — all eight checks", and `validate` as "scan entity ids
for integrity violations". These are exactly what a client needs when its corpus
is sick, and shipped guidance mentions them only incidentically — `install/mod.just`
and `install/hymns/role/worker.md`. An agent cannot be told to run a check it has
never been told exists.

**`observation` — re-verified 2026-09-26, NOT a gap.** This item's earlier claim
that the ledger is documented nowhere is false. `install/using-doctrine.md:68-105`
is the documented home: what an observation is, `doctrine observation record
friction`, reading the corpus back with `list`/`search`, correction via
`supersede`/`retract`, the interface table (CLI / MCP tool / hand-back), and that
records are authored and committed rather than disposable. The instruction this
item quoted — the "Instrumentation" section telling every agent to record
friction — is this repo's own `.doctrine/governance.md` surfaced into the boot
snapshot, not shipped text. Recorded here so the next sweep does not re-derive a
gap that does not exist (SL-267 design review `RV-391` `F-6`).

**The reports group — `status`, `next`, `blockers`, `survey`, `explain`,
`findings`.** No shipped signpost at all; `doctrine next` appears once in shipped
text (`install/routing-process.md`) with no orientation. These are the "what
should I do now" verbs, and a fresh agent's first question.

**`config`.** "Inspect and modify `doctrine.toml` `[priority]` coefficients" has
no orientation. What ships is the config file itself (`install/doctrine.toml`,
`.example`), which a client *has* but which does not explain the surface it
configures.

**The facets group — `estimate`, `value`, `compare`, `risk`, `tag`.** No shipped
orientation. `tag` in particular is the only way to make a corpus greppable by
subject, and nothing tells a client it exists.

**`supersede`, `serve --mcp`, `worktree`.** No dedicated shipped orientation.
`relating-entities` partially covers `supersede`; `claude-activation.md`
partially covers MCP activation.

**Judged correctly out of scope, but record the judgement.** `reseat`, `export`,
`prompt`, `reservation`, `verify`, `catalog` are doctrine-developer surfaces —
`verify` describes itself as "doctrine's own runbook checks". Not gaps. Stated so
the next sweep does not re-derive the decision.

## Structural gaps

Three, and they matter more than any single missing topic because they bound what
a client can *find*:

- **`mem.signpost.doctrine.skill-map` is the shortest shipped memory in the
  corpus (778 bytes)** — and it is the entry point every routing decision depends
  on. The most load-bearing shipped artifact is the thinnest.
- **The boot snapshot's Memory index enumerates signposts only**, so the fourteen
  explanatory memories — `mem.concept.*`, `mem.fact.*`, `mem.pattern.*`: the
  entity engine, storage model, memory model, conventions, core loop, TDD loop,
  CLI-as-source-of-truth, storage tiers, reading entities, routing gate, boot
  snapshot, hymn cascade, work-intake membership, close-drift-discharge — are
  unreachable from the index whose stated job is to make memory reachable. Boot
  does say to run `/retrieve-memory`, so this may be intended; it should be a
  recorded decision rather than an emergent property.
- **Three of the twenty-one signposts are absent from that same index**
  (`concept-map`, `rec`, `rfc` — ISS-215). So the index is incomplete in *kind*
  and, separately, incomplete *within* the kind it does enumerate.

## Boundaries

- **Corpus authoring only.** No engine behaviour change. If a gap here is really
  a missing *skill* rather than missing shipped prose, it belongs to that skill's
  owner, not to this item.
- **Do not fix sufficiency by adding a signpost per verb.** The CLI spine is a
  product surface, not a table of contents; a signpost for every verb would
  reproduce `--help` in prose and rot (the anti-pattern `using-doctrine.md`'s own
  header already warns about: *"names verbs and states discipline — it never
  reproduces `doctrine --help`"*). The real question — which surfaces deserve
  shipped orientation, and at which tier — is a design decision for the
  remediation slice.
- **Distinct from the accuracy axis** (CHR-080) and the citation axis (ISS-309).
  All three seed the same remediation slice.

## Links

Sibling to ISS-309 (citation correctness) and CHR-080 (accuracy). ADR-005 is the
governing tiering decision — skills route, reference docs explain, memory
orients — and is the frame this item's "which surface deserves what" question
should be settled in.
