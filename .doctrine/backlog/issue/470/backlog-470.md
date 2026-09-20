# ISS-470: Capsule tier never moves the slice lifecycle

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Surfaced as `RV-372` `F-11` at `SL-246`'s reconciliation audit.

## Observed

`SL-246` was driven by `/capsule-driver`. All six phases reached `completed` and
every phase sheet recorded its transitions correctly — the phase tier tracked
fine. The slice tier never moved:

    doctrine slice status SL-246
    SL-246  ready [warn]  phases: 6/6
      ⚠ divergent: phases complete but lifecycle not terminal

## The seam

The capsule tier is a **substitution for `/execute`**, and it inherited
`/execute`'s phase-tier duties without its slice-tier ones.

| path | who moves the slice |
|---|---|
| `/execute` | itself — `SKILL.md:27` to `started` at the first phase, `:65` to `audit` after the last |
| `/dispatch` | the orchestrator — `SKILL.md:17` commits the slice-status write, `:159` moves it to `audit` |
| `/capsule-driver` | **nobody**. Three agent definitions plus the driver skill contain zero occurrences of `slice status` |

Both tier boundaries are stated and both are individually correct. The driver's
loop ends at step 6 — *"Halt when phase implementation is complete. Do not begin
audit."* The orchestrator's Bounds — *"Never begin audit. That is a lifecycle
transition for the seat with a human attached."* Neither picks up what falls
between them.

This is structural rather than a lapse by any worker. The orchestrator holds no
`Edit`/`Write` **by design** (an orchestrator that fixes things itself burns the
context budget the tiering exists to protect); the driver is explicitly *"Do not
implement"*; the worker has the tools but is scoped to one phase. No seat in the
tier has both the authority and the remit.

## Why it matters

The `audit → reconcile` transition and the close gate both key off the slice's
lifecycle field. A divergence left standing surfaces as a **refusal at whatever
later moment someone tries to advance** — far from its cause, and to an agent
with no context for why the field is wrong.

Mitigating, and the reason this is the *cheaper* of the pair: `slice status`
prints `divergent` unprompted, so the tooling does detect it. `ISS-471` is the
same seam with no detector at all.

## Shape of a fix

Cheap end: the driver's halt step, or the orchestrator's final hand-back, moves
the slice to `audit`.

**Do not blur the boundary to get there.** *Moving the slice to `audit`* and
*beginning the audit* are different acts, and the orchestrator's bound is
protecting the second. Whatever lands should restate that distinction
deliberately rather than quietly widening the bound.

Subsumed by `ISS-471`'s scribe option if that route is taken — a seat that owns
slice-tier state owns this too. Sequence accordingly: this one is a line of
prose, and should not wait on the item that needs design.

Related: `RV-372` `F-11`, `SL-246`, `ISS-471`.
