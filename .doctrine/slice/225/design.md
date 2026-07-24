# SL-225 — Funnel false-red elimination — Design

> Cluster 1 / move B of RFC-016. Two false-reds — a green worker delta reported
> as a failure it cannot distinguish from its own damage. Both are **project-tier**
> dogfooding artifacts (POL-002): the fixes live in the `justfile`, governance, and
> this repo's test suite. The engine is touched only by a **single neutral signal
> line** in `worker_commit` (a `DOCTRINE_DISPATCH_GATE` marker on the gate spawn) —
> no cargo layout, no resolution policy, no governance logic. The one coverage
> residual the fork-skip leaves is closed **project-side** — a fresh-binary corpus
> gate at the orchestrator's close, entirely in this repo's `justfile` / governance /
> close ritual, zero engine surface, inert for client repos. See DEC-003.

## Problem

A dispatch worker iterates in an isolated fork and must reach a green local suite
before `worker_commit`. Two recurring reds are **not** the worker's delta:

- **#1 (ISS-218, 6+ repros).** The commit gate's `just validate` shells bare
  `doctrine` → PATH `~/.cargo/bin/doctrine`, built from edge/main and RO in the
  jail. When an earlier phase shipped a binary-level rule change (new role /
  allowlist / check) on `dispatch/<slice>`, the PATH binary doesn't know it →
  `doctrine doctor` false-reds a correct fork. Independent of the current phase's
  delta (recurs on a pure-JS phase — SL-206-P14).
- **#2 (CHR-044, 7+ repros).** ~30 e2e goldens spawn the CLI for authored writes
  (`backlog new`, `install`, …). In a worker fork the worker-mode guard correctly
  refuses those writes under the marker. The server-side gate already clears the
  marker for its own run (SL-199 F2, `worker_commit.rs:147`), so the residual burn
  is the **worker agent's own** `cargo test` / `just check` — run to check its
  work — hitting the marker and reading the refusal as own-delta red.

Neither is a real regression; both burn tokens on diagnosis of noise.

## Fix #1 — the worker gate does not re-run coord's governance self-checks

The commit gate is `check commit` → `DEFAULT_COMMIT = ["just","check"]`
(verify.rs:149) → `check: fmt lint lint-js validate test build` (justfile:17). Of
those legs, **only `validate` shells the installed `doctrine`** — the others use
cargo/node toolchains or build a fresh binary. So `validate` is the entire ISS-218
surface (justfile:28):
```
validate:
  doctrine prompt check      # → PATH doctrine (stale in a dispatch fork)
  doctrine doctor
```

### Why chasing a fork-consistent binary is the wrong frame (RV-292)

The prior two designs tried to make `validate` shell a *fork-consistent* binary —
first by publishing `current_exe()` (RV-291: redundant-or-harmful), then by
self-locating the coord build from git (RV-292 F-1: **impossible** — git's worktree
model is flat, so a worker fork and its coord tree share the *primary/edge* common
`.git`; `dirname $(git rev-parse --git-common-dir)` returns the edge root, and
`git.rs:557` already documents that `parent(--git-common-dir)` is no worktree-root
oracle). Both founder on the same rocks RV-292 named: the coord binary cannot be
*located* from a fork (F-1), is never *built* by dispatch (F-3), and — since
`validate` precedes `build` in the belt — no binary reflecting the current phase's
own delta *exists* at validate time (F-2).

The frame is wrong because **`doctrine prompt check` / `doctrine doctor` validate the
repo's *authored* `.doctrine/` state**, and a worker **cannot write `.doctrine/`**
(the worker-mode guard refuses it). So in any worker fork their input is
byte-identical to coord's — which coord already validated green — and the *only*
variable that can change their verdict is the binary version. They therefore carry
**zero worker-delta signal** in a fork; they can only manufacture the stale-binary
false-red. The gate's job is to validate the worker's *code delta*, not to re-run
coord's governance self-audit.

### The fix — skip the governance legs of `validate` in a worker context

```
validate:
  #!/usr/bin/env bash
  set -euo pipefail
  # In a dispatch worker fork the authored .doctrine/ state is coord's (the worker
  # cannot write it), so these governance self-checks add no worker-delta signal —
  # they can only false-red on a stale binary. Coord owns them. (SL-225 #1, DEC-003.)
  if [ -n "${DOCTRINE_DISPATCH_GATE:-}" ] || [ "${DOCTRINE_WORKER:-}" = "1" ] \
       || [ -f .doctrine/state/dispatch/worker ]; then
    echo "validate: skipping governance self-checks in a worker fork (coord owns them)"
    exit 0
  fi
  # Off the fork path (dev / CI / the orchestrator's close gate): validate the AUTHORED
  # corpus with the SOURCE-CONSISTENT binary — the fresh coord build when present, else
  # PATH. This is what closes the fork-skip's residual, at close, where coord IS built
  # (SL-225 #1 (ii)). Exact `= "1"` on DOCTRINE_WORKER matches env_worker_set()
  # (marker.rs:127) — a stray non-`1` value must not false-skip.
  doc="${DOCTRINE_BIN:-./target/debug/doctrine}"
  command -v "$doc" >/dev/null 2>&1 || doc=doctrine
  "$doc" prompt check
  "$doc" doctor
```

**The worker-context signal — three legs, the same predicate as fix #2's
`under_worker_marker()`:**

1. **`DOCTRINE_DISPATCH_GATE`** — the one engine touch. `worker_commit`'s
   `run_commit_gate` (worker_commit.rs:151-156) already spawns `just check` with
   `.current_dir(dir).env_remove("DOCTRINE_WORKER")`; it gains **one line** —
   `.env("DOCTRINE_DISPATCH_GATE", "1")`. Required because the gate *clears the
   marker and removes `DOCTRINE_WORKER`* for its run (so fix #2's goldens execute),
   so neither of the other two legs is visible in that exact window.
2. **`DOCTRINE_WORKER`** — the subprocess/pi arm's env leg (a worker's own
   `just check`).
3. **the marker file** (`.doctrine/state/dispatch/worker`) — the claude arm's
   worker doing a manual `just check` to inspect its work.

A generic host / CI sets none → `validate` runs `doctor` / `prompt check` (resolving
the fresh coord build when present, else PATH — a true generic host with neither
`DOCTRINE_BIN` nor `./target/debug/doctrine` falls to PATH `doctrine`, exactly as
today). This is the *same* env-or-marker predicate fix #2 uses, applied to the recipe
instead of the test helper — one concept for both fixes.

### What the one engine line is, and why it is POL-002-clean

`DOCTRINE_DISPATCH_GATE` is a **neutral signal** — "you are running inside the
`worker_commit` gate" — carrying no cargo layout, no path, no resolution or
governance policy. The engine merely *announces context*; the **project recipe**
decides to skip its own governance checks. A generic host's justfile would never
read the variable. So the platform/project boundary holds: the engine says *where*,
the project decides *what* (contrast the rejected designs, which tried to make the
engine or a git inference resolve *which binary* — a project/cargo concern).

### What this dissolves (RV-292 findings)

- **F-1 (locate coord)** — gone. No coord binary is located; there is none in the
  design.
- **F-2 (existence ≠ freshness; validate before build)** — gone. No `doctor` runs
  in the fork, so no fork-consistent-binary requirement exists.
- **F-4 (undeclared Git 2.31+ floor)** — gone. No git plumbing in the recipe.
- **F-3 (coord never built; governance still mandates the frozen rule)** —
  **retained and repurposed, not deleted.** The coord-build lifecycle *is* needed —
  not for the fork gate (which skips), but to close the fork-skip's one residual: the
  orchestrator's fresh-binary corpus gate at **close** (§ "Closing the residual"). So
  this slice keeps the `DOCTRINE_BIN`→coord-build precondition **single-source in
  `.doctrine/governance.md`** (STD-001) and **reframes** it — it is no longer a
  fork-side *launch-time* env contract (RV-291 F-1 killed that), but a coord-side
  *close-time* build the orchestrator performs on the tree it already owns. It reaches
  the agent-facing CLAUDE.md surface via the `@.doctrine/state/boot.md` inline of
  `governance.md`, refreshed by `doctrine boot` — **not** a separate CLAUDE.md file
  edit (reconciled RV-294 F-1: the earlier draft naming CLAUDE.md as a second edit
  target would have violated single-source).
  Establishing the coord build at close is the direct answer to F-3's "promised but
  never established" — established where it is finally tractable (coord *is* built
  there; no flat-worktree topology to defeat it).

### Closing the residual — the orchestrator's fresh-binary corpus gate (project-tier)

The fork-skip removes only false-reds, but it leaves **one** coverage question: a
phase that changes `doctrine doctor` / `prompt check`'s *own logic* (a Rust change to
what governance-consistency *means*) will not have that new rule exercised against the
real authored corpus *by a fresh binary in the fork* — and never was (the fork gate
always ran the *stale PATH* binary; that is ISS-218). The honest question RV-292's
spirit demands: **is that fresh-binary corpus check performed anywhere before the
slice closes?**

Traced, today the answer is *no*. The orchestrator funnel runs **no** corpus gate:
the claude-arm `dispatch_import` (dispatch.rs:293) runs only the pure classify belt
(`.doctrine/`/`.claude/` prefix-reject + scope leg — not consistency), composes the
tree, and lands one commit; the pi-arm import gate is `run_prove_on` = the `prove`
cadence = `fmt-check lint` only (import.rs, justfile:20); `dispatch_conclude_phase`
just flips the gitignored sheet + boundary commit. And `close`'s `doctrine check
gate` → `validate` shells **bare PATH** `doctrine` (justfile) — the stale installed
binary, not the landed one. So a landed governance-logic delta is exercised fresh
against the corpus *nowhere* until the installed binary is rebuilt.

**The fix (project-tier, zero engine).** The orchestrator *is* coord: it runs on the
coord/landing tree, the slice source has **landed** there, so a build there yields a
**source-consistent** binary — the fork-side blockers (flat git topology,
coord-never-built) do not apply. The one blocker that *does* reach close — the belt's
own **validate-before-build** ordering (RV-292 F-2's ghost: `gate` runs `validate` 4th,
`build` 6th, so a plain `check gate` validates against a *prior* build) — is neutralised
by reordering the belt, so freshness is **enforced by the gate itself, not by prose**:

1. `validate` resolves the **fresh coord build** (`${DOCTRINE_BIN:-./target/debug/doctrine}`,
   PATH fallback) instead of bare `doctrine`.
2. **Reorder the belt so `build` precedes `validate`** in `check`/`gate`
   (`fmt lint lint-js build validate test[-all]`). Then `doctrine check gate` *alone*
   guarantees a this-invocation-fresh `./target/debug/doctrine` before `validate` reads
   the landed corpus — single-source, enforced, harmless in-fork (validate still skips),
   and it also fixes the dev inner-loop's stale-binary `validate`. This is what *owns*
   the freshness lifecycle F-3 demanded — **in the belt, not a checklist step an agent
   can skip**. The `.doctrine/governance.md`/close-ritual build note is thereby
   downgraded to belt-and-suspenders documentation. (`quick` is left fast — it may
   validate against a prior build; a documented low-severity bound, not the close path.)

**POL-002 — why this must be, and is, project-only.** The residual exists *only*
because doctrine develops doctrine — a slice can mutate the governing binary's own
rules. A client repo governed *by* doctrine never rebuilds the doctrine binary, so its
installed `doctrine` is always current for its corpus and **the residual cannot occur
there**. A mechanism to close it therefore must not ship and must not assume
cargo/`target`. It doesn't: (ii) lives entirely in this repo's `justfile`,
`.doctrine/governance.md`, `CLAUDE.md`, and close ritual — **zero engine surface, zero
shipped-skill surface, inert for clients**. The retained `DOCTRINE_BIN`/coord-build
knowledge is *project operator* knowledge, which is exactly where cargo/`target`
awareness belongs under POL-002 (contrast the three rejected designs, which pushed it
into the engine or a git inference). The slice's whole engine touch remains the single
`DOCTRINE_DISPATCH_GATE` neutral-signal line for the fork skip.

## Fix #2 — marker-aware skip in the authored-write goldens

### Detection helper (project test-support, SL-162 pattern)

Integration tests are a separate crate — `pub(crate) marker_present` (marker.rs:118)
is unreachable and the fork-root path must be resolved test-side. Add to
`src/test_support.rs` (single source, `#[path]`-included by `tests/common/mod.rs`
alongside `doctrine_bin`/`repo_root`):

```rust
/// True when running inside a dispatch worker fork — the env leg (subprocess arm)
/// OR the marker file (claude arm, marker-file-only; case-note #6). Authored-write
/// e2e goldens skip on this so a worker's own `cargo test` reflects delta health,
/// not the worker-mode guard's (correct) refusals. The server-side commit gate
/// CLEARS the marker before its run (SL-199 F2), so the goldens still execute there
/// — coverage is preserved; only the worker's manual run skips.
pub(crate) fn under_worker_marker() -> bool {
    std::env::var_os("DOCTRINE_WORKER").is_some()
        || repo_root().join(WORKER_MARKER_REL).exists()
}
```

Each authored-write golden gains a guarded early-return:
```rust
if common::under_worker_marker() { return; }  // SL-225 #2: skip in a worker fork
```

### Which goldens

The authored-write spawners only (those the marker guard refuses), **enumerated
empirically at implementation** from a `DOCTRINE_WORKER=1` e2e sweep — the 27
marker-poisoned suites the sweep surfaced (pinned in the PHASE-02 phase sheet's
Findings). Read-only goldens are untouched.

> **Reconciled RV-294 F-2.** The illustrative names an earlier draft carried —
> `e2e_worker_guard.rs`, `e2e_dispatch_sync.rs`, `e2e_doctor_golden.rs` — are **not**
> targets: `e2e_worker_guard` *exercises* the guard (sets the signal per-child and
> asserts refusal, so it must stay unguarded), and the other two do not false-red
> under the worker signal. The plan's "enumerated at implementation" is authoritative;
> the empirical sweep is the source of truth.

## Code impact (design-target selectors)

| Path | Change | Fix |
|---|---|---|
| `justfile` | `validate` recipe: (a) **skip** `doctrine prompt check`/`doctor` under the worker-context signal (fork); (b) off the fork path, resolve the **fresh coord build** (`${DOCTRINE_BIN:-./target/debug/doctrine}`, PATH fallback) instead of bare `doctrine`. Plus (c) **reorder `check`/`gate` so `build` precedes `validate`** — belt-enforces close-gate freshness (kills the validate-before-build order) | #1 (ii) |
| `src/mcp_server/worker_commit.rs` | `run_commit_gate` spawns the gate with `.env("DOCTRINE_DISPATCH_GATE","1")` — the one neutral signal line; the slice's **only** engine touch | #1 |
| `.doctrine/governance.md` | § orchestration — **retain + reframe** the `DOCTRINE_BIN`→coord-build precondition (now a coord-side *close-time* build governing the fresh-binary corpus gate, not a fork-side launch contract); add the build-before-`check gate` beat to the close ritual | #1 (ii) |
| `CLAUDE.md` | *(no direct edit — reconciled RV-294 F-1)* — carries the reframed rule via the `@.doctrine/state/boot.md` inline of `.doctrine/governance.md` (the single authored source, STD-001); refreshed by `doctrine boot` | #1 (ii) |
| `src/test_support.rs` | `under_worker_marker()` + `WORKER_MARKER_REL` const | #2 |
| `tests/common/mod.rs` | re-export `under_worker_marker` | #2 |
| the 27 authored-write goldens enumerated empirically (`DOCTRINE_WORKER=1` sweep; PHASE-02 sheet) — **not** `e2e_worker_guard`/`_dispatch_sync`/`_doctor_golden` (reconciled RV-294 F-2) | marker-guard early-return in authored-write goldens | #2 |
| `tests/e2e_*` (new) | VT-1 discriminating gate proof + VT-1b generic-host no-mask + VT-1c signal-legs unit | #1 |

**Engine surface is one line** — `worker_commit.rs` gains a neutral env signal on
the gate spawn (a `design-target` selector for #1), with **no** change to gate
logic, staging, or the guard. The `justfile` `validate` edit (both the fork skip and
the fresh-binary resolution) is a project recipe `design-target`. `.doctrine/governance.md`
is an authored/prose edit (rule **retention/reframe** + the close ritual beat) —
reaching CLAUDE.md via the boot inline, not a distinct CLAUDE.md edit — not a worker
code-delta, so it is not a `design-target` selector.
**(ii) adds nothing to the engine or to any shipped skill** — it is entirely project-tier.

## Verification alignment

- **VT-1 (#1, discriminating end-to-end — dissolves ISS-218 through the real seam).**
  A fixture stands up a repo whose authored governance would `doctrine doctor`-**red**
  under the installed/stale binary (the ISS-218 shape — a rule the PATH binary lacks),
  forks a worker (non-Rust delta), and drives the **real `worker_commit` gate**. The
  gate goes **green** because `validate` sees `DOCTRINE_DISPATCH_GATE` and skips the
  governance legs — where the *same* fixture with the signal absent (generic-host path)
  would `commit-gate-red` on `doctor`. The green-vs-red pivot is the skip, gated on the
  worker signal; that is the discriminating proof ISS-218 is gone (RV-292 F-3).
- **VT-1b (#1, generic-host no-mask — the safety negative).** With **no** worker
  signal (no gate env, no `DOCTRINE_WORKER`, no marker), `just validate` runs
  `doctor`/`prompt check` normally and a genuinely broken authored state still
  **reds**. Proves the skip is strictly worker-gated and never masks a real
  governance regression on the main arm (the mirror of #2's no-mask property).
- **VT-1c (#1, signal-legs unit).** `validate` skips under each leg independently
  (`DOCTRINE_DISPATCH_GATE`, `DOCTRINE_WORKER=1`, marker file) and runs under none — a
  narrow unit over the predicate. Includes the **exact-match negative**:
  `DOCTRINE_WORKER=0` (or any non-`1`) does **not** skip, matching `env_worker_set()`
  (marker.rs:127) rather than a lax `-n` test.
- **VT-1d (#1 (ii), close-gate freshness — *enforced* closure).** Two legs:
  **(1) resolution** — off the fork path, `validate` resolves
  `${DOCTRINE_BIN:-./target/debug/doctrine}` (PATH fallback): `DOCTRINE_BIN` → a stub
  that **reds** the corpus ⇒ gate reds; → a stub that **greens** ⇒ gate greens.
  **(2) belt-order (discriminating)** — a fixture whose *landed* source changes a
  `doctor` rule such that a **fresh** binary reds the authored corpus while a **stale
  prior-build** binary greens it: `just gate` (now `build`→`validate`) **reds**, where
  the old validate-before-build order would **false-green**. The green→red pivot is the
  reorder — the proof (ii) *enforces* closure, not merely resolves a binary. Standalone
  `just validate`/`just quick` without a prior build is a documented low-severity bound
  (not the close path).
- **VT-2 (#2):** an authored-write golden returns early (skips) when
  `DOCTRINE_WORKER` is set or the marker file is present, and runs normally when
  neither is — proving marker-gated, never masking a real regression on the main arm.

**Engine VT (behaviour-preservation).** The `worker_commit` change is a single added
env var on the gate spawn; the existing `worker_commit` suites must stay green
unchanged (the gate's staging/commit/guard behaviour is untouched), and one test
asserts the gate spawn carries `DOCTRINE_DISPATCH_GATE`.

## Invariants & boundary conditions

- **POL-002.** No cargo/`./target` layout, path, or binary-resolution policy enters
  **engine** code. The engine's only touch is a **neutral context signal**
  (`DOCTRINE_DISPATCH_GATE`) — it announces *that* a gate run is underway; the
  **project recipe** decides *what* to skip. A generic host never reads the variable.
  The rejected designs violated this by making the engine (or a git inference)
  resolve *which binary* — a project/cargo concern. (DEC-003.)
- **(ii) is entirely project-tier.** The fresh-binary corpus gate lives in this repo's
  `justfile`, `.doctrine/governance.md`, `CLAUDE.md`, and close ritual — it ships to no
  client and is **inert** for them (a client never rebuilds the doctrine binary, so the
  residual cannot occur). The retained `DOCTRINE_BIN`/`./target` knowledge is *project
  operator* knowledge, which is exactly where cargo/`target` awareness belongs under
  POL-002. Zero engine surface; zero shipped-skill surface.
- **The skip carries no worker-delta signal, so it loses none — and its one residual is
  closed, not conceded.** `doctrine doctor` / `prompt check` read authored `.doctrine/`
  state, which a worker cannot write; in a fork their verdict equals coord's
  already-green verdict up to the binary version. Skipping them removes only the
  stale-binary false-red. The sole residual — a phase changing doctor's *own logic* —
  is closed by the orchestrator's fresh-binary corpus gate at **close** (project-tier),
  backstopped by the phase's own unit tests; it is **not** left to the stale `close`
  gate (which shells bare PATH `doctrine`) as an earlier draft wrongly claimed.
- **Close-gate freshness is belt-enforced, not prose.** `check`/`gate` run `build`
  before `validate`, so `doctrine check gate` validates the landed corpus with a
  this-invocation-fresh binary — the residual closure does **not** depend on an operator
  remembering a build-first step (the earlier draft's VA/prose beat, which `gate`'s own
  validate-before-build order silently defeated — Opus review, 2026-07-24).
- **Signal predicate matches the engine contract.** The skip tests `DOCTRINE_WORKER` by
  **exact** `= "1"` (== `env_worker_set()`, marker.rs:127), never `-n` — a stray
  non-`1` value cannot false-skip. `DOCTRINE_DISPATCH_GATE` is engine-set to `1`; the
  marker leg is presence-only.
- **Stale-marker bound (low).** A leftover `.doctrine/state/dispatch/worker` from a
  crashed dispatch would skip the governance legs on a later *manual* `just validate` in
  that tree. Bounded: the marker is gitignored runtime state, the skip **echoes** a
  visible line (not silent), and the property is shared with fix #2's existing
  `under_worker_marker()` predicate — this slice adds no new persistence risk.
- **Gate behaviour otherwise unchanged.** `worker_commit` still clears/restores the
  marker, removes `DOCTRINE_WORKER`, stages by path, and lands one commit; the added
  env var changes none of that. No `current_exe()` publish, no `DOCTRINE_BIN` write.
- **Coverage preserved (#2).** #2 skips only when marked; the gate clears the marker,
  so the goldens still run in the gate. Strictly marker-gated — the main (unmarked)
  arm always runs them, so a real authored-write regression cannot hide.
- **Generic-host / CI no-op.** A non-dogfooding host sets none of the three signal
  legs, so `validate` runs `doctor`/`prompt check` unchanged and never sets the #2
  marker either — both fixes are inert off the dispatch path.

## Design decisions & residual open questions

- **DEC-003** — the worker gate **does not re-run coord's governance self-checks**:
  `validate` skips `doctrine doctor` / `prompt check` under a worker-context signal
  (`DOCTRINE_DISPATCH_GATE` set by `worker_commit` | `DOCTRINE_WORKER` | marker),
  because a worker cannot write the `.doctrine/` state those checks read, so they add
  no worker-delta signal and can only stale-binary false-red. Rejected, across three
  passes: the engine-publish (`current_exe()` redundant-or-harmful, RV-291), the
  launch-frozen `DOCTRINE_BIN` precondition (not temporally achievable, RV-291 F-1),
  and the git-derived self-locating recipe (git's flat worktree model can't find the
  coord tree; coord binary never built; validate-before-build, RV-292 F-1/2/3). The
  engine surface is one neutral signal line. The fork-skip's one residual (a delta to
  `doctor`/`prompt check`'s *own logic*) is closed **project-side** by the
  orchestrator's fresh-binary corpus gate at **close** (§ "Closing the residual"), so
  the `DOCTRINE_BIN` governance rule is **retained and reframed** — a coord-side
  close-time build — not deleted (flips the earlier F-3 disposition, RV-292; codex
  external read, 2026-07-24).
- **OQ-1 (STD-001 single-sourcing).** `WORKER_MARKER_REL` (`.doctrine/state/dispatch/worker`)
  duplicates `marker.rs:114`'s `marker_path`. The dual-compilation seam (CHR-014)
  blocks a shared `crate::` const from `test_support.rs` (included into both the lib
  and the separate test crate). Resolve in `/plan`: either host the const in
  `test_support.rs` and have `marker.rs` reference *it*, or accept one documented
  carve-out with a cross-pointer comment (same class CHR-014 already tolerates).
