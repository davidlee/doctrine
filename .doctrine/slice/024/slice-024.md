# Harden TOML render: escape user free-text through a shared seam

## Context

Every entity scaffold renderer splices user-supplied free-text — `title`, and
`slug` when passed via explicit `--slug` — into a **TOML string literal** with a
raw `.replace("{{title}}", title)`, no escaping. `input::resolve_title` only
trims; `resolve_slug` only checks non-empty (derived slugs are normalised by
`entity::derive_slug`, but an explicit `--slug` bypasses that). A value carrying
a `"`, `\`, newline, or `]` writes a **syntactically broken** `*-NNN.toml`: the
`new` verb succeeds, then every later read (`show` / `list` / `validate`) over
that tree fails to parse — a latent data-integrity defect, silent at write time.

`src/memory.rs` already solved this for its own corpus (audit note A-1):
`toml_string(s) = toml::Value::String(s).to_string()` escapes through the `toml`
serializer, and `toml_array_inner` does the same for array elements. `memory.rs`
routes `title`/`summary`/`repo`/`ref_name`/scope-arrays through them. The other
five renderers do not. This slice extracts that seam and routes every TOML
renderer through it.

Surfaced as SL-020 audit finding F1. Durable capture:
`mem.pattern.render.toml-splice-escape-user-values`.

## Scope & Objectives

Extract `toml_string` (and `toml_array_inner`) out of `src/memory.rs` into a
shared render-escape seam, and route every renderer that writes user free-text
into a **TOML literal** through it — eliminating the raw splice corpus-wide.

**Affected surface — the `render_*_toml` functions only:**

- `src/adr.rs` — `render_adr_toml` (`:74` slug, `:75` title)
- `src/spec.rs` — `:249` slug, `:250` title
- `src/slice.rs` — `:71` slug, `:72` title
- `src/requirement.rs` — `:120` slug, `:121` title
- `src/backlog.rs` — `render_backlog_toml` (`:452` slug, `:453` title)
- `src/memory.rs` — the **source** of `toml_string`/`toml_array_inner`; becomes a
  consumer of the shared seam (no behaviour change — it is already correct).

**Done is judged by:**

- One shared escaping helper (single definition); `memory.rs`'s private copies
  removed and re-imported, its existing suite green **unchanged** (behaviour
  gate — `memory.rs` output is byte-identical).
- A title / explicit-slug containing `"`, `\`, newline, or `]` round-trips: each
  affected entity's `new` writes a `*-NNN.toml` that **re-parses** via its own
  reader. One adversarial-input test per renderer (or a shared table test).
- `cargo clippy` zero warnings (bins/lib); `just check` clean. TDD red/green:
  the red test is an injection title that currently produces an unparseable file.

## Non-Goals

- **The markdown body renderers** (`render_*_md`, e.g. `backlog.rs:464`,
  `memory.rs:671`). Raw `.replace` into a markdown body is correct — MD is
  free-form prose, never structurally parsed (the storage rule). Out of scope.
- **`src/state.rs:336`** (`{{name}}` into a runtime phase sheet) — gitignored
  runtime state, not an authored entity; lower stakes. Fold in only if the
  shared seam makes it free; otherwise defer.
- **Input sanitisation / validation policy** — this slice escapes at render, it
  does not reject or rewrite titles. Whether `--slug` should be normalised like
  a derived slug is a separate question (note in Open Questions).
- **New entity behaviour** — no new fields, verbs, or templates.

## Affected surface — see Scope.

## Risks, Assumptions & Open Questions

**Assumptions:**
- `toml::Value::String(s).to_string()` emits a complete, quoted, escaped literal
  (verified in `memory.rs` use) — the template's surrounding `"` must therefore
  be **removed** when switching a field from raw splice to `toml_string` (the
  helper supplies its own quotes). This is the sharp edge: a mechanical
  find-replace that leaves the template `"{{title}}"` quotes in place yields
  `""value""`. Each template edit pairs with its renderer edit.

**Risks:**
- **Behaviour-preservation gate (shared render seam).** Five entity suites +
  memory's must stay green; the only intended output change is for inputs that
  previously produced *broken* TOML. Snapshot/round-trip tests are the proof.
- **Template/renderer coupling.** Moving quotes from template into the helper
  touches both `install/templates/*.toml` and the `render_*_toml` fns in lockstep
  (rust-embed re-embed footgun: a lone template edit is invisible until the
  embedding crate recompiles — `mem.pattern.embed.rustembed-recompile-and-symlinks`).

**Open Questions — resolved in `design.md`:**
- **Q1 — home of the shared seam → RESOLVED (design D4).** New leaf
  `src/tomlfmt.rs` (not `render.rs` — name collides with the `render_*` fns that
  stay per-module; not `entity.rs`/`lexical.rs` — wrong altitude/cohesion).
- **Q2 — template-quote convention → RESOLVED (design D3).** Self-quoting
  corpus-wide (helper emits quotes, template drops them: `title = {{title}}`),
  matching `memory.toml`. Keeps `toml_string`'s signature unchanged for the
  byte-identical move.
- **Q3 — `--slug` normalisation → deferred follow-up** (design OQ-1). Orthogonal
  to escaping; not bundled.

## Verification / Closure Intent

- Adversarial-input round-trip test per affected renderer (red→green).
- `memory.rs` output byte-identical (behaviour gate); all existing suites green
  unchanged.
- `cargo clippy` zero; `just check` clean; TDD red/green/refactor.

## Follow-Ups

- `--slug` explicit-value normalisation (Q3), if accepted.
- `src/state.rs:336` phase-name escaping, if not folded in here.
