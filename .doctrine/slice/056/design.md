# SL-056 Design — Orchestrator spawn seam: worktree mechanism into CLI verbs

> Clean redesign (8th-inquisition target). The round-1→7 reasoning trail lives in
> `design-history.md`; the per-charge dispositions in `inquisition-1.md`…`-7.md`.
> This document states **what we build**, not how we argued to it. Scope:
> `slice-056.md`. Evidence base: `.doctrine/slice/055/research/worktree-orchestration.md`
> (shared with sibling SL-055).

## 1. Thesis

**Mechanism belongs in the CLI verb; judgment and harness concessions belong in
prose.** The worktree/dispatch creation ladder, the import funnel, the solo land
merge, build isolation, and the worker-mode guard move out of fail-open skill prose
into fail-closed, golden-testable CLI verbs — identical under claude/codex/pi by
construction — with an **orchestrator-owned fork + a disk marker as the
harness-agnostic keystone.**

This is the **pure/imperative wall lifted to the orchestration layer.** The binary
is the pure mechanism core; the harness spawn — a subprocess for codex/pi, the
`Agent` tool for claude — is the thin impure shell. Every decision below applies
that wall.

## 2. Decision tree

```mermaid
flowchart TD
    A[work to land] --> B{mode?}
    B -->|solo /execute| C{isolated worktree?}
    C -->|in-place on trunk| C1[commit directly · no marker · no land]
    C -->|isolated worktree| C2[land --no-ff → gc]
    B -->|/dispatch| D{harness?}
    D -->|claude · /dispatch-agent| E[Agent tool spawns subagent\nsubagent_type = dispatch-worker]
    E --> E1[WorktreeCreate hook OWNS creation\ngit worktree add + provision + stamp marker\nfail-closed · gated on DISPATCH_WORKER_AGENT_TYPE\npost-add failure → rollback → no orphan]
    E1 --> F
    D -->|codex/pi · /dispatch-subprocess| G[fork --worker\ncreates + stamps marker + provisions]
    G --> G1[spawn subprocess\ncwd-bound + DOCTRINE_WORKER=1 + per-wt env]
    G1 --> F
    F[worker edits SOURCE only] --> H[import · .doctrine/ belt · apply --3way]
    H --> I[verify → branch-point guard → one commit] --> J[gc]
```

The harness axis splits **only the spawn shell** (§4); the cadence after a worker
produces its delta — `import → verify → branch-point → one commit → gc` — is the
identical CLI verb sequence for both (§7, the slice's whole payoff). Solo bypasses
`import` (it lands a multi-commit branch via `land`, §6).

## 3. Worker identity — disk marker primary

Worker-mode is a property of the **worker**, signalled by a **disk marker the
trusted orchestrator stamps before the worker runs.** Disk is the one identity
substrate *every* harness has; an env seam is not (claude's `Agent` tool has none,
and `claude -p` is API-billed + harness-specific — rejected).
[[mem.pattern.dispatch.spawn-backend-harness-agnostic-no-free-env-seam]] binds this
floor.

```
marker path:  <root>/.doctrine/state/dispatch/worker      (withheld runtime tier)
worker_mode(root) := (is_linked_worktree(root) && marker_present(root))  // PRIMARY, agnostic
                     OR env DOCTRINE_WORKER set                          // codex/pi worker-on-main catch
guard (in run(), before dispatching a write-classed OR Orchestrator Command):
    if worker_mode(root): refuse(verb)   // names the verb
```

- **Marker is primary and harness-agnostic.** Present in a linked worktree ⇒ writes
  refused. Presence-only, no contents.
- **`DOCTRINE_WORKER=1` env is a codex/pi *optimisation*, not the identity.** Its one
  job: catch the **worker-on-main** hazard (ADR-006 D2b — the harness drops the worker
  on the coordination root, where no marker exists and `is_linked_worktree` is false).
  Available only where a subprocess spawn carries env (codex/pi). For **claude** it is
  unavailable, so worker-on-main stays the deferred D2b residual, mitigated by
  always-isolating the worker + the hook-stamped marker.
- Solo `/execute` (in-place or isolated) sets **neither** signal → writes freely.
  **Mode, not location, decides** (ADR-006 D6a). `is_linked_worktree` is the existing
  predicate (memory squash-warn, RV-verb refusal — now a third consumer).
- **Lifecycle (owned):** written by `fork --worker` (codex/pi) or the WorktreeCreate
  hook (claude); removed by `gc`; rolled back if `fork` fails; cleared by `marker
  --clear` for a stray marker (below). A tree may become a coordination/direct-writer
  root only after an **assert-marker-absent** check that, on a stray marker, refuses
  and **names the remedy** (`marker --clear`) — detection carries a cure. This gates
  *every* transition of a linked worktree into a direct-writer role, **solo
  `/execute` included** (D6a makes solo a full self-orchestrator in a linked worktree).
- **`marker --clear` (bespoke class — see §5).** Removes the marker at the cwd tree
  root, loud receipt. Refused if `DOCTRINE_WORKER` set, if cwd is not the marker's
  tree root, and — when the cwd tree is a **linked worktree** — unless `--operator` is
  passed (accident-fence). **Never** refused by the marker conjunct itself (locking the
  marker's only remover behind the marker is the self-brick we reject).
- **Observability (required):** `doctrine worktree` status prints `worker fork: yes —
  writes refused; signal: env|marker`, so the mode is discoverable without knowing the
  gitignored path.
- **Withheld tier:** `.doctrine/state/**` is already gitignored, provision-dropped, and
  absent from the import delta — the marker inherits every exclusion with zero new tier
  logic (confirm in the `is_withheld` test).

**Env blast-radius bound.** `DOCTRINE_WORKER` is set **only in the spawned child's
env**, never `export`ed into the orchestrator's shell; the orchestrator never sets it
on itself. When the **env disjunct** trips `worker_mode` on a tree that is **not a
linked worktree** (main, a plain checkout — where a real worker fork never is), the
state is provably either a worker-on-main *or* a leak. Every verb refused this way —
authoring (`slice new`/`design`/`plan`) **and** `Orchestrator` funnel verbs — carries
a **named dual-cause** message ("`DOCTRINE_WORKER` set outside a worker worktree: a
worker was dropped on the coordination root → re-dispatch isolated; **or** the env
leaked into this process → unset it"), never a bare "worker refused."

## 4. Per-harness spawn

The mechanism/concession line falls between **what the binary does** (create-or-mark
+ provision + per-wt env *contract* emission — harness-identical) and **how the
worker is spawned** (harness-shaped → prose, selected by the `/dispatch-*` router).

### 4a. codex/pi — `/dispatch-subprocess`

`doctrine worktree fork --base <B> --branch <name> --dir <path> --worker` creates the
worktree, stamps the marker, provisions, emits the per-wt env contract on stdout. The
orchestrator then spawns the subprocess **with its cwd bound to the fork**:

```sh
fork_env="$(doctrine worktree fork --base "$B" --branch "$BR" --dir "$D" --worker)" \
  || { echo "fork failed: $?" >&2; exit 1; }      # halt, do NOT spawn
env -C "$D" DOCTRINE_WORKER=1 $fork_env codex exec "<pre-distilled prompt>"
#       ^ cwd→fork    ^ worker-on-main optim   ^ per-wt env    ^ harness-shaped line
```

- **`env -C "$D"` binds the worker process cwd to the fork.** Without it `codex exec`
  inherits the orchestrator's cwd (the coordination root it ran `fork` from) and the
  worker's *source* edits land on the trusted branch — bypassing `import`, the belt,
  and branch-point discipline; the `DOCTRINE_WORKER` guard catches only
  doctrine-mediated writes, never a raw editor write to cwd. The cwd-bind is a
  **spawn-shell mechanism**, not a prompt instruction. Portable fallback if `env -C` is
  absent: `( cd "$D" && exec env DOCTRINE_WORKER=1 $fork_env codex exec … )`. Under D6,
  `bwrap … --chdir "$D"` is the confined equivalent.
- **Capture + check `$?`; never `eval "$(…)"`** — `eval` swallows the exit status, a
  fail-open trap. `$fork_env` is the stdout env block; status went to stderr.

`fork` steps (deterministic, harness-identical) — **compensating cleanup, not a true
transaction:** git mutations are not atomic, so any failure after step 1 triggers a
best-effort rollback (`git worktree remove --force`, `git branch -D`, reap the dir);
a rollback that itself fails **names the leftover and exits non-zero** — never a
silent half-rollback:
1. `git worktree add -b <branch> <dir> <B>` (correct syntax: `-b <branch>` for a new
   branch at `<B>`). Refuses if `<dir>`/`<branch>` exist or `<B>` is not a commit.
   `<dir>` must be **unique per branch** and outside the repo root or a gitignored
   in-repo path (else a concurrent same-slice batch collides / dirties the tree).
2. `doctrine worktree provision <dir>` (existing sole-copier; withheld tier excluded).
3. If `--worker`: write the marker **before** any spawn window. Solo omits `--worker`.
4. Emit the **per-worktree env contract** on stdout (`KEY=value` per line); human
   status to stderr. The contract is *generalisable* — the project declares its per-wt
   env; doctrine-the-repo declares `CARGO_TARGET_DIR=<jail-root>/wt/<branch>` (§8, a
   project-local consumer, not a framework primitive).

### 4b. claude — `/dispatch-agent`

No `fork` verb, no env seam. The orchestrator launches the worker via the `Agent` tool
with `subagent_type: dispatch-worker` and `isolation: worktree`. Claude Code fires a
**`WorktreeCreate` hook** which, per the harness docs, **replaces the default git
worktree creation**: a command hook **prints the worktree path on stdout**, and **any
non-zero exit fails creation.** So the orchestrator-installed hook *is* the claude
analog of `fork` — it owns creation, provisioning, and stamping in one trusted act:

```
WorktreeCreate hook command:  doctrine worktree create-fork   (reads payload JSON on stdin → prints worktree path on stdout)
    parse stdin JSON                                 # TRUST BOUNDARY — harness-facing untrusted input
        malformed JSON                  → refuse bad-payload          (non-zero, creation FAILS)
        missing agent_type              → refuse missing-agent-type   (NEVER silently "benign")
        missing/empty cwd               → refuse missing-cwd
        <dir> underivable or escapes    → refuse bad-dir              (path-traversal / not under repo)
    if payload.agent_type == DISPATCH_WORKER_AGENT_TYPE:   # τ: ONE binary const, not a literal
        git worktree add -b <branch> <dir> <HEAD>    # base = session HEAD (= B under stationary-head, §7c)
        doctrine worktree provision <dir>            # ADR-006 D9 allowlist (withheld tier excluded)
        write_marker(<dir>)                          # stamp the disk marker
        ── on ANY failure after `git worktree add` (ρ): COMPENSATING ROLLBACK before exit ──
            git worktree remove --force <dir>; git branch -D <branch>; reap <dir>
            rollback clean      → refuse provision-failed | stamp-failed   (non-zero, NO orphan)
            rollback half-fails → refuse orphan-leftover  (distinct token, names dir+branch left, non-zero)
        print <dir>                                  # required output
    else:                                            # benign isolation:worktree / --worktree launch
        create a SERVICEABLE default worktree by DOCTRINE's own naming/path conventions,
        print path, NO marker        # σ: suffice for general isolated-subagent duty;
                                      #    NOT a byte-for-byte mirror of Claude's native layout
    any post-add failure rolls back FIRST → non-zero exit → creation FAILS → no worktree → no worker  (FAIL-CLOSED)
```

- **Fail-closed by construction (resolves SR-1).** A claude `Agent` worktree is
  harness-born, **not** `fork`-provisioned — so provisioning (D9) and stamping must come
  from a hook. Putting them *inside creation* means a worker **cannot exist
  unprovisioned or unstamped**: a provision/stamp failure fails the worktree, not just
  the guard. This is strictly stronger than stamping *after* creation (a post-creation
  stamp that dies leaves an unstamped worker writing freely — fail-open). The claude
  path now mirrors codex/pi `fork --worker`: one trusted act creates + provisions +
  marks. ([[mem.pattern.dispatch.claude-agent-worktree-not-fork-provisioned]])
- **Compensating cleanup, not a transaction (ρ — mirrors §4a `fork`).** The hook
  *itself* runs `git worktree add`, so it is the creator and therefore the rollback
  owner — "any failure → no worktree" is honest only if `create-fork` reaps what it
  made. git mutations are not atomic, so **any failure after `git worktree add`
  succeeds** (a `provision` or `write_marker` death) triggers a best-effort rollback —
  `git worktree remove --force` (a provisioned fork is dirty; plain `remove` refuses),
  `git branch -D`, reap the dir — **before** the non-zero exit, so Claude's honoured
  "creation failed" leaves **no orphan worktree+branch** behind. The rollback is itself
  fallible: a clean rollback refuses `provision-failed`/`stamp-failed`; a rollback that
  half-fails refuses with a **distinct `orphan-leftover`** token that **names** the dir
  and branch left on disk and exits non-zero — never a silent half-rollback. Without
  this, a post-`add` failure is fail-**closed against spawning** (Claude honours the
  exit) but fail-**open against disk hygiene** — exactly the orphan that later seeds
  `land`'s `worktree-gone` and `gc`'s idempotence with debris no verb owns. `create-fork`
  reuses `run_fork`'s rollback core (§11) — one cleanup implementation, two callers.
- **Bad-payload refusals (ψ — the stdin JSON is a trust boundary).** `create-fork`
  parses harness-supplied JSON and **derives `<dir>` from `payload.cwd`** — untrusted
  input — so the bad path is **fail-closed**: malformed JSON → `bad-payload`; missing
  `agent_type` → **`missing-agent-type`** (it **REFUSES** — it must NEVER silently fall
  to the else-branch and mean "benign/replicate-default": that is τ/σ's fail-open by
  another road); missing-or-empty `cwd` → `missing-cwd`; a `<dir>` that fails to derive
  or escapes the repo → `bad-dir`. Each is a distinct non-zero exit that **fails
  creation** — a worker the hook cannot classify must not be born. Goldens in §12.
- **`agent_type` is the discriminator — one source of truth (τ).** The literal
  `"dispatch-worker"` is replicated across the `Agent` `subagent_type`, the hook gate,
  `install/agents/claude/dispatch-worker.md`, and the `/dispatch-agent` skill — and
  **drift fails OPEN**: a one-character mismatch sends `create-fork` to the else-branch,
  which writes **no marker**, so the worker is born with `marker_present == false` ⇒
  `worker_mode == false` ⇒ writes **not** refused — an unbranded free-writer on the one
  harness with no env leg and no bwrap (§4c). So a single **`const
  DISPATCH_WORKER_AGENT_TYPE`** in the binary is the source of truth `classify_create`
  reads; the installed agent-def `name` and the skill's `subagent_type` are **pinned**
  to it; a cross-surface **drift test REDS on mismatch** (§12). The const is the gate's
  only key — never a free-floating literal in four files.
- **Replicate-default else-branch — every worktree, not just dispatch (σ).** Because the
  hook **replaces default creation for *every* worktree** (incl. `--worktree` launches and
  benign isolation:worktree subagents), the non-`dispatch-worker` else-branch **must still
  create a worktree** — the hook now **intermediates creation for ALL isolated subagents**
  (§10/G3 confesses this blast radius). Two outcomes, **decided by the O3 spike's gating
  matcher question (§9/§12), not a sub-question**:
  - **Matcher available** → scope the hook to `dispatch-worker`; the else-branch is
    **DELETED** and σ evaporates — no replica to maintain.
  - **No matcher** → the else-branch creates a **SERVICEABLE default worktree using
    doctrine's own naming/path conventions**. It **MUST NOT** attempt to mirror Claude's
    native layout (`worktree-agent-<id>` / `.git/worktrees/agent-<id>` /
    `.claude/worktrees/agent-<id>`) byte-for-byte — the bar is "**suffice for general
    isolated-subagent duty**," NOT fidelity to Claude (whose default isn't that great —
    explicit user steer). The golden asserts the produced worktree is **valid + usable +
    bears NO marker** — never byte-equality with Claude's native act.
- **Identity = the disk marker.** No env, no arm sentinel, no lease, no serial
  constraint. Each WorktreeCreate fires independently for its own worktree ⇒
  **concurrent file-disjoint claude dispatch is first-class *in execution* (SR-2).**
  **v1 buys parallel EXECUTION, not parallel LANDING (υ).** The funnel-back is serialized
  by `import`'s stationary-head precond (§7c — one orchestrator, `HEAD == B`): the
  orchestrator's own sequential imports bump HEAD `B→B+1` after each landing, so the next
  sibling (forked at B) then hits `head-moved` (§7a) and must **re-dispatch onto the
  bumped base** — in-verb re-anchor is deferred (§13, IMP-043). So a concurrent batch of
  N workers lands the **first**, and the N−1 siblings re-dispatch — **one landing per
  base**, not an orderly N-drain. "Serialized" here is first-wins-rest-reanchor, NOT a
  batch serialized to completion (§7c extends the head-moved framing to the
  orchestrator's own batch-imports, not only external committers).
  `DOCTRINE_WORKER` and bwrap are unavailable; the worker shares the jail-wide build
  target (§8); worker-on-main is the deferred D2b residual.
- **Spike-gated (§9/§12) — the decision hinges on one probe, and it conflicts with our
  own.** The earlier probe saw `name`, **not `agent_type`**, on WorktreeCreate — **but
  it spawned an *unnamed* subagent (no custom agent defined)**, so agent_type absence is
  *expected*, not a refutation; the docs say agent_type is present "when the hook fires
  inside a subagent." The O3 spike must confirm a **named `dispatch-worker` subagent
  reliably propagates `agent_type` through WorktreeCreate**, that the hook fires before
  the worker, the payload shape, and — as a **hard gating outcome (σ)** — whether a hook
  **matcher** can scope it to `dispatch-worker` (matcher-green deletes the
  replicate-default else-branch). **Fallback
  ladder if agent_type is absent from WorktreeCreate:** (1) **SubagentStart-stamp** — let
  Claude create the worktree, provision + stamp at SubagentStart (which the probe *did*
  confirm carries `agent_type`, `cwd`, race-free per subagent
  [[mem.pattern.dispatch.claude-subagentstart-worker-identity]]), accepting the
  **fail-open created-but-unstamped window**; (2) prompt-enforced worker-sole-writer.
  Named, altitude-table-confessed (§10), symmetric with the D6 bwrap back-out.
- **Base-pinning unchanged.** No base param reaches the hook, so it forks from session
  HEAD (= B under stationary-head §7c) — implicit for claude, as before; WorktreeCreate
  does not make it explicit.
- **`create-fork` privilege.** The hook runs in the orchestrator's session (process cwd
  = orchestrator root, `worker_mode` false), reads `payload.cwd`/derives `<dir>`, and
  **mints** the marker — like `fork --worker`, never self-refused (§5).

### 4c. claude self-clear residual (confessed, not closed)

The marker is a file *inside* the worker's worktree and the env is shell-strippable, so
a **non-compliant** worker of any harness can `rm` the marker or `env -u
DOCTRINE_WORKER` directly. The `--operator`/env-lock hardening on `marker --clear`
fences **accidents, not malice, on every harness.** For claude there is **no mechanism
floor against a deliberate self-clear** (no env-lock; bwrap is codex/pi-only) until a
free env seam or OS confinement (IDE-004 channels / userns-bwrap) lands. claude
worker-sole-writer is therefore **accident-fenced + prompt-enforced, not malice-proof**
— stated plainly in the §10 per-harness altitude table. The malice containment on the
**dispatch funnel** is the import `.doctrine/`-rejection belt (§7), not the marker.

## 5. Privilege classes

`write_class` (exhaustive over `Command`) is **unchanged** (behaviour-preserving). Two
classes join the worker-mode guard, one stays open, one is bespoke:

| Class | Members | Refused under `worker_mode`? |
|---|---|---|
| **Orchestrator** | `fork`, `import`, `land`, `gc` | **Yes** — they mutate git refs/dirs (create/remove worktrees, delete branches, merge commits, reap dirs). Classifying them `Read` because they spare the authored TOML corpus is a category error (a worker could delete branches, violating ADR-006 D2). |
| **write** | authoring writes (`slice new`/`design`/`plan`/`memory record`/status-transition) | Yes (unchanged) |
| **Read** | `provision`, `check-allowlist`, `branch-point-check`, status | No — open to workers |
| **`marker --clear`** | bespoke 4th class (§3) | **No** — locking the marker's only remover behind the marker is the self-brick. Refused instead by env-set, cwd-not-tree-root, and the `--operator` accident-fence in a linked worktree. |

The claude `create-fork` hook verb (§4b; and its `marker --stamp-subagent` fallback)
**mints** the marker; it runs in the orchestrator's session at the coordination root
(worker_mode false there) targeting the payload-derived worktree dir, so it is never
self-refused — the same posture as `fork --worker`'s marker write.

## 6. `land` — solo `/execute`'s coordination merge

Solo's analog of dispatch's `import`: a fail-closed verb that lands a solo
isolated-worktree TDD branch onto the coordination branch, **structurally non-squash**,
so gc's ancestry leg (§6.1) and memory-anchor sha-stability (ADR-006 D8) both hold.
`Orchestrator`-classed. **Solo-only** — dispatch uses `import` (single distilled
commit, ancestry severed); `land` is for solo's multi-commit branch (ancestry
preserved). The in-place solo path needs no `land` (it commits directly).

```
doctrine worktree land --fork <branch>      # runs at the coordination root
```

1. **precond** — tree clean (`git status --porcelain --untracked-files=no`-empty, same
   scoping as `import`) else `tree-unclean`; HEAD is the coordination branch;
   `<branch>` exists else `no-such-fork`; `<branch>`'s **live linked worktree** does
   *not* bear the worker marker else `dispatch-fork` (a marker-bearing fork is a
   dispatch worker — its delta must funnel through the belted `import`, never `land`'s
   beltless merge); **and `<branch>` must *have* a live linked worktree** — the marker
   is uncommitted and unreachable once the worktree is gone, so a worktree-less branch
   would pass `dispatch-fork` *vacuously* → refuse **`worktree-gone`** ("cannot verify
   it is not a dispatch fork; re-create the worktree, route through `import`, or
   `--force` knowingly"). The marker guard is honestly a **live-worktree
   accident-fence**, not a universal provenance proof.
2. `git merge --no-ff <branch>` — **never `--squash`** (the verb cannot express one).
   Ancestry preserved ⇒ fork commits reachable ⇒ gc's ancestry leg reaps.
3. on conflict → **`git merge --abort` first** (restore the clean tree step 1 demands),
   *then* refuse `merge-conflict`, report + halt — never auto-resolve. `git merge`
   mutates the index/tree and sets `MERGE_HEAD` *before* reporting the conflict, so the
   half-merge **must** be aborted else it wedges the tree against the verb's own
   re-entry guard. The abort is itself fallible and guarded to fire **only mid-merge**
   (`MERGE_HEAD` present): step 3 reached with no merge in progress → refuse
   **`inconsistent-merge-state`** (never a silent abort masquerading as a clean
   conflict). Abort success → ordinary `merge-conflict`, tree guaranteed clean. Abort
   **failure** → a **distinct non-zero `wedged-merge`** naming `MERGE_HEAD`, the
   unmerged paths, that the tree is **not** clean, and the manual remedy.

**Refusal set:** `{tree-unclean, no-such-fork, dispatch-fork, worktree-gone,
merge-conflict, wedged-merge, inconsistent-merge-state}`.

Pure core: `classify_land(tree_status, head, fork_state) -> Result<Merge, Refusal>`
where `fork_state = {exists, has_live_worktree, bears_marker}`; imperative shell drives
`git merge --no-ff` and the mid-merge-guarded abort. Reuses the tree-clean check shared
with `import` (no parallel implementation).

## 7. The funnel belt + `import`

### 7a. `import` (dispatch funnel)

```
doctrine worktree import --base <B> --fork <branch>      # runs at coordination root
```

`Orchestrator`-classed. **Dispatch path only** (a single distilled worker commit,
ancestry severed); solo uses `land`. **v1 is the stationary-head case only**, each step
a hard refusal (no auto-merge):
1. **precond — two guards:** `HEAD == B` (`branch-point-check`, a ref-equality compare)
   **and** the tree is clean (`git status --porcelain --untracked-files=no`-empty —
   tracked + staged only). Untracked files are **excluded deliberately** (benign
   scratch/memory/withheld sheets must not false-`tree-unclean` the common case).
   `HEAD != B` → `head-moved`; dirty → `tree-unclean`.
2. `S^ == B` (single-non-merge fork delta) else `multi-commit`.
3. **belt:** reject if the `B..S` **name-only** diff touches any `.doctrine/` path
   (prefix-match, tracked files only) else `doctrine-touch`. A forced-added marker is
   caught too (defense in depth).
4. `git apply --3way --index` (non-committing). Under **both** preconds the patch
   applies onto the exact tree it was cut from ⇒ cannot conflict ⇒ `apply-conflict` is
   **not** a v1 refusal. The orchestrator commits **separately** (ADR-006 D7 cadence;
   import ≠ commit). **No runtime receipt is stamped** — a flag born before the commit,
   in the gitignored tier, survives a crash and lies "landed" to `gc`; instead `gc`
   derives landed-ness from durable git state (§6.1).

**Refusal set (v1):** `{head-moved, tree-unclean, multi-commit, doctrine-touch}`.

Pure core: `classify_import(diff, base, head) -> Result<Apply, Refusal>`; imperative
shell drives git + apply.

### 7b. Belt scope (honest)

The `.doctrine/`-rejection belt is the **dispatch/import-path** containment — a
dispatch worker's doctrine delta never funnels back through `import`. It is **not** an
unconditional all-funnel containment: solo's `land` (§6) is a second, **beltless**
sanctioned funnel — solo is a trusted self-orchestrator that *legitimately* lands
doctrine, so a belt there is a category error. The belt's true scope: the
import/dispatch path, conditioned on dispatch deltas routing through `import` and never
`land` — mechanised by `land`'s `dispatch-fork` + `worktree-gone` guards (§6).

### 7c. Quiescence constraint

Stationary-head v1 import **requires a coordination branch with no concurrent HEAD
movers — external *or* the orchestrator's own batch (υ).** Two invalidation sources, one
mechanism:
- **External committers.** On a live main, each external commit moves HEAD to `B+1` and
  forces every in-flight worker's import to refuse `head-moved` → re-dispatch →
  re-invalidated — **livelock**. The constraint: **a live main mandates delta-branch
  coordination (ADR-006 D8 team mode)**; solo-on-main dispatch is safe only when main is
  quiescent for the run.
- **The orchestrator's own concurrent-batch imports (υ — the same invalidation).** Even
  with no external committer, a concurrent batch self-invalidates: importing worker A,
  then committing (§7a step 4), moves HEAD `B→B+1`; worker B — **also forked at B** — then
  reads `HEAD != B` and refuses `head-moved`, re-dispatching onto the bumped base. This is
  why v1 lands **one worker per base**, not a whole batch (§4b): parallel execution, not
  parallel landing.

In both cases the orchestrator **detects** a moved coordination HEAD via the branch-point
guard and **reports the mover** (external committer named; own-batch advance acknowledged)
rather than silently re-dispatching into livelock. The in-verb re-anchor (and/or a single
multi-fork import) is the real fix — deferred (§13, IMP-043).

## 8. `gc`

```
doctrine worktree gc --fork <branch> [--superseded-head <SHA>] [--force] [--dry-run]
```

`Orchestrator`-classed. Reaps, in one act: (1) `git worktree remove` the fork dir
(removing its marker); (2) `git branch -D` the fork branch (never a git-ancestor, so
`-d` always refuses — the patch-id gate, not `-d`, is the safety); (3) reap the
`wt/<branch>` target dir (closes the §8 disk loop); (4) warn (stderr) that
`env!(CARGO_MANIFEST_DIR)`-baked test binaries need recompile.

**Ordering is forced:** `git branch -D` refuses a branch checked out in a live
worktree, so the worktree must go first. The crash window between (1) and (2) — *branch
alive, worktree gone, marker unreachable* — is **intrinsic to git**, closed downstream
by `land`'s `worktree-gone` refusal (§6) and within `gc` by idempotent rerun (§8.2).

### 8.1. The "landed" oracle — durable patch-id, no receipt

`gc` deletes **only** when the fork's commit has *provably landed*, tested against
durable git state (not a runtime flag). `--merged` is wrong (apply-funnel branch is
never a git-ancestor); delta-emptiness is also unsound (`git diff B..fork` is never
empty for real work; `diff HEAD..fork` false-diverges when a sibling moves HEAD). v1
uses **two legs, union** via `git cherry <coordination-HEAD> <fork-branch>`:
- **ancestry** — `<fork-tip>` is an ancestor of `<coordination-HEAD>` (the `land`
  route: fork commits reachable), **OR**
- **patch-id** — every commit `git cherry` lists is `-` (the `import` route: ancestry
  severed, but each commit's *patch* landed).

A non-ancestor tip with any `+` ⇒ not (fully) landed ⇒ refuse unless
`--superseded-head`/`--force`. **A squash-merge is structurally uncertifiable** (it
destroys both ancestry and per-commit patch-id) — so solo **must** land via the
non-squash `land` verb; a manually squash-merged fork trips neither leg and gc refuses
with a **named** message ("cannot certify a squash-merge — re-land via `worktree land`
(--no-ff), or `--force` knowingly"). **Crash-proof:** a crash between apply and commit
leaves no commit ⇒ `git cherry` reports `+` ⇒ gc refuses (a receipt would have lied
"landed" and reaped the only copy).

**Superseded forks — `--superseded-head <SHA>`, no stored flag.** Moved-HEAD
re-dispatch is the common case: a re-dispatched fork is spent yet never landed (`+`) →
bare gc would demand `--force`, training the reflex the oracle exists to kill.
`--superseded-head <SHA>` reaps **iff** `<SHA>` equals the branch's current head — an
**operator assertion** that this exact, still-current commit is spent-and-abandoned
(the head-match is a TOCTOU movement-guard, **not** a landing proof). Fail-safe both
ways: a lost SHA only costs a `--force` (gc refuses — the safe side); a wrong SHA
cannot match a live head unless it *is* that head. No stored record ⇒ no removal owner
to forget, no branch-name key to false-match.

**Observability:** `gc --fork <b>` / `--dry-run` prints the per-fork verdict ("`<b>`:
landed ✓ / not-landed — `--force` to reap"), computed from git, so the operator never
`--force`s blind.

### 8.2. `gc` is an idempotent state machine

`gc` that crashed between any two destructive steps **completes on rerun or names the
leftover** — never strands its own debris. Pure `classify_gc(state) -> GcPlan` over
impure-gathered `{branch_exists, worktree_present, target_present, landed_verdict?}`:
- **Gate runs only while the branch lives.** The oracle requires B; it is monotone and
  re-runnable (a landed fork stays landed), so a rerun re-certifies and resumes.
- **Branch-gone ⇒ the deletion *is* the certificate.** A fork branch is deleted only
  through `branch -D` after the gate passed, so branch-absent ⇒ already-certified. The
  only branch-gone residue is **T** (the `wt/<branch>` target dir) — derived build
  cache, never the only copy, path a pure function of the branch *name*
  (`target_dir_for_branch`), so a rerun reaps it from `--fork <branch>` alone. (gc's
  own ordering removes W before deleting B; "branch gone + live linked worktree" is
  git-impossible — `branch -D` refuses a checked-out branch.)
- **Skip completed steps** — reaping an absent thing is a no-op.
- **Every destructive step is honest on failure** — worktree-remove / `branch -D` /
  target-reap each name their leftover and exit non-zero; a stale administrative
  worktree entry is folded into the worktree leg (`git worktree prune`).

**Cleanup ownership:** the caller of `fork` owns `gc`. `/dispatch` concludes with `gc`
(after `import`); solo `/execute` ends with `land` then `gc`. The two-leg oracle spans
both routes.

## 9. Install + wiring — `claude install`

Skills currently install via `doctrine skills install` (RustEmbed from `plugins/`,
direct symlink into `.claude/skills` for Claude, `npx skills` delegation otherwise).
This slice **renames `skills install` → `claude install`** and extends it to install,
for the Claude surface:
- **skills** (unchanged behaviour);
- **agents** — `install/agents/claude/dispatch-worker.md` symlinked into
  `.claude/agents/` (parallel to skills; user-serviceable markdown — name +
  description + tool allowlist — not a Rust type);
- **the WorktreeCreate hook** (SubagentStart on the fallback ladder) — merged into
  `.claude/settings.local.json` via the existing `HookSpec` merge core (`src/boot.rs`),
  the same machinery that wires the `SessionStart` boot/sync hooks. The hook command is
  the one line in §4b.

> **Scope note.** The rename touches the CLI surface, goldens, and skill docs that say
> "skills install" — a deliberate SL-056 inclusion, not just the claude mechanism. The
> agents leg and the hook leg are the load-bearing parts; the rename is the
> consolidation that makes one Claude-surface installer.
>
> **Orphan-reference sweep (SR-3).** Renaming strands references: the memory
> `[[mem.pattern.distribution.skill-refresh-command]]` and any skill docs that say
> "skills install". Resolution: keep `skills install` as a **hidden deprecated alias**
> dispatching the same handler (no flag-day break), sweep the docs to `claude install`,
> and update the memory. The alias is a plan-level deliverable, called out here so it is
> not forgotten.

`doctrine worktree create-fork` is the verb the hook calls: reads the WorktreeCreate
payload JSON on stdin, `classify_create(payload) -> ForkWorker | PlainCreate | Refuse`
(a plain function — but the type is **three-valued, never two**: a malformed/missing
payload must classify to `Refuse`, never silently to `PlainCreate`, ψ), gated on
`payload.agent_type == DISPATCH_WORKER_AGENT_TYPE` (the single binary const, τ). On the
match it runs git-add + provision + `write_marker` with §4b's compensating rollback (ρ)
and **prints the worktree path**; otherwise it creates a **serviceable default worktree**
by doctrine's own conventions and prints the path, no marker (σ — unless a matcher
deletes this branch). The bad-payload refusals (`bad-payload`, `missing-agent-type`,
`missing-cwd`, `bad-dir`) fail creation fail-closed (ψ). The gate lives in the binary
(testable), the hook stays dumb — honoring the §1 thesis without over-engineering. (The
SubagentStart fallback uses a thinner `marker --stamp-subagent` that only stamps an
already-created worktree — §4b ladder.)

## 10. Governance deliverables

Decisions govern → land first; the design *produces the drafts*. Sequence: **G1+G3 →
O3 spike → G2 → G4 → remaining code** (the guard+privilege spike precedes the ADR-006
amend it validates).

- **G1 — ADR-008 revise→accept** (the gate). Fold §5.1 evidence; record D-B2 (ro
  `~/.cargo/bin` ⇒ no in-jail install race); re-scope D-B3 around the userns question.
- **G2 — ADR-006 amend.** (a) for **codex/pi**, demote the native WorktreeCreate hook as
  a *creation* preference (base-pinning + subprocess spawn supersede it); for **claude**,
  **promote a custom WorktreeCreate hook as the create+provision+stamp seam** (it
  *replaces* default git creation — fail-closed), with SubagentStart-stamp as the named
  fallback. (b) replace the `DOCTRINE_WORKER=1` self-arm with the **disk-marker-primary**
  signal (agnostic), env a codex/pi optimisation, plus the `Orchestrator` verb class.
  State the per-harness enforcement altitude. **Spike-first:** the guard/privilege model
  and the claude WorktreeCreate marker path (incl. named-subagent `agent_type`
  propagation) are validated by the O3 spike *before* G2 amends the accepted ADR.
- **G3 — ADR (new, id via `doctrine adr new` — likely ADR-011).** The spawn-seam
  **contract** (orchestrator owns fork-or-mark + provision + per-wt env emission;
  worker identity is the disk marker) + a **per-harness capability/altitude table**.
  **Blast-radius confession (σ).** On claude, absent a WorktreeCreate matcher (§4b/§9
  spike), the hook **intermediates creation for ALL `isolation: worktree` subagents** —
  not only dispatch workers. The contract for the non-dispatch else-branch is small (a
  serviceable default worktree, no marker), but the **blast radius is real**: a defect in
  the replicate-default path breaks every benign isolated subagent and `--worktree`
  launch, not just dispatch. A matcher that scopes the hook to `dispatch-worker` deletes
  the else-branch and the blast radius alike.
  - **codex/pi:** subprocess spawn ⇒ env-arm + per-wt env + bwrap (full mechanism floor
    *under D6*; accident-fenced absent D6).
  - **claude:** `Agent` tool + **WorktreeCreate-hook create+provision+stamp**, a
    **first-class** backend (not a degraded rung), marker-only altitude (no env, no per-wt
    target, no bwrap); **accident-fenced + prompt-enforced, not malice-proof** against a
    deliberate self-clear (§4c) — deferred to IDE-004 / userns-bwrap. The fail-closed
    altitude is **TWO-VALUED and O3-spike-contingent (φ) — the headline must not outrun the
    footnote:**
    - **O3 green** (named-subagent `agent_type` propagates through WorktreeCreate) →
      **fail-closed** via WorktreeCreate: no worktree without a marker. *(cell: `proposed`
      until the O3 gate greens.)*
    - **O3 red** → fail-open **SubagentStart-stamp** window (created-but-unstamped) →
      **prompt-enforced** worker-sole-writer. This row is the achievable altitude if the
      spike reds — it must be shown, not hidden behind the fail-closed headline.
    - **Concurrency** is execution-only in both arms: concurrent file-disjoint *execution*
      is first-class, but v1 funnels **one landing per base** (υ, §7c) — no serial
      *execution* constraint, but **not** parallel landing.
  - No harness-specific command (`claude -p`) is a required element. The fail-closed cell
    and the env/spike claims stay `proposed` until the O3 gate is green.
- **G4 — SPEC-012 rewrite.** Reframe Overview/Concerns (the funnel is now enforced
  code); rewrite D3 (fail-open env → fail-closed marker-primary guard); state the
  achievable altitude per harness, the quiescence constraint, the solo non-squash-land
  constraint, and the belt's honest scope; add FRs (fork, import, land, gc, marker
  guard, per-wt env contract).

Untouched: ADR-007, ADR-001/003/004, the withheld-tier model.

## 11. Code impact

| Path | Change |
|---|---|
| `src/worktree.rs` | `run_fork` (compensating-cleanup rollback, honest non-zero), `run_import` (`classify_import`), `run_land` (`classify_land`; `git merge --abort` mid-merge-guarded → `wedged-merge`/`inconsistent-merge-state`; `worktree-gone`), `run_gc` (**idempotent state machine** — `classify_gc(state) -> GcPlan`; two-leg oracle `--is-ancestor` OR `git cherry` patch-id; `--superseded-head`; squash → named refusal), `run_marker_clear` (`--operator`), **`run_create_fork`** (claude WorktreeCreate handler — parses stdin payload with **bad-payload refusals** `bad-payload`/`missing-agent-type`/`missing-cwd`/`bad-dir` (ψ, fail-closed), gates on `DISPATCH_WORKER_AGENT_TYPE` (τ), on a match does git-add+provision+`write_marker` **with §4a's compensating rollback** (ρ — post-`add` failure → `git worktree remove --force`+`branch -D`+reap before non-zero exit; half-failed rollback → distinct `orphan-leftover` token; **reuses `run_fork`'s rollback core**), else creates a serviceable default worktree by doctrine conventions (σ) and prints the path, no marker), plus a thinner **`run_stamp_subagent`** for the SubagentStart fallback. Pure: `target_dir_for_branch`, `marker_path`, `classify_import`, `classify_land`, `classify_gc`, `classify_create` (**three-valued: `ForkWorker | PlainCreate | Refuse`**, ψ)/`classify_stamp`. **`const DISPATCH_WORKER_AGENT_TYPE`** is the single source of truth `classify_create` reads (τ). New `write_marker`/`marker_present`/`remove_marker` (`write_marker` invoked by `fork --worker` and `create-fork`). Third `is_linked_worktree` consumer. **Deleted vs history: `run_marker_arm`/`run_marker_disarm`, `arm_path`, the lease/single-slot apparatus — obviated by the per-worktree-creation hook.** |
| `src/main.rs` | `fork`/`import`/`gc`/`land` subcommands + `marker {--clear --operator, --stamp-subagent}` (watch bool/arg clippy ceilings, [[mem.pattern.lint.cli-handler-args-struct]]). Worker-mode guard `worker_mode(root) = (is_linked_worktree && marker_present) OR env DOCTRINE_WORKER`. `write_class` unchanged. **`fork`/`import`/`gc`/`land` are the new `Orchestrator` class** (refused under `worker_mode`, NOT `Read`). The env-leg refusal on a non-linked tree carries the named dual-cause message for authoring **and** funnel verbs. |
| `src/skills.rs` → install surface | **Rename `skills install` → `claude install`** (keep `skills install` as a **hidden deprecated alias** → same handler, SR-3); add the **agents** leg (symlink `install/agents/claude/*.md` into `.claude/agents/`) and trigger the WorktreeCreate hook merge. Update `Write("skills install")` audit label + goldens; sweep docs + the `[[mem.pattern.distribution.skill-refresh-command]]` memory. **χ: every leg is golden-pinned in §12** — alias→same-handler, agent-def symlink presence, hook merge that preserves pre-existing hooks, idempotent reinstall, rename audit-label. |
| `src/boot.rs` | A **WorktreeCreate** `HookSpec` (SubagentStart on the fallback ladder) reusing the existing merge core; wired by `claude install`. The hook command **creates + provisions + stamps** with the ρ compensating rollback (fail-closed, SR-1), gated on `agent_type == DISPATCH_WORKER_AGENT_TYPE` (τ const); non-dispatch agent_types get a **serviceable default-creation** branch (σ — doctrine conventions, no marker; deleted if a matcher scopes the hook) or a bad-payload refusal (ψ). |
| `src/git.rs` | new reads behind the verbs: worktree list, **patch-id reachability** (`git cherry`), `B..S` name-only diff. Impure seam only. |
| `install/agents/claude/dispatch-worker.md` | **New** — the dispatch-worker subagent definition (name, description, tool allowlist). Its `name` is **pinned to `DISPATCH_WORKER_AGENT_TYPE`** (τ); the drift test reds if it diverges. |
| `plugins/doctrine/skills/{worktree,dispatch,execute}/SKILL.md` + new `{dispatch-subprocess,dispatch-agent}/SKILL.md` | Rewrite prose to *call* the verbs. **`/dispatch` becomes a harness router** → `/dispatch-subprocess` (codex/pi) \| `/dispatch-agent` (claude). Router input: the agent's harness self-belief **cross-checked against env-marker detection** (`CLAUDECODE` etc., names resolved in-skill/at spike — see IDE-005 for pushing this into the binary); routes only when detection **agrees**; mismatch/unknown → refuse **naming the cause**, never a blind spawn. The detection signal is itself spike-gated **per harness** (a green for claude does not bless codex/pi). `/dispatch-subprocess` binds the worker cwd (`env -C "$D"` / bwrap `--chdir`); `/dispatch-agent` spawns `subagent_type: dispatch-worker` — **the literal pinned to `DISPATCH_WORKER_AGENT_TYPE`** (τ; drift test reds on mismatch). One identical cadence, two ~2-line spawn templates. Re-embed ritual [[mem.pattern.distribution.skill-refresh-command]]. |
| ADR-008 / ADR-006 / **ADR-011 (new)** / SPEC-012 | G1–G4. |
| `flake.nix` | none for the spike; a `dispatch-worker` bwrap profile only if D6 lands (`--ro-bind`s the marker so a confined worker cannot `rm` it). |

## 12. Verification

- **Black-box CLI goldens** ([[mem.pattern.testing.black-box-cli-golden]], `force_no_tty`):
  `fork` (env on stdout, status on stderr, marker written; `git worktree add -b` syntax
  pinned); `import` happy + each refusal (`head-moved`, `tree-unclean`, `multi-commit`,
  `doctrine-touch`); `land` happy (`--no-ff` commit, fork commits reachable) + refusals
  (`tree-unclean`, `no-such-fork`, `dispatch-fork`, `worktree-gone`, `merge-conflict`
  with verified `git merge --abort` + clean tree, `wedged-merge`,
  `inconsistent-merge-state`); `gc` (worktree+branch+target reaped, two-leg oracle,
  squash named-refusal, `--superseded-head` honesty, `--dry-run` verdict).
- **Worker-mode guard — invariant test driving `run()`, not a pure helper**
  ([[mem.pattern.review.invariant-test-must-drive-the-write-seam]]): (a) linked worktree
  + marker → authoring/status-transition refuse (the **primary** signal); (b)
  `DOCTRINE_WORKER` on the coordination root → refuse (env optim); (c) worktree without
  marker, no env (solo) → allowed; (d) non-worktree tempdir, no env → allowed. Tests
  unset `DOCTRINE_WORKER` *and* run outside a marked worktree
  ([[mem.pattern.dispatch.worker-verify-unset-doctrine-worker]]).
- **`Orchestrator`-class refusal — exhaustive, every current member:** from a marked
  fork **and** from an env-set process, **`fork`, `import`, `land`, `gc`** are each
  refused — drive `run()`. `marker --clear` is **kept out** of this class and its
  bespoke refusal rules are tested separately (§3). *(This is the round-7 Charge π fix:
  the list is exhaustive and includes `land`.)*
- **`create-fork` gate (the claude path):** `classify_create` golden — `agent_type ==
  DISPATCH_WORKER_AGENT_TYPE` → git-add + provision + marker written + worktree path
  printed on stdout; other agent_type → serviceable default-creation (σ), path printed,
  **no marker** (a benign subagent is never branded). Reads `agent_type`/derives `<dir>`
  from the **payload**, not the hook's process cwd (SR-4). The thinner `marker
  --stamp-subagent` fallback verb has its own golden (stamp-only on an already-created
  worktree).
- **`create-fork` orphan cleanup (ρ — the orphan must be GONE, not merely unspawned):**
  a forced `provision`/`write_marker` failure **after** `git worktree add` succeeds →
  non-zero exit **AND** the worktree+branch are reaped (assert `git worktree list` /
  `git branch` show no leftover — not merely that the worker did not spawn). A rollback
  that itself half-fails → distinct **`orphan-leftover`** exit that **names** the dir+branch
  left on disk. A pre-`add` failure leaves no fork at all.
- **`create-fork` bad-payload refusals (ψ — fail-closed):** goldens for malformed JSON →
  `bad-payload`; **missing `agent_type` → `missing-agent-type`** (it REFUSES — assert it
  does NOT silently replicate-default and write no worker); missing/empty `cwd` →
  `missing-cwd`; underivable/escaping `<dir>` → `bad-dir`. Each a distinct non-zero exit
  that **fails creation** (no worktree born).
- **`dispatch-worker` drift test (τ — reds on mismatch):** assert the installed agent-def
  `name` and the `/dispatch-agent` skill's `subagent_type` **both resolve to
  `DISPATCH_WORKER_AGENT_TYPE`**; a divergent literal **REDS** the test. The const is the
  gate's only key — a typo cannot silently send `create-fork` to the else-branch.
- **σ serviceable default (no-matcher arm):** the non-dispatch else-branch produces a
  worktree that is **valid + usable + bears NO marker** — assert validity/usability, NOT
  byte-equality with Claude's native `worktree-agent-<id>` layout (the bar is "suffice for
  general isolated-subagent duty," not Claude fidelity). If the O3 matcher greens, this
  branch is deleted and the test retires with it.
- **`marker --clear` (self-brick cure):** a stale marker on a linked-worktree
  coordination root → writes + `gc` refused; `marker --clear --operator` (env unset)
  restores both from within the CLI; refused when `DOCTRINE_WORKER` set or run outside
  the marker's tree; a bare `--clear` in a linked worktree refuses (accident-fence).
- **`fork` compensating cleanup:** a forced provision failure rolls back leaving no
  orphan; a rollback that half-fails exits non-zero naming the leftover; a pre-marker
  failure leaves no unmarked fork.
- **`gc` idempotent rerun (round-7 Charge ξ):** inject/simulate failure after **each**
  destructive step, rerun `gc`, assert either full cleanup or a named non-zero leftover.
  Include the **branch-gone / target-present** case (reap T from the branch name alone,
  no live branch) and the gc-own-ordering case (W removed before B).
- **`claude install` surface (χ — the installer is mechanism, golden-pinned like the
  verbs):**
  - **alias→same-handler:** `skills install` (hidden deprecated alias) and `claude
    install` dispatch the **identical** handler — golden the two surfaces produce the same
    effect (SR-3, no flag-day break).
  - **agents leg:** after `claude install`, `install/agents/claude/dispatch-worker.md` is
    present as a symlink under `.claude/agents/` (assert the link lands and resolves).
  - **hook merge preserves pre-existing hooks:** merging the WorktreeCreate `HookSpec`
    into a `.claude/settings.local.json` that **already** carries unrelated hooks leaves
    those hooks intact (cite the boot `HookSpec` merge tests as prior art — same merge
    core, §11).
  - **idempotent reinstall:** running `claude install` twice yields no duplicate symlinks
    and no duplicate hook entries.
  - **rename audit-label/golden:** the `Write(...)` audit label and goldens reflect
    `claude install` (the renamed surface), not the stale `skills install` string.
- **O3 spike (claude marker-via-WorktreeCreate) — THE gate.** Confirm a **named
  `dispatch-worker` subagent reliably propagates `agent_type` through the WorktreeCreate
  payload** (the prior probe saw only `name` — but it used an *unnamed* subagent, so that
  is expected, not a refutation). Confirm the custom hook **replaces default creation**
  (path-on-stdout honored, non-zero fails creation), fires before the worker, and the
  payload shape. **HARD GATING OUTCOME (σ, not a sub-question): can a WorktreeCreate
  matcher scope the hook to `dispatch-worker`?** — matcher GREEN deletes the
  replicate-default else-branch (and the σ blast radius) entirely; matcher RED mandates the
  serviceable-default else-branch + its golden. Record the answer as a gate result, not a
  footnote. Confirm **two concurrent** dispatch-worker creations each get their own
  create+provision+stamp (no shared slot). Exercise the **fallback ladder**: if
  WorktreeCreate lacks `agent_type` → SubagentStart-stamp (which the probe confirmed
  carries `agent_type`, accepting the fail-open window) → prompt-enforced. Per-harness: a
  green for claude does not bless codex/pi env propagation, which is its own gate.
- **D5 (codex/pi):** two parallel worktree builds, no cargo-lock contention, each spawns
  its correct `CARGO_BIN_EXE`. (Claude shares the jail-wide target — the §5.1 rituals
  are the proof there, not isolation.)
- **D6 (if landed):** an out-of-tree write from the worker process is OS-denied; the
  confined worker cannot `rm` its ro-bound marker.
- **Behaviour-preservation gate — precise.** The migration legitimately *changes*
  worker-mode behaviour (env→marker trigger): the old `DOCTRINE_WORKER` guard tests are
  **rewritten** to the marker, not kept green. What stays green *unchanged* (the
  preservation proof): `select_copies`/provision, `branch-point-check`,
  `is_withheld`/allowlist, the `git.rs` born-frame seam.

## 13. Open questions (post-lock)

- **OQ-1 (IMP-043):** moved-HEAD import (`--allow-reanchor`: 3-way onto a moved HEAD +
  computable path-disjointness, [[mem.pattern.dispatch.reanchor-base-on-disjoint-head-move]])
  is a named backlog follow-up — **not** v1 scope, **not** fail-open prose. v1 refuses
  `head-moved` → re-dispatch.
- **OQ-2:** bwrap userns feasibility — empirical at the D6 spike (probe `bwrap
  --unshare-user --ro-bind / / true` inside the jail).
- **OQ-3:** disk pressure under N concurrent `wt/<branch>` targets — gc reaps; a worktree
  cap or `sccache` only if it bites.
- **OQ-4 (IDE-005):** push harness detection into the binary to shrink the `/dispatch-*`
  router decision surface — a named idea, **not** SL-056 scope.
- **OQ-5 (IMP-045):** macOS OS-confinement (Seatbelt / `sandbox-exec`, with bwrap
  fallback) — the cross-platform analog of D6/O7 nested bwrap: the OS floor under §4c's
  deliberate-self-clear residual on non-Linux. A named backlog follow-up (the structured
  edge already exists), **not** SL-056 scope.

## Appendix — resolved findings

Eight adversarial passes shaped this design. The full reasoning trail and per-charge
dispositions are preserved, not re-litigated here:
- `design-history.md` — the round-1→7 narrative (superseded).
- `inquisition-1.md` … `inquisition-7.md` — the charges and sentencing per round.

Headline dispositions carried into this clean design:
- **Round-8 ρ** (`create-fork` orphan on post-`add` failure) → §4b compensating-cleanup
  rollback + `orphan-leftover`, §11 reuses `run_fork`'s core, §12 asserts the orphan GONE.
- **Round-8 σ** (replicate-default else-branch hand-waved, repo-wide blast radius) → §4b
  matcher promoted to a HARD gating O3 outcome (matcher-green deletes the branch); absent
  it, a *serviceable* doctrine-convention default (NOT a Claude-fidelity mirror); §10/G3
  confesses all-isolated-subagent intermediation.
- **Round-8 τ** (`"dispatch-worker"` literal drift fails open) → §4b/§11
  `const DISPATCH_WORKER_AGENT_TYPE`, agent-def + skill pinned, §12 drift test reds.
- **Round-8 ψ** (`create-fork` stdin trust boundary, no bad-state refusal) → §4b/§9
  `bad-payload`/`missing-agent-type`/`missing-cwd`/`bad-dir`, fail-closed, §12 goldens.
- **Round-8 υ** (concurrency oversold) → §4b/§7c/§10 re-scoped: parallel execution, NOT
  parallel landing; own-batch imports named as a head-moved invalidation source.
- **Round-8 φ** (fail-closed altitude oversold vs O3-contingent) → §10/G3 two-valued
  altitude (O3-green fail-closed / O3-red fail-open SubagentStart window), cell `proposed`.
- **Round-8 χ** (install surface unverified) → §12 alias/agents/hook-merge/idempotent/
  rename goldens.
- **Round-7 ν** (codex/pi cwd not bound to fork) → §4a `env -C "$D"` / bwrap `--chdir`.
- **Round-7 ξ** (gc no idempotent recovery) → §8.2 idempotent state machine.
- **Round-7 ο** (arm-lease uses timeout as proof-of-death) → **dissolved**: the
  per-worktree-creation-hook redesign removes the arm sentinel entirely (§4b); there is
  no lease, no race.
- **Round-7 π** (Orchestrator-class verification omits `land`) → §12 exhaustive list
  (`fork`/`import`/`land`/`gc`).
- **Rounds 3–6 (B/C/γ/θ/κ/μ/ζ/η/λ/ι …)** — the marker/sentinel/belt/router lineage; the
  arm-sentinel charges (γ/θ/κ/ο) are obviated by the empirical hook findings
  ([[mem.pattern.dispatch.claude-subagentstart-worker-identity]],
  [[mem.pattern.dispatch.claude-agent-worktree-not-fork-provisioned]]); the surviving
  invariants (belt scope, land guards, router cross-check, altitude honesty) are stated
  in §3–§10.
