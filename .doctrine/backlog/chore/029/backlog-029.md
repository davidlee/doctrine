# CHR-029: Audit codebase for magic strings per STD-001

## Context

[[STD-001]] mandates that any literal value with meaning recurring across the
codebase — paths, keys, ref prefixes, env-var names, format tokens, magic
numbers, sentinel strings — must be named once and referenced by that name
everywhere. The standard was forged from [[ISS-055]], where `"doctrine.toml"`
was hand-typed at ~20 sites while a `DOCTRINE_TOML` const sat unused beside
them.

This chore is a systematic audit of the entire `src/` tree to surface every
magic string that violates STD-001.

## Scope

- All source under `src/` (and tests where relevant).
- String and numeric literals that carry meaning and recur, or that already
  have a named constant defined elsewhere but are duplicated as raw literals.
- Excluded: genuinely one-shot, self-evident literals with a single call site
  and no sibling constant.

## Approach

1. Identify existing named-constant surfaces (const / static items, enums,
   `kinds::` prefix constants, etc.).
2. grep/ripgrep for raw literals that match existing constants.
3. grep for any literal that appears at >1 site without being named.
4. For each finding, either:
   - replace with the existing constant (if one exists), OR
   - extract a new named constant and replace all call sites.
5. Ensure tests pass and lint is clean after each change.

## References

- [[STD-001]] — the governing standard.
- [[ISS-055]] — the motivating defect.
- [[IMP-184]] — prior art: DRY record-kind membership (~17 sites hardcode
  prefix literals instead of reading `kinds::RECORD`).
