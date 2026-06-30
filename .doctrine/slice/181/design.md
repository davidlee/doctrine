# SL-181 Design — Worker-safety: accidental ref-corruption guard + OQ-D reframe

Status: draft (self-reviewed). Governed by ADR-006 (D2a/D2b), ADR-012 (OQ-D).
Originates from IMP-065. Residual tracked in RSK-014.

## 0. Thesis (read this first)

The dispatch worker-mode marker is a **cooperative flag, not an enforced
boundary**, on every spawn arm except bwrap-confined pi-subprocess (RSK-014).
ADR-006 D2a already concedes this ("the fence … is the funnel + the jail, **not**
a fail-closed CLI floor"). Therefore the IMP-065 "positive coordination marker"
**cannot close** the OQ-D / ADR-011 D6/M2 impersonation gap — a capable worker
`cd`s into the coord tree or forges any presence-only marker trivially.

What is real and uncaught today: an **accidental, cooperative, unstamped worker
that stays in its own tree and runs an Orchestrator-classed verb there**. The R-5
import belt catches `.doctrine/` *file* writes at import but **not ref mutation**
(`gc`/`sync`/`import`/`candidate` rewrite shared refs, absent from the `B..S` file
delta). This slice closes exactly that honest-failure path, and re-frames OQ-D
honestly via a Revision. It is **defence-in-depth against accident, not an
impersonation boundary** — and is honest about it everywhere.

## 1. Current vs target behaviour

**Current.** Every Write/Orchestrator CLI verb passes through `worker_guard`
(`src/commands/guard.rs`). The verdict is `resolve_mode(root)` →
`describe_mode(is_linked, worker_marker_present, env_set)`:

```
refused = (is_linked_worktree && worker_marker_present) || DOCTRINE_WORKER
```

A **markerless linked worktree** (no worker marker, no env) is NOT refused. Two
trees share that profile and are **indistinguishable**:
- the SL-064 coordination worktree (markerless by design, `run_coordinate`), and
- an **unstamped worker** whose stamp hook failed (ISS-011; still live post-SL-124/125
  because `SubagentStart` is un-failclosable — a crash/OOM mid-stamp leaves no marker).

So an unstamped worker may invoke the **Orchestrator verb class**
(`fork`/`import`/`land`/`gc`/`coordinate`/`dispatch sync`/`record-boundary`/
`refresh-base`/`arm-spawn`/`candidate create|admit`) and mutate shared refs —
**fail-open**.

**Target.** For the **Orchestrator class only**, in a linked worktree that is not
the coordination tree, refuse. The coordination tree is identified by a
**positive, non-forgeable-by-accident signal**: its `HEAD` rides the
`dispatch/<NNN>` branch (guaranteed by `run_coordinate`, both create and resume
paths; preserved through merge-conflict ride-through). General `Write` class is
**unchanged** (D6a: solo `/execute` in a worktree still writes — not re-breaking
the reverted-G2 hazard).

## 2. Mechanism

Three small pieces. No new marker, no new `Cause` variant, no stamp/teardown verb.

### 2.1 DRY a git helper into `git.rs`

`current_branch` exists privately at `src/dispatch.rs:1297`
(`symbolic-ref --quiet --short HEAD`, `Ok(None)` on detached HEAD). **Promote** it
to `pub(crate) fn current_branch(root) -> anyhow::Result<Option<String>>` in
`src/git.rs`; update the `dispatch.rs` call sites. (DRY — removes the duplication
the predicate would otherwise create.)

### 2.2 A named constant (STD-001)

The `dispatch/` branch prefix is currently a magic string scattered across
`ledger.rs`, `create.rs`, `slice.rs`, `state.rs`
(`format!("dispatch/{slice:03}")`). Introduce a single named constant —
`pub(crate) const DISPATCH_BRANCH_PREFIX: &str = "dispatch/";` — and use it in the
new predicate. (Retro-fitting the existing call sites is **out of scope**; logged
as a STD-001 follow-up so this slice stays small. The *new* code is constant-clean.)

### 2.3 The pure-ish predicate (`worktree/shared.rs`)

```rust
/// True iff `root` is a dispatch coordination worktree: a linked worktree whose
/// HEAD rides a `dispatch/<NNN>` branch (run_coordinate's create+resume invariant).
/// The POSITIVE coordination-tree signal (SL-181) — replaces the marker-absence
/// dependence for the Orchestrator-verb gate. Anti-accident DiD, NOT a boundary:
/// a capable worker can `git checkout -b dispatch/N` to forge it (D2b/RSK-014).
pub(crate) fn is_coordination_worktree(root: &Path) -> anyhow::Result<bool> {
    if !is_linked_worktree(root)? {
        return Ok(false);
    }
    Ok(current_branch(root)?
        .is_some_and(|b| b.starts_with(crate::worktree::DISPATCH_BRANCH_PREFIX)))
}
```

`shared.rs` already `use crate::git;` and owns `is_linked_worktree` — natural home,
no new coupling.

### 2.4 The guard clause (`commands/guard.rs`, Orchestrator class only)

In `worker_guard`, after the existing `resolve_mode` verdict, BEFORE returning Ok
for the not-refused case, add an Orchestrator-only clause:

```rust
let mode = crate::worktree::resolve_mode(&root);
if !mode.refused {
    // NEW (SL-181): the Orchestrator class needs the POSITIVE coord signal in a
    // linked tree; a markerless non-coord linked tree (an unstamped worker) is
    // refused here instead of fail-open. General Write keeps absence-allow (D6a).
    if matches!(write_class(cmd), WriteClass::Orchestrator(_))
        && mode.is_linked
        && !crate::worktree::is_coordination_worktree(&root).unwrap_or(false)
    {
        anyhow::bail!(
            "non-coordination worktree (signal: branch): refusing Orchestrator verb \
             `{verb}` — Orchestrator verbs run only in the dispatch coordination \
             worktree (HEAD on `dispatch/<NNN>`). If this IS the coord tree with a \
             detached HEAD, re-attach: `git checkout dispatch/<NNN>`."
        );
    }
    return Ok(());
}
// … existing refused-path messaging unchanged …
```

`unwrap_or(false)` on the predicate is deliberate: a git failure degrades to
"not a coord tree" ⇒ refuse the Orchestrator verb (fail-closed for the privileged
class on an unreadable tree — the safe direction; general Write is unaffected).

## 3. Truth table (the test matrix)

| context | is_linked | worker marker | branch | Orchestrator verb | Write verb |
|---|---|---|---|---|---|
| primary tree (solo/root) | no | — | (any) | **allow** (unchanged) | allow |
| coord tree (dispatch) | yes | absent | `dispatch/N` | **allow** (positive) | allow |
| worker fork, stamped | yes | present | (any) | **refuse** (marker, unchanged) | refuse |
| worker fork, unstamped — codex | yes | absent | `agent-<hex>` | **REFUSE (new)** | allow* |
| worker fork, unstamped — claude | yes | absent | **detached** (`None`)§ | **REFUSE (new)** | allow* |
| solo `/execute` fork | yes | absent | `slice/SL-N-…` | refuse (won't reach†) | **allow** (D6a) |
| coord tree, detached HEAD | yes | absent | none | refuse (self-brick‡) | allow |

\* general Write stays absence-allow by design — the R-5 belt catches its
`.doctrine/` writes at import; only ref-mutating Orchestrator verbs are the
uncaught gap this slice closes.
† OQ-2 verdict: solo forks never invoke Orchestrator verbs from inside the fork;
the spawner runs `land`/`gc` from root. So this row is unreachable in practice.
‡ self-brick, not regression: no normal op detaches the coord HEAD; cure printed
in the refusal.
§ **Probe-corrected (RSK-014 probe-h1, finding 5):** doctrine's `WorktreeCreate →
worktree create-fork` makes a **detached-HEAD** tree at HEAD for a benign spawn —
NOT a `worktree-agent-<id>` branch (the earlier assumption). `current_branch`
returns `None` ⇒ predicate false ⇒ refuse. Verdict unchanged; the *fork machinery*
(not the harness) owns the branch, which strengthens OQ-A.

## 4. Per-harness value (honest accounting)

- **claude `Agent`** — no bwrap, no env seam *today*: the branch-check is the
  **sole** structural fence against an unstamped worker. **Load-bearing — but
  time-bound.** RSK-014 probe-h1 **proved** claude-arm confinement is achievable
  (`PreToolUse(Bash)→nested bwrap` + `Edit|Write` pathcheck); once that lands as
  its own slice the ro-`.git` wall hard-enforces no ref mutation (RSK-014 OQ-4) and
  this check demotes to belt-and-suspenders, same as codex/pi. Until then,
  load-bearing — and it stays so on the macOS claude arm until `sandbox-exec`
  lands (IMP-045).
- **codex/pi subprocess (bwrap)** — `--ro-bind / /` already makes shared `.git`
  read-only and `DOCTRINE_WORKER=1` already trips the env leg: the branch-check is
  **redundant / belt-and-braces.**
- **vanilla pi subagent** — no confinement: cooperative, like claude.

In all arms it is **anti-accident**, never impersonation-proof (RSK-014).

## 5. The Revision (primary deliverable)

A **REV** against ADR-012 (and the ADR-006 D2a/D2b notes):
- Retract "the positive coordination marker (IMP-065) is **the real close**" of
  OQ-D — it is not; the genuine close is **confinement** (bwrap, linux-only;
  `claude -p`, cost-untenable).
- **Confinement is now proven-achievable on the claude `Agent` arm** (RSK-014
  probe-h1: `PreToolUse(Bash)→nested bwrap` + `Edit|Write` pathcheck; full escape
  battery contained, necessity + fail-open closed). So the residual is **not
  "unclosable on claude"** — it is **"closable, not-yet-landed."** The genuine
  close is a forthcoming slice that graduates `probe-h1/` into the doctrine skill
  hooks (Linux now; macOS via `sandbox-exec`, IMP-045). SL-181 ships the
  **anti-accident interim**; that slice ships the boundary.
- Reclassify the residual as **enforcement-bound, consciously accepted *until the
  confinement slice lands***, pointing at **RSK-014**.
- Record that SL-181 delivers OQ-D plan-gate (i) (the "trusted orchestrator path"
  restriction — never shipped in SL-064; OQ-3) as an **anti-accident** branch-check,
  and plan-gate (ii) impersonation tests as **accident** tests, not malice proofs.

ADR-006 D2a/D2b are owner-locked (VH); the REV is the sanctioned amendment path
(routed via reconcile, not hand-edit).

## 6. Code impact (design-target touch-set)

| path | change |
|---|---|
| `src/git.rs` | promote `current_branch` here (DRY); add `DISPATCH_BRANCH_PREFIX`? (or in worktree — see §2.2) |
| `src/worktree/shared.rs` | new `is_coordination_worktree`; re-export prefix const |
| `src/worktree/mod.rs` | `pub(crate)` re-export `is_coordination_worktree` + the const for `guard.rs` |
| `src/commands/guard.rs` | the Orchestrator-only clause in `worker_guard` |
| `src/dispatch.rs` | drop the private `current_branch`, call `git::current_branch` |

Marker model (`marker.rs`, `subagent.rs`) is **not** touched — the original scope's
new-marker plan is dropped.

## 7. Verification alignment

- **Unit (`is_coordination_worktree`)** — linked+`dispatch/003` ⇒ true; linked+
  `agent-x` / `slice/…` / detached ⇒ false; primary tree ⇒ false.
- **Guard goldens (`worker_guard`)** — the §3 truth table, one assertion per row.
  Orchestrator verb from `dispatch/N` linked tree ⇒ Ok; from `agent-*` /
  `worktree-agent-*` / detached linked tree ⇒ refuse with the branch-signal message;
  general Write from any markerless linked tree ⇒ Ok (D6a preserved).
- **Impersonation tests (OQ-D plan-gate ii)** — explicitly labelled **anti-accident**:
  an unstamped worker (markerless, non-`dispatch/N` branch) is refused for every
  Orchestrator verb. A comment states these do NOT prove the guard stops a *capable*
  worker (it cannot — RSK-014).
- **Behaviour preservation** — `e2e_worktree_coordinate` and existing worker-guard
  suites stay green unchanged (the coord tree rides `dispatch/N` ⇒ still allowed; no
  test encoded the absence dependence for the coord tree specifically).
- **`just gate` / `just check`** green; run with `env -u DOCTRINE_WORKER`
  (mem: worker-verify-unset-doctrine-worker) so tempdir test mints aren't false-refused.

## 8. Design decisions & residual open questions

- **D1 — branch-pattern, not a marker file.** Committed (crash/resume-surviving),
  zero new infra, closes the `--path` seam (guard judges the *caller's* cwd branch),
  DRY. A marker file would add a forgeable runtime artefact + a teardown verb for the
  same cooperative value. **Locked.**
- **D2 — unconditional on the Orchestrator class.** OQ-2 verdict: the coord tree is
  the sole legitimate linked-worktree Orchestrator caller, so no legitimate flow
  breaks. **Locked.**
- **D3 — general Write untouched.** D6a / anti-G2. **Locked.**
- **OQ-A (residual, test-pinned — now better-supported).** The claude unstamped
  worker must run on a non-`dispatch/N` branch during execution for the guard to
  catch it — the `dispatch/N` association happens at collapse-time, after the worker
  process exits. **RSK-014 probe-h1 finding 5 directly observed it: doctrine's
  `WorktreeCreate → worktree create-fork` makes a detached-HEAD tree** (`cwd ==
  .worktrees/agent-<id>`), so `current_branch` returns `None` ⇒ predicate false ⇒
  refuse. The *fork machinery* (doctrine, not the harness) owns the branch — so the
  "future harness runs the worker on `dispatch/N`" break is **less** likely than
  first feared, but still **harness-version-fragile** (a harness that forks on a
  branch, or a doctrine change to the fork verb, could shift it). Pinned by the
  impersonation test (breaks loudly otherwise); residual in RSK-014. **Still the
  inquisition's sharpest target** — verify the dispatch-agent spawn path (`arm-spawn
  --base B`) also yields detached/non-`dispatch/N`, not just the bare `isolation:
  worktree` spawn the probe used.
- **OQ-B (accepted).** A hand-created `dispatch/N` branch in a worker tree forges the
  signal. Out of scope (D2b / capable-worker / RSK-014).
- **OQ-C (minor).** Detached-HEAD in the coord tree self-bricks Orchestrator verbs.
  Acceptable (no normal op detaches; cure printed). Confirm no merge-conflict path
  leaves the coord HEAD detached (refresh-base/land restore the branch — design says
  so; verify in execute).

## 9. Self-review (adversarial pass — integrated)

Findings from the internal hostile read, and their resolution:

- **F1 — "redundant on codex/pi, why ship it?"** Kept: load-bearing on claude (the
  dominant arm), and the cost is ~30 lines + tests. §4 states the redundancy
  honestly rather than hiding it. Accept.
- **F2 — "branch-check is also cooperative — same critique as the marker."** True and
  now stated as the thesis (§0). The slice's value claim is downgraded to anti-accident
  everywhere; the REV retracts the boundary claim. This is the *point*, not a flaw.
- **F3 — claude-worker-on-`dispatch/N`?** Promoted to OQ-A; could silently void the
  whole guard on the load-bearing arm if wrong. Test-pinned; flagged as the
  inquisition's primary target. **Highest residual risk.**
- **F4 — `unwrap_or(false)` swallows git errors.** Deliberate and safe: degrades to
  refuse for the privileged class only; general Write unaffected. Documented inline.
- **F5 — does `resolve_mode` already cover the linked check so the predicate
  double-reads git?** Minor perf (a couple of `git` shells per Orchestrator verb,
  which are rare and already git-heavy). Acceptable; no caching needed.
- **F6 — STD-001 scatter.** The new code uses a named constant; the pre-existing
  `dispatch/` scatter is logged as a follow-up rather than fixed here (scope control).
- **F7 — module layering (ADR-001).** `command/guard.rs → worktree → git` is the
  established direction (guard already calls `crate::worktree::*`); no cycle. Confirmed.

Unresolved-by-design (carried into execute/inquisition): **OQ-A** (claude branch
assumption), **OQ-C** (coord detached-HEAD paths).
