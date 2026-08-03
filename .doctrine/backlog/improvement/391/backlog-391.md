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

## What is here

- The two checkpoint acts on the wire, riding `DEC-088`'s content-bound
  attestation rather than a new primitive.
- The artefact each produces as structured state — the governance edge set; the
  reviewed graph plus the confirmed blocking set — recorded so the condition
  derives over it (`DEC-120`'s 2026-08-03 sharpening: the artefact is what makes a
  subjective act derivable).
- The runbook steps in `exploring.toml` that prompt each interaction.
- Empty-case handling: a governance sweep finding nothing and an exploration
  raising no question are the strict paths, shown with what was searched, never
  skippable.
- CLI rendering of each artefact, so the user reads it from the tool rather than
  through an agent's paraphrase.

## The interim state this closes

Between `SL-244` closing and this landing, `exploring → inquiring` passes on the
runbook alone. That is stated and accepted, not accidental — it is no worse than
the status quo, where the two conditions were satisfied by an unexamined claim
against an unrelated subject.

Related: `DEC-121`, `DEC-120`, `DEC-088`, `ISS-285`, `SL-244`.
