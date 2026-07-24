---
name: harvest
description: Use at the end of any coherent unit — a phase, task, review close, or slice wrap — to sweep what the unit produced, learned, and left open into their durable sinks, and to maintain the Harvest section of the governing slice's notes.
---

# Harvest

The routed entry point for the end-of-unit **harvest** — the sweep of what the
unit produced, learned, and left open into their durable sinks. The procedure,
the three legs, and the sink table are **owned by `harvest.md`**; consult it
and follow it. Do not re-derive the routing here — point there.

During execution, working notes belong in the **active runtime phase sheet**
(`state/.../phases/phase-NN.md`) — disposable working context. The harvest is
the wrap-up sweep that moves what matters into durable homes; the slice's
`notes.md` (`doctrine slice notes <ID>` scaffolds it on demand) carries the
durable prose. Honour the storage rule: live progress lives in the state tree,
never in authored files.

If you don't know which slice owns the work, find it with `doctrine slice list`.
**No governing slice** (an RFC, spec, or review close)? The harvest still
happens — the legs route to their sinks per `harvest.md` §5; there is just no
`## Harvest` manifest to maintain.

## The three legs

- **produced** — the work delta. Be concise, but record:
  - what's done
  - any:
    - surprises encountered or adaptations required
    - potential rough edges, omissions, or refactorings for later
    - follow-up actions advisable
    - open questions relating to completed or upcoming work
    - durable facts, patterns, or gotchas that should become a memory
    - relevant commit hash(es), or: uncommitted work
    - whether `.doctrine` changes were committed promptly per repo doctrine, or
      are still pending and why
    - if committed, whether they went out with code or separately when that
      matters for the next agent
  - whether the verification gate (`doctrine check gate`) has run successfully
    since code was last modified, or: outstanding errors
- **learned** — reusable guidance and citable epistemic observations, routed to
  their sinks per `harvest.md`.
- **open** — the decisions, questions, assumptions, and constraints the unit
  carries forward. This is the leg a produced-only sweep silently drops: route
  it via **`/knowledge`** (→ DEC / QUE / ASM / CON).

Each leg's sink is owned by `harvest.md`'s §2 table — cite it, don't restate it.

## `## Harvest` maintenance

At each pass, update the single-copy `## Harvest` section in the slice's
`notes.md` per `harvest.md` §3: restamp the `fresh-as-of` line (date · lifecycle
position · head commit), keep entries pointer-only (ids + one clause, never a
status), and drop settled or superseded entries — git holds the history, so the
section stays current rather than cumulative.
