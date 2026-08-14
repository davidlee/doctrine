# REQ-252: The orchestrator computes and emits a per-worktree env contract (`KEY=value` per line on stdout) whose consumers the project declares (e.g. `CARGO_TARGET_DIR`); emission is harness-identical, delivery is subprocess-only (codex/pi), never a framework primitive.

## Statement

> **AMENDED — NARROWED (SL-254, 2026-08-14).** The title line's
> "delivery is subprocess-only (codex/pi)" parenthetical is falsified in its
> *scope*, not its substance: there is no longer a non-subprocess arm to contrast
> with. Every harness — claude included — is spawned as a confined subprocess by
> `scripts/spawn-confined.sh <harness>`, so subprocess delivery is now the
> universal case rather than a codex/pi-only one. The title is retained
> unchanged; the statement below is what must hold.
>
> **Separately, and NOT SL-254's to repair:** the emission itself has been dead
> since SL-156, when the platform exited the build-env business —
> `src/worktree/fork.rs` emits no `KEY=value` lines at all and each worktree
> compiles into its own in-tree `<dir>/target`. That is pre-existing drift from
> another slice, recorded here for the orchestrator, deliberately not fixed
> under SL-254. See also REQ-248 point 4.

Where a project declares per-worktree environment consumers, the orchestrator
computes them and emits them as `KEY=value`, one per line, on stdout. The
emission is harness-identical — the same bytes for every harness — and delivery
is by ordinary process environment on the one subprocess spawn path, never a
framework primitive of any particular harness.

## Rationale

Keeping the contract as lines on stdout, rather than a harness-supplied
environment primitive, is what made it harness-identical in the first place: the
orchestrator owes every harness the same bytes and owes none of them a bespoke
delivery mechanism. Collapsing to one confined subprocess arm removes the
contrast the original parenthetical was drawing, so the clause now narrows to a
statement about *mechanism* (process environment) rather than about *which
harnesses get it*.
