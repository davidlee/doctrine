# DRY backlog test-fixture TOML builders into one helper

## Context

Captured as **ISS-001** (`backlog-test-fixture-dry`) out of SL-020 (Backlog
entity v1). The backlog unit-test module in `src/backlog.rs` carries three
hand-rolled helpers that each build a `backlog-NNN.toml` fixture string by raw
`format!` and write it to a temp tree:

- `write_item` (`src/backlog.rs:1417`) — the base item: core field head +
  `tags`, no facet, no relationships.
- `write_assessed_risk` (`src/backlog.rs:1928`) — a risk item: core head +
  `[facet]` (likelihood/impact/origin/controls) + empty `[relationships]`.
- `write_related` (`src/backlog.rs:1943`) — an item carrying seeded outbound
  `slices`/`specs` under `[relationships]`.

All three duplicate the same core TOML head —
`id`/`slug`/`title`/`kind`/`status`/`resolution`/`created`/`updated`/`tags` —
the `2026-06-08` date literals, the `backlog-{name}.toml` path computation, and
a `"quote"`-and-join list-literal closure (`write_item`'s `tags_lit`,
`write_related`'s `lit`). They diverge only in the optional `[facet]` and
`[relationships]` trailers. That triplication is the debt ISS-001 names: a
field added or renamed on the backlog schema means editing the literal in three
places, and the closures are copy-pasted.

This is a **test-only** refactor. No production code changes; no behaviour
changes. The existing assertions are the proof — every backlog test stays green
**unchanged** (behaviour-preservation gate).

## Scope & Objectives

Collapse the three fixture builders into **one parameterised TOML-fixture
builder** in the `src/backlog.rs` test module, such that:

1. The core field head, the date literals, the path computation, and the
   list-literal quoting live in exactly **one** place.
2. The optional `[facet]` and `[relationships]` trailers are expressed as
   parameters of the single builder (present → emitted, absent → omitted),
   covering every shape the three current helpers produce.
3. The three named helpers survive as thin wrappers delegating to the core; their
   ~30 call sites are unchanged. A **fourth** head-copy — an inline literal at
   `src/backlog.rs:1813` (`backlog_show_json_is_faithful_item_state`), a
   fully-assessed risk the old narrow helpers could not express — folds into the
   unified builder (the one call-site change). Existing assertions are untouched;
   the suite stays green.

Affected surface (concrete): the `#[cfg(test)] mod tests` block of
`src/backlog.rs` only — the three builders, the `:1813` inline literal, and (for
the wrappers) their unchanged call sites.

## Non-Goals

- **No production-code change.** `render_backlog_toml` / `backlog_scaffold` and
  the rest of the non-test surface are untouched.
- **Not reusing the production renderer as the fixture source.** The production
  scaffold seeds only slug/title/date; tests need arbitrary
  status/resolution/tags/facet/relations, so a test-local builder remains the
  right tool. (If a clean reuse seam surfaces during design, `/consult` — do not
  improvise it here.)
- **No new fixture capabilities** beyond what the three current helpers plus the
  `:1813` literal express — this is consolidation, not feature growth.
- **Inline parser / error-path literals stay explicit.** `:1161` (in-memory
  `toml::from_str` round-trip), `:1190` (unknown-enum error), and `:2075`
  (malformed-edit) feed bytes directly to the parser and must show those exact
  bytes — they are not fixture builders and do not move.
- **No change to the e2e fixture** in `tests/e2e_backlog_filter_alias.rs` (a CLI
  driver, not a TOML builder).

## Risks / Assumptions / Open Questions

- **R1 — over-parameterisation.** A single builder with too many positional args
  becomes less readable than three named helpers (cf. the repo's
  bool/arg-ceiling clippy lints, `mem.pattern.lint.cli-handler-args-struct`).
  Mitigation: shape the API in `/design` — likely a small fixture struct with
  defaults + builder-style overrides, or a base builder plus thin trailer
  composition, rather than one mega-signature.
- **R2 — readability regression at call sites.** The named helpers document
  intent (`write_assessed_risk`). The unified form must keep call sites
  self-explaining; thin named wrappers over the core builder are acceptable if
  they remove the duplication that matters (the TOML literal), not just rename
  it.
- **A1 — assertions are behaviour, helpers are implementation.** Refactoring the
  helpers without touching assertions preserves tested behaviour by definition.

## Verification / Closure Intent

- `cargo test` backlog suite green, assertions unchanged.
- `cargo clippy` zero warnings (`just check`).
- The core TOML head + list-literal quoting exist in exactly one place; the
  `created = \"2026-06-08\"` literal drops from 7 → 4 occurrences (one unified
  builder + three deliberately-explicit parser/error fixtures).
- ISS-001 transitioned to its resolving state at `/close`.
