# bootstrap doctrine

> just --list        # common tasks
> just check         # pre-commit gate (lint + test + format) — must pass before commit
> just list-memories # do it - now - to see what's there

Use this "system" for recording / retrieving memories instead of claude built-in
memory. Add a memory by dropping a markdown file in `doc/memories/`; recall via
`just list-memories`.

The CLI is not on PATH during dev — invoke the built binary directly:
`./target/debug/doctrine …` (after `cargo build`). Examples below use `doctrine`.

---

## layout

```
.doctrine/slice/nnn/
  slice-nnn.toml  metadata, relations, lifecycle status
  slice-nnn.md    scope document
  design.md       canonical technical design
  plan.toml       authored phase plan (objectives, EN/EX/VT criteria, links)
  plan.md         plan prose — rationale & sequencing (no queried data: storage rule)
  notes.md        durable implementation notes (on-demand)
  handover.md     disposable agent context — GITIGNORED
  audit.md        verification / code-review / drift findings (hand-made; no scaffold yet)
  phases ->       symlink into the runtime state tree (GITIGNORED)

.doctrine/state/slice/nnn/phases/   runtime phase tracking (phase-NN.{toml,md}) — GITIGNORED
doc/*       evergreen, authoritative specs. not yet structurally supported by doctrine
doc/memories/  the bargain-bin memory store (see above)
install/    sources copied to .doctrine by the installer; plugins/skills handled special
src/        rust code
```

## doctrine cli

```
doctrine slice new [TITLE] [--slug S] [-p ROOT]      # allocate next id, scaffold slice
doctrine slice design  <ID> [-p ROOT]                # scaffold design.md sibling
doctrine slice plan    <ID> [-p ROOT]                # scaffold plan.toml + plan.md
doctrine slice phases  <ID> [--prune] [-p ROOT]      # materialise phase tracking from plan.toml
doctrine slice phase   <ID> <PHASE-NN> --status <S> [--note N] [-p ROOT]
                                                     #   S = planned|in_progress|completed|blocked
doctrine slice notes   <ID> [-p ROOT]                # scaffold durable notes.md (on-demand)
doctrine slice list    [--status S] [-p ROOT]        # rows: id status slug title

doctrine memory record --type T [--key K] [--status S] [--summary S] [--tag T]…  # mint uid, scaffold
doctrine memory show   <UID|KEY> [-p ROOT]           # header + body-as-data
doctrine memory list   [--type T] [--status S] [--tag T] [-p ROOT]  # newest first; AND-filter
                                                     #   find / retrieve (scope query) = SL-007

doctrine adr new  [TITLE] [-p ROOT]                  # allocate next id, scaffold ADR
doctrine adr list [--status S] [-p ROOT]             # rows: id status slug title
doctrine adr status <ID> --status S [-p ROOT]        # edit-preserving lifecycle transition

doctrine install [-p ROOT]                           # install/refresh .doctrine from embedded sources
doctrine skills list                                 # available skills + install status
doctrine skills install …                            # install skills into agents
```

- **Root detection**: walks up for a marker (`.git`, `.jj`, `.project`, `Cargo.toml`);
  override with `-p/--path`.
- **Scaffolds refuse to clobber** — an existing target file is an error, not an
  overwrite. The writer is transactional (partial scaffolds roll back).
- **`--prune` is destructive** — removes tracking sheets whose plan phase is gone.

## core process

```
doctrine slice new      # define scope (edit slice-nnn.md)
doctrine slice design   # author design.md
```
design iteration — interview; present decisions, tradeoffs, alternatives; refine open
questions; **adversarially review** (the slice-002/003/004 rhythm — a second agent or
codex mcp); repeat until decisions lock.

```
doctrine slice plan     # plan implementation against the existing surface
doctrine slice phases   # materialise per-phase tracking sheets
```
plan each phase in detail just prior to execution (fill `state/.../phase-NN.md`),
flip status with `doctrine slice phase … --status in_progress`, implement, end each
phase green, `--status completed`.

```
/code-review & audit against the design  ->  audit.md   (hand-authored)
```
harvest durable risks/decisions/findings from the disposable phase sheets into
`notes.md` / `audit.md` at close-out.

## storage model (the storage rule)

Three tiers — know which one you're writing:
- **authored** (`*.toml` + `*.md` under `slice/nnn/`): committed, diffable, reviewed.
  Structured data in TOML; prose in MD; **never queried/derived data in prose.**
- **runtime state** (`.doctrine/state/`, the `phases` symlink, `handover.md`):
  GITIGNORED, disposable, `rm -rf`-able. Progress lives here, never in authored files.
- **derived**: regenerable indexes/caches — gitignored.

Phase ids (`PHASE-NN`) and criteria ids (`EN-/EX-/VT-`) are **immutable** — edits
append, never renumber or reuse.

## conventions

- **no code without an approved plan** (the gate).
- **frequent conventional commits**; scope with the slice id, e.g.
  `fix(SL-004): …`, `doc(SL-005): …`, `plan(SL-005): …`. Commit on `main`.
- **ask, don't infer.** correctness comes first and last.
- **pure/imperative split** (slices-spec § Architecture): no clock, rng, git, or disk
  in the pure layer — pass them in as inputs (the date/uid pattern). Impurity lives in
  the thin shell.
- **behaviour-preservation gate**: when changing shared machinery (the entity engine),
  the existing suites are the proof — they must stay green unchanged.
- **TDD** red/green/**refactor**; test behaviour, not trivial implementation.
- **lint as you go** — `cargo clippy` zero warnings; `just check` before every commit.
- **no parallel implementation** — ride existing seams; find duplication before writing.

## known CLI gaps (todo as the tooling surface expands)

- **no `slice audit` scaffold** — every other artifact has one; `audit.md` is hand-made.
- **no slice status rollup** — phase progress (`X/Y complete`) is invisible without
  reading the per-phase state tomls; `slice list` shows only the hand-edited
  `slice-nnn.toml` status, which is not derived from phase completion.
- **no slice lifecycle transition** — `slice-nnn.toml` `status` is hand-edited; no
  command moves a slice proposed→…→done or links it to phase state.
- **no standalone plan validation** — a malformed `plan.toml` only surfaces when
  `slice phases` parses it.
- **memory retrieval** — `record/show/list` shipped (SL-005, done); scope-aware
  `find`/`retrieve` (ranking + git staleness) is SL-007 (proposed). Read-by-id
  only until it lands.

## environment

nixos; bubblewrap jails (mounted into /workspace/*).
