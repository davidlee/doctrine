# Entity renderers splice user free-text into TOML literals unescaped (toml_string is the fix)

Every entity scaffold renderer splices a user-supplied free-text value — most
notably `title` — into a TOML string literal with a raw
`.replace("{{title}}", title)`, no escaping. `input::resolve_title` only trims.

Affected: `adr.rs`, `spec.rs`, `slice.rs`, `requirement.rs`, `backlog.rs`. A
title containing `"`, `\`, or a newline writes a **syntactically broken**
`*-NNN.toml`: the `new` verb succeeds, then every later read (`show` / `list` /
`validate`) over that tree fails to parse.

`src/memory.rs::toml_string` (line ~653) is the **existing in-tree fix** — memory
audit note A-1: "user-influenced value escaped through the serializer, never
spliced raw". `memory.rs` routes `title`/`summary`/`repo`/`ref_name` through it.

## Fix (corpus-wide, its own slice)

- Extract `toml_string` (and `toml_array_inner`) from `memory.rs` into a shared
  render-escape seam.
- Route every entity renderer's user free-text through it; controlled tokens
  (`kind`, `date`, ids) stay raw.
- Touches the shared render seam across 5 modules under the
  behaviour-preservation gate → a dedicated slice, NOT a backlog-local fork
  (forking the escaper violates *no parallel implementation*).

Surfaced as SL-020 audit finding F1. See [[mem.pattern.lint.string-build-no-push-format]]
and [[mem.pattern.install.authored-entity-wiring]].
