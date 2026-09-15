# A VT's `test_file` is a plan-time guess

`plan.toml`'s `VT` rows carry `test_file` + `keywords`, and `vtgate`
(`src/vtgate.rs`) greps that literal path: a test homed elsewhere reads `Fail`,
not `Pass`. So the mandate feels binding at execution time. It is not — it was
authored **before the code existed**, as a prediction of where the behaviour
would be observable.

## The trap

`SL-259` `PHASE-03` had four of five mandates naming
`src/design_run/payload_contract.rs`. Deferring to them would have sited a
**value-judging walk** inside the largest production module in its tree (4015
lines) and the one whose own doc calls it a *describing* module that "does
nothing to a run" — because a grep string said so. The human called it: *"that
feels like the tail wagging the dog."*

## The move

Amend the mandate, at phase-plan time, before writing code.

- Changing `test_file` is a **field edit, not a renumber** — `PHASE-NN` and
  `VT-N` ids stay, `expects` prose and keywords stay. The immutability rule
  binds ids, not every field.
- Record the retarget and its reason in `plan.md`'s verification-posture
  section, so an auditor reads a decision rather than a discrepancy.
- Re-run `doctrine slice verify-vt <id>` after, and expect `UNATTRIBUTABLE`
  until the phase's delta is recorded
  ([[mem_019f89125fb275a2895bf58b5e29ed95]]).

## Retarget in both directions, and check per criterion

The same slice retargeted twice for opposite reasons, which is the tell that
this is a rule rather than an incident:

- `PHASE-01` moved three mandates **to** a file, `run.rs` — `Pending`'s fields
  are private to it, so no test outside could observe the rows at all.
- `PHASE-03` moved four **off** a file, on cohesion.

Judge each criterion separately. `PHASE-03`'s `VT-4` correctly **stayed** on
`payload_contract.rs`: it asserts every contract row declares
`UnknownKeys::Refused`, which is a claim about the table's own
self-description, not about any payload. That the five split cleanly along the
seam being argued was itself evidence the seam was real.

## The general form

Observability is the criterion — *where does this behaviour become visible?* —
and it is knowable at phase-plan time in a way it was not at plan time. Where
observability and cohesion agree, follow them. Where the authored mandate
disagrees with both, the mandate is the stale artefact.

Related: [[mem_019fdf3ffdd175c18a6e27616694a0b2]] (which *phase* owns a VT
title — the same question on the other axis),
[[mem_01a08024f2a57f60b7606dbd24b48acf]] (a keyword must be a literal the file
will actually contain).
