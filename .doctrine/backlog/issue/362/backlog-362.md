# ISS-362: Act admissible at the wrong stage silently no-ops

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## What

A `dispose` batch submitted at `stage = exploring` is **accepted**: exit 0, the
revision bumps, a receipt is written — and **no change rows** are produced. The act
is admissible in form but inert at the current stage, and the engine reports that
as success rather than as a typed refusal naming the stage the act requires.

## Why it matters

Stage is the design run's core sequencing constraint; an act at the wrong stage is
precisely what the state machine exists to catch. Reporting it as success inverts
the guarantee — the machine's own ordering rule fails open.

The cost compounds with `ISS-355` (*successful apply prints no change row*): with
no change rows on legitimate success either, an agent has **no observable at all**
that separates "landed", "absorbed" (`ISS-346`) and "inert at this stage" (this
issue). Three distinct outcomes, one indistinguishable response.

## Evidence

- `019fd226-ad55` — dispose batch at wrong stage accepted, no change rows

## Shape of a fix

A typed refusal naming both the current stage and the act's required stage, and —
importantly — **no revision bump**, since nothing happened. That last part matters
for `ISS-361`: a revision consumed by a no-op is a revision the caller cannot
account for.

Worth checking whether this is act-specific (`dispose`) or general to every act
whose stage guard is absent — `run.rs` dispatches on id kind alone for `declare`,
which suggests the guard is missing broadly rather than in one arm.

## References

- `ISS-355` — successful apply prints no change row
- `ISS-346` — design apply silently absorbs unknown payload keys
- `ISS-361` — unparseable submission applied and receipt-locked
- `QUE-219` — submission strictness against snapshot forward-compatibility

## Premise disproved — struck from SL-259, 2026-09-14

Reproduced on `SL-259`'s own design run at `stage = exploring`, against
`./target/debug/doctrine` at revision 5–7:

```text
cp- dispose  at exploring → exit 0, revision 6, row: checkpoint_disposed cp-1 node=inq-2 disposition=non-durable
sec- body    at exploring → exit 0, revision 7, row: section_created sec-1 fingerprint=7324891ee255
```

Both acts were **honoured**, not inert: the node resolved, the section exists,
and each emitted its change row. The original witness inferred inertness from
the absence of change rows — but that absence was `ISS-355` (*successful apply
prints no change row*), which `SL-256` has since fixed. With rows printing, the
act is visibly landing.

Two consequences.

1. **There is no silent no-op here to fix.** The issue as written describes
   behaviour this codebase does not have, so it is struck from `SL-259`
   (*Truthful apply*) rather than carried as one of its legs.
2. **A different, smaller question survives it**: these acts are honoured at
   *every* stage, so stage is not a guard on them at all. Whether it should be
   is a scoping question about the state machine, not a correctness defect in
   apply's exit signal — and it is deliberately out of `SL-259`'s scope.

One detail of the original report does still hold: a `dispose` aimed at an
`inq-` subject **is** refused (`SL-249`'s key-home table), naming the subject
kind it is honoured for. Resolution routes through a `cp-` subject carrying
`disposes`. That is a refusal working, not the failure reported here.
