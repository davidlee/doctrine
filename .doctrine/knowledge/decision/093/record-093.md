# DEC-093: Global project-root flag over scattered write-path checks

## Decision

`-p/--path` becomes a single `global = true` argument on the top-level `Cli`
struct (`src/main.rs`), mirroring the existing `--color` precedent. The ~202
per-subcommand `-p` declarations are deleted. `worker_guard` receives the
resolved explicit path and passes it to `root::find`, so worker confinement is
evaluated against the tree the command actually operates on.

Taken for SL-236 (worker-guard root-resolution skew, ISS-028 / ISS-267).

## Alternatives rejected

**Extend `write_class` to yield class + path.** Rejected on *capability*, not
taste. `worker_guard` runs on the outer `Command` enum before dispatch, and 35
of 54 `Command` variants carry no top-level `path` — including `Adr`, a newtype
variant whose `-p` lives on the inner `AdrCommand`. So this cannot fix
`e2e_adr_cli_golden`, one of the two tests ISS-028 was filed about.

**Push the marker check down into the write functions.** They already receive a
correct `root`, so no path-threading would be needed. Rejected because there is
no single write chokepoint — `src/entity.rs` alone exposes four write entry
points, and state / config / relation / observation writes are separate — so the
check would scatter across every write path. That converts SPEC-012 REQ-192's
compile-time exhaustiveness (*"a new verb is a compile error"*) into a discipline
concern, and puts confinement policy in the leaf layer ADR-001 keeps thin.

## Why the scale objection does not hold

The obvious counter is the `--color` fencepost: SL-079 added that global flag,
required migrating every handler individually, and missed one.

An earlier version of this record argued the precedent does not transfer because
removing a field makes every missed site a compile error, so "the compiler
enumerates the work list". **That was too strong** — corrected 2026-07-28 after
an empirical clap-4.6.1 spike (SL-236 design §10 F-8):

- Deleting a `path` field does make every *read* of it a compile error, so a
  half-finished deletion cannot ship broken code. The compiler prevents
  **breakage**. ✓
- But a *surviving* declaration breaks nothing: clap accepts a `global = true`
  `-p` alongside a subcommand's own `-p` — they share an arg id, and both fields
  receive the value. The compiler exerts **no pressure toward completeness**, and
  a partially-swept tree compiles and passes behavioural tests.

So the guarantees come from two different places: `rustc` ensures the sites you
touch are correct; a **source-scanning test (VT-s)** is the only thing ensuring
you touched them all.

This also retracts the atomicity constraint: because there is no collision, the
sweep **can be staged** module by module rather than landing in one commit.

The decision itself is unchanged — the change remains large but mechanical, and
the rejected push-down alternative remains small but permanently unguarded.

## Consequences

- Collapses ~202 duplicated declarations to one — the CLI-surface improvement
  stands independent of the bug.
- Unifies genuinely divergent surfaces: `Boot` today declares its own `path`
  documented as *"Used by the bare regenerate; `boot install` carries its own
  `-p`"*, so `doctrine boot -p X install` and `doctrine boot install -p X` are
  currently two different flags.
- **Stageable**, not atomic (corrected — see above). Intermediate states parse
  and behave correctly.
- Churns byte-exact `--help` goldens — but less than feared: measured, the `-p`
  line stays in place within each subcommand's `Options:` block; what changes is
  the **description text**, since help then prints the global's doc comment.
  Adopting the 116-occurrence majority wording confines churn to the ~40
  minority-worded subcommands.
- Preserves ADR-006 D2a's worker-mode formula, REQ-192 exhaustiveness, guard
  laziness, and the root-independence of the `DOCTRINE_WORKER` env leg.

## Evidence

`.doctrine/slice/236/research/research.md` (✓-marked rows). Related:
[[ISS-028]], [[ISS-267]], [[ADR-006]], [[ADR-008]], [[SPEC-012]].

## REJECTED 2026-07-29 — the premise, not just the option

Status flipped `proposed` → `rejected`. Not because the alternatives argued
below won, but because the axis this record chose *between* was itself invalid.

**RV-319 F-2.** Every candidate here — the global flag, and the `ArgMatches`
walker (A4) that later contested it — re-keys `worker_guard` to the tree named
by `-p`. That inverts confinement: the worker marker identifies the **actor**,
and every tree a worker must not write to (coordination tree, primary repo) is
**markerless**, while the only marked tree is the worker's own fork. Reproduced:
from a marked fork, `adr new -p <markerless coord tree>` succeeds where it is
otherwise refused. Three `e2e_dispatch_sync` tests already encode the actor
contract and go red.

**RV-319 F-1**, separately fatal to the global flag specifically: a per-verb
`-p` declaration is the machine-checkable record that the verb *consumes* a
project root. Globalising makes acceptance universal while consumption stays
per-verb, so four guarded pathless variants (`Command::Onboard`,
`WorktreeCommand::{CreateFork, Nominate, Denominate}`) would accept a root
nothing reads — `create-fork` derives its root from the stdin payload cwd, so
the flag would steer its guard away from the tree it writes to.

**Where the two halves went.** Guard behaviour returns to [[ISS-028]], rewritten
around a topological fix (clear the marker around a worker's manual test run, as
the commit gate already does). The CLI-surface cleanup — shared `#[derive(Args)]`
bundle, one help wording, and the two real defects (`boot -p X install` silently
discarding `X`; `serve` lacking a short `-p`) — carries forward as [[IMP-348]],
which explicitly rejects the global shape. [[SL-236]] is abandoned.

Durable lesson: [[mem.fact.dispatch.worker-confinement-is-actor-based]].
