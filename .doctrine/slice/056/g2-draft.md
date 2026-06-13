# SL-056 PHASE-03 (G2) — pivot delta + ADR amendment drafts (B)

> **Status: DRAFT for adversarial review (C) → owner LOCK (D).** Nothing here is
> applied yet. The accepted ADR-006 and the proposed ADR-011 are **untouched on
> disk** until D folds these in and runs `doctrine adr status`. This is the single
> review surface for the pivot.

## 0. The pivot in one line

The O3 spike (PHASE-02) came back **RED** for the WorktreeCreate fail-closed path
and the PHASE-03-A probes settled the SubagentStart altitude. **SubagentStart-stamp
becomes the PRIMARY claude worker-identity mechanism; the WorktreeCreate
`create-fork` verb is deferred.** This *selects a row the locked design already
authored* (ADR-011 D6 was explicitly two-valued; design §4b named SubagentStart-stamp
as the fallback ladder rung-1) — it is not a design change, it is the spike resolving
the φ cell.

## 1. The empirical basis (PHASE-02 + PHASE-03-A, all confirmed)

| Fact | Source | Consequence |
|---|---|---|
| WorktreeCreate payload has **no `agent_type`**, no usable target/base | PHASE-02 empirics + official docs (`hooks.md`) | `create-fork` cannot gate-on-type, cannot target, cannot base-pin ⇒ **not buildable** |
| WorktreeCreate **is** fail-closed (non-zero exit fails creation) | docs + PHASE-02 | the *only* fail-closed creation hook — but unusable without type/path |
| SubagentStart carries `agent_type` + `cwd`=worktree; **matcher on agent_type is doc-supported** | PHASE-02 + docs | stamp can be scoped to `dispatch-worker` cleanly |
| SubagentStart command hook is **sync-blocking** — gates worker execution until it exits | PHASE-03-A (sleep 3s/10s, 1:1 scaling) | stamp is **guaranteed present before the worker's first command — on hook success** |
| SubagentStart is a **read-only event**: exit 2 non-blocking, cannot abort the subagent | docs + PHASE-03-A (2-step worker completed under exit-2 hook) | **NOT fail-closable**; on stamp failure the worker runs unstamped, un-gateable |

Memory: `mem.pattern.dispatch.subagentstart-blocking-but-not-failclosable`,
`mem.pattern.dispatch.claude-worktreecreate-payload-minimal-no-type-no-path`,
`mem.pattern.dispatch.claude-subagentstart-worker-identity`.

---

## 2. ADR-006 G2 amendment — DRAFT prose (to fold at D)

ADR-006 is `accepted`; these are the minimal edits G2 (§10 design) specified, with
the claude clause **inverted by the spike**.

### 2a. D9 (creation ladder) — claude rung

> **Amend D9.** For **claude**, the creation/identity path is **SubagentStart-stamp,
> not a creation-replacing hook**: Claude performs its own default `git worktree add`
> at `Agent isolation:worktree` spawn; a **matcher-scoped `SubagentStart` hook**
> (`matcher: dispatch-worker`) then **provisions** the fork (D9 `.worktreeinclude`
> allowlist, withheld tier excluded) and **stamps the marker** into the payload `cwd`.
> Because SubagentStart is **sync-blocking**, provision+stamp complete *before* the
> worker's first command (stamp-before-worker on hook success). The WorktreeCreate
> `create-fork` create+provision+stamp-in-one-act path of the original draft is
> **deferred**: the deployed WorktreeCreate payload carries no `agent_type`,
> `worktree_path`, or base, so it can neither classify, target, nor base-pin a worker.
> It returns only if the harness payload grows those fields (or an IDE-004 channel
> lands). codex/pi keep `fork --base <B> --worker` (create+provision+mark, base-pinned)
> unchanged.

### 2b. D2a (signal) — disk-marker-primary + Orchestrator class — *unchanged by the pivot*

> **Amend D2a** exactly as G2 originally specified — this clause is **independent of
> which hook stamps the marker**: replace the `DOCTRINE_WORKER=1` self-arm with the
> **disk-marker-primary** signal (`worker_mode = (is_linked_worktree && marker_present)
> OR env DOCTRINE_WORKER`); env is a codex/pi optimisation, not the identity. Add the
> **`Orchestrator`** verb class (`fork`/`import`/`land`/`gc` refused under `worker_mode`).

### 2c. Altitude (the honest claude row)

> **claude enforcement altitude (firm):** accident-fenced + prompt-enforced,
> **fail-OPEN on stamp failure — not fail-closed.** The stamp-before-worker guarantee
> holds only when the SubagentStart hook succeeds; SubagentStart is a read-only event
> (no exit-code can abort the subagent), so an unstamped worker cannot be stopped *by
> the hook*. The fence against an unstamped/non-compliant worker is the **`import`
> `.doctrine/`+`.claude/` rejection belt + the `DOCTRINE_WORKER` worker-mode guard +
> the pre-distilled prompt**, never a hook exit status. Base-pinning is a **confessed
> residual, sharper than charge-2 assumed** (§3 below).

---

## 3. ADR-011 firming — DRAFT (φ to O3-red; σ moot; D5 sharper)

ADR-011 is `proposed`; PHASE-03 firms it. `doctrine adr status … accepted` runs
**only at D** after lock.

### 3a. D6 (φ) — FIRM to the O3-red row, sharpened

Replace the two-valued D6 with the resolved row:

> **D6 — claude fail-closed altitude (RESOLVED by PHASE-02/03 spike → O3-red).** The
> WorktreeCreate fail-closed arm is **not achievable on the deployed harness** (payload
> lacks `agent_type`/`worktree_path`/base). The achievable altitude is
> **SubagentStart-stamp**: a matcher-scoped, **sync-blocking** hook provisions+stamps
> the marker into `cwd` **before the worker's first command on hook success**. This is
> **stronger than the original O3-red framing on the race axis** — there is **no
> worker-execution race**, the marker is mechanically present before any worker write.
> But SubagentStart is a **read-only event**: no exit status (incl. `exit 2`) can abort
> the subagent, so the residual fail-open is a **hook-failure case** (stamp errors), not
> a timing race, and it is **un-gateable by the hook**. ⇒ claude worker-sole-writer is
> **accident-fenced + prompt-enforced, not malice-proof and not fail-closed**; malice
> containment is the `import` belt (SL-056 §7), not the marker. The WorktreeCreate
> fail-closed path is **deferred**, recoverable only if the payload grows type+path
> (or IDE-004). *(Cell promoted from `proposed` to asserted at lock.)*

### 3b. D7 (σ) — MOOT / resolved-by-mechanism

Replace the σ blast-radius confession with a resolution:

> **D7 — σ blast-radius (RESOLVED, no longer a residual).** σ was a **WorktreeCreate
> artifact**: that hook *replaced default creation for every worktree*, forcing a
> shared create+provision core and a benign else-branch (the blast radius). The
> SubagentStart-stamp path **does not intermediate creation at all** — Claude creates
> every worktree by default; the hook only **fires per-subagent and stamps**, and its
> **matcher scopes the stamp to `dispatch-worker`**. So there is **no else-branch and
> no shared creation core**: benign isolated subagents are untouched (Claude default
> creation, no doctrine provisioning, no marker). **This is a simplification, not a
> hidden loss** — the "benign subagents gain `.worktreeinclude` provisioning" upside was
> a *side-effect* of creation-replacement, never a requirement; doctrine never owed
> provisioning to non-dispatch subagents. The σ confession, the matcher question, and
> the delete-vs-keep-else-branch disposition all **dissolve**.

### 3c. D5 / D3-table (charge-2) — base-pinning residual STANDS, sharper

> **D5 — claude base-pinning (sharper).** Under SubagentStart-stamp, **Claude performs
> the worktree creation and chooses the base** — the payload exposes no base/parent
> field, and the PHASE-02 spike observed the default-created worktree HEAD was **not**
> the orchestrator's HEAD. So the base is **opaque and not orchestrator-controlled** —
> *worse* than the original "forks from session HEAD" framing, which at least assumed
> the orchestrator's HEAD. The mitigation is unchanged and still sound: a wrong base is
> **caught late** by `import`'s `head-moved` refusal → the worker re-dispatches onto the
> bumped base (cost: **a wasted worker run, not a base-integrity breach**). No hook-time
> `base-moved` pre-refusal is possible (the hook sees no base); deferred IMP-043.

**D3 altitude-table edits (claude column):** Marker writer → *matcher-scoped
SubagentStart `stamp-subagent` (Claude does default creation; hook provisions+stamps)*;
Base pinning → *opaque, not orchestrator-controlled (D5)*; Fail-closed altitude → *O3-red:
SubagentStart-stamp, blocking, not fail-closable (D6)*.

---

## 4. PHASE-10 re-scope (code, not done in B/C/D — recorded for the drive)

- `run_create_fork` (WorktreeCreate handler) — **deferred / dropped from v1.** Not
  buildable on the deployed payload.
- `run_stamp_subagent` (SubagentStart handler) — **promoted to the primary** claude
  verb, and **thinner** than `create-fork`: it does **not** run `git worktree add`
  (Claude already created the worktree) — it validates the payload `cwd` is our
  worker's linked worktree, **provisions** (D9), and **writes the marker**. No
  compensating-rollback of a `git worktree add` it never ran (ρ shrinks); no
  else-branch (σ moot).
- `classify_create` three-valued (`ForkWorker|PlainCreate|Refuse`) → **collapses** to a
  simpler stamp classifier (no `PlainCreate` else-branch). ψ bad-payload refusals shrink
  to: missing/empty `cwd`, `cwd` not under repo / not a linked worktree, missing
  `agent_type` — still fail-closed *on the stamp decision*, but the worker runs
  regardless (read-only event), so a refusal here means **unstamped worker**, fenced by
  the belt+guard+prompt, not a blocked spawn.
- `src/boot.rs`: the **WorktreeCreate `HookSpec` → a matcher-scoped `SubagentStart`
  `HookSpec`** (`matcher: dispatch-worker`), reusing the merge core; wired by `claude
  install`.
- §5 privilege classes: `create-fork` Hook-mint entry → `stamp-subagent` (still
  Hook-mint, refused under `worker_mode`, legit hook exempt at the coordination root).
  Unchanged in spirit.
- Drift test (τ) still applies: `DISPATCH_WORKER_AGENT_TYPE` const ↔ the `Agent`
  `subagent_type` ↔ the SubagentStart **matcher** ↔ `install/agents/claude/
  dispatch-worker.md` name — now the matcher is the gate, not a WorktreeCreate body
  branch.

---

## 5. Open items for the C review (hunt these)

1. **Provision-at-SubagentStart timing / the lost baseline-verify guarantee.** ADR-006
   D9 promises the orchestrator **baseline-verifies the fork builds+tests green before
   dispatch**. Under SubagentStart-stamp, **Claude creates the worktree at spawn — the
   orchestrator has no pre-spawn moment to baseline-verify.** Provision now runs *inside*
   the blocking SubagentStart hook (cheap `.worktreeinclude` copy is fine; a `cargo
   build` baseline would block the worker for the whole build, sharing the jail target
   per ADR-008 D-B1). **Is the D9 baseline-verify guarantee weakened for claude, and is
   that acceptable / where does it move?** (Candidate: drop baseline-verify to a
   best-effort in-hook step, or accept first-worker-command-fails-fast.) **Load-bearing —
   review must rule.**
2. **Sharper opaque-base (charge-2/D5).** Confirm the bounded-cost framing holds when the
   base is fully Claude-chosen (not even session-HEAD): is "wasted worker run" still the
   worst case, or can an opaque base land a worker on a tree that *passes* `import`
   head-check while being semantically wrong? Probe the `import` oracle against an
   opaque base.
3. **What WorktreeCreate-primary silently covered, now dropped.** The fail-closed
   capability (no worktree without a marker). Confirm nothing else in the design leaned
   on creation-replacement (e.g. forced provisioning of every fork, the `--worktree`
   launch path).
4. **The deferred `create-fork` gap.** Is "deferred until the payload grows type+path /
   IDE-004" a clean defer, or does any v1 obligation secretly require it?
5. **σ-moot soundness.** Verify the claim that benign-subagent provisioning was never a
   requirement — grep the design/skills for any consumer that assumed every
   isolation:worktree subagent is doctrine-provisioned.
