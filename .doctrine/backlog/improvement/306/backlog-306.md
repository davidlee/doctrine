# IMP-306: Consolidate capture skills into harvest + handoff

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## The knot

`/notes`, `/backlog`, `/next`, `/handover` overlap heavily. Reframed by intent:

- **`/notes`** is really just a reminder / sub-step — a prompt for an agent to
  run the durable-capture pass itself. It isn't a distinct capability; it's the
  capture reflex under a name.
- **`/handover`** = *first* ensure any durable notes are captured, *then* write a
  handover for what does **not** belong as durable state (the transient "start
  here" scaffolding).
- **`/next`** = the same shape as handover, just a lighter-weight continuation
  prompt.
- **`/backlog`** is one of the durable sinks the capture pass routes to (work
  intake), alongside memory / knowledge / notes.

So there are really two capabilities hiding behind four skills.

## Desired shape

Collapse to **two** invocable skills:

1. **Capture durable state** — "do what's necessary" to sweep produced /
   learned / open into their durable sinks. This *is* the harvest reflex
   (`harvest.md`), promoted from a sub-step to the primary capability. Subsumes
   `/notes`.
2. **Capture + continuation** — does (1), then additionally emits a continuation
   prompt for the next agent (the transient, gitignored handover packet).
   Subsumes `/handover` and `/next` (the latter being the lighter-weight variant
   — possibly a single skill with a verbosity dial rather than two).

The distinction between the two is *only* whether a continuation prompt is
emitted; the durable-capture half is identical and shared, not duplicated.

## Why this wasn't addressed by SL-215

SL-215 authored the harvest reflex (`harvest.md`) and wired the **slice/phase
spine** to cite it (code-review, audit, close, execute, notes, handover's *phase*
branch). It did **not** rationalise the skill surface: `/notes`, `/next`, and
handover's "another artifact" (non-slice, e.g. RFC) branch still carry no harvest
cue — an RFC handover runs no capture pass at all. 215 fixed *where findings
land*; it left *which skills you invoke* untouched. That overlap is this item.

## Notes / open questions

- Is the continuation-emitting skill one skill with a light/heavy dial, or two?
  (`/handover` vs `/next` today differ only in weight.)
- The shared durable-capture half should be the single owner of the sink-routing
  discipline (`harvest.md` §2 sink table) so both skills cite, never restate it.
- Backwards-compat: existing `/notes`/`/handover`/`/next` invocations and their
  routing-table entries would need aliasing or migration.
- Relate to SL-215 (`originates_from` / follows) once sliced.
