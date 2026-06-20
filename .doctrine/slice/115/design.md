# SL-115 — Design: decompose `main.rs`

Behaviour-preserving decomposition of `src/main.rs` (7264 LOC). Relocate the
stranded `run_*` command shells and the entire clap surface out of `main.rs`,
into command-tier modules, under one convention. No CLI behaviour changes
(names, args, output identical — the behaviour-preservation gate).

## 1. The convention (resolves the `commands/`-vs-dispatch ambiguity)

> A command's clap surface **and** its dispatch live with its **owning module
> when that module is command-tier**. When the owning data/policy lives **below**
> command tier (leaf/engine), the CLI shell goes in a thin **command-tier
> `commands/` module** that calls *down* into it. `main.rs` is reduced to a
> ~250-LOC stub (shared leaf-only clap bundles + `Cli` + `main()`; §5). Pure
> leaf/engine modules stay pure — **clap never enters a leaf** (ADR-001, already
> the rule at the `CommonListArgs` site).

There is **one** folder convention (`commands/`), riding the existing precedent
(`commands/serve.rs`, `commands/map.rs` already pair `Args` + `run_`). No
parallel `cli/` folder is introduced — that would re-create the very ambiguity
the slice closes.

### Load-bearing decisions

- **D1 — the dispatch arm moves *with* the enum.** A new kind must not touch
  `main.rs`. Both the subcommand enum and its match-arm body move into the kind
  module behind `pub(crate) fn dispatch(command, color)`; the call site collapses
  to one line per kind. Moving the enum alone would leave `main.rs` growing per
  kind (the arm).
- **D1a — sideways-call corollary (route to where the impl lives).** An arm body
  moved from `main` into a kind module *mints a production edge from that kind to
  every command-tier module the body calls* — edges that were inert `main→X`
  before. If the destination kind does **not already import** that module, this is
  a NEW command-tier edge that can grow the tangle baseline (the sink proof does
  **not** cover it — §5). **Rule:** route a kind's dispatch to the module whose
  `run_*` the body actually calls, **not the nominal kind name**. Concretely: (i)
  if the body calls only the kind's own module → clean own-module fold; (ii) if it
  calls *sideways* into a command-tier module the nominal kind doesn't import,
  either fold into **that** module when it is the implementation's real home and
  the edge already exists/is a self-edge, or keep the dispatch in a `commands/`
  **sink shell** (sink-safe by §5). Never fold into a nominal kind that mints a new
  cross-command cycle. **Two known instances** (the plan audit confirms no others):
  - **`MemoryCommand::Sync`** calls `corpus::run_sync`/`run_sync_install`
    (`main.rs:3688,3690`); `corpus` already imports `memory` (`corpus.rs:38`), so
    folding Sync into `memory.rs` mints `memory→corpus` → `corpus↔memory` 2-cycle →
    tangle 120→≥121. **Fix:** Sync's dispatch stays in `commands/` (the residual
    match → `commands/cli.rs`), not `memory.rs` — a `commands→corpus` sink out-edge.
  - **`SpecReqCommand`** calls `spec::run_req_*` (`main.rs:4139,4145,4150`), which
    live in `spec.rs`; `requirement.rs` has no CLI and doesn't import `spec`, while
    `spec` imports `requirement`. **Fix:** `SpecReq` folds into **`spec.rs`**
    (own-module, zero edge), not `requirement.rs` (which would mint
    `requirement→spec` → `spec↔requirement` cycle). See §4.

  (Top-level verbs `Install`/`Status`/`Reseat` are unaffected — they home in
  `commands/cli.rs`, so their `install`/`status`/`integrity` calls are harmless
  `commands→X` sink out-edges; `MemoryCommand::Find`/`Retrieve` are fine because
  `memory→retrieve` already exists.)
- **D2 — `write_class` stays centralized** (`commands/guard.rs`), not distributed
  per kind. It is the SL-056 / ADR-008 worker-mode write gate — security-critical.
  One auditable `Read`/`Write`/`Orchestrator` match beats 25 scattered
  classifications where a missed `Write` silently ungates a worker. Deliberate
  consequence: adding a write verb still touches `guard.rs` — a tripwire forcing a
  conscious gate classification, unlike the mechanical dispatch glue.
- **D3 — same-stem command shells are allowed and intentional.**
  `commands::supersede` (shell) over `crate::supersede` (engine policy);
  `commands::dep_seq` over `crate::dep_seq` (leaf data). Rust disambiguates by
  path; the parallel naming signals the shell↔data pairing.

## 2. Target topology

```
src/
  main.rs              shared cross-kind clap arg bundles that depend ONLY on
                       leaves (CommonListArgs + into_list_args) + `Cli` + `fn main()`.
                       Stays at crate root because `main` is INERT to the layering
                       gate (see §5). ~7264 → ~250 LOC.
  adr.rs, slice.rs …   kind modules (already command-tier): each GAINS its own
                       subcommand enum + `pub(crate) fn dispatch(cmd, color)`
  memory.rs            ALSO receives FindRetrieveArgs (memory-coupled clap bundle —
                       cannot be shared without a memory↔X cycle; §4)
  commands/            ALL collapse to ONE gate unit `commands` — must stay a SINK
    mod.rs
    map.rs, serve.rs   (exist — the precedent)
    cli.rs             `Command` enum + the thin dispatch match
    guard.rs           write_class + worker_guard + WriteClass  (was main.rs)
    relation.rs        link / unlink         → relation(eng), memory, integrity
    dep_seq.rs         needs / after / after-remove / after-prune → dep_seq(leaf)
    supersede.rs       supersede             → supersede(eng), knowledge
    validate.rs        validate              → integrity, relation_graph
    inspect.rs         inspect               → memory, relation_graph, priority
    facet.rs           estimate + value      → estimate(leaf), value(leaf), facet_write
    coverage.rs        coverage subcommands  → coverage_store/verify/view (all engine)
    test_helpers.rs    cfg(test) seed_* fixtures (mirrors src/catalog/test_helpers.rs)
```

**Note:** there is no `commands/list.rs`. The shared `CommonListArgs` bundle
stays in `main.rs` (crate root) — see §4 F-B (revised) and the §5 sink invariant
for why routing it through `commands/` would break the build.

## 3. Orphan runner relocation (thrust 1)

Every orphan runner is **command-tier** (each does `root::find` + IO + calls a
command-tier module). The slice's original premise — relocate into
`links.rs`/`dep_seq.rs`/`supersede.rs` — is wrong on two axes and is **superseded**
by this design:

1. **Naming collision.** `links.rs` is *wikilink extraction* (leaf, `out=0`,
   pure). `run_link`/`run_unlink` are the `doctrine link`/`unlink` **structural
   relation** verbs (`relation` + `memory` + `integrity`). Zero overlap; dropping
   them into `links.rs` is semantically wrong and turns a pure leaf into a
   command-dep importer (ADR-001 upward-edge violation).
2. **Tier mismatch.** `dep_seq` is **leaf** (`out=0`), `supersede` is **engine**.
   The runners are command-tier shells; the *data/policy* they need already sits
   at the right tier (`dep_seq::append`, `supersede::validate_matrix`,
   `relation::append_edge`). Only the thin shell was stranded.

| destination | runners | private helpers move with it | tests |
|---|---|---|---|
| `commands/relation.rs` | `run_link`, `run_unlink` | `resolve_link_path` | `link_*` / `unlink_*` |
| `commands/dep_seq.rs` | `run_needs_edge`, `run_after_edge`, `run_after_remove`, `run_after_prune` | `resolve_dep_seq_src`, `resolve_dep_seq_src_path`, `is_work_like` | `is_work_like_*` |
| `commands/supersede.rs` | `run_supersede` | `resolve_supersede_path`, `rel_array` | `supersede_*` (12) |
| `commands/validate.rs` | `run_validate` | — | (from big `mod tests`) |
| `commands/inspect.rs` | `run_inspect` | — | (golden-covered) |
| `commands/facet.rs` | `run_estimate_set/clear`, `run_value_set/clear` | `resolve_entity_path_and_canonical` | `estimate_value_tests` (whole module) |
| `commands/guard.rs` | `write_class`, `worker_guard` | `WriteClass` | `write_class_tests` (whole module) |

`estimate` + `value` collapse into one `commands/facet.rs` — they share
`resolve_entity_path_and_canonical`, the codebase treats them as one facet family
(`facet_write` leaf, combined `estimate_value_tests`). File boundary = where the
private helpers stop being shared.

### F-A — shared id→path core is already owned by SL-129 / IMP-067

The four resolvers (`resolve_link_path`, `resolve_supersede_path`,
`resolve_dep_seq_src_path`, `resolve_entity_path_and_canonical`) are **not**
duplicates of each other — each layers distinct domain logic (relation rule,
supersede policy, dep-seq source, canonical id) over one **shared core**: the
`<dir>/<NNN>/<stem>-<NNN>.toml` id→path formula. That core is exactly what
**SL-129** introduces as `entity::id_path` (corpus-wide; backlog **IMP-067**).

So SL-115 does **not** file a new dedup item and does **not** merge the
resolvers. It relocates them as-is into their `commands/` shells; SL-129 (when it
lands) replaces the path-formula line inside each with `entity::id_path`. Each
relocated resolver carries a one-line breadcrumb comment → SL-129. See the
**SL-129 coordination** note below — these two slices overlap on `main.rs` and
must be sequenced.

## 4. Clap-enum redistribution (thrust 2)

**Dispatch entry (uniform):**
`pub(crate) fn dispatch(command: AdrCommand, color: bool) -> anyhow::Result<()>`.
The sole global is `--color`, resolved once in `main()` to a `bool`
(`tty::resolve_color → bool`); `path` already lives in each variant; the
per-variant `if json { Json } else { format }` normalization moves into the kind
dispatch.

- **Clean kind homes** (already command-tier): `Adr`→adr, `Policy`→policy,
  `Standard`→standard, `Rfc`→rfc, `Spec`→spec, **`SpecReq`→`spec`** (see note),
  `Backlog`→backlog, `Knowledge`→knowledge, `Memory`→memory, `Slice`→slice,
  `ConceptMap`→concept_map, `Review`→review, `Rec`→rec, `Revision`→revision,
  `Skills`→skills, `Worktree`→worktree, `Dispatch`→dispatch, `Boot`→boot,
  `Catalog`→catalog/.
  - **`SpecReq`→`spec`, NOT `requirement`** (codex round 3): `requirement.rs` has
    no standalone CLI (`requirement.rs:19`, "spec-mediated"); the `run_req_*`
    shells already live in `spec.rs` (`spec.rs:853,913,1578`), so `SpecReq` folds
    into `spec.rs` as an **own-module** move (zero new edge). Routing it to
    `requirement` (which does not import `spec`) would mint `requirement→spec` and
    close a `spec↔requirement` cycle (D1a). General rule: route a dispatch to the
    module its `run_*` body actually calls, not the nominal kind name.
- **Nested enums ride their parent:** `CandidateCommand`⊂Dispatch,
  `SyncCommand`⊂Memory, `ExportCommand`⊂ConceptMap, `RevisionChangeCommand`⊂Revision.
- **`commands/` shells** (data below command tier): `Coverage`→
  `commands/coverage.rs` (all `coverage*` modules are engine), `Map`→
  `commands/map.rs` (exists).

### F-B (revised) — shared clap bundles stay at crate root; never in `commands/`

**Correction (codex review):** the original F-B was solving a phantom. The
layering gate (`tests/architecture_layering.rs`) discovers units by **top-level
module** and *excludes `main.rs`* (`discover_units` skips it; `count_tangle_edges`
ranges only over discovered same-tier units). So a kind enum referencing
`crate::CommonListArgs` (a crate-root type defined in `main.rs`) produces **no
unit-level edge** — it is inert. There is no `main ↔ kind` tangle cycle to fix.

Worse, the proposed fix would have *created* the real defect: every
`src/commands/*.rs` collapses to the single unit `commands`, so putting
`CommonListArgs` in `commands/list.rs` makes each kind enum's
`list: CommonListArgs` field a **`kind → commands`** edge — and the dispatch
shells already give **`commands → kind`** (e.g. `commands/relation.rs → memory`).
Together: `commands ↔ kind` cycles → `commands` joins the command SCC → tangle
blows past 120 → gate fails.

**Resolution:**
- **`CommonListArgs`** (depends only on `listing` + `tty`, both leaf) **stays in
  `main.rs`** (crate root, inert). Kind enums reference `crate::CommonListArgs` —
  no unit edge. No new module, no `layering.toml` row.
- **`FindRetrieveArgs`** depends on `memory::{MemoryType, Status, Lifespan}`
  (command-tier) and is flattened into `MemoryCommand::{Find,Retrieve}`. It
  **cannot** be a shared module (anything importing it ↔ `memory`). It moves into
  **`memory.rs`**, co-located with `MemoryCommand`. Safe: the `Find`/`Retrieve`
  dispatch bodies call `retrieve::*`, and `memory→retrieve` is an existing
  production edge. **But** the sibling `MemoryCommand::Sync` arm calls `corpus::*`
  and `memory→corpus` does **not** exist today — so Sync's dispatch does **not**
  move with the rest of `MemoryCommand`; it stays in a `commands/` sink shell
  (D1a). Only the clap struct + the non-Sync arms land in `memory.rs`.
- **General rule:** a shared clap bundle with leaf-only deps → crate root
  (`main`, inert); a clap bundle coupled to a command/engine module → that
  module. **Never `commands/`** (would break the sink — §5). A plan task audits
  `main.rs` for any *other* shared clap type and routes it by this rule.

## 5. The gate model & the sink invariant (the real safety argument)

**How the gate actually measures** (`tests/architecture_layering.rs`, verified):
- **Units = top-level modules.** `src/foo/bar.rs` collapses to unit `foo`. So
  **every `src/commands/*.rs` is the single unit `commands`** — the gate cannot
  see per-file structure inside it.
- **`main.rs` is excluded** from `discover_units`, and `count_tangle_edges` ranges
  only over discovered same-tier units. Edges to/from `main` are inert; crate-root
  types (`crate::CommonListArgs`) are not units, so referencing them adds no edge.
- **Tangle** = edges whose *both* endpoints lie in the same non-trivial
  (size > 1) same-tier SCC. A unit with no inbound edge from its tier cannot be in
  an SCC → contributes **0** tangle, regardless of out-degree.

### The sink invariant (replaces the old file-level argument)

> **`commands` must stay a pure sink** — no command-tier module may import it.

Verified: today only `main` imports `crate::commands` (and `main` is inert), so
`commands` is already a sink contributing 0 tangle. The decomposition *adds many
out-edges* from `commands` (`cli`→every kind enum, `relation`→`memory`,
`coverage`→`coverage_store`, …) but **no inbound** command-tier edge — provided no
kind ever imports `commands`. That is exactly why the §4 shared-args rule forbids
`commands/list.rs`: a kind's `list: CommonListArgs` field must not point into
`commands`. Hold the invariant and `commands` stays out of the SCC → it adds **0**
tangle no matter how many kind/engine modules the shells call.

**Scope of the sink proof (what it does NOT cover).** The sink invariant proves
only that `commands` itself stays out of the command SCC. It says nothing about
edges **between two kind/engine modules** that the refactor introduces. D1 relocates
each dispatch arm's body from inert `main` into a command-tier kind module, so any
command-module call inside a body becomes a *new* production kind→X edge. If X is
command-tier and the kind didn't already import it, that edge can enlarge the
command SCC and grow tangle past 120 — **independently of the sink**. Verified
exemplar: `MemoryCommand::Sync → corpus::*` would mint `memory→corpus`, cycling with
the existing `corpus→memory`. The D1a carve-out (sideways callers stay in a
`commands/` sink shell) neutralises this class. **Therefore `[tangle_baseline]
command = 120` is not asserted as automatically unchanged — it is an obligation the
plan must verify per batch (V3), and the per-arm sideways-call audit (below) is a
mandatory plan task, not an assumption.**

### F-C (revised) — relocation is for LOC, not cycles

Moving `Command` + the dispatch match → `commands/cli.rs`, and
`write_class`/`worker_guard` → `commands/guard.rs`, is purely to shrink `main.rs`.
There was never a `main ↔ guard` *tangle* risk — `main` is inert. `Cli` (the
`clap::Parser` with the `--color` global) **stays in `main.rs`** with `main()`,
the orchestration root: `Cli::parse → resolve color → guard::worker_guard(&cmd)?
→ cli::dispatch(cmd, color)`. `guard` importing `commands::cli::Command` is an
*intra-`commands`* reference (same unit — self-edges are dropped), so it adds
nothing. **`main.rs` retains the leaf-only shared clap bundles (`CommonListArgs`)
+ `Cli` + `main()`** — ends ~250 LOC (not ~30; the shared args must stay at crate
root, by §4). Still a ~96% cut.

## 6. Verification

| # | evidence | mode | proves |
|---|---|---|---|
| V1 | `tests/e2e_*` golden suites pass **unchanged** (zero golden-file edits) | VT | runtime CLI behaviour byte-identical for covered paths (behaviour-preservation gate, AGENTS.md). **Scope caveat:** `e2e_list_conformance` is *parse-only* over 8 `SPINE_KINDS` — it does **not** cover `--help` text or every `CommonListArgs` consumer (e.g. `ConceptMapCommand::List` is outside it). See V7. |
| V2 | relocated `#[cfg(test)]` modules pass in new homes, assertions unchanged (only `use super::*` / path fixups) | VT | logic preserved through the move |
| V3 | `tests/architecture_layering.rs` green: **no new `[[accepted_violation]]`**; `[tangle_baseline] command = 120` **unchanged** | VT | ADR-001 honored — no upward edge, `commands` stays a sink (the §5 invariant makes this provable). **Run the gate per batch** — the sink proof does not cover kind→kind edges minted by D1 body relocation (§5 scope note); any growth is a stop/restructure per D1a, never an auto-accept. |
| V4 | `just gate` green — clippy zero-warn `--workspace`, fmt | VT | house standard |
| V5 | grep: no relocated `run_*` remains in `main.rs`; no `enum *Command` in `main.rs` (top-level `Command` now in `commands/cli.rs`); `commands/` contains no `*ListArgs`/shared-arg struct (sink invariant) | VA | decomposition complete + invariant held |
| V6 | `wc -l src/main.rs` ≈ 7264 → ~250 (shared leaf-only clap bundles + `Cli` + `main()` stay) | VA | closure's "materially reduced" LOC objective |
| V7 | **new** — (a) capture a clap `CommandFactory` `--help` snapshot (top-level + representative nested subcommands) **before** the refactor; assert byte-identical **after**; (b) **parse-regression tests** (`Cli::try_parse_from`, extending the existing seam at `main.rs:5284`) over the parser-only contracts a `--help` diff cannot see — `CommonListArgs` `value_delimiter`; `FindRetrieveArgs` `conflicts_with="offset"` + `value_parser` on memory types; `Retrieve` `value_parser=retrieve::parse_min_trust`; `After` `required_unless_present`/conflicts; `DispatchCommand::Sync` `requires="trunk"`; (c) parse coverage for every moved `CommonListArgs` consumer not in `SPINE_KINDS`. | VT | help text **and** arg-group / `value_parser` / `conflicts_with` / `value_delimiter` / `requires` wiring unchanged by the enum moves — `--help` snapshot alone proves only displayed surface (codex round 2) |

**No `layering.toml` edits required.** No new top-level unit is introduced
(`CommonListArgs` stays in `main`; all shells collapse into the existing
`commands` unit). `commands` stays a uniform `command` umbrella — every sub-file
is at-or-below command tier, and `commands` stays a sink, so neither the
`MixedUmbrella` assertion nor the tangle ratchet fires. Courtesy only: refresh the
dep-list comment on the `commands` row.

## 7. Adversarial review findings

- **Dispatch signature** corrected `ResolvedColor` → `bool` (`tty::resolve_color`
  returns `bool`).
- **`is_work_like`** confirmed dep_seq-local (used only inside
  `resolve_dep_seq_src`) → clean move to `commands/dep_seq.rs`, no external edge.
- **`write_class`** confirmed called only by `worker_guard` in production (rest
  are tests) → moves cleanly to `commands/guard.rs`.
- **F-A reframed** — the resolvers' shared id→path core is SL-129/IMP-067's
  surface, not a new dedup item (IMP-131 was filed then closed as duplicate).

### Codex round (external adversarial pass) — two blockers, verified & folded

Codex (GPT-5.5) attacked the layering proof; I verified each claim against
`tests/architecture_layering.rs` and **both blockers held**:

- **B1 — wrong graph granularity.** The original §5 reasoned at *file* level, but
  the gate discovers units by **top-level module** and **excludes `main.rs`**, and
  collapses all `src/commands/*.rs` into the single unit `commands`. The
  file-level "cycle-free invariant" was unprovable as written. **Fixed:** §5
  rewritten around the real model — the **sink invariant** (`commands` must have
  zero command-tier inbound edges; verified it does today).
- **B2 — the old F-B fix created the real cycle.** Extracting `CommonListArgs`
  into `commands/list.rs` would make each kind's `list:` field a `kind → commands`
  edge, and the shells give `commands → kind`, so `commands` would join the SCC
  and blow the tangle baseline. Also `FindRetrieveArgs` depends on `memory`
  parsers → can't be a shared `commands/` type at all. **Fixed:** F-B inverted —
  leaf-only shared args stay at crate root (`main`, inert); `FindRetrieveArgs`
  goes to `memory.rs`; **no `commands/list.rs`**.
- **B3 (major) — §6 over-claimed behaviour preservation.** `e2e_list_conformance`
  is parse-only over 8 `SPINE_KINDS`; no `--help` snapshot exists; some moved
  `CommonListArgs` consumers (`ConceptMapCommand::List`) are uncovered. **Fixed:**
  V1 caveated, V7 added (CommandFactory `--help` snapshot + parse coverage for all
  moved consumers).

### Codex round 2 (confirmation pass) — one blocker, one finding, verified & folded

Thread `019ee5ec-7d6e-71b2-855e-83d1502a640f`. Re-ran GPT-5.5 read-only on the
revised §4/§5/§6. It **confirmed** the sink invariant (point 1) and the shared
arg-bundle audit (point 3 — the seven `Args` structs in `main.rs` are exhaustively
routed; only `FindRetrieveArgs` carries command-tier deps). Two items, both
verified against source before folding:

- **B4 (blocker) — D1 mints kind→kind edges the sink proof never covered.** Moving
  a dispatch arm's body from inert `main` into a command-tier kind module turns its
  command-module calls into new production edges. Concrete: `MemoryCommand::Sync`
  calls `corpus::run_sync`/`run_sync_install` (`main.rs:3688,3690`); `corpus`
  already imports `memory` (`corpus.rs:38,313`; `memory→corpus` absent from the
  `memory` layering row) → folding Sync into `memory.rs` closes a `corpus↔memory`
  2-cycle → tangle 120→≥121. **Verified** the full dispatch region: this is the
  **only** new cycle-forming edge — `install`/`status`/`integrity::run_reseat` are
  top-level verbs homing in `commands/cli.rs` (sink), `boot` is own-module,
  `retrieve` is a pre-existing `memory` edge. **Fixed:** D1a carve-out + §5 scope
  note + V3 per-batch gate + the plan audit task; `MemoryCommand::Sync` dispatch
  stays in a `commands/` shell.
- **B5 (finding) — V7 `--help` snapshot proves only displayed surface.** It misses
  `value_delimiter`, `conflicts_with`, `value_parser`, `required_unless_present`,
  `requires` (codex cited `main.rs:124,146,229,716,2134,1027,1040,1047`). **Fixed:**
  V7 extended with `Cli::try_parse_from` parse-regression tests over those
  contracts.

### Codex round 3 (confirmation of the round-2 fold) — one blocker, folded

Same thread. Re-ran GPT-5.5 read-only on the folded design to verify the D1a fix.
It **confirmed** the carve-out is sink-safe (point 1 — nothing on the `corpus`
side transitively imports `commands`), that the memory→corpus direction argument
is stated correctly, that no engine-tier carve-out is needed (body relocation
mints `command→X` edges, never `engine→engine`), and that the phase ordering is
forced and correct (point 4). It **refuted** my "only known instance" claim:

- **B6 (blocker) — `SpecReq → requirement` is a second cycle-former.**
  `SpecReqCommand` arms call `spec::run_req_add/status/list`
  (`main.rs:4139,4145,4150`), implemented in `spec.rs:853,913,1578`.
  `requirement.rs` has no standalone CLI (`requirement.rs:19`) and does not import
  `spec`; `spec` already imports `requirement` (`spec.rs:41`). So §4's
  `SpecReq→requirement` would mint `requirement→spec` → new `spec↔requirement`
  cycle. **Verified** all five facts against source. **Fixed:** route `SpecReq`
  into `spec.rs` (own-module, zero edge) — §4 + D1a updated; the audit rule
  generalised to "route to where the `run_*` lives, not the nominal kind name."

### R1 — SL-129 coordination (cross-slice conflict, decision needed)

**SL-129** (`entity::id_path` corpus-wide; in design) and **SL-115** both heavily
edit `src/main.rs` and the same ~13–16 kind modules:

- SL-129 replaces ~93 id→path sites across 13 files, incl. `main.rs` (8,
  test-only) and KIND-decl / `format!` sites in `adr.rs`, `slice.rs`,
  `backlog.rs`, …; it also removes `KindRef::stem` (in `integrity.rs`).
- SL-115 guts `main.rs` (→ ~250 LOC), **moves** its test modules into `commands/`,
  and appends an enum + `dispatch` to each kind module.

These collide: SL-115 *moves* the very `main.rs` test sites and kind-module
regions SL-129 *edits*. A find-replace slice (SL-129) is fragile to files moving
underneath it.

**Recommendation: sequence SL-129 → SL-115** with an `after` edge (`SL-115 after
SL-129`). SL-129 consolidates id→path on the *current* layout (its inventory stays
valid); SL-115 then relocates the now-thinner shells (the F-A breadcrumb concern
evaporates — the path core is already `entity::id_path` before the move).
Alternative orders (SL-115 first, or parallel) force a stale SL-129 site inventory
or manual conflict resolution on a 93-site sweep. **Resolved:** the `SL-115 after
SL-129` edge is written; SL-129 lands first.

## Decisions log

- D1 dispatch arm moves with the enum · **D1a** route dispatch to where the `run_*`
  lives, not the nominal kind — sideways cycle-formers go to a `commands/` sink
  shell or their real home (proven: `MemoryCommand::Sync`→`corpus` → shell;
  `SpecReq`→`spec` not `requirement`) · D2 `write_class` centralized in
  `commands/guard.rs` · D3 same-stem shells allowed.
- F-A resolver path-core owned by SL-129/IMP-067 (no in-slice dedup) · **F-B
  (revised)** leaf-only shared args stay in `main` (inert), `FindRetrieveArgs`→
  `memory.rs`, **no `commands/list.rs`** · **F-C (revised)** `Command`+dispatch→
  `commands/cli.rs`, `guard`→`commands/guard.rs` for LOC (not cycles).
- §5 **sink invariant**: `commands` must stay a pure sink — the whole-slice
  layering-safety argument (verified against the gate).
- R1 sequence SL-129 → SL-115 (`after` edge written; SL-129 lands first).
- Scope: full (all ~25 enums + 7 orphan units land in this slice; `/plan` phases
  per-domain).

## Open questions

None blocking. `/plan` settles phase granularity (suggested: phase-1 the V7
`--help`/parse-regression snapshot baseline + `commands/{cli,guard}.rs` scaffold +
`FindRetrieveArgs`→`memory.rs` → then orphan shells → then kind-enum batches by
domain).

**Mandatory plan task (from codex rounds 2–3 / D1a):** before relocating any
clean-kind arm, audit each arm body for calls into a command-tier module the owning
*nominal* kind does **not** already import; route the dispatch to the module its
`run_*` actually calls (own-module fold) or a `commands/` sink shell — never to a
nominal kind that mints a cross-command cycle. **Two known instances:**
`MemoryCommand::Sync`→`corpus` (→ `commands/` shell) and `SpecReqCommand`→`spec`
(fold into `spec.rs`, NOT `requirement.rs`). The gate (V3) is run per batch and any
tangle growth halts the batch (D1a), never auto-accepts.
