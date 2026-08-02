# IMP-390: Envelope reports state, not what to do next

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## The gap

The turn envelope is an excellent state report and gives almost no affordance. It
says what *is* — revision, stage, posture, cursor, totals, frontier, blockers,
change log — and almost never what to *do*, what will be *accepted*, or what the
run needs *next*.

Measured over CHR-049's run (RFC-026 **E8.7**): **33 reads of `src/design_run/**`
or the design-prompt assets against 52 `doctrine design` calls** — 0.63 source
reads per engine call. Every one is contract discovery, not curiosity. The
categories say what was missing:

| reads | being looked up |
|---|---|
| 15 | payload shapes — what JSON may I send |
| 4 | gate conditions — what does this run need to let me advance |
| 2 | runbook step lists, fetched from the prompt assets directly |
| 1 | **`tests.rs`, read to learn how to clear a gate** |

An agent reading the test suite to find out what the machine wants is the whole
issue in one line.

## Three faces of it

**1. `next_obligation` exists and is empty.** The field is on the snapshot and
renders when present (`render/envelope.rs:256,408,987`). It read `none recorded`
at revision 9 and at revision 17. It is populated from the runbook, so it goes
null exactly when a stage's steps are all discharged — the moment the agent most
needs *advance the stage*. SL-243 has sat in `exploring` since revision 6 with
its runbook cleared and nothing anywhere proposing the move. `Locked` is only
reachable through that move.

**2. The `declare` hint is one example, not a contract.** It shows a single
declaration shape and a `traversal` example carrying `pin`/`posture`/`authority`
— omitting `cursor`, the one key a resuming agent must set. Fifteen reads of
`submission.rs` is the cost of that omission.

**3. Gate conditions are invisible until refused.** An agent cannot ask what a
stage transition requires; it reads `gate.rs`, or submits and reads the refusal.
The refusals are honest about the objection but not always about the remedy —
`verify research-current` names the exact fix, while `design apply` leaked serde
vocabulary (*"invalid type: string, expected internally tagged enum Provenance"*)
and left the agent to find the shape in source.

## Shape

Not one change. The unifying question is whether the envelope's job is to report
state or to carry the turn — today it is the former and the skill assets are
expected to supply the rest, which E8.7 shows they do not.

Candidates, roughly by cost:

- Populate `next_obligation` past runbook exhaustion — name the stage advance and
  its unmet conditions.
- Render unmet gate conditions for the *next* stage, so the requirement is
  readable before it is violated.
- Make the payload contract fetchable rather than exemplified — a schema surface,
  or a `--help` that enumerates each act's fields.
- On refusal, name the remedy, not just the objection. `verify research-current`
  is the in-repo bar.

## Relationship to the neighbours

[[ISS-299]] is the map never reaching the *user*. This is the machine never
telling the *agent* what it wants — same thinness, opposite audience.
[[IMP-389]] (derive `next`) is the traversal-shaped instance of the same
complaint.

## Confound, stated rather than left to the reader

An agent with the engine's source in its tree will read it, and doctrine
dogfooding itself makes that unusually cheap. This does not soften the finding.
The 33 reads are a **lower bound on the confusion** — the source resolved it
here — and an **upper bound on the remedy**, since an installed client project
has no source to read. The same opacity elsewhere produces guessing.

## Provenance

Argument and instrument: **RFC-026 E8.7**. Raised from the observation that the
subject kept leaving the delivered surface for source at every stage of the run —
first after a serde refusal, later pre-emptively.
