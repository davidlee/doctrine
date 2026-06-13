# SL-056 Design — Orchestrator spawn seam: worktree mechanism into CLI verbs

Scope: `slice-056.md`. Evidence base (all `research §N` cites below):
`.doctrine/slice/055/research/worktree-orchestration.md` — this slice is a sibling
of SL-055 and shares its research spine; cites resolve there, not under 056
(inquisition Charge X). Thesis: *mechanism in prose is the design smell* —
mechanism belongs in the CLI (identical under claude/codex/pi by construction);
judgment and harness concessions belong in prose. This design moves the
worktree/dispatch creation ladder, the import funnel, build isolation, and the
worker-mode guard out of fail-open prose into fail-closed, golden-testable CLI
verbs, with **orchestrator-owned fork + a disk marker as the harness-agnostic
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
  only by `DOCTRINE_WORKER`-env-set (a real codex/pi worker cannot self-unmark) and
  by cwd-is-not-this-tree, **never by the marker conjunct** — so the env-unset
  orchestrator with a stray coordination-tree marker always escapes. (A claude
  marker-only worker self-clearing is the accepted D2b residual: the guard fences
  accidents/misplacement, not malice — that is D6/bwrap.)

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

*claude (`/dispatch-agent`)* — no fork verb, no env. The orchestrator configures the
WorktreeCreate hook (ADR-006 D9) to provision + stamp the marker, then launches the
worker via the `Agent` tool with `isolation:worktree`. Identity is the hook-stamped
marker (disk); `DOCTRINE_WORKER` and bwrap are unavailable; the worker shares the
jail-wide target (D5); worker-on-main is the deferred D2b residual (DC-2).

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
- **`marker --clear` (Charge II remedy).** `doctrine worktree marker --clear` removes
  the marker at the cwd tree root and prints a loud receipt of what it cleared.
  Refused if `DOCTRINE_WORKER` is set (a spawned codex/pi worker cannot self-unmark)
  or if cwd is not the marker's tree root; **not** refused by the marker conjunct of
  `worker_mode` — that is the lock Charge II condemned. Restores writes + `gc` to an
  orchestrator self-bricked by a stale marker, entirely in-CLI (no filesystem
  surgery).
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
verb it **asserts its own env clean and fails loud with a named error** ("`DOCTRINE_
WORKER` set on the orchestrator — env polluted, unset it; workers do not run this
verb") rather than presenting a leak as a routine guard refusal. The test discipline
extends to the orchestrator path.

## D3 — `doctrine worktree import` (the funnel belt)

**Current.** ~60 lines of dispatch prose replay: precond (tree clean + `HEAD==B`)
→ net diff `B..S` → assert `S^==B` → single-non-merge check → R-5 `.doctrine/`
name-only belt → `git apply --3way --index` non-committing. Fail-open prose; the
R-5 belt is called "the real protection" yet lives as an instruction.

**Target.** One fail-closed, golden-testable verb:

```
doctrine worktree import --base <B> --fork <branch>     # runs at coordination root
```

`Orchestrator`-classed (DC-3) — refused under worker identity. Mechanical
sequence, **v1 is the stationary-head case only** (inquisition Charge II; A2
struck — see below), each step a hard refusal on violation (no auto-merge, no
judgment):
1. precond — **two guards, neither assumed** (Charge V): `HEAD == B`
   (`branch-point-check` — a **ref-equality** compare, blind to the working tree)
   **and** the coordination tree is **clean** (a separate `git status
   --porcelain`-empty check, which `branch-point-check` does *not* perform). `HEAD !=
   B` → refuse `head-moved` (orchestrator re-dispatches from the moved HEAD — no
   in-verb re-anchor in v1; see the quiescence constraint below — XII). Dirty tree →
   refuse `tree-unclean`.
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
disposes Charge IX): the fork commit marked `-` ⇒ its patch is already present in
coordination's history ⇒ safe to reap; marked `+` ⇒ not landed ⇒ refuse unless
`--force` (the explicit "I know it's spent" override). This survives the two failure
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

**Superseded forks — a non-`--force` disposition (Charge VII).** Moved-HEAD
re-dispatch is the *common* case (XII), and a re-dispatched fork genuinely *is* spent
yet never landed → patch-id `+` → bare gc would demand `--force`, training the very
reflex the landed-oracle exists to kill. So **re-dispatch records the abandoned
fork-head as `superseded`** (a small runtime record), and gc reaps on **patch-id
landed OR a `superseded` record**. The superseded record is **fail-safe**, unlike
the eliminated receipt: its *absence* only costs a `--force`, never an erroneous reap
(Charge I's hazard does not recur). `--force` is thereby reserved for the
genuinely-unknown fork, so the reflex never becomes muscle.

Cleanup ownership becomes trivial: **the caller of `fork` owns `gc`.** `/dispatch`
concludes with it; solo `/execute` ends with it.

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
everything else (a write to main's `.doctrine` denied by the OS). Too costly →
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
  bwrap), with `Agent` a **first-class** backend, not a degraded rung. **No
  harness-specific command (`claude -p`) is a required element.** The
  **env-reliability claim stays `proposed`** until the O3 propagation gate is green
  (Charge III) — governance trails proven mechanism. ADR-006-references;
  framework-level (harness-agnostic).
- **G4 — SPEC-012 rewrite.** Reframe Overview + Concerns (drop "the funnel is a
  discipline, not enforced code" — now enforced); rewrite D3 (fail-open env →
  fail-closed **marker-primary** guard) and **state the achievable enforcement
  altitude per harness** (Charge XIII) — no uniform fail-closed claim; **state the
  quiescence constraint** (v1 dispatch requires a non-churning coordination branch; a
  live main mandates delta-branch coordination — Charge XII); add a D for the verb
  family; add FRs (fork, import, gc, marker guard, per-wt env contract).

Untouched: ADR-007, ADR-001/003/004, the withheld-tier model.

## Code impact

| Path | Change |
|---|---|
| `src/worktree.rs` | `run_fork`, `run_import`, `run_gc`, `run_marker_clear` (imperative shells, **compensating-cleanup** fork rollback — `remove --force`, honest non-zero on rollback failure, Charge VIII); pure: `target_dir_for_branch`, `marker_path`, `classify_import`. gc landed-oracle is a **git patch-id check** (`git cherry`), not a runtime receipt (Charge I). Reuse `select_copies`/`branch-point` core. New `write_marker`/`marker_present`/`remove_marker` (`write_marker` also invoked by claude's WorktreeCreate hook — Charge XIII; `remove_marker` behind `marker --clear` — Charge II). Third `is_linked_worktree` consumer. |
| `src/main.rs` | `fork`/`import`/`gc` subcommands + arg structs (watch the bool/arg clippy ceilings, `[[mem.pattern.lint.cli-handler-args-struct]]`). Worker-mode guard = `worker_mode(root)` = `(is_linked_worktree && marker_present) OR env DOCTRINE_WORKER set` — **marker primary, env a codex/pi optimisation** (DC-2 / Charge XIII). `write_class` unchanged. **fork/import/gc are a new `Orchestrator` class — refused under `worker_mode`, NOT `Read`** (they mutate git refs/dirs; inquisition Charge IV / DC-3). A marker-stamping entry point (claude WorktreeCreate hook) + a marker-clear path (Charge II) join the verb family. |
| `src/git.rs` | new reads behind the verbs: worktree list, **patch-id reachability** (`git cherry`, gc landed-oracle — Charge I), `B..S` diff name-only (import). Impure seam only. |
| ADR-008 / ADR-006 / **ADR-011 (new)** / SPEC-012 | G1–G4. |
| `plugins/doctrine/skills/{worktree,dispatch,execute}/SKILL.md` + new `{dispatch-subprocess,dispatch-agent}/SKILL.md` | rewrite prose to *call* the verbs (the token/agnostic payoff); **`/dispatch` becomes a harness router** → `/dispatch-subprocess` (codex/pi) \| `/dispatch-agent` (claude), Charge XIII; re-embed ritual `[[mem.pattern.distribution.skill-refresh-command]]`. |
| `flake.nix` | none for the spike; `dispatch-worker` bwrap profile only if D6 lands. |

## Verification alignment

- **Black-box CLI goldens** (`[[mem.pattern.testing.black-box-cli-golden]]`,
  `force_no_tty`): `fork` (env on stdout, status on stderr, marker written);
  `import` happy path + each refusal (`head-moved`, `multi-commit`,
  `doctrine-touch`, `apply-conflict`); `gc` (worktree+branch+target-dir reaped,
  unmerged refusal, stale-binary warning).
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
| VII | refused-then-re-dispatched forks need `--force` to gc; reflex returns | MED | **Resolved.** Re-dispatch records the abandoned fork-head as `superseded`; gc reaps on patch-id-landed **OR** superseded record. The record is **fail-safe** (absence only costs a `--force`, never an erroneous reap — Charge I's hazard does not recur); `--force` reserved for the genuinely-unknown (D4). |
| VIII | "transactional fork" overclaims; rollback half-fail / dirty `worktree remove` needs `--force` | MED | **Resolved.** Renamed **compensating cleanup**, not a transaction; rollback uses `git worktree remove --force` (provisioned fork is dirty), is best-effort, and on rollback failure **reports the leftover by name + exits non-zero** — no silent/success-coded half-rollback (D1). |
| IX | gc receipt lookup key `{base, fork-head}` underspecified; base unsuppliable | LOW | **Disposed by Charge I** — no receipt key; `git cherry` computes the merge-base internally, so gc needs only `--fork` (D4). |
| X | no receipt observability surface | LOW | **Resolved (via Charge I).** `gc --fork <b>` / `--dry-run` prints the live patch-id verdict per fork ("landed ✓ / not-landed — `--force` to reap"), computed from git (D4). |
| XI | env leg location-unqualified → leaked `DOCTRINE_WORKER` bricks main-side authoring + self-aborts dispatch | HIGH | **Resolved (shrunk by XIII).** Identity is off env, so the blast radius is small; bounded further by (a) setting `DOCTRINE_WORKER` **child-only**, never `export`ed into the orchestrator shell, and (b) the orchestrator **asserting its own env clean** before any funnel verb, failing **loud with a named error** (a leak is a leak by construction — the orchestrator never self-sets it) rather than a silent guard refusal (D2). |
| XII | stationary-only import livelocks vs expected concurrent main-side authoring | HIGH | **Resolved.** Named + enforced **quiescence constraint**: v1 dispatch requires a coordination branch with no external committers; a live main **mandates delta-branch coordination (D8)**; the orchestrator detects a moved coordination HEAD and **reports the external mover by name** rather than livelocking. IMP-043 re-anchor is the eventual fix (D3, G4/SPEC-012). |
| — | receipt-vs-branch-point independence; orchestrator never env-worker-id'd at call time; pure/imperative wall | **acquitted ×3** | sound — no change |

## Invariants preserved

Provision remains the sole copier; `check-allowlist` green ≠ complete;
`select_copies` is the guarantee; the funnel cadence order (D7) is unchanged;
exclusion-by-construction holds at every new verb (the marker rides the existing
withheld tier; import never sees `.doctrine/`).
