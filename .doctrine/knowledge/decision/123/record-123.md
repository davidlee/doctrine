# DEC-123: Contract structure rides a const table, not the prose asset

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## The split

A contract has two readers. The **agent** reads what the edge wants and how to
discharge it — narrative, and `DEC-122` puts it in an asset. The **engine** needs
which `DEC-120` kind the condition is, which act discharges it, and what a refusal
should cite — structure.

Structure stays in Rust, as a static table beside `boundary_conditions` and
`boundary_runbook`. The asset stays purely narrative. The closed `Condition`
vocabulary keys both.

Adding a third static table beside two existing ones is unremarkable. Adding a
third loader would need the argument `DEC-101` had to make for runbooks, and
nothing here forces it.

## What this buys

- No parser, no validation domain, no fourth prose system arriving unannounced.
- Totality strengthens beyond `DEC-122`'s promise: the set-equality test runs over
  **three** enumerations, so a condition missing either its const row or its prose
  fails, on the `WRITER_ACTS` template.
- The drift hazard shrinks to one detectable case. Have the renderer **inject**
  the kind and the discharging act from the const into the rendered contract, so
  the prose never restates them and cannot contradict them. What the prose says
  beyond that is craft, and craft drifting is a review problem rather than a type
  problem.

## The verifier, considered on its merits

Runbook steps may carry a `verify` argv, and `explore.research` uses one
(`doctrine verify research-current --slice {slice} -p {repo_root}`). The question
raised: should a condition carry one — `doctrine verify governance-recorded
{slice}` — particularly given the extensibility angle.

**It is not blocked by `DEC-101`, and the reason matters.** That record's own
consequences state that *"a verifier result is exactly a fact Doctrine must
derive"* — routing one through caller payload compiles and is then refused by
`Refusal::DerivedConditionClaimed`; results enter shell-side through
`DerivedInput`. So a verifier is derivation, not claim. Nor is it the type error:
that error is mapping the **open** set of step ids onto the closed ten, and a
condition-keyed verifier is keyed by the closed vocabulary itself. Nothing is
narrowed.

**It is not an alternative to this split.** A verifier answers *does this hold
right now*. It does not say which kind the condition is, which act discharges it,
or what the refusal should cite. It evaluates a condition; it does not describe
one. The two are orthogonal, so taking the split costs nothing against it.

**No hook is needed now.** The const table is the one place in this design where
adding structure later is genuinely free: compile-time data, never serialised,
never persisted, no wire version, no snapshot migration, no override seam. A
`verify` address can join a const row the day a consumer wants one. Adding a
field that is `None` for all ten members today is untested speculation, and it is
precisely the knee this slice was told not to build.

**Where it lands instead.** Two places, both real:

1. `doctrine verify` is a family with **one** member, and `DEC-101` explicitly
   left *"which shipped checks `doctrine verify` launches with beyond
   research-current"* open at phase-plan. A per-condition check is a strong
   candidate for `inq-6`'s fetchable channel — an agent asking *would I pass?*
   rather than submitting and reading the refusal.
2. Project extensibility on this edge already has a home. `DEC-121` divides the
   labour: the runbook step is the agent-facing, project-substitutable guard, and
   `DEC-101` designed step ids as API precisely so *"a project MAY substitute a
   verifier"* once `IMP-372` lands. A project wanting its own governance check
   authors a step, not a condition.

**And a constraint worth carrying forward.** `DEC-101` also holds that *"the pure
core executes nothing, and no subprocess sits inside `apply`'s admit→persist
span"*. So a condition verifier can never run inside `advance` — it runs
shell-side and enters as derived input. That is fine for a fetchable check the
agent invokes, and it rules out the shape where the gate shells out to itself to
answer a question the in-process snapshot already holds.

## The residual, named rather than claimed away

Prose asserting *"the engine derives this"* while the const says Attested is not
mechanically detectable in general. The injection rule above removes the cheap and
common subset; the rest is craft under review. Thread 3 of `SL-244`'s research
called this irreducible for conditions the engine does not enforce, and this
record does not claim otherwise.

Related: `DEC-122` (the prose home), `DEC-120` (the kind a contract states),
`DEC-121` (whose contracts these are), `DEC-101` (verifier results are derived
facts; the `doctrine verify` family; the no-subprocess-in-apply constraint),
`IMP-372`, `IDE-047`.
