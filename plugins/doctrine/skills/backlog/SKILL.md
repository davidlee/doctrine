---
name: backlog
description: Use when the user wants to create, survey, triage, tag, or transition a doctrine backlog item (issue / improvement / chore / risk / idea) — `doctrine backlog` is the CLI surface; use this skill to drive the correct verb for the intent.
---

# Backlog

The backlog is the **work-intake home** — latent work intent captured as
`issue`, `improvement`, `chore`, `risk`, or `idea` items, triaged and
promoted into slices. 

The CLI is the source of truth for exact flags: `doctrine backlog --help`

## Creating new backlog items:

- [ ] Determine kind membership / validity
- [ ] Survey backlog for potential duplicates / neighbors
- [ ] Prefer expanding / improving an appropriate existing item
- [ ] Choose a clear, concise title
- [ ] Create the new backlog item 
- [ ] Read & fill its template
- [ ] Tag it appropriately and consistently
- [ ] Record any dependecies and appropriate sequencing priorities

## Verbs


| intent | verb |
|---|---|
| capture a new item | `doctrine backlog new <kind> [title]` |
| survey items | `doctrine backlog list [--kind …] [--status …] [--tag …] [--all]` |
| inspect one item | `doctrine backlog show <ID>` |
| transition status | `doctrine backlog edit <ID>` (prompts) |
| add/remove tags | `doctrine backlog tag <ID> --add <tag> [--remove <tag>]` |
| record a hard dep | `doctrine backlog needs <ID> <DEP-ID> [<DEP-ID> …]` |
| record soft ordering | `doctrine backlog after <ID> <PREDECESSOR-ID>` |


## Kind membership

- **issue** — a bug or defect
- **improvement** — a tooling/UX gap or enhancement (no new feature)
- **chore** — housekeeping, tech-debt, refactor
- **risk** — unresolved work-risk (uncertain future harm needing mitigation,
  acceptance, or expiry); NOT a general epistemic note
- **idea** — a speculative proposal, not yet scoped or committed

The **work-intake membership test** (`mem.concept.backlog.work-intake-membership`):
a candidate that does not fit the work-status lifecycle
(`open|triaged|started|resolved|closed`) is not a backlog item.

## Status lifecycle

`open → triaged → started → resolved → closed`

- Terminal statuses (`resolved`, `closed`) require a **resolution** (prompted).
- Non-terminal statuses forbid a resolution (re-opening auto-clears it).
- `doctrine backlog edit <ID>` walks the transition in-place.

## Tags

Lowercased, `[a-z0-9_:-]`. Colon namespacing (e.g. `area:backlog`). Add/remove
in one call: `doctrine backlog tag <ID> --add area:cli --remove stale`.

## Dependencies

- **`needs`** — hard prerequisite; validates every ref exists, refuses a closing
  dependency cycle (names the members; nothing written). Use for "X blocks Y."
- **`after`** — soft sequencing preference; validates target exists, never rejects
  a cycle (evicted at `order` time). Use for "do X before Y if convenient."

## Rules

- The id prefix (`ISS-`, `IMP-`, `CHR-`, `RSK-`, `IDE-`) auto-selects the kind —
  no need to pass `--kind` to `show`/`edit`/`tag`/`needs`/`after`.
- A risk admitted to the backlog must be *unresolved work-risk*, not a general note.
- Don't hand-edit backlog TOML — use the verb. Prose (`*.md`) is hand-edited.
- A terminal item is hidden from `list` by default; use `--all` or `--status closed`
  to reveal.
