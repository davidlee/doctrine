# Doctrine entity engine

Every authored artifact type in doctrine — slices, ADRs, specs, requirements,
and memories — rides one shared entity engine rather than a bespoke
implementation each. When you add or change an artifact, you are working *with*
this engine, not around it.

What the engine gives every entity:

- **Identity + a claim seam** — a stable uid minted once, plus the
  born-frame/claim mechanism that records where an entity came from. The impure
  capture lives behind a thin seam; the pure layer never touches
  clock, rng, git, or disk directly (the date/uid pattern).
- **Relations and edges** — typed references between entities live in the small
  sister TOMLs (e.g. `slice-nnn.toml`), never in prose bodies, so a registry can
  parse them cheaply. This is the same TOML/MD split the storage rule mandates
  ([[mem.concept.doctrine.storage-model]]).
- **The behaviour-preservation gate** — the engine is shared machinery. When you
  change it, the existing test suites are the proof of correctness: they must
  stay green *unchanged*. Don't rewrite a passing suite to accommodate an engine
  change; that defeats the gate.

Don't build a parallel implementation for a new entity type — ride the existing
seam and look for duplication first.

Point of truth: `reference/using-doctrine.md` for the one storage rule and the
TOML/MD split every entity rides. See
[[mem.signpost.doctrine.file-map]] for the layout and [[mem.pattern.doctrine.conventions]]
for the pure/imperative split and no-parallel-implementation rules. For the
relation-authoring surface, see [[mem.signpost.doctrine.relating-entities]].
The concept map ([[mem.signpost.doctrine.concept-map]]) provides a visual
overview of these entity relationships.
