# SL-056 Design — Orchestrator spawn seam: worktree mechanism into CLI verbs

Scope: `slice-056.md`. Thesis: *mechanism in prose is the design smell* —
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
- **DC-2 (marker).** A durable, fail-closed disk marker distinguishes a
  dispatch-worker fork (writes refused) from a solo `/execute` fork (writes
  allowed) — both are linked worktrees, so `is_linked_worktree` alone cannot tell
  them apart (D6a: *mode, not location, decides*). Env alone fails open (a new
  shell without the export → guard silent); a disk marker fails closed (present →
  refuse, however entered). Marker lives in the withheld runtime tier at
  `.doctrine/state/dispatch/worker` (self-labelling sibling dir). Rejected: git-dir
  marker (lower observability for no real gain — import-leak is already impossible
  via gitignore + the R-5 belt); env+marker hybrid (env half adds no safety the
  marker lacks — pure surface).

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

Steps (all deterministic, harness-identical):
1. `git worktree add <dir> <branch> <B>` (subsumes ladder rung 3; the native hook
   is demoted to opportunistic, G2(a)). Refuses if `<dir>` exists or `<B>` is not
   a valid commit.
2. `doctrine worktree provision <dir>` (the existing sole-copier; withheld tier
   excluded by construction — unchanged).
3. If `--worker`: write the marker (D2) into the fork. Solo `/execute` omits
   `--worker` → no marker → writes allowed.
4. Emit the worker env contract on **stdout** (machine; one `KEY=value` per line);
   human status to **stderr**. v1 emits exactly:
   `CARGO_TARGET_DIR=<jail-root>/wt/<branch>` (D5).

Orchestrator usage (the thin, irreducibly harness-specific prose shell). Capture
and **check the exit code** before consuming env — `eval "$(cmd)"` swallows the
status of `cmd` (a fail-open trap, ironic here), so we do not use it:
```sh
fork_env="$(doctrine worktree fork --base "$B" --branch "$BR" --dir "$D" --worker)" \
  || { echo "fork failed: $?" >&2; exit 1; }   # halt, do not spawn
env $fork_env claude -p "<pre-distilled prompt>"
#   ^ env lines     ^ the one harness-shaped line (bwrap-wrapped once D6 lands)
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

**Target.** The guard trigger moves from env to the disk marker. `write_class`
itself is unchanged (behaviour-preserving).

```
marker path:  <root>/.doctrine/state/dispatch/worker
guard (in run(), before dispatching a write-classed Command):
    if is_linked_worktree(root) && marker_present(root):
        refuse(verb)        // names the verb, as today
```

- `is_linked_worktree` is the existing predicate (two consumers today: memory
  squash-warn, RV-verb refusal — now three).
- The marker is content-light: optionally the base SHA (already an `--base`
  input — no clock/rng dependency invented). Its *presence*, not contents, is the
  signal. **Observability surface (required, not assumed):** `is_linked_worktree`
  + marker is surfaced by the CLI — minimally a line in `doctrine worktree` /
  status output ("worker fork: yes — writes refused") so the mode is discoverable
  without knowing the gitignored path. Without this surface the marker is as
  hidden as the rejected git-dir option; the surface is what earns DC-2's
  observability claim.
- **D6a preserved.** The orchestrator (trusted, source root, no marker on its own
  tree) writes the marker into the *worker* fork before the worker exists. Solo
  `/execute` forks carry no marker → write freely. Mode, not location, decides.
- Withheld tier: `.doctrine/state/**` is already gitignored, already dropped by
  `provision`, already absent from the import delta — the marker inherits all
  exclusions with zero new logic. The new `dispatch/` sub-path needs no separate
  tier entry (the `State` glob `.doctrine/state/**` already covers it; confirm in
  `is_withheld` test).

`DOCTRINE_WORKER` env is retired as the guard mechanism. (Tests that unset it —
`[[mem.pattern.dispatch.worker-verify-unset-doctrine-worker]]` — change: the green
gate no longer needs `env -u DOCTRINE_WORKER`; a tempdir fixture is not a linked
worktree, so the marker guard never trips there. Net simplification.)

## D3 — `doctrine worktree import` (the funnel belt)

**Current.** ~60 lines of dispatch prose replay: precond (tree clean + `HEAD==B`)
→ net diff `B..S` → assert `S^==B` → single-non-merge check → R-5 `.doctrine/`
name-only belt → `git apply --3way --index` non-committing. Fail-open prose; the
R-5 belt is called "the real protection" yet lives as an instruction.

**Target.** One fail-closed, golden-testable verb:

```
doctrine worktree import --base <B> --fork <branch>     # runs at coordination root
```

Mechanical sequence, each step a hard refusal on violation (no auto-merge, no
judgment):
1. precond: coordination tree clean, `HEAD == B` (`branch-point-check` reused).
2. `S^ == B` assert (single-non-merge fork delta).
3. R-5 belt: reject if the `B..S` **name-only** diff touches any `.doctrine/`
   path. Match semantics pinned: prefix-match on `.doctrine/` over the name-only
   diff (tracked files only — gitignored runtime/derived never appears in a diff,
   so "all `.doctrine/`" and "authored-only `.doctrine/`" coincide in practice;
   the test pins this). A forced-added marker would therefore also be caught —
   defense in depth.
4. `git apply --3way --index` (non-committing). The orchestrator commits
   separately (ADR-006 D7 cadence preserved — import ≠ commit).

**Refusal vs adjudication (OQ-1).** The verb stays strict and *reports*; it does
not decide re-anchor-vs-re-dispatch. On `HEAD != B` it exits non-zero with a
machine-readable reason (`head-moved`, `multi-commit`, `doctrine-touch`,
`apply-conflict`); the orchestrator skill adjudicates re-anchor (provable
disjointness, `[[mem.pattern.dispatch.reanchor-base-on-disjoint-head-move]]`) vs
re-dispatch. Disjointness *computation* could later move into the verb
(`--allow-reanchor`), deferred — the v1 verb is the strict belt; adjudication is
judgment → stays prose. (Honest scope: this keeps one judgment step in prose by
design, consistent with the thesis split.)

Pure core: classification over a diff (`classify_import(diff, base, head) ->
Result<Apply, Refusal>`); imperative shell drives git + apply.

## D4 — `doctrine worktree gc`

**Current.** "GC the dispatch debris" — one prose sentence, no owner (IMP-041).
Stale `env!(CARGO_MANIFEST_DIR)` binaries strand after removal
(`[[mem.pattern.dispatch.worktree-removal-stale-manifest-dir-false-red]]`).

**Target.** `doctrine worktree gc --fork <branch> [--force]` reaps, in one act:
1. `git worktree remove` the spent fork dir.
2. delete the fork branch.
3. **reap the `wt/<branch>` target dir** (closes the D5 disk loop — IMP-041 and
   D-B1 hygiene are the same verb).
4. warn (stderr) that `env!(CARGO_MANIFEST_DIR)`-baked test binaries need
   recompile before the next close-time `just check`.

**The "landed" oracle (finding — `--merged` is wrong here).** The funnel imports
via `git apply --3way` (non-committing, D3), so the fork branch is **never a
git-ancestor** of the coordination commit — `git branch --merged` reports it
*unmerged* even after a clean import. So gc cannot use `--merged` as the
safe-to-delete predicate for the funnel path. v1 resolution: the safety check is
**delta-emptiness** — `git diff <B-or-HEAD>..<fork>` against the coordination tree
is empty (the fork's content has landed) ⇒ safe; non-empty ⇒ refuse unless
`--force`. This correctly covers both a true `--merged` branch (empty diff) and an
apply-imported branch (empty diff). `--force` is the explicit override for the
"I know it's spent" case. (A future `import` could stamp an "imported" record to
make this exact rather than diff-inferred — deferred, noted.)

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
them. Sequence: **G1+G3 → G2 → G4 → code**.

- **G1 — ADR-008 revise→accept** (the gate). Fold §5.1 evidence; record D-B2 as
  standing fact (ro `~/.cargo/bin` ⇒ no in-jail install, no race); re-scope D-B3
  around the userns question. Acceptance gates IMP-004.
- **G2 — ADR-006 amend.** (a) D5/D9 ladder: demote native hook to opportunistic,
  cite SL-050/051. (b) D2a mechanism: replace `DOCTRINE_WORKER=1` self-arm with
  the fork-resident marker (D2). Withheld-tier D1/D4/D9 invariants preserved.
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
| `src/worktree.rs` | `run_fork`, `run_import`, `run_gc` (imperative shells); pure: `target_dir_for_branch`, `marker_path`, `classify_import`, gc selection. Reuse `select_copies`/`branch-point` core. New `write_marker`/`marker_present`. Third `is_linked_worktree` consumer. |
| `src/main.rs` | `fork`/`import`/`gc` subcommands + arg structs (watch the bool/arg clippy ceilings, `[[mem.pattern.lint.cli-handler-args-struct]]`). Move the worker-mode guard trigger from env to `is_linked_worktree && marker_present`. `write_class` unchanged. fork/import/gc are read-side w.r.t. the *authored* corpus → not blocked by the guard. |
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
- **Marker guard — invariant test driving `run()`, not a pure helper**
  (`[[mem.pattern.review.invariant-test-must-drive-the-write-seam]]`): on a linked
  worktree with marker, `memory record` / `slice new` / status-transition refuse;
  same worktree without marker (solo) — allowed; non-worktree tempdir — allowed.
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

- **OQ-1 (named in D3):** does disjointness-driven re-anchor ever move into the
  import verb (`--allow-reanchor`)? v1: no — adjudication stays prose. Revisit if
  the prose proves error-prone.
- **OQ-2:** bwrap userns feasibility — empirical at the D6 spike.
- **OQ-3:** disk pressure under N concurrent `wt/<branch>` targets — gc reaps;
  worktree cap or D-B4 (`sccache`) only if it bites.
- **OQ-4:** ADR-011 spawn-backend enumeration per harness (claude `-p` flags,
  `codex exec`, pi self-subagent depth) — the thin prose call is harness-templated
  in the *skill*, never the binary. ADR-011 records the contract, not the flags.

## Adversarial self-review — findings integrated

| # | Finding | Resolution |
|---|---|---|
| F-gc | `--merged` is the wrong safe-to-delete oracle — the apply-funnel branch is never a git-ancestor | gc uses **delta-emptiness**, not `--merged`; `--force` override (D4) |
| F-eval | the example spawn prose `eval "$(fork…)"` swallows exit code — fail-open, ironic | capture + check `$?`, never `eval "$(…)"` (D1) |
| F-preservation | env→marker is a real behaviour change; old guard tests can't stay "green unchanged" | preservation proof scoped to provision/branch-point/select_copies; guard tests rewritten (Verification) |
| F-belt | R-5 match semantics unpinned | prefix-match on `.doctrine/` over name-only tracked diff; test pins it (D3) |
| F-obs | DC-2's observability leaned on an unspecified surface | required CLI status surface added (D2) |
| F-clock | marker provenance invented an ISO-date/clock dep | dropped — presence is the signal; optional base-SHA only (D2) |
| F-adr-id | ADR-011 hardcoded | allocate via `doctrine adr new` (G3) |
| F-slash | branch `/` in target-dir path | nested path, cargo-accepted, unique (D5) |
| F-d6-shell | bwrap wrap can't be expressed in the env-emit contract | D6 extends the *prose* shell, not the verb — consistent with DC-1 (D1) |

Residual (named, not closed): OQ-1 keeps the re-anchor *adjudication* in prose by
design (judgment, not mechanism); a worker invoking `import`/`gc` from its own fork
is nonsensical but not write-classed — low risk, noted not guarded.

## Invariants preserved

Provision remains the sole copier; `check-allowlist` green ≠ complete;
`select_copies` is the guarantee; the funnel cadence order (D7) is unchanged;
exclusion-by-construction holds at every new verb (the marker rides the existing
withheld tier; import never sees `.doctrine/`).
