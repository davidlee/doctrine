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
> ~30-LOC entrypoint. Pure leaf/engine modules stay pure — **clap never enters a
> leaf** (ADR-001, already the rule at the `CommonListArgs` site).

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
  main.rs              ~30-LOC entrypoint: parse Cli → resolve color →
                       guard::worker_guard(&cmd)? → cli::dispatch(cmd, color)
  adr.rs, slice.rs …   kind modules (already command-tier): each GAINS its own
                       subcommand enum + `pub(crate) fn dispatch(cmd, color)`
  commands/
    mod.rs
    map.rs, serve.rs   (exist — the precedent)
    cli.rs             top-level `Cli` + `Command` enum + the thin dispatch match
    guard.rs           write_class + worker_guard + WriteClass  (was main.rs)
    list.rs            CommonListArgs + into_list_args  (was main.rs; imports only `listing`)
    relation.rs        link / unlink         → relation(eng), memory, integrity
    dep_seq.rs         needs / after / after-remove / after-prune → dep_seq(leaf)
    supersede.rs       supersede             → supersede(eng), knowledge
    validate.rs        validate              → integrity, relation_graph
    inspect.rs         inspect               → memory, relation_graph, priority
    facet.rs           estimate + value      → estimate(leaf), value(leaf), facet_write
    coverage.rs        coverage subcommands  → coverage_store/verify/view (all engine)
    test_helpers.rs    cfg(test) seed_* fixtures (mirrors src/catalog/test_helpers.rs)
```

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

### F-A — pre-existing parallel resolvers (recorded, deferred)

Four near-identical id→toml-path resolvers (`resolve_link_path`,
`resolve_supersede_path`, `resolve_dep_seq_src_path`,
`resolve_entity_path_and_canonical`) are adjacent in `main.rs` today. This
relocation scatters them across four `commands/` files, burying an obvious DRY
target. Consolidation is **behaviour-changing** and out of this slice's
mechanical scope. Captured as **IMP-131** before the scatter; each relocated
resolver carries a one-line breadcrumb comment → IMP-131.

## 4. Clap-enum redistribution (thrust 2)

**Dispatch entry (uniform):**
`pub(crate) fn dispatch(command: AdrCommand, color: ResolvedColor) -> anyhow::Result<()>`.
The sole global is `--color` (resolved once in `main()`); `path` already lives in
each variant; the per-variant `if json { Json } else { format }` normalization
moves into the kind dispatch.

- **Clean kind homes** (already command-tier): `Adr`→adr, `Policy`→policy,
  `Standard`→standard, `Rfc`→rfc, `Spec`→spec, `SpecReq`→requirement,
  `Backlog`→backlog, `Knowledge`→knowledge, `Memory`→memory, `Slice`→slice,
  `ConceptMap`→concept_map, `Review`→review, `Rec`→rec, `Revision`→revision,
  `Skills`→skills, `Worktree`→worktree, `Dispatch`→dispatch, `Boot`→boot,
  `Catalog`→catalog/.
- **Nested enums ride their parent:** `CandidateCommand`⊂Dispatch,
  `SyncCommand`⊂Memory, `ExportCommand`⊂ConceptMap, `RevisionChangeCommand`⊂Revision.
- **`commands/` shells** (data below command tier): `Coverage`→
  `commands/coverage.rs` (all `coverage*` modules are engine), `Map`→
  `commands/map.rs` (exists).

### F-B — shared clap bundle forces phase-1 sequencing

`CommonListArgs` (the list-surface spine + `into_list_args`) is in `main.rs` and
flattened into every kind's `list` variant. Left there, each relocated kind enum
imports `crate::CommonListArgs` → **kind → main**; with the existing **main →
kind** (the `Command` field types) that is a `main ↔ kind` 2-cycle for ~16 kinds
→ grows `[tangle_baseline] command = 120` → `just gate` fails.

**Fix (phase 1, the unblock):** extract `CommonListArgs` + `into_list_args` into
`commands/list.rs`, importing only `listing` (leaf). Kinds then depend on
`commands::list` (acyclic — it imports no kind), not back on `main`. A plan task
audits `main.rs` for any *other* clap type referenced by a relocated enum and
pulls it into `commands/list.rs` too.

## 5. `main.rs` end-state & the cycle-free argument

### F-C — `guard` ↔ `Command` cycle avoidance

`guard.rs` must import the `Command` type (it matches the whole tree); whoever
calls `worker_guard` must import `guard`. If `Command` stays in `main.rs` and
`main()` calls `worker_guard`, that is `main ↔ guard` → tangle growth.

**Resolution:** `Cli` + `Command` + the dispatch match → `commands/cli.rs`;
`guard.rs` imports `commands::cli::Command`; `main()` remains the orchestration
root —
`parse → resolve color → guard::worker_guard(&cmd)? → cli::dispatch(cmd, color)`.

**Cycle-free invariant (the whole-slice safety argument):** every new `commands/`
unit (`cli`, `guard`, `relation`, `dep_seq`, `supersede`, `validate`, `inspect`,
`facet`, `coverage`, `list`) is imported only by the **acyclic root `main`** (or
by `cli`/`guard`, which `main` roots, in one direction). Nothing imports `main`.
Therefore no new unit joins the command SCC, and the cyclic-edge count cannot
grow. `main.rs` ends at ~30 LOC.

## 6. Verification

| # | evidence | mode | proves |
|---|---|---|---|
| V1 | `tests/e2e_*` golden suites pass **unchanged** (zero golden-file edits) | VT | CLI behaviour byte-identical (behaviour-preservation gate, AGENTS.md) |
| V2 | relocated `#[cfg(test)]` modules pass in new homes, assertions unchanged (only `use super::*` / path fixups) | VT | logic preserved through the move |
| V3 | `tests/architecture_layering.rs` green: **no new `[[accepted_violation]]`**; `[tangle_baseline] command = 120` **unchanged** | VT | ADR-001 honored — no upward edge, no new cycle (F-B + F-C make this provable) |
| V4 | `just gate` green — clippy zero-warn `--workspace`, fmt | VT | house standard |
| V5 | grep: no relocated `run_*` remains in `main.rs`; no `enum *Command` in `main.rs` (incl. top-level `Command`, now in `commands/cli.rs`) | VA | decomposition complete, not partial |
| V6 | `wc -l src/main.rs` ≈ 7264 → ~30 | VA | closure's "materially reduced" LOC objective |

**No `layering.toml` edits required.** `commands` stays a uniform `command`
umbrella — every new sub-file is at-or-below command tier (`commands::list`
reaches only `listing`(leaf), *below*, so no forced sub-classification). Courtesy
only: refresh the dep-list comment on the `commands` row.

## Decisions log

- D1 dispatch arm moves with the enum · D2 `write_class` centralized in
  `commands/guard.rs` · D3 same-stem shells allowed.
- F-A resolver dedup deferred → IMP-131 · F-B `commands/list.rs` extracted first ·
  F-C `Command`+dispatch in `commands/cli.rs`, `guard` separate, `main` roots both.
- Scope: full (all ~25 enums + 7 orphan units land in this slice; `/plan` phases
  per-domain).

## Open questions

None blocking. `/plan` settles phase granularity (suggested: phase-1
`commands/list.rs` + `cli.rs` + `guard.rs` scaffold → then orphan shells → then
kind-enum batches by domain).
