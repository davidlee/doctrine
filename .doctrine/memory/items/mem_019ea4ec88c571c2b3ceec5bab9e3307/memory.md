# Tech-spec descent relation is named descends_from, not realises

The cross-family relation from a `tech` spec to the product spec (`PRD-NNN`) it
descends from is stored as a single-valued outbound scalar field
**`descends_from = "PRD-NNN"`** on `spec-NNN.toml` — NOT `realises`.

`realises` was proposed and rejected (twice): it overclaims. Code *realises*
product intent; a technical spec *describes the how* — it does not realise
anything. The relation is structural descent (the *what* → the *how* lineage),
which PRD-012's own language already calls "descent" / "descends from". Plain
English "realises" in prose is fine; as a relation/field name it is wrong.

The objection first surfaced during PRD-012 authoring but was never recorded, so
it resurfaced when SL-022's design reached for the field name. PRD-012 OQ-1
(resolved) explicitly left the concrete key/serialisation to the technical spec +
design — so the field name is SL-022's to fix, and it is `descends_from`.

Direction and shape follow ADR-004: outbound on the tech spec, single-valued, the
reverse view ("which tech specs descend from this PRD") derived by registry scan,
never stored. Validated by `spec validate` (target must resolve to a *product*
spec); rendered outbound-only by `spec show`.

Note: REQ-082's title still reads "...the product capability it realises" (PRD-012
prose). Reconciling that wording is a PRD-012 follow-up, not part of the field
naming. See `mem.system.spec.composition-seam` for the surrounding spec edge model.
