# IMP-126: Dispatch [dispatch] trunk_preference config overrides trunk-ladder freshness default

Surfaced by SL-127 design, codex review finding **C1** (ladder neutrality).

SL-127 changes `git::trunk_ladder` from literal `origin/HEAD`-first order to a
**freshest-descendant** fold (advance to a later candidate only if it descends the
current pick). This fixes the witnessed local-first base-staleness (origin/HEAD
lags local `main`), but `most-advanced-wins` is a *default policy opinion*: a
consumer who treats `origin/HEAD` as the authoritative trunk yet runs it *behind*
local `main` would get `main` as the base. SL-127 accepts this residual — the
escape hatch is an explicit `DOCTRINE_TRUNK_REF` per command — and records the
stance in the ADR-006 D3 amendment.

**This item** is the durable, neutral fix: a `[dispatch] trunk_preference`
config field (in `doctrine.toml`) that names the authoritative trunk ref (or the
selection policy) once, per project, so neither the freshness default nor the
env-prefix workaround is load-bearing for a consumer with a different trunk model.

Overlaps **IMP-124 / IMP-101** (`[dispatch] deliver_to` — single-source trunk
delivery ref); likely the same config surface. Consider folding the two: one
`[dispatch]` trunk-ref source feeding both ladder *selection* and integrate
*delivery*. Bears on SL-127 (the freshness default this overrides) and ADR-006 D3.
