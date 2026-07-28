# Design SL-236: Worker-guard honours explicit project root

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

## 1. Design Problem

`worker_guard` evaluates worker-mode confinement against the tree it finds by
walking up from **CWD**, not the tree the command is operating on. When a command
carries an explicit `-p <root>`, the guard still resolves from CWD, finds an
enclosing worktree's marker, and refuses a write that was never aimed at a marked
tree.

Two consequences, one visible and one not:

- **Visible:** inside a stamped worker fork, ✓ 51 of 85 `tests/e2e_*.rs` drive the
  CLI with `-p <tempdir>` without setting `current_dir` and are latently exposed.
  A dispatch worker therefore has no trustworthy local green signal.
- **Latent (§5.5, I-2):** the inverse case is a real confinement hole. A write
  aimed at a *marked* tree from an *unmarked* CWD is allowed today, because the
  guard inspects the wrong tree in that direction too.

Evidence: `.doctrine/slice/236/research/research.md` (✓ rows). Originating
records: ISS-028 (canonical), ISS-267 (test-side symptom), ISS-240 (closed
duplicate), ISS-260 (resolved predecessor).

## 2. Current State

```rust
// src/main.rs:236-242
let color = tty::resolve_color(cli.color);
guard::worker_guard(&cli.command)?;            // no path in scope
let Cli { command, .. } = cli;
cli::dispatch(command, color)

// src/commands/guard.rs (fn opens :473)
pub(crate) fn worker_guard(cmd: &Command) -> anyhow::Result<()> {
    let verb = match write_class(cmd) { … };                          // Read ⇒ early return
    let Ok(root) = root::find(None, &default_markers()) else { … };   // ← CWD, always
    let mode = worktree::resolve_mode(&root);
```

✓ `Cli` (`src/main.rs:107-114`) has exactly two fields: `color` (`global = true`)
and `command`. There is no `-p` on `Cli`.

✓ `-p` is declared **204 times across 27 files** — 202 as `#[arg(short = 'p',
long)]`, plus 2 long-only (`src/commands/serve.rs:17`, `src/commands/map.rs:13`).
All bind an `Option<PathBuf>`; none carries a `default_value`; ✓ all mean
"project root". ✓ No `-p` in the codebase binds a non-path field.

✓ `guard.rs` is the sole *unintentional* `root::find(None, …)`. The other two
(`src/commands/cli.rs`, `src/state.rs:692`) carry comments marking them
deliberate.

## 3. Forces & Constraints

| Constraint | Source | Bite |
|---|---|---|
| Worker-mode formula `(is_linked_worktree && marker_present) OR env` | ADR-006 D2a | Unchanged — this slice fixes *which tree*, not the test |
| `write_class` exhaustive — "a new verb is a compile error" | SPEC-012 REQ-192 | **Compile-time** guarantee; must not degrade to discipline |
| Guard laziness — only a Write verb resolves the root | `guard.rs:473-474` | A Read verb in a rootless CWD must gain no failure path |
| Env leg is root-independent | ADR-006 D2a | Making the marker leg path-aware must not infect the env leg |
| Behaviour preservation on shared machinery | AGENTS.md | Existing suites stay green unchanged, help goldens excepted |
| Thin impure shell; leaf layer stays pure | ADR-001, POL-002 | Confinement policy does not belong in leaf write helpers |

Governance is **not** a discriminator between the candidate fixes — Thread 1
found all permitted, requiring no Revision or ADR amendment. The decision is
engineering.

## 4. Guiding Principles

- Ask the classifier only what it can answer. The path problem is a CLI-surface
  problem, not a classification problem.
- Prefer a large **compiler-enumerated** change over a small **unguarded** one.
- Delete duplication rather than route around it.
- Write verification that binds the implementer, not verification that reports.

## 5. Proposed Design

### 5.1 System Model

`-p/--path` becomes a single `global = true` argument on `Cli`, mirroring the
existing `--color` precedent (`src/main.rs:109`). The 204 per-subcommand
declarations are deleted. The parsed path is handed to the guard and threaded
through dispatch to the leaves.

```
clap parse ─► Cli { color, path, command }
                      │      │
                      │      ├─► worker_guard(&command, path.as_deref())
                      │      │      └─► root::find(Some(path)) ─► resolve_mode(root)
                      │      └─► dispatch(command, color, path)
                      │                └─► <kind>::dispatch(cmd, color, path)
                      │                       └─► leaf run(path, …)   ← unchanged
```

### 5.2 Interfaces & Contracts

```rust
// src/main.rs
struct Cli {
    #[arg(long, default_value = "auto", global = true)]
    color: clap::ColorChoice,

    /// Explicit project root (default: auto-detect from CWD).
    #[arg(short = 'p', long, global = true)]
    path: Option<PathBuf>,

    #[command(subcommand)]
    command: crate::commands::cli::Command,
}

// src/commands/guard.rs
pub(crate) fn worker_guard(cmd: &Command, explicit: Option<&Path>) -> anyhow::Result<()>
```

**D1 — `root::find`'s signature does not change.** It stays
`find(Option<PathBuf>, &[String])`. It has ~150 call sites; widening to
`Option<&Path>` would ripple for no gain. The guard converts at its one call site.

**D2 — `write_class` is not touched.** It keeps its exhaustive match and
`WriteClass` return type. REQ-192 holds *by construction*, because the classifier
is never made responsible for paths.

### 5.3 Data, State & Ownership

The path is owned by `Cli`, borrowed by the guard, moved into `dispatch`. No new
shared or global state — the rejected push-down alternative (§7, A2) would have
required carrying the write-class decision into the leaf layer.

Leaf `run()` functions are **unchanged**: they already take
`path: Option<PathBuf>`. Only the routing between `main` and the leaves changes.

### 5.4 Lifecycle, Operations & Dynamics

Sub-dispatchers gaining one parameter: `adr`, `policy`, `standard`, `rfc`,
`spec`, `slice`, `memory`, `review`, `rec`, `revision`, `coverage`, `backlog`,
`knowledge`, `tag`, `worktree`, `dispatch`, `concept_map`, `relation`,
`reservation`, `observation`, `compare`, `facet`, `check`, `prompt`, `serve`,
`map`, `search`.

Two `#[derive(Args)]` structs lose a field: `ServeArgs`, `MapServeArgs`.

**Atomicity — RETRACTED (F-8).** An earlier draft asserted that a `global = true`
short flag collides with any surviving local `-p`, forcing one atomic commit.
✗ **Measured false** (clap 4.6.1 spike, §10 F-8): global and local share an arg
id, clap accepts the duplicate, and both fields receive the value. **The sweep
can be staged** module by module; each intermediate state parses and behaves
correctly. Phase boundaries are free.

**D4 — the global's doc comment is `/// Explicit project root (default:
auto-detect).`** — the 116-occurrence majority wording among the 204 sites. Help
goldens print the *global's* text once the local is deleted, so matching the
majority minimises churn to the ~40 minority-worded subcommands (§6, OQ-1).

**Surfaces unified as a side effect.** ✓ `Boot` declares `path` documented as
*"Used by the bare regenerate; `boot install` carries its own `-p`"*
(`src/commands/cli.rs:553`, `src/boot.rs:2166`) — so `doctrine boot -p X install`
and `doctrine boot install -p X` are two different flags today. ✓ `doctrine serve`
accepts `--path` but not `-p`. The global collapses both inconsistencies.

### 5.5 Invariants, Assumptions & Edge Cases

- **I-1** ADR-006 D2a formula unchanged; `resolve_mode` receives a correct root.
- **I-2** Confinement changes in **both** directions, both correct:
  - *looser* — `-p <markerless tree>` from a marked CWD now succeeds (the bug);
  - *stricter* — `-p <marked tree>` from an unmarked CWD now refuses (closes a
    live hole). Verified as VT-b, not left as a side effect.
- **I-3** Laziness preserved: `root::find` still called only after a Write-classed
  match.
- **I-4** Env leg stays root-independent (VT-d).
- **A-1** Marker semantics are correct; only tree selection was wrong. Supported
  by ADR-006 D2a's SL-064 amendment retiring location-pinning.
- **Edge** `worktree fork`'s `-p` is documented as the *source* project root
  (`src/worktree/mod.rs:144`) — still "the root this command operates on", so the
  global serves it.

## 6. Open Questions & Unknowns

- **OQ-1 — golden churn volume is unmeasured.** clap renders `global = true` args
  in each subcommand's help, so `-p` may appear identically, at a different
  position, or move to a global section. Churn could be near-zero or touch every
  help golden.
  **Largely RESOLVED by the F-8 spike.** Measured on clap 4.6.1: a global `-p`
  renders inside each subcommand's `Options:` block at effectively the same
  position a local occupied. The line itself does not move. What changes is the
  **description text**, because the help now prints the *global's* doc comment.
  ✓ The 204 sites today carry ~4 wordings (116× "Explicit project root (default:
  auto-detect).", 37× "…(default: auto-detect from CWD).", 9×, 4×).
  **Mitigation → D4:** adopt the 116-occurrence majority as the global's doc
  string, reducing churn to the ~40 minority-worded subcommands plus any ordering
  shift where `-p` was not the last flag before `-h`.
  *Residual:* exact golden count still unmeasured, but it is now a bounded
  tidy-up rather than an unknown. Since F-8 also frees phase boundaries, goldens
  can be absorbed per module as each is swept.
- **Q1 — CLOSED, no code change.** "Is `under_worker_marker()` right for
  tempdir-rooted tests, given ✓ it checks `repo_root()`
  (`src/test_support.rs:85-87`)?" The sequencing dissolves it: the ✓ 19
  tempdir-rooted files stop needing the helper once the guard is fixed, and the ✓
  10 residual files operate on the real tree, for which `repo_root()` is exactly
  right.

## 7. Decisions, Rationale & Alternatives

**D3 — global `-p` on `Cli`.** Recorded as [[DEC-093]].

**A1 — rejected: extend `write_class` to yield class + path.** Rejected on
*capability*. The guard runs on the outer `Command` before dispatch, and ✓ 35 of
54 variants carry no top-level `path` — including ✓ `Command::Adr`
(`src/commands/cli.rs:462-465`), a newtype whose `-p` lives on `AdrCommand`
(`src/adr.rs:42,52,70,85,108`). So it cannot fix `e2e_adr_cli_golden`, ✓ one of
the two tests ISS-028 was filed on.

**A2 — rejected: push the marker check into the write functions.** They already
receive a correct `root`, needing no path-threading. Rejected because there is no
single write chokepoint — `src/entity.rs` alone exposes four write entry points
(`materialise`, `materialise_fresh_prebuilt`, `materialise_named`, `write_body`),
and state / config / relation / observation writes are separate. The check would
scatter across every write path, converting REQ-192's compile-time exhaustiveness
into a discipline concern and placing confinement policy in the leaf layer
ADR-001 keeps thin.

**A4 — NOT rejected; CONTESTS D3. Read the explicit path from `ArgMatches` by arg
id, sweeping nothing.** ✓ All 204 declarations share the clap arg id `path`, and
`ArgMatches` is not depth-limited: walking to the deepest subcommand's matches
and reading `try_get_one::<PathBuf>("path")` reaches every shape. Measured on the
pinned clap 4.6.1 spike (§10 F-13) against doctrine's real forms — inline variant,
newtype (`Adr` → `AdrCommand::New`, **the case that rejected A1**), 3-level
nesting, and the flattened `ServeArgs` bundle — all four resolved; the derived
`Cli` still parses from the same matches.

A4 is **not** A2: the check stays at the single existing chokepoint in
`worker_guard`, so REQ-192's compile-time exhaustiveness and ADR-001's thin leaf
are both untouched. It is not blocked by what blocked A1, because it never routes
through the typed enum. Footprint: one ~10-line helper plus `Cli::try_parse()` →
`Cli::command().try_get_matches()` + `from_arg_matches` in `main.rs`.

**A4's walker rule is LEAF-ONLY, not deepest-non-`None` (RV-319 F-1).** Descend to
the *selected* leaf's matches and read only that level's `path`. The first draft
(keep the last `Some` seen along the chain) fails F-1's own principle: ✓
`src/boot.rs:2209-2214` destructures `Install { path: install_path, .. }` and calls
`run_install(install_path, …)`, **dropping the parent `Boot.path`** — so
`doctrine boot -p X install` consumes nothing, yet the draft rule would hand `X`
to the guard. Measured, clap 4.6.1:

| argv | consumed by handler | deepest-non-`None` | leaf-only |
|---|---|---|---|
| `boot -p PARENT install` | *nothing* (CWD) | ✗ `PARENT` | ✓ `None` |
| `boot install -p LEAF` | `LEAF` | ✓ | ✓ |
| `boot -p BARE` | `BARE` | ✓ | ✓ |
| `adr new T -p X`, `collide -p X`, `serve --path X` | `X` | ✓ | ✓ |

**And the pathless class is refused at parse time.** ✓ `onboard -p UNMARKED`
returns `UnknownArgument` under A4 — a guarded verb that declares no `path`
cannot have `-p` *supplied*, let alone honoured. Under D3 the same argv parses
and steers the guard (F-1). A4 is therefore not merely un-steerable here; the
bypass is unrepresentable.

*Cost, stated against D3:*

| | D3 (global flag) | A4 (matches walker) |
|---|---|---|
| the invariant | **structural** — one declaration, compiler-visible | **conventional** — every `-p` must stay named `path`, typed `Option<PathBuf>`; nothing enforces it |
| footprint | 204 deletions / 27 files | ~10 lines, 1 call site |
| R3 (half-swept tree) | primary risk; VT-s is the sole guard | dissolved — nothing to sweep |
| OQ-1 golden churn | ~40+ goldens, bounded but real | zero — help output unchanged |
| surface unification (`boot -p X install`, `serve` sans `-p`) | fixed | **not** fixed |
| cohesion | one parse surface | two (string-keyed matches beside the derive) — precedent: ✓ `main.rs:217` `subcommand_help_path` already walks the clap tree by string (SL-208) |

A4's residual failure mode is the *same class* as R3, relocated: a future
subcommand naming its field `project_root` is silently unguarded. It is testable
by a convention scan far smaller than VT-s. **VT-a…VT-d apply unchanged under
either option** — they bind guard behaviour, not the sweep. Only VT-s is
D3-specific.

**Scope observation (raised with the decision).** This slice is *"Worker-guard
honours explicit project root."* D3 bundles a CLI-surface refactor into a guard
bug fix; the refactor is therefore never judged on its own merits. A4 fixes the
defect proportionately and leaves surface unification to stand or fall as its own
change. See §10 F-13 disposition.

**Why the scale objection to D3 does not hold — restated after F-8.** The
original argument ("the compiler enumerates the work list") was **too strong**
and is corrected here:

- Deleting a `path` field makes every *read* of it a compile error, so a
  half-finished deletion cannot ship broken code. The compiler prevents
  **breakage**. ✓
- But ✗ *not* deleting a declaration breaks nothing: F-8 shows a surviving local
  coexists happily with the global. The compiler exerts **no pressure toward
  completeness**.

So the safety and the completeness guarantees come from different places:
`rustc` guarantees the sites you touch are correct; **VT-s** (§9) is the only
thing guaranteeing you touched them all. This makes VT-s load-bearing rather than
a nicety — without it, a 60%-swept tree is green.

The `--color` fencepost (SL-079) remains instructive but not analogous: there,
omission produced a *wrong-behaving* handler; here, omission produces a
*correct-behaving but undeleted* declaration. The failure mode is untidiness plus
a still-blind guard for that verb — which VT-a catches only for the verbs it
exercises, hence VT-s scanning all of them.

**A3 — rejected: rename `--path` to `--root`.** Scope creep; user-visible
breaking change across skills and docs. Separate slice if ever worthwhile.

## 8. Risks & Mitigations

| id | Risk | Mitigation |
|---|---|---|
| R1 | ~~Deletion cannot be staged~~ — **RETRACTED**, F-8 measured no collision | Phase boundaries are free; stage per module |
| R2 | Golden churn volume | Largely resolved (OQ-1); D4 minimises it; absorb per module |
| R3 | Implementer adds the global but leaves locals in place — **now the primary risk**, since a half-swept tree compiles *and* passes behavioural tests (F-8) | **VT-s** source scan — the sole completeness guarantee |
| R4 | Implementer fixes only inline variants (= rejected A1) | **VT-a parameterised over a newtype verb** |
| R5 | Confinement silently weakened while tests go green | VT-b/VT-c/VT-d; anti-cheat clauses §9 |
| R6 | ~~Nested global propagation may not work~~ — **RESOLVED**, confirmed to 3 levels on the shipped binary (F-9) | none needed |

## 9. Quality Engineering & Validation

VTs are specified by **the wrong implementation they kill**, on the premise that
any latitude left in a VT will be taken.

**Fixture requirement (F-2/F-3, load-bearing).** ✓ `resolve_mode`
(`src/worktree/marker.rs:161-166`) computes `marker = is_linked &&
marker_present(root)`. A tempdir carrying a marker file is **not** refused — the
root must be a genuine **linked git worktree**. Every "marked tree" below means a
real linked worktree with a marker, built by reusing the existing machinery in
`tests/e2e_worker_guard.rs` (`init_repo` + its VT-1 linked-worktree helper), not
a hand-rolled tempdir. A bare-tempdir fixture makes VT-a and VT-b pass
**trivially, both before and after the change**, proving nothing.

| id | Setup | Must assert |
|---|---|---|
| VT-a | write verb, `-p <markerless tempdir>`, CWD inside a **linked, marked** fork; parameterised over `adr new` (newtype) **and** `link` (inline) | exit 0; entity created **at the tempdir** |
| VT-b | write verb, `-p <linked, marked tree>`, CWD unmarked | non-zero; stderr has `signal: marker` **and** the verb. Red before, green after |
| VT-c | write verb, no `-p`, CWD inside marked fork | non-zero; `signal: marker` — regression guard |
| VT-d | `DOCTRINE_WORKER` set, `-p <markerless tempdir>` | refuses with the **env-leg/dual-cause** message, not the marker-leg one |
| VT-e | read verb, rootless CWD, no `-p` | exit 0 — laziness |
| VT-s | source scan of `src/`, riding `tests/architecture_layering.rs` precedent | exactly **one** `short = 'p'` declaration (in `main.rs`); **zero** long-only `path` args |

**VT-s is the sole completeness guarantee (F-8).** Because a global and a
surviving local `-p` coexist without error, a tree where only half the 204
declarations were deleted **compiles and passes every behavioural VT**. Nothing
else in the suite notices. VT-s must therefore run on every phase, not only at
the end, and its failure message should name the offending files.

**VT-a's newtype clause is load-bearing.** Written against `link` alone (an
inline variant), the rejected A1 passes it — silently re-admitting the option
DEC-093 rejected.

**VT-d's message discrimination is load-bearing.** Asserting only "refused" lets
an implementation that made the env leg path-aware pass.

**Anti-cheat clauses, binding on the tests:**

1. Fixtures self-validate — VT-a/VT-c assert the fork marker *exists* **and that
   the tree is a linked worktree** before invoking; VT-b asserts the same of its
   target. Asserting marker-file presence alone is insufficient (F-2).
2. No `current_dir` in VT-a/b/c — CWD skew is the condition under test.
3. No bare `assert!(status.success())`; every refusal asserts the specific signal,
   every success an observable effect.
4. No `#[ignore]`, no marker-skip — these must execute inside a worker fork.
5. `WORKER_MARKER_REL`, never a literal path (STD-001).

**Regression posture.** Existing suites stay green unchanged; help goldens are the
sole sanctioned churn. A non-golden test needing an edit signals a design fault,
not a chore.

**Closure evidence.** `e2e_adr_cli_golden` and `e2e_relation_migration_storage`
pass **inside a stamped worker fork**, not merely on the coordination tree.

**Residual (ISS-267).** After the guard fix: re-measure the ✓ 29
`env_remove("DOCTRINE_WORKER")` files; the ✓ 19 that pass `-p` should go green
unaided; sweep the ✓ 10 that genuinely run against a marked tree onto
`under_worker_marker()`. Re-measure — do not assume the split holds.

## 10. Review Notes

Internal adversarial pass, 2026-07-28. Findings integrated above; recorded here
with disposition.

**F-1 — SUPERSEDED BY F-8.** The finding claimed OQ-1's pre-sweep spike was
unworkable because a global and a local `-p` cannot coexist. F-8 measured that
premise false, so both F-1 and the R1 constraint it rested on are withdrawn.
Retained here because F-1's *conclusion* (sweep, then measure) was reached from a
false premise and must not be cited as standing reasoning.

**F-8 — R1 (atomicity) is false; the compile-error argument was too strong
(critical, integrated §5.4/§6/§7/§8).** Settled empirically with an isolated
clap-4.6.1 spike rather than left as an obligation.

Measured, on the pinned clap version:

- `Cli::command().debug_assert()` **accepts** a `global = true` `-p` alongside a
  subcommand's own `-p`. No panic, no error.
- `spike collide -p LOCAL` parses, and the value lands in **both** the global and
  the local field — they share an arg id, so clap treats them as one argument.

Three consequences:

1. **R1 is retracted.** The sweep can be staged module by module; every
   intermediate state parses and behaves correctly. Phase boundaries are free —
   materially better for a dispatched worker.
2. **The §7 scale argument is corrected.** `rustc` guarantees the sites you
   *touch* are correct, but exerts no pressure to touch them all: a
   partially-swept tree compiles and passes behavioural tests. Completeness now
   rests solely on **VT-s**, which is promoted from tidiness check to primary
   guarantee (R3).
3. **OQ-1 is largely answered** — see §6 and D4.

*Method note:* run in an isolated scratch project, not by temporarily editing
`src/main.rs`, because the primary worktree is shared with other agents
(AGENTS.md) and a transient edit there could collide with concurrent work.

**F-9 — R6 (nested propagation) resolved at zero cost.** ✓ `--color` is already
`global = true` on `Cli`, so the shipped binary is the experiment:
`doctrine adr list --color never` (2 levels) and
`doctrine slice selector list 236 --color never` (3 levels) both parse. Globals
propagate to at least the depth this design needs. No spike required — the
question was answerable from existing behaviour and should have been asked
before it was written down as a risk.

**F-2 — VT-b's fixture would have tested nothing (major, integrated §9).** ✓
`resolve_mode` requires `is_linked && marker_present`. A tempdir with a marker
file is never refused, so VT-b as originally written would pass after the change
for the wrong reason — the tree isn't linked — and the confinement tightening
(I-2), the whole justification for calling this a hole worth closing, would go
unverified.

**F-3 — VT-a had the same latent defect (major, integrated §9).** Its "marked
fork" CWD must likewise be a genuine linked worktree; otherwise VT-a passes both
before and after and provides no signal. Anti-cheat clause 1 has been
strengthened to assert linkage, not merely marker presence.

**F-4 — `worktree fork`'s guard semantics shift, unexamined (moderate, accepted).**
✓ Its `-p` is documented as the *source* project root
(`src/worktree/mod.rs:144`). Post-globalisation the guard evaluates the marker of
that source tree. This is probably correct — forking writes into the source
tree's state — but it is a semantic change for one verb that no VT covers.
*Disposition:* accepted as a conscious assumption; named here so it is not
mistaken for an oversight. Worth a VT if the phase plan has room.

**F-5 — VT-s wording (minor, noted).** "exactly one `short = 'p'`" must be
specified as counting clap **attribute declarations** under `src/`, not raw
string occurrences, or a doc comment or test fixture mentioning `-p` will break
it.

**F-6 — surface expansion (minor, accepted).** Unit variants that never had a
path (e.g. `Onboard`) will accept `-p` after globalisation. Harmless; a global
flag is accepted everywhere by construction.

**F-7 — nested propagation unverified (minor, verification obligation).** clap
globals are expected to propagate to *nested* subcommands (`Command::Adr` →
`AdrCommand::New`, two levels deep). Expected to hold, unconfirmed by run. Phase
1 should assert it on a real nested verb before the sweep proceeds — a failure
here would invalidate D3 entirely.

**F-10 — a help-output scan cannot substitute for VT-s; D4 is what blinds it
(major, integrated §9 by reaffirmation).** Proposed alternative: enumerate every
command's `--help` and verify the `-p` migration from the rendered surface,
standing in for the source scan. Settled empirically on the pinned clap 4.6.1
spike (three unswept flavours vs. two swept).

Measured — the `-p` help line, swept vs. unswept:

| subcommand state | rendered `-p` line | detectable? |
|---|---|---|
| unswept, local doc **differs** from global's | `-p, --path <PATH>  Path to the source project root…` | yes |
| unswept, local `-p` has **no** doc comment | `-p, --path <PATH>` (no description) | yes |
| unswept, local doc **matches** global's | `-p, --path <PATH>  Explicit project root (default: auto-detect)` | **no — byte-identical** |
| swept (inherits global), 1 level | `-p, --path <PATH>  Explicit project root (default: auto-detect)` | baseline |
| swept (inherits global), 2 levels | identical to above | baseline |

A surviving local arg **shadows** the propagated global rather than duplicating
it, so help renders exactly one `-p` line either way; only its *description* can
betray the leftover. That makes detection a function of doc-comment text — and
**D4 sets the global's text to the 116× majority precisely to minimise golden
churn**. Re-measured on `src/`: of the 202 `short = 'p'` declarations, **116
already carry D4's exact text**. A help-based check is therefore structurally
blind to ~57% of the sweep, and blind *by construction of D4* — the two ideas are
in direct tension. Worse, its blind spot is silent: it reports green on a
half-swept tree, the exact F-8 failure mode VT-s exists to catch.

*Disposition:* rejected as a VT-s substitute. **VT-s stays the sole completeness
guarantee.** The instrument is retained for a different job — see F-11.

**F-11 — help enumeration is the right instrument for OQ-1, not for completeness
(minor, opportunity).** The same enumeration that fails as a completeness check
is well-suited to measuring **golden churn**: dump every command's `--help`
before and after the sweep and diff. That answers OQ-1 (still unmeasured) with a
number rather than an estimate, and gives the phase plan a churn budget. It rides
existing machinery — ✓ `tests/e2e_help_families_golden.rs` and
✓ `tests/e2e_subcommand_help.rs` already spawn the binary and assert on help
output. Not a VT; a planning instrument.

**F-12 — VT-s is cheaper than F-5 implied (minor, closes F-5's concern).** ✓ `syn`
is already a dev-dependency (`Cargo.toml:88`, workspace `syn = { version = "2",
features = ["full", "visit"] }`), and ✓ `tests/architecture_layering.rs` already
parses `src/` with it (24 `syn::` uses). F-5's hazard — counting raw `-p` string
occurrences and tripping over a doc comment or fixture — dissolves at an AST
visitor over `#[arg(…)]` attributes, which is the precedent's existing shape.
VT-s needs no new dependency and no new technique.

**F-13 — DEC-093's dilemma is not exhaustive; A4 is small AND centrally guarded
(critical, integrated §7 as A4; DECISION RE-OPENED).** DEC-093 closes on the
framing *"the change remains large but mechanical, and the rejected push-down
alternative remains small but permanently unguarded"* — large-and-guarded vs.
small-and-unguarded. That dichotomy holds only if A1 and A2 exhaust the space.
They do not.

A1's rejection is correct **as stated** — the guard cannot reach a newtype
variant's path *through the typed enum*. But it is a rejection of one access
route, not of early path access. Measured on the clap 4.6.1 spike:

| probe | modelled on | walker resolved |
|---|---|---|
| `collide -p INLINE` | `Command::Link` | `INLINE` |
| `adr new Title -p NEWTYPE` | `Command::Adr` → `AdrCommand::New` — A1's blocker | `NEWTYPE` |
| `slice selector list 236 -p DEEP` | 3-level nesting | `DEEP` |
| `serve --path FLATTENED` | `ServeArgs` flattened bundle | `FLATTENED` |
| `swept --marker`, `collide` (no `-p`) | negative controls | `None` |

`Cli::from_arg_matches` succeeded on every one, so the walker composes with the
derive rather than replacing it.

Compounding the finding: D3's original justification (*"the compiler enumerates
the work list"*) was measured false at F-8 and retracted, and nothing replaced it
but VT-s — a test needed **because** the compiler does not help. The decision now
rests on weaker ground than when it was taken, and was not re-opened at F-8.

*Disposition:* **not settled here.** A4 is recorded as a live contender, not a
replacement; the structural-vs-conventional invariant trade (§7 table) is a real
judgement call with a defensible answer either way. Routed to external
adversarial review before any implementation.

**Not found.** No governance conflict surfaced; Thread 1's applicability sweep
holds. No ADR/policy/standard was misread or weakly applied on re-check.
