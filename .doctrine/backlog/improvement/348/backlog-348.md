# IMP-348: Consolidate project-root CLI args behind a shared bundle

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

A prototype for an eventual cleanup slice. Scoped deliberately narrowly: this is
the part of SL-236 that survives its retirement, separated from the guard fix
that was driving it.

## What is actually wrong

`-p/--path` is declared **204 times across 27 files** (202 as
`#[arg(short = 'p', long)]`, plus 2 long-only in `commands/serve.rs:17` and
`commands/map.rs:13`). All bind `Option<PathBuf>`; all mean "project root".

Three distinct problems live in that repetition, and they are worth naming
separately because only the first two are duplication at all:

1. **The help text is one decision spelled four ways** — 116× "Explicit project
   root (default: auto-detect).", 37× "…(default: auto-detect from CWD).", 9×
   without the full stop, 4× "Explicit project root.". User-visible in `--help`,
   and genuinely duplicated: one sentence, one meaning, four renderings.
2. **The declaration triple is repeated verbatim** — `#[arg(short = 'p', long)]`
   plus its doc line plus `path: Option<PathBuf>`, five times in `src/adr.rs`
   alone. Syntactic repetition with an idiomatic remedy the codebase already
   owns and barely uses (below).
3. **Two real surface defects, embedded in it:**
   - `doctrine boot -p X install` **silently discards `X`**. `boot::dispatch`
     (`src/boot.rs:2209-2214`) destructures `Install { path: install_path, .. }`
     and forwards only the leaf's path; the parent `Boot.path` is dropped on the
     floor. A flag is accepted and does nothing — a footgun, not untidiness.
   - `doctrine serve` accepts `--path` but not `-p`, alone among root-taking
     verbs.

## The fix, and the shape to avoid

**Do:** a `#[derive(Args)]` project-root bundle, `#[command(flatten)]`-ed into
the verbs that take a root. That removes the syntactic repetition, normalises the
help wording in one place, and preserves the per-verb information. It rides an
existing seam rather than adding a parallel mechanism: ✓ `CommonShowArgs`
(`src/main.rs:192-207`) already carries exactly this `path` field and is used at
only **5 sites across 3 files** — the mechanism exists and is under-applied.

Fix the two defects directly while there: delete the parent `Boot.path` (or make
`dispatch` fall back to it), and add `short = 'p'` to `serve`.

**Do NOT** promote `-p` to a `global = true` argument on `Cli`. That was SL-236's
D3 / [[DEC-093]] and it is **rejected on evidence**, not taste:

- Each per-verb declaration is not repetition of a *decision* — it is the
  machine-checkable record that **this verb consumes a project root**. Making
  acceptance universal while consumption stays per-verb destroys that
  information.
- Concretely, RV-319 F-1: four guarded verbs are pathless unit variants
  (`Command::Onboard`, `WorktreeCommand::{CreateFork, Nominate, Denominate}`).
  Under a global they would accept a `-p` nothing consumes; `create-fork` in
  particular derives its root from the stdin payload cwd and never the process
  cwd, so the flag would steer its guard away from the tree it writes to. Today
  clap rejects `onboard -p X` outright — an invariant now pinned by
  `tests/arg_path_convention.rs` and
  `tests/e2e_worker_guard_explicit_root.rs::pathless_guarded_verbs_reject_explicit_root`.
- The completeness argument for the sweep was measured false (SL-236 design
  §10 F-8): a surviving local `-p` coexists with a global silently, so a
  half-swept tree compiles and passes every behavioural test.

## Prior art to read before designing this

`.doctrine/slice/236/design.md` — retired, but §7 (D3/A1/A2/A4 with the cost
table) and §10 (F-8, F-10, F-13) hold the measured evidence and should not be
re-derived. Note the STOP banner above §10: the *guard* premise is retracted, but
the CLI-surface analysis in §5.4/§6 stands. `RV-319` carries both findings.

## Not in scope

The worker-guard root-resolution question ([[ISS-028]]) — it is independent, and
bundling the two is what made SL-236 unlandable.
