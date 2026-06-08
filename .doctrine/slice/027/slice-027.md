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
3. All call sites move to the unified builder. Existing assertions are not
   touched beyond the mechanical call-site swap; the suite stays green.

Affected surface (concrete): the `#[cfg(test)] mod tests` block of
`src/backlog.rs` only — the three builders above and their call sites.

## Non-Goals

- **No production-code change.** `render_backlog_toml` / `backlog_scaffold` and
  the rest of the non-test surface are untouched.
- **Not reusing the production renderer as the fixture source.** The production
  scaffold seeds only slug/title/date; tests need arbitrary
  status/resolution/tags/facet/relations, so a test-local builder remains the
  right tool. (If a clean reuse seam surfaces during design, `/consult` — do not
  improvise it here.)
- **No new fixture capabilities** beyond what the three current helpers express
  — this is consolidation, not feature growth.
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
- The core TOML head + list-literal quoting exist in exactly one place;
  `grep` for the duplicated `created = \"2026-06-08\"` literal returns a single
  builder.
- ISS-001 transitioned to its resolving state at `/close`.
