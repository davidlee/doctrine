# Design SL-024: Harden TOML render: escape user free-text through a shared seam

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

## 1. Design Problem

Every entity scaffold renderer splices user free-text — `title`, and `slug` when
passed via explicit `--slug` — into a **TOML string literal** with a raw
`.replace("{{title}}", title)`. No escaping. A value carrying `"`, `\`, a
newline, or `]` writes a syntactically **broken** `*-NNN.toml`: the `new` verb
succeeds, then every later read (`show`/`list`/`validate`) over that tree fails
to parse. A latent data-integrity defect, silent at write time. Surfaced as
SL-020 audit finding F1.

`memory.rs` already solved this for its own corpus (SL-008 A-1): `toml_string`
escapes through the `toml` serializer; the other five renderers do not. The
design extracts that escaper into a shared leaf and routes every TOML renderer
through it, eliminating the raw splice corpus-wide with no parallel
implementation.

## 2. Current State

Five `render_*_toml` fns interpolate `slug`/`title` raw into a quoted template
token (`title = "{{title}}"`):

- `slice::render_toml`, `adr::render_adr_toml`, `requirement::render_requirement_toml`,
  `spec::render_spec_toml` (both subtypes via `subtype.toml_template()`),
  `backlog::render_backlog_toml` (plain + risk templates).
- `input::resolve_title` only trims; `resolve_slug` only checks non-empty.
  Derived slugs are normalised by `entity::derive_slug` (safe charset), but an
  explicit `--slug` bypasses that — so both `title` and explicit `slug` are
  untrusted at render.

`memory.rs` is the lone correct renderer: `title`/`summary`/`repo`/`ref_name`
and the scope arrays route through private `toml_string` / `toml_array_inner`,
and `memory.toml` carries a **bare** token (`title = {{title}}`) because the
helper supplies its own quotes.

## 3. Forces & Constraints

- **ADR-001 layering** (leaf ← engine ← command): the escaper is a pure
  string→TOML-literal function. It must be a leaf — `entity.rs` (engine) is the
  wrong altitude for a presentation helper.
- **No parallel implementation:** one escaper, corpus-wide. A backlog-local or
  per-module copy forks the contract.
- **Behaviour-preservation gate:** the shared render seam is touched, so the
  existing suites are the proof — five entity suites plus memory's must stay
  green *unchanged*. `memory.rs` output stays byte-identical (it becomes a
  consumer of the byte-for-byte-moved seam).
- **Pure/imperative split:** the escaper takes a `&str`, returns a `String`. No
  clock, rng, git, or disk.
- **rust-embed re-embed footgun** (`mem.pattern.embed.rustembed-recompile-and-symlinks`):
  a lone `install/templates/*.toml` edit is invisible until the embedding crate
  recompiles — `touch src/install.rs && cargo build` after template edits.

## 4. Guiding Principles

Escape at render, do not validate or rewrite input. The slice makes broken-input
produce *valid* TOML; it does not reject titles or normalise slugs (that is a
separate policy — see OQ-1). One escaper, one convention, applied uniformly.

## 5. Proposed Design

### 5.1 System Model

New leaf `src/tomlfmt.rs` (`mod tomlfmt;` in `main.rs`). Imports only the `toml`
crate. Depended on by `adr`, `slice`, `spec`, `requirement`, `backlog`,
`memory` — all command-tier, so no cycle (ADR-001: a leaf may be depended on by
any command module).

```
adr  slice  spec  requirement  backlog  memory      (command tier)
  \    \      |        /          /        /
   \    \     |       /          /        /
    +----+----+--- tomlfmt ---+-+--------+            (leaf: pure escape)
                       |
                     toml (crate)
```

### 5.2 Interfaces & Contracts

Two fns, bodies moved verbatim from `memory.rs:653`/`:661` (D1 — verbatim move is
the byte-identical guarantee):

```rust
/// `s` → a quoted, escaped TOML basic-string literal (supplies its own quotes).
pub(crate) fn toml_string(s: &str) -> String {
    toml::Value::String(s.to_owned()).to_string()
}

/// Inner of a TOML array literal: each element through `toml_string`, comma-joined
/// (caller supplies the surrounding `[ ]`).
pub(crate) fn toml_array_inner(xs: &[String]) -> String {
    xs.iter().map(|s| toml_string(s)).collect::<Vec<_>>().join(", ")
}
```

`toml_array_inner` has no new consumer (only memory's scope arrays use it) but
moves *with* `toml_string` to keep the escaping seam single — splitting it forks
the contract being consolidated (D2).

### 5.3 Data, State & Ownership

The escaper is stateless and pure. Ownership of the *fields it escapes* is
unchanged: only `title` and `slug` (user free-text) route through it. `id` (u32),
`date` (generated ISO), `kind` (enum `as_str`), `ref`/`status` are typed or
generated — safe, untouched. The `render_*_md` body splices stay raw: markdown is
free-form prose, never structurally parsed (storage rule, Non-Goals).

### 5.4 Lifecycle, Operations & Dynamics

Convention (D3): **self-quoting** — template carries the bare token, helper
supplies quotes. Two paired edits per field, **lockstep**:

```
template:  slug = "{{slug}}"    →   slug = {{slug}}
renderer:  .replace("{{slug}}", slug)  →  .replace("{{slug}}", &tomlfmt::toml_string(slug))
```

Seven templates change (`slice`, `adr`, `requirement`, `spec-product`,
`spec-tech`, `backlog`, `backlog-risk`); `memory.toml` is already bare. Five
renderers route `slug`+`title`; `memory.rs` deletes its two private fns and adds
`use crate::tomlfmt::{toml_string, toml_array_inner};`.

**Sharp edge:** removing the renderer route while leaving the template's `"`
yields `""value""`; doing the reverse yields a bare unquoted value. The two edits
are a pair, never split (R1).

### 5.5 Invariants, Assumptions & Edge Cases

- **Identity on safe input:** `toml_string("Fast boot")` → `"Fast boot"`, exactly
  the old splice output. So existing round-trip and `!body.contains("{{title}}")`
  assertions pass untouched — the only behaviour change is broken-input → valid.
- **Round-trip invariant:** for any `title`/`slug`, the rendered `*.toml`
  re-parses via its own reader and the value round-trips verbatim.
- **Edge cases the helper must survive:** for a *quoted string literal* the
  breakers are `"`, `\`, and newline — these escape or break the document. A `]`
  (or `,`) inside a quoted `title`/`slug` is already harmless (`title = "a]b"`
  parses fine); `]`/`,` only break the **array** case, which is why
  `toml_array_inner` carries its own breakout tests. Consequence for TDD (R1): a
  renderer red-test driven by `]` alone is **green already** — the hostile driver
  must contain `"` (and ideally newline + `\`) to actually break the file.

## 6. Open Questions & Unknowns

- **OQ-1 — `--slug` normalisation.** Escaping makes an explicit `--slug` *safe*
  to store, but does not normalise it (a derived slug is lowercased/hyphenated;
  an explicit one is not). Whether explicit slugs should be normalised like
  derived ones is a separate policy question. Deferred to a follow-up (Q3); not
  bundled.
- **OQ-2 — `state.rs:336`.** The runtime phase sheet splices `{{name}}` raw into
  a gitignored TOML sheet. Lower stakes (disposable runtime state). The shared
  seam does not make it free (different template/value shape), so deferred to a
  follow-up rather than folded in.

No blocking unknowns.

## 7. Decisions, Rationale & Alternatives

- **D1 — verbatim move, not rewrite.** The escaper bodies move byte-for-byte from
  `memory.rs`, which keeps memory's output byte-identical by construction (the
  behaviour gate) and avoids re-deriving a solved escaper.
- **D2 — move both fns.** `toml_array_inner` travels with `toml_string` even
  though only memory consumes it: the seam is *TOML-literal escaping*, one leaf,
  one contract. Alternative (leave array-inner in memory) rejected — forks the seam.
- **D3 — self-quoting convention, corpus-wide.** Matches the precedent
  (`memory.toml`), keeps `toml_string`'s signature unchanged (so the move is
  byte-identical), and is DRY. Alternative — templates keep `"{{title}}"` and the
  helper emits an inner-only escaped string — rejected: it changes the helper's
  contract from memory's, breaking the verbatim move, and an inner-only escaper
  must still escape `"` so it buys nothing.
- **D4 — new leaf `src/tomlfmt.rs`, not fold-in.** Single responsibility, honest
  name (escaping ≠ the per-entity `render_*_toml` fns). Alternatives rejected:
  `entity.rs` (engine, wrong altitude); `lexical.rs` (tokenize/rank cohesion,
  unrelated); the scope's proposed `render.rs` (name collides with the
  `render_*` fns that stay in each module).
- **D5 — escape only `title`+`slug`.** The other tokens are typed/generated and
  cannot carry injection. Escaping them would be noise and risks a behaviour diff.

## 8. Risks & Mitigations

- **R1 — `""value""` from a half-applied edit.** Template+renderer edits are a
  lockstep pair; the per-renderer round-trip test (a real value must re-parse and
  round-trip) catches both a stray `"` and a missing quote.
- **R2 — silent template edit (rust-embed).** A lone template edit is invisible
  until the embedding crate recompiles. Mitigation: `touch src/install.rs &&
  cargo build` after template edits; the round-trip test runs against the
  embedded asset, so a stale embed fails the test.
- **R3 — behaviour regression in the shared seam.** Five entity suites + memory's
  are the proof; they must stay green unchanged. The verbatim move + safe-input
  identity make this hold by construction.

## 9. Quality Engineering & Validation

- **Behaviour gate (inertness proof):** all five entity suites + memory's stay
  green *unchanged*.
- **New evidence — adversarial round-trip per renderer (red→green):** a `title`
  (and explicit `slug`) carrying `"` + newline + `\` (the quoted-literal breakers
  — *not* `]` alone; see § 5.5) must render a `*.toml` that re-parses via that
  module's reader and round-trips the value verbatim. The reader is `meta::Meta`
  in every module. **Four** modules already round-trip their `render_*_toml`
  output directly (call the render fn, `toml::from_str` the body): `adr.rs:229`
  (`render_adr_toml_round_trips_to_metadata`), `slice.rs:557`
  (`render_toml_round_trips_to_metadata`), `requirement.rs:272`
  (`render_requirement_toml_round_trips_to_metadata`), `backlog.rs:1015`
  (`rendered_toml_round_trips_into_meta_and_backlog_item`) — these four are
  **extended** with the hostile input. **Spec has no such direct test:** its only
  round-trip (`spec_list_meta_parses_scaffolded_spec_toml`, `spec.rs:1145`) goes
  through `fresh` → disk → `meta::read_meta`, never parsing a `render_spec_toml`
  string — so spec needs a **new** test that calls the private `render_spec_toml`
  **directly** and `toml::from_str`s it (`mod tests` is in-file, so the private fn
  is reachable; the direct call is mandatory — routing a hostile `--slug` through
  the disk path would strike `<id>-<slug>` symlink creation *before* the TOML
  round-trip, a false-red from the wrong stratum). `Meta` carries no
  `deny_unknown_fields`, so the entities with extra fields parse through it
  unchanged. TDD red: the injection title currently yields an unparseable file.
  Per-module (private fns ⇒ not a cross-module table): **four extended, one new
  (spec).**
- **Leaf unit tests:** `tomlfmt.rs` tests `toml_string` and `toml_array_inner`
  directly for `"`/`\`/newline/`]` escaping and array breakout.
- **Gate:** `cargo clippy` zero (bins/lib); `just check` clean; TDD
  red/green/refactor.

## 10. Review Notes

Internal adversarial pass (pre-`/inquisition`):

- **A1 — `]` is not a string-literal breaker (design defect, fixed).** The first
  draft listed `"`/`\`/newline/`]` as quoted-literal breakers and let the red
  driver use `]`. But `title = "a]b"` is valid TOML — `]`/`,` break only the
  *array* case. A `]`-only red would be green already (false red). Corrected
  § 5.5 and § 9: quoted-literal breakers are `"`/`\`/newline; the hostile driver
  must contain `"`; `]`/`,` breakout is tested on `toml_array_inner` only.
- **A2 — reader claim corrected (post-`/inquisition`, SL-024 inquisition.md
  Charge 1).** The first draft swore every module already round-trips its
  `render_*_toml` output via `meta::Meta` and cited `spec.rs:1144` as a witness.
  False: `spec.rs:1144` is a section comment, and spec's only round-trip
  (`spec_list_meta_parses_scaffolded_spec_toml`, `spec.rs:1145`) reads from **disk**
  via `fresh`/`read_meta`, never parsing a render string. Corrected: **four**
  modules (`adr.rs:229`, `slice.rs:557`, `requirement.rs:272`, `backlog.rs:1015`)
  round-trip render output **directly** and are extended; **spec gets a new
  direct-render test** (see § 9). No `deny_unknown_fields` exists in the tree, so
  backlog/spec (extra fields) parse through `Meta`.
- **A3 — verbatim move stays clippy-clean (no change).** `toml_array_inner`'s
  `.map(|s| toml_string(s))` is not a `redundant_closure` hit: the element is
  `&String` and `toml_string` takes `&str`, so the closure is load-bearing (deref
  coercion). It ships through the gate in `memory.rs` today; moving it verbatim
  preserves that.
- **Doctrinal alignment:** ADR-001 (leaf altitude) — satisfied by the new leaf;
  no governance conflict, no `/consult` trigger. Behaviour-preservation gate
  (shared seam) — D1's verbatim move + § 5.5 safe-input identity discharge it by
  construction.
