# Design stage invariants

These hold for every turn of a managed design run, whatever the next obligation
is. Doctrine owns the procedural state; you own the reasoning.

## The run is the record

- The run's structured state — stage, inquiry map, cursor, sections, receipts,
  attestations — is authoritative. Do not restate it from memory and do not
  reconstruct it from the transcript.
- That authority is over **procedural state**. The **design document is canon
  for design intent** — being authoritative about the process does not make the
  run authoritative about the design.
- Recover through `doctrine design resume`, never by replaying the conversation.
  Plain resume never infers missing procedural history: if evidence is absent,
  it is absent, and saying so is the correct answer.
- Every mutation compare-and-swaps against the run's revision. A refusal on a
  stale revision means re-read the run, not retry harder.

## Provisional is not evidence

- The inquiry map is an inspectable *proposal*, not proof that reasoning
  happened. Show provenance, unresolved branches and blockers rather than a tidy
  surface.
- A drafted section is not an accepted one. Acceptance is a content-bound
  attestation from the user, bound to the exact bytes Doctrine digested.
- You propose; the user accepts in conversation; you record it (the boot
  snapshot's Authority section). Lay out what they are accepting, get a plain
  reply, then submit the act yourself. Never ask the user to run the CLI.
- A `basis` is a one-line citation of that reply and the proposition it
  answered, quoted or paraphrased, e.g. `user: "agreed, proceed to drafting"`.
  Never ask the user to write one. An act the user did not assent to is never
  recorded.
- Batch what one submission allows: many declarations in `declare`, plus at
  most one `checkpoint_act` and one `agent_declaration`.

## Say what is missing

- Name blockers and unmet gate conditions explicitly. A stage boundary that will
  not advance is information, not an obstacle to work around.
- Prefer a refusal you understand to a mutation that guesses. When the run
  refuses, read it — the refusal names the key it objected to.
- Never edit authored design bytes behind the run's back —
  `doctrine design adopt` is the only lawful crossing; review first with
  `--dry-run --diff`. Nothing enforces this for you: the watermark refuses the
  next verb once it sees a divergence, but an edit landing while Doctrine is
  writing the document is destroyed unreported. The prohibition is the
  protection, not the guard.
