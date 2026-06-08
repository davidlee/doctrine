# IMP-005: Normalise explicit --slug like a derived slug at entity new

Surfaced by the SL-024 code review (🟠) and deferred there as design OQ-1 / scope
Q3. SL-024 made user free-text non-corrupting to the scaffolded `*.toml` document
(render-escape through the `tomlfmt` seam), but escaping closes the **TOML wound,
not the slug wound**.

`title` flows only into a TOML literal — escaping fully secures it. `slug` flows
into the literal **and** the `<id>-<slug>` symlink filename. A derived slug is
normalised by `entity::derive_slug`; an explicit `--slug` bypasses that
(`input::resolve_slug` only checks non-empty). So a hostile `--slug` (a `"`, `/`,
newline) now round-trips the TOML cleanly but still detonates downstream at
symlink creation — the round-trip tests passing must not be read as "hostile slug
is safe end-to-end." It is not.

Scope:
- Normalise an explicit `--slug` through the same `entity::derive_slug` path a
  derived slug already takes (single normalisation seam — no parallel impl), so
  the slug is filesystem-safe before it reaches either the TOML or the symlink.
- Decide the policy: silently normalise vs. reject a slug that does not survive
  normalisation. (Reject is the more honest contract — a `--slug` the user typed
  that gets rewritten under them is surprising.)
- Cover every entity `new` that accepts `--slug` (adr/slice/spec/requirement/
  backlog), not just one.

This is a **live risk carried out of SL-024**, not a closed one — the render-escape
is necessary but not sufficient for slug safety.

Governing: SL-024 (design OQ-1).
