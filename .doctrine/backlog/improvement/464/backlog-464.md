# IMP-464: Locking a design should be one command, not eleven

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## The friction

Locking a design run whose review policy is `human-only` costs the human **one
`doctrine design apply` submission per section, plus two more** — nine sections
on `SL-246`, then `review-disposed`, then `design-accepted`. Eleven hand-built
JSON payloads, each of which must carry a fresh `known_revision` because every
apply bumps the revision by one, and each of which is a `declare` whose only
substantive content is `{subject, attests, reviewer}`.

The act being recorded is *"I read this and I stand behind it."* Nothing in the
eleven payloads carries more information than that, repeated nine times with an
index changed.

Observed on `SL-246` at revision 61→62, 2026-09-19. The user's words: *"this
should amount to: read the materialised doc, tell the agent I've read it (or, in
the absence of a mechanism that stops an agent forging it, execute a single
command)."*

## Why it is this shape, and where the shape is load-bearing

Per-section attestation is not decoration. `DEC-073` binds an attestation to the
section's content fingerprint, and the `section-attestations-current` contract
exists to catch the specific failure of *a design that passes every check while
carrying one part nobody read* — the thing whole-document review reliably misses,
because attention spreads over whatever unit the process asks for.

So the granularity is right and the **submission count is not derived from it**.
Those are separable. A single submission can carry nine attestations; a single
command can build them. What must survive any repair:

- each attestation still binds *its own* section fingerprint, so a later edit to
  one section voids one attestation and not the set;
- the human still has to have seen each section, or the gate is theatre;
- forgeability does not get worse than it already is (an agent drives the CLI
  either way — see `ISS-310`, `sections_attested` ignores reviewer identity).

## What is already there, and therefore what is actually missing

**Batched declaration already works.** `ApplyRequest.declare` is a
`Vec<Declaration>` and the apply path validates the whole set as one batch —
`src/design_run/run.rs:377`, `Batch::of(declarations).validate(…)`. Nine
attestations fit in one submission today. Nothing was missing; nothing told
anyone.

That reclassifies the defect. It is not a missing capability, it is:

- **discoverability** — the contract (`doctrine design contract --format prompt`)
  prints the payload *types* and says nothing about which shape performs an
  attestation, nor that the array is a batch. Both facts were found by trial;
- **the `known_revision` treadmill** — real, and the thing that made the ritual
  look eleven-deep in the first place, since a per-section submission forces a
  re-read of `design show` line 1 between every call;
- **the two closing acts** — `review-disposed` and `design-accepted` are
  `checkpoint_act`, singular, so they stay two more bespoke payloads.

## Candidate repairs, cheapest first

1. **Say so.** The contract, the `reviewing` obligation text, and the `/design`
   skill should all name the batch form and the `att-` subject shape. Zero code.
2. **A verb that does it.** `doctrine design attest <SLICE> --sections all`
   (or `1-4,7`), reading the current revision itself rather than making the human
   re-read `design show` line 1 between each call. Subsumes (1) behind a name a
   human can remember.
3. **Pair it with the read.** The complement of `IMP-430` (per-section read verb):
   if a verb can show section N, the same verb family can attest what it just
   showed — read-then-attest as one motion, which is what the user actually
   described.
4. **Fold in the two closing acts** so `review-disposed` and `design-accepted`
   are not two more bespoke `checkpoint_act` payloads.

## Incidental friction found on the way (cheap fixes, same area)

- An `att-` subject refuses both `summary` and `body` as inert (`summary` is for
  `fnd-`, `body` for `sec-`). So the attestation records the binding and **no
  reasoning** — the human's grounds for signing are unrecordable at the point of
  signing. Refusal messages are good; the gap is real.
- `known_revision` must be re-read between every submission. The replay window
  tolerates some staleness (it starts at `revision 31` here), but the human has no
  way to know that without reading the refusal.
- The contract (`doctrine design contract --format prompt`) documents the payload
  types but not *which* shape performs an attestation; that it is `declare` with
  an `att-` subject rather than `checkpoint_act` with `act: section-reviewed` was
  found by trying both.

## Relations

- `IMP-430` — per-section read verb; the read half of the same motion.
- `ISS-310` — `sections_attested` ignores reviewer identity; bears on the
  forgeability constraint above.
- `PRD-019` — managed design workflow, the product this sits in.
- `RFC-011` — dispatch token efficiency; eleven hand-built payloads is a
  measurable instance.
