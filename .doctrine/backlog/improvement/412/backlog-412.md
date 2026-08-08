# IMP-412: Locked design run has no handover exit

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## The gap

`/handover`'s "Design work short-circuits" section branches on exactly two
cases, and a finished design falls into neither cleanly:

| `design show <SLICE> --format status` | skill says |
|---|---|
| a run prints | short-circuit — emit `doctrine design resume <SLICE>` as the continuation |
| the command errors | no run; continue with the dial |

A **locked** run prints. So the prose routes it to the first branch and the
handover emits `design resume` — which drops the next agent back into design,
the one stage that is provably finished. When the whole point of the handover is
"design is done, go plan", that continuation is actively wrong.

Observed on SL-248, whose handover note recorded the deviation rather than
following the rule:

> I did not short-circuit onto the design run, despite the skill's rule that a
> live run holds its own continuation. The run is locked at revision 93 and
> design is finished — emitting `doctrine design resume 248` would drop the next
> agent back into design, which is the one thing this handover exists to
> prevent.

This has come up several times as a source of near-confusion — the agent has to
reason its way *out* of an explicit skill instruction each time, and only
sometimes records that it did.

## Neither side carries the exit

**The CLI.** `design resume` on a locked run emits the ordinary 22-line
projection — accepted decisions, evidence references, empty `open_questions` —
with `active_path none` and `next_obligation none recorded`. Nothing in it says
*design is complete, the next stage is `/plan`*. It is a design-shaped re-entry
into a stage with nothing left to do.

**The skill.** `handover/SKILL.md:30-69` has no third branch. The no-run branch
is carefully written — it splits the error into two sub-cases and warns against
`design start --from-design` as a way to manufacture a handover target. The
run-prints branch has no equivalent care about *which stage* the run is in.

## Shape of a fix

Two candidate surfaces; they are not exclusive and the cheap one may be enough.

1. **Prose only.** Add a locked/terminal branch to the short-circuit section:
   a locked run's continuation is the *next lifecycle stage*, not `resume`.
   Cheapest, and fixes the recurring reasoning tax immediately.
2. **CLI affordance.** Have `resume` (or the status envelope) name the forward
   step when the stage is terminal, so the skill can defer to the tool rather
   than re-encode the lifecycle. This is the locked-state instance of IMP-390's
   general complaint — `next_obligation` is declared and never written, so the
   one field that could carry "you are done, go plan" is permanently empty.

Prefer (1) standalone; (2) belongs with IMP-390 if that is picked up, and would
let the prose in (1) shrink to a pointer.

## Related

- IMP-390 — envelope reports state, not what to do next (`next_obligation`
  declared but never written). Adjacent and partly overlapping: this item is the
  locked-stage instance plus a skill-prose half that IMP-390 does not cover.
- DEC-058 — "Design handover short-circuits to the managed run", the accepted
  decision the current prose implements. The decision is not wrong; it is
  under-qualified for terminal stages.
- SL-248 — where it was observed.
