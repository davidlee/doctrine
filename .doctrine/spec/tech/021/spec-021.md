# SPEC-021: Dispatch orchestrator process

<!-- Reference forms: entity ids padded (SPEC-007, ADR-004); doc-local refs bare
     (D1 decision, OQ-1 open question). See .doctrine/glossary.md § reference forms. -->

> **AMENDED — ARM COLLAPSE (SL-254, 2026-08-14).** This spec was authored over a
> **two-arm** dispatch posture (an in-session claude `Agent` arm and a subprocess
> arm for codex/pi). SL-254 collapsed both onto **one confined subprocess arm**:
> every harness, claude included, is spawned by `scripts/spawn-confined.sh
> <harness>` inside a kernel-level jail, the worker cannot commit, and worker
> identity is the `DOCTRINE_WORKER` environment variable and nothing else
> (`DEC-207`, `DEC-208`, `DEC-213`). Passages that *described* the shipped
> mechanism have been rewritten; recorded decisions, hypotheses and concerns
> carry a dated amendment banner and retain their historical body. The title
> still holds — this is still the orchestrator process — but "arm", "altitude
> tier" and "marker" no longer name anything live.

## Overview

This component is the **orchestrator-facing process** layer of the dispatch &
worktree container (`parent: SPEC-012`). SPEC-012 owns the *mechanism* — the verb
family (`provision`/`fork`/`import`/`land`/`gc`), the worker-mode guard, the
branch-point check, the born-frame git seam — each cited here by its `REQ-NNN`.
This spec owns the *process that wields them*: the **funnel cadence** (the order of
acts and the halt discipline), the **concurrency and landing decision logic**, the
**spawn-and-confinement contract** (amended SL-254 — formerly the per-harness altitude
contract), the **two-stage integration projection**, and the **load-bearing operational
gotchas** that recur across real dispatch runs. It descends
from **PRD-015** (the product intent for concurrent isolated dispatch) through its parent
container SPEC-012; ADR-006 (policy-agnostic worktree posture), ADR-011 (harness-agnostic
spawn — its `D3`/`D5`/`D6` amended by SL-254), and ADR-012 (integration topology) are the
governing decisions.

The container's keystone (SPEC-012) is that *the funnel is enforced CLI mechanism, not
prose an LLM may skip*. The complement this spec records is the irreducibly-prose
residue the mechanism cannot absorb: **what order to invoke the verbs in, when to halt
versus proceed, what the confinement floor does and does not enforce, and the
environmental footguns** (proxy-masked git, patch corruption, the jail cwd-revert) that
a fresh orchestrator would otherwise rediscover. There is no longer an arm to choose:
SL-254 collapsed dispatch onto one confined subprocess spawn path, so the routing
decision this spec once owned has been *removed*, not relocated (amended SL-254). It is
the synthesis of truth currently scattered across the three dispatch/worktree skills
(`/dispatch`, `/dispatch-spawn`, `/worktree` — the runbook), ADR-006/008/011/012 (the
decisions), and the dispatch memory corpus (the gotchas) — collated into the durable
spec tier, citing each source, restating no verb internal.

> **Posture** (amended SL-254). Largely **retrospective** — the cadence, the
> concurrency discipline, and the coordination-tree lifecycle are shipped (SL-056,
> SL-064; ISS-029 resolved 2026-06-19), and the single confined spawn path is shipped
> (SL-254). The arm split and the per-harness altitude split it rested on are **gone**,
> not pending. Of the two pieces once named as forward-intent, the positive
> coordination-tree marker (IMP-065) is **closed as moot** — there is no marker at all
> now, and coordination-tree write authority rests on the orchestrator process simply
> not carrying `DOCTRINE_WORKER` (`DEC-207`); the in-verb import re-anchor /
> content-base assertion (IMP-043) remains **open**, still deferred to a sync-time
> report. Requirements stay `pending`; coverage is reconciled, never inferred.

## Responsibilities

Mirrors the structured `responsibilities` list. Verb *internals* are SPEC-012's and
are not restated; what follows is the process that sequences and decides among them.

### The funnel cadence — an ordered contract, halt-on-breach

Per batch, the orchestrator captures `B = git rev-parse HEAD` **before any spawn
window**, then after workers return runs, in exact order (ADR-006 D7; SL-056 §7,
SL-064 §3):

1. **Precondition** — coordination tree clean, `HEAD == B` (`branch-point-check`,
   SPEC-012 REQ-191).
2. **Delta-check** — the worker hands back an **uncommitted working tree**, so the
   delta is its live `diff HEAD` plus its untracked adds; the single-non-merge-commit
   check is **vacuous** on this path (amended SL-254 — the worker's `.git` is
   read-only inside the jail and it cannot commit at all). The committed-fork form
   (`import --fork`, net `B..S` a single non-merge commit with `S^ == B`) survives as
   the CLI's other import source, not as the dispatch drive's.
3. **R-5 belt** — reject any `.doctrine/` or `.claude/` touch in the delta paths
   (`import`'s `doctrine-touch`/`claude-touch`, SPEC-012 REQ-249), then the
   `--slice` scope leg. These prefixes are the belt's **only** hard floors: it does
   not enforce any configurable forbidden-writes list (amended SL-254).
4. **Import** — apply the surviving net delta onto `B` (`import --from-worktree`),
   **non-committing** (SPEC-012 REQ-249), then the in-process reject-and-halt
   `prove` gate: an unformatted or lint-red delta halts **staged, never auto-fixed**
   (a worker-delta defect, distinct from a dirty-base defect).
5. **Verify** — run the project's combined-tree verify; if RED, isolate the offender
   per delta.
6. **Branch-point guard** — coordination HEAD still `B`? (demoted under coordination-
   tree isolation, DD-5/SL-064 §3 — within a run HEAD moves only at the orchestrator's
   own commit; its real job relocates to integration-sync, where trunk may have moved).
7. **Commit** — exactly **one** commit on the coordination branch.
8. **Record** — knowledge (memory, AC evidence, notes) trails the confirmed commit.

The discipline is **report-and-halt** on conflict, moved HEAD, authored-tree touch, or
unrecoverable RED — **never auto-resolve**. The invariant is *knowledge always trails
confirmed code*: the coordination branch is the durable store, orchestrator context is
disposable, so crash ≡ handover ≡ resume-from-coordination-branch (ADR-006 D7).

### Spawn and concurrency decision logic

**Spawn — one path, no routing decision** (`/dispatch` router → `/dispatch-spawn`;
amended SL-254, `DEC-208`/`DEC-213`). There is no arm selection: every harness is
spawned as a confined subprocess by `scripts/spawn-confined.sh <harness>`, which owns
the whole pre-spawn sequence itself —

1. **capability probe — Linux only** (corrected SL-254). `spawn-confined.sh:61` probes
   `command -v bwrap` and refuses by name (`bwrap-unavailable`) before minting a fork.
   The Darwin arm **skips the probe entirely** and there is no `sandbox-exec`
   presence check anywhere: `have_bwrap` is `#[cfg(not(target_os = "macos"))]`
   (`jail.rs:147`) with no Darwin sibling, and `jail_prefix.rs`'s macOS arm
   (`:150-169`) fails only on topology, policy, or profile-write errors. So a macOS
   host without `sandbox-exec` gets a *successful* prefix write and then an
   **unnamed** exec failure. Still fail-closed — nothing spawns unconfined — but the
   named-refusal half of `DEC-208` is a Linux property, and its absence on macOS is a
   gap, not a design choice. An unresolvable jail **aborts the spawn**; there is **no
   unconfined fallback and no degraded/reduced-enforcement rung** (`DEC-208`).
2. **fork** — `worktree fork --base <B> --branch <BRANCH> --dir <DIR> --worker`,
   orchestrator-classed, from the project root before any confinement. The fork is
   deliberately **unbound** (no `--slice`/`--phase`), so no funnel row lands and
   `dispatch next` is advisory on this drive, not the driver.
3. **confine + exec** — `timeout <BACKSTOP> <jail prefix> <harness exec>`. The writable
   set is the fork dir **and the harness config dir** (`~/.claude` / `~/.pi`), plus a
   private `/tmp` and the device sinks; on Linux everything else is read-only under
   `--ro-bind / /`, on macOS everything else is write-denied by the SBPL
   `(deny file-write*)` floor while reads and exec stay open. `DOCTRINE_WORKER=1` rides
   the same argv — the same argv that establishes the write floor establishes worker
   identity, and both die with the process on Linux (`--die-with-parent`; Seatbelt has
   no analog) (`DEC-207`).
   **The config-dir grant is a real carve-out, not an implementation detail:**
   `~/.claude` holds `settings.json`, hooks and agent definitions, so a confined claude
   worker can write the orchestrator's own harness configuration. It is granted
   deliberately — claude needs it for the subscription credential (`DEC-210`) and pi
   writes it at runtime — and the narrowed mount set remains a known, untaken
   refinement.

The only surviving per-harness difference is the exec line, the config dir, and the
completion signal (`claude -p` exits when its turn ends; `pi --mode rpc` never
self-exits, so that profile holds stdin open on a fifo and polls for a typed settle
event). **The spawn path accepts exactly two harnesses — `pi` and `claude`** — and
hard-exits on anything else (`spawn-confined.sh:35-42`); it has no default of its own,
the harness is the script's first argument (corrected SL-254 — this paragraph
previously said the default harness is `pi`, which no code establishes). Note the live
mismatch: `DispatchConfig`'s `SubprocessHarness` still defaults to `Codex`
(`src/dispatch_config.rs:39-45`, echoed by `install/doctrine.toml.example:84`), so the
**declared default harness cannot be spawned through the shipped path at all**. That is
a code defect, not a spec position; it is carried out of this phase as a backlog
candidate rather than repaired here. Harness choice remains a *preference*, not a
routing decision, and never a different mechanism (IMP-101).

**Serial vs parallel** — `plan-next` plans **parallel batches only when file-disjoint**;
default serial (one worker per phase). The asymmetry is load-bearing: parallel
*execution* is first-class, but v1 lands **one worker per base** under the stationary-
head precondition (importing+committing worker A moves HEAD `B→B+1`, so worker B —
also forked at `B` — refuses `head-moved`). Serial-dependent phases **self-base**: the
orchestrator advances coordination HEAD to phase N's integrated tip before spawning
N+1, so the next worker forks the dependency for free (SL-056 §7c, SL-064 §7c).

### Coordination-tree placement and lifecycle

The orchestrator always runs on a dedicated `dispatch/<slice>` coordination worktree,
provisioned per run via `worktree coordinate` (ADR-012 D1, SL-064 §2). ADR-006 D6a's
principle — *mode, not location, decides who may write* — is unchanged and now
**absolute**: there is no marker on any tree (amended SL-254, `DEC-207`), so what makes
the coordination tree writable is that the orchestrator's own process does not carry
`DOCTRINE_WORKER`. Identity is a property of the process, not of a directory. It is
created **inside the project root** (convention `.dispatch/SL-<n>`): under a
cwd-confining jail a `cd` to an outside sibling silently reverts to root on the next
Bash call, leaving the session on `main` (ISS-029, resolved 2026-06-19; ISS-031;
`mem.pattern.dispatch.claude-arm-coord-placement`). Bash cwd stays **parked in the
coordination tree for the whole drive loop** — so that `B` is captured, and every
funnel verb runs, against the coordination HEAD rather than the session root. The
worker's base is **not** inherited from that cwd: it is pinned explicitly by
`fork --base <B>` (amended SL-254 — base-by-placement was the in-session arm's story
and retired with it). Concurrent same-slice
dispatch is refused at creation; the worktree directory is removed at conclude while
the `dispatch/<slice>` branch is kept as a deliverable (worktree-life < branch-life,
DD-6).

### Per-harness altitude — uniform contract, honest non-uniform reach

> **AMENDED — FALSIFIED (SL-254, 2026-08-14).** There is no longer a per-harness
> altitude split, because there is no longer more than one arm. Every harness —
> claude included — is spawned as a confined subprocess by
> `scripts/spawn-confined.sh <harness>`, so **every** harness reaches the same
> floor: explicit `fork --base B`, kernel-level bwrap/`sandbox-exec` confinement
> whose writable set is the fork's worktree **plus the harness config dir**,
> `DOCTRINE_WORKER=1` set by that same confining argv, and a fail-closed refusal —
> *named* on Linux (`bwrap-unavailable`), unnamed on macOS for want of a backend
> probe — when the jail cannot be established (`DEC-208`). The reach is now uniform
> across **harnesses** *and* non-negotiable rather than uniform-in-contract and
> unequal-in-reach. It is **not** uniform across **platforms**: the floor is uniform
> in intent and in write-fencing, but the Linux and Darwin argv differ on network
> (Darwin denies it, the Linux inline bwrap array carries no `--unshare-net` and so
> leaves the worker fully networked) and on process lifetime (`--die-with-parent` has
> no Seatbelt analog) — see SPEC-012's spawn-arm section for the enumerated
> divergences. The
> historical two-arm table below is retained for the record; it no longer describes
> the shipped posture, and every claude-column cell in it names something deleted
> (the `Agent`-tool spawn, the disk marker, the `SubagentStart` stamp,
> `verify-worker`, base-by-placement).

What holds now, in the terms the table used: identity is the `DOCTRINE_WORKER`
environment variable and nothing else (`DEC-207` — the disk marker, its writers
`marker --stamp-subagent`/`--clear`, and the `Cause` truth table are all gone, and
`marker::describe_mode` is a two-row table over one input); the base is always
explicit; confinement is always the OS floor; the pre-dispatch base-clean assertion
(`doctrine check prove` on `B`) is always the orchestrator's, because the
orchestrator always owns `fork`. The residual the old claude column confessed —
enforcement that depended on a fail-open hook — is dissolved rather than mitigated:
a worker that cannot be confined is never spawned. One residual is **new** and
must not be read out of the table: `worker_commit` was the only production reader
of `DispatchConfig::worker_forbidden_writes`, so with it retired (`DEC-204`) the
*configurable* tail of that list (`.agents/**`, `install/agents/**`, `flake.nix`)
now enforces nothing; the import belt only ever enforced its two hard-coded prefix
floors (`.doctrine/**`, `.claude/**`) plus the `--slice`-scoped `undeclared-scope`
leg (`import.rs:136-152`) — never the configurable tail. Closing that gap at spawn time via read-only binds
is carried forward to SL-255 (IDE-051).

*Historical (pre-SL-254 two-arm altitude table), retained for the record:*

| Axis | **codex / pi** (`/dispatch-subprocess`) | **claude** (`/dispatch-agent`) |
|---|---|---|
| Spawn | subprocess (`codex exec` / pi RPC), cwd bound via `env -C "$D"` / bwrap `--chdir` | in-session `Agent` tool, `isolation: worktree` — first-class, not a degraded rung |
| Identity | disk marker (primary) **+** `DOCTRINE_WORKER` env (optimisation) | **disk marker only** — no env channel |
| Marker writer | `fork --worker` (orchestrator-owned, before any spawn window) | matcher-scoped `SubagentStart` hook `marker --stamp-subagent` (claude creates the worktree; hook provisions+stamps into `cwd`) |
| Base | **explicit `fork --base B`** | base==B by **placement** (cwd==coord tree, `baseRef='head'`) + post-spawn `verify-worker` (`merge-base --is-ancestor B HEAD`) |
| Pre-dispatch baseline-verify | **yes** (orchestrator owns `fork`) | **no** — unbuildable fork caught late at `import → verify` (a wasted worker run) |
| Worker-on-main catch | **yes** (env leg) | **no** — deferred D2b residual, mitigated by always-isolating + the hook-stamped marker |
| Build isolation | per-worktree `CARGO_TARGET_DIR` (ADR-008 D-B1) | none — shares the jail-wide target |
| OS confinement | nested bwrap (ADR-008 D-B3, marker ro-overlay *after* the rw worktree bind; never ro-bind `settings.local.json`) | none — `Agent` is not a subprocess to wrap |
| Fail-closability | full mechanism floor | SubagentStart is a **read-only event** — the stamp is **not fail-closable**; an unstamped worker is contained by the marker-absent fail-closed privilege rule + the `import` belt, not by the hook |

> **AMENDED — FALSIFIED (SL-254, 2026-08-14).** The paragraph below describes
> **Mode B**, the confined in-session orchestrator tier. It is **retired entirely**
> (`DEC-217`) — its entry point is gone, not merely one of its tools: the in-session
> `Agent` spawn it named as "today's realization" no longer exists, `worker_commit`
> (its gated self-commit) is deleted (`DEC-204`), and the shipped surface
> (`install/hymns/role/orchestrator.md`, `install/workflows/drive-slice.js`,
> `install/agents/claude/dispatch-orchestrator.md`, `install/agents/claude/dispatch-probe.md`)
> was deleted whole. There is exactly one altitude now: a **main-thread orchestrator**
> holding direct write authority over the coordination `.git`, driving confined
> subprocess workers. The mediated-write tools (`import` → `conclude` → `reap`) and
> their declared-args routing survive as machinery and are unchanged, but no tier
> consumes them as its *sole* write path.
>
> **`REQ-335` is NOT retired by this revision, deliberately.** It is the
> confined-orchestrator tier, and `DEC-217` retires Mode B's *shipped entry
> point* — but `REQ-335` was never `active`; it is a `pending` forward-intent
> contract, and `SL-254`'s locked design (`§6`, `OQ-2`) states explicitly that
> its tier **stays `pending` as a contract**. Retiring it would assert that the
> tier will never be built, which is a decision beyond this slice's `DEC-2xx`
> set. So it stays `pending`, and what changes is only that nothing implements
> it today. The confined-orchestrator transport named in `REQ-387` is *narrowed*
> on the same reasoning, not deleted. The historical analysis below is retained
> for the record.

Both columns above are **main-thread** altitudes — the orchestrator holds direct
write authority over the coordination `.git` and runs the funnel through the CLI.
A third, **harness-neutral** tier (FR-007, REQ-335) drops that authority: a
**confined orchestrator** runs *inside* the coordination worktree under a
cwd-confining jail with the shared object store **read-only**, so it cannot
compose the coordination commit itself. It lands every worker delta and
coordination write through a **gated write-funnel** of mediated tools — `import`
(folds the worker's committed delta) → `conclude` (disposable runtime flip + one
boundary commit, self-healing) → `reap` — never a direct `.git` write, and
reads authored/runtime state raw while performing all mutation through the funnel
(the reads-raw/writes-mediated **wall**). Trunk-facing verbs
(`refresh-base`/`candidate`/`integrate`) write **outside** the jail, so this tier
**cannot** perform them — it **reports-and-halts** them to the delegating parent.
The funnel tools route by their **declared args alone** (D-B5, VT-4:
`resolve_coord`, cwd/agent-id/payload-independent), so the tier is a property of
the **mediated-write contract**, not of any single harness: the claude nested-`Agent`
spawn is today's realization, but an out-of-jail transport reuses the same tools
unchanged. The trust-bearing core stays harness-identical; the confined tier adds
a *lower* write-authority altitude, not a new mechanism.

### Two-stage, audit-gated integration projection

The coordination branch is the funnel's SSoT; the sync verb reads the completed
`dispatch/<slice>` and projects outward in two stages (ADR-012 D4/D5, SL-064 §4):

- **Stage 1 — `dispatch sync --prepare-review`**: materialise the reviewable refs —
  `review/<slice>` (impl bundle), `phase/<slice>-NN` (code, cut from `dispatch/<slice>`
  at sync time so the deliverable is spawn-path-universal — ADR-012 D3; "on the claude
  arm" in the original wording is moot now that there is one arm, but sync-time cutting
  is retained because the worker never commits and so never mints a phase ref of its
  own, amended SL-254) — and
  a **journal committed to `dispatch/<slice>` before any external ref mutation**. Every
  ref update is a compare-and-swap on `expected_old_oid`. **No trunk write.**
- **Audit** runs from the parent/root context against the prepared refs (RV review
  verbs refuse on a worktree fork; the coordination worktree is removed at conclude).
- **Stage 2 — `dispatch sync --integrate`**: optional projection to trunk/`edge`,
  **opt-in, fast-forward-only, expected-tip-CAS**; a moved/non-ff target ⇒ **report,
  never auto-resolve, never force-push**.

Default routing is to `review/<slice>`, **never trunk-by-default**. The impl bundle
holds together by default; only knowledge explicitly marked slice-orthogonal projects
ahead independently (the four-bucket temporal classifier, ADR-012 D2).

### Operational gotchas as durable constraints

These are environment-and-harness realities the orchestrator must honour; each is a
durable constraint, not a single run's accident. See **Concerns** for the catalogue.

## Concerns

- **Output-rewriting proxies (rtk) silently corrupt the git plumbing the funnel reads.**
  `git diff` is stat-proxied (returns a `path | N +++` summary, not a patch), inner git
  exit codes are masked in piped/chained invocations, and `--name-only`/`rev-parse`/
  `ls-tree` can return phantom hits. Funnel guards that branch on a chained exit code or
  pipe `git diff` into `git apply` take the wrong branch silently. **Constraint:** read
  decisions from **printed output** (`ls-tree --name-only`, `diff --name-only | grep`,
  `git cherry`), capture `rc=$?` on its own line when an exit code is genuinely needed,
  and bypass the proxy with `rtk proxy git …` for any blob-level query. The **combined-
  tree project verify is the real gate** — it, not import's own fidelity check, catches
  a reverted wiring. (`mem.pattern.tooling.git-cat-file-e-exit-masked-use-ls-tree`,
  `mem.pattern.dispatch.rtk-masks-git-plumbing-during-funnel-reanchor`,
  `mem.pattern.dispatch.rtk-git-diff-stat-use-checkout-import`.)
- **`git apply` patch corruption / proxying → the checkout-import idiom.** When
  `worktree import` (or a raw `git apply --3way`) fails `corrupt patch` / `No valid
  patches in input`, substitute, running the verb's belts by hand on the trusted side:
  prove `HEAD==B`, clean tree, `S^==B`, single non-merge, R-5 clean, then
  `git checkout S -- $(git diff --name-only B..S)` (valid because the batch is disjoint
  and coord==B, so S's blobs *are* the net delta), verify `git diff S -- <paths>` empty,
  and continue the cadence. (`mem.pattern.dispatch.worktree-import-corrupt-patch-use-checkout`,
  `mem.fact.doctrine.import-corrupt-patch`.)
- **Never widen a worker's delta when integrating.** Stage **exact declared paths**;
  `git add -A` / `commit -a` sweep foreign untracked/WIP files on a shared tree (a
  foreign untracked slice TOML was swept once and amended out). The dedicated
  coordination worktree removes most of this contention *by construction* (SL-064), but
  the staging discipline stands. (`mem.system.dispatch.orchestrator-on-shared-main-contention-cost`,
  `mem.pattern.dispatch.glob-add-sweeps-foreign-untracked-on-shared-main`.)
- **Re-anchor only on a proven-disjoint HEAD move.** When HEAD legitimately moves
  between capture and import, prefer re-anchoring `B → current HEAD` over re-dispatch
  (which reproduces an identical delta) — but **only** after a per-path byte-identical
  disjointness proof (`git diff --stat <oldB>..<newHEAD> -- <each delta path>` empty,
  read raw under a proxy) and intervening commits touching only unrelated trees. A
  moved path needs a real `git apply --3way`, not checkout-import. The in-verb re-anchor
  is deferred (IMP-043). (`mem.pattern.dispatch.reanchor-base-on-disjoint-head-move`,
  `mem.pattern.dispatch.three-way-import-onto-moved-shared-main`.)
- **The landed oracle is durable git state, never a runtime receipt.** `import`'s
  `apply --3way` severs ancestry, so `branch --merged` and delta-emptiness are unsound
  reap oracles, and a gitignored "landed" flag survives a crash-before-commit and lies.
  `gc` reaps only on `git cherry` (ancestry **or** every-commit patch-id `-`); a squash
  is indistinguishable from never-landed, which is *why solo `land` must be non-squash*
  (SPEC-012 REQ-250/251). (`mem.pattern.dispatch.landed-oracle-needs-import-receipt`.)
- **Claude integration can collapse the worktree onto the parent.**
  **AMENDED — FALSIFIED (SL-254, 2026-08-14):** the mechanism is gone — there is no
  in-session `Agent` spawn, the worker's `.git` is read-only inside its jail so it
  produces no commit to collapse, and its hand-back is an uncommitted working tree the
  orchestrator imports. The *derived constraint survives and is now mechanised*:
  gates run **post-landing** on the orchestrator side, on the proven delta, never on
  worker self-report — `import --from-worktree` runs the reject-and-halt `prove` gate
  in-process, and the verify beat is the orchestrator's own. Historical premise, for
  the record: the `Agent` tool's `isolation: worktree` may integrate the worker commit
  onto the parent branch on completion rather than leaving an isolated fork, and
  cross-worktree LSP/reads are stale.
  (`mem.pattern.dispatch.claude-agent-worktree-integrates-commit-onto-parent`.)
- **Record durable memory on trunk, not in a fork.** Memory committed inside a worker
  branch is orphaned by a squash/sever (content survives, the git anchor points at a
  commit that never lands; staleness fires). (`worktree` skill; SPEC-012 NF-002 context.)
- **Identity gap, consciously accepted.**
  **AMENDED — FALSIFIED (SL-254, 2026-08-14):** the gap this concern names is closed by
  construction, not by the fix it proposed. Identity is now the `DOCTRINE_WORKER`
  environment variable and nothing else (`DEC-207`), set by the same confining argv that
  establishes the worker's write floor and dying with the process — so worker-mode is
  *positively asserted*, never inferred from an absence, and an unconfined worker cannot
  exist to be unstamped (`DEC-208`: no jail ⇒ no spawn). Both proposed closes are
  **closed as moot**: the positive coordination-tree marker (IMP-065) and the
  orchestrator post-spawn marker check (IMP-052). Historical position, for the record:
  v1 rested coordination-tree write-permission on marker-*absence*, indistinguishable
  from an unstamped worker; the D2b fence (R-5 belt, IMP-052 post-spawn check, env
  worker-on-main catch, bwrap-no-push) was defence-in-depth, **not a coverage proof**
  for the full Orchestrator verb class, and the real close was to be the positive
  marker (IMP-065). (ADR-012 OQ-D, ADR-006 D2b.)

## Hypotheses

- **The mechanism cannot absorb the order or the footguns.** SPEC-012
  moved each funnel *step* into a verb; what stays prose is the *sequence* and the
  *environmental gotchas*. (The third member of that list, the *arm choice*, was not
  absorbed but **abolished** — SL-254 left one spawn path, so there is no choice to
  carry in prose; amended SL-254.) Collating that residue into one spec — citing
  the verbs, never restating them — is higher-value than leaving it spread across the
  dispatch/worktree skills, four ADRs, and twenty memories.
- **Placement, not a ref-redirect, controls the claude base.**
  **AMENDED — FALSIFIED (SL-254, 2026-08-14):** the base is now pinned explicitly by
  `fork --base <B>` for every harness; nothing is inferred from the Bash cwd's HEAD,
  because there is no in-session `Agent` spawn to infer it. The hypothesis was true of
  the arm it described and is retained for the record: because `Agent`
  `isolation: worktree` forks the Bash cwd's HEAD, parking cwd on the coordination tree
  (HEAD==B) yielded `base==B` without any orchestrator-supplied base reaching a hook —
  empirically confirmed (SL-064 §8.6, controlled marker-commit test). Parking cwd in
  the coordination tree remains the practice, for the funnel-verb and `B`-capture
  reasons in *Coordination-tree placement*, not for the base.
- **Isolation by construction beats trust.** A dedicated coordination worktree makes the
  shared-main contention surfaces (dirty foreign index, foreign-WIP collisions) *unreach-
  able*, rather than defending against them per batch (SL-064, vs the SL-060 retrospect).
- **Parallel execution, serial landing.** File-disjoint phases run concurrently for
  throughput, but landing stays one-per-base under stationary-head — cheaper and crash-
  safe versus a parallel-landing re-anchor the orchestrator would have to prove each time.

## Decisions

- **D1 — the funnel cadence is a fixed ordered contract, report-and-halt.** The
  eight-step per-batch sequence and the no-auto-resolve discipline are owned here; the
  verbs are SPEC-012's. Knowledge records only after the confirmed code commit.
- **D2 — arm routing is deterministic and refuses on disagreement.**
  > **AMENDED — FALSIFIED (SL-254, 2026-08-14).** Superseded, not revised: there is no
  > arm routing left to be deterministic about. `DEC-208`/`DEC-213` collapsed dispatch
  > onto one confined subprocess spawn path for every harness; the
  > `doctrine.toml [dispatch] claude-force-subprocess-dispatch` key is deleted, and
  > `/dispatch-agent` and `/dispatch-subprocess` were merged into the single
  > `/dispatch-spawn` skill. What replaces this decision is *fail-closed confinement*:
  > an unresolvable jail aborts the spawn rather than selecting a lesser path. `REQ-288`
  > (the arm-routing requirement this decision carried) is retired by SL-254. The
  > historical decision is retained below for the record.

  `doctrine.toml`
  override, then env-marker; a self-belief↔env-marker mismatch refuses naming the cause.
- **D3 — parallel execution is first-class, landing is one-per-base (v1).** Serial-
  dependent phases self-base by advancing coordination HEAD before the next spawn.
- **D4 — the orchestrator runs on a dedicated coordination worktree inside
  the project root.** Always-on, per-run, concurrent same-slice refused; cwd parked
  there for the drive loop; worktree-life < branch-life. ("markerless" struck, amended
  SL-254 — no tree carries a marker now; write authority is the orchestrator process's
  absent `DOCTRINE_WORKER`, `DEC-207`.)
- **D5 — per-harness altitude is a uniform contract with honest non-uniform reach.**
  > **AMENDED — FALSIFIED (SL-254, 2026-08-14).** The reach is now uniform *and* the
  > contract is fail-closed, so there are no confessed residuals to be honest about:
  > every harness is a confined subprocess with an explicit `fork --base B`, the OS jail
  > as its write floor (writable set: the fork worktree **plus the harness config dir**),
  > `DOCTRINE_WORKER == "1"` as its whole identity, and a fail-closed refusal —
  > *named* on Linux (`bwrap-unavailable`), unnamed on macOS for want of a backend
  > probe — instead of a lesser rung (`DEC-207`, `DEC-208`). Uniform across harnesses;
  > **not** byte-uniform across platforms (network and process lifetime differ). The claude
  > half of this decision names four deleted things: the `Agent`-tool spawn, the disk
  > marker, the `SubagentStart` stamp, and `verify-worker`. `REQ-291` is rewritten by
  > SL-254 to state the uniform confined contract; the one **new** residual — the
  > unenforced configurable tail of `worker_forbidden_writes` after `worker_commit`'s
  > retirement (`DEC-204`) — is carried to SL-255 as IDE-051, and is not a per-harness
  > asymmetry. The historical decision is retained below for the record.

  codex/pi reach the full floor (explicit base, env catch, pre-dispatch verify, bwrap);
  claude reaches base==B-by-placement + post-spawn `verify-worker`, marker-only, fail-
  open SubagentStart stamp, no pre-dispatch verify — confessed residuals, not parity.
- **D6 — integration is two-stage and audit-gated.** Stage-1 materialises review refs +
  a CAS journal (no trunk write); audit gates; stage-2 is opt-in, ff-only, expected-tip-
  CAS, report-never-resolve.
- **D7 — the operational gotchas are durable constraints, not run accidents.** Proxy-safe
  git reads, checkout-import on patch corruption, never-widen-the-delta, proof-gated
  re-anchor, durable-git landed oracle, memory-on-trunk — each binds every future run.
