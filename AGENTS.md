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

.doctrine/adr/nnn/   adr-nnn.{toml,md} + nnn-slug symlink — project-global ADRs (authored)
.doctrine/state/slice/nnn/phases/   runtime phase tracking (phase-NN.{toml,md}) — GITIGNORED
doc/*       evergreen, authoritative specs. not yet structurally supported by doctrine
doc/memories/  the bargain-bin memory store (see above)
install/    sources copied to .doctrine by the installer; plugins/skills handled special
src/        rust code (incl. src/git.rs — the impure born-frame capture seam,
            forgettable's forget.remote.v1/forget.checkout.v1 reproduced byte-for-byte)
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
                       [--path-scope P]… [--glob G]… [--command C]… [--repo R]   #   scope + born git anchor
doctrine memory show   <UID|KEY> [-p ROOT]           # header + body-as-data (+ real anchor line)
doctrine memory verify <UID|KEY> [-p ROOT]           # attest vs the working tree (refuses a dirty tree)
doctrine memory list   [--type T] [--status S] [--tag T] [-p ROOT]  # newest first; AND-filter
doctrine memory find   [--path-scope P]… [--glob G]… [--command C]… [--tag T]… [--query Q]
                       [--type T] [--status S] [--include-draft] [-p ROOT]  # ranked rows; risk visible
doctrine memory retrieve <same query/filter flags> [--limit N] [--min-trust L] [-p ROOT]
                                                     #   framed data-not-instruction blocks; trust
                                                     #   holdback (low∧sev≥high suppressed, non-bypass)
                                                     #   scope+anchor capture, verify = SL-007;
                                                     #   find / retrieve (scope query) = SL-008 (done)

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
- **`skills install` (Claude) symlinks, never copies** (SL-010). It materialises a
  canonical `.doctrine/skills/<id>` tree (derived/gitignored, always overwritten
  from the embed) and links `.claude/skills/<id>` → it, so a re-install always
  refreshes — no `--force`. Ownership is by target equality: a link is doctrine's
  iff its value is our canonical target; anything else (a foreign symlink, or a
  real dir) is **kept + warned**, never clobbered. **Override hatch:** replace the
  managed symlink with a real copy (`rm` the link, `cp -rL`, then `git add -f` —
  `.claude` is gitignored) and install reports `kept …`, leaving it untouched.
  Non-Claude agents still delegate to `npx skills` (unchanged).

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
- **authored** (`*.toml` + `*.md` under `slice/nnn/` and `adr/nnn/`): committed,
  diffable, reviewed. Structured data in TOML; prose in MD; **never queried/derived
  data in prose.** ADRs are authored entities too — status lives in `adr-nnn.toml`.
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
- **memory retrieval — SHIPPED.** `record/show/list` (SL-005); SL-007 producer
  (`record` scope+git-anchor capture, `verify`, `src/git.rs` seam); SL-008 reader
  (scope-aware `find`/`retrieve`, 9-key ranking, git staleness, the `retrieve`
  trust holdback). No retrieval gap remains. Producer follow-up still open: `record`
  has no `--trust`/`--severity` flag (audit A-3) — the risk axes are TOML-only.

## environment

nixos; bubblewrap jails (mounted into /workspace/*).
