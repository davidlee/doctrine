# IMP-302: Retire ADR-001 install::asset_text upward wart — redirect callers to leaf asset_source

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Context

Surfaced/confirmed by the SL-223 audit (RV-288). ADR-001 layering.toml:180
documents a pre-existing upward wart: engine/leaf callers reach *up* into
`install::asset_text` for scaffolding byte reads. SL-223 extracted the byte-read
into the new leaf `asset_source` (PHASE-01, D-B) and left `install::asset_text` a
**delegating shim** — a deliberate minimal-blast-radius scope boundary, not the
full cleanup.

## The work

Redirect the ~30 engine/leaf callers currently reaching into `install::asset_text`
to the leaf `asset_source` read seam (engine→leaf / leaf→leaf, downward), so the
upward edge in ADR-001 layering.toml can retire. Keep `install::asset_text` as a
thin delegating shim only where a genuine install-tier caller remains, or remove
it once no upward caller is left.

## Why deferred (scope boundary)

SL-223's D-B kept the shim to hold REQ-376's neutral-ownership criterion honestly
met without a wide refactor blast radius. The redirect is orthogonal to the
publication seam and touches many call sites — a clean standalone improvement,
not SL-223 scope. layering.toml:180 already names this follow-up as the retirement
path.

## Done when

- No engine/leaf module imports `install::asset_text` for a byte read.
- The ADR-001 layering.toml upward-edge annotation (line ~180) is removed or
  downgraded; the layering gate (`tests/architecture_layering.rs`) stays green.
- `install::asset_text` is either gone or a shim used only by install-tier callers.
