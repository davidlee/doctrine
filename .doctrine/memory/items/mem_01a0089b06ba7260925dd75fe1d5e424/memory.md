`doctrine design contract [--format json|prompt]` prints the **complete** wire
contract for a `doctrine design apply` payload: every key, its type, whether it
may be omitted and what omission means, per-variant payload placement, and what
happens to a key the contract does not list. It needs no slice, no design run,
no project root — it answers from outside a doctrine project entirely.

**Do not read `src/design_run/submission.rs` to learn payload shape.** That was
the old route and it cost 15 source reads out of 33 across one measured design
run (RFC-026 E8.7) — the largest single category. It is also unavailable to an
installed client project, which has no source to read.

Landed by SL-251 (2026-08-16), which exists for exactly this.

## What it covers that the old worked example did not

- **`cursor`.** The `declare` hint's `traversal` example omitted it, and it is
  the one key a *resuming* agent must set. It now renders with its sparse
  semantics stated, and the envelope's worked example carries it too (DEC-228).
- **The full recursive closure**, ~25 types — not top-level only. Both failures
  observed during SL-251's own design run were nested: an unknown key on
  `Declaration`, and `AgentAct`'s external tagging inside `agent_declaration`.
- **The extern region.** `CreateRecord.kind`'s seven admissible record kinds and
  the facet keys each one opens — derived from `knowledge::RecordKind`, and the
  one part of the contract recoverable no other way.
- **Tagging semantics per variant**, which is where the round trips came from:
  an externally tagged unit variant is a `BARE STRING`, not an object.

## Where else the address appears

Three push points, all single-sourced from `PAYLOAD_CONTRACT_POINTER`: a payload
parse refusal appends it on an indented continuation line; the turn envelope
carries a `contract_pointer` field and a `contract <address>` line; and
`doctrine design apply --help` names it. The same content is published as
`reference/design-payload-contract.md` — read it with
`doctrine library show reference/design-payload-contract.md`.

## The trap it does NOT close

A misspelt **top-level** key is still discarded in silence and the command still
exits 0 — `ApplyRequest` carries `#[serde(flatten)]`, which serde cannot
reconcile with `deny_unknown_fields`. SL-251 deliberately did not repair that
mechanism (ISS-333 stays open on its serde axis); it removed the probing loop
the opacity caused. Nested types mostly *do* refuse: `Declaration`,
`CheckpointActDeclaration` and `AgentActDeclaration` are `deny_unknown_fields`,
the other eight are not. The contract states which is which per type, so check
it rather than assuming a silent success landed.
