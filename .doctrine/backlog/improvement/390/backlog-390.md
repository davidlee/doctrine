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

**1. `next_obligation` exists and has never been written.** The field is on the
snapshot and renders when present (`render/envelope.rs:256,408,987`). It read
`none recorded` at revision 9 and at revision 17 — and at every other revision,
because **nothing anywhere assigns it**. It is constructed `None`
(`snapshot.rs:466`) and has no writer in the repository; the only other mentions
are the declaration, the projection, and the render. It is not populated from the
runbook and does not "go null" when a stage's steps discharge. It has been
permanently empty since it shipped.

SL-233 knew this and left it deliberately — `prior-findings.md:35`
(PHASE-07 `F-12` · PHASE-14 `F-7`) lists it beside `FragmentGroup` as a
staged-but-unwritten surface, and `notes.md:2164` records the constraint that
kept it that way:

> `RunHeader.next_obligation` (rendered as *elided prose*, so it could not serve
> as EX-2's closed enum without a v1 wire change)

That constraint bounds the remedy. Writing the field so it can *name a stage
advance* wants a closed vocabulary, but the envelope renders it as elided prose,
so the fix is a **v1 envelope wire change** under `DEC-064` — not filling in a
blank. `notes.md:2164` also assigns the writerless family to one item at harvest
rather than three, so whether this face or [[IMP-367]] owns the disposition is
unsettled.

The finding survives the correction and is sharpened by it: SL-243 has sat in
`exploring` since revision 6 with its runbook cleared and nothing anywhere
proposing the move, and the field that exists to propose it was never capable of
doing so. `Locked` is only reachable through that move.

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

## What SL-244 discharged, and what is left (2026-08-05)

`SL-244` names this item in its `originates_from` set and delivered **one** of the
four candidates above — *on refusal, name the remedy, not just the objection* —
plus the discoverability the third face asked for. This item stays **open**; the
other three candidates are untouched.

Delivered:

- Every gate condition now carries a contract (kind, subject binding, reach, and
  the act that discharges it) in one macro-generated table, so `remedy()` answers
  the refusal and the stage-entry receipt injects the same rows.
- The contract is readable *before* it is violated and without source: nine
  narratives at `design-prompts/conditions/`, and a generated stage diagram
  published at `reference/design-run-stages.md`.
- Face 3's measured cost — the four gate-condition reads and the one `tests.rs`
  read out of E8.7's 33 — has its answer shipped.

Not delivered, and still this item's:

- **`next_obligation` still has no writer.** Confirmed at close: `snapshot.rs:558`
  constructs it `None`, and every other mention is the declaration, the projection
  or the render. The `DEC-064` v1-wire-change constraint recorded above is
  untouched, as is the open question of whether this face or `IMP-367` owns the
  disposition.
- **The payload contract is still exemplified, not fetchable** — face 2, the
  fifteen `submission.rs` reads, and the `declare` hint that omits `cursor`.
- **Unmet conditions for the *next* stage are still not rendered** in the
  envelope; the receipt is delivered at stage entry, not as a forward look.
