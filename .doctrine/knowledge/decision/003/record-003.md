# DEC-003: worker_commit gate binary — zero-engine self-locating recipe

> Revised 2026-07-24 after two SL-225 adversarial passes: the first rejected the
> engine-publish (redundant-or-harmful); the second (RV-291) rejected the
> launch-frozen `DOCTRINE_BIN`→coord-build *precondition* as not temporally
> achievable. The engine is untouched; the gate self-locates its binary at run time.

## Decision

For SL-225 #1 (ISS-218 — the `worker_commit` commit gate's `just check` →
`just validate` → `doctrine doctor` false-reds because `validate` shells the stale
PATH `~/.cargo/bin/doctrine`):

- **Project tier only.** This repo's `justfile` `validate` recipe resolves the
  `doctrine` it shells in **layered, self-locating** order:
  `${DOCTRINE_BIN}` (optional override) → the fork's own `./target/debug/doctrine`
  (if executable) → the **git-derived coord build**
  `"$(dirname "$(git rev-parse --path-format=absolute --git-common-dir)")"/target/debug/doctrine`
  → bare `doctrine` (PATH). The PATH tail is *correct* for a generic host.
- **Self-location, not a launch-time precondition.** The gate runs with CWD = the
  worker fork, always a **linked git worktree** under the coord tree, so
  `--git-common-dir`'s parent is the coord root. The recipe finds the coord build —
  which is on `dispatch/<slice>` and carries every prior phase's rules — **at
  gate-run time**, needing no launch-frozen env, state file, or restart. A
  fork-gated refusal (`--git-dir ≠ --git-common-dir` and no coord build) emits a
  diagnostic instead of silently using the stale PATH binary.
- **`DOCTRINE_BIN` is an optional override**, no longer required for correctness.
- **Engine: no change.** `worker_commit` / the gate are not touched.

## Why zero-engine (the first adversarial reversal)

The engine-publish (`worker_commit` exports `DOCTRINE_BIN=current_exe()`) was
rejected on evidence:

- **`.mcp.json` already launches the server via `${DOCTRINE_BIN:-doctrine}`**, and
  the gate is a child of the server process, so it **already inherits** whatever
  `DOCTRINE_BIN` the server has. When `DOCTRINE_BIN` is set → publish is redundant.
- When `DOCTRINE_BIN` is **unset**, `current_exe()` is the *stale* server binary;
  publishing it **pins the recipe's first rung to the stale binary**, pre-empting
  the fall-through to a fresher build. Strictly *worse* than doing nothing.

So the publish never helps and sometimes hurts.

## Why not a launch-frozen `DOCTRINE_BIN` precondition (the RV-291 reversal, F-1)

The revised-but-still-wrong design demanded the operator set `DOCTRINE_BIN` to the
coord build for a dispatch session. That precondition is **not temporally
achievable**: `.mcp.json` launches the server via `${DOCTRINE_BIN:-doctrine}` and
the client resolves that expansion **at server launch** (boot.rs:544-549), freezing
the server env — but the coord worktree/build is created by `dispatch setup`
(dispatch SKILL.md:13) **after** the session is already running. A later operator
`export` cannot mutate the running server, and the gate is that server's child. No
"set the env before launch" rule can name a binary that does not yet exist.

Self-location dissolves the cycle: resolve from git at run time, not from a
launch-frozen variable. The only surviving precondition — **a built coord tree** —
is established after setup, before phases run, and *verified at gate time* (rung 3's
`-x` + the refusal), so it needs no restart and no env-forwarding.

## Why (POL-002 — do not conflate doctrine-as-project with doctrine-as-platform)

The stale-binary false-red is a **dogfooding artifact**: only an in-flight slice
that changes doctrine's *own* binary makes the installed binary stale relative to
the fork. A generic host never hits it. So the fix is **project-tier**; the engine
must not learn cargo's `./target/debug` layout, and must not resolve the coord path
either (both POL-002 violations). The recipe keeps `target/debug` (project/cargo
knowledge) in the project justfile and locates the coord tree with **generic git
plumbing** (`--git-common-dir`), never the dispatch `.worktrees` layout.

## Why not the alternatives

- **Bake `./target/debug` or a `current_exe()`/coord-path resolution into the
  engine gate** — POL-002 violation (cargo layout / resolution policy in platform
  code); and `current_exe()` is redundant-or-harmful (above).
- **Recipe existence-check alone, no coord rung** — holes on a **non-Rust phase**:
  `validate` runs *before* the gate's `build` leg (`DEFAULT_COMMIT = ["just",
  "check"]`, verify.rs:149), and a pure-JS / docs phase never builds the fork
  binary, so it falls back to the stale PATH binary — re-masking ISS-218's
  SL-206-P14 repro. The **git-derived coord rung** closes that hole without a local
  build, and without the unachievable launch-time precondition.
- **Launch-frozen `DOCTRINE_BIN`→coord precondition** — not temporally achievable
  (F-1, above).

## Preconditions / carried assumptions

- **Precondition (achievable):** the coord tree is built (`cargo build` in coord) —
  a natural part of dispatch setup, established before phases run and **verified at
  gate time**; absence inside a fork is *refused with a diagnostic*, not silently
  false-red. No launch-time env contract, no accepted non-Rust-phase residual.
- **ASM (own-`target/`):** each worktree builds into its own in-tree `target/`
  (AGENTS.md — no shared `CARGO_TARGET_DIR`), so `$coord/target/debug/doctrine` is
  the coord's own build. A shared-target redirect would collapse the coord rung into
  the fork-build rung — degraded, not incorrect.

## Links

- Governs SL-225 #1; fulfils ISS-218 (IMP-270 dup-closed).
- POL-002 (platform independence) is the gate that forced project/platform split,
  killed the engine-publish, and keeps coord-location on generic git plumbing.
- The softened `DOCTRINE_BIN` note (now "optional override; the gate self-locates")
  lives in `.doctrine/governance.md` (§ orchestration) and CLAUDE.md (§ Dispatch
  precondition).
- Sibling #2 (marker-aware e2e skip, CHR-044) is likewise project-tier — the engine
  already provides the gate marker-clear (SL-199 F2).
