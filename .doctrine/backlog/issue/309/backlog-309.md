# ISS-309: Shipped assets cite repo-private ids that collide in client repos

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## The defect

Doctrine's shipped corpus — everything embedded under `install/` and published
through `doctrine library show` — cites **this repository's own entity ids and
file paths**. Those assets are read by agents working in *client* repos, where
neither resolves correctly.

The path half is an ordinary broken link. The id half is worse, and is the
reason this is an issue rather than a tidy-up.

**Entity ids are per-repo sequential.** A client repo mints its own `DEC-101`,
its own `SL-233`, its own `ADR-007`, counting from its own installation. So a
shipped asset citing `DEC-101` does not dangle in a client repo — `doctrine
knowledge show DEC-101` returns *a different, unrelated record*, with no error
and no signal that the citation was never about that record. A broken link
announces itself. This does not: it silently substitutes one claim for another,
in guidance an agent is being asked to act on.

## Confirmed instances

Sampled, not exhaustive — the sweep is part of the work.

- `install/design-prompts/inquiring.toml:21` cites `sketches/thin-adapter.md`.
  The file is `.doctrine/slice/233/sketches/thin-adapter.md`, in this repo only.
  There is no correct client-repo spelling of that path, so it cannot be fixed by
  repathing — the content must move or the citation must go.
- `install/design-prompts/exploring.toml` — `SL-233`, `DEC-101`, `IMP-372`.
- `install/design-prompts/inquiring.toml` — `SL-233`, `DEC-101`, `DEC-104`,
  `RV-325`.
- `install/dispatch-mechanics.md` — `ADR-005`, `ADR-006`, `ADR-008`, `ADR-011`,
  `ADR-012`, `ADR-019`, `ISS-234`, `SL-211`.
- `install/review-ledger.md` — `ADR-005`, `ADR-007`, `ADR-019`, `SL-147`.

Note the asymmetry worth checking during the sweep: `ADR-` citations may be
*less* wrong in practice than `DEC-`/`SL-`/`ISS-` ones, because a client repo is
less likely to have minted many ADRs — but "less likely to collide" is not a
property to rely on, and it fails silently in exactly the same way when it does.

Also in scope but different in kind: `install/glossary.md:116`,
`install/project-orientation.md:49` and `install/templates/seed-onboarding.md:50`
reference `.doctrine/spec/` and `.doctrine/adr/` **as directory conventions**.
Those are correct — every client repo has its own — and must not be swept up with
the rest. The distinction is *citing the client's structure* versus *citing this
repo's contents*.

## Why it has gone unnoticed

Nothing checks it. Publication validation confirms that a declared asset has a
backing and that a backing is reachable; it has no view of whether the asset's
*prose* refers to anything a reader can resolve. And the failure mode is
invisible from inside this repo, where every citation resolves perfectly.

## Shape of the fix

Two parts, and the second is what stops the drift returning.

1. **The sweep.** Decide per citation: inline the fact (usually right for a
   one-clause rationale), drop it, or replace it with something a client can
   reach — a published address under `reference/`, or a description that stands
   without the id. `sketches/thin-adapter.md` is the hard case: the reasoning it
   carries is load-bearing for two runbook steps, so it needs a home in the
   shipped corpus or an inlined summary.
2. **A check.** A lint over the shipped corpus refusing repo-private entity-id
   and path citations, run by `doctrine doctor` / `check gate`. Without it the
   corpus re-drifts on the next asset, because the defect is invisible to the
   author writing it.

Open: whether shipped assets should be able to cite *anything* durable, and if
so what the vocabulary is. A stable published address (`reference/<name>.md`) is
resolvable everywhere and is the obvious candidate; entity ids are not, in any
form, because the namespace is per-repo by construction.

## Origin

Surfaced during `SL-244`'s design run, from the constraint that decided `DEC-127`
— a client repo has no access to this repo's spec, so any repo-external consumer
or citation must reference an ensured-up-to-date published copy rather than a
private artefact. `DEC-127` establishes the rule for the asset it was deciding;
this item owns the existing corpus and the check.
