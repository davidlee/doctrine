# Spec [[source]] anchors rot silently — validate never checks them; identifier is path-form, module is the Rust path

Nothing verifies a spec's `[[source]]` anchor still resolves — `spec validate`
FK-checks members/interactions only, so a spec anchoring a vanished path reports
clean and misreads as covered. Check with `test -e` before trusting an anchor as
coverage.

## Why it matters

An anchor is the only mechanical binding from a tech spec to the code it governs,
so a spec-coverage assessment naturally treats "has an anchor" as "is covered".
That inference is unsound. Anchors are descriptive, never enforced: no validate
leg, no doctor check, no test asserts the path exists. A spec can anchor a file
deleted a dozen slices ago and still print `validate: SPEC-0NN clean`.

Measured at CHR-046 (2026-07-25): **72 anchors across 26 tech specs, 9 broken**
(87.5% live). All 26 specs validated clean throughout.

## The two failure classes — they need different fixes

**Genuine rot** — the path moved or died. Found: five specs (SPEC-006/007/008/
010/014) each anchoring a `doc/*-spec.md` in a docs tree that no longer exists,
plus SPEC-010's `src/skills.rs`. Fix: repoint or drop.

**Malformed identifier form** — the code is fine, the field is wrong. The
convention is `identifier` = repo-relative **path**, `module` = the **Rust path**:

```toml
[[source]]
language = "rust"
identifier = "src/map_server/mod.rs"      # path-form
module = "doctrine::map_server"           # Rust path
```

SPEC-020 was the sole deviation of 26, writing the module path into `identifier`
and duplicating it in `module` (`identifier = "doctrine/estimate"`). Every file
existed; only the form was wrong, so the anchors resolved to nothing and SPEC-020
silently misreported as covered. Fixed in 92d46941. 64 of 72 anchors already used
path form — treat path-form as the convention, not a preference.

Distinguishing the classes matters because a raw dead-anchor count overstates the
damage and points at the wrong repair.

## How to apply

- Auditing spec coverage: sweep every anchor for existence **before** counting any
  as coverage. One line:
  `grep -A3 '^\[\[source\]\]' .doctrine/spec/tech/*/spec-*.toml | grep '^identifier' | cut -d'"' -f2 | while read -r p; do [ -e "$p" ] || echo "DEAD $p"; done`
- Authoring a new anchor: path-form in `identifier`, Rust path in `module`.
- Anchors are declared at **module-root granularity** — SPEC-001 anchors only
  `src/priority/mod.rs` for 21k loc; SPEC-025 anchors `web/map/src/app.ts` for the
  whole SPA. Unanchored sibling files inside an anchored module are covered, not
  dark. A per-file anchor audit manufactures false gaps; don't run one.
- Repairing an anchor on an **active** spec is a direct edit, not a REV — precedent
  e65a7602e (IMP-295) repaired SPEC-001/012/019/021/022 that way. ADR-013's
  revision route governs governance→work dependency, and
  [[mem.fact.revision.spec-prose-modify-target]] covers spec-prose *decision*
  amendments; an identifier repair is neither.

## Worth mechanising

A `doctrine doctor` check for anchor liveness plus identifier form would make this
mechanical instead of a census someone has to remember to run. Prior art shows the
need: e65a7602e repaired anchors found by an earlier census and still left these 9
behind, because nothing re-checks. Captured against IMP-295 (spec-coverage
support, deterministic-aids axis).

Related: [[mem.concept.doctrine.reading-entities]] (two-tier reads — an anchor
lives in the TOML tier and never surfaces in prose).
