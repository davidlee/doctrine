# SPEC-012: Dispatch & worktree

<!-- Reference forms: entity ids padded (SPEC-007, ADR-004); doc-local refs bare
     (D1 decision, OQ-1 open question). See .doctrine/glossary.md § reference forms. -->

> **AMENDED — SL-254 (2026-08-14).** SL-254 re-shaped this container in two moves:
> it collapsed dispatch's **two spawn arms** (an in-session claude `Agent`-tool arm
> and a codex/pi subprocess arm) onto **one confined subprocess arm** — every harness
> is spawned by `scripts/spawn-confined.sh <harness>` inside a kernel-level jail
> (`DEC-208`, `DEC-217`) — and it replaced the **disk worker marker** with
> process-scoped `DOCTRINE_WORKER` identity (`DEC-207`). Sections that are
> present-tense *description of shipped mechanism* (Overview, Responsibilities, the
> mechanism sections) have been **rewritten** to describe what ships. Sections that
> are *records of reasoning* (Concerns, Hypotheses, Decisions) keep their historical
> bodies and carry per-item amendment banners. Requirement bodies are amended
> separately and are not restated here.

## Overview

Dispatch & worktree is the isolation-and-coordination container for concurrent
work. It sits beneath the whole-system root (SPEC-003) and carries **no descent**:
no PRD owns it — ADR-006 (worktree posture: policy-agnostic framework,
orchestrator-sole-writer dispatch, amended SL-056 G2) and ADR-011 (harness-agnostic
orchestrator spawn interface and per-harness capability altitude — its
*per-harness-altitude* half retired by SL-254 `DEC-208`; the harness-agnostic spawn
interface half stands, now realised as one confined arm) are its governing decisions.

The container's keystone is that **the orchestrator funnel is enforced CLI
mechanism, not prose an LLM may skip** (ADR-011 context — "mechanism in prose is the
design smell"). Worker-sole-writer rides the **`DOCTRINE_WORKER` environment
variable, and nothing else** (SL-254 `DEC-207`): identity is a property of the worker
**process**, set by the same confinement argv that establishes its write floor, and
it dies with the process. The provision core is **harness-identical and
golden-testable**; the verbs that carry the funnel — `fork`, `import`, `land`, `gc` —
are CLI verbs, refused under `worker_mode`, not a discipline carried in skill text.
The spawn line is no longer prose either: `scripts/spawn-confined.sh <harness>` is
the **sole** spawn path for every harness, and a harness-specific command (for claude,
`claude -p --output-format stream-json`) is now a **required element of that shipped
path** rather than the disqualifying smell ADR-011 D3 treated it as. There is no
`/dispatch-*` router and no second arm to select between.

It owns these mechanisms specific to isolation: **fork provisioning** with a
two-layer tier exclusion the copy physically cannot leak; the **orchestrator verb
family** (`fork`/`import`/`land`/`gc`) that creates, funnels, and reaps forks under
the worker-mode guard; the **worker-mode guard** — one signal, `DOCTRINE_WORKER == "1"`
on the worker process; the **confinement seam** (`jail.rs`/`jail_prefix.rs`, the
`worktree jail-prefix` verb) that wraps every worker exec in a bwrap (Linux) or
`sandbox-exec` (macOS) jail whose writable set is the fork worktree **plus the
harness config dir**, and that **fails closed by name on Linux** when `bwrap` is
absent (macOS has no equivalent backend-presence probe — see Concerns); the **branch-point guard**, a
HEAD-stationarity assertion at the batch-commit boundary; and the **born-frame git
seam** that confines all git/disk/process impurity to one shell. Shared substrate — identity, the atomic
claim, id allocation, the scaffold/render pipeline, the storage rule and the
pure/imperative split as system-wide principles — lives in the parent (SPEC-003) and
the entity-engine container (SPEC-004) and is not restated here. Trunk-side id minting
and the reseat verb that resolves offline collisions (ADR-006 D3/D8) belong to the
id-lifecycle container; this container provides the isolation those acts run beneath.

> **Posture (rewritten SL-254).** The forward-intent posture SL-056 recorded here is
> **discharged**: the funnel-verb family (`fork`/`import`/`land`/`gc`), the worker-mode
> guard, the three read verbs (`provision`/`check-allowlist`/`branch-point-check`), the
> born-frame git seam, and the confinement seam (`jail-prefix` + `spawn-confined.sh`)
> are all **shipped**. What was forward-intent and did *not* ship in this shape is the
> **marker** guard and the **per-harness spawn paths** — both retired at SL-254
> (`DEC-207`, `DEC-208`). Coverage is reconciled, never inferred.

## Responsibilities

Mirrors the structured `responsibilities` list: provision a fork as the sole copier
with guaranteed tier exclusion; carry the funnel as an orchestrator verb family
(`fork`/`import`/`land`/`gc`) refused under the worker-mode guard; enforce
worker-sole-writer via a **process-identity guard keyed on `DOCTRINE_WORKER`**
(amended SL-254 — it was a disk-marker-primary, fail-closed-on-ambiguity guard);
**confine every worker exec in a kernel-level jail whose writable set is the fork
worktree plus the harness config dir, refusing rather than spawning unconfined**; assert
HEAD-stationarity at the batch boundary; capture the impure born frame for anchoring;
and defend tier merge-safety by the tier's absence in the fork.

### Fork provisioning — the sole copy path with guaranteed tier exclusion

`worktree provision <fork>` is the **only** copy path into a fork. The pure core
(`select_copies`, `parse_allowlist`, `is_withheld`) takes paths and strings as
inputs — no disk, git, clock, or rng (ADR-001 leaf) — and the thin impure shell
(`run_provision`) reads `.worktreeinclude`, drives `git ls-files`/`rev-parse`
through the `git.rs` runners, and copies via the `fsutil` safe-copy helper. The
exclusion is **two-layer** (design OQ-3-B): `select_copies` is the *guarantee* — it
drops any file matching the coordination/runtime tier even when a broad `**`
allowlist would otherwise admit it, so the copy physically cannot leak the tier;
`allowlist_violations` behind `check-allowlist` is a static *smell test* whose green
result is explicitly **not** completeness. The withheld tier — the five `Tier`
variants: `.doctrine/state/`, the relative `phases` symlink, `handover.md`,
inquisition scratch, and memory caches — is classified in `is_withheld` by `Tier`.
(The worker marker at `.doctrine/state/dispatch/worker` used to be named here as a
path that inherited every withheld-tier exclusion with no new tier logic; that file no
longer exists — worker identity is process-scoped, SL-254 `DEC-207` — so the clause is
moot, not merely stale. Everything else under `.doctrine/state/dispatch/` is still
withheld by the same `Tier`.) As a fail-fast convenience, provision aborts before
copying if any `.worktreeinclude` pattern names a withheld tier — the same smell test,
not a substitute for the copy-time `select_copies` guarantee that runs regardless —
and `verify_sibling_worktree` refuses to provision the source tree onto itself.

### The orchestrator verb family — `fork` / `import` / `land` / `gc`

The funnel is four `Orchestrator`-classed verbs, each refused under `worker_mode`
(they mutate git refs/dirs — create/remove worktrees, delete branches, merge commits,
reap dirs; classifying them `Read` because they spare the authored TOML corpus would
be a category error, ADR-006 D2/D2a).

- **`fork --base <B> --branch <name> --dir <path> [--worker] [--slice N --phase PHASE-NN]`**
  (orchestrator-owned creation, one path for every harness — amended SL-254): one act —
  `git worktree add -b <branch> <dir> <B>`, then `provision` (sole copier, withheld
  excluded). It stamps **nothing**: worker mode is a property of the worker *process*
  (`DOCTRINE_WORKER`, set by the confining spawn), never of the directory, so `--worker`
  now only gates the **durable fork binding** — with `--slice`/`--phase` the fork is
  bound to `(slice, phase)` at creation; without them it is minted **unbound**, which is
  what the SL-254 spawn path does (`OQ-1`), leaving the funnel row named by the
  orchestrator's explicit `PHASE-NN`. Status goes to **stderr and stdout stays empty** —
  the per-worktree env contract it once emitted on stdout went out with the shared
  `CARGO_TARGET_DIR` redirect; each fork builds into its own in-tree `<dir>/target`
  (ADR-008 D-B1). It is **compensating cleanup, not a transaction** — git mutations are not
  atomic, so any failure after `git worktree add` triggers a best-effort rollback
  (`git worktree remove --force` + `git branch -D` + reap dir); a rollback that
  itself fails **names the leftover and exits non-zero**.
- **`import --base <B> (--from-worktree <dir> | --fork <branch>) [--slice N]`** — the
  dispatch funnel (single distilled delta, ancestry severed). Since SL-254 the worker
  **cannot commit** — it is confined to a jail whose git metadata it may not write — so
  `--from-worktree` (gather the *uncommitted* working-tree delta from the worker's
  persisted fork dir) is the shipped dispatch source; `--fork <branch>` survives for the
  already-committed-fork case (amended SL-254). v1 is the stationary-head case, each step
  a hard refusal, no auto-merge: precond `HEAD == B` (`branch-point-check`) **and** a
  clean coordination tree (tracked+staged only); for the `--fork` source, `S^ == B`
  (single non-merge delta); then the **belt**, then `git apply --3way --index`
  (non-committing — the orchestrator commits **separately**, ADR-006 D7 cadence). **No
  runtime receipt is stamped** — a flag born before the commit would survive a crash and
  lie "landed" to `gc`.
  The belt rejects on exactly three grounds: the tracked name-only delta touches
  `.doctrine/` (`doctrine-touch`) or `.claude/` (`claude-touch`) — **two hard-coded
  floors, and nothing else** — plus, when `--slice` is given, a path no design-target
  selector declares (`undeclared-scope`, SL-180). It has **never** enforced the
  configurable `worker-forbidden-writes` list (amended SL-254; see Concerns).
- **`land --fork <branch>`** — solo `/execute`'s analog (multi-commit branch, ancestry
  preserved): `git merge --no-ff <branch>`, **structurally non-squash** (see D7). It
  refuses a **dispatch-shaped fork branch** (`dispatch-fork` — that delta must funnel
  through the belted `import`) and a worktree-gone fork (`worktree-gone` — there is
  nothing live to land from, so the dispatch-fork test would be answering about a
  corpse). The dispatch-fork test is `src/worktree/shared.rs::is_dispatch_fork_branch`
  — branch **shape**, `dispatch/<name>` with a non-numeric suffix, which excludes both
  the coordination branch `dispatch/<NNN>` and solo `/worktree` isolation branches
  (amended SL-254 `DEC-207`; it was a cross-tree read of the other tree's marker, which
  the process-scoped env leg cannot do, and it is **strictly stronger** — it fires
  whether or not a fork was ever stamped). It is *not* `classify_worktree_role`
  returning `"fork"`: that returns `"fork"` for every linked non-coord worktree,
  including the solo branches `land` exists to land. A conflicted merge is **aborted
  before** the refusal (a half-merge wedges the tree against the verb's own re-entry
  guard).
- **`gc --fork <branch> [--superseded-head <SHA>] [--force] [--dry-run]`** — reaps
  worktree + branch + target-dir in one act, **only** when the fork's commit *provably
  landed* against durable git state via `git cherry <coordination-HEAD> <fork-branch>`:
  **ancestry** (the `land` route) **OR** every listed commit is `-` (the `import`
  route's patch-id). It is an idempotent state machine — a `gc` that crashed between
  destructive steps completes on rerun or names the leftover. The `--superseded-head
  <SHA>` reaps a re-dispatched (spent-but-never-landed) fork iff `<SHA>` matches the
  branch head (a TOCTOU movement-guard, not a landing proof).

### The worker-mode guard — one signal, a property of the process

> **REWRITTEN — SL-254 (2026-08-14).** This section formerly described a
> disk-marker-primary, fail-closed-on-ambiguity guard: `worker_mode = (is_linked_worktree
> && marker_present) OR env DOCTRINE_WORKER`, an eight-row `Cause` truth table, a
> `marker --stamp-subagent` mint exempt by verb identity, and a `marker --clear
> --operator` self-brick escape. **All of it is gone** (`DEC-207`). The reasoning that
> produced it is retained as a record in D3 below; what follows is the shipped
> mechanism.

Worker-sole-writer is enforced in the CLI by a guard in `run()`
(`src/commands/guard.rs::worker_guard`), before dispatching a `Write`- **or**
`Orchestrator`-classed `Command` (ADR-006 D2a, ADR-011 D1 as amended):

```
worker_mode := env DOCTRINE_WORKER == "1"   // the whole truth (SL-254 DEC-207)
if worker_mode: refuse(verb)                // names the verb
```

The predicate is a **value** compare, not a presence test (`marker.rs::env_worker_set`
against `jail::ENV_WORKER_ON`): `DOCTRINE_WORKER=0` — or any value other than `1` — is
*set* but is **not** worker mode. Only the literal `1` the confining argv writes turns
the guard on.

Identity is a property of the **process**, not of a tree. `DOCTRINE_WORKER` is set by
the same confinement argv that establishes the worker's write floor — on Linux by
`--setenv DOCTRINE_WORKER 1` in `scripts/spawn-confined.sh`'s inline bwrap array
(`:183`), on macOS by the trailing `env DOCTRINE_WORKER=1` token
`jail.rs::sandbox_exec_argv` appends. `jail.rs::bwrap_argv` sets **no** env: the
Linux env leg lives in the shell, not in the Rust argv builder (amended SL-254 —
this section formerly cited `bwrap_argv` as a setter). It dies
with the process — so there is no stale class to detect, no cure verb to gate, and no
tree-topology conjunct to disambiguate. The predicate that *observes* identity and the
argv that *establishes* it share the same two constants (`jail::ENV_DOCTRINE_WORKER`,
`jail::ENV_WORKER_ON`) so they cannot drift (STD-001). `marker.rs` survives only as the
env leg: `describe_mode(env_set) -> StatusLine` is a **two-row** truth table where
SL-056's was eight rows over three inputs, and it remains the single source for both
the `worktree status` human line and the guard's refusal (the anti-parallel-implementation
property). Because the signal cannot go stale, `worktree status --assert`, `marker
--stamp-subagent`, `marker --clear --operator`, and the `Cause` truth table all retired
with the marker — the `marker` verb family no longer exists.

`write_class` is an **exhaustive** match over every `Command` variant with no wildcard
arm — a future verb is a compile error, never a silently-permitted write (design X4).
It has three classes: `Read`, `Write(verb)`, and `Orchestrator(verb)` — the funnel
verbs plus the human-in-the-loop admission family; the `Hook-mint` class retired with
the hook family it minted for. `Write` and `Orchestrator` are refused by the same
branch. Reads stay open, and `provision`/`check-allowlist`/`branch-point-check`/`status`
are deliberately `Read` (they write *fork* files, not the doctrine state the guard
protects). Observation writes carry a capability-aware refusal whose text still offers
a confined claude worker the `observation_record` MCP broker
(`src/commands/guard.rs:460-465`) — but that is a **dead branch on the shipped path**
(amended SL-254, agreeing with SPEC-028): `scripts/spawn-confined.sh:221` execs
`claude -p` with `--strict-mcp-config` and **no** `--mcp-config`, so no MCP server is
reachable and `install/agents/claude/dispatch-worker.md` tells the worker it has no MCP
tools (`DEC-216`). The operative row of that refusal is the other one — report the
friction signal in the hand-back for orchestrator-side capture. The guard string is
retained, not relied on; it becomes live again only if a spawn profile ever grants a
worker MCP.

The legitimate orchestrator is unaffected because it is simply not a worker process —
no root resolution, no cwd-shaped failure path, and no ambiguity about the tree it
happens to be standing in.

Raw-tree confinement (a worker hand-editing a file or running a bare `git commit`) is
still **not** CLI-stoppable (ADR-006 D2b) — but it is no longer deferred. It is
enforced **below** the CLI by the confinement seam (ADR-008, ADR-020): every worker
exec is wrapped in a kernel-level jail — bwrap on Linux, `sandbox-exec` on macOS. The
two backends fence writes by different means and must not be conflated (amended SL-254):

- **Linux (bwrap).** `--ro-bind / /` makes the whole filesystem read-only, then the
  writable set is re-opened by explicit binds — the fork worktree (`--bind "$D" "$D"`),
  the harness config dir (`--bind "$CFG_DIR" "$CFG_DIR"`), a private `/tmp` tmpfs, and
  `/dev`. Reads and exec are otherwise open; writes outside that set are kernel-denied.
- **macOS (Seatbelt).** There is no `--ro-bind`. The profile is `(allow default)` plus a
  coarse `(deny file-write*)` floor (`jail.rs:208-209`), re-opened by
  `(allow file-write* (subpath …))` for the worktree, its `.tmp`, the device sinks, and
  one `RW0…RWn` allow per `--extra-rw` grant — the harness config dir rides `RW0`
  (`spawn-confined.sh:159`, `jail.rs:482-486`). So on macOS **only writes are fenced**;
  reads and exec are unrestricted.

**The harness config dir is a writable carve-out on both platforms, and it is
security-relevant.** `$HOME/.claude` (claude) / `$HOME/.pi` (pi) is bound read-write
into the jail — deliberately, because claude needs it for the subscription credential
(`DEC-210`) and pi writes it at runtime — so a confined worker **can write the
orchestrator's own harness configuration**: `settings.json`, hooks, agent definitions.
The narrowed mount set is a known, deliberately-untaken refinement
(`spawn-confined.sh:30-34`). The shipped `dispatch-spawn` skill states this correctly
and this spec now agrees with it: the writable set is *the fork worktree and the
harness config dir*, not the fork worktree alone.

Git metadata is read-only inside the jail, which is *why* the worker
cannot commit and hands back an uncommitted working tree. The prefix is minted by
`worktree jail-prefix` (fail-closed: any resolve/validate/write error ⇒ nonzero exit,
a named reason on stderr, and no partial output file). **On Linux** the spawn script
**probes first and refuses by name** — `bwrap-unavailable`
(`jail.rs::REASON_NO_BWRAP`) — minting no fork and spawning nothing. **On macOS there
is no backend-presence probe at all** (amended SL-254): `have_bwrap` is
`#[cfg(not(target_os = "macos"))]` with no Darwin sibling, `spawn-confined.sh:61` skips
the probe on Darwin, and `jail_prefix.rs`'s macOS arm fails only on topology, policy, or
profile-write errors. A macOS host lacking `sandbox-exec` therefore gets a *successful*
prefix write followed by an **unnamed** exec failure — still fail-closed, but not named.
That is a gap in `DEC-208`'s named-refusal guarantee, not a property of it. There is **no
unconfined fallback and no reduced-enforcement rung** (SL-254 `DEC-208`). The
`import` belt remains as the funnel-side containment, but it is now the second line,
not the only one.

### The branch-point guard — HEAD-stationarity, not merge-base

`worktree branch-point-check --base B` asserts that coordination HEAD still equals
the orchestrator's pre-spawn captured base `B`. It is a **ref-equality compare**
(`matches(base, head)`) — exit 0 when HEAD is stationary, exit 1 ⇒ re-dispatch from
the moved HEAD. Because a file-disjoint batch imports onto the single `B` and commits
once, HEAD moves only at the orchestrator's own batch commit; a mismatch therefore
names an *external* mover. This is a stationarity check, never a merge-base
computation (design C-V).

### The born-frame git seam — impurity confined, normalizers byte-for-byte

`git.rs` is the impure born-frame producer for memory anchoring (SL-007). It confines
all git, disk, and process impurity to one module: the `git_bytes`/`git_text`
runners, `capture` of the working-tree frame, remote selection, and submodule
rejection. It freezes the `GitContextFrameV1` normalizers **byte-for-byte** —
`forget.remote.v1` (`normalize_remote_url`, versioned by the `REMOTE_NORMALIZER` tag)
for routing-identity remote-URL normalization, and `forget.checkout.v1`
(`checkout_state_id`, the `CHECKOUT_NORMALIZER` tag) for content-bearing dirty-tree
hashing — so a `repo_id` and checkout-state id derived here are stable across captures
exactly. The version tags make the algorithm replaceable without silently re-anchoring
existing memories.

### Tier merge-safety by construction

The container defends ADR-006 D4 not by trust but by absence: the coordination/runtime
tier is never copied into a fork, so a worker has nothing shared-mutable to corrupt —
a copied phase sheet would be invisibly mutable across worktrees. The orchestrator's
pre-distilled worker prompt (ADR-006 D6) substitutes for the withheld coordination
state; provisioning substitutes for the absent execution environment. No central index
or counter exists to reintroduce a conflict.

### One confined spawn arm — uniform across harnesses, NOT uniform across platforms

> **REWRITTEN — FALSIFIED (SL-254, 2026-08-14).** This section formerly described
> **two arms at two altitudes**: a codex/pi subprocess arm (`/dispatch-subprocess`,
> marker + env identity, explicit base, bwrap) and a first-class in-session claude arm
> (`/dispatch-agent`, the `Agent` tool at `isolation: worktree`, `SubagentStart`-hook
> marker stamping, an opaque Claude-chosen base, no bwrap). SL-254 collapsed the two
> onto **one** confined subprocess arm and abolished the altitude axis itself
> (`DEC-208`, `DEC-217`). The historical two-arm analysis is preserved in ADR-011 D3/D5/D6
> and in D6 below, both amended there; it is not restated here, because a spec section
> is a description of shipped mechanism and this one described mechanism that no longer
> exists.

There is one arm. `scripts/spawn-confined.sh <harness>` forks the worktree, resolves a
confinement prefix through `worktree jail-prefix`, and execs the harness inside it —
claude included. Consequences, all uniform across **harnesses** — the axis SL-254
collapsed. Uniformity across **platforms** is a separate and weaker claim: the floor is
uniform *in intent and in write-fencing*, but the Linux and Darwin argv differ, and that
divergence is stated in its own bullet below rather than folded into the word "uniform".

- **Identity** is `DOCTRINE_WORKER`, set by the confining argv. No marker, no
  per-harness identity medium, no worker-on-main hazard to catch (the variable travels
  with the process, not the directory).
- **Base is explicitly pinned** for every harness — the orchestrator owns `fork --base
  <B>`, so the opaque Claude-chosen base is gone along with the
  clean-applying-but-semantically-wrong import it admitted.
- **Pre-dispatch baseline-verify holds** for every harness, because the orchestrator
  creates and provisions the fork before any spawn window.
- **Confinement is the floor, not an enhancement.** On Linux, no bwrap ⇒ a *named*
  refusal (`bwrap-unavailable`) and no spawn. On macOS there is no backend-presence
  probe, so a missing `sandbox-exec` fails closed *unnamed*, at exec (amended SL-254 —
  the previous text claimed a named refusal on both). Either way there is no degraded
  rung to fall to.
- **The two platform argv are NOT byte-parallel (amended SL-254 — "uniform" over-claimed
  here).** Three live divergences, all verified in shipped code:
  1. **Network.** `JailPolicy::network` defaults false (`src/worktree/mod.rs:230-232`)
     and the Darwin arm calls `jail-prefix` without `--network`
     (`spawn-confined.sh:158-159`), so the Seatbelt profile emits `(deny network*)`
     (`jail.rs:487-490`). The **Linux inline bwrap array carries no `--unshare-net`**
     (`spawn-confined.sh:176-183`), so a Linux worker has **full network**. The
     `--unshare-net` leg exists only in `jail.rs::bwrap_argv`, which the Linux spawn
     path does not use.
     **Sharp consequence, unresolved:** a macOS `claude -p` worker is network-denied and
     so could not reach the API at all — the Darwin claude arm *as wired* appears
     non-functional. This is a live question for SL-254 `PHASE-09`/`VA-2`'s Darwin
     census, not a documentation nicety.
  2. **Process lifetime.** `--die-with-parent` (`jail.rs:428`) has no Seatbelt analog, so
     reap semantics differ between the platforms.
  3. **Read/exec surface.** Linux is `--ro-bind / /` (reads open, writes denied by the
     bind topology); macOS is `(allow default)` + `(deny file-write*)`, so reads and
     exec are unrestricted and only writes are fenced.
- **A harness-specific command is now a required element** of the shipped path — the
  claude leg execs `claude -p --output-format stream-json --strict-mcp-config
  --permission-mode bypassPermissions`. ADR-011 D3 treated that as a disqualifying
  smell; SL-254 accepted it as the price of one uniform floor. Per-harness argv is
  contained in one script, not spread across router skills.
- **Build isolation** is per-worktree by cargo default (each fork builds into its own
  in-tree `target/`), not by a `CARGO_TARGET_DIR` redirect (ADR-008 D-B1).
- **The worker cannot commit.** Git metadata is read-only inside the jail, so the
  worker hands back an uncommitted working tree and the orchestrator imports the
  working-tree diff (`import --from-worktree`). The gated `worker_commit` MCP tool that
  once let the claude arm self-commit is deleted (`DEC-204`).

## Concerns

- **Raw-tree confinement is no longer the deferred residual (amended SL-254).** The
  funnel is enforced CLI mechanism (the `Orchestrator` verbs + the worker-mode guard),
  and what the CLI *cannot* stop — a worker hand-editing a file or running a bare
  `git commit` (ADR-006 D2b) — is now stopped **below** it: the harness *does* confine
  workers to their worktree, by a kernel-level jail wrapping every worker exec, with a
  named fail-closed refusal when no backend exists (`DEC-208`). The ADR-008 deferral
  this concern recorded is **discharged**. The `import` belt remains as the funnel-side
  second line.
  > **Retained for the record (the pre-SL-254 text):** "Raw-tree confinement is the
  > deferred residual, not the funnel. … the harness does not confine workers to their
  > worktree. This is a known live risk, deferred to sandbox/harness work (ADR-008), and
  > contained on the dispatch funnel by `import`'s `.doctrine/`/`.claude/` belt."
- **The residual is now the unenforced configurable forbidden-writes tail (SL-254).**
  `DispatchConfig::worker_forbidden_writes` (the `worker-forbidden-writes` key) parses
  and compiles, but its **only production reader was the `worker_commit` MCP tool**,
  which SL-254 deleted (`DEC-204`). The import belt never read it — see the next
  concern. So the configurable tail (`.agents/**`, `install/agents/**`, `flake.nix`, …)
  currently enforces **nothing**. The gap is carried forward to SL-255 (`IDE-051`),
  whose intended fix is read-only bwrap binds at *spawn* time rather than a post-import
  belt.
- **The claude altitude is weaker, and that is stated, not hidden.**
  > **AMENDED — FALSIFIED (SL-254, 2026-08-14).** There is no per-harness altitude
  > axis any more, and no in-session claude arm to be weaker: every harness is spawned
  > by the one confined subprocess path, with a pinned base, a pre-dispatch
  > baseline-verified fork, and `DOCTRINE_WORKER` identity (`DEC-208`, `DEC-217`). Each
  > named concession below is thereby dissolved rather than mitigated: no
  > `SubagentStart` stamp exists to fail, no late-only baseline verify, no opaque base.
  > The historical concern is retained for the record.

  *(Historical body.)* SubagentStart-stamp is **not fail-closable** (read-only event);
  the stamp-failure case is contained by the marker-absent fail-closed privilege rule,
  claude has **no pre-dispatch baseline-verify** (caught late at `import → verify`), and
  its **base is opaque** (a clean-applying-semantically-wrong import is possible,
  IMP-043 deferred). codex/pi keep the explicit base and pre-dispatch gate.
- **The import belt's scope is honest and narrow — and narrower than once claimed.**
  It enforces exactly **two hard-coded floors**, `.doctrine/**` and `.claude/**` (plus
  the `--slice`-scoped `undeclared-scope` check, SL-180); it has **never** consulted
  the configurable `worker-forbidden-writes` list, and `DEC-204`/`DEC-213`'s premise
  that `classify_import` was `worker_commit`'s surviving enforcement replacement for
  that list is **false** — verified in code at SL-254 PHASE-06 (amended SL-254). The
  `.doctrine/`/`.claude/`
  rejection belt is the **dispatch/import-path** containment, not an unconditional
  all-funnel guard — solo's `land` is a second, **beltless** sanctioned funnel (a
  trusted self-orchestrator legitimately lands doctrine). Because the diff is
  tracked-files-only and *all* of `.claude/` is gitignored, the `.claude/` leg contains
  **only force-add injection** (`git add -f` of a `.claude/` path) — normal installer/
  harness output never enters the diff and cannot ride back. The worker running the
  installer at all is a separate concern, handled by the `claude install` write-class
  refusal (ADR-006 D2, not ride-back).
- **v1 import requires a quiescent coordination branch.** Stationary-head import
  refuses on any HEAD mover — external *or* the orchestrator's own batch (υ). A live
  main mandates delta-branch coordination (ADR-006 D8 team mode); solo-on-main dispatch
  is safe only when main is quiescent. Even with no external committer, v1 lands **one
  worker per base** (not a whole batch): importing+committing worker A moves HEAD
  `B→B+1`, so worker B — also forked at `B` — refuses `head-moved`. Parallel
  *execution* is first-class; parallel *landing* is not. The orchestrator detects the
  moved HEAD and reports the mover rather than silently re-dispatching into livelock;
  the in-verb re-anchor is deferred (IMP-043).
- **Smell test is not a guarantee.** `check-allowlist` green never means the fork is
  clean; only `select_copies` guarantees tier exclusion. The two must not be conflated.
- **Normalizer fidelity is load-bearing.** A drift from the frozen byte-for-byte
  algorithm would silently mis-anchor memory; the version tags are the seam that makes
  a deliberate change visible rather than silent.

## Hypotheses

- **Mechanism in a verb, not in prose.** Moving each funnel step into a CLI verb makes
  the worktree/dispatch skills shorter **and** more harness-agnostic in one edit, and
  makes trust golden-testable instead of resting on a prose ritual an LLM may skip
  (ADR-011 context).
- **Identity rides disk, not an env seam.** A disk marker is the harness-agnostic floor
  because disk is the one medium every harness has; an env channel is not (claude's
  `Agent` tool has none). `DOCTRINE_WORKER` is an optimisation of the marker, never the
  identity.
  > **AMENDED — FALSIFIED (SL-254, 2026-08-14).** Inverted. `DOCTRINE_WORKER` **is** the
  > identity and the marker is gone (`DEC-207`). The hypothesis' premise — that no env
  > channel reaches claude — was true only of the in-session `Agent` tool; once every
  > harness is spawned as a confined subprocess, the env channel is universal, and it is
  > strictly better than disk: it describes the **process** (so it cannot go stale, be
  > self-cleared, or be inherited by an unrelated tree) and it is set by the same argv
  > that establishes the write floor.
- **Fail-closed on ambiguity.** A linked worktree with no marker is refused, not
  trusted — so a stamp-failure or a self-clear *loses* privilege rather than gaining it.
  > **AMENDED — FALSIFIED (SL-254, 2026-08-14).** There is no ambiguity left to fail
  > closed on: one signal, present or absent, and neither a stamp-failure nor a
  > self-clear exists to be defended against (`DEC-207`). The fail-closed instinct
  > survives one level down, at spawn: a missing confinement backend refuses by name
  > rather than spawning unconfined (`DEC-208`).
- **Exclude by construction, not by trust.** Withholding the coordination/runtime tier
  from the fork outright is preferred over copying it and trusting workers not to mutate
  it — the tier's *absence* is what makes worker-sole-writer free.
- **Stationarity over merge-base.** Asserting HEAD has not moved from the captured base
  is cheaper and more direct than a merge-base, and sufficient because the batch commits
  exactly once onto `B`.
- **One impure git module.** Concentrating all git/disk/process impurity in `git.rs`
  keeps the rest of the container testable as pure functions, honouring the system-wide
  pure/imperative split.

## Decisions

- **D1 — provision is the sole copier, exclusion is a guarantee not a check.**
  `select_copies` drops the coordination/runtime tier even under a broad allowlist;
  `check-allowlist` is a static smell test whose green result is explicitly not
  completeness.
- **D2 — the branch-point check is ref-equality, not merge-base.** It asserts
  coordination HEAD still equals the pre-spawn base `B`; a mismatch means an external
  mover ⇒ re-dispatch, never auto-merge.
- **D3 — worker-mode is enforced in the CLI by a disk-marker-primary, fail-closed
  guard.**
  > **AMENDED — FALSIFIED (SL-254, 2026-08-14).** Superseded by:
  > `worker_mode := env DOCTRINE_WORKER == "1"`, and nothing else (`DEC-207`). The
  > marker file, the `is_linked_worktree` conjunct, the fail-closed-on-marker-absent
  > rule, the `Hook-mint` class, and the marker-minting verb's identity exemption are
  > **all gone** — the two fail-opens the fail-closed rule existed to close (the
  > `SubagentStart` stamp-failure and the deliberate self-clear) cannot occur when
  > identity is a property of the process. What survives verbatim: the guard sits in
  > `run()` before dispatch, refuses `Write`- and `Orchestrator`-classed verbs by name,
  > and `write_class` is still a wildcard-free exhaustive match. The historical decision
  > is retained below for the record.

  *(Historical body.)* `worker_mode = (is_linked_worktree && marker_present) OR env
  DOCTRINE_WORKER`; the marker is the harness-agnostic primary, env a codex/pi
  worker-on-main optimisation. A **marker-absent linked worktree is fail-CLOSED** — the
  write/`Orchestrator`/`Hook-mint` classes are refused there (closing both the
  SubagentStart stamp-failure case and the marker self-clear). `write_class` is a
  wildcard-free exhaustive match (a new verb is a compile error); the marker-minting
  verb is exempt by verb identity, not location. (Rewritten from the prior
  env-fails-open framing — ADR-006 D2a / ADR-011 D1, SL-056 G2.)
- **D4 — the born frame is impure-isolated and normalizer-versioned.** All git
  impurity lives in `git.rs`; `forget.remote.v1`/`forget.checkout.v1` are reproduced
  byte-for-byte and tagged so the algorithm is replaceable without silent re-anchoring.
- **D5 — the funnel is an enforced verb family, not a prose discipline.** `fork`,
  `import`, `land`, `gc` are `Orchestrator`-classed CLI verbs (refused under
  `worker_mode`) that carry creation, the dispatch funnel, the solo merge, and reaping
  — each a pure classifier (`classify_import`/`classify_land`/`classify_gc`) over an
  impure git shell. The trust-bearing core is **create + provision + confine**
  (amended SL-254 — it read "create-or-mark + provision + marker + per-wt env
  *emission*"; the mark/marker legs went with `DEC-207` and the env emission with the
  per-worktree `CARGO_TARGET_DIR` redirect), and it is harness-identical and
  golden-testable (ADR-011 D2/D5).
- **D6 — per-harness altitude is a uniform contract with honest non-uniform reach.**
  > **AMENDED — FALSIFIED (SL-254, 2026-08-14).** The altitude axis is abolished, not
  > levelled up: there is one confined subprocess arm for every harness (`DEC-208`,
  > `DEC-217`), so every harness reaches the same floor — explicit base-pinning,
  > pre-dispatch baseline-verify, `DOCTRINE_WORKER` identity, kernel-level confinement,
  > and no unconfined fallback. Two of this decision's clauses invert rather than
  > merely lapse: the claude concessions (`O3-red` SubagentStart stamp, late-only
  > baseline verify, opaque base) are **dissolved**, and "no harness-specific command
  > is a required element" is **no longer true** — `claude -p --output-format
  > stream-json` is a required element of the shipped spawn path, contained in one
  > script. The historical decision is retained below for the record.

  *(Historical body.)* codex/pi reach the full mechanism floor (explicit base-pinning,
  env-arm, per-wt env
  delivery, pre-dispatch baseline-verify, bwrap); claude reaches an **O3-red
  SubagentStart-stamp** altitude — marker-only, **not fail-closable** (contained by the
  marker-absent rule), **no pre-dispatch baseline-verify** (caught late at import), and
  an **opaque Claude-chosen base** (a confessed residual, not parity — IMP-043). No
  harness-specific command is a required element (ADR-011 D3/D5/D6).
- **D7 — the funnel's honest scope: belt narrow, solo non-squash, import quiescent.**
  (a) The `.doctrine/`/`.claude/` belt is the import/dispatch-path containment only —
  and it is exactly those **two hard-coded floors**, plus the `--slice`-scoped
  `undeclared-scope` check; it never consulted the configurable `worker-forbidden-writes`
  list (amended SL-254). The
  `.claude/` leg contains exactly **force-add injection** (the rest of `.claude/` is
  gitignored and invisible to the tracked-files diff); solo's `land` is a second,
  beltless sanctioned funnel. (b) Solo **must** land via the **structurally non-squash**
  `land` (`git merge --no-ff`) so `gc`'s ancestry leg and memory-anchor sha-stability
  both hold — a squash-merge is structurally uncertifiable. (c) v1 `import` requires a
  **quiescent** coordination branch and lands **one worker per base**; a live main
  mandates delta-branch coordination (ADR-006 D8).
