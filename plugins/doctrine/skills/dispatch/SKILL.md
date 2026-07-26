---
name: dispatch
description: Use to drive a slice's phases to completion through sub-agent workers in isolated worktrees — you orchestrate and are the sole writer, the workers execute. Routes to `/dispatch-subprocess` (codex/pi) or `/dispatch-agent` (claude); overridable via `[dispatch] claude-force-subprocess-dispatch` in `doctrine.toml`. The per-phase funnel is driven by `doctrine dispatch next`, which prescribes exactly one action at a time and is identical on both arms. Default serial (one worker per phase); parallelize file-disjoint phases. Conflicts report-and-halt, never auto-merge.
---
# Dispatch (router)
Drive a slice's phases to completion through sub-agent **workers** — you are the
orchestrator and **sole writer**, they execute.

**Announce at start:** "Using the dispatch skill to run workers under the
orchestrator funnel."

## The outer loop
1. `dispatch setup --slice <N> --dir <path>` — create/resume coordination worktree.
   On the claude arm `--dir` MUST resolve inside the project root (convention
   `.dispatch/SL-<n>`); an outside-root dir fails closed (ISS-031 — the pre-spawn
   `cd` silently reverts under a jail, forking `main` not B).
2. **Claude arm only:** `cd` into the coordination directory and park Bash cwd
   there for the full drive loop. The Agent tool's `isolation: worktree` forks off
   the Bash cwd HEAD — base==B holds only if the cwd is parked at B before the
   spawn (a mis-parked cwd forks the wrong base). Step out to the session root
   only for authored writes (slice status, memory, audit). Commit those authored
   writes with `dispatch commit --slice N -m <msg> -- <path>…` (pathspec-mandatory,
   ISS-234-guarded), never a raw `git commit`.
3. `dispatch plan-next --slice <N>` — find next actionable phase(s); plan parallel batches when file-disjoint
4. Route to the correct arm:
   - Check `doctrine.toml` → `[dispatch]` → `claude-force-subprocess-dispatch`
     (default `false` if the file or key is absent).
   - If `true`, route workers via [`/dispatch-subprocess`](../dispatch-subprocess/SKILL.md)
     (default to `pi` arm until `preferred-subprocess-harness` selection is wired — IMP-101).
   - Otherwise, route per env-marker: `.claude/` present →
     [`/dispatch-agent`](../dispatch-agent/SKILL.md); otherwise →
     [`/dispatch-subprocess`](../dispatch-subprocess/SKILL.md).
   Then spawn worker(s) per the chosen arm's template.
5. Drive the batch's funnel by `doctrine dispatch next --slice <N>` (below) until it
   says `all-reaped`
6. Conclude: `slice verify-vt <id>` (VT gate, coord tree) → on green
   `dispatch sync --prepare-review` → remove coord worktree → audit

## The funnel (per batch)

**Base-clean precondition (pre-spawn, NON-mutating).** Before capturing `B` /
spawning any worker — and on `main` before you branch the coordination fork —
assert the base is prove-clean:
```
doctrine check prove     # fmt-check + lint; asserts, never fixes
```
A RED base is a **BASE defect**, DISTINCT from any worker finding: it means the
tree you are about to fork from is already unformatted or lint-red. Remediate it
operator-side — a format-and-commit you own (or an isolated prep worktree) — and
NEVER fold it into a worker delta, NEVER auto-fix it. A worker spawned off a dirty
base would either inherit the red (and be halted at import for a defect it did not
cause) or launder it. Run this exactly once per batch here on the hot path (the
only other prove run is the post-import gate below — do not double-run).

Capture `B = git rev-parse HEAD` pre-spawn, then capture the S1 regression
baseline on the coord tree at `B`:
```
doctrine check regression capture --base "$B"     # suite @ B; no-op on cache hit
```
**INV-1 — normalise filter state before BOTH this capture and the verify diff**:
clear the worker marker (`doctrine worktree marker --clear --operator`) and force a
real rebuild, so capture and diff run an identical suite invocation + test
selection. Same tree alone is insufficient — a leaked `DOCTRINE_WORKER`/marker
changes which tests run and breaks the cancellation property (a fingerprint
mismatch is then a cache miss → honest re-capture, never a poisoned baseline).

Worker self-reports are advisory; trust only the funnel verify beat.

### The oracle drives — there is no remembered sequence

Once the workers are spawned, **do not hold an ordered checklist and do not read
the funnel state out of git by hand.** Ask the funnel:

```
doctrine dispatch next --slice <N>          # add --json for the payload
```

It reads the committed funnel record and prescribes **exactly one** action, with
the runnable literal in the surface that owns that verb. Do that one thing, then
ask again. Repeat until it says `all-reaped`. It is read-only, never heals, and
always exits 0 — a red prescription is information, not a command failure.

| `kind` | what it prescribes |
|---|---|
| `spawn` | no funnel row yet — route to the arm skill and spawn (arm-specific, so `next` emits no literal) |
| `await-worker` | the fork is armed; wait for the worker's own commit |
| `import` | land the worker delta (`dispatch_import`, MCP-only) |
| `verify` | run the phase suite and land evidence (`doctrine dispatch verify`, CLI) |
| `reverify-stale` | the pass evidence no longer describes the tip — re-run the suite |
| `triage-verify-failure` | RED evidence — a judgment beat, no command; triage before anything else |
| `conclude` | land the phase boundary (`dispatch_conclude_phase`) |
| `reap` | remove the spent fork (`dispatch_reap`) |
| `all-reaped` | the phase funnel is done — hand off to `doctrine dispatch status --slice <N>` |

Parallel file-disjoint phases stay single-prescription: red evidence anywhere
triages first (the suite is coord-tree-global), an awaiting phase never
suppresses a runnable one, and the other in-flight phases are named in `detail`
at every rung.

**Two altitudes.** `next` answers the *phase funnel* only. Everything past it —
prepare-review, base freshness, candidates, admission, close — is `dispatch
status`, which is where `all-reaped` sends you.

**A refusal is the recovery procedure.** Every funnel verb that refuses prints
``<verb> refused at position `<pos>` — <reason>; expected: <next>``. Read it and
act on it. Do not improvise a repair, do not go digging for the state by hand,
and do not re-drive around it — a refusal is a defect or a halt, never a detour.

### The beats the oracle cannot emit

- **Verify's regression half.** At the `verify` prescription also run `doctrine
  check regression diff --base "$B"` (suite @ S, the SAME normalised filter state
  as the capture). Exits non-zero on `new ∪ changed` (a slice regression
  regardless of which test binary/env it surfaces under) OR an unobtainable run
  (a compile error / panic / format change is a hard halt, never a silent green
  ∅). `persistent` (same key + same signature at B and S) is the tolerated env
  artifact; a non-empty baseline is surfaced as a trunk warning to fix, not
  laundered as "env". On halt the named `new`/`changed` keys ARE the offenders;
  nothing further needs isolating. (Carry-forward of the green current-set as
  `baseline-<B'>` is a deferred cost optimisation; steady state still re-captures.)
- **The fallback import's prove gate.** The claude arm's primary path is already
  commit-gate-green at source (`worker_commit` ran the gate) and
  `dispatch_import` composes + commits server-side. The CLI
  `import --from-worktree` fallback is non-committing and runs the
  **reject-and-halt prove gate** in-process (`doctrine check prove` on the
  post-import tree): an unformatted OR lint-red delta HALTS the import (staged,
  NOT committed) and is reported — never auto-fixed (ADR-012 sole-writer:
  land-or-reject, never rewrite). A red here is a WORKER-delta defect, distinct
  from the pre-spawn BASE defect above.
- **The codex/pi arm's registry write.** At the `conclude` prescription that arm
  additionally runs `doctrine slice record-delta <SL> PHASE-NN --commit <S>` —
  its commit-scoped `[S^,S]` write into the primary-tree conformance registry
  (symmetric derive deferred, D6/IMP-171; `/dispatch-subprocess`). On the claude
  arm `dispatch_conclude_phase` lands the boundary row and flips the phase sheet
  in one call. Neither is a "remember to also record" hand-step: the Conclude
  beat's completeness gate halts if a landed phase is missing its row (below).
- **Per-phase review.** Between `import` and `conclude`, weigh a review of the
  landed delta per the code-review skill's `## Cadence`: default on below the
  adherence bar, mandatory on any tripwire (deleted tests, `Deviations: NONE`,
  waived/uncheckable VT, out-of-scope touch).
- **Knowledge** trails the confirmed commit, never precedes it.

**Report-and-halt** on conflict, moved HEAD, or authored-tree touch — never auto-resolve.

## Handover cadence
Hand over at a committed boundary: after `handover_after` batches (default 5) or
`handover_delta` cumulative reviewed-delta lines (default 2000), whichever first.

## Base freshness (mid-drive)
A long drive lets trunk advance under the coordination branch; the drift stays
invisible until `dispatch sync`/candidate-create conflicts on a merge-base
divergence — the most disruptive place to discover it. `dispatch status` surfaces
it (`trunk: moved (N commit(s) ahead of fork-point)`). When status shows movement,
run `dispatch refresh-base --slice <N>` — it merges current trunk into
`dispatch/<N>` in the live coordination worktree, advancing the base early and in
context so each conflict is one phase's delta. Conflicts there report-and-halt for
manual resolve in the coord tree — never auto-merged.

## Conclude
When all phases land, run the conclude cadence **in the coord tree, before it is
removed** (SL-170 S3/S6): `slice verify-vt <id>` → on green
`dispatch sync --prepare-review` → remove coordination worktree directory (KEEP
the refs) → `slice status <id> audit` → `/audit` from parent/root. Stage-2
integrate is `/close`'s job, post-audit — never land code pre-audit.

`slice verify-vt <id>` is the **VT existence/shape gate** (S3): it reads the coord
tree's `plan.toml` and checks every `VT`-mode criterion's mandated `test_file` +
`keywords`. A `Fail` exits non-zero and **HALTS handover** — do not prepare-review
past it; `/consult` → revise-or-waive the authored plan (never self-relax a
mandate), then re-run. `Uncheckable` / `Waived` are visible but non-halting. The
fs reader suffices here because the orchestrator (sole writer) has committed any
mid-dispatch waiver onto `dispatch/<slice>`, so the coord working tree == the
committed graph `prepare-review` projects (INV-6).

**Embed at handover (S6):** carry the `verify-vt` VT summary block **and** a
one-line S1 regression status (lifted from the verify beat's `check regression
diff`) into the conclude output and the `/handover` packet — so a gap (incl.
`UNCHECKABLE` / `WAIVED`, rendered distinctly) is visible at handover, not at audit.

`prepare-review` is the **enforced** conformance beat (ISS-052): before projecting
refs it commits the boundaries ledger, **derives** the registry from that committed
ledger on the claude arm (the committed ledger is the source), then runs a completeness
**gate** that `bail!`s if any completed phase lacks a registry row — both arms. So
the registry is guaranteed complete by audit; a gap halts here (no refs created),
the operator commits the ledger / `record-delta`s the gap, and re-runs.

## Red Flags
IMPORTANT: READ VERY CLOSELY

**Never:** spawn without routing; let a worker write `.doctrine/`/`.claude/`;
commit per worker; replay fork history; auto-merge conflicts; auto-adapt plan/
design (`/consult` forks); drive on session `main`; integrate at conclude; delete
deliverable refs; bail to inline execution; **hand-sequence the funnel from
memory, or work the funnel state out of git by hand — ask `dispatch next`**;
improvise a repair around a refusal (its text IS the procedure).
! NEVER bail to inline execution - if you are about to `/execute`, STOP.
! NEVER use git like a drunk with a chainsaw - if you are about to do something potentially risky, STOP.
**Always:** route to correct arm on confirmed harness; keep context lean
(capped reports, stat-first diffs); let `dispatch next` set the order and do the
ONE thing it prescribes; pre-distill self-contained worker prompts; trail
knowledge after the confirmed commit.
