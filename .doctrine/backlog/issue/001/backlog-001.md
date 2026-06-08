# ISS-001: DRY the three backlog test-fixture TOML builders into one helper

Surfaced as SL-020 audit finding F5 (🔵, tolerated drift). Test-only; no
behaviour impact.

Three test helpers in `src/backlog.rs` each hand-assemble a `backlog-NNN.toml`
string with overlapping-but-different field sets:

- `write_item` (`tests`) — id/slug/title/kind/status/resolution/tags
- `write_assessed_risk` — adds the `[facet]` + `[relationships]` blocks
- `write_related` — adds `[relationships]` only

Same `id = …\nslug = "…"\n…` boilerplate three times. A schema change means
editing three string templates. Fold into one parameterized builder (CLAUDE.md:
"build & improve test helpers"). Keep each call-site readable — the variation is
the facet/relationships tail, so a base + optional-blocks shape fits.

Note: taxonomically closer to a `chore` (maintenance, no user-visible change)
than an `issue`; captured as `issue` per request. Kind is fixed at capture.
