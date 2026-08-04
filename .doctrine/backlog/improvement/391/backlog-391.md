# IMP-391: Build the exploring-stage user checkpoints

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`DEC-121` decides that `exploring → inquiring` is guarded by two attested user
checkpoints — a governance confirmation and an inquiry-graph review with a
blocking-set declaration — replacing the two claimed conditions that stood in for
an interaction `SL-233` specified in a bullet and never built.

`SL-244` specifies those interactions, their contracts, and their artefact shapes,
and ships the diagrams. **This item builds them.** It arrives with its contract
already written, which is the whole reason the split is worth making.

## Narrowed 2026-08-04 — `DEC-139`

The first two bullets below **arrive done**. `RV-344`'s `F-3` found that
`SL-244` ships them anyway: its `sec-4` specifies the wire acts
(`GovernanceConfirmed`, `GraphReviewed`, `BlockingSetDeclared`) and the artefact
storage for each, so the slice was doing this item's work and still declining to
turn the guard on. `DEC-139` turns it on — both `exploring → inquiring`
conditions are enforced from `SL-244`.

So this item is **the interaction, not the mechanism**: the runbook prompting,
the CLI rendering, and the empty-case affordance. The two struck bullets are kept
below rather than deleted, because what they describe is the contract this item's
interaction has to serve.

## What is here

- ~~The two checkpoint acts on the wire~~ — **done in `SL-244`**, riding
  `DEC-088`'s content-bound attestation rather than a new primitive.
- ~~The artefact each produces as structured state~~ — **done in `SL-244`**: the
  governance edge set; the reviewed graph plus the confirmed blocking set,
  recorded so the condition derives over it (`DEC-120`'s 2026-08-03 sharpening:
  the artefact is what makes a subjective act derivable).
- The runbook steps in `exploring.toml` that prompt each interaction.
- Empty-case handling: a governance sweep finding nothing and an exploration
  raising no question are the strict paths, shown with what was searched, never
  skippable. Note that `SL-244` already enforces the *admission* half — a
  `GovernanceConfirmed` with an empty basis is refused on write, by the same rule
  that refuses an empty acceptance basis. What is left here is the affordance:
  making the strict path something a user can take deliberately rather than
  discover by refusal.
- CLI rendering of each artefact, so the user reads it from the tool rather than
  through an agent's paraphrase.

## The interim state this closes

The edge is **guarded** from `SL-244`; what is unfinished is how well it is
served. Between `SL-244` closing and this landing, no runbook step prompts either
checkpoint and no CLI renders either artefact, so the user confirms an artefact
the agent paraphrases. `DEC-121` wants better than that, and this item is where
it arrives. Superseded reading: an earlier version of this section said
`exploring → inquiring` passes on the runbook alone until this item lands, which
was true of the original scope split and is not true after `DEC-139`.

One consequence of the enforcement landing early belongs here rather than in the
slice: both conditions are cumulative, and no run holds an exploring-stage act or
can acquire one retroactively, so every in-flight run is barred at its next
forward move until the acts are given by hand. `SL-243` is the only such run.

Related: `DEC-121`, `DEC-120`, `DEC-088`, `ISS-285`, `SL-244`.
