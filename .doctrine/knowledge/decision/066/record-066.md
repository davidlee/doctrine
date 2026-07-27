# DEC-066: Design stages enforce only load-bearing boundary gates

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

The coarse design-run stage mechanically gates only load-bearing forward
boundaries:

- `exploring → inquiring`: a concern frontier exists and required context checks
  are acknowledged; research drift is surfaced under its advisory contract.
- `inquiring → drafting`: the user accepted the design basis, no blocking
  inquiry remains open, and every resolved inquiry is dispositioned.
- `drafting → reviewing`: declared sections are aligned, `design.md` is
  materialised from that basis, and known scope/design divergence is absent.
- `reviewing → locked`: adversarial review exists, every finding is
  dispositioned, scope/relations are reconciled, and the user explicitly
  approves lock.

Within-stage sequencing remains advisory model judgement: question selection,
branching, option generation, section order, and conversational loops are
obligations and recommendations rather than additional FSM transitions.

Back-edges are supported and reasoned; their invalidation and re-clearance
semantics are resolved separately by QUE-186.
