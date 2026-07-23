<!-- Shipped reference (ADR-005 PULL tier). Edit the source in
     `install/harvest.md`; the installed copy at `.doctrine/harvest.md` is inert.
     Owns the end-of-work harvest procedure once; consuming skills cite it and
     carry only their own freshness check inline. -->

# Harvest

How to close out a coherent unit of work: sweep what it produced, learned, and
left open into their durable sinks — once, from one place.

## §1 The harvest moment

Harvest at the end of any **coherent unit**: a phase wrap, a slice wrap, a review
close, a walkthrough or feedback close. It is **judgment-gated** — you weigh what
the unit produced, learned, and left open against what is already durable. A clean
unit harvests nothing, and that is a **valid outcome, not a skipped step**: the
judgment is the work, not the writing.

## §2 Three legs, one sink table

Every harvest walks three legs; each leg routes to a durable sink.

| Leg | What it carries | Sink |
|---|---|---|
| **produced** | the work delta — commits/refs, and durable follow-up **work** | notes prose (summarised) · `backlog new` for work |
| **learned** | reusable guidance, and citable epistemic observations | `/record-memory` · EVD/CPT via `/knowledge` |
| **open** | decisions / questions / assumptions / constraints carried forward | DEC / QUE / ASM / CON via `/knowledge` |

- **produced** — summarise the commits/refs into notes prose; route durable
  follow-up **work** to `backlog new`. The home arbitration (work vs knowledge vs
  notes) is owned by `using-doctrine.md` (§ Which home) — **cite it; do not restate
  the boundary here**.
- **learned** — reusable agent guidance goes to `/record-memory`; citable
  epistemic observations (evidence, concepts) become EVD / CPT via `/knowledge`.
  The knowledge-kind discriminator is owned by `/knowledge` — **cite it; do not
  restate the kind table here**.
- **open** — the decisions, questions, assumptions, and constraints the unit
  carries forward: decisions → DEC, questions → QUE, assumptions → ASM,
  constraints → CON, all via `/knowledge`. This is the **previously-missing leg** —
  the one a produced/learned-only sweep silently drops; call it out and work it.

## §3 Canonical output

The single maintained `## Harvest` section in the governing slice's `notes.md`.
Its shape:

```markdown
## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-07-24 · PHASE-03 · a1b2c3d

### Produced
- PHASE-03 done — <one line> (commits a1b2c3d..d4e5f6a)
- minted: IMP-241 — <one clause>; ISS-102 — <one clause>

### Learned
- mem.pattern.dispatch.import-tripwires
- EVD-014 — <one clause>

### Open
- DEC-011 — <one clause>
- QUE-023 — <one clause>
```

Rules:

- **single-copy** — the section is updated **in place** each harvest, never
  appended to per-event. One section, rewritten; not a log.
- **stamp** — the `fresh-as-of` line is `date · lifecycle position · head commit`,
  where lifecycle position is a `PHASE-NN` **or** a stage name (for non-phase
  moments such as a review or walkthrough close).
- entries are **ids + one clause, never a status** — a status is queried data,
  forbidden in authored prose by the storage rule; a consumer queries the CLI for
  the current status of any id it finds here.
- settled or superseded entries **drop at the next pass** — git holds the history,
  so the section stays current rather than cumulative.
- the section **stub seeds from the notes template**, so a new slice's `notes.md`
  already carries the shape ready to fill.

## §4 Consumer contract

The load-bearing part. A consumer of a `## Harvest` section checks its
`fresh-as-of` stamp against the **actual lifecycle position**:

- **fresh** → cite the ids directly; **never re-survey** what the harvest already
  swept.
- **stale** → the harvest is **owed**: route to `/notes` first to bring it current,
  never silently re-derive the manifest by hand.

**ADR-005 conformance.** This freshness check rides **inline in each consumer
skill's own body** — the doc explains the contract; the skill carries the
behavioural rule. Demoting the check to a mere pulled pointer into this doc would
be an error: the behaviour must be present where the skill acts.

## §5 No governing slice

When there is **no governing slice** there is no manifest to maintain — but the
harvest still happens. The three legs still route to their sinks (`backlog new`,
`/record-memory`, `/knowledge`); for reviews the `## Synthesis` carries the story
in the ledger; and consumers fall back to **entity queries** for anything they
would otherwise read from a `## Harvest` section.
