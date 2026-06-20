# Design SL-123: Claude dispatch arm fail-closed base integrity

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

## 1. Design Problem

ISS-034 Defect A. The claude `/dispatch` arm (`/dispatch-agent`) places each
worker at base==B by `cd`-ing the Bash cwd into the coordination worktree before
spawning an `Agent` with `isolation: worktree` (which forks the Bash-cwd HEAD under
`worktree.baseRef:"head"`). Under a busy shared single clone, concurrent
`git worktree` ops contend on git's repo-global locks; when a worker's worktree
creation loses the race the Claude Code subagent **silently falls back to the main
worktree**, where `baseRef:"head"` then tracks a moving `main`. The worker runs on
a wrong/dirty/moving base instead of B (SL-121 PHASE-02, failed 3× consecutively;
one worker started correct and was clobbered to `main` mid-run).

Make the arm **fail closed** against this fallback — so it is caught loudly,
independent of a hand-added prompt guard. (Defect B — the `SubagentStart`
`(deleted)` stamp-hook path — is folded into ISS-011 and out of scope.)

## 2. Current State

`doctrine worktree verify-worker --base <B> --dir <worktree>` (`src/worktree.rs`)
already runs post-spawn and refuses with a distinct token on:
- `no-worker-head` — worker HEAD does not resolve;
- `unstamped` — worker marker absent;
- `wrong-base` — `merge-base --is-ancestor B HEAD` false.

It is read-classed (survives worker-mode confinement, ISS-028), pure/imperative
split (pure `classify_worker_verify` + impure `run_verify_worker` shell, ADR-001).
`is_linked_worktree(dir)` (`worktree.rs:404`, git-dir ≠ git-common-dir) already
exists and is the exact, jail-layout-independent test for "is this the main tree."

The `dispatch-agent` skill spawn template `cd`s into the coord tree, spawns, then
runs `verify-worker`. The base-guard that made SL-121 fail closed was **ad-hoc
prose**, not in the template. No defined response to a missing `worktreePath:`
footer (the signal that no isolated tree was created).

**Residual gaps (Defect A, narrowed):**
- G1 — `verify-worker` catches a *wrong-base* fallback but NOT a fallback-to-main
  where B is *coincidentally* an ancestor (e.g. phase-1, B==main tip): the worker
  ran un-isolated in a dirty, concurrently-mutated tree yet passes ancestry.
- G2 — the base-guard is ad-hoc, not standard.
- G3 — a missing `worktreePath:` footer has no defined orchestrator response.

## 3. Forces & Constraints

- **ADR-006** orchestrator-sole-writer — the worker is untrusted; the authoritative
  gate must be orchestrator-side. Read-only worker guards are fine.
- **ADR-008 / ISS-028** worker-mode confinement — any worker-callable verb must be
  read-classed.
- **ADR-011 D6** — pre-worker fail-closure is NOT available on the claude arm; the
  accepted residual class is "loud-and-late, post-worker." This slice strengthens
  *within* that class; it does not claim pre-worker fail-closure.
- **ADR-001** leaf/engine layering + CLAUDE.md pure/imperative split — gather →
  pure-classify → act; no git/disk/clock in the pure layer.
- **Behaviour-preservation gate** — existing `verify-worker` suites must stay green
  (verdicts unchanged for non-fallback cases).
- **Skill-budget VT** — `e2e_skills_dispatch_shrinkage.rs` caps `dispatch-agent`
  body at ≤61 lines (file is currently exactly 61).

## 4. Guiding Principles

- The authoritative belt lives orchestrator-side (trust boundary, ADR-006).
- Reuse the existing classifier + `is_linked_worktree`; add no new git plumbing.
- Layered, independent belts — each sufficient alone, none load-bearing alone.
- As simple as possible: close the one code-detectable hole (G1); keep the
  worker-side guard as template prose (an LLM follows a prompt either way).

## 5. Proposed Design

### 5.1 System Model

Three independent belts, in order of action:

1. **In-worker base-guard (fail-fast, prose).** Worker's first action; STOP if the
   tree is dirty, not an isolated linked worktree, B is not an ancestor, or
   prerequisite seams are absent. Saves wasted work; not the authoritative belt.
2. **Footer check (orchestrator, no-tree case).** A missing `worktreePath:` footer
   in the Agent return ⇒ no isolated tree was created ⇒ abort before import.
3. **`verify-worker not-isolated` (orchestrator, authoritative code belt).** Catches
   the fallback even when a footer is present and base is coincidentally an
   ancestor — the case G1 that belts 1–2 can miss.

### 5.2 Interfaces & Contracts

**Pure classifier** — new fact `is_isolated`, new variant `NotIsolated`
(token `not-isolated`), inserted #2 (after head, before marker/base):

```rust
pub(crate) enum WorkerVerifyRefusal { NoWorkerHead, NotIsolated, Unstamped, WrongBase }
// token(): NotIsolated => "not-isolated"

pub(crate) fn classify_worker_verify(
    head_resolved: bool,
    is_isolated: bool,       // NEW — git-dir ≠ git-common-dir
    marker_present: bool,
    base_is_ancestor: bool,
) -> Result<WorkerVerify, WorkerVerifyRefusal> {
    if !head_resolved    { return Err(NoWorkerHead); }
    if !is_isolated      { return Err(NotIsolated); }   // fork exists but it's the main tree
    if !marker_present   { return Err(Unstamped); }
    if !base_is_ancestor { return Err(WrongBase); }
    Ok(WorkerVerify::Ok)
}
```

**Shell** (`run_verify_worker`) — reuse `is_linked_worktree`, short-circuit on head:

```rust
let head_resolved = git::git_opt(dir, &["rev-parse","--verify","HEAD"])?.is_some();
let is_isolated   = head_resolved && is_linked_worktree(dir)?; // missing dir → NoWorkerHead wins
let marker        = marker_present(dir);
let base_is_ancestor = git::git_status_ok(dir, &["merge-base","--is-ancestor",base,"HEAD"])?;
```

**Skill** (`dispatch-agent/SKILL.md`) — see §5.4.

### 5.3 Data, State & Ownership

No new persistent state. `verify-worker` remains diagnostic-only (never removes a
fork; the orchestrator owns disposition of a refused worker). The base-guard block
is template text the orchestrator embeds (filling `<B>` and `<seams>` at distill
time). The trust boundary is unchanged: orchestrator authoritative, worker
untrusted.

### 5.4 Lifecycle, Operations & Dynamics

`dispatch-agent/SKILL.md` changes:

**(a) Mandatory base-guard block** (worker prompt preamble, the orchestrator fills
`<B>`/`<seams>`):

```
BASE GUARD — run FIRST, before any read/edit/commit. STOP and write nothing if any check fails:
  1. git status --porcelain                         → MUST be empty (clean tree)
  2. git rev-parse --git-dir vs --git-common-dir    → MUST differ (isolated linked worktree, not main tree)
  3. git merge-base --is-ancestor <B> HEAD          → MUST exit 0 (HEAD descends from base <B>)
  4. grep prerequisite seams: <seams>               → MUST be present
On any failure: STOP, author/commit nothing, report "base-guard-failed: <check>".
```

Check #2 is the in-worker mirror of the orchestrator's `not-isolated`.

**(b) Post-spawn cadence** (replaces the current Post-spawn section):

```
## Post-spawn
1. Read the Agent return footer. NO `worktreePath:` footer ⇒ no isolated tree was created
   (fallback-to-main under lock contention) ⇒ ABORT, do NOT import. Re-dispatch, or switch
   to the subprocess arm if main is churning.
2. doctrine worktree verify-worker --base <B> --dir <worktreePath>
   Abort on any refusal: no-worker-head / not-isolated / unstamped / wrong-base.
```

**(c) Red Flags** — add: never import a worker that returned no `worktreePath:`
footer; always embed the base-guard block in the distilled prompt.

`dispatch/SKILL.md` (router) is **untouched** — the `worktreePath:` footer is a
claude `Agent` artifact, so the footer/guard cadence is claude-arm-specific.

### 5.5 Invariants, Assumptions & Edge Cases

- **Ordering invariant:** `head_resolved` is checked before `is_isolated`, so the
  short-circuit (`is_isolated=false` when head unresolved) never misreports — a
  missing/nonexistent `--dir` always surfaces as `no-worker-head`.
- **Edge — primary tree that is stamped and ancestor:** previously passed
  (`Ok`); now refused `not-isolated`. This is the G1 fix and the only intentional
  verdict change. The behaviour-preservation gate covers all *isolated* cases.
- **Edge — genuine git spawn failure** (broken env): propagates loud via `?`, not
  masked as `not-isolated`.
- **Mid-run clobber:** `verify-worker` samples the *final* cwd post-return → catches
  a clobber that the in-worker pre-flight guard (belt 1) cannot. If the worker
  committed to a clobbered main branch instead of its fork branch, the funnel's
  import/delta check finds an empty delta — a further backstop, unchanged.
- **Assumption:** the funnel already calls `verify-worker --base B` pre-import
  (confirmed: skill Post-spawn). This slice strengthens that belt, adds no stage.

## 6. Open Questions & Unknowns

- **OQ-1 — `baseRef`→SHA pinning (ISS-034 remedy 1): DROPPED.** `settings.local.json`
  is repo-wide (can't flip per-dispatch); placement+verify is the ADR-011 model;
  even a SHA-valued `baseRef` would still need the verify belt for the race. No net
  assurance gain.
- **OQ-2 — `WorktreeCreate` pre-worker hook (IMP-072): DEFERRED.** Within ADR-011 D6
  the post-worker class is accepted; true pre-worker fail-closure is a separate,
  larger change. IMP-072 stays open with SL-123 as trigger context.
- **OQ-3 — main-worktree identification: RESOLVED.** `is_linked_worktree`
  (git-dir vs git-common-dir) is jail-layout-independent and already in use.

## 7. Decisions, Rationale & Alternatives

- **D1 — belt-set = B (skill + `not-isolated` in code).** Rejected A (skill-only:
  leaves G1 uncaught by code) and C (worker-run `assert-base` CLI: adds NO
  correctness assurance over B — the worker is untrusted, so the authoritative gate
  must be orchestrator-side regardless; C is logically subsumed by `verify-worker`,
  and for mid-run clobber the post-spawn sample is strictly stronger; C buys only
  fail-fast/UX at the cost of a new verb + worker-mode plumbing + goldens). The
  worker-side guard stays as template prose (belt 1) for fail-fast only.
- **D2 — `NotIsolated` ordered #2** (after head, before marker/base). Being the main
  tree is the *root cause* of the fallback; marker/base are downstream symptoms a
  main tree can coincidentally satisfy. Report the structural fault first.
- **D3 — skill-budget bump is deliberate.** The slice adds *required safety prose*;
  raise the `dispatch-agent` cap (≤61 → ≤~78, exact at execute) and add presence
  asserts (`base-guard`, `not-isolated`, `worktreePath`) so the cap stays
  meaningful rather than a pure size gate.

## 8. Risks & Mitigations

- **R1 — false positive `not-isolated`** on a legitimate fork. Mitigated: the test
  is git-dir ≠ git-common-dir, the same primitive already trusted by
  `verify_sibling_worktree`/the worker-mode guard; integration test asserts `Ok`
  on a real linked worktree.
- **R2 — skill prose drift** (base-guard not embedded by the orchestrator).
  Mitigated by the presence asserts (D3) + the two orchestrator belts that do not
  depend on belt 1.
- **R3 — budget bump masks future bloat.** Mitigated by the added presence asserts
  pinning the *content* the budget must cover.

## 9. Quality Engineering & Validation

- **VT** — pure goldens: `not_isolated_refuses_after_head_before_marker`; existing
  `classify_worker_verify_*` goldens take the new `is_isolated` arg, verdicts
  unchanged (behaviour-preservation).
- **VT** — `run_verify_worker` integration: refuses `not-isolated` against a primary
  tree, `Ok` against a linked worktree on B (existing worktree test harness).
- **VT** — `e2e_skills_dispatch_shrinkage.rs::dispatch_agent_skill_is_shrunk`:
  raise cap; add `base-guard` / `not-isolated` / `worktreePath` presence asserts.
  Router test untouched.
- **VA** — `dispatch-agent` prose: base-guard block carries the 4 checks; Post-spawn
  carries footer-abort + the four refusal tokens.
- **Closure** — ISS-034 Defect A reconciled (resolved); IMP-052 overlap noted (the
  footer-abort cadence partly delivers its "abort an un-isolated/unstamped worker"
  intent — do not auto-close).

## 10. Review Notes

(adversarial pass pending — codex mcp)
