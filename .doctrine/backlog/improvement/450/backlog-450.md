# IMP-450: Published payload contract omits the state axis

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`install/design-payload-contract.md` is generated from `payload_contract` and
pinned to it by test, and opens by promising "Every key a design-run submission
may carry, what may be sent under it, whether it may be omitted and what
omission means, and what happens to a key this contract does not list."

`SL-259` `PHASE-04` added the **state axis** (`DEC-246`, `ISS-327`): `KeyWhen`
on `Declaration::WIRE_KEYS` (`src/design_run/submission.rs:598-611`) marks
`provenance`, `concerns` and `blocking` as `KeyWhen::Only(SubjectState::Absent)`,
and `Declaration::inert_at_state` (`submission.rs:794-805`) **refuses** them —
not drops them — when the run already holds the subject.

The rendered contract still says (`install/design-payload-contract.md:53,58,60`):

```text
  provenance  Provenance  optional
  concerns    id(sec-)    optional
  blocking    boolean     optional
```

So a payload that fully satisfies the published contract is hard-refused, for a
rule the contract does not state.

## Cause

Two tables describe the same keys. The state axis lives in
`submission::Declaration::WIRE_KEYS`; the rendering reads
`payload_contract::DECLARATION`. Each is pinned to serde's key set, neither to
the other, and only one is published.

## What keeps this an improvement rather than an issue

- The refusal itself is legible: *"`blocking` is inert at fnd-1: the run already
  holds it, and `blocking` is honoured only where the run does not yet hold it.
  Omit the key to leave the held value as it is."*
- No shipped skill or template hands an agent a payload carrying these keys —
  checked across `.agents/skills/` and `install/`.
- Contract legibility is an explicit Non-Goal of `SL-259` (`IMP-390`, `ISS-298`,
  `ISS-360`), so this joins that family rather than reopening the slice.

Raised on `RV-367` (code-review of `SL-259`) as `F-6`, disposed follow-up.
