# SL-056 Design — Orchestrator spawn seam: worktree mechanism into CLI verbs

Scope: `slice-056.md`. Evidence base (all `research §N` cites below):
`.doctrine/slice/055/research/worktree-orchestration.md` — this slice is a sibling
of SL-055 and shares its research spine; cites resolve there, not under 056
(inquisition Charge X). Thesis: *mechanism in prose is the design smell* —
mechanism belongs in the CLI (identical under claude/codex/pi by construction);
judgment and harness concessions belong in prose. This design moves the
worktree/dispatch creation ladder, the import funnel, the solo land merge, build
isolation, and the worker-mode guard out of fail-open prose into fail-closed,
golden-testable CLI verbs, with **orchestrator-owned fork + a disk marker as the harness-agnostic
keystone**. The subprocess spawn seam (`codex exec`, pi self-subagent) is a
*codex/pi enhancement layer*, not the keystone — `claude -p` is API-billed +
harness-specific so claude runs the agnostic core via the `Agent` tool at
marker-only altitude (Charge XIII; see DC-1/DC-2 and the per-harness altitude
table in D7/G3).

The unifying principle: **the pure/imperative wall, lifted to the orchestration
layer.** The binary is the pure mechanism core; the harness spawn (a subprocess for
codex/pi, the `Agent` tool for claude — Charge XIII) is the thin impure shell. Every
decision below is an application of that wall.

## Locked decisions

Two cruxes were adjudicated before drafting:

- **DC-1 (seam boundary, per-harness spawn — Charge XIII).** The
  mechanism/concession line still falls between **what the binary does**
  (create-or-mark + provision + per-wt env *contract* emission — harness-identical)
  and **how the worker is spawned** (harness-shaped → prose, selected by the
  `/dispatch-*` router). The harness templates differ in *who creates the worktree
  and how identity is stamped*:
  - **codex/pi:** `doctrine worktree fork` creates the worktree, stamps the marker,
    provisions, emits the per-wt env contract; the orchestrator spawns the
    subprocess (`codex exec` / pi) with that env (+ `DOCTRINE_WORKER`, + bwrap).
  - **claude:** the `Agent` tool creates its *own* worktree (no dir param, no env
    seam), so the marker is stamped by the orchestrator-configured **WorktreeCreate
    hook** (ADR-006 D9) — disk identity, no subprocess, no env. Per-wt env is
    unreachable (degraded to jail-wide; D5).
  Rejected: marker-only-in-prose (leaves the creation ladder + identity self-armed —
  the very smell); a `claude -p` *required* backend (API-billed + harness-specific —
  Charge XIII); a full spawn verb pulling the harness invocation into the binary
  (re-couples + the config-knob Rube Goldberg ADR-006 D1 rejects).
- **DC-2 (worker identity — disk marker primary, env a codex/pi optimisation;
  Charge XIII).** Worker-mode is a property of the **worker**, signalled by a
  **disk marker the trusted orchestrator stamps** before the worker runs. Disk is
  the one identity substrate *every* harness has; an env seam is not (claude's
  `Agent` tool has none, and `claude -p` is non-viable — Charge XIII). So:
  - **fork marker (PRIMARY, harness-agnostic)** — at
    `.doctrine/state/dispatch/worker` (withheld runtime tier, self-labelling sibling
    dir). Stamped by `doctrine worktree fork` (codex/pi) or by the WorktreeCreate
    hook (claude). Fail-closed: present in the worker's linked worktree ⇒ writes
    refused.
  - **`DOCTRINE_WORKER=1` env (codex/pi OPTIMISATION)** — set by the orchestrator
    *only* on a subprocess it spawns. It buys one thing the marker cannot: it
    catches the **worker-on-main** hazard (ADR-006 D2b: harness drops the worker on
    the coordination root, where no fork marker exists and `is_linked_worktree` is
    false). For codex/pi this closes worker-on-main; **for claude it is
    unavailable**, so worker-on-main reduces to the already-deferred D2b residual,
    mitigated by always-isolating the worker (`Agent isolation:worktree`) + the
    hook-stamped marker — not closed. (The prior draft made env the *only*
    worker-on-main catch and *primary*; Charge XIII showed that collapses to
    fail-open on the dominant harness.)
  - **Guard:** refuse a write-classed OR `Orchestrator` verb when
    `(is_linked_worktree && marker_present) OR env DOCTRINE_WORKER set`. The marker
    conjunct is the agnostic floor; the env disjunct is the codex/pi worker-on-main
    catch. Solo `/execute` sets neither → writes freely (D6a: mode, not location,
    decides). Marker lifecycle is owned (DC-3 below; D2) and clearable (Charge II).
  - Rejected: env-primary (collapses to fail-open on claude — Charge XIII);
    marker-only with *no* env even for codex/pi (discards a free worker-on-main
    catch where the seam exists); git-dir marker (lower observability, no real
    gain).
- **DC-3 (verb privilege — fork/import/gc orchestrator-only; `marker --clear`
  deliberately not).** `fork`, `import`, and `gc` **mutate git refs and directories**
  (create/remove worktrees, delete branches, reap target dirs, `--force`).
  Classifying them `Read` because they spare the *authored TOML corpus* is a category
  error (inquisition Charge IV): it lets the untrusted worker delete branches,
  violating ADR-006 D2 (workers mutate **source only**). They are a new
  **`Orchestrator`** class, refused under worker identity (`marker OR env`) exactly
  as write-classed verbs are. Only the non-mutating helpers (`provision`,
  `check-allowlist`, `branch-point-check`) stay `Read` and open to workers.
  **`marker --clear` (Charge II) is a deliberate fourth class:** it mutates runtime
  state but is *not* `Orchestrator`-classed — locking the marker's only remover
  behind the guard the marker trips is the self-brick Charge II names. It is refused
  only by `DOCTRINE_WORKER`-env-set (a real codex/pi worker cannot self-unmark), by
  cwd-is-not-this-tree, and — when the tree is a **linked worktree** (where a marker
  is presumptively a worker's) — by the absence of an explicit **`--operator`**
  acknowledgement, **never by the marker conjunct**. The `--operator` ack is the
  accident-fence: a bare `marker --clear` cannot fire inside a linked worktree, so a
  stray-marker mishap or an absent-minded invocation is caught, while the team-mode
  coordination operator (D8, on a linked delta-branch tree) clears deliberately with
  `--operator` (Charge II's in-CLI remedy preserved). **Charge B (round 3) —
  confessed, not closed, for claude.** The env-lock is the mechanism that stops a
  *deliberate* self-clear; claude has no env seam, so a non-compliant claude worker
  can pass `--operator` and self-unmark — restoring write- and `Orchestrator`-class
  verbs to itself. This is **not** bwrap's job alone (bwrap is codex/pi-only, D6);
  for claude there is **no mechanism floor against a deliberate self-clear** until a
  free env seam or OS confinement (IDE-004 channels / userns-bwrap) lands. So claude
  worker-sole-writer is **accident-fenced + prompt-enforced, not malice-proof** — the
  altitude this design honestly claims for claude (DC-2/Charge XIII extended; stated
  in the G3/ADR-011 per-harness altitude table). **The marker is a file *inside* the
  worker's worktree (D2), and the env is shell-strippable (round-4 Charge β):** a
  *non-compliant* worker of **any** harness can `rm` the marker directly (never
  touching `marker --clear`) or strip the env disjunct with `env -u DOCTRINE_WORKER`,
  so the `--operator`/env-lock hardening fences **accidents, not malice, on every
  harness**. The malice containment on the **dispatch funnel** is the **import
  `.doctrine/`-rejection belt** (D3) — a dispatch worker's doctrine delta never funnels
  back **through `import`**, regardless of marker/env. **This is not an
  *unconditional* containment over *all* funnels (round-5 Charge ζ):** solo's `land`
  verb (D4b) is a **second, beltless** sanctioned funnel — solo is a trusted
  self-orchestrator (D6a) that *legitimately* lands doctrine, so a `.doctrine/` belt
  there is a category error. The belt's true scope is therefore **the import/dispatch
  path, conditioned on dispatch deltas routing through `import` and never `land`**.
  That condition is **mechanised, not prose**: **`land` refuses a fork bearing the
  worker marker** (`dispatch-fork`, D4b) — a misrouted orchestrator running `land` on a
  dispatch worker's branch is *named-refused*, not silently belt-bypassed. A
  *malicious* worker reaching `land` to funnel doctrine is the **already-confessed D2b
  raw-tree residual** (a worker that can reach the coordination root to run `land` can
  already write `main/.doctrine/` directly; under D6/bwrap it can reach neither),
  **not** a new capability. A genuine **mechanism floor** against a malicious worker
  is **D6-contingent for codex/pi**: bwrap denies out-of-tree writes **and ro-binds
  the marker path** (D6) so the confined worker cannot `rm` it; **absent D6, codex/pi
  is accident-fenced like claude.** **No harness claims a "full mechanism floor"
  unconditionally** (round-4 Charge β), and **no funnel claims unconditional
  doctrine-containment** (round-5 Charge ζ); the G3/ADR-011 altitude table states each
  honestly — import belt on the dispatch path; `land`-refuses-marker-fork on the solo
  path; bwrap+ro-marker for codex/pi under D6; claude deferred to IDE-004/userns-bwrap.

## D1 — `doctrine worktree fork` (codex/pi creation verb) + claude's hook path

**Current.** The `/worktree` skill prose drives a creation ladder (existing
isolation → Claude `WorktreeCreate` hook → `git worktree add` → work-in-place).
The dispatch worker *self-forks* rung-3 from prompt instructions — drift from
ADR-006 D9, which already mandates the orchestrator provision + baseline-verify
"before handing the worker its task." `DOCTRINE_WORKER=1` self-arm and
`CARGO_TARGET_DIR` have no spawn seam under Claude's `Agent` tool (no env seam).

**Target (codex/pi creation path).** One verb, run by the trusted orchestrator at
the source root. (For **claude** the orchestrator does not call `fork` — the `Agent`
tool creates its own worktree and the WorktreeCreate hook provisions + stamps the
marker; see the per-harness orchestrator-usage templates below — Charge XIII.)

```
doctrine worktree fork --base <B> --branch <name> --dir <path> [--worker]
```

Steps (all deterministic, harness-identical). **Compensating cleanup, not a true
transaction (Charge VIII)** — git mutations are not atomic, so any failure after
step 1 triggers a *best-effort* rollback: `git worktree remove --force` (a
provisioned fork is dirty — plain `remove` refuses it), `git branch -D`, reap the
target dir. The rollback is itself fallible; on a rollback failure the verb **reports
the leftover state by name and exits non-zero** — never a silent or success-coded
half-rollback. The goal is unchanged (no orphan dir, no **unmarked** silently-write-
allowed worktree), but the verb does not *claim* an atomicity git cannot provide:
1. `git worktree add -b <branch> <dir> <B>` (subsumes ladder rung 3; the native
   hook is demoted to opportunistic, G2(a)). Correct git syntax is
   **`-b <branch>`** for a new branch at `<B>` — `add <dir> <branch> <B>` (three
   positionals) is invalid git (inquisition Charge VI). Refuses if `<dir>` exists,
   `<branch>` exists, or `<B>` is not a valid commit. `<dir>` must be **unique per
   worker** (per branch, not per slice) and either outside the repo root or a
   gitignored in-repo path — else a concurrent same-slice batch collides and an
   un-ignored in-repo fork dirties the coordination tree, breaking the next
   `import` clean-precond (inquisition Charge VII; research §9 first-fork seam).
2. `doctrine worktree provision <dir>` (the existing sole-copier; withheld tier
   excluded by construction — unchanged).
3. If `--worker`: write the marker (D2) into the fork **before** any window in
   which a worker could be spawned. Solo `/execute` omits `--worker` → no marker.
4. Emit the **per-worktree env contract** on **stdout** (machine; one `KEY=value`
   per line); human status to **stderr**. The contract is *generalisable* — the
   project declares its per-wt env needs; doctrine-the-repo declares
   `CARGO_TARGET_DIR=<jail-root>/wt/<branch>` (D5, a project-local consumer, **not**
   a framework primitive — Charge XIII). For a codex/pi `--worker` fork the
   orchestrator additionally sets `DOCTRINE_WORKER=1` on the spawned subprocess —
   the **DC-2 codex/pi optimisation** (worker-on-main catch), *not* the identity
   (identity is the step-3 marker). Both are spawn-time env on a subprocess; claude
   (Agent tool, no env seam) consumes neither — its identity is the hook-stamped
   marker and it shares the jail-wide target (D5).

Orchestrator usage is the thin, harness-specific prose shell, selected by the
`/dispatch-*` router (Charge XIII). **Two templates:**

*codex/pi (`/dispatch-subprocess`)* — fork verb creates+marks+provisions; capture
and **check the exit code** before consuming env (`eval "$(cmd)"` swallows the
status — a fail-open trap, so we never use it):
```sh
fork_env="$(doctrine worktree fork --base "$B" --branch "$BR" --dir "$D" --worker)" \
  || { echo "fork failed: $?" >&2; exit 1; }   # halt, do not spawn
env DOCTRINE_WORKER=1 $fork_env codex exec "<pre-distilled prompt>"
#   ^ codex/pi worker-on-main optimisation   ^ per-wt env   ^ harness-shaped line
```
`$fork_env` is the stdout per-wt env block; status went to stderr. When D6 lands the
bwrap wrap (`bwrap <profile> env … codex exec …`) it extends *this prose shell*, not
the verb — confinement is a harness/OS concession, DC-1 keeps it in prose.

*claude (`/dispatch-agent`)* — no fork verb, no env. Before launching the worker the
orchestrator **arms a dispatch sentinel** via **`doctrine worktree marker --arm`** (a
transient signal in its own runtime tier the hook reads; **single-slot — refuses
`already-armed` if a sentinel is armed or a stamp is still awaited**, round-5 Charge
θ); it then launches the worker via the `Agent` tool with `isolation:worktree`. The orchestrator-configured WorktreeCreate hook (ADR-006 D9)
provisions the fork **and**, *only when armed*, stamps the marker — then **disarms**
(Charge C, round 3: the hook fires on **every** Agent worktree, so an unconditional
stamp would brand non-dispatch isolated worktrees and brick their writes; the
arm/stamp/disarm gate confines branding to dispatch workers). Marker-stamping is a
**distinct act** from allowlist provisioning and is **permitted to write the
provisioning-excluded `.doctrine/state/` tier** — D9's allowlist excludes that tier
from *copying*; the marker is *minted*, not copied. Identity is the hook-stamped
marker (disk); `DOCTRINE_WORKER` and bwrap are unavailable; the worker shares the
jail-wide target (D5); worker-on-main is the deferred D2b residual (DC-2). **Fallback
(Charge C): the arming-read and stamp-in-time are O3-spike-gated; if the spike refutes
them, claude dispatch degrades to prompt-enforced worker-sole-writer (no marker
mechanism)** — altitude-table-confessed, symmetric with D6's bwrap back-out.
**Concurrency constraint (round-4 Charge γ; mechanised round-5 Charge θ): the
arm/stamp/disarm gate is sound only under SERIAL claude dispatch — and serial-only is
a CLI mechanism, not orchestrator discipline.** The `Agent` tool returns **no
worktree handle**, so the orchestrator cannot bind an arming to the *specific*
WorktreeCreate it intends; under a concurrent batch a second `isolation:worktree`
fire would read the armed sentinel and brand the **wrong** tree (the intended worker
then unmarked → fail-open). So claude marker-stamping requires **one armed spawn in
flight at a time** — and that single-slot is **enforced by the arming verb itself**:
`doctrine worktree marker --arm` **refuses (`already-armed`) if a sentinel is already
armed or a stamp is still awaited** (a single-slot lock in the runtime tier). An
orchestrator that arms then fires the *blessed* parallel batch has its **second
`--arm` physically refused** → it either serialises (arm succeeds, launch **exactly
one** Agent, await stamp+disarm) or, going parallel, **cannot arm at all** → the
honest **prompt-enforced, no-marker** degrade. The **fail-open** armed-and-concurrent
middle round-4 left to orchestrator self-restraint (the *faith-not-works* this slice
condemns) is now **unreachable**: nothing fail-open is reachable by an orchestrator
obeying the blessing, because the second arm refuses. **Parallel file-disjoint claude
dispatch therefore degrades to prompt-enforced** (no marker, no arm), symmetric with
the D6/spike back-outs. The O3 spike is **widened** to confirm the serial
arm→stamp→disarm, **that a second `--arm` while armed refuses**, and that the
concurrent case produces **no second stamp** — so serial-only is mechanised and
evidence-based, not assumed (Charge III's "spike what you rely on").

**Why a verb, not the skill.** Steps 1–4 are pure mechanism; only the spawn line
(`env … codex exec` for codex/pi; `Agent`+hook for claude) differs across harnesses,
selected by the `/dispatch-*` router. The verb makes the spawn obey the same
no-impurity-in-the-core wall the rest of the codebase obeys.

**Pure/imperative split** (ADR-001 leaf). Pure: `target_dir_for_branch(jail_root,
branch) -> PathBuf`, marker path derivation. Imperative shell (`run_fork`): git
add, provision call, marker write, stdout/stderr emission.

## D2 — Worker-marker (fail-closed worker-mode guard)

**Current.** `main` checks `DOCTRINE_WORKER=1` env; `write_class` (exhaustive over
`Command`) bails on every authored/doctrine-mediated write. Fails open: only the
prompt can self-arm, nothing enforces it (ADR-006 D2b; research §3 C-I).

**Target.** The guard trigger is the **DC-2 signal — disk marker primary, env a
codex/pi optimisation** (Charge XIII). `write_class` itself is unchanged
(behaviour-preserving); a new `Orchestrator` class (DC-3) joins it under the same
guard.

```
marker path:  <root>/.doctrine/state/dispatch/worker
worker_mode(root)  :=  (is_linked_worktree(root) && marker_present(root))   // primary, agnostic
                       OR env DOCTRINE_WORKER set                            // codex/pi worker-on-main catch
guard (in run(), before dispatching a write-classed OR Orchestrator Command):
    if worker_mode(root):
        refuse(verb)        // names the verb, as today
```

- **Marker is the primary, harness-agnostic identity.** Disk is the one substrate
  every harness has; the orchestrator stamps the marker into the worker's worktree
  (via `fork` for codex/pi, via the WorktreeCreate hook for claude — Charge XIII)
  before the worker runs. Present in a linked worktree ⇒ writes refused.
- **Env is the codex/pi worker-on-main optimisation.** A worker the harness leaves
  on the coordination root (D2b hazard) carries no marker and is not a linked
  worktree — the marker conjunct is blind to it (inquisition Charge III). An
  orchestrator-set `DOCTRINE_WORKER` env catches it — but **only where a subprocess
  spawn carries env (codex/pi); claude has no env seam** (Charge XIII), so for
  claude worker-on-main stays the deferred D2b residual, mitigated by
  always-isolating the worker + the hook-stamped marker.
- `is_linked_worktree` is the existing predicate (two consumers today: memory
  squash-warn, RV-verb refusal — now three).
- The marker is **presence-only** — no contents. (The earlier "optionally the
  base SHA" is dropped: it was written and never read — dead/misleading state,
  inquisition Charge XI.)
- **Lifecycle (owned, not assumed — inquisition Charge V).** Written by
  `fork --worker` (with compensating-cleanup rollback, D1) for codex/pi, or by the WorktreeCreate hook
  for claude (Charge XIII); **removed by `gc`** (D4); rolled back if `fork` fails;
  **cleared by `doctrine worktree marker --clear`** for a stray marker on a tree the
  operator wants as coordination root (Charge II — a non-`Orchestrator` verb the
  guard cannot strangle; DC-3). A tree may serve as a coordination root only after an
  **assert-marker-absent** check; on a stray marker that check **refuses and names
  the remedy** (`marker --clear`), so detection now carries a cure, not just a
  diagnosis (Charge II). Marker-absence on the coordination tree is *guarded*, not
  presumed.
- **`marker --clear` (Charge II remedy; hardened by round-3 Charge B).** `doctrine
  worktree marker --clear` removes the marker at the cwd tree root and prints a loud
  receipt of what it cleared. Refused if `DOCTRINE_WORKER` is set (a codex/pi worker
  cannot self-unmark), if cwd is not the marker's tree root, and — when the cwd tree
  is a **linked worktree** (where a marker is presumptively a worker's) — unless an
  explicit **`--operator`** ack is given; **never** refused by the marker conjunct of
  `worker_mode` (the lock Charge II condemned). The `--operator` ack is an
  **accident-fence**: a bare clear cannot fire inside a linked worktree, so a
  stray-marker mishap is caught while the team-mode coordination operator (D8, a
  linked delta-branch tree) clears deliberately. Restores writes + `gc` to an
  orchestrator self-bricked by a stale marker, entirely in-CLI (no filesystem
  surgery). **Residual (Charge B, confessed): for claude the env-lock is absent**, so
  a non-compliant claude worker can supply `--operator` and self-unmark — claude
  worker-sole-writer is **accident-fenced + prompt-enforced, not malice-proof** (no
  env-lock, and bwrap is codex/pi-only, D6); the full mechanism floor is deferred to
  IDE-004/userns-bwrap. codex/pi: the env-lock is a real anti-self-clear mechanism.
  Stated in the G3/ADR-011 per-harness altitude table.
- **Solo `/execute` is a second direct-writer class (Charge VI).** D6a makes solo
  `/execute` a full self-orchestrator that writes doctrine state directly while in a
  linked worktree (`is_linked_worktree` true) — a stale marker in a reused dir would
  fail-close it exactly as it would a coordination root. So **assert-marker-absent
  (and the `marker --clear` remedy) gate *every* transition of a linked worktree into
  a direct-writer role — solo `/execute` included**, not only coordination-root
  promotion. The legitimate writer is defined by *write-mode*, not by the word
  "coordination."
- **Observability surface (required, not assumed):** `worker_mode` is surfaced by
  the CLI — minimally a line in `doctrine worktree` / status output ("worker fork:
  yes — writes refused; signal: env|marker") so the mode is discoverable without
  knowing the gitignored path.
- **D6a preserved.** The orchestrator (trusted, source root, marker absent —
  asserted) stamps the marker into the *worker* worktree before the worker exists
  (and, for codex/pi, additionally sets the worker's env at spawn). Solo `/execute`
  carries neither → writes freely. Mode, not location, decides.
- Withheld tier: `.doctrine/state/**` is already gitignored, already dropped by
  `provision`, already absent from the import delta — the marker inherits all
  exclusions with zero new logic. The new `dispatch/` sub-path needs no separate
  tier entry (the `State` glob `.doctrine/state/**` already covers it; confirm in
  `is_withheld` test).

`DOCTRINE_WORKER` env is **retained as a codex/pi optimisation** (DC-2 / Charge
XIII), not the identity — the disk marker is identity. It is **orchestrator-set on
the spawned subprocess**, never prompt-self-armed; its sole job is the
worker-on-main catch, available only where a subprocess seam carries env (codex/pi,
not claude). Tests that unset it
(`[[mem.pattern.dispatch.worker-verify-unset-doctrine-worker]]`) still run the green
gate with `env -u DOCTRINE_WORKER` *and* outside a marked linked worktree, so
neither guard signal trips in a tempdir fixture.

**Env blast-radius bound (Charge XI).** Now that identity is the marker (not env —
Charge XIII), the env leg is small, but a *leaked* `DOCTRINE_WORKER` must not
silently fail-close legitimate main-side authoring or self-abort the dispatch. Two
rules: (a) `DOCTRINE_WORKER` is set **only in the spawned child's env**
(`env DOCTRINE_WORKER=1 … codex exec`), **never `export`ed into the orchestrator's
shell** — a hard rule, not an example; (b) the orchestrator **never sets the var on
itself** (acquittal: it is the top-level process), so any `DOCTRINE_WORKER` it reads
in its *own* env is a leak by construction — before any `Orchestrator`-classed funnel
verb it **asserts its own env clean and fails loud with a named error**,
defence-in-depth on the orchestrator path. **And the guard itself names the cause
(round-3 Charge D).** Whenever the **env disjunct** trips `worker_mode` on a tree
that is **not a linked worktree** — main, a plain checkout: a place a genuine worker
fork never is — the env is provably either a **worker-on-main** (DC-2's target) *or*
a **leak**, the two being the same ambient state. So for **every** guarded verb
tripped this way — write-classed *authoring* (`slice new`/`design`/`plan`) **and**
`Orchestrator` funnel verbs alike, not the funnel verbs only — the refusal carries a
**named dual-cause message** ("`DOCTRINE_WORKER` set outside a worker worktree: a
worker was dropped on the coordination root → re-dispatch isolated; **or** the env
leaked into this process → unset it"), never a bare "worker refused." This closes
Charge XI's masquerade on the **authoring** path it was raised about — a leaked var
bricking a concurrent main-side `slice new` now reports the leak by name. The test
discipline extends to both the authoring and the orchestrator paths.

## D3 — `doctrine worktree import` (the funnel belt)

**Current.** ~60 lines of dispatch prose replay: precond (tree clean + `HEAD==B`)
→ net diff `B..S` → assert `S^==B` → single-non-merge check → R-5 `.doctrine/`
name-only belt → `git apply --3way --index` non-committing. Fail-open prose; the
R-5 belt is called "the real protection" yet lives as an instruction.

**Target.** One fail-closed, golden-testable verb:

```
doctrine worktree import --base <B> --fork <branch>     # runs at coordination root
```

`Orchestrator`-classed (DC-3) — refused under worker identity. **This funnel is the
`/dispatch` path only** (a single distilled worker commit applied with `apply --3way`,
ancestry severed); **solo `/execute` does not use it** — it lands its multi-commit
TDD branch onto the coordination branch via **`doctrine worktree land` (--no-ff,
D4b)**, so the `multi-commit` refusal below is a dispatch-funnel constraint, not a
solo one (round-3 Charge E / round-4 Charge α; see D4b/D4).
Mechanical sequence, **v1 is the stationary-head case only** (inquisition Charge II;
A2 struck — see below), each step a hard refusal on violation (no auto-merge, no
judgment):
1. precond — **two guards, neither assumed** (Charge V): `HEAD == B`
   (`branch-point-check` — a **ref-equality** compare, blind to the working tree)
   **and** the coordination tree is **clean** (a separate `git status --porcelain
   --untracked-files=no`-empty check — tracked modifications + staged changes only —
   which `branch-point-check` does *not* perform). Untracked files are **excluded
   deliberately** (round-3 Charge F): they cannot affect a tracked-delta `git apply`,
   and the repo's ordinary working state carries benign untracked paths (gitignored
   scratch, memory items, withheld review sheets) — refusing on them would
   false-`tree-unclean` the common case. `HEAD != B` → refuse `head-moved`
   (orchestrator re-dispatches from the moved HEAD — no in-verb re-anchor in v1; see
   the quiescence constraint below — XII). Dirty tree → refuse `tree-unclean`.
2. `S^ == B` assert (single-non-merge fork delta) — else `multi-commit`.
3. R-5 belt: reject if the `B..S` **name-only** diff touches any `.doctrine/`
   path — else `doctrine-touch`. Match semantics pinned: prefix-match on
   `.doctrine/` over the name-only diff (tracked files only — gitignored
   runtime/derived never appears in a diff, so "all `.doctrine/`" and
   "authored-only `.doctrine/`" coincide in practice; the test pins this). A
   forced-added marker would therefore also be caught — defense in depth.
4. `git apply --3way --index` (non-committing). Under **both** preconds — `HEAD == B`
   *and* tree-clean (step 1) — the patch `B..S` applies onto the exact tree it was
   cut from, so it **cannot conflict**; `apply-conflict` is therefore **not** a v1
   refusal reason (purging it — round-1 Charge II). The purge is now sound on **both**
   conjuncts, not just the ref-equality one — the `tree-unclean` guard closes the gap
   Charge V found (a dirty tree was the unhandled `apply-conflict` path). The
   orchestrator commits separately (ADR-006 D7 cadence preserved — import ≠ commit).
5. **No runtime receipt is stamped (Charge I, round 2).** The round-1 design stamped
   an `{base, fork-head}` receipt here, at *apply* time — but a flag born before the
   separate commit, living in the gitignored runtime tier, survives a
   crash-before-commit and lies "landed" to `gc`, which then reaps unmerged work (the
   exact hazard `gc` exists to prevent). Instead `gc` derives landed-ness from
   **durable git state** after the orchestrator commits (D4 patch-id oracle) — no
   apply-time flag outlives the commit it would certify.

**Refusal set (v1, exhaustive over permitted states):** `{head-moved, tree-unclean,
multi-commit, doctrine-touch}`. Each is machine-readable on a non-zero exit; the
orchestrator skill acts (re-dispatch / report+halt).

**Moved-HEAD re-anchor — deferred to a follow-up (A2 struck).** §5.4's
moved-shared-main case (`git apply --3way` of `B..S` onto a *moved* HEAD, then
re-anchor on a disjointness proof,
`[[mem.pattern.dispatch.reanchor-base-on-disjoint-head-move]]`) is **out of v1
scope**. v1 refuses `head-moved` and re-dispatches — truthful and shippable. The
in-verb moved-head path (`--allow-reanchor`, with the computable path-disjointness
test) is a **named follow-up (IMP-043)**, *not* fail-open prose. This strikes the
contradiction the inquisition caught: the original design claimed both "the verb
must encode re-anchor" (scope A2) and "adjudication stays prose" (OQ-1). v1 claims
neither — it honestly handles only the stationary case.

**Quiescence constraint (Charge XII — named and enforced, not assumed).**
Stationary-head v1 import **requires a coordination branch with no concurrent
external committers.** In solo mode the coordination branch *is main*, and
concurrent design work on main is *expected*
(`[[mem.system.coordination.concurrent-design-shared-main-worktree]]`): each external
commit moves HEAD to `B+1` and forces every in-flight worker's import to refuse
`head-moved` → re-dispatch → which the next commit invalidates again — **livelock
under ordinary activity**. The constraint: **a live main mandates delta-branch
coordination (ADR-006 D8 team mode)**, which isolates the funnel from main churn;
solo-on-main dispatch is safe only when main is quiescent for the run. The
orchestrator **detects** a moved coordination HEAD via the existing branch-point
guard and **reports the external mover by name** rather than silently re-dispatching
into a livelock. The cheaper in-verb re-anchor (IMP-043) is the real fix; until it
lands the constraint is *stated and enforced* (G4/SPEC-012), not assumed.

Pure core: classification over a diff (`classify_import(diff, base, head) ->
Result<Apply, Refusal>`); imperative shell drives git + apply (no receipt write —
Charge I, round 2).

## D4 — `doctrine worktree gc`

**Current.** "GC the dispatch debris" — one prose sentence, no owner (IMP-041).
Stale `env!(CARGO_MANIFEST_DIR)` binaries strand after removal
(`[[mem.pattern.dispatch.worktree-removal-stale-manifest-dir-false-red]]`).

`Orchestrator`-classed (DC-3) — refused under worker identity (a worker must not
delete branches; inquisition Charge IV). **Target.** `doctrine worktree gc --fork
<branch> [--force]` reaps, in one act:
1. `git worktree remove` the spent fork dir (removing its marker — DC-2/D2
   lifecycle).
2. delete the fork branch with `git branch -D` (the funnel branch is never a
   git-ancestor, so `-d` would *always* refuse — `-D` is mandatory, which is
   exactly why the **patch-id gate** below is the real safety, not `-d`'s
   merged-check).
3. **reap the `wt/<branch>` target dir** (closes the D5 disk loop — IMP-041 and
   D-B1 hygiene are the same verb).
4. warn (stderr) that `env!(CARGO_MANIFEST_DIR)`-baked test binaries need
   recompile before the next close-time `just check`.

**The "landed" oracle — durable patch-id, not a runtime receipt or tree diff
(Charge I, rounds 1+2).** `--merged` is wrong (the apply-funnel branch is never a
git-ancestor). The
*replacement* the self-review reached for — **delta-emptiness** (`git diff
<B-or-HEAD>..<fork>` empty ⇒ safe) — is **also unsound**, and was rejected under
cross-examination:
- `git diff B..fork` is the worker's whole delta — **never empty** for a fork
  that did work ⇒ gc refuses *every* imported fork.
- `git diff HEAD..fork` after the batch commit is `diff (B+1)..S`; the instant a
  sibling moves the coordination HEAD (§5.4, the *common* case) the tree
  legitimately diverges ⇒ non-empty ⇒ gc refuses a spent fork.
- Either way the operator learns the `--force` reflex and the safety gate
  collapses to "delete whatever I point at" — reaping unmerged work, the exact
  hazard gc exists to prevent.

**v1 resolution (Charge I, round 2): a durable patch-id reachability check, no
runtime receipt.** gc deletes **only** when the fork's commit has *provably landed*
on the coordination branch — tested by **patch-id equivalence against durable git
state**, not a runtime-tier flag. Concretely `git cherry <coordination-HEAD>
<fork-branch>` (merge-base computed internally, so no `--base` is needed — this also
disposes Charge IX) lists every commit in the fork's `B..fork` range, each marked `-`
(patch already present in coordination's history) or `+` (not present). gc reaps when **either** leg confirms
landing: **(ancestry)** `<fork-tip>` is an ancestor of `<coordination-HEAD>` (the
non-squash `land` case — D4b — fork commits reachable), **OR** **(patch-id)** every
commit listed by `git cherry` is `-` (the `apply --3way` dispatch case — ancestry
severed, so reachability is gone but each commit's *patch* landed). A **non-ancestor
tip with any `+`** ⇒ not (fully) landed ⇒ refuse unless `--superseded-head`/`--force`.
The two-leg union (round-3 Charge E, sharpened round-4 Charge α) is what lets one gc
serve both callers: dispatch's single distilled commit (`apply --3way` → patch-id
`-`) and solo's multi-commit branch (`land --no-ff` → reachable → ancestry true; a
partial merge leaves it neither-ancestor-nor-all-`-` and gc refuses). **A squash-merge
is structurally uncertifiable** (round-4 Charge α: a squash destroys *both* ancestry
*and* per-commit patch-id, leaving only the combined tree-diff this design already
rejected as an oracle) — so solo **must** land via the non-squash `land` verb (D4b);
a manually squash-merged fork trips **neither** leg, and gc refuses with a **named**
message ("cannot certify a squash-merge — re-land via `worktree land` (--no-ff), or
`--force` knowingly"), never a silent `--force`. This survives the two failure
modes that sank the alternatives: it is robust to a sibling moving HEAD (patch-id
matches the *commit's patch*, not a whole-tree diff — §5.4 no longer false-refuses)
and to `apply --3way` severing ancestry (patch-id ≠ ancestry, so a non-ancestor
applied commit still matches). Crucially it is **crash-proof**: a crash between apply
and commit leaves no commit on the coordination branch, so `git cherry` reports `+`
(not landed) and gc **refuses** — the round-1 receipt, born at apply-time in the
gitignored tier, would have lied "landed" and reaped the only copy. No receipt means
no receipt lifecycle to own (disposing Charge IV) and no receipt key to specify
(Charge IX).

**Observability (Charge X):** `gc --fork <b>` (and a `--dry-run`) prints the live
patch-id verdict per fork — "`<b>`: landed ✓ / not-landed — `--force` to reap" —
computed from git, so the operator never defaults to `--force` blind.

**Superseded forks — a non-`--force`, no-stored-flag disposition (Charge VII;
re-armoured by round-3 Charge A).** Moved-HEAD re-dispatch is the *common* case (XII),
and a re-dispatched fork genuinely *is* spent yet never landed → patch-id `+` → bare
gc would demand `--force`, training the very reflex the landed-oracle exists to kill.
The round-2 fix — a **stored `superseded` runtime record** — was itself the burned
receipt in new robes (round-3 Charge A): a disposable, crash-surviving, name-matchable
flag with no removal owner, gating `branch -D` on its mere *presence*. **Struck.**
Instead, re-dispatch reaps the spent fork by **naming its exact commit**: `gc --fork
<branch> --superseded-head <SHA>` reaps without patch-id-landing **iff** the supplied
`<SHA>` **equals the branch's current head**. **This is not an oracle (round-4 Charge
δ):** it is an **operator assertion** that this exact, still-current commit is
spent-and-abandoned; the head-match is a **TOCTOU movement-guard** (the named SHA
still points where the operator last saw it — not a stale name), **not a proof of
landing**. It buys strictly more than blind `--force` — a named-SHA audit trail and a
moved-branch catch — but it does **not** certify the work landed; it reaps *unlanded*
work on the operator's word. The orchestrator holds the abandoned fork-head
SHA in its own **disposable** re-dispatch context (the abandoned branch is untouched
after abandonment, so its head stays that SHA) and passes it at gc time; it stamps
**no durable-tier flag**. This is fail-safe in **both** directions: a *lost* SHA
(orchestrator crash/handover) only costs a `--force` — gc refuses, the safe side, so
Charge A's crash-reaps-live-work hazard cannot fire, because nothing reaps without a
*live-supplied, head-matching* SHA; and a *wrong* SHA cannot match a live fork's head
unless it *is* that head, which is the operator deliberately naming it. No stored
record ⇒ no removal owner to forget (disposing the re-incurred Charge IV) and no
branch-name key to false-match across a `gc`-freed name (disposing the re-incurred
Charge IX). `--force` stays reserved for the genuinely-unknown fork. (Observability,
D4: `gc --dry-run` shows "not-landed" so the operator names the SHA or `--force`s
knowingly, never blind.)

Cleanup ownership becomes trivial: **the caller of `fork` owns `gc`** — but the two
callers land their work by **different routes**, and gc's two-leg oracle (above)
spans both (round-3 Charge E / round-4 Charge α). **Dispatch** funnels a single
distilled commit through `import` (`apply --3way`, ancestry severed) → gc's patch-id
leg. **Solo `/execute`**
does *not* use the import funnel: it lands its multi-commit TDD branch onto the
coordination branch (main, D8) via **`doctrine worktree land` (--no-ff, D4b)** —
ancestry preserved, never squashed (round-4 Charge α), so the `multi-commit` `import`
refusal (D3) is a dispatch-funnel constraint, not a solo one. After either route gc
reaps on the same two-leg oracle (ancestry for `land`, patch-id for `import` — D4).
`/dispatch` concludes with gc; solo `/execute` ends with `land` then gc.

## D4b — `doctrine worktree land` (solo `/execute`'s coordination merge)

**Current.** Solo `/execute`'s land-to-coordination step was **unmechanised prose**
("normal git merge") — the very *mechanism-in-prose* smell this slice exists to burn
(round-4 Charge α), and it silently assumed a non-squash merge the gc oracle depends
on. ADR-006 Consequences even anticipates a solo *squash*-merge (anchor-orphaning),
so "merges normally" was an unenforced wish, not a guarantee.

**Target.** Solo's analog of dispatch's `import` — a fail-closed verb that lands a
solo isolated-worktree TDD branch onto the coordination branch, **structurally
non-squash**, so gc's ancestry leg (D4) *and* memory-anchor sha-stability (ADR-006
D8) both hold:

```
doctrine worktree land --fork <branch>     # runs at the coordination root
```

`Orchestrator`-classed (DC-3) — it mutates coordination git refs (a merge commit),
refused under worker identity. **Solo-only:** dispatch uses `import` (apply-3way,
single distilled commit, ancestry severed by design — ADR-006 D7, import≠commit);
`land` is for solo `/execute`'s multi-commit branch (ancestry **preserved**). The
**in-place** solo path (D6a: solo on trunk, no worktree) needs no `land` — it commits
directly; `land` applies only to the **isolated-worktree** solo path.

Mechanical sequence, each step a hard refusal on violation (no auto-resolve):
1. precond — coordination tree clean (`git status --porcelain --untracked-files=no`-
   empty, **same scoping as import**, round-3 Charge F) else `tree-unclean`; HEAD is
   the coordination branch; `<branch>` exists else `no-such-fork`; **`<branch>`'s
   linked worktree (if any) does *not* bear the worker marker** else `dispatch-fork`
   (round-5 Charge ζ — `land` is solo-only; a marker-bearing fork is a dispatch worker
   whose delta must funnel through the belted `import`, never `land`'s beltless merge).
2. `git merge --no-ff <branch>` — **never `--squash`** (the verb cannot express a
   squash; that is its entire reason to exist). Ancestry preserved ⇒ fork commits
   reachable from the new coordination HEAD ⇒ gc's ancestry leg reaps (D4).
3. on a merge conflict (genuine code coupling, or coordination moved under the run) →
   **`git merge --abort`** first (restore the clean tree `land`'s own step-1 precond
   demands), *then* refuse `merge-conflict`, **report + halt** — never auto-resolve.
   The abort makes the refusal a *true* mirror of import's leave-nothing-behind
   report-don't-merge posture (ADR-006 D2): solo fixes the coupling **at source** and
   **re-runs `land`** onto a clean coordination tree. **Round-5 Charge η:** `git merge`
   mutates the index/tree and sets `MERGE_HEAD` *before* it reports the conflict (where
   `git apply` under import's preconds never mutates), so the half-merge **must** be
   aborted — else it wedges the coordination tree against the verb's own `tree-clean`
   re-entry guard (and against every other `Orchestrator` verb) until manual surgery,
   the exact toil this verb family abolishes.

**Refusal set:** `{tree-unclean, no-such-fork, dispatch-fork, merge-conflict}` —
machine-readable on a non-zero exit.

Pure core: precond classification (`classify_land(tree_status, head, fork_exists) ->
Result<Merge, Refusal>`); imperative shell drives `git merge --no-ff`. Reuses the
tree-clean check shared with `import` (no parallel implementation).

**Symmetry restored (round-4 Charge α).** Both landing routes are now CLI verbs —
`import` for dispatch, `land` for solo — so the slice thesis (*mechanism in the verb,
not prose*) holds for **both** callers, and gc's two-leg oracle (D4) has a verb on
each side. The unmechanised-prose hole is closed at its root, not papered over.

## D5 — Per-worktree env provisioning (generalisable) + build isolation (its project-local instance; ≡ ADR-008 D-B1)

The framework primitive is the **per-worktree env contract** D1 emits: the project
declares its per-wt env, the orchestrator injects it where the spawn backend carries
env. Doctrine-the-repo's instance is build isolation —
`CARGO_TARGET_DIR = <jail-root>/wt/<branch>`, computed by `fork` (pure
`target_dir_for_branch`; branch names carry `/` — `slice/SL-056-x` → nested
`wt/slice/SL-056-x`, which cargo accepts; collision-safe since branch names are
unique). This is **ADR-008 / project-local, not an ADR-011 framework primitive**
(Charge XIII).

**Harness-conditional deliverability.** Not baked in the flake (ADR-008 D-B5); cargo
env-precedence means a fork-resident `.cargo/config.toml` cannot override the
ambient jail-wide var — only spawn-set env can. So:
- **codex/pi:** the orchestrator sets it at subprocess spawn → per-wt isolation;
  obsoletes the three §5.1 mitigation rituals. Warm across launches (in-jail
  `~/.cargo` persists) → cold cost is per-branch, not per-session; disk residual
  reaped by D4. **No flake change for the spike.**
- **claude:** no env seam → the worker **shares the jail-wide target** and the three
  §5.1 mitigation rituals (touch+re-run, fingerprint-rm, cordage recompile)
  **stand**. Build isolation is a perf/false-green concern, not a trust signal, so
  this is *confessed, not closed* (Charge XIII). A future channels backend (IDE-004)
  could lift it.

## D6 — Per-worker bwrap confinement (ADR-008 D-B3, spike — codex/pi-only)

Timeboxed spike, OS-enforced discharge of ADR-006 D2b. **Subprocess-only (Charge
XIII):** bwrap wraps a spawned process, which claude's in-session `Agent` tool is
not — so this rung is codex/pi-only; for claude, D2b stays the deferred residual.
Feasibility gate is unprivileged userns *inside* the jail (outer bwrap may
seccomp-block
`clone(CLONE_NEWUSER)`) — empirical, probe `bwrap --unshare-user --ro-bind / / true`
at spike time. Land → worker rw-mounts only its worktree + target dir, ro
everything else (a write to main's `.doctrine` denied by the OS). **The profile also
`--ro-bind`s the marker path (`.doctrine/state/dispatch/worker`) over the otherwise-rw
worktree (round-4 Charge β)** — so the confined worker **cannot `rm` its own marker**,
making the codex/pi mechanism floor real (out-of-tree denial **and** marker
immutability), not just out-of-tree denial. This is what makes DC-3's codex/pi floor
**D6-contingent**: absent this bind the marker is a removable in-tree file and
codex/pi degrades to accident-fenced (the import belt remains the unconditional
containment, D3). Too costly →
back out to D5 + the D2 marker guard, leave D2b deferred. Depends on D5. `bubblewrap`
is pre-staged in `jailPkgs`; the only added surface is a `dispatch-worker` bwrap
profile.

## D7 — Governance deliverables (produced as design outputs, sequenced)

Decisions govern → land first; the design *produces the drafts*, code consumes
them. Sequence: **G1+G3 → O3-guard-spike → G2 → G4 → remaining code** (the
DC-2/DC-3 guard+privilege spike precedes the ADR-006 amend it validates —
inquisition Charge IX).

- **G1 — ADR-008 revise→accept** (the gate). Fold §5.1 evidence; record D-B2 as
  standing fact (ro `~/.cargo/bin` ⇒ no in-jail install, no race); re-scope D-B3
  around the userns question. Acceptance gates IMP-004.
- **G2 — ADR-006 amend.** (a) D5/D9 ladder: demote the native hook as a *creation*
  preference (base-pinning + subprocess spawn supersede it for codex/pi), cite
  SL-050/051 — **but promote it as claude's marker-stamping seam** (the
  WorktreeCreate hook provisions + stamps the marker; Charge XIII). (b) D2a
  mechanism: replace the `DOCTRINE_WORKER=1` *self-arm* with the **DC-2 signal —
  disk marker primary** (harness-agnostic), env a **codex/pi optimisation** (the
  worker-on-main catch), plus the DC-3 `Orchestrator` verb class. (Not "env→marker"
  wholesale, nor "env-primary" — marker is primary and agnostic; env is retained as
  a codex/pi-only enhancement, its arming moved from prompt to orchestrator —
  Charges III/XIII.) State the **per-harness enforcement altitude** in D2b (claude:
  marker-only, worker-on-main deferred; codex/pi: full). Withheld-tier D1/D4/D9
  invariants preserved. **Spike-first (Charge IX):** the guard + privilege model
  (DC-2/DC-3) *and* the claude marker-via-hook path are validated by a small O3 code
  spike *before* G2 amends the accepted ADR — symmetry with the D6 bwrap spike-first.
  Governance follows proven mechanism, not the reverse.
- **G3 — ADR (new): the spawn-seam contract + per-harness capability profile.** ADR
  id allocated via `doctrine adr new` at authoring (likely ADR-011 — next free —
  not hardcoded). Records a **harness-agnostic contract** (orchestrator owns
  fork-or-mark + provision + per-wt env emission; worker identity is the disk
  marker) and a **per-harness capability profile + altitude table** (Charge XIII):
  codex/pi = subprocess spawn buys env-arm + per-wt env + bwrap (full); claude =
  `Agent` tool, marker-via-hook, marker-only altitude (no env, no per-wt target, no
  bwrap), with `Agent` a **first-class** backend, not a degraded rung. **Round-3 B/C
  sharpen claude's floor honestly:** the marker is **accident-fencing +
  prompt-enforced**, not malice-proof — there is no env-lock against a deliberate
  `marker --clear` (Charge B), and the marker-via-hook stamp is O3-spike-contingent
  with a prompt-enforced fallback (Charge C). So claude's mechanism floor against a
  *non-compliant* worker is deferred to IDE-004 channels / userns-bwrap; the altitude
  table **says so plainly** rather than claiming a fail-closed inversion claude does
  not yet reach. **No harness-specific command (`claude -p`) is a required element.** The
  **env-reliability claim stays `proposed`** until the O3 propagation gate is green
  (Charge III) — governance trails proven mechanism. ADR-006-references;
  framework-level (harness-agnostic).
- **G4 — SPEC-012 rewrite.** Reframe Overview + Concerns (drop "the funnel is a
  discipline, not enforced code" — now enforced); rewrite D3 (fail-open env →
  fail-closed **marker-primary** guard) and **state the achievable enforcement
  altitude per harness** (Charge XIII) — no uniform fail-closed claim; **state the
  quiescence constraint** (v1 dispatch requires a non-churning coordination branch; a
  live main mandates delta-branch coordination — Charge XII); **state the solo
  non-squash-land constraint** (solo `/execute` lands via `worktree land --no-ff`; a
  squash-merge is structurally uncertifiable by gc — round-4 Charge α); **state the
  belt's scope** (the `.doctrine/` belt is the **dispatch/import-path** containment,
  **not** an all-funnel one — `land` is a solo-trusted, beltless funnel guarded by a
  `dispatch-fork` marker refusal — round-5 Charge ζ); add a D for the verb family; add
  FRs (fork, import, **land**, gc, marker guard, per-wt env contract).

Untouched: ADR-007, ADR-001/003/004, the withheld-tier model.

## Code impact

| Path | Change |
|---|---|
| `src/worktree.rs` | `run_fork`, `run_import`, `run_gc`, `run_land` (round-4 Charge α — solo's non-squash merge; **`git merge --abort` on conflict** before refusing, round-5 Charge η), `run_marker_clear` (imperative shells, **compensating-cleanup** fork rollback — `remove --force`, honest non-zero on rollback failure, Charge VIII); pure: `target_dir_for_branch`, `marker_path`, `classify_import`, `classify_land` (round-4 Charge α — reuses the shared tree-clean check, no parallel impl; refuses `dispatch-fork` on a marker-bearing fork, round-5 Charge ζ). gc landed-oracle is **two legs** — `git merge-base --is-ancestor` (the `land` route) **OR** a `git cherry` patch-id check (the `import` route), not a runtime receipt (Charge I / round-4 Charge α); a squash-merge trips neither and gc refuses with a named message. Reuse `select_copies`/`branch-point` core. New `write_marker`/`marker_present`/`remove_marker` (`write_marker` also invoked by claude's WorktreeCreate hook — Charge XIII; `remove_marker` behind `marker --clear` — Charge II). Third `is_linked_worktree` consumer. |
| `src/main.rs` | `fork`/`import`/`gc`/`land` subcommands + arg structs (watch the bool/arg clippy ceilings, `[[mem.pattern.lint.cli-handler-args-struct]]`). Worker-mode guard = `worker_mode(root)` = `(is_linked_worktree && marker_present) OR env DOCTRINE_WORKER set` — **marker primary, env a codex/pi optimisation** (DC-2 / Charge XIII). `write_class` unchanged. **fork/import/gc/land are a new `Orchestrator` class — refused under `worker_mode`, NOT `Read`** (they mutate git refs/dirs; inquisition Charge IV / DC-3 / round-4 Charge α — `land` writes a coordination merge commit). A marker-stamping entry point (claude WorktreeCreate hook, gated by an orchestrator **arming sentinel** via `marker --arm`, a **single-slot lock** refusing `already-armed` — round-5 Charge θ, **serial-dispatch-only** — round-3 Charge C / round-4 Charge γ) + a marker-clear path (Charge II) join the verb family. `gc` gains `--superseded-head <SHA>` (round-3 Charge A — an operator **assertion** of a spent-and-abandoned head, TOCTOU-guarded; **not a landed oracle** — round-4 Charge δ) and a **two-leg** landed check (`--is-ancestor` for `land`, `git cherry` for `import`; squash → named refusal — round-4 Charge α); `marker --clear` gains `--operator` (Charge B); the worker-mode guard's env-leg refusal on a **non-linked** tree carries the named dual-cause message for authoring **and** funnel verbs (Charge D). |
| `src/git.rs` | new reads behind the verbs: worktree list, **patch-id reachability** (`git cherry`, gc landed-oracle — Charge I), `B..S` diff name-only (import). Impure seam only. |
| ADR-008 / ADR-006 / **ADR-011 (new)** / SPEC-012 | G1–G4. |
| `plugins/doctrine/skills/{worktree,dispatch,execute}/SKILL.md` + new `{dispatch-subprocess,dispatch-agent}/SKILL.md` | rewrite prose to *call* the verbs (the token/agnostic payoff); **`/dispatch` becomes a harness router** → `/dispatch-subprocess` (codex/pi) \| `/dispatch-agent` (claude), Charge XIII. **Routing input (round-3 Charge G, mechanised round-4 Charge ε, spike-gated round-5 Charge ι): the dispatching agent's harness self-knowledge, cross-checked against env-marker detection.** Self-belief (it runs *as* claude/codex/pi) is no longer trusted alone — *belief alone routes nothing*. The orchestrator (which runs bash) probes **env markers** (`CLAUDECODE` for Claude Code; the codex/pi equivalents — precise names resolved in-skill/at the O3 spike, not hardcoded in the binary) and routes **only when detection *agrees* with self-belief**; **mismatch *or* unknown → refuse**, never guess (no blind `claude -p`/`codex exec`). **The refusal names the cause** (round-5 Charge ι: "env marker for claimed harness `<h>` not found — harness mis-seeded, renamed, or launch-mode-stripped; dispatch refused", never a bare "refused"), so a marker-name drift is diagnosable, not a silent brick. **The detection *signal* is itself spike-gated, symmetric with C and D6 (round-5 Charge ι):** the O3 spike must confirm a **stable, harness-unique, launch-mode-robust** marker exists per harness (env markers vary across headless/cron/nested/IDE launches — version-fragile, cf. `[[mem.pattern.parse.toml-error-classification-fragile]]`). **Named fallback if the spike finds no reliable marker:** the router **refuses *all* dispatch with that diagnostic** rather than silently reverting to self-belief-only (which would reopen ε); the operator falls back to manual or a corrected marker name. The ε cross-check claim stays **`proposed` until the marker-existence gate is green** (symmetric with the env-propagation claim, G2/G3). This closes the *confident-misidentification* gap "unknown⇒refuse" left open (a wrong-but-certain belief now fails the cross-check instead of routing to the wrong spawn). **No duplicated cadence (Charge G):** the funnel cadence *is* the CLI verb sequence (`fork`→`import`/`land`→verify→branch-point→one commit→`gc`/record), called **identically** by both sub-skills — the slice's whole thesis; the sub-skills differ only in the ~2-line spawn template. Re-embed ritual `[[mem.pattern.distribution.skill-refresh-command]]`. |
| `flake.nix` | none for the spike; `dispatch-worker` bwrap profile only if D6 lands (the profile `--ro-bind`s the marker path so the confined worker cannot `rm` it — round-4 Charge β). |

## Verification alignment

- **Black-box CLI goldens** (`[[mem.pattern.testing.black-box-cli-golden]]`,
  `force_no_tty`): `fork` (env on stdout, status on stderr, marker written);
  `import` happy path + each refusal (`head-moved`, `tree-unclean`, `multi-commit`,
  `doctrine-touch` — `apply-conflict` purged round-1 Charge II); `land` happy path +
  refusals (`tree-unclean`, `no-such-fork`, `merge-conflict` — round-4 Charge α);
  `gc` (worktree+branch+target-dir reaped, unmerged refusal, stale-binary warning).
- **Worker-mode guard — invariant test driving `run()`, not a pure helper**
  (`[[mem.pattern.review.invariant-test-must-drive-the-write-seam]]`): (a) linked
  worktree + marker → `memory record` / `slice new` / status-transition refuse
  (the **primary, agnostic** signal); (b) **`DOCTRINE_WORKER` set on the
  coordination root (worker-on-main) → refuse** (the codex/pi env optimisation;
  Charges III/XIII); (c) same worktree without marker and no env (solo) → allowed;
  (d) non-worktree tempdir, no env → allowed.
- **`Orchestrator`-class refusal (Charge IV):** from a marked fork (or with env
  set), `fork` / `import` / `gc --force` are **refused** — drive `run()`, not a
  pure helper. The worker cannot delete branches.
- **`fork` compensating cleanup (Charge VIII):** a forced provision failure triggers
  `worktree remove --force` + branch `-D` + target reap, leaving no orphan; a
  rollback that itself half-fails **exits non-zero naming the leftover**; a
  pre-marker failure leaves no unmarked fork.
- **`fork` git syntax (Charge VI):** black-box golden pins `git worktree add -b …`.
- **Marker lifecycle (Charge V):** a stale marker in a reused dir does **not**
  fail-close a tree promoted to coordination root (assert-marker-absent gate).
- **`gc` landed-oracle (Charge I):** (a) sibling moves HEAD between spawn and import;
  gc still reaps the **landed** fork (patch-id `-`) and a moved HEAD does *not*
  false-refuse it (delta-emptiness would); (b) **crash before commit** → no
  coordination commit → patch-id `+` → gc **refuses** (no `--force`) — the
  crash-survives-and-lies hazard is closed; (c) `--dry-run` prints the per-fork
  verdict.
- **`marker --clear` (Charge II):** a stale marker on a team-mode linked-worktree
  coordination root → orchestrator writes + `gc` refused; `worktree marker --clear`
  (env unset) restores both **from within the CLI**; the same verb is **refused**
  when `DOCTRINE_WORKER` is set (a worker cannot self-unmark) or run from outside the
  marker's tree.
- **Claude marker-via-hook + per-harness altitude (Charge XIII):** the O3 spike
  confirms the WorktreeCreate hook stamps the marker into the Agent-created worktree
  (claude worker → marker present → writes refused) **without** a subprocess or env;
  and that a codex/pi subprocess worker reads the orchestrator-set `DOCTRINE_WORKER`
  (the env optimisation; Charge III propagation gate). The altitude table is
  asserted per-harness, not uniform.
- **D5** (codex/pi path): two parallel worktree builds, no cargo-lock contention,
  each spawns its own correct `CARGO_BIN_EXE`. (Claude shares the jail-wide target —
  the §5.1 rituals are the proof there, not isolation.)
- **D6** (if landed): an out-of-tree write from the worker process is OS-denied.
- **Behaviour-preservation gate — be precise about what is preserved vs what
  changes.** The migration legitimately *changes* worker-mode behaviour
  (env→marker trigger): the existing `DOCTRINE_WORKER=1` guard tests are
  **rewritten** to the marker, not kept green. What stays green *unchanged* — and
  is the preservation proof — is the orthogonal machinery: `select_copies` /
  provision, `branch-point-check`, `is_withheld` / allowlist, the `git.rs`
  born-frame seam. Conflating the two would hide a real behaviour change behind a
  "green" claim.

**Third-inquisition additions.**
- **gc superseded — no stored flag (Charge A):** `gc --fork <b> --superseded-head
  <SHA>` reaps a re-dispatched fork **only** when `<SHA>` == the branch's live head;
  a stale/mismatched SHA **refuses**; **no record** is written or read; a crash that
  loses the orchestrator's SHA list ⇒ gc **refuses** (safe), never reaps; a reused
  branch name **cannot** false-match (no name key).
- **gc two-leg oracle (Charge E, sharpened round-4 Charge α):** the dispatch
  single-commit fork reaps via the `git cherry` patch-id leg (`-`); a solo `/execute`
  multi-commit branch reaps via the **ancestry leg** after `land --no-ff` (D4b — fork
  commits reachable); a partial landing trips neither leg and **refuses**; a
  squash-merge is uncertifiable → **named refusal** (Charge α). Solo multi-commit work
  does **not** traverse the `multi-commit`-refusing `import`.
- **`marker --clear --operator` (Charge B):** a bare `marker --clear` in a **linked
  worktree refuses** (accident-fence); `--operator` clears it; refused under
  `DOCTRINE_WORKER` (codex/pi worker cannot self-unmark). The **claude residual** (no
  env-lock) is asserted as a **documented altitude**, not a passing guard test — the
  G3/ADR-011 table states claude worker-sole-writer is prompt-enforced against a
  deliberate self-clear.
- **claude hook arming (Charge C):** the O3 spike confirms the WorktreeCreate hook
  stamps the marker **only** when the orchestrator armed it (a non-dispatch
  `isolation:worktree` Agent worktree is **not** branded), **and** exercises the
  spike-failure branch (prompt-enforced fallback, no marker mechanism).
- **env named dual-cause (Charge D):** a leaked `DOCTRINE_WORKER` on main **refuses a
  concurrent `slice new`** (authoring verb) with the **named dual-cause** message, not
  a bare worker refusal; worker-on-main still refuses (same ambient state, same
  message).
- **import untracked-clean (Charge F):** import **succeeds** with benign untracked
  files present in the coordination tree; **refuses** `tree-unclean` on a tracked
  uncommitted modification or a staged change.
- **router self-knowledge (Charge G):** an unknown harness **refuses** to dispatch
  (no blind subprocess); one cadence golden proves the verb sequence is identical
  across both sub-skills (two spawn shells, one cadence).

**Fourth-inquisition additions.**
- **`land` non-squash verb (Charge α):** black-box goldens — `land` on a clean
  coordination tree produces a `--no-ff` merge commit, fork commits reachable; then
  `gc` reaps (ancestry leg); refusals `tree-unclean` (tracked mod present),
  `no-such-fork`, **`dispatch-fork`** (marker-bearing fork → refuse, round-5 Charge ζ —
  a dispatch worker's delta must funnel through belted `import`, not `land`),
  `merge-conflict` (report+halt, no auto-resolve).
- **`land` conflict aborts (round-5 Charge η):** a golden where `land` hits a **real
  merge conflict** asserts the verb ran **`git merge --abort`** and the coordination
  tree is **clean** afterward (`git status --porcelain --untracked-files=no` empty) —
  no wedged half-merge; re-running `land` after a source-side fix succeeds. **Squash-refusal:**
  a **manually squash-merged** multi-commit solo fork trips neither gc leg → gc
  **refuses with the named "cannot certify a squash-merge" message**, never a silent
  `--force` (the round-4 Charge α hole, closed).
- **gc two-leg oracle (Charge α):** `land --no-ff` fork reaps via `--is-ancestor`;
  `import` fork reaps via `git cherry` all-`-`; a partial `land` (conflict
  half-resolved out-of-band) is neither-ancestor-nor-all-`-` → refuse.
- **marker ro under bwrap (Charge β):** the D6 spike asserts a confined worker
  **cannot `rm`** its ro-bound marker (write denied by the OS); and that `env -u
  DOCTRINE_WORKER` does not lift the **marker** conjunct (marker present → still
  refused). Absent D6, the test documents the degrade-to-accident-fenced and the
  import-belt as the standing containment.
- **serial-only sentinel (Charge γ; mechanised round-5 Charge θ):** the **widened** O3
  spike asserts (a) serial arm→launch-one-Agent→stamp→disarm brands the right tree;
  (b) **two concurrent `isolation:worktree` spawns against one armed sentinel
  mis-brand** (the evidence that grounds the serial-only constraint and the
  parallel-claude prompt-enforced degrade); (c) **a second `marker --arm` while armed
  refuses `already-armed`** (the single-slot lock that makes the fail-open
  armed-and-concurrent middle *unreachable*, not merely discouraged — round-5 Charge
  θ).
- **router env-marker cross-check (Charge ε; spike-gated Charge ι):** a golden where
  the detected env marker **contradicts** the claimed self-belief → the router
  **refuses** (no spawn); agreement → routes; **marker absent → refuses *naming the
  cause*** (round-5 Charge ι), never a bare refusal. The widened O3 spike confirms a
  stable, launch-mode-robust, harness-unique marker exists per harness; if none, the
  **named fallback** (refuse-all-dispatch with the diagnostic) is exercised. The
  cross-check claim stays `proposed` until that marker-existence gate is green.
- **`--superseded-head` honesty (Charge δ):** a moved branch (recorded SHA ≠ live
  head) → **refuse**; a lost SHA → **refuse** (forces a knowing `--force`); the verb
  is exercised as an operator assertion, **not** asserted to prove landing.

## Open questions (post-lock)

- **OQ-1 (named in D3):** moved-HEAD import (`--allow-reanchor`: 3way onto moved
  HEAD + computable path-disjointness) is a **named backlog follow-up**, not v1
  scope and not fail-open prose (A2 struck — inquisition Charge II). v1 refuses
  `head-moved` → re-dispatch. The re-anchor-vs-re-dispatch *policy* is the judgment
  the follow-up must home.
- **OQ-2:** bwrap userns feasibility — empirical at the D6 spike.
- **OQ-3:** disk pressure under N concurrent `wt/<branch>` targets — gc reaps;
  worktree cap or D-B4 (`sccache`) only if it bites.
- **OQ-4:** ADR-011 records the harness-agnostic **contract** + per-harness
  **capability profile** (Charge XIII), not spawn flags. Per-harness spawn templates
  (`codex exec`, pi self-subagent depth, claude `Agent`+hook) live in the
  `/dispatch-*` *skills*, never the binary. `claude -p` is excluded (API-billed +
  harness-specific).

## Adversarial self-review — findings integrated

| # | Finding | Resolution |
|---|---|---|
| F-gc | `--merged` is the wrong safe-to-delete oracle — the apply-funnel branch is never a git-ancestor | ~~gc uses delta-emptiness~~ → ~~import receipt~~ → **round-2 Charge I: gc gates on a durable git patch-id check (`git cherry`)**, no runtime receipt (D4) |
| F-eval | the example spawn prose `eval "$(fork…)"` swallows exit code — fail-open, ironic | capture + check `$?`, never `eval "$(…)"` (D1) |
| F-preservation | env→marker is a real behaviour change; old guard tests can't stay "green unchanged" | preservation proof scoped to provision/branch-point/select_copies; guard tests rewritten (Verification) |
| F-belt | R-5 match semantics unpinned | prefix-match on `.doctrine/` over name-only tracked diff; test pins it (D3) |
| F-obs | DC-2's observability leaned on an unspecified surface | required CLI status surface added (D2) |
| F-clock | marker provenance invented an ISO-date/clock dep | dropped — presence is the signal; optional base-SHA only (D2) |
| F-adr-id | ADR-011 hardcoded | allocate via `doctrine adr new` (G3) |
| F-slash | branch `/` in target-dir path | nested path, cargo-accepted, unique (D5) |
| F-d6-shell | bwrap wrap can't be expressed in the env-emit contract | D6 extends the *prose* shell, not the verb — consistent with DC-1 (D1) |

Residual (named, not closed): moved-HEAD import (`--allow-reanchor`) is deferred to
a named backlog follow-up (not fail-open prose — Charge II); the
re-dispatch-vs-re-anchor *policy* that follow-up will need still has a prose owner.

## Inquisition findings integrated (`inquisition.md`)

External hostile pass — Opus + GPT-5.5 (codex mcp), converged. Adjudicated via
`/consult` (Cruxes A/B) and a scope decision (Charge II).

| # | Charge | Sev | Resolution |
|---|---|---|---|
| I | gc delta-emptiness oracle unsound (false-negates on moved HEAD; `branch -d` always refuses) | CRIT | ~~import receipt `{base, fork-head}`~~ **⚠ Superseded by round-2 Charge I:** the receipt was itself unsound (certified apply, not commit; crash-survives-and-lies); gc now gates on a durable **patch-id** check (`git cherry`), no runtime receipt (D4). |
| II | import refuses moved-HEAD; A2 unmet; `apply-conflict` dead code | CRIT | **stationary-only v1**; refusal set `{head-moved, multi-commit, doctrine-touch}`; `apply-conflict` purged; **A2 struck**; moved-head → named follow-up (D3) |
| III | marker guard fail-opens worker-on-main (`is_linked_worktree &&` blind) | CRIT | **DC-2 dual signal** — orchestrator-set env (catches worker-on-main) *or* marker (backstop). **⚠ Superseded by round-2 Charge XIII:** env-primary collapses on claude (no env seam); marker is now primary+agnostic, env a codex/pi-only optimisation. |
| IV | fork/import/gc `Read` → untrusted worker deletes refs | CRIT | **DC-3 `Orchestrator` class** — refused under worker identity |
| V | marker has no removal owner; stale marker bricks coordination writer | HIGH | marker lifecycle owned: gc removes; fork rollback; assert-marker-absent before coordination-root (D2) |
| VI | `git worktree add <dir> <branch> <B>` invalid git | HIGH | `git worktree add -b <branch> <dir> <B>` + golden (D1) |
| VII | dir uniqueness unspecified; consumer `.worktrees/` dirties tree | HIGH | unique per-worker dir; outside-repo-or-gitignored guard (D1) |
| VIII | fork not transactional → orphan / unmarked fork | HIGH | transactional fork with rollback (D1) |
| IX | G2 amends accepted ADR-006 before code validates marker | HIGH | O3 guard-spike **before** G2 (sequencing) |
| X | design cites SL-055's research; handover path nonexistent | MED | citations re-pathed to `slice/055/...`; handover corrected |
| XI | marker stores base-SHA never read | LOW | dropped — presence-only (D2) |
| — | pure/imperative wall | **acquitted** | `target_dir_for_branch`/`classify_import`/`marker_path` take inputs; no clock/git/disk/rng crosses the signature |

## Second inquisition findings integrated (`inquisition-2.md`)

Confirmatory re-pass; `nihil obstat` denied. 3 CRITICAL + 5 HIGH + 5 lesser. **All
13 now dispositioned** this re-lock pass — XIII (the keystone, gating the rest)
resolved first via `/consult`, then I/II, then III/V/XI/XII and VI/VII/VIII; IV/IX/X
disposed as a side-effect of eliminating the receipt (Charge I). 3 acquittals stand.
Awaiting a **third confirmatory inquisition** (fresh adversarial agent) for `nihil
obstat` before `/plan`.

| # | Charge | Sev | Resolution |
|---|---|---|---|
| XIII | keystone `claude -p` API-billed + harness-specific → subprocess seam unusable for claude → DC-2 env leg dead → worker-on-main reopens | **CRIT** | **`/consult`-resolved.** Spawn-subprocess demoted to a **codex/pi enhancement layer**; agnostic keystone = orchestrator-owned fork + **disk-marker-primary** identity (DC-1/DC-2). `claude -p` rejected as required; claude uses `Agent` + WorktreeCreate-hook marker (first-class), env an agnostic→codex/pi optimisation. Per-wt env generalised (CARGO_TARGET_DIR a project-local consumer; D5). bwrap codex/pi-only (D6). Per-harness altitude table in slice scope + G3/ADR-011 + G4/SPEC-012. `/dispatch` → harness router (`/dispatch-subprocess`\|`/dispatch-agent`, O8). Channels follow-up = IDE-004. |
| I | import receipt certifies *apply* not *commit*; gc trusts crash-surviving runtime-tier flag | CRIT | **Resolved.** Receipt eliminated; gc's landed-oracle is a **durable git patch-id check** (`git cherry <coord-HEAD> <fork>`) run *after* the commit — crash-before-commit ⇒ patch-id `+` ⇒ gc refuses (no false "landed"). D3 step 5 / D4. |
| II | stray coordination-tree marker has no remover; gc (Orchestrator-classed) locked behind the guard it trips | CRIT | **Resolved.** New **non-`Orchestrator`** `doctrine worktree marker --clear` — refused only by `DOCTRINE_WORKER`-env-set (no worker self-unmark) + cwd-is-this-tree, **never by the marker conjunct**; assert-marker-absent now names the remedy. DC-3 / D2. |
| III | DC-2 env leg propagation unvalidated; spike scoped to guard logic not propagation | HIGH | **Resolved (reshaped by XIII).** Env is no longer the keystone — identity is the marker, so a failed env propagation no longer reopens worker-on-main universally. The O3 spike gains an explicit propagation gate (a real codex/pi subprocess worker reads the orchestrator-set `DOCTRINE_WORKER`) **and** the claude marker-via-hook gate; ADR-011's env-reliability claim stays `proposed` until that gate is green (G2/G3, Verification). |
| IV | import receipt has no removal owner | HIGH | **Disposed by Charge I** — no receipt exists, so no lifecycle to own (D4). |
| V | import refusal set omits `tree-unclean`; `apply-conflict` purge unsound without it | HIGH | **Resolved.** Added a named `tree-unclean` refusal + a real `git status --porcelain`-empty check (separate from `branch-point-check`, which is ref-equality-blind to the tree); refusal set now `{head-moved, tree-unclean, multi-commit, doctrine-touch}`; the `apply-conflict` purge is sound on **both** conjuncts (D3 step 1/4). |
| VI | assert-marker-absent scoped to coordination root; solo `/execute` direct-writer ungated | MED | **Resolved (amplified by XIII).** assert-marker-absent + `marker --clear` now gate **every** linked-worktree→direct-writer transition, solo `/execute` included — the writer is defined by *write-mode*, not the word "coordination" (D2). |
| VII | refused-then-re-dispatched forks need `--force` to gc; reflex returns | MED | ~~Re-dispatch records the abandoned fork-head as `superseded`~~ **⚠ Superseded by round-3 Charge A:** the stored `superseded` record was itself an unsound runtime-tier flag (false-*presence* reaps live work; no removal owner; name-matchable). Replaced by `gc --superseded-head <SHA>`, reaping only when the named SHA == the branch's live head — **no stored record** (D4). |
| VIII | "transactional fork" overclaims; rollback half-fail / dirty `worktree remove` needs `--force` | MED | **Resolved.** Renamed **compensating cleanup**, not a transaction; rollback uses `git worktree remove --force` (provisioned fork is dirty), is best-effort, and on rollback failure **reports the leftover by name + exits non-zero** — no silent/success-coded half-rollback (D1). |
| IX | gc receipt lookup key `{base, fork-head}` underspecified; base unsuppliable | LOW | **Disposed by Charge I** — no receipt key; `git cherry` computes the merge-base internally, so gc needs only `--fork` (D4). |
| X | no receipt observability surface | LOW | **Resolved (via Charge I).** `gc --fork <b>` / `--dry-run` prints the live patch-id verdict per fork ("landed ✓ / not-landed — `--force` to reap"), computed from git (D4). |
| XI | env leg location-unqualified → leaked `DOCTRINE_WORKER` bricks main-side authoring + self-aborts dispatch | HIGH | **Resolved (shrunk by XIII).** Identity is off env, so the blast radius is small; bounded further by (a) setting `DOCTRINE_WORKER` **child-only**, never `export`ed into the orchestrator shell, and (b) the orchestrator **asserting its own env clean** before any funnel verb, failing **loud with a named error** (a leak is a leak by construction — the orchestrator never self-sets it) rather than a silent guard refusal (D2). |
| XII | stationary-only import livelocks vs expected concurrent main-side authoring | HIGH | **Resolved.** Named + enforced **quiescence constraint**: v1 dispatch requires a coordination branch with no external committers; a live main **mandates delta-branch coordination (D8)**; the orchestrator detects a moved coordination HEAD and **reports the external mover by name** rather than livelocking. IMP-043 re-anchor is the eventual fix (D3, G4/SPEC-012). |
| — | receipt-vs-branch-point independence; orchestrator never env-worker-id'd at call time; pure/imperative wall | **acquitted ×3** | sound — no change |

## Third inquisition findings integrated (`inquisition-3.md`)

Third confirmatory re-pass (fresh adversarial agent); `nihil obstat` **denied** — 2
CRIT + 3 HIGH + 2 MED + 1 LOW, every charge attacking the welds the round-2
remediation *opened* (the `superseded` record, `marker --clear`, the claude hook, the
env-assertion's scope, gc's second caller). **All 8 now dispositioned** this re-lock
pass. Round-2 acquittals and the soundly-reshaped resolutions (patch-id oracle,
quiescence constraint, compensating cleanup, env-not-self-set, the pure/imperative
wall) **stand unchanged**. Awaiting a **fourth** confirmatory inquisition before
`/plan`.

| # | Charge | Sev | Resolution |
|---|---|---|---|
| A | `superseded` runtime record re-incurs the burned-receipt anti-pattern — disposable, crash-surviving, name-matchable flag with no removal owner gating `branch -D`; "fail-safe" defends only false-*absence* | **CRIT** | **Resolved.** Stored record **struck**; re-dispatch reaps by **naming the exact commit** — `gc --superseded-head <SHA>` reaps iff `<SHA>` == the branch's **live head**. No durable flag (crash ⇒ safe refusal), SHA-keyed (no name false-match), no removal owner. Disposes the re-incurred IV/IX (D4). |
| B | `marker --clear` self-pressable by a **claude** worker (no env-lock, bwrap codex/pi-only) → restores write- + `Orchestrator`-class verbs → reopens round-1 III+IV on the dominant harness | **CRIT** | **Confessed, not faked (XIII-consistent).** `marker --clear` in a linked worktree now needs an explicit `--operator` ack (accident-fence) + env-lock (codex/pi anti-self-clear). For claude (no env-lock) the residual is **stated as a documented altitude**: worker-sole-writer is accident-fenced + prompt-enforced, malice-floor deferred to IDE-004/userns-bwrap (DC-3, D2, G3 altitude table). |
| C | claude marker-via-hook over-broad (brands every Agent worktree, no `--worker` discriminator), tier-incoherent, and unfallbacked if the spike refutes it | HIGH | **Resolved.** Orchestrator **arms a sentinel**; the hook stamps **only when armed**, then disarms (non-dispatch worktrees unbranded). Marker-stamping declared a distinct act, permitted to *mint* into the copy-excluded `.doctrine/state/` tier. **Named fallback**: spike-refutes ⇒ claude degrades to prompt-enforced (D1, code-impact, Verification). |
| D | Charge XI named-error guards only `Orchestrator` funnel verbs; the leaked-env-bricks-*authoring* hazard it was raised about still misreports as bare "worker refused" | HIGH | **Resolved.** The guard now names the cause for **every** verb tripped by the **env disjunct on a non-linked tree** (authoring + funnel alike): a **dual-cause** message (worker-on-main → re-dispatch / leak → unset), the two being one ambient state. Orchestrator self-assertion kept as defence-in-depth (D2). |
| E | gc oracle + import funnel tuned for the single-commit dispatch fork, but D4 conscripts gc for multi-commit solo `/execute` — unreconciled commit shape | HIGH | **Resolved.** gc's `git cherry` oracle ranges over **all** `B..fork` commits (reap iff every `-`); **solo `/execute` merges normally** (ancestry intact, no import funnel — `multi-commit` is a dispatch-funnel constraint only). One oracle, two landing routes (D3, D4). |
| F | `tree-unclean` via `git status --porcelain`-empty over-refuses on benign untracked files — the repo's *ordinary* state | MED | **Resolved.** Scoped to `--untracked-files=no` (tracked mods + staged only); untracked files cannot affect a tracked-delta `apply`. Golden: import succeeds with untracked present, refuses on a tracked uncommitted mod (D3, Verification). |
| G | per-harness router (O8) names no detection mechanism; misdetection → wrong altitude; cadence may be duplicated across two sub-skills | MED | **Resolved.** Routing input = the dispatching agent's **harness self-knowledge** (runs *as* claude/codex/pi); **unknown refuses, never guesses**. Cadence **is** the shared CLI verb sequence (no duplication; sub-skills are ~2-line spawn shells) (code-impact). |
| H | durable memory `mem_019ebed8` still prescribes the **eliminated receipt** as "the sound oracle," scoped to `src/worktree.rs`/gc | LOW | **Actioned.** Memory superseded/rewritten to record the `git cherry` patch-id oracle as sound and the receipt as the rejected predecessor (keeps its true damnation of `--merged`/delta-emptiness). |

## Fourth inquisition findings integrated (`inquisition-4.md`)

Fourth confirmatory re-pass (fresh adversarial agent); `nihil obstat` **denied** — 1
CRIT + 2 HIGH + 2 MED, every charge attacking the welds the round-3 remediation
*opened* (`--superseded-head`, the `marker --clear` hardening, the claude arming
sentinel, the harness router, and the author-flagged solo-merge residual). **All 5
now dispositioned** this re-lock pass. Round-3 acquittals and soundly-reshaped
resolutions (patch-id oracle crash-proofness, quiescence constraint, compensating
cleanup, env-not-self-set, the pure/imperative wall) **stand unchanged**. Awaiting a
**fifth** confirmatory inquisition before `/plan`.

| # | Charge | Sev | Resolution |
|---|---|---|---|
| α | Solo squash-merge defeats the gc all-commits oracle (squash destroys *both* ancestry and per-commit patch-id); the round-3 Charge-E resolution rested on an unstated, unenforced non-squash assumption that contradicts ADR-006 Consequences | **CRIT** | **Resolved (verb).** New **`doctrine worktree land --fork <b>`** (D4b) — a fail-closed, `Orchestrator`-classed, **structurally non-squash** (`--no-ff`) merge verb, solo's analog of `import`; closes the unmechanised-prose smell at root. gc gains a **two-leg** oracle — `--is-ancestor` (the `land` route) **OR** `git cherry` all-`-` (the `import` route); a squash trips neither and gc **refuses with a named message**, never silent `--force`. Non-squash also preserves memory anchors (ADR-006 D8) — fully ADR-aligned (D4b, D4, G4, Verification). |
| β | The marker is a raw-`rm`-able file inside the worker-rw worktree, and the env is `env -u`-strippable; "full mechanism floor" for codex/pi over-claims past ADR-006 D2b (raw-tree confinement deferred) | HIGH | **Resolved (honest re-claim + ro-bind).** DC-3 re-stated: the `--operator`/env-lock layer fences **accidents, not malice, on every harness**; the **unconditional** containment is the import `.doctrine/`-rejection belt. A genuine codex/pi malice floor is **D6-contingent** — bwrap denies out-of-tree writes **and ro-binds the marker path** so the worker cannot `rm` it; absent D6, codex/pi is accident-fenced like claude. **No harness claims a "full mechanism floor" unconditionally** (DC-3, D6, G3 altitude table, Verification). |
| γ | The arm/stamp/disarm sentinel races under parallel claude dispatch (the `Agent` tool gives no worktree handle to correlate); the round-3 Charge-C fix is sound only serially, and the spike scope was too narrow to know | HIGH | **Resolved (serial-only + widened spike).** Claude marker-stamping requires **one armed spawn in flight at a time** (arm → launch exactly one Agent → await stamp+disarm → nothing else worktree-creating in the window). **Parallel file-disjoint claude dispatch degrades to prompt-enforced** (no marker), symmetric with the D6/spike back-outs. The O3 spike is **widened** to confirm the serial path *and* that the concurrent case mis-brands (D1, code-impact, Verification). |
| δ | `gc --superseded-head <SHA>` is `--force` plus a TOCTOU checksum mislabelled an "oracle"; the head-match proves non-movement, not landing | MED | **Resolved (honest reframe).** The verb is kept (named-SHA audit trail + moved-branch catch beats blind `--force`) but **re-stated as an operator assertion** that this exact, still-current commit is spent-and-abandoned; the head-match is a **TOCTOU movement-guard, not a proof of landing** — it reaps *unlanded* work on the operator's word (D4, code-impact, Verification). |
| ε | The `/dispatch-*` router routes on unmechanised harness self-belief — the "faith not works" smell the slice condemns; "unknown⇒refuse" misses *confident misidentification*; available env-marker detection was rejected without analysis | MED | **Resolved (mechanised cross-check).** The router probes **env markers** (orchestrator runs bash) and routes **only when detection *agrees* with self-belief**; **mismatch or unknown → refuse**. Belief alone routes nothing; a wrong-but-certain belief now fails the cross-check instead of mis-spawning. Precise marker names resolved in-skill/at the spike, not hardcoded (code-impact, Verification). |
| — | patch-id oracle crash-proofness & ancestry-independence; quiescence constraint; compensating cleanup; env-not-self-set; pure/imperative wall | **acquitted (stand)** | round-3 acquittals + soundly-reshaped resolutions re-affirmed — no change |

## Fifth inquisition findings integrated (`inquisition-5.md`)

Fifth confirmatory re-pass (fresh adversarial agent); `nihil obstat` **denied** — 3
HIGH + 1 MED, every charge attacking the welds the round-4 remediation *opened* (the
new `land` verb, the belt-as-unconditional re-claim, the serial-only sentinel, the
router env cross-check). Charge δ's reframe **acquitted clean**. **All 4 now
dispositioned** this re-lock pass. Round-4 acquittals and soundly-reshaped
resolutions (patch-id oracle crash-proofness, gc ancestry-leg monotonicity,
quiescence constraint, compensating cleanup, env-not-self-set, the pure/imperative
wall) **stand unchanged**. Awaiting a **sixth** confirmatory inquisition before
`/plan`.

| # | Charge | Sev | Resolution |
|---|---|---|---|
| ζ | Belt is **not** unconditional containment; `land` (D4b) is a second, **beltless** sanctioned funnel — a misrouted orchestrator lands a worker's `.doctrine/` delta beltless; the round-4 α and β fixes contradict | HIGH | **Resolved (honest re-scope + mechanised condition).** The belt's scope is **the import/dispatch path, conditioned on dispatch deltas routing through `import` not `land`**; `land` is a **solo-trusted, beltless** funnel (solo is a self-orchestrator, D6a — legitimately lands doctrine; a belt there is a category error). The condition is **mechanised**: `land` **refuses a marker-bearing fork** (`dispatch-fork`) — a misroute is named-refused, not silently belt-bypassed; a *malicious* worker reaching `land` is the already-confessed **D2b raw-tree residual**, not a new capability. "No funnel claims unconditional doctrine-containment" carried into DC-3, G3/ADR-011, G4/SPEC-012 (DC-3, D4b, Verification). |
| η | `land`'s `merge-conflict` path is **destructive and unowned** — `git merge` mutates-then-conflicts, leaving a half-merge that `land`'s own `tree-clean` precond rejects; "mirrors import's report-don't-merge" is mechanically false; no `--abort` specified | HIGH | **Resolved (`git merge --abort` on conflict).** D4b step 3 now **aborts the merge** before refusing `merge-conflict`, restoring the clean tree the precond demands — a *true* mirror of import's leave-nothing-behind posture; solo fixes the coupling at source and re-runs `land` onto a clean tree. Golden asserts the coordination tree is **clean** after a real conflict (D4b, code-impact, Verification). |
| θ | "Serial-only" claude stamping is **prose with no enforcement point**; the blessed parallel batch re-enters a **fail-open** race the γ remedy claimed to retire — *faith, not works* | HIGH | **Resolved (single-slot arming mechanism).** Arming is `doctrine worktree marker --arm`, which **refuses `already-armed` if a sentinel is armed or a stamp is awaited** (single-slot runtime lock). A second `--arm` while armed is **physically refused** → the racy armed-and-concurrent middle is **unreachable**; an orchestrator going parallel **cannot arm** → honest prompt-enforced no-marker degrade. Serial-only is now mechanism, not discipline. Widened O3 spike asserts a second `--arm` refuses and the concurrent case produces no second stamp (D1, code-impact, Verification). |
| ι | The ε router cross-check mechanised the **logic** but deferred its **load-bearing input** (marker names + reliability) with **no spike-gate and no fallback**, unlike siblings C/D6; Charge III recursion | MED | **Resolved (spike-gate + named fallback + cause-naming).** The detection *signal* is **spike-gated** (O3 confirms a stable, harness-unique, launch-mode-robust marker per harness); **named fallback** if none exists: refuse **all** dispatch with the diagnostic, never a silent revert to self-belief; the refusal **names the cause** (marker mis-seeded/renamed/launch-mode-stripped). The cross-check claim stays **`proposed`** until the marker-existence gate is green (router prose, Verification, G3). |
| δ | `gc --superseded-head` reframe — stray "oracle"/"proving-landed" language | — | **Acquitted (round 5).** No stray oracle/proving language survives near `--superseded-head`; the reframe ("operator assertion … TOCTOU movement-guard, not a proof of landing") is clean. No change. |
| — | gc ancestry-leg monotonicity under advancing HEAD; patch-id crash-proofness; quiescence; compensating cleanup; env-not-self-set; pure/imperative wall | **acquitted (stand)** | round-4 acquittals + soundly-reshaped resolutions re-affirmed — no change |

## Invariants preserved

Provision remains the sole copier; `check-allowlist` green ≠ complete;
`select_copies` is the guarantee; the funnel cadence order (D7) is unchanged;
exclusion-by-construction holds at every new verb (the marker rides the existing
withheld tier; import never sees `.doctrine/`).
