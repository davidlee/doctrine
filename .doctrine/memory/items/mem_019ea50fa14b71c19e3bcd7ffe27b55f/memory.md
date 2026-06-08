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

## Fix — SHIPPED (SL-024)

`toml_string`/`toml_array_inner` extracted from `memory.rs` into the shared leaf
`src/tomlfmt.rs` (pure, imports only `toml` — leaf, ADR-001). All six entity
renderers route `title`+`slug` through it; the eight `*.toml` templates carry
**bare self-quoting tokens** (`title = {{title}}`) since the helper supplies its
own quotes. Controlled tokens (`kind`, `date`, ids) stay raw. Done as a dedicated
slice under the behaviour-preservation gate, NOT a backlog-local fork (forking the
escaper violates *no parallel implementation*).

## Refinement: render-escape secures the *document*, not a value's downstream use

Escaping makes a hostile `title`/`slug` non-corrupting to the scaffolded TOML —
but `slug` also becomes the `<id>-<slug>` **symlink filename**. A slug that
re-parses cleanly can still be hostile as a path component. Render-escape is
necessary but **not sufficient** for slug safety; explicit-`--slug` normalisation
is the separate, still-open fix (**IMP-005**, SL-024 design OQ-1). General lesson:
escaping at one boundary (the TOML literal) does not sanitise a value for a
*different* boundary (the filesystem).

Surfaced as SL-020 audit finding F1; closed by SL-024. See
[[mem.pattern.lint.string-build-no-push-format]] and
[[mem.pattern.install.authored-entity-wiring]].
