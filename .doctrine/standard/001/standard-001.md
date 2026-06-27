# STD-001: No magic strings — single-source named constants

<!-- Body sections reuse the tuned prior art from spec-driver/supekku
     templates/standard-template.md; its YAML frontmatter is dropped — metadata
     lives in the sister standard.toml (storage rule / design D1). -->

## Statement

A literal value with meaning — a path, key, ref prefix, env-var name, format
token, magic number, sentinel string — is **named once** and referenced by that
name everywhere. Do **not** hand-type the same literal at two sites. When a
constant for a value already exists, **use it**; do not paste the raw literal
beside it.

Authors of magic strings will be sentenced to drowning as witches.

## Rationale

A named constant is the **single point of change**. The moment a literal is
duplicated it becomes N independent points that drift apart silently — and
nothing flags the divergence, because each copy looks locally correct.

The motivating heresy: `"doctrine.toml"` was hand-typed at ~20+ sites while a
`DOCTRINE_TOML` const sat right beside them, unused by the stragglers. The
config path could not be changed in one place, so it fractured into two homes
(root vs `.doctrine/`) read by different subsystems — a defect invisible until a
user's edit silently did nothing ([[ISS-055]]). Magic strings are not a cosmetic
nit; they **defeat the very single-point-of-change that prevents this class of
bug.**

## Scope

Applies to: all source under `src/` (and tests). Covers string and numeric
literals that carry meaning and recur, or that a named constant already exists
for. **Excluded:** genuinely one-shot, self-evident literals with a single call
site and no sibling constant (e.g. `0`, `""`, a format string used once). The
test is recurrence + existing-name, not literal-phobia — name what repeats or
what already has a name.

## Verification

`VH` — review-time. The reviewer (and `/inquisition`) greps for a raw literal
that duplicates a sibling constant or another occurrence; a hit is a finding.
A clippy lint for the common cases is a desirable follow-up but not the gate.

## References

- [[ISS-055]] — the config-surface split this standard was forged from.
- CLAUDE.md / AGENTS.md — DRY, "no parallel implementation", "write less code".
- POL-001 — the precedent for a flavoured, enforced project prohibition.
