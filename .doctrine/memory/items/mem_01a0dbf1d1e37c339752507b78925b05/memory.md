`governed_by` does not mean "is scoped by / sits under a spec". Its **target**
is restricted to the governance kinds (ADR, POL, STD) — the verb refuses
anything else:

    $ doctrine link ISS-490 governed_by PRD-017
    Error: `governed_by` target must be one of [ADR, POL, STD], got a PRD

`sources` is wide (`SL, PRD, SPEC, CM`, every knowledge record, and all five
backlog kinds), and its `inbound_name` is `governs`. So the label reads
"this thing is governed by a governance record", authored on the governed side
and rendered on the ADR/POL/STD side as "governs".

**A backlog item or slice scoped by a PRD/SPEC takes `related`.** That is what
`IMP-309` does with `SPEC-026`. The two labels that look like alternatives are
not:

- `descends_from` — sources are `[SPEC]` only.
- `shapes` — sources are the knowledge `RECORD` set only.

So there is no admissible label for "issue → its governing spec"; `related`
carries it and the prose carries the governance claim. Do not spend calls
probing for one.

`RelationLabel::GovernedBy`'s rule row is in `src/relation.rs` (~line 558,
`target: TargetSpec::Kinds(GOV)`).
