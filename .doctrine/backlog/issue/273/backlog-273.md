# ISS-273: Tech spec scaffold teaches malformed source identifier

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`install/templates/spec-tech.toml:25` — the template every `doctrine spec new
tech` scaffolds from — documents a `[[source]]` anchor as:

```toml
#   [[source]]
#   language = "rust"
#   identifier = "doctrine/cli"
#   module = "doctrine::cli"   # optional
```

`identifier` is module-path shaped there. The convention is that `identifier`
is a **repo-relative path** and `module` is the Rust path — as every conformant
spec in the corpus has it, e.g. SPEC-003:

```toml
identifier = "src/search.rs"
module = "doctrine::search"
```

So the scaffold teaches the malformed form to every new tech spec.

## Why it matters

Nothing validates a `[[source]]` anchor — `spec validate` FK-checks members and
interactions only — so a wrong identifier is silent. SPEC-020 was the corpus's
sole deviation and it **misreported as covered** until repaired in `92d46941`
(CHR-046). The template reproduces the defect that chore existed to clean up.

Surfaced authoring SPEC-029 under SL-233 PHASE-01 (phase sheet finding F-2).

## Scope

- `install/templates/spec-tech.toml:25` — the shipped template. This is the fix.
- `src/spec.rs:3153` and `:3655` — unit-test fixtures using the same malformed
  form. Not user-visible, but they entrench the wrong shape as the in-repo
  example; worth correcting in the same pass.

Consider whether the anchor should be validated rather than only documented —
an identifier that resolves to no path in the tree is a cheap check, and it is
the check that would have caught SPEC-020 without a manual sweep. That may be
its own item; the template repair should not wait on it.
