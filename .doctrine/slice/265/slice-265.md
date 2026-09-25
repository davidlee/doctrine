# Unified doctrine show: one verb for any canonical ref

## Context

Reading one entity costs a kind lookup the caller already has. `doctrine
knowledge show DEC-031` is the current spelling; so are `rfc show`, `adr show`,
`backlog show`, … — **twelve** top-level commands whose `show` verb differs only
in which module owns the renderer (`slice`, `spec`, `adr`, `policy`, `standard`,
`rfc`, `backlog`, `knowledge`, `review`, `rec`, `revision`, `concept-map`). The id
already carries the kind: `DEC-031` names one kind unambiguously, because
`SPEC-013` fixes the prefix as the canonical identity (`REQ-201`).

The friction is on record. `RFC-011`'s case-notes (`a2077b93d`, from the
token-efficiency benchmark) record the failure mode directly:

> `doctrine backlog show RFC-016` rejected (`unknown backlog prefix RFC`) — RFC is
> its own kind (`doctrine rfc show`), not a backlog prefix, but the search listing
> renders RFC-016 in the same column as IMP/ISS rows, inviting the wrong verb. One
> failed call. Cross-kind `search` results don't hint which `show` verb each id
> needs.

The proposal is the obvious one: `doctrine show <REF>` resolves a canonical ref to
its kind and calls that kind's own `show`. `doctrine show RFC-031` ≡ `doctrine rfc
show RFC-031`.

## Scope & Objectives

Add a top-level, kind-blind `show` verb that resolves a ref to its kind and
delegates to that kind's existing renderer.

1. **Route, don't re-render.** `doctrine show <REF>` produces output
   **byte-identical** to `doctrine <kind> show <REF>`. No new renderer, no
   normalised cross-kind envelope, no per-kind content change.
2. **Resolution rides the existing authority.** Kind resolution uses
   `kinds::parse_resolvable_ref` — the same function `link`, `needs`, `after`,
   `tag`, `supersede` and `facet` already use. Unknown prefix, dangling ref, and
   ambiguous bare id keep that function's error contract.
3. **The clap surface rides the existing seam.** The new command flattens
   `CommonShowArgs` (`src/main.rs:217`) — the struct whose own doc already says
   *"the prefix selects the kind"*.
4. **Governance moves with it.** `SPEC-013` ("CLI surface") owns the command
   grammar: `REQ-197` pins uniform `<kind> <verb>`, and `REQ-198` pins a
   kind-blind **list** spine. A kind-blind **show** router is a new grammar
   element: it takes a REQ sibling to `REQ-198`, the `SPEC-013` prose that names
   it, and the governance dependency carried as a `REV` per `ADR-013`.
5. **Reachable and visible.** A `FAMILIES` `explore` entry — the family that
   already holds the two other kind-blind top-level read verbs, `search` and
   `inspect` — so `--help` and `--boot-map` name it.

## Non-Goals

- **No flag unification.** The `show` flag triple (`--format`/`--json`/`--path`)
  is hand-declared in seven kinds; only `backlog`/`knowledge` flatten
  `CommonShowArgs`. Rolling every kind onto the flatten is a separate change (see
  Follow-Ups).
- **No MCP tool.** This is a CLI read verb; the MCP surface is a hand-authored
  tool list and is not widened here.
- **No output normalisation.** Delegation preserves each kind's table and JSON
  envelope exactly. A "uniform" envelope is a different, much larger change and
  drifts from the kind-owned renderers.
- **No change to any existing `<kind> show`.** Every current invocation keeps its
  byte-for-byte behaviour.
- **No `search` output change.** The case-note's complaint admits a second,
  complementary fix (a verb-hint column on `search`); that is not this slice.
- **No re-addressing of non-canonical identity.** Memory (`mem_…` /
  `[[mem.key]]`), observations and design-run refs are separate address spaces
  with their own verbs; whether the router accepts them is `OQ-2`, and the
  default is numbered canonical refs only.

## Affected surface

Coarse fence; the exact touch-set is `/design`'s, and after that `/plan`'s.

- `src/commands/cli.rs` — the `Command::Show` variant, its dispatch arm, the
  `FAMILIES` entry.
- `src/commands/` — the router (provisional file `src/commands/show.rs`, command
  tier).
- `src/commands/guard.rs` — one `Read` arm (compile-enforced by the exhaustive
  match).
- `tests/` — the equivalence property, the prefix-totality guard, and the
  `families_partition_the_visible_command_tree` census count.
- `.doctrine/spec/tech/013/**` and a new `REQ` — the grammar amendment.
- **Not** any kind module's `show` implementation.

## What already exists (ride these seams, do not rebuild)

- **The uniform signature.** Every numbered kind's show already takes
  `(path: Option<PathBuf>, reference: &str, format: Format)`: `slice`; `spec`
  (`PRD`/`SPEC`); `rec` (`REC`); `revision` (`REV`); `knowledge` (the seven
  record prefixes — it **already self-resolves by prefix**); `backlog` (the five
  item prefixes — likewise); and `adr`/`policy`/`standard`/`rfc`, each a
  one-line call into `governance::run_show(&GovKind, …)`. `spec::run_req_show`
  covers `REQ`. Three arms are not uniform: `review::run_show` returns
  `ReviewOutput` (the kind's own handler prints it via `print_review`),
  `concept_map::run_show` takes two extra booleans, and `memory::run_show` takes
  a writer.
- **The cross-kind dispatch precedent.** `catalog::scan::outbound_for` is
  already "one data-driven match over all `KINDS` rows", dispatching on
  `kind.prefix` to the owning module — including the three shapes the router
  needs (a top-level kind command; the `PRD`/`SPEC` subtype pair; the record and
  backlog tails via `from_prefix`). It also carries the
  `debug_assert!(false, "unrouted KINDS prefix")` fallthrough that `OQ-4` should
  copy. The router is that shape, at command tier.
- **The resolution authority.** `kinds::parse_resolvable_ref` (canonical or bare,
  with dangling / unknown-prefix / ambiguous-bare errors) over the 24-row `KINDS`
  table, whose discriminant is `prefix`.
- **Canonicalisation before delegation is not optional.** Every per-kind `show`
  parses its *own* prefix and rejects a foreign one — `listing::parse_ref` strips
  `PREFIX-` and digit-parses the rest, and `spec::resolve_spec_ref` requires a
  hyphen outright — so a bare id routed on its own fails inside the callee. The
  contract is `parse_resolvable_ref` → `kinds::canonical_id(prefix, id)` →
  `<kind>::run_show(canonical)`. `REQ` additionally reaches
  `requirement::load`, which wants the canonical `REQ-NNN` FK.
- **The clap seam.** `CommonShowArgs` — `id`, `format`, `json`, `path`.
- **The family home.** `search` and `inspect` are the existing kind-blind
  top-level read verbs; the router joins them in `explore`.

## Risks & assumptions

- **R1 — ADR-001 layering / command tangle.** The router reaches ~12 command-tier
  modules. Homed in a **new top-level** module it would add new top-level edges
  and grow the command tangle, which is ratcheted at
  `[tangle_baseline] command = 76` in `.doctrine/adr/001/layering.toml` ("may not
  grow"). Homed **under `src/commands/`** it adds none: the gate records edges by
  top-level module, and `commands::cli` already imports every one of those kinds.
  The design must state the home; this is a constraint, not a preference.
- **R2 — the census assertion is intentional churn.** The
  `families_partition_the_visible_command_tree` test asserts the visible command
  count (56 today) and that every visible command sits in exactly one family.
  Both assertions must be updated deliberately, not relaxed.
- **R3 — `REQ`.** It is the one numbered prefix whose `show` is *not*
  `spec::run_show` (`resolve_spec_ref` accepts only `PRD`/`SPEC`); it needs its
  own arm onto `spec::run_req_show`.
- **R4 — goldens.** The boot-map and help goldens pin family structure and member
  *presence*, not family lines, so no byte churn is expected there — but the boot
  snapshot is regenerated (`doctrine boot`), and any golden that does pin the
  member set must be updated, not worked around.
- **A1 — assumed:** every numbered prefix in `KINDS` has a working `show` verb
  today. If one does not, `OQ-4`'s totality guard is what surfaces it.

## Open questions

For `/design` to close.

- **OQ-1 — fidelity contract.** Byte-identical delegation (cheap, testable, and
  the scope default) versus a normalised cross-kind envelope?
- **OQ-2 — address scope.** Numbered canonical prefixes only, or also `mem_…`,
  observation uids and design refs?
- **OQ-3 — bare ids.** Accept `doctrine show 31` through `parse_resolvable_ref`
  (ambiguous → that function's existing error naming the candidates), or require
  the canonical form?
- **OQ-4 — totality.** How is "every `KINDS` row is routed" pinned so a kind
  added later cannot land silently unrouted? The kind-dispatched rows elsewhere
  carry a `debug_assert!(false)` fallthrough; a router wants a test.

## Verification / closure intent

- **VT** — for one fixture entity of every numbered prefix, `doctrine show <REF>`
  emits stdout byte-identical to `doctrine <kind> show <REF>`, in both formats.
- **VT** — an unknown prefix, a dangling ref, and an ambiguous bare id each fail
  with the resolution authority's error.
- **VT** — every `KINDS` row resolves to a routed arm (totality).
- **VA** — no kind module's `show` was modified: the equivalence byte-goldens pass
  with the kind implementations untouched.
- **VA** — `doctrine show` appears in `explore` in both `--help` and
  `--boot-map`; the family census assertion is updated, not deleted.
- **VA** — the `SPEC-013` amendment lands with its new `REQ` and its coverage
  record; the `REV` carries the governance change.

## Summary

## Follow-Ups

- **`CommonShowArgs` unification** — collapse the hand-declared
  `--format`/`--json`/`--path` triples onto one flatten, the same DRY move
  `CommonListArgs` already made for `list` (`REQ-199`). A companion slice or a
  backlog item; deliberately not absorbed here.
- **`search` verb hint** — a column naming the `show` verb an id needs, so the
  RFC-011 case-note's failed call is prevented at its source as well as
  dissolved.
- **MCP parity** — if a `doctrine_show` tool earns its place, it is a separate
  decision needing `IDE-016`'s evidence.
