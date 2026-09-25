<!-- doctrine:section sec-1 -->
## What changes and why

Reading one entity costs a kind lookup the caller already has. Twelve
entity-kind commands own a `show` — `slice`, `spec`, `adr`, `policy`,
`standard`, `rfc`, `backlog`, `knowledge`, `review`, `rec`, `revision`,
`concept-map` — and they differ only in which module owns the renderer. (Three
non-entity commands — `memory`, `coverage`, `design` — also expose a `show`, and
are outside the numbered address space this slice routes: `DEC-296`.) The
canonical id already carries the kind: `SPEC-013` fixes the prefix as identity
for numbered kinds (`REQ-201`), so `DEC-031` names exactly one kind and
`doctrine knowledge show DEC-031` spells a fact the ref already stated.

**Why it is worth doing, first for a human.** The reader already holds the
answer: `SPEC-013` fixes the prefix as identity for numbered kinds (`REQ-201`)
and `STD-002` makes the id authoritative, so `DEC-031` names exactly one kind.
Being made to restate it — `doctrine knowledge show DEC-031` — is typing the
caller has already done, and the long kind names are where it costs most
(`knowledge`, `concept-map`, `revision`). One predictable read verb removes the
lookup without asking anything of the id that the id does not already carry.

**Second, for an agent.** The same redundancy is where an agent fails. RFC-011's
case-notes (`a2077b93d`) record a failed call: `doctrine backlog show RFC-016`
was rejected (`unknown backlog prefix RFC`) because the cross-kind `search`
listing renders `RFC-016` in the same column as `IMP`/`ISS` rows without hinting
which `show` verb each id needs. The router dissolves that at its source rather
than patching the `search` output.

This slice adds one top-level, kind-blind verb:

```text
doctrine show RFC-031   ≡   doctrine rfc show RFC-031
```

The router resolves the ref to its kind and delegates to that kind's existing
`show`. Nothing about how a kind renders changes; the new verb is a surface over
the twelve that already exist.

**Boundary.** The change is confined to the command layer — a `Command::Show`
variant, a router under `src/commands/`, one exhaustive-match arm in `guard.rs`,
one `FAMILIES` entry, a `SPEC-013` amendment, and the two agent-facing surfaces an
agent reads *at the moment it chooses a verb*: `install/using-doctrine.md`'s verb
table and the `install/routing-process.md` guardrail, which is inlined into every
boot snapshot (`sec-4` regenerates it). The remaining guidance — the canon and
walkthrough skills, `authority-model.md`, and the shipped memories — is out of
scope and filed as `CHR-078`: those edits take a different mechanism (a shipped
memory needs a rebuild, `memory sync` and `install`), and the leading intent here
is the human read path. No kind's `show` implementation is touched, no `--format`
semantics change, no MCP tool is added (`RV-384` `F-18`).

The cost is a resolution and a call: no new renderer, no normalised envelope, no
per-kind content change.

The five decisions this rests on are `DEC-295`–`DEC-299` (from `inq-1`–`inq-5`).

<!-- doctrine:section sec-2 -->
## The router: resolve, canonicalise, delegate

### The contract

```text
kinds::parse_resolvable_ref(root, ref) -> (&'static KindRef, u32)
kinds::canonical_id(prefix, id)       -> String
<kind>::run_show(path, &canonical, format) -> Result<()>
```

Canonicalisation before delegation is required, though not for the reason a
blanket claim would give. Four of the twelve kinds' parsers reject a bare id
outright — `backlog`, `knowledge`, `spec` and `review`. The rest accept one
through `listing::parse_ref`'s fallback to the raw string, which then digit-parses
it (`slice adr policy rfc revision rec concept-map`). The router canonicalises for
every kind regardless, which is what makes `show 31` and `show SL-031` converge on
one call and keeps the four strict callees working.

`kinds::parse_resolvable_ref` (`src/kinds/resolve.rs`) is the resolution
authority: it accepts both `PREFIX-NNN` and bare `NNN`, verifies the entity
directory exists (a dangling ref is refused), and for a bare id scans every
`KINDS` row, refusing an ambiguous match by naming the candidate refs. The router
inherits that error contract unchanged (`DEC-297`).

The router accepts the 24 numbered prefixes in `kinds::KINDS` and nothing else
(`DEC-296`). `mem_…`/`mem.<key>`, SPEC-028 observation uids and design-run refs
are separate address spaces with their own verbs; including them would be a
second route with a different argument shape, not a table row.

**Case.** The router accepts the canonical `PREFIX-NNN` form, the bare `NNN`
form, and any casing of the prefix: it ASCII-uppercases the prefix before
resolution, so `dec-031`, `Dec-031` and `DEC-031` all resolve. It does **not**
try to mirror the per-kind parsers, because they do not share one rule —
`listing::parse_ref` strips exactly `PREFIX-` or its all-lowercase spelling (so
`dec-031` resolves and `Dec-031` does not), `knowledge::resolve_ref` and
`backlog::parse_ref` uppercase the prefix and so ignore case entirely, and
`spec::resolve_spec_ref` never uppercases. Mirroring any one of them would refuse
a ref some other kind's own `show` accepts, and would make the design's central
equivalence false for that spelling (`RV-384` `F-20`, `F-25`). The equivalence is
therefore stated against the parsers, not against a case rule: **for every ref a
kind's own `show` accepts, `doctrine show` emits the same bytes.** Unconditional
uppercasing is a superset — it also accepts a few refs a case-strict kind
(`spec`) refuses.

```mermaid
flowchart TD
  A["doctrine show &lt;REF&gt;"] --> B["kinds::parse_resolvable_ref"]
  B -->|"canonical or bare, entity exists"| C["kinds::canonical_id(prefix, id)"]
  B -->|"unknown prefix · dangling · ambiguous bare"| X["refusal — the resolver's own error"]
  C --> D{"route(prefix)"}
  D --> E["&lt;kind&gt;::run_show(path, canonical, format)"]
  E --> F["stdout — byte-identical to &lt;kind&gt; show &lt;REF&gt;"]
```

*Purpose: the router resolves and delegates; it never renders.*

### Delegation shapes

The dispatch mirrors `catalog::scan::outbound_for`'s groupings, and covers all
24 `KINDS` rows.

| group | prefixes | call |
|---|---|---|
| singleton kinds | `SL` `RV` `REC` `REV` `CM` | `slice::run_show`; `review::run_show` then `print_review`; `rec::run_show`; `revision::run_show`; `concept_map::run_show(path, ref, format, false, false)` |
| governance spine | `ADR` `POL` `STD` **`RFC`** | `adr::run_show` / `policy::run_show` / `standard::run_show` / `rfc::run_show` — each a one-line wrapper over `governance::run_show(&Kind, …)` |
| spec pair | `PRD` `SPEC` | `spec::run_show(path, ref, format)` — the subtype is resolved inside by `resolve_spec_ref`, so the router passes none |
| requirement | `REQ` | `spec::run_req_show(path, ref, format)` |
| knowledge records | `ASM` `DEC` `QUE` `CON` `EVD` `HYP` `CPT` | `knowledge::run_show` via `knowledge::RecordKind::from_prefix` |
| backlog items | `ISS` `IMP` `CHR` `RSK` `IDE` | `backlog::run_show` via `backlog::kind_from_prefix` |

The router calls the **per-kind wrappers** (`adr::run_show`, `policy::run_show`,
`standard::run_show`, `rfc::run_show`) rather than `governance::run_show`
directly. `src/commands/` already imports those four modules (through `cli.rs`),
so the new file adds no top-level layering edge; reaching into `governance`
would add one (`RV-384` `F-8`).

Two deviations from the uniform `(path, ref, format)` shape are handled
explicitly rather than absorbed:

- `review::run_show` returns a `ReviewOutput` value; the kind's own dispatch
  prints it via `print_review`. The router calls the same two functions in the
  same order, so the bytes match.
- `concept_map::run_show` takes `edges: bool, nodes: bool` beyond the shared
  shape. The router passes both `false`, which is exactly the bare
  `doctrine concept-map show <REF>` it mirrors.

### The format shorthand

`CommonShowArgs` (`src/main.rs`) carries `--format` and the `--json` shorthand.
The shorthand is resolved inline at four call sites — `backlog.rs:290` (Show) and
`:298` (Inspect), `knowledge.rs:3879` (Show) and `:3887` (Inspect) — each
`if json { Format::Json } else { format }`. The router needs the same
resolution, and a fifth copy is precisely the drift this slice should not add.
One accessor — `CommonShowArgs::format()` — becomes the single home for the
shorthand; the router and all four existing sites use it. The kind `run_show`
implementations and their goldens are untouched.

<!-- doctrine:section sec-3 -->
## Totality: the route lookup and its guard

A `match prefix` over `&str` cannot be compiler-exhaustive. A wildcard is
unavoidable, and a wildcard is where a kind added later lands silently. The
design splits the guarantee into three parts, each enforced where it can be:

1. **Resolution is a pure, testable lookup.** `route(prefix) -> Option<Route>`
   names every `KINDS` row explicitly, matching on the named prefix constants
   `kinds` already exports (`kinds::SL`, `kinds::ADR`, …) rather than on string
   literals — `STD-001`, and `outbound_for` matches literals, which is the
   precedent *not* to copy (`RV-384` `F-24`). It falls through to `None`
   otherwise, and it carries **no assertion**. `outbound_for` can hold a
   `debug_assert!(false)` fallthrough because nothing probes it with an unrouted
   prefix; `route()` is probed exactly that way by the negative control below, and
   `cargo test` builds with `debug_assertions` on, so an assertion here would
   panic rather than return the `None` the test observes. The loud failure lives
   at the **dispatch site**, where `None` becomes the unrouted-prefix refusal
   (`RV-384` `F-16`).
2. **Handling is compiler-exhaustive.** `Route` is a closed enum —
   `Slice`, `Adr`, `Policy`, `Standard`, `Rfc`, `Spec`, `Req`, `Knowledge`,
   `Backlog`, `Review`, `Rec`, `Revision`, `ConceptMap` (13 variants over the 24
   prefixes) — and dispatch is a `match route` with no wildcard, so a new variant
   cannot be left unhandled. The knowledge and backlog variants carry **no
   payload**: `knowledge::run_show` and `backlog::run_show` take
   `(path, reference, format)` and re-resolve their own kind, so a carried
   `RecordKind`/`ItemKind` would be a never-read field — the defect `F-9` fixed
   for `Route::Spec`, and a `dead_code` warning under the zero-warning gate
   (`RV-384` `F-19`). `route()` classifies those rows through
   `RecordKind::from_prefix` and `backlog::kind_from_prefix` and then discards the
   value, so the classification stays single-sourced rather than becoming a
   prefix list.
3. **Coverage is pinned by a unit test over the resolver's own table.**
   `every_kinds_row_routes` is a `#[cfg(test)]` unit test in
   `src/commands/show.rs` (it reaches `pub(crate)` `route()` and `KINDS`, which
   an integration test cannot — `RV-384` `F-5`). It iterates the **`KINDS`**
   table's prefixes — the table `parse_resolvable_ref` and `kind_by_prefix`
   resolve against — asserts every prefix yields `Some(_)`, and carries a
   negative control asserting a synthetic unrouted prefix returns `None`.

The iterated set is `KINDS`, not the sibling `ALL_KINDS` prefix literal
(`RV-384` `F-1`). A prefix reaches `route()` only through `kind_by_prefix`, which
scans `KINDS`; `ALL_KINDS` is a separate `&[&str]` with no structural tie to it,
so a test over `ALL_KINDS` would not notice a `KINDS` row that
`parse_resolvable_ref` resolves and `route()` does not.

```mermaid
flowchart LR
  P["prefix from parse_resolvable_ref"] --> R{"route(prefix)"}
  R -->|"Some(Route)"| M["match Route — exhaustive, no wildcard"]
  R -->|"None"| DA["dispatch site: unrouted-prefix refusal"]
  T["every_kinds_row_routes over KINDS (unit test)"] -.->|"pins"| R
```

*Purpose: resolution is observable and handling is compile-checked, so neither
half can drift alone.*

<!-- doctrine:section sec-4 -->
## The surface: clap, families, guard

- **`Command::Show`** flattens `CommonShowArgs` (`id`, `format`, `json`, `path`)
  — the struct whose own doc reads *"the prefix selects the kind"*. A leaf
  variant modelled on `Command::Search(SearchArgs)`.
- **`FAMILIES`** gains `show` in `explore`, the family that already holds the two
  other kind-blind top-level reads, `search` and `inspect`. `--help` and
  `--boot-map` render from `FAMILIES`, so the verb is discoverable without a
  second enumeration.
- **The census assertion** `families_partition_the_visible_command_tree`
  (`src/commands/cli.rs`) pins the visible top-level command count and that every
  visible command sits in exactly one family. Both are updated deliberately —
  56 → 57 — not relaxed (`R2`).
- **`guard.rs`** gains `Command::Show { .. }` in the combined leaf `Read` arm.
  That match is exhaustive with no wildcard, so the verb is a compile error until
  classified (`REQ-192 §3`); `show` is a read.
- **The `--json` shorthand** resolves through the new `CommonShowArgs::format()`
  accessor; the router does not restate the `json`-over-`format` rule.

The boot-map/help goldens pin member presence and the leaf-no-subline rule, not a
full member list; `show` is a leaf in an existing family and needs no golden
rewrite. The boot snapshot is regenerated (`doctrine boot`).

<!-- doctrine:section sec-5 -->
## The governance change is its own phase

`SPEC-013` owns the two-level `<kind> <verb>` grammar (`REQ-197`) and the
kind-blind **list** spine (`REQ-198`). `REQ-197` mandates that every numbered
entity kind present through the per-kind grammar, while *allowing* non-entity
capability containers to reuse it; it says nothing about a kind-blind top-level
verb. `REQ-198` is the list spine. A kind-blind **show** is therefore neither,
and takes a new requirement sibling to `REQ-198`. The governance dependency
routes through a Revision (`ADR-013`), and the precedent is `REV-038`, which
amended `SPEC-013`'s own member requirements.

This is a phase of its own, not a footnote to the code phase (`DEC-299`). Its
acts are discrete and separately verifiable:

1. **Author the `REV`.** A revision is born a skeleton (`doctrine revision new
   "<title>"`); `revision change add` appends **one** `[[change]]` row per
   invocation, so the introduce row is its own command:
   `doctrine revision change add REV-NNN --action introduce --new-label FR-006
   --member-of SPEC-013 --new-statement "…"` — the frozen label is `FR-006`, the
   next free functional label (`FR-001`–`FR-005` are taken). The `SPEC-013`
   § *Uniform command grammar* prose naming the router is **not** a typed row: a
   `modify` row takes a live peer FK, and prose rides the `revision-NNN.md`
   companion, which `ADR-013` surfaces for manual handling. **Surfacing is not
landing, and nothing else does it:** no CLI verb edits spec prose — `spec edit`
sets the descent/parent scalars only (`src/commands/spec.rs`), as the revision's
own companion is prose — so the companion is applied by hand, and the phase has to
name that as an act rather than assume it (`RV-384` `F-17`).
2. **Manual apply.** After `revision approve` and `revision apply` (which surface
   the introduce row for manual handling rather than landing it), `doctrine spec
   req add SPEC-013 --kind functional --label FR-006 --title …` mints the
   requirement. It lands **`pending`** (`src/requirement.rs`), and a pending
   requirement with no cells reads **`Coherent`** — not `Indeterminate`. The
   closure lever is the reconcile flip: an `active` requirement with no verified
   evidence is residual drift `REQ-113` refuses, so the evidence must exist
   before the flip.
3. **Record coverage, then verify it.** `doctrine coverage record --slice 265
   --requirement <new REQ> --change SL-265 --mode VT --command cargo --command
   test --command=--test --command e2e_show_equivalence --matcher-source stdout
   --matcher-pattern "<test name> \.\.\. ok" --regex` binds a runnable check.
   Without the check binding the cell is a bare attestation that merely *says*
   `VT`; with it, the cell is a recipe that leans `Planned` until re-derived. So
   the phase then runs `doctrine coverage verify 265`, and **the exit criterion
   is the cell reading `Verified`, not merely recorded** (`RV-384` `F-13`,
   `F-7`).

4. **Land the prose by hand, then close the `REV`.** Two authored edits, neither
   reachable from a verb: `spec-013.md` § *Uniform command grammar*, whose line
   "A top-level `Command` enum names each entity kind" a top-level `show`
   falsifies, and its § *Responsibilities* paragraph — plus the matching entry in
   `spec-013.toml`'s structured `responsibilities` list, which mirrors that
   paragraph. Then `doctrine revision status REV-NNN done`: `ADR-013` requires
   every `[[change]]` row landed before `done`, and the introduce row landed at
   step 2 while the prose lands here, so this transition is the phase's last act
   rather than a formality (`RV-384` `F-17`).

The phase is sequenced **after** the code phase, because the evidence it records
is the code phase's test. At reconcile the requirement flips `pending → active`
with its `Verified` cell already in hand.

The slice's authored deliverable (the `REV` entity and
`.doctrine/spec/tech/013/**`) has no `src/` selector and reports as `undeclared`
in slice conformance — expected, disposed `aligned`.

<!-- doctrine:section sec-6 -->
## Code impact

| path | intended change |
|---|---|
| `src/commands/cli.rs` | `Command::Show` variant + dispatch arm; `show` added to the `explore` family; census 56 → 57 |
| `src/commands/show.rs` (new) | the router: `Route`, `route()`, `run_show()`, and the `every_kinds_row_routes` unit test — command tier, no layering row |
| `src/commands/guard.rs` | `Command::Show { .. } => Read` in the combined leaf arm |
| `src/main.rs` | `CommonShowArgs::format()` accessor; the router's parse/`write_class` test |
| `src/backlog.rs`, `src/knowledge.rs` | call the accessor at all four `--json` sites instead of the inline shorthand (behaviour-preserving; `run_show` bodies untouched) |
| `tests/e2e_show_equivalence.rs` (new) | byte-equivalence per numbered prefix, both formats |
| `tests/e2e_show_refusals.rs` (new) | unknown prefix / dangling ref / ambiguous bare id |
| `.doctrine/spec/tech/013/**`, a new `REQ` | the grammar amendment (governance phase) |
| `.doctrine/revision/NNN/**` | the `REV` staging the introduce row and the prose companion (governance phase) |
| `install/using-doctrine.md` | the read-entity row of the verb table names `doctrine show <REF>`, and the read-entity paragraph at `:138` follows |
| `install/routing-process.md` | the guardrail's "read entities via `doctrine <kind> show <ID>`" names the router; the boot snapshot regenerates (`sec-4`) |

The totality assertion is a **unit** test inside `src/commands/show.rs`, not an
integration test: `route()` and `KINDS` are `pub(crate)`, and `src/lib.rs`
exports no `kinds` items (`RV-384` `F-5`).

**Not touched:** any kind's `run_show` implementation; the `show` flag
declarations of the seven kinds that hand-declare them (`CommonShowArgs`
unification is a Follow-Up); the MCP tool list; `search` output.

Module home is `src/commands/show.rs` (command tier). `src/commands/mod.rs`
declares it with one `pub(crate) mod` line; `layering.toml` needs no new row. The
router imports `adr`/`policy`/`standard`/`rfc` and the other kind modules, all of
which `src/commands/cli.rs` already imports, so the home adds no top-level
layering edge (`RV-384` `F-8`).

<!-- doctrine:section sec-7 -->
## Verification

- **VT — byte-equivalence.** The property is that for every ref a kind's own
  `show` accepts, `doctrine show <REF>` emits stdout byte-identical to that
  kind's own `show` invocation; the test witnesses it with one fixture entity of
  every numbered prefix, in both `--format table` and `--format json`. This is
  the slice's central property; the fidelity contract is only as good as the
  bytes. The
  reference command is the kind's own verb, and there is no `doctrine req show`:
  `REQ` compares against `doctrine spec req show`; `PRD`/`SPEC` against
  `doctrine spec show`; the seven record prefixes against `doctrine knowledge
  show`; the five backlog prefixes against `doctrine backlog show`; the rest
  against `doctrine <kind> show`.
- **VT — totality (unit).** `every_kinds_row_routes` asserts every `KINDS` row
  yields a routed arm, plus a negative control asserting a synthetic unrouted
  prefix returns `None`.
- **VT — refusals.** An unknown prefix, a dangling ref and an ambiguous bare id
  each fail with `kinds::parse_resolvable_ref`'s error, unchanged.
- **VT — the new requirement.** The governance phase records a check-bound
  coverage cell and re-derives it with `coverage verify 265`; the exit is the
  cell reading `Verified`.
- **VA — behaviours preserved.** Every existing `<kind> show` golden passes
  unchanged; no kind `run_show` was modified. The `write_class` parse test
  asserts `show` classifies `Read`.
- **VA — surface visible.** `doctrine show` appears in `explore` in `--help` and
  `--boot-map`; the census assertion is updated, not deleted.
- **VA — governance landed.** `SPEC-013` carries the new `REQ` with its frozen
  label; `spec-013.md` § *Uniform command grammar* and § *Responsibilities*, and
  `spec-013.toml`'s `responsibilities` list, all name the top-level router; the
  `REV` reads `done` with every row landed; and the coverage cell reads
  `Verified`.
- **VA — guidance names the router.** `install/using-doctrine.md`'s verb table
  and `install/routing-process.md`'s guardrail name `doctrine show`, and the
  regenerated boot snapshot carries the guardrail with the router named. The
  guardrail's sentence is **not** covered today by
  `every_command_named_by_core_process_is_accepted_by_the_binary`, which parses
  the *core-process* paragraph and executes each invocation it finds there; the
  `<kind> show` line sits in the **Guardrails** sentence. The phase therefore
  extends that test with a `guardrails_paragraph` sibling to its
  `core_process_paragraph`, so the router the guardrail names is executed rather
  than left knowingly unguarded (`RV-384` `F-27`).

The equivalence test needs one entity per prefix; the corpus supplies one, and
the plan names them while the test enumerates them by group. **Both sides are
stdout from the same binary, invoked twice** — the reference is the kind's own
verb, not a stored golden, so there are no copied bytes to age and the test
compares this build against itself, reading the repo corpus it is run in
(`RV-384` `F-21`).

<!-- doctrine:section sec-8 -->
## Risks and residuals

- **`R1` — layering.** The home is `src/commands/show.rs` because a new
  top-level module (`src/show.rs`) would need a `layering.toml` tier row and trip
  the gate's `Unclassified` finding, not because a new file under `commands/`
  grows the tangle. The command tangle is ratcheted per top-level module; the
  router's home adds no such edge, provided it calls the per-kind wrappers rather
  than `governance::run_show` directly (`RV-384` `F-8`).
- **`R2` — census churn.** The census assertion is updated 56 → 57 deliberately.
- **`R3` — `REQ`.** The one numbered prefix whose `show` is not `spec::run_show`;
  it takes its own arm onto `spec::run_req_show`.
- **`R4` — goldens.** Boot-map/help goldens pin member presence and the
  leaf-no-subline rule, not a full list; the boot snapshot is regenerated.
- **`A1` — every numbered prefix has a working `show` today.** The totality unit
  test and the equivalence VT surface a counterexample if that stops holding.
- **Not a residual — `SPEC-013`'s existing members.** Its eight active
  requirements read `Indeterminate` today (`coverage show SPEC-013`), and
  `REQ-113`'s closure gate does **not** block on them. The gate's set is built
  from the requirements a slice's own `coverage.toml` physically covers —
  `coverage_scan::slice_local_covered_reqs`, whose own doc calls it "the
  closure-gate `covered` term" — plus what the slice lists explicitly and names in
  its own reconciliation records. Untouched members are outside it by
  construction, so no REC-excuse is owed. Stated from the code rather than
  deferred to planning (`RV-384` `F-22`).
- **Residual — `CommonShowArgs` unification.** Seven kinds still hand-declare the
  `--format`/`--json`/`--path` triple. Collapsing them onto the flatten is a
  companion change (`REQ-199`'s precedent), deliberately not absorbed here.
- **Residual — the guidance sweep.** The canon and walkthrough skills,
  `authority-model.md`, and the shipped memories still prescribe the per-kind
  form; filed as `CHR-078` (`RV-384` `F-18`).
- **Residual — `search` verb hint.** A column naming the `show` verb an id needs
  would prevent the RFC-011 failure at its source as well as dissolving it.
- **Residual — MCP parity.** No `doctrine_show` tool; the MCP surface is a
  hand-authored list and widening it is a separate decision.

