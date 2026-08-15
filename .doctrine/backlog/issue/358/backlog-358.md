# ISS-358: Research baseline restamps on every scope edit

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## What

The `explore.research` step hashes `slice-NNN.md` and `design.md` as its staleness
baseline. The managed design run **legitimately edits both** — that is what
`inquire.scope` and `materialise` do. So discharging a scope step regresses the
research step **by construction**, and the agent owes a `--restamp` each time.

## Why it matters

Two costs, and the second is the real one.

1. **A treadmill.** Every normal forward move in the run manufactures a staleness
   warning that must be manually cleared. The cost is per-revision, and the runs
   that do the most design work pay it most.
2. **The signal is lossy.** A genuine research invalidation (the code moved under
   the research finding) and a benign scope edit (the run rewrote a sentence in
   `slice-NNN.md`) are **indistinguishable** at the baseline. Once an agent has
   restamped past a dozen benign warnings, it will restamp past the real one. The
   advisory trains the behaviour that defeats it.

## Evidence

- `019fd6a7-d40b` — research-baseline restamp treadmill (run edits slice card)
- `019ff8b4-a819` — `inquire.scope` regresses `explore.research` (one edits what the other hashes)

Adjacent, same mechanism, different gate — the sweep's boundary set:
`019fdf00-30b0`, `019ffb22-c247`, `019fe0db-748c` (research staleness advisory
fires on normal progression through `/plan` / `/phase-plan`).

## Shape of a fix

The baseline is hashing the wrong thing. Research findings are invalidated by
**source movement**, not by prose movement in the artefacts the run authors.
Candidates, cheapest first:

1. **Narrow the baseline** — hash only the code/spec paths the research threads
   actually read, not the slice card and design doc.
2. **Section-scope it** — hash only the `slice-NNN.md` sections research consumed.
3. **Separate the axes** — keep a prose-drift signal advisory and a source-drift
   signal blocking.

## Scope note

Not `cluster:design-run`. The defect is in the **research stage's** baseline
contract; the design run is merely the loudest trigger. Tagged `area:research`.

## References

- `SL-229` — Pre-design research stage v1 (where the baseline was introduced)
- `IMP-314` — research artefact has no harvest pointer at close
