# DEC-227: Contract totality is the full recursive wire closure

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## `WRITER_ACTS` is the wrong axis, and correctly so

`ApplyRequest::WRITER_ACTS` (`submission.rs:989`) enumerates nine acts;
`ApplyRequest` carries twelve top-level wire keys. `delegation` is absent from the
table, and this is not a defect to repair. That table answers *does this payload
write*, for `EX-2`, and the delegation proposal channel must not count as a write.
The omission is the table doing its job.

The consequence for this slice is that a contract keyed off `WRITER_ACTS` would
ship a payload field no caller can discover — the scope's third objective names
this as the asymmetry to resolve rather than inherit. So the contract closes over
the payload, not over the writer-act table. That much was settled before inquiry.

What was open was the **depth**.

## The fork

`submission.rs` holds twelve wire-participating structs: `ApplyRequest`,
`SubmissionEnvelope`, `Declaration`, `Dispose`, `CreateRecord`,
`AcceptanceDeclaration`, `DischargeDeclaration`, `AdoptAuthored`,
`StageDeclaration`, `TraversalDeclaration`, `ReviewPolicyDeclaration`,
`CheckpointActDeclaration`, `AgentActDeclaration` — plus the enums they admit,
including `AgentAct` and `ActKind` over in `attestation.rs`.

- **Top-level only** — the twelve keys of `ApplyRequest` and the enums they
  directly admit. One `fully_populated` literal, and cheap.
- **Full recursive closure** — every reachable wire type. Roughly twelve struct
  literal pins under `DEC-221`, plus generated enum vocabularies.
- **Bounded recursion** — recurse only into types reachable from an act field,
  excluding the envelope and internal machinery.

## Measured, not argued

The decisive fact is that **both failures observed during this slice's own design
run were nested**.

- The unknown key was on `Declaration` (`submission.rs:123`), one level below
  `ApplyRequest.declare`.
- The tagging error was on `AgentAct` (`attestation.rs:671`), inside
  `agent_declaration`.

A top-level contract would have prevented neither. It would have enumerated
`declare` and `agent_declaration` as accepted keys and stopped exactly where the
caller's question began. That is not a close call about thoroughness; it is a
contract that fails to cover the cost the slice was scoped against.

## Why bounded recursion is worse than either

It saves perhaps two literals, and it buys back the problem. Deciding what counts
as *internal machinery* is a judgement, and it is the same class of judgement that
put `delegation` outside `WRITER_ACTS` — correctly for that table, wrongly for
this one. A boundary drawn by opinion will be redrawn by the next person with a
different opinion, and the contract will be silently incomplete in between.

Reachability from `ApplyRequest` is a fact about the types. It is mechanically
checkable, and a new nested wire type joins the closure by construction: failing
to pin it is a compile error at its own literal, not an omission from a list
somebody forgot to update.

## Cost, stated plainly

Roughly twelve struct literal pins, each mechanical, plus the generated enum
vocabularies. This is the dominant implementation cost of the slice, and it was
accepted as such rather than discovered later.

The closure crosses from `submission.rs` into `attestation.rs`. `ADR-001` is
unaffected — `design_run` is a single leaf-tier module with out-degree 0
(`layering.toml:31`), so this creates no new edge in the module graph.

It also lands squarely on one of the two conditions `DEC-221` named for revisiting
its pin split: *the count of nested payload types makes per-type
`fully_populated` literals the dominant cost*. Twelve is a bounded, known cost, so
this does not overturn `DEC-221` — but an implementer who finds it worse in
practice has standing to change the mechanism rather than work around it.
