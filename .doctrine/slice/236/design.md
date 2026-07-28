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

**Atomicity.** A `global = true` short flag collides with any surviving local
`-p`, so the deletion cannot be staged across commits. Phase boundaries must
respect this (§8, R1).

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
  help golden. **Resolution: phase-1 spike** — add the global, delete ~3
  declarations, diff one help golden. Measure before committing to the sweep.
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

**Why the scale objection to D3 does not hold.** The `--color` fencepost (SL-079
added a global flag, migrated handlers individually, missed one) does not
transfer: adding a global flag does not force call sites to consume it, so
omission is **silent**. D3 *removes* a field from 204 declarations, making every
missed read a **compile error**. The work list is enumerated by `rustc`; the
change is done when it compiles.

**A3 — rejected: rename `--path` to `--root`.** Scope creep; user-visible
breaking change across skills and docs. Separate slice if ever worthwhile.

## 8. Risks & Mitigations

| id | Risk | Mitigation |
|---|---|---|
| R1 | Deletion cannot be staged — partial application does not compile | Treat the sweep as one phase; phase plan must not split it |
| R2 | Golden churn volume unknown | OQ-1 spike as phase 1, before the sweep |
| R3 | Implementer adds the global but leaves locals in place | **VT-s** source scan |
| R4 | Implementer fixes only inline variants (= rejected A1) | **VT-a parameterised over a newtype verb** |
| R5 | Confinement silently weakened while tests go green | VT-b/VT-c/VT-d; anti-cheat clauses §9 |

## 9. Quality Engineering & Validation

VTs are specified by **the wrong implementation they kill**, on the premise that
any latitude left in a VT will be taken.

| id | Setup | Must assert |
|---|---|---|
| VT-a | write verb, `-p <markerless tempdir>`, CWD inside a marked fork; parameterised over `adr new` (newtype) **and** `link` (inline) | exit 0; entity created **at the tempdir** |
| VT-b | write verb, `-p <marked tree>`, CWD unmarked | non-zero; stderr has `signal: marker` **and** the verb. Red before, green after |
| VT-c | write verb, no `-p`, CWD inside marked fork | non-zero; `signal: marker` — regression guard |
| VT-d | `DOCTRINE_WORKER` set, `-p <markerless tempdir>` | refuses with the **env-leg/dual-cause** message, not the marker-leg one |
| VT-e | read verb, rootless CWD, no `-p` | exit 0 — laziness |
| VT-s | source scan of `src/`, riding `tests/architecture_layering.rs` precedent | exactly **one** `short = 'p'` declaration (in `main.rs`); **zero** long-only `path` args |

**VT-a's newtype clause is load-bearing.** Written against `link` alone (an
inline variant), the rejected A1 passes it — silently re-admitting the option
DEC-093 rejected.

**VT-d's message discrimination is load-bearing.** Asserting only "refused" lets
an implementation that made the env leg path-aware pass.

**Anti-cheat clauses, binding on the tests:**

1. Fixtures self-validate — VT-a/VT-c assert the fork marker *exists* before
   invoking; VT-b asserts its target is marked.
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
