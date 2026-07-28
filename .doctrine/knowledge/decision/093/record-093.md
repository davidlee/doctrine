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

That precedent does not transfer, because the operations are opposite in kind.
Adding a global flag does not force any call site to consume it, so an omission
is **silent**. This change *removes* a field from ~202 variant declarations,
which makes every existing destructure a **compile error**. The compiler
enumerates the entire work list; a missed site cannot ship.

So the change is large but mechanical and machine-checked, whereas the rejected
push-down alternative is small but permanently unguarded.

## Consequences

- Collapses ~202 duplicated declarations to one — the CLI-surface improvement
  stands independent of the bug.
- Unifies genuinely divergent surfaces: `Boot` today declares its own `path`
  documented as *"Used by the bare regenerate; `boot install` carries its own
  `-p`"*, so `doctrine boot -p X install` and `doctrine boot install -p X` are
  currently two different flags.
- Necessarily **atomic**: a `global = true` short flag collides with any
  surviving local `-p`, so the removal cannot be staged across commits.
- Churns byte-exact `--help` goldens, since `-p` relocates in every subcommand's
  help output.
- Preserves ADR-006 D2a's worker-mode formula, REQ-192 exhaustiveness, guard
  laziness, and the root-independence of the `DOCTRINE_WORKER` env leg.

## Evidence

`.doctrine/slice/236/research/research.md` (✓-marked rows). Related:
[[ISS-028]], [[ISS-267]], [[ADR-006]], [[ADR-008]], [[SPEC-012]].
