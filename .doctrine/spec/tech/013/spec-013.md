# SPEC-013: CLI surface

<!-- Reference forms: entity ids padded (SPEC-007, ADR-004); doc-local refs bare
     (D1 decision, OQ-1 open question). See .doctrine/glossary.md § reference forms. -->

## Overview

The CLI surface is the one uniform contract every entity kind presents at the
command line. It sits beneath the whole-system root (SPEC-003) and rides the
shared entity engine (SPEC-004) for materialisation, identity, and the atomic
claim — none of which it restates. This container owns only what is specific to
*the surface*: the `<kind> <verb>` command grammar, the kind-blind read spine
that every `list` shares (`src/listing.rs`), the `CommonListArgs` flatten that
makes that spine mandatory, the canonical id form, the `--columns` projection
model, and the conformance matrix plus black-box goldens that pin the surface
byte-exact. The per-kind `show` *content* — which facets render and how a kind
reassembles its TOML and prose — belongs to each kind's own component; this
container owns the uniform *shape* those verbs take and the shared read
machinery they delegate into.

## Responsibilities

Mirrors the structured `responsibilities` list: impose the uniform
`<kind> <verb>` grammar; own the kind-blind read spine; carry the shared
`CommonListArgs` flatten; own the `--columns` projection model; fix the
canonical id form; and pin the surface with the conformance matrix and
black-box goldens.

### Uniform command grammar

The surface is a two-level clap subcommand tree. A top-level `Command` enum
names each entity kind (`Slice`, `Memory`, `Adr`, `Policy`, `Standard`, `Spec`,
`Backlog`, …), each delegating to a per-kind subcommand enum (`AdrCommand`,
`PolicyCommand`, …). The verbs within are the shared set — `new`, `list`,
`show`, `paths`, and (for lifecycle kinds) `status` — so the invocation shape is
identical across kinds: `doctrine <kind> <verb>`. A `show` reassembles one
entity's metadata, relationships, and prose body; a `list` enumerates the
corpus. The grammar is the predictability contract; the engine seam each verb
crosses to materialise or read an entity is the parent container's, used here
unchanged.

### The kind-blind read spine

Every `list` shares one pure leaf, `src/listing.rs`. It is a **pure leaf**
(ADR-001: leaf ← engine ← command) importing neither clap nor `entity`: it
reads no clock, rng, git, or disk. It carries the *invariant* axes of a list
surface — the filter semantics (`retain` over `FilterFields`), the status
known-set check (`validate_statuses`), the canonical id form (`canonical_id`),
the generic table layout (`render_table`/`render_columns`), and the JSON
envelope (`json_envelope`). What stays *per-kind* is the variant axis: the row
type, the column projection, the ordering, and any kind-specific flags — none
of which live in the leaf. This is the shared substrate every per-kind `list`
delegates into.

### The CommonListArgs spine flatten

`CommonListArgs` is the clap-facing arg bundle every kind's `list` flattens —
`--filter`/`-f`, `--regexp`/`-r`, `--case-insensitive`/`-i`, `--status`/`-s`,
`--tag`/`-t`, `--all`/`-a`, `--format`, `--json`, and `--columns`. Flattening
it is the mechanism that makes the shared semantics *mandatory* rather than
merely available: a kind cannot quietly drop a flag or shadow it with a bespoke
one. `into_list_args` lowers the parsed clap bundle onto the clap-free
`ListArgs` leaf input — the single seam where command-layer clap types stop and
the pure spine begins.

### The column projection model

`--columns id,status,slug` selects and orders the visible table columns. A
`Column<R>` is a pre-materialised pair: a static header name and a pure,
*non-capturing* `fn` cell extractor (so columns are cheap, `Copy`-friendly
descriptors borrowed as `&[Column<R>]`, never moved). `select_columns`
validates each requested name against the kind's available set — an unknown
name errors with the available set, never silently ignored — and honours the
requested order. `render_columns` projects the chosen columns into a grid over
`render_table`. The projection has no effect under `--json` (JSON rows are
faithful and full) and is rejected on `memory list`, which is not yet on the
column model.

### The canonical id form

`canonical_id(prefix, id)` is the single id-form authority for prefixed kinds:
a kind prefix joined to a zero-padded-to-three-digit number (`SL` + `25` →
`SL-025`), matching the citation convention. Memory is conformant-by-exception
— its uid *is* its canonical id, so it does not route through this helper. The
form is consumed by the regex filter axis (regex matches over canonical-id +
slug + title) and by every id column.

### Conformance and goldens

The surface is pinned two ways because clap exposes no structural "is this
flattened?" check. Conformance is proven *behaviourally*: the parse-conformance
matrix asserts that, for each of the seven spine kinds (`adr`, `policy`,
`standard`, `slice`, `spec`, `backlog`, `memory`), every shared spine flag — in
both short and long forms — parses and the command succeeds; a dropped flatten
or a shadowed flag trips it. The rendered output is pinned by per-verb
black-box goldens over the built binary plus a cross-verb column-model golden
net, so any drift in the table layout or JSON envelope turns a golden red.
`skills list` is deliberately excluded from the spine — it does not flatten
`CommonListArgs`.

## Concerns

- **No structural flatten check.** clap cannot assert at compile time that a
  kind's `list` actually flattens `CommonListArgs`; the guarantee is upheld only
  by the behavioural conformance matrix, which must enumerate every spine kind
  and every shared flag or a regression ships silently.
- **Memory's conformant-by-exception status.** Memory rides the grammar and the
  filter spine but sits outside the canonical id form and the `--columns` model;
  the exception must be explicit (rejected, not silently ignored) so the
  surface stays predictable rather than quietly inconsistent.
- **Variant/invariant boundary discipline.** The row type, ordering, and
  per-kind flags must stay command-side; leaking any into the pure leaf would
  re-couple it to a kind and erode the kind-blindness the spine exists to
  guarantee.

## Hypotheses

- **One flattened arg bundle beats per-kind list args.** Making the shared
  semantics mandatory by flattening `CommonListArgs` — rather than asking each
  kind to re-declare the flags — is preferred so the surface cannot drift kind
  by kind, and the conformance matrix has one spine to assert against.
- **A pure clap-free leaf is the right cut.** Keeping the filter/format/layout
  logic in a leaf that imports neither clap nor `entity` is preferred so the
  read spine is unit-testable without a built binary and obeys the ADR-001
  layering without a clap dependency bleeding inward.
- **Behavioural conformance is the only honest proof.** Since clap offers no
  structural flatten assertion, proving the contract by parsing every flag on
  every kind over the built binary is preferred to any reflective check that
  clap does not support.

## Decisions

- **D1 — the surface is `<kind> <verb>`, uniform across kinds.** A top-level
  kind enum delegating to per-kind verb enums with a shared verb set is the
  fixed grammar; predictability is the contract, not each kind's internals.
- **D2 — the list spine is a pure clap-free leaf.** `src/listing.rs` owns the
  invariant list axes (filter, status known-set, id form, table layout, JSON
  envelope) and imports no clap and no `entity`; the variant axes stay
  command-side.
- **D3 — `CommonListArgs` is flattened, making the spine mandatory.** A kind's
  `list` cannot opt out of or shadow a shared flag; `into_list_args` is the one
  seam lowering clap onto the pure leaf.
- **D4 — columns are pre-materialised descriptors with non-capturing
  extractors.** `Column<R>` pairs a static header with a pure `fn` cell
  extractor so columns are borrowed, never moved; `select_columns` validates
  and orders, erroring on unknown names rather than ignoring them.
- **D5 — the surface is pinned behaviourally and byte-exactly.** A
  parse-conformance matrix proves every spine kind flattens every shared flag,
  and black-box goldens over the built binary pin the rendered table and JSON.
