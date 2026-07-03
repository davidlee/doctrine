# Doctrine hymn cascade — bands, trait-keyed model band, seal/expose, precedence

The hymn cascade composes agent-facing prose from a corpus of small snippets.
Getting the authoring model wrong (treating the model band as a model registry,
or the provenance tiebreak as a suppression mechanism) produces content that
mis-describes its own machinery. The corpus corrects to this model.

## Bands — closed, fixed order

`preamble · harness · model · role · stage · project` — a closed registry, in
fixed order. A band decides WHERE a snippet lands in the composed output; it
never enters matching. A selector may pin ANY axis regardless of its own
snippet's band (a `role/worker` snippet may still pin a model trait). Bands
are closed; the trait trees WITHIN the model band are open — extending the
model vocabulary never touches the band registry.

## The model band is a trait vocabulary, not a model registry

Model identity (`vendor/name`) is a leaky proxy for a trait tuple: it does not
carry to the next model release and mis-fires when a model's real traits
diverge from its vendor path. The honest axes are TRAITS, authored as
orthogonal trees within the model band: `adherence/*` (e.g. `adherence/low`),
`capability/code/*`, `capability/reasoning/*`. A `vendor/name` identity key
stays expressible — it is just another path — but carries no privileged
semantics. Keys are opaque to the engine; meaning lives entirely in the
authored corpus, and the vocabulary is user-definable, never a central
registry. A worker definition declares its trait-key set in frontmatter
(assigned, never self-asserted for adherence).

## Path → slot, sidecar overlay

`<band>/<label>.md` → slot `{band, label}`. The label is the full relative key
under the band — a model label (`anthropic/claude-sonnet-4`) or a trait path
(`adherence/low`) alike. A sidecar `<file>.toml` overlays selector axes
per-axis; an axis it does not declare keeps its path-derived default.

## Seal / expose — two sides of one coin, both single-emit

Two roots layer: `install/hymns/` (compile-embedded, the framework superset)
unioned with `.doctrine/hymns/` (on-disk, user overlay + projected editable
starters). Provenance is DERIVED from the source root, never stored as a
flag.

- **Sealed** slot: the framework snippet is authoritative — any
  user-provenance snippet at that slot is dropped BEFORE matching. Framework
  wins by active exclusion of its disk twin.
- **Exposed** slot: the projector writes an editable starter to disk AND
  writes a sidecar `replaces = <own slot>`. The user twin self-`replaces` its
  own framework origin, making it the unique strict top of the slot — user
  wins by SUPPRESSION, not by the provenance tiebreak below.

`replaces` is the ONLY suppression mechanism, legal only on the unique
most-specific active snippet of its slot (self-targeting is exactly the
expose mechanism). Duplicate targets, non-top replacers, and cycles are
authoring errors surfaced by `prompt check`/`validate` — never silently
alpha-ordered.

## Precedence key — provenance is ordering, not suppression

Ascending; last word wins:

    band → specificity → provenance(framework < user) → alpha(full slot path)

Specificity dominates provenance — a framework exact-trait snippet outranks a
user broad one. Within a band, specificity leads with the band's namesake
axis, then the sum of other pinned axis depths. The same-slot provenance leg
is ORDERING ONLY: it reorders same-slot twins, it never suppresses one. The
exposed starter's override of its framework origin is delivered by
self-`replaces` (the expose mechanism above), NOT by winning this tiebreak —
there is no "user always wins at equal specificity" rule.

Point of truth: `.doctrine/spec/tech/023/spec-023.md` (SPEC-023, "Prompt
cascade") for the full engine contract, and `install/hymns/README.md` /
`.doctrine/hymns/README.md` for the authoring quick-reference this memory
summarises.
