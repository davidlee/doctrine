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
exists: it distinguishes the **primary** tree from a **linked** worktree
(jail-layout-independent). It does NOT prove "this is the intended fresh worker
fork" — only "this is/ isn't the primary tree." That is exactly the discriminator
the `not-isolated` belt needs for the observed primary-tree fallback; the
non-primary misplacements are caught by the marker/base/import belts (§5.1).

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

### 5.1 System Model — layered defense, NOT "each belt sufficient"

This slice does not close every wrong-base manifestation with a single new belt;
it adds the belt that closes the **observed** one (primary-tree fallback) and shows
the remaining manifestations are already caught downstream by the
**harness-identical funnel import belt** (`dispatch/SKILL.md:37-46`,
`classify_import`, `worktree.rs:665`). Defense is layered and scenario-specific.

Map of manifestation → the belt that catches it (✦ = added by this slice):

| Failure manifestation | Caught by |
|---|---|
| No isolated tree created at all (no `worktreePath:` footer) | ✦ footer-abort, pre-funnel (§5.4b) |
| Worker starts in the **primary** main worktree (footer present, base coincidentally an ancestor) | ✦ `verify-worker not-isolated` + in-worker guard #2 |
| Worker lands in the **coordination** tree (markerless) | existing `verify-worker unstamped` |
| Worker lands in **another** linked fork (HEAD ⊉ B) | existing `verify-worker wrong-base` |
| Footer `worktreePath`/`worktreeBranch` incoherent (point at different states) | ✦ `verify-worker branch-mismatch` (`--branch`, §5.2) |
| Mid-run clobber: commit diverted to `main`/elsewhere (parent ≠ B, or empty/orphan fork) | existing funnel import belt: `single_commit` = `S^ == B` (`multi-commit`) / `head-moved` (`worktree.rs:665`) |

So the `not-isolated` belt's job is narrow and precise: close the **primary-tree
fallback** that `wrong-base` misses when B is coincidentally an ancestor. The
`branch-mismatch` belt binds the two footer fields so the `not-isolated`/`wrong-base`
checks (on `--dir`) and the import check (on `S`) are about ONE worker state. The
"fail-closed" claim is for the *belt-set*, not any single belt — and it verifies
properties, not provenance (§5.2 residual).

**Residual (honest, out of scope):** a clobbered worker may still *dirty the
primary worktree*. This cannot enter slice history (the funnel runs in the
coordination tree and gates on its own `head_at_base`/`tree_clean`,
`worktree.rs:721`/`:765`). But the side-effect is **not cosmetic**: a dirtied or
moved primary checkout can disrupt any *other* concurrent human or agent working in
it (the shared-clone blast radius this whole issue lives in). This slice does not
prevent that; true pre-worker isolation is IMP-072.

### 5.2 Interfaces & Contracts

**Pure classifier** — two new facts (`is_isolated`, `head_is_branch_tip`), two new
variants (`NotIsolated`, `BranchMismatch`). `not-isolated` inserted #2 (after head,
before marker/base); `branch-mismatch` last (it binds `--dir` to `S`, the
most-specific cross-field check, and only when `--branch` is supplied):

```rust
pub(crate) enum WorkerVerifyRefusal { NoWorkerHead, NotIsolated, Unstamped, WrongBase, BranchMismatch }
// token(): NotIsolated => "not-isolated", BranchMismatch => "branch-mismatch"

pub(crate) fn classify_worker_verify(
    head_resolved: bool,
    is_isolated: bool,        // NEW — git-dir ≠ git-common-dir
    marker_present: bool,
    base_is_ancestor: bool,
    head_is_branch_tip: bool, // NEW — HEAD(dir) == tip(S); `true` when no --branch given (skip)
) -> Result<WorkerVerify, WorkerVerifyRefusal> {
    if !head_resolved      { return Err(NoWorkerHead); }
    if !is_isolated        { return Err(NotIsolated); }   // fork exists but it's the main tree
    if !marker_present     { return Err(Unstamped); }
    if !base_is_ancestor   { return Err(WrongBase); }
    if !head_is_branch_tip { return Err(BranchMismatch); } // footer dir/branch incoherent
    Ok(WorkerVerify::Ok)
}
```

**Shell** (`run_verify_worker`, now `--base <B> --dir <path> [--branch <S>]`) — reuse
`is_linked_worktree`, short-circuit on head:

```rust
let head_resolved = git::git_opt(dir, &["rev-parse","--verify","HEAD"])?.is_some();
let is_isolated   = head_resolved && is_linked_worktree(dir)?;     // missing dir → NoWorkerHead wins
let marker        = marker_present(dir);
let base_is_ancestor = git::git_status_ok(dir, &["merge-base","--is-ancestor",base,"HEAD"])?;
// Coherence: the worktree's checked-out HEAD must equal the branch the funnel will import as S.
// Binds the two footer fields to ONE worker state (codex pass-2 blocker). Skipped if no --branch.
let head_is_branch_tip = match branch {
    Some(s) => {
        let head = git::git_opt(dir, &["rev-parse","--verify","HEAD"])?;
        let tip  = git::git_opt(dir, &["rev-parse","--verify",&format!("{s}^{{commit}}")])?;
        matches!((head,tip), (Some(h),Some(t)) if h == t)         // either unresolved ⇒ false ⇒ refuse
    }
    None => true,
};
```

**Why this closes the footer-incoherence blocker.** `verify-worker` inspects only
`--dir`; `import` inspects only `S`. Without binding, a stale footer can satisfy
both against *different* trees. With `head_is_branch_tip`, both authoritative belts
are provably about the **same** commit: the worktree at `--dir` has `S` checked out
at its tip. The funnel then imports exactly the verified state.

**Honest residual — properties, not provenance.** The belt-set verifies
*properties* (isolated, stamped, descends from B, single commit S with `S^==B`,
dir↔branch coherent) — not the *provenance* that this worktree is "the Agent we just
spawned." A worktree satisfying every property IS a valid worker result by
construction, so accepting it is correct even if its origin is unexpected. The only
unguarded case is a harness that returns a *coherent footer naming a wholly
unrelated yet fully-valid worktree* — indistinguishable from a correct result and
therefore harmless to import. Provenance binding (e.g. a per-spawn nonce in the
marker) is explicitly out of scope (→ IMP-072 territory).

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

**(b) Post-spawn cadence** (replaces the current Post-spawn section). This is a
claude-arm **pre-funnel gate**: the `worktreePath:`/`worktreeBranch:` footer is the
source of both `--dir` (for `verify-worker`) and `S` (the worker fork branch the
funnel's delta check `B..S` already requires, `dispatch/SKILL.md:39`). No footer ⇒
neither is locatable ⇒ the batch cannot be funnelled:

```
## Post-spawn (pre-funnel gate, claude arm)
1. Read the Agent return footer for `worktreePath:` / `worktreeBranch:`.
   NO footer ⇒ no isolated tree was created (fallback-to-main under lock contention) ⇒
   ABORT, do NOT enter the funnel. Re-dispatch, or switch to the subprocess arm if main churns.
2. doctrine worktree verify-worker --base <B> --dir <worktreePath> --branch <worktreeBranch>
   Abort on any refusal: no-worker-head / not-isolated / unstamped / wrong-base / branch-mismatch.
   (`--branch` binds dir↔branch — both belts then verify ONE worker state.)
3. Hand <worktreeBranch:> to the funnel as S.
```

**(c) Red Flags** — add: never funnel a worker that returned no `worktreePath:`
footer; always embed the base-guard block in the distilled prompt.

**`dispatch/SKILL.md` (router funnel) is untouched, and that is correct.** The
footer parse is a claude `Agent` artifact (codex review Major-4): it is an arm-level
*pre-funnel* gate that produces the `S`/`--dir` inputs the funnel already consumes.
The funnel's own ordered belts (import `S^==B`, head-moved, R-5) are unchanged and
remain harness-identical — this slice does not alter the funnel contract, it feeds
it. (Slice scope §affected-surface reconciled to match: no router edit.)

### 5.5 Invariants, Assumptions & Edge Cases

- **Ordering invariant:** `head_resolved` is checked before `is_isolated`, so the
  short-circuit (`is_isolated=false` when head unresolved) never misreports — a
  missing/nonexistent `--dir` always surfaces as `no-worker-head`.
- **Edge — primary tree that is stamped and ancestor:** previously passed
  (`Ok`); now refused `not-isolated`. This is the G1 fix and the only intentional
  verdict change. The behaviour-preservation gate covers all *isolated* cases.
- **Edge — genuine git spawn failure** (broken env): propagates loud via `?`, not
  masked as `not-isolated`.
- **Mid-run clobber (corrected per codex review):** `verify-worker --dir <X>`
  samples whatever path the orchestrator passes (the footer's `worktreePath`), NOT
  an authoritative live cwd — so if the footer still names the original (now
  orphaned) fork, `verify-worker` can pass. The belt that actually contains this is
  the **funnel import belt**, not `verify-worker`: a commit diverted to `main`
  (parent ≠ B) or an empty fork fails `classify_import`'s `S^ == B` single-commit
  check (`worktree.rs:665`, token `multi-commit`) / `head-moved`. The bad work
  cannot enter slice history. Exact booleans (codex pass-2): `classify_import`
  refuses unless `head_at_base ∧ tree_clean ∧ single_commit` all hold
  (`worktree.rs:665`); `single_commit` = `<fork>^^{commit} == B` (`worktree.rs:733`).
  An orphan fork still at B (`S == B`) has no parent equal to B ⇒ `single_commit`
  false ⇒ `multi-commit` refusal (`worktree.rs:737`) — confirmed. The in-worker
  guard (belt 1) only samples the initial tree and does NOT close this — claimed
  accordingly, not overclaimed.
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
  leaves the primary-tree G1 uncaught by code). Rejected C (worker-run `assert-base`
  CLI) on **complexity/redundancy** grounds: C *cannot be the authoritative belt*
  (the worker is untrusted; the authoritative gate is orchestrator-side per ADR-006)
  and is subsumed by `verify-worker` + the funnel import belt. C is not valueless —
  a worker-side check on the live cwd could catch context drift earlier than prose
  (codex review minor) — but it buys fail-fast/UX, not new correctness, at the cost
  of a new read-classed verb + worker-mode plumbing + goldens. Not worth it for this
  slice. The worker-side guard stays as template prose (belt 1) for fail-fast only.
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
  pinning the *content* the budget must cover. Codex review (minor) flags the deeper
  hazard: a line-count cap on *safety-critical* prose incentivises compression and
  ambiguity. Mitigation: bump generously (don't shave the base-guard to fit) and
  treat the presence asserts — not the count — as the real gate.
- **R4 — `not-isolated` narrows, does not alone close.** It catches only the
  primary-tree fallback; the belt-set (marker/base/import) closes the rest (§5.1).
  Mitigated by NOT claiming single-belt closure and by leaning on the existing,
  harness-identical funnel import belt for the residual manifestations.
- **R5 — contract change to `verify-worker`.** A stamped, ancestor primary tree now
  returns `not-isolated` instead of `Ok`. No current production caller runs
  `verify-worker` against a primary tree on purpose (codex review confirmed: only
  skill prose + tests call it), but the verb's contract changes — documented here
  and in the verb's doc-comment at execute.

## 9. Quality Engineering & Validation

- **VT** — pure goldens: `not_isolated_refuses_after_head_before_marker`;
  `branch_mismatch_refuses_last`; the `head_is_branch_tip=true`-when-no-branch
  skip; existing `classify_worker_verify_*` goldens take the two new args, verdicts
  unchanged (behaviour-preservation).
- **VT** — `run_verify_worker` integration: refuses `not-isolated` against a primary
  tree; refuses `branch-mismatch` when `--branch` names a branch whose tip ≠ the
  worktree HEAD; `Ok` against a linked worktree on B with a coherent `--branch`
  (existing worktree test harness).
- **VT** — `e2e_skills_dispatch_shrinkage.rs::dispatch_agent_skill_is_shrunk`:
  raise cap; add `base-guard` / `not-isolated` / `worktreePath` presence asserts.
  Router test untouched.
- **VA** — `dispatch-agent` prose: base-guard block carries the 4 checks; Post-spawn
  carries footer-abort + the four refusal tokens.
- **Closure** — ISS-034 Defect A reconciled (resolved); IMP-052 overlap noted (the
  footer-abort cadence partly delivers its "abort an un-isolated/unstamped worker"
  intent — do not auto-close).

## 10. Review Notes

### Adversarial pass — codex mcp (GPT-5.x), 2026-06-20

Findings and dispositions (all integrated):

- **[blocker] footer belt is theatre / no `worktreePath:` consumer.** PARTLY
  ACCEPTED, REFRAMED. The footer IS consumed — it supplies `S` (the worker fork
  branch) that the funnel's existing delta check `B..S` requires
  (`dispatch/SKILL.md:39`) and `--dir` for `verify-worker`. The current skill prose
  hand-waved this; §5.4b now makes the footer→`S`/`--dir` linkage explicit and the
  belt a defined pre-funnel gate. The "three independent belts, each sufficient"
  language was wrong → replaced with the layered scenario→belt map (§5.1).
- **[blocker] mid-run clobber not closed; "empty delta" unsupported.** ACCEPTED,
  CORRECTED. `verify-worker` samples the passed `--dir`, not a live cwd, and can
  pass on an orphaned fork. The real containment is the funnel import belt
  (`classify_import` `S^==B`, `worktree.rs:665`), not `verify-worker`. §5.5 + §5.1
  rewritten to claim only what that belt delivers; residual main-tree side-effect
  named (out of scope → IMP-072).
- **[major] `is_linked_worktree` is primary-vs-linked, not "the main tree exactly";
  fallback into another linked tree (incl. coordination) passes `not-isolated`.**
  ACCEPTED. §2 wording corrected; §5.1 map shows coordination-tree (markerless →
  `unstamped`) and other-fork (`wrong-base`) are caught by sibling belts. The
  "fail-closed" claim is now scoped to the belt-set.
- **[major] scope vs design contradiction on footer-belt location.** ACCEPTED.
  Design is correct (router untouched; arm-level pre-funnel gate); slice scope
  §affected-surface reconciled to drop `dispatch/SKILL.md` (§5.4b note).
- **[minor] option-C trust-boundary rejection overclaims.** ACCEPTED. D1 softened:
  rejected on complexity/redundancy, not "zero value."
- **[minor] budget test is presence+count theatre; line pressure on safety prose.**
  ACCEPTED. R3 updated; presence asserts (not the count) are the gate; bump
  generously.
- **[confirmed sound]** short-circuit ordering + error propagation; no legitimate
  false-positive (`git-dir==git-common-dir` only on the primary tree); no live
  caller broken by the `Ok→not-isolated` change (still a documented contract
  change, R5).

Net: codex's "narrows, does not close" verdict was right about `not-isolated` in
isolation. Resolution is to lean on the **existing harness-identical funnel import
belt** for the residual manifestations rather than to widen `not-isolated` — the
design now states this explicitly rather than overclaiming a single-belt fix.

### Adversarial pass 2 — codex mcp, 2026-06-20

- **[blocker] `worktreePath`/`worktreeBranch` never cross-checked** — a stale footer
  could point `verify-worker` (`--dir`) and `import` (`S`) at different valid-looking
  trees, both pass. ACCEPTED + CLOSED IN CODE: added `--branch` + `branch-mismatch`
  belt to `verify-worker` binding HEAD(dir) == tip(S) (§5.2, §5.1 table). This was a
  genuine miss in pass-1's reframe.
- **[major] slice scope still said "two independent belts each sufficient"** —
  contradicted the design's layered framing. ACCEPTED: `slice-123.md` objective
  rewritten to the layered belt-set (no "each sufficient").
- **[major] scenario table hole — stale linked worktree stamped+clean+on-B+`S^==B`
  passes everything.** ACCEPTED: this is the identity-under-specification root the
  `branch-mismatch` belt now addresses; the irreducible remainder (a *coherent*
  footer naming a wholly-valid unrelated tree) is named as the
  properties-not-provenance residual (§5.2) — harmless because such a tree is a
  valid result by construction; provenance binding deferred to IMP-072.
- **[minor] import-belt prose imprecise** — tightened §5.5 to the exact
  `head_at_base ∧ tree_clean ∧ single_commit` booleans (`worktree.rs:665/733/737`).
  Codex confirmed the orphan-fork (`S==B`) case refuses `multi-commit`.
- **[minor] "dirty main" residual under-honest** — §5.1 residual now names the
  concurrent-checkout blast radius, not "harmless leftover."

Pass-2 verdict was "not ready to lock; fix the coherence check + the scope
objective." Both done (coherence belt in §5.2; scope objective in `slice-123.md`).
