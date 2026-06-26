# Design SL-156: Per-worktree CARGO_TARGET_DIR for dispatch workers

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

## 1. Design Problem

Dispatch workers compile in isolated git worktrees, but the jail exports **one
shared** `CARGO_TARGET_DIR` (`~/.cargo/doctrine-target-jail`, `flake.nix:80`)
across every worktree. Cargo's fingerprint then reuses an artifact built in tree
W when tests run from tree Z → **false-RED** (stale binary, deleted tests still
run) and **false-GREEN** (verify passes against another branch's artefacts). The
verify surface (`just check` / `just gate`) is exactly where this bites.

Two coupled defects:

1. **No per-worktree isolation.** The codex/pi arm partially works around it by
   emitting a per-worktree `CARGO_TARGET_DIR` from `worktree fork --worker`
   stdout; the **claude arm has no such channel** and inherits the shared dir.
2. **POL-002 violation in the platform.** The shipped product hardcodes a *cargo*
   convention: `project_env_contract` emits `CARGO_TARGET_DIR`, `coordinate`
   consumes it, `gc` reaps the derived cargo path. A non-cargo doctrine client
   would get a spurious env var injected and GC managing dirs that never exist.

Goal: per-worktree build isolation on **both arms**, with the **platform made
build-tool-agnostic** — correctness resting on a doctrine-owned contract, not a
host build tool (POL-002).

## 2. Current State

- `flake.nix:80` sets `CARGO_TARGET_DIR=/home/david/.cargo/doctrine-target-jail`
  (jail-wide; host stays on default `target/`). Rationale in the comment:
  cross-mount binary safety + warm cache + clean tree.
- **Platform cargo coupling** (the POL-002 violation):
  - `src/worktree/fork.rs::project_env_contract` → `CARGO_TARGET_DIR=<base>/wt/<branch>`
  - `src/worktree/fork.rs::run_fork` + `src/worktree/coordinate.rs:255` emit it on stdout
  - `src/worktree/gc.rs:151-157` replicate the cargo target-base to reap `wt/<branch>` dirs
- codex/pi arm (`dispatch-subprocess`) captures `$fork_env` and sets it in the
  subprocess env; claude arm (`dispatch-agent`, `Agent` tool) has no env seam.

### Empirical knowns (this session, all probed)

- **`.cargo/config.toml` cannot override an inherited `CARGO_TARGET_DIR`** —
  `[env] force=true` loses to the env var (Probe 1).
- **No Claude hook can inject env into a worker** (Probe 2,
  `mem.fact.dispatch.claude-worker-no-per-worktree-env`): SubagentStart fires in
  the worktree cwd but has no `CLAUDE_ENV_FILE`; CwdChanged (which has it) never
  fires for the spawn; the worker's Bash has no `CLAUDE_ENV_FILE`. The worker just
  inherits the orchestrator's env frozen at spawn.
- `/target` is root-anchored in `.gitignore`; registered linked worktrees are
  excluded from the parent tree's `git status`.

## 3. Forces & Constraints

- **POL-002 (required).** Shipped behaviour must rest on doctrine-owned contracts,
  never a host convention (cargo/justfile/flake are doctrine-the-repo's *client*
  habits). Platform must not load-bear on them.
- **ADR-008 (project-local).** D-B1 prescribed "per-worktree `CARGO_TARGET_DIR`,
  set at worker spawn"; D-B5 "keep flake minimal, justfile unchanged." This slice
  **moves the mechanism** → requires a Revision (§7 D1, §Governance).
- **Behaviour-preservation gate.** Removing platform machinery must leave the
  existing worktree/dispatch suites green unchanged.
- **No parallel implementation.** One isolation mechanism for both arms, not two.
- **Jail cross-mount.** Jail and host bind the repo at different absolute paths;
  the design must not reintroduce the stale-`CARGO_BIN_EXE` cross-mount spawn-fail.

## 4. Guiding Principles

- **Platform owns worktree creation; project owns build-target.** The clean
  seam: doctrine creates and reaps the worktree (its contract); the build tool,
  finding itself in that worktree, targets its own dir (the project's concern).
- **Simplest correct mechanism.** Prefer the default that is correct by
  construction over machinery that patches one entrypoint.
- **Strict-and-owned beats lenient-and-coupled** (POL-002 rationale).

## 5. Proposed Design (B1 — retire the shared env; in-tree per-worktree target)

### 5.1 System Model

**Retire the jail-wide `CARGO_TARGET_DIR` export.** With no shared env var, cargo
falls back to its default: `<worktree>/target`. Each git worktree — main tree and
every dispatch fork — compiles into **its own in-tree, gitignored `target/`**.
Per-worktree isolation is then **correct by construction**, on both arms, for
`just` *and* raw `cargo`, with **no env channel required** (which the claude
worker lacks anyway).

The platform **exits the build-env business**: it creates and reaps worktrees and
says nothing about cargo.

```
before:  flake → CARGO_TARGET_DIR=~/.cargo/doctrine-target-jail (shared)
         platform fork --worker → emits CARGO_TARGET_DIR=<base>/wt/<branch>
         codex worker: env set from $fork_env ; claude worker: inherits shared (BUG)

after:   flake → (no CARGO_TARGET_DIR)
         every worktree → cargo default <worktree>/target  (isolated, gitignored)
         platform → no build env emitted ; both arms identical
```

### 5.2 Interfaces & Contracts

Removals (platform → build-tool-agnostic):

- **`flake.nix:80`** — delete the `(set-env "CARGO_TARGET_DIR" …)`; update the
  comment to record the in-tree-per-worktree rationale (cross-mount safety now via
  distinct mount paths, §5.5).
- **`src/worktree/fork.rs`** — remove `project_env_contract`; `run_fork` no longer
  emits an env contract on stdout (its other output — the created path — stands).
- **`src/worktree/coordinate.rs:255`** — drop the env-contract emission.
- **`src/worktree/gc.rs:151-157`** — remove the cargo target-base reaping; GC
  reaps the worktree, and the in-tree `target/` dies with it.
- **`.agents/skills/dispatch-subprocess/SKILL.md`** — stop capturing/passing
  `$fork_env`; the codex worker inherits the (now unset) env and defaults in-tree.
- **`AGENTS.md`** (§95 + the `just rebuild-stale` guidance) — the shared-target
  warning and rebuild ritual are obsolete; rewrite to the in-tree model.
- **Tests asserting the removed contract** (deleted/rewritten *with* the code, not
  retrofitted): `tests/e2e_worktree_coordinate.rs:205` and `tests/e2e_worktree_fork.rs:153`
  (`stdout contains "CARGO_TARGET_DIR="`); `tests/e2e_worktree_gc.rs` (the entire
  external-target-base `wt/<branch>` reaping scaffold — `run_pinned` / target_base
  helpers). The `*.env_remove("CARGO_TARGET_DIR")` setup other e2e tests use to
  neutralise the jail env stays valid (it now matches production).
- **Project stale-target mitigations** (scope item 2) — retire what the shared
  target made necessary: `just rebuild-stale`, the `justfile:50` staleness
  comment, touch-`main.rs` rituals, and the `mem.fact.build.rebuild-stale-skips-
  test-binaries` / shared-target memory cluster (mark superseded). `./target/debug/
  doctrine` becomes the live binary again (the redirect that made it stale is gone).

No new platform interface. No new env contract.

### 5.3 Data, State & Ownership

- **Build artefacts:** per worktree, in `<worktree>/target/` (gitignored,
  disposable). Owned by the build tool, not doctrine.
- **GC:** **free.** `target/` lives inside the worktree → reaped with it on
  `worktree remove`. No orphaned-dir registry, no platform reaping path.
- **Host:** unchanged — it never had the jail export; it stays on its own in-tree
  `target/` at the host mount path.

### 5.4 Lifecycle, Operations & Dynamics

- **Persistent trees (main/edge):** keep a warm cache — the in-tree `target/` is
  part of the bound working tree, which persists across jail relaunches.
- **Ephemeral forks:** **cold-build** on first compile (no shared warm cache).
  Accepted cost; the warm-fork-cache answer is **D-B4 (sccache), deferred in
  ADR-008** — pulled forward only if cold builds measurably hurt (§6 OQ-1, §8 R1).
- **Worker verify:** `just check` / `just gate` in the fork now reports honest
  pass/fail — no cross-worktree artefact thrash, no touch+re-run ritual.

### 5.5 Invariants, Assumptions & Edge Cases

- **INV — isolation by construction:** with no shared `CARGO_TARGET_DIR`, no two
  worktrees can share a target dir. (Holds unless something re-exports it — guard
  in §9.)
- **Cross-mount preserved:** jail builds into `/workspace/doctrine/target`, host
  into `/home/.../doctrine/target` — distinct physical dirs ⇒ no shared binary ⇒
  no stale-`CARGO_BIN_EXE` cross-mount spawn-fail. The original flake concern is
  satisfied *by* in-tree targets, not by the redirect.
- **Gitignore:** `/target` is root-anchored per worktree → each `<worktree>/target`
  is ignored; registered linked worktrees are excluded from the parent's status,
  so forks' `target/` never dirties the main tree.
- **Edge case — anything re-exporting `CARGO_TARGET_DIR`:** a user shell/profile
  or CI that still exports it would re-share. Mitigation: the flake is the only
  in-jail source; §9 adds a verify that two worktrees produce distinct target
  paths.
- **Assumption:** no consumer depends on `project_env_contract`'s stdout env line
  beyond the dispatch-subprocess skill (verified in §2; confirm in tests).

## 6. Open Questions & Unknowns

- **OQ-1 — cold-fork-build cost.** How much does a cold debug build per fork add
  to dispatch wall-clock? If material, pull D-B4 (sccache) forward. Measure during
  validation; do not pre-optimize.
- **OQ-2 — disk.** N concurrent worktrees = N `target/` dirs. Bounded by the
  worktree cap and reaped on removal; monitor, no cap added here.
- **OQ-3 — stale-target memory cluster.** Which memories are fully superseded vs
  still true for the host? Triage at reconcile (§Governance).

## 7. Decisions, Rationale & Alternatives

- **D1 — B1 (retire shared env; in-tree per-worktree target) over B2 (persisted
  `DOCTRINE_CARGO_TARGET_ROOT` + token + project GC).** B1 is simplest, GC-free,
  fixes cross-mount, needs no worktree-id token, and satisfies POL-002 by the
  platform simply *not exporting*. B2 buys warm fork cache at the cost of a
  derivation token, a justfile redirect, and a lazy GC recipe — complexity better
  spent on D-B4 (sccache), which solves warm-fork-cache properly. *Trade: cold
  fork builds now; sccache later if it bites.*
- **D2 — platform exits build-env; reject a "generic owned env-contract seam."**
  A doctrine.toml-driven generic env injector would *launder* the smell (codex
  consult): the claude arm cannot consume platform-emitted env at all (Probe 2),
  so the seam is half-fiction, and it invites the product to keep owning client
  build behaviour under a politer name. Per-worktree build env is a **project**
  concern, full stop.
- **D3 — retire the flake `CARGO_TARGET_DIR` entirely (not keep-as-base).** Keeping
  the shared export would leave raw `cargo`/ad-hoc scripts thrashing the shared
  target — "teaching one wrapper to dodge the bug" (codex). Retiring it makes the
  in-tree default correct everywhere.

### Alternatives rejected

- **A — prompt base-guard `export`.** Per-call agent compliance; fails exactly at
  the verify surface. Fragile.
- **C — `.cargo/config.toml` `build.target-dir`/`[env]`.** Proven dead (Probe 1):
  inherited env wins.
- **D — hook → `CLAUDE_ENV_FILE`.** Proven dead for the claude worker (Probe 2):
  no env channel reaches the subagent.
- **B2 — persisted root + token.** See D1; deferred to D-B4 if warm fork cache is
  needed.

### Governance — ADR-008 Revision (required before lock)

ADR-008 D-B1 ("per-worktree `CARGO_TARGET_DIR` set at worker spawn" — i.e. by the
platform) and D-B5 ("justfile unchanged; per-worktree env set at spawn") are
**superseded in mechanism** by this slice: isolation now comes from the *absence*
of a shared export (project/flake), not platform env injection. The *intent* of
D-B1 (per-worktree isolation, correct `CARGO_BIN_EXE`) is **preserved and better
served**. Route the change through a **Revision** to ADR-008 (ADR-013), recording:
D-B1 mechanism change, D-B5 (flake loses the export; the removal *is* the
mechanism), and POL-002 as the forcing function. D-B2/D-B3/D-B4 unchanged
(D-B4 gains relevance as the warm-fork-cache lever).

## 8. Risks & Mitigations

- **R1 — cold-fork-build latency.** Mitigate: measure (OQ-1); D-B4 sccache if
  material. Not a correctness risk.
- **R2 — migration-order regression.** Removing platform emission before the codex
  skill migrates drops codex workers back to the (then-unset) env mid-flight.
  Mitigate: **phase project-side first** — (i) retire flake export + confirm
  in-tree isolation, (ii) migrate dispatch-subprocess skill/docs/tests, (iii)
  remove `project_env_contract`/`coordinate`/`gc` coupling last.
- **R3 — premature removal of stale-target rituals.** Some mitigations may still
  serve the host or non-jail flows. Mitigate: re-evaluate each individually; mark
  memories superseded only when confirmed.
- **R4 — disk growth.** Bounded + reaped with worktrees; monitor (OQ-2).
- **R5 — flake effect needs a jail relaunch.** `set-env` applies at jail launch,
  so removing it is not live in the authoring session. Mitigate: validate the
  mechanism in-session by simulating with `.env_remove("CARGO_TARGET_DIR")` (the
  e2e harness pattern already in the suite); the true end-to-end check (VT-1/VT-2)
  runs after a jail relaunch. Note for the executor: the first build post-relaunch
  is cold for every tree (the old `~/.cargo/doctrine-target-jail` is abandoned —
  one-time; remove it out-of-band).

## 9. Quality Engineering & Validation

- **Behaviour-preservation — scoped precisely.** The gate holds for worktree
  *creation / provision / marking* and dispatch coordination — those suites must
  stay green unchanged. It does **not** cover the env-emission contract: removing
  `project_env_contract` is a *deliberate* contract change, so the tests that
  assert it (`e2e_worktree_coordinate.rs:205`, `e2e_worktree_fork.rs:153`) and the
  gc target-base scaffold (`e2e_worktree_gc.rs`) are **deleted/rewritten with the
  code**, not retrofitted. Distinguish the two in review: a *creation* test going
  red is a regression; an *env-contract* test going red is expected and its
  deletion is the change.
- **VT-1 — isolation by construction:** two worktrees on different branches each
  build a binary their *own* e2e tests spawn successfully; the target paths are
  distinct (`<wt>/target`), no cross-thrash. (Discharges ADR-008 D-B1 verification.)
- **VT-2 — both arms honest verify:** a claude-arm and a codex-arm worker each run
  `just check` reporting correct pass/fail with no touch+re-run ritual.
- **VT-3 — gc no longer references a cargo path:** `gc` reaps a worktree and its
  in-tree `target/` without any `wt/<branch>` base logic.
- **VA — POL-002 conformance:** `grep` confirms no `CARGO_TARGET_DIR` / cargo
  literal remains in shipped `src/`.

## 10. Review Notes

- **Probe 1/2 (this session):** config-env override dead; claude worker env-channel
  dead. Recorded in `research.md` + `mem.fact.dispatch.claude-worker-no-per-worktree-env`.
- **External consult — codex / GPT-5.5 (thread 019f01e9):** approved PLATFORM→
  PROJECT; rejected generic env seam; flagged the id-token blocker (moot under B1,
  no token), mandated retiring the flake export (D3), and the migration order (R2).
### Internal adversarial pass (2026-06-26)

Hostile read of the draft + grounding greps. Findings, integrated above:

- **AP-1 — "suites green unchanged" was imprecise.** `e2e_worktree_coordinate.rs:205`
  and `e2e_worktree_fork.rs:153` assert the `CARGO_TARGET_DIR=` stdout line; that
  contract is deliberately removed. §9 now scopes behaviour-preservation to
  creation/marking and names the env-contract tests as deleted-with-code.
- **AP-2 — gc tests entangled.** `e2e_worktree_gc.rs` builds external-target-base
  `wt/<branch>` reaping scaffolds for `gc.rs:151-157`. Added to the touch-set (§5.2).
- **AP-3 — live project doc.** `AGENTS.md:95` + `just rebuild-stale` guidance is
  stale under B1. Added to §5.2 removals.
- **AP-4 — jail-relaunch latency.** flake `set-env` is launch-time; mechanism not
  live in-session. Added R5 (§8) — simulate via `.env_remove`, true check
  post-relaunch.
- **AP-5 — related entities (reconcile relations).** `CHR-014` (closed) is the
  path-baking *cousin* axis, not the implementer; the shared-target *artifact* axis
  this slice fixes is tracked by the open backlog item (research cites IMP-004). See
  also `review/158`, backlog issues 044/037/008 — touch-points for the memory/ritual
  cleanup (scope item 2).

### External adversarial review (codex / GPT-5.5)

- **Architecture consult (thread 019f01e9, pre-draft):** done — see §10 above /
  `research.md`. Approved PLATFORM→PROJECT; mandated D3 + migration order.
- **Design-doc hostile pass:** _pending_ — handed to the next agent (this session's
  `/handover`). Target: the full `design.md` against ADR-008/POL-002, the
  behaviour-preservation scoping (AP-1), and the migration-order phasing (R2).
