# Doctrine conventions

The rules of the road a driver must honour while working a slice. These live in
the project's `CLAUDE.md` and `.doctrine/state/boot.md` — this points at them; read
those for the authoritative wording.

- **Frequent, slice-scoped conventional commits.** Scope every commit with the
  slice id: `fix(SL-004): …`, `doc(SL-005): …`, `plan(SL-005): …`. Commit small
  and often, on `main` (or the slice's worktree branch).
- **Pure / imperative split.** No clock, rng, git, or disk in the pure layer — pass
  them in as inputs (the date/uid pattern). Impurity lives in a thin shell. See
  [[concept.doctrine.entity-engine]].
- **Behaviour-preservation gate.** When you change shared machinery (the entity
  engine), the existing suites are the proof: they must stay green *unchanged*. A
  diff to a shared seam that needs its tests rewritten is suspect.
- **Immutable ids.** Phase ids (`PHASE-NN`) and criteria ids (`EN-/EX-/VT-`) never
  change. Corrections *append*; never renumber or reword in place.
- **Ask, don't infer.** Correctness comes first and last. Use the CLI as the source
  of truth for command shapes — don't guess ids, flags, or paths
  ([[fact.doctrine.cli-source-of-truth]]).
- **No parallel implementation.** Ride existing seams; find duplication before
  writing new code. Lint as you go (zero warnings; your project's lint-and-check gate before commit).

Honour the storage rule when you write artifacts — structured data in TOML, prose in
MD, never queried/derived data in prose ([[concept.doctrine.storage-model]],
[[fact.doctrine.storage-tiers]]). These conventions wrap the whole lifecycle
([[pattern.doctrine.core-loop]]). See [[signpost.doctrine.reference-docs]] for
the glossary of entity kinds and reference forms.
