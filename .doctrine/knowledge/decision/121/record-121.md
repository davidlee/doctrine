# DEC-121: The exploring edge is guarded by attested user checkpoints

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## What ISS-285 got wrong about its own question

`ISS-285` offered three futures for `governing-context-recorded` and
`initial-concerns-recorded`, which guard `exploring → inquiring`
(`gate.rs:164-167`): specify them with a subject rule that means something, derive
them from framework-owned run state, or retire them and let the runbook guard the
edge alone.

All three read the pair as **run state** — something the engine either observes,
computes, or stops asking about. None of them is available, because the pair is
not about run state. It is the residue of a user interaction that `SL-233`'s
design specified in a single bullet (`slice/233/design.md:265`) and never built.

The evidence is the runbooks. `exploring.toml` has five required steps and
`inquiring.toml` has two. **All seven are agent-solo.** Not one names a user
interaction. Across the entire orientation-and-interrogation span the only user
contact is a single blanket `user-accepts-sufficiency` at the far end — one
yes/no covering everything, with nothing decomposed and no way for a refusal to
say which part is missing.

## The decision

The pair is **specified**, in `DEC-120`'s Attested kind: each becomes a user act
whose artefact the engine records, binds, and derives over.

**Governing context.** The agent puts in front of the user what it believes binds
*and* what it checked and dismissed, with reasons. The dismissal list is the
higher-value half — a wrong inclusion shows up later as a constraint that does not
fit, while a wrong dismissal is invisible forever. The user corrects, adds, and
confirms. The artefact is the resulting **edge set** (`governed_by`,
`references --role concerns`), which is queryable, feeds the audit-time
conformance delta, and survives the run.

*Enumerating those ids in prose is not the artefact and does not substitute for
it* — a "Governance" section listing ids is queried data in prose, which the
storage rule forbids. Citations belong at the decision they constrain.

**Initial concerns.** Two acts, not one. The user reviews the seeded inquiry graph
and steers before interrogation goes deep; and the agent declares which questions
it considers settled or settleable without asking, for confirmation. The second is
the more consequential and currently has no expression at all: an agent marks its
whole graph blocking by fiat, and the difference between eight round-trips and
four is invisible.

**The empty case is the strict case, in both.** A governance sweep that finds
nothing, and an exploration that raises no open question, are the results least
distinguishable from a failed search
(`mem_019fbe301c5074e2a65778ec5374bb82` — an emptiness-claim witness is the shape
most likely to be wrong and green-looking). Empty must be shown *with what was
searched* and must never be the skippable path.

**Arbitration of relative import is not part of this.** Where two constraints
genuinely conflict, that is a `/consult`, not a checkpoint field. Asking for a
ranking routinely invites ranking things that need none.

## Why the edge keeps two guards

The runbook and the conditions have looked redundant on this edge since they were
built. They are not, and this is the explanation that was missing:

| guard | actor | binding | failure |
|---|---|---|---|
| runbook step | agent | step-definition digest | went stale, or the check stopped clearing |
| condition | **user** | `DEC-088` content-bound attestation | the artefact moved out from under the acceptance |

The step *prompts* the interaction; the condition holds the *user's* attested
artefact. Different actors, different bindings, different repairs. Each can state
a contract, which is what `SL-244` asks of them.

## Scope of this record

`SL-244` specifies these interactions and their contracts, and ships spec-adjacent
documentation carrying canonical diagrams of them. **Building the interaction
spawns.** Until it lands, `exploring → inquiring` passes on the runbook alone —
a stated interim state, not an accident, and no worse than the status quo it
replaces.

Related: `DEC-120` (the kinds), `DEC-088` (the attestation primitive this rides),
`ISS-285` (closable on this decision), `ISS-286` (its subject-rule complaint is
answered as a side effect: the attested artefact *is* the subject, replacing the
arbitrary section fingerprint), `SPEC-029`, `RFC-026`.
