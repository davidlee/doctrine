# SL-056 Design — Orchestrator spawn seam: worktree mechanism into CLI verbs

Scope: `slice-056.md`. Evidence base (all `research §N` cites below):
`.doctrine/slice/055/research/worktree-orchestration.md` — this slice is a sibling
of SL-055 and shares its research spine; cites resolve there, not under 056
(inquisition Charge X). Thesis: *mechanism in prose is the design smell* —
mechanism belongs in the CLI (identical under claude/codex/pi by construction);
judgment and harness concessions belong in prose. This design moves the
worktree/dispatch creation ladder, the import funnel, build isolation, and the
worker-mode guard out of fail-open prose into fail-closed, golden-testable CLI
verbs, with the orchestrator-owned spawn seam as the keystone.

The unifying principle: **the pure/imperative wall, lifted to the orchestration
layer.** The binary is the pure mechanism core; the harness subprocess invocation
is the thin impure shell. Every decision below is an application of that wall.

## Locked decisions

Two cruxes were adjudicated before drafting:

- **DC-1 (seam boundary).** `doctrine worktree fork` owns create + provision +
  marker + env-emission; the orchestrator's only harness-specific act is invoking
  `claude -p` / `codex exec` / pi with the emitted env. The line falls exactly
  between env-emission (mechanism, harness-identical → binary) and invocation
  (concession, harness-shaped → prose). Rejected: marker-only (leaves the creation
  ladder + env in prose — the very smell); full spawn verb (pulls the harness
  invocation into the binary — re-couples + the config-knob Rube Goldberg ADR-006
  D1 rejects).
- **DC-2 (worker identity — env *and* marker, post-inquisition).** Worker-mode is
  a property of the **process**, not the disk location. Two complementary signals,
  both set by the trusted orchestrator at spawn:
  - **`DOCTRINE_WORKER=1` env** — the *process-identity* signal. The orchestrator
    now owns the subprocess spawn (`claude -p`/`codex exec`), so it sets this env
    **reliably**. This obsoletes the original DC-2 premise: env "failed open" only
    under Claude's `Agent` tool, which exposed no env seam (self-arm by prompt,
    nothing enforcing). **The spawn seam this very slice introduces removes that
    constraint** — orchestrator-set env catches the **worker-on-main** hazard
    (ADR-006 D2b: harness drops the worker on the coordination root) that a
    fork-resident marker is blind to.
  - **fork marker** — the *disk backstop* at `.doctrine/state/dispatch/worker`
    (withheld runtime tier, self-labelling sibling dir). Fails closed if env is
    lost (a new shell without the export → marker still present in the fork).
  - **Guard:** refuse a write-classed verb when
    `env DOCTRINE_WORKER set OR (is_linked_worktree && marker_present)`. Env covers
    worker-on-main; marker covers dropped-env-in-fork; together neither failure
    mode is open. Solo `/execute` sets neither → writes freely (D6a: mode, not
    location, decides). Marker lifecycle is owned (DC-3 below; D2).
  - Rejected: marker-only (the original draft — fail-open on worker-on-main, since
    a worker the harness leaves on `main` is not a linked worktree and carries no
    marker; inquisition Charge III); git-dir marker (lower observability, no real
    gain).
- **DC-3 (verb privilege — fork/import/gc are orchestrator-only).** `fork`,
  `import`, and `gc` **mutate git refs and directories** (create/remove worktrees,
  delete branches, reap target dirs, `--force`). Classifying them `Read` because
  they spare the *authored TOML corpus* is a category error (inquisition Charge IV):
  it lets the untrusted worker delete branches, violating ADR-006 D2
  (workers mutate **source only**). They are a new **`Orchestrator`** class,
  refused under worker identity (`env OR marker`) exactly as write-classed verbs
  are. Only the non-mutating helpers (`provision`, `check-allowlist`,
  `branch-point-check`) stay `Read` and open to workers.

## D1 — `doctrine worktree fork` (the spawn seam)

**Current.** The `/worktree` skill prose drives a creation ladder (existing
isolation → Claude `WorktreeCreate` hook → `git worktree add` → work-in-place).
The dispatch worker *self-forks* rung-3 from prompt instructions — drift from
ADR-006 D9, which already mandates the orchestrator provision + baseline-verify
"before handing the worker its task." `DOCTRINE_WORKER=1` self-arm and
`CARGO_TARGET_DIR` have no spawn seam under Claude's `Agent` tool (no env seam).

**Target.** One verb, run by the trusted orchestrator at the source root:

```
doctrine worktree fork --base <B> --branch <name> --dir <path> [--worker]
```

Steps (all deterministic, harness-identical). **Transactional** — any failure
after step 1 rolls back (remove worktree, delete branch, reap target dir) so a
partial fork never leaks an orphan dir or an **unmarked** (silently write-allowed)
worktree (inquisition Charge VIII):
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
4. Emit the env contract on **stdout** (machine; one `KEY=value` per line); human
   status to **stderr**. v1 emits exactly:
   `CARGO_TARGET_DIR=<jail-root>/wt/<branch>` (D5). The orchestrator additionally
   exports `DOCTRINE_WORKER=1` into the spawned subprocess for a `--worker` fork
   (DC-2 process-identity signal) — it is the spawn-time env, not a fork-emitted
   line, because identity belongs to the process the orchestrator launches.

Orchestrator usage (the thin, irreducibly harness-specific prose shell). Capture
and **check the exit code** before consuming env — `eval "$(cmd)"` swallows the
status of `cmd` (a fail-open trap, ironic here), so we do not use it:
```sh
fork_env="$(doctrine worktree fork --base "$B" --branch "$BR" --dir "$D" --worker)" \
  || { echo "fork failed: $?" >&2; exit 1; }   # halt, do not spawn
env DOCTRINE_WORKER=1 $fork_env claude -p "<pre-distilled prompt>"
#   ^ process identity  ^ env lines  ^ the harness-shaped line (bwrap-wrapped once D6 lands)
```
`$fork_env` is the stdout env block (`CARGO_TARGET_DIR=…`); status went to stderr.
When D6 lands the wrap (`bwrap <profile> env $fork_env claude -p …`) extends *this
prose shell*, not the verb — confinement is a harness/OS concession, DC-1 keeps it
in prose.

**Why a verb, not the skill.** Steps 1–4 are pure mechanism; only the final `env …
claude -p` differs across harnesses. The verb makes the spawn obey the same
no-impurity-in-the-core wall the rest of the codebase obeys.

**Pure/imperative split** (ADR-001 leaf). Pure: `target_dir_for_branch(jail_root,
branch) -> PathBuf`, marker path derivation. Imperative shell (`run_fork`): git
add, provision call, marker write, stdout/stderr emission.

## D2 — Worker-marker (fail-closed worker-mode guard)

**Current.** `main` checks `DOCTRINE_WORKER=1` env; `write_class` (exhaustive over
`Command`) bails on every authored/doctrine-mediated write. Fails open: only the
prompt can self-arm, nothing enforces it (ADR-006 D2b; research §3 C-I).

**Target.** The guard trigger is the **DC-2 dual signal** — env *or* marker.
`write_class` itself is unchanged (behaviour-preserving); a new `Orchestrator`
class (DC-3) joins it under the same guard.

```
marker path:  <root>/.doctrine/state/dispatch/worker
worker_mode(root)  :=  env DOCTRINE_WORKER set
                       OR (is_linked_worktree(root) && marker_present(root))
guard (in run(), before dispatching a write-classed OR Orchestrator Command):
    if worker_mode(root):
        refuse(verb)        // names the verb, as today
```

- **Env is the worker-on-main fix.** A worker the harness leaves on the
  coordination root (D2b hazard) carries no marker and is not a linked worktree —
  the marker conjunct is blind to it (inquisition Charge III). The
  orchestrator-set `DOCTRINE_WORKER` env catches it: identity rides the process,
  not the disk. The marker remains the fail-closed backstop for a fork shell that
  loses the env.
- `is_linked_worktree` is the existing predicate (two consumers today: memory
  squash-warn, RV-verb refusal — now three).
- The marker is **presence-only** — no contents. (The earlier "optionally the
  base SHA" is dropped: it was written and never read — dead/misleading state,
  inquisition Charge XI.)
- **Lifecycle (owned, not assumed — inquisition Charge V).** Written by
  `fork --worker` (transactionally, D1); **removed by `gc`** (D4); rolled back if
  `fork` fails. A tree may serve as a coordination root only after an
  **assert-marker-absent** check — so a reused/stale fork dir cannot fail-close a
  legitimate writer. Marker-absence on the coordination tree is now *guarded*, not
  presumed.
- **Observability surface (required, not assumed):** `worker_mode` is surfaced by
  the CLI — minimally a line in `doctrine worktree` / status output ("worker fork:
  yes — writes refused; signal: env|marker") so the mode is discoverable without
  knowing the gitignored path.
- **D6a preserved.** The orchestrator (trusted, source root, marker absent —
  asserted) writes the marker into the *worker* fork before the worker exists and
  sets the worker's env at spawn. Solo `/execute` forks carry neither → write
  freely. Mode, not location, decides.
- Withheld tier: `.doctrine/state/**` is already gitignored, already dropped by
  `provision`, already absent from the import delta — the marker inherits all
  exclusions with zero new logic. The new `dispatch/` sub-path needs no separate
  tier entry (the `State` glob `.doctrine/state/**` already covers it; confirm in
  `is_withheld` test).

`DOCTRINE_WORKER` env is **retained as the process-identity signal** (DC-2) — *not*
retired (the original draft retired it; the inquisition restored it as the only
signal that sees worker-on-main). It is now **orchestrator-set at spawn**, never
prompt-self-armed, which is what makes it reliable (the spawn seam). Tests that
unset it (`[[mem.pattern.dispatch.worker-verify-unset-doctrine-worker]]`) still run
the green gate with `env -u DOCTRINE_WORKER` *and* outside a marked linked worktree,
so neither guard signal trips in a tempdir fixture.

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
1. precond: coordination tree clean, `HEAD == B` (`branch-point-check` reused).
   `HEAD != B` → refuse `head-moved`; the orchestrator **re-dispatches** from the
   moved HEAD (no in-verb re-anchor in v1).
2. `S^ == B` assert (single-non-merge fork delta) — else `multi-commit`.
3. R-5 belt: reject if the `B..S` **name-only** diff touches any `.doctrine/`
   path — else `doctrine-touch`. Match semantics pinned: prefix-match on
   `.doctrine/` over the name-only diff (tracked files only — gitignored
   runtime/derived never appears in a diff, so "all `.doctrine/`" and
   "authored-only `.doctrine/`" coincide in practice; the test pins this). A
   forced-added marker would therefore also be caught — defense in depth.
4. `git apply --3way --index` (non-committing). Under the `HEAD == B` precond the
   patch `B..S` applies onto the exact tree it was cut from, so it **cannot
   conflict** — `apply-conflict` is therefore **not** a v1 refusal reason
   (purging it; it was unreachable under the preconditions — inquisition Charge
   II). The orchestrator commits separately (ADR-006 D7 cadence preserved — import
   ≠ commit).
5. **Stamp an import receipt** keyed `{base, fork-head}` into the withheld runtime
   tier on success — the *only* sound landed-oracle for `gc` (D4; inquisition
   Charge I). Tree-diff inference cannot tell landed from doomed once HEAD moves.

**Refusal set (v1, exhaustive over permitted states):** `{head-moved,
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

Pure core: classification over a diff (`classify_import(diff, base, head) ->
Result<Apply, Refusal>`); imperative shell drives git + apply + receipt write.

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
   exactly why the receipt gate below is the real safety, not `-d`'s
   merged-check).
3. **reap the `wt/<branch>` target dir** (closes the D5 disk loop — IMP-041 and
   D-B1 hygiene are the same verb).
4. warn (stderr) that `env!(CARGO_MANIFEST_DIR)`-baked test binaries need
   recompile before the next close-time `just check`.

**The "landed" oracle — import receipt, not tree diff (inquisition Charge I).**
`--merged` is wrong (the apply-funnel branch is never a git-ancestor). The
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

**v1 resolution:** gc deletes **only** on a positive **import receipt** (stamped
by `import`, D3 step 5, keyed `{base, fork-head}`). Receipt present ⇒ the fork
provably landed ⇒ safe to reap. No receipt ⇒ refuse unless `--force` (the explicit
"I know it's spent" override). Receipt-gating is exact, not diff-inferred — it was
the design's own deferred note (`a future import could stamp an imported record`);
the inquisition made it the v1 gate, because delta inference cannot work once
`apply --3way` severs ancestry **and** HEAD legitimately moves.

Cleanup ownership becomes trivial: **the caller of `fork` owns `gc`.** `/dispatch`
concludes with it; solo `/execute` ends with it.

## D5 — Per-worktree build isolation (≡ ADR-008 D-B1)

`CARGO_TARGET_DIR = <jail-root>/wt/<branch>`, computed by `fork` (pure
`target_dir_for_branch`; branch names carry `/` — `slice/SL-056-x` → nested
`wt/slice/SL-056-x`, which cargo accepts; collision-safe since branch names are
unique), emitted on stdout, set by the orchestrator at subprocess
spawn (D1). Not baked in the flake (ADR-008 D-B5); cargo env-precedence means a
fork-resident `.cargo/config.toml` cannot override the ambient jail-wide var — only
the spawn-set env can. **No flake change for the spike.** Warm across launches
(in-jail `~/.cargo` persists) → cold cost is per-branch, not per-session; disk is
the residual, reaped by D4. Obsoletes the three §5.1 mitigation rituals.

## D6 — Per-worker bwrap confinement (ADR-008 D-B3, spike)

Timeboxed spike, OS-enforced discharge of ADR-006 D2b. Feasibility gate is
unprivileged userns *inside* the jail (outer bwrap may seccomp-block
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
- **G2 — ADR-006 amend.** (a) D5/D9 ladder: demote native hook to opportunistic,
  cite SL-050/051. (b) D2a mechanism: replace the `DOCTRINE_WORKER=1` *self-arm*
  with the **DC-2 dual signal** — orchestrator-set `DOCTRINE_WORKER` env (now
  reliable via the spawn seam) *plus* the fork-resident marker — and the DC-3
  `Orchestrator` verb class. (Not "env→marker"; env is retained, its arming moved
  from prompt to orchestrator — inquisition Charge III.) Withheld-tier D1/D4/D9
  invariants preserved. **Spike-first (inquisition Charge IX):** the guard +
  privilege model (DC-2/DC-3) is validated by a small O3 code spike *before* G2
  amends the accepted ADR — symmetry with the D6 bwrap spike-first treatment.
  Governance follows proven mechanism, not the reverse.
- **G3 — ADR (new): the spawn seam.** ADR id allocated via `doctrine adr new` at
  authoring (likely ADR-011 — next free — but not hardcoded). Orchestrator-owned fork + subprocess
  spawn + what it buys (env-arm, per-wt target, bwrap wrap); `Agent` is the
  degraded rung where no subprocess exists. ADR-006-references; framework-level
  (harness-agnostic).
- **G4 — SPEC-012 rewrite.** Reframe Overview + Concerns (drop "the funnel is a
  discipline, not enforced code" — now enforced); rewrite D3 (fail-open env →
  fail-closed marker); add a D for the verb family; add FRs (fork, import, gc,
  marker guard).

Untouched: ADR-007, ADR-001/003/004, the withheld-tier model.

## Code impact

| Path | Change |
|---|---|
| `src/worktree.rs` | `run_fork`, `run_import`, `run_gc` (imperative shells, **transactional fork** rollback); pure: `target_dir_for_branch`, `marker_path`, `classify_import`, gc receipt-check. Reuse `select_copies`/`branch-point` core. New `write_marker`/`marker_present`/`remove_marker`, import-receipt read/write. Third `is_linked_worktree` consumer. |
| `src/main.rs` | `fork`/`import`/`gc` subcommands + arg structs (watch the bool/arg clippy ceilings, `[[mem.pattern.lint.cli-handler-args-struct]]`). Worker-mode guard = `worker_mode(root)` = `env DOCTRINE_WORKER set OR (is_linked_worktree && marker_present)` (DC-2). `write_class` unchanged. **fork/import/gc are a new `Orchestrator` class — refused under `worker_mode`, NOT `Read`** (they mutate git refs/dirs; inquisition Charge IV / DC-3). |
| `src/git.rs` | new reads behind the verbs: worktree list, merged-branch test (gc), `B..S` diff name-only (import). Impure seam only. |
| ADR-008 / ADR-006 / **ADR-011 (new)** / SPEC-012 | G1–G4. |
| `plugins/doctrine/skills/{worktree,dispatch,execute}/SKILL.md` | rewrite prose to *call* the verbs (the token/agnostic payoff); re-embed ritual `[[mem.pattern.distribution.skill-refresh-command]]`. |
| `flake.nix` | none for the spike; `dispatch-worker` bwrap profile only if D6 lands. |

## Verification alignment

- **Black-box CLI goldens** (`[[mem.pattern.testing.black-box-cli-golden]]`,
  `force_no_tty`): `fork` (env on stdout, status on stderr, marker written);
  `import` happy path + each refusal (`head-moved`, `multi-commit`,
  `doctrine-touch`, `apply-conflict`); `gc` (worktree+branch+target-dir reaped,
  unmerged refusal, stale-binary warning).
- **Worker-mode guard — invariant test driving `run()`, not a pure helper**
  (`[[mem.pattern.review.invariant-test-must-drive-the-write-seam]]`): (a) linked
  worktree + marker → `memory record` / `slice new` / status-transition refuse;
  (b) **`DOCTRINE_WORKER` set on the coordination root (worker-on-main) → refuse**
  (the env signal; Charge III); (c) same worktree without marker and no env (solo)
  → allowed; (d) non-worktree tempdir, no env → allowed.
- **`Orchestrator`-class refusal (Charge IV):** from a marked fork (or with env
  set), `fork` / `import` / `gc --force` are **refused** — drive `run()`, not a
  pure helper. The worker cannot delete branches.
- **`fork` transactionality (Charge VIII):** a forced provision failure leaves no
  orphan worktree/branch/target-dir; a pre-marker failure leaves no unmarked fork.
- **`fork` git syntax (Charge VI):** black-box golden pins `git worktree add -b …`.
- **Marker lifecycle (Charge V):** a stale marker in a reused dir does **not**
  fail-close a tree promoted to coordination root (assert-marker-absent gate).
- **`gc` receipt oracle (Charge I):** sibling moves HEAD between spawn and import;
  gc still reaps the **receipted** fork and **refuses** an unreceipted one (no
  `--force`); delta-emptiness is *not* the gate.
- **D5**: two parallel worktree builds, no cargo-lock contention, each spawns its
  own correct `CARGO_BIN_EXE`.
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
- **OQ-4:** ADR-011 spawn-backend enumeration per harness (claude `-p` flags,
  `codex exec`, pi self-subagent depth) — the thin prose call is harness-templated
  in the *skill*, never the binary. ADR-011 records the contract, not the flags.

## Adversarial self-review — findings integrated

| # | Finding | Resolution |
|---|---|---|
| F-gc | `--merged` is the wrong safe-to-delete oracle — the apply-funnel branch is never a git-ancestor | ~~gc uses delta-emptiness~~ **SUPERSEDED by inquisition Charge I** — delta-emptiness is *also* unsound; gc gates on an **import receipt** (D4) |
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
| I | gc delta-emptiness oracle unsound (false-negates on moved HEAD; `branch -d` always refuses) | CRIT | **import receipt** `{base, fork-head}`; gc deletes only on receipt; `-D` after receipt (D3.5, D4) |
| II | import refuses moved-HEAD; A2 unmet; `apply-conflict` dead code | CRIT | **stationary-only v1**; refusal set `{head-moved, multi-commit, doctrine-touch}`; `apply-conflict` purged; **A2 struck**; moved-head → named follow-up (D3) |
| III | marker guard fail-opens worker-on-main (`is_linked_worktree &&` blind) | CRIT | **DC-2 dual signal** — orchestrator-set env (catches worker-on-main) *or* marker (backstop); env reinstated, reliable via spawn seam |
| IV | fork/import/gc `Read` → untrusted worker deletes refs | CRIT | **DC-3 `Orchestrator` class** — refused under worker identity |
| V | marker has no removal owner; stale marker bricks coordination writer | HIGH | marker lifecycle owned: gc removes; fork rollback; assert-marker-absent before coordination-root (D2) |
| VI | `git worktree add <dir> <branch> <B>` invalid git | HIGH | `git worktree add -b <branch> <dir> <B>` + golden (D1) |
| VII | dir uniqueness unspecified; consumer `.worktrees/` dirties tree | HIGH | unique per-worker dir; outside-repo-or-gitignored guard (D1) |
| VIII | fork not transactional → orphan / unmarked fork | HIGH | transactional fork with rollback (D1) |
| IX | G2 amends accepted ADR-006 before code validates marker | HIGH | O3 guard-spike **before** G2 (sequencing) |
| X | design cites SL-055's research; handover path nonexistent | MED | citations re-pathed to `slice/055/...`; handover corrected |
| XI | marker stores base-SHA never read | LOW | dropped — presence-only (D2) |
| — | pure/imperative wall | **acquitted** | `target_dir_for_branch`/`classify_import`/`marker_path` take inputs; no clock/git/disk/rng crosses the signature |

## Invariants preserved

Provision remains the sole copier; `check-allowlist` green ≠ complete;
`select_copies` is the guarantee; the funnel cadence order (D7) is unchanged;
exclusion-by-construction holds at every new verb (the marker rides the existing
withheld tier; import never sees `.doctrine/`).
