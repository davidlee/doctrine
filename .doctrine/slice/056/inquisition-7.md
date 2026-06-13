# Seventh Inquisition — SL-056 design (`design.md`, re-lock target)

Convened for `nihil obstat` after six prior passes. The sixth-round remediations
are present in the design: sentinel lifecycle, `wedged-merge`, `worktree-gone`,
and the per-harness router fallback have all been integrated. This pass therefore
does not re-charge κ/λ/μ/ι as written.

Doctrine consulted: `doctrine slice show SL-056`; `SL-056` `design.md`;
`inquisition-6.md`; ADR-006 (D2/D2b/D6a/D7/D9); ADR-008 (D-B1/D-B3); SPEC-012
(`draft`, pre-rewrite); the boot storage/process rules; and the scoped memory
retrieval for `.doctrine/slice/056`.

**`Nihil obstat` is DENIED.**

## Charges

### Charge ν (nu) — CRITICAL — the codex/pi spawn template creates the fork but never binds the worker process to that fork

**Doctrine violated.** ADR-006 D2 says dispatch workers mutate source code only
inside their worker context while doctrine writes funnel through the orchestrator;
ADR-006 D2b names raw-tree writes as the residual risk to be confined, not a path
the blessed spawn line may accidentally take. `SL-056`'s own thesis is that
mechanism belongs in the verb/shell, not in prompt faith.

**Evidence.** D1 says the trusted orchestrator runs `doctrine worktree fork` at the
source root and creates a worktree at `--dir <path>` (`design.md:181-188`). The
codex/pi usage template then captures env and invokes:

```sh
env DOCTRINE_WORKER=1 $fork_env codex exec "<pre-distilled prompt>"
```

with no `cd "$D"`, no `--cwd "$D"`, no `--chdir "$D"`, and no equivalent
worktree-directory binding (`design.md:225-236`). D2's env leg catches only
doctrine-mediated writes on the coordination root (`design.md:369-375`,
`design.md:439-460`); it does not and cannot stop a worker editor or shell command
from mutating source files in the current working directory. D6's bwrap confinement
is later, spike-first, codex/pi-only, and explicitly conditional (`design.md:789-808`);
absent D6, the shown spawn line leaves the worker on the coordination root.

**Risk.** The core dispatch guarantee collapses for the subprocess harnesses:
`fork` can create and mark the correct worktree while `codex exec` runs in the
coordination checkout. The env guard may refuse `doctrine slice new`, but raw
source edits land directly on the trusted branch, bypassing `import`, verification,
the `.doctrine/` belt, branch-point discipline, and `gc`. This is the exact
worker-on-main hazard with the dangerous half still open: source mutation.

**Sentencing.** Make the subprocess spawn shell bind cwd as a first-class
mechanism. Either the harness template must `cd "$D"` before spawning, use a
verified harness cwd flag, or run bwrap with an explicit `--chdir "$D"`; the
choice belongs in `/dispatch-subprocess`, not in the prompt. If the fork stdout
contract carries the directory, name it explicitly (for example
`DOCTRINE_WORKTREE_DIR=<path>`) and validate/quote it as machine data. Add a
codex/pi spike/golden that runs a trivial worker command from the template,
asserts `pwd` is the fork, writes a source file, and proves the coordination root
stays clean.

### Charge ξ (xi) — HIGH — `gc` owns a non-atomic cleanup sequence but has no idempotent recovery model for its own partial states

**Doctrine violated.** The design's mechanism admission rule requires every
cleanup path to name its remover, failure states, and verification
(`design.md:150-164`). Charge VIII's standard, already applied to `fork` and
`land`, is that fallible cleanup reports the leftover by name and exits non-zero,
never leaving an operator to infer the state.

**Evidence.** `gc` performs three destructive cleanup steps: remove worktree,
delete branch, reap target dir (`design.md:557-568`). The design acknowledges an
intrinsic crash window after `git worktree remove` and before `git branch -D`,
which leaves `branch alive, worktree gone, marker unreachable` (`design.md:571-580`).
The round-6 fix teaches `land` to refuse that state (`design.md:705-717`,
`design.md:1034-1038`), but it does not say how `gc` resumes and completes the
cleanup it started. Nor does D4 specify named behavior for later half-failures
such as branch deletion succeeding but target-dir reaping failing, after which a
branch-keyed rerun may no longer have the same facts available.

**Risk.** The cleanup owner can strand its own debris. A crash after step 1 leaves
a branch that `land` now correctly refuses, but a rerun of `gc` may fail at
"remove worktree" because the worktree is already gone unless an idempotent
state machine is designed. A failure after branch deletion can orphan the
`wt/<branch>` target dir indefinitely. The design closes the downstream
mis-merge hazard while leaving the cleanup verb itself under-specified.

**Sentencing.** Specify `gc` as an idempotent state machine over the presence of
`{branch, linked worktree, target dir}`. It should compute the landed/superseded
permission while the branch still exists, skip already-completed steps, complete
safe remaining steps, and name any leftover path/ref on failure. Verification
must inject or simulate failure after each destructive step, then rerun `gc` and
assert either full cleanup or a named non-zero leftover report. Include the
branch-gone/target-present case so disk reaping does not depend on a live branch.

### Charge ο (omicron) — HIGH — the arm lease can expire a slow-but-live Agent back into the mis-brand race

**Doctrine violated.** The sixth-round sentinel fix is supposed to make
serial-only Claude stamping mechanised, not faith-based. The design itself states
that the Agent tool returns no worktree handle, so the orchestrator cannot bind an
arm to the specific WorktreeCreate event it intended (`design.md:260-263`).

**Evidence.** The new sentinel lifecycle uses lease expiry: a later `--arm` that
finds an expired lease auto-reclaims it, emits `stale-arm-cleared`, and proceeds
(`design.md:300-310`). The safety argument is that the lease is "longer than any
plausible spawn→WorktreeCreate window" and therefore only reclaims abandoned
slots (`design.md:307-310`). But the same design says the shell/harness boundary is
impure and uncorrelated: a first Agent can be delayed rather than dead. If that
first WorktreeCreate fires after the lease was declared stale and a second arm has
been created, the old Agent can consume the new live sentinel and stamp the wrong
tree, recreating the exact γ/θ race the single-slot mechanism was introduced to
remove.

**Risk.** A timeout is being used as a proof of death in a mechanism that lacks a
process handle or correlation token. Under scheduler stalls, hook delay, IDE
latency, or a slow first Agent, `stale-arm-cleared` can turn a safe brick into a
fail-open mis-brand. The design has moved from "no key" to "a key that may open
the wrong cell."

**Sentencing.** Do not let expiry both clear and proceed unless the mechanism can
prove the prior Agent cannot still create a worktree. If Claude's hook cannot
receive and verify an unambiguous arm token, expired arms should become an
`expired-arm` refusal requiring explicit operator recovery, and that run should
degrade to prompt-enforced/no-marker rather than immediately re-arming. Add a
delayed-first-Agent spike: arm, let the lease expire without stamp, attempt a
second arm, then fire the first hook late. The required result is no wrong-tree
stamp and no self-belief-only fallback.

### Charge π (pi) — MEDIUM — the Orchestrator-class verification list is stale and omits `land`

**Doctrine violated.** DC-3 creates an `Orchestrator` privilege class for
git/ref/directory-mutating verbs and says it is refused under worker identity
(`design.md:73-83`). D4b adds `land` to that class because it writes a coordination
merge commit (`design.md:691-696`). The verification must drive `run()`, not a
pure helper, for the guarded command surface.

**Evidence.** The main verification bullet for `Orchestrator` refusal still
lists only `fork` / `import` / `gc --force` from a marked fork or env-set process
(`design.md:902-904`). Later additions cover `marker --arm`/`--disarm`
(`design.md:1020-1027`), but no line explicitly pins `land` as refused under
worker identity. The code-impact row says `land` is wired and classified
(`design.md:880-881`), but the black-box guard proof does not enumerate it.

**Risk.** This is a verification gap on the exact class that previously let a
worker delete refs when mutating verbs were treated as `Read`. If `land` or a
future Orchestrator variant misses the guard, a worker regains a coordination-ref
mutation surface.

**Sentencing.** Replace the stale list with an exhaustive Orchestrator-class
guard invariant that enumerates every current member: `fork`, `import`, `gc`,
`land`, `marker --arm`, and `marker --disarm`. Keep `marker --clear` out of that
class intentionally and test its bespoke refusal rules separately. The test must
drive `run()` from both marker and env worker-mode signals.

## Questions

1. For codex/pi, what exact mechanism sets the worker process cwd to the fork
   directory before any source edit can occur?
2. After `gc` crashes between worktree removal and branch deletion, is rerunning
   `gc --fork <branch>` required to complete cleanup? If yes, where is that
   idempotent state model specified?
3. Can Claude's WorktreeCreate hook verify an arm token tied to the intended
   spawn, or is the lease expiry only a timeout heuristic?
4. Should `land` be added to the existing Orchestrator-class guard verification,
   or is there another proof that a marked worker cannot invoke it?

## Pronounce Judgement

`SL-056` is not ready for `/plan`. The sixth-round repairs were real, but the
seventh pass finds three live design faults and one verification gap:

- codex/pi workers are forked but not necessarily run inside the fork;
- `gc` acknowledges a partial cleanup state but assigns recovery to `land`
  instead of to `gc`;
- the Claude arm lease uses timeout as liveness proof despite the lack of a
  hook correlation handle;
- `land` is missing from the Orchestrator-class refusal proof.

The pattern is narrower than the earlier rounds: the core topology is mostly
stable, but the remaining risk sits in harness shell mechanics and partial-state
recovery. Those are precisely the places prose tends to smuggle mechanism back in.

## Sentencing

1. Fix ν first. Specify and verify subprocess cwd binding to the fork before any
   worker command runs. Include the bwrap `--chdir` equivalent if D6 lands.
2. Fix ξ. Make `gc` resumable over partial cleanup states and verify rerun-after-
   crash for every destructive step.
3. Fix ο. Replace auto-reclaim-and-proceed with either a correlated arm token or a
   conservative expired-arm refusal/manual recovery path.
4. Fix π. Update the Orchestrator-class guard verification to include `land` and
   all current class members.

After remediation, integrate these findings into `design.md`, then convene an
eighth confirmatory inquisition before `/plan`.
