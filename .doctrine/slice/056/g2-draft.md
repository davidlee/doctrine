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

---

## 6. C review — findings + D disposition

Two adversarial passes: **codex (GPT-5.5)** primary + an independent **Opus**
verify-and-extend pass. Net: **3 blockers, 4 majors, 3 minors.** Several reshape the
ADR drafts; **B1 is a genuine owner decision** (claude as a weaker enforcement class).
Disposition tags: **FOLD** = fold the fix into the §2/§3 drafts at lock; **OWNER** =
needs a VH-1 ruling before lock; **CORRECT** = fix a misstatement; **DESIGN-BODY** =
also edit design §4b/§5/§11/§12, not only the ADRs.

### Blockers

- **B1 — Lost pre-dispatch baseline-verify (OWNER + DESIGN-BODY).** ADR-006 D9
  (adr-006.md:128-130,184-185) promises the orchestrator baseline-verifies the fork
  builds+tests green **before** dispatch; `/worktree` (SKILL.md:196-205) and `/dispatch`
  (SKILL.md:115) operationalise it as a hard pre-handoff gate. Under SubagentStart-stamp,
  Claude creates the worktree **at spawn** — there is no pre-spawn orchestrator moment,
  and a failing hook still yields a running worker (read-only event). The guarantee
  **moves from pre-dispatch to post-import verify** (the funnel's `import → verify` still
  catches an unbuildable fork, but only after a wasted worker run). My §2a draft
  **silently dropped** the baseline clause rather than reconciling it. **Owner must rule:
  is claude still inside D9, or is it now a weaker class where "worker may start on an
  unverified fork" is accepted doctrine?** Whichever, D9's baseline sentence must be
  amended explicitly, not left stale. **Compounds with §7c (Opus M4):** moving
  baseline-verify post-import multiplies its cost by batch width — pre-dispatch verify
  ran *once* before fan-out; post-import a broken first worker wastes the whole
  one-landing-per-base round (υ) before the build failure surfaces.

- **B2 — Stamp-failure privilege fail-open (FOLD — fix is sound).** On stamp/hook
  failure the worker runs with marker absent + no env leg ⇒ `worker_mode == false`
  (ADR-011 D1:46-49; §11:662-663) ⇒ the **Orchestrator/Hook-mint verbs are NOT refused**
  (§5:329-330): the unstamped worker can `fork`/`import`/`land`/`gc`, mutate refs, or
  write `.doctrine/` directly **before any import** — the belt only filters the
  funnel-back, not what the live process already did. This is a **NEW hole** the
  fail-closed WorktreeCreate path closed by construction (no stamp ⇒ no worktree ⇒ no
  worker). My draft's stated fence ("belt + worker-mode guard + prompt") is
  **internally inconsistent** here — the guard is exactly what mis-fires. **Fix (both
  reviewers + verified sound):** in a linked worktree, treat **`is_linked_worktree &&
  !marker_present` as fail-CLOSED** — refuse Orchestrator/Hook-mint/write rather than
  trust-as-orchestrator. The legit orchestrator runs at the coordination root
  (`!is_linked_worktree`), so it is unaffected; and this **also closes the deliberate
  marker self-clear** (clearing now *refuses* the privileged verbs instead of enabling
  them). This is the sharper form of ADR-011 D1's "worker-on-main catch," applied to
  worker-in-worktree. **Adopt and add to ADR-006 D2a + §5.**

- **B3 — The legit-hook privilege exemption BREAKS under SubagentStart (FOLD +
  DESIGN-BODY; reconcile with B2).** The §5 privilege model exempts the stamp verb
  "because it runs at the coordination root where `worker_mode` is false" (§5:330,
  §4b:306-308). True for WorktreeCreate (orchestrator session). But **SubagentStart's
  payload `cwd` IS the worker's linked worktree** (notes.md:67-72) — so `run_stamp_subagent`
  runs with `is_linked_worktree == true`, and stamping flips `marker_present` true
  mid-run. The location-based carve-out **does not hold**; worse, B2's fix would make a
  linked-worktree-without-marker context refuse Hook-mint — i.e. **refuse the stamp hook
  itself on its first run.** **Fix:** the stamp verb must be exempt **by verb identity
  (it is the marker-minter), not by location** — an explicit hook-context bypass.
  Reconcile B2 + B3: `run_stamp_subagent` is the one Hook-mint verb allowed in a
  marker-absent linked worktree; everything else is refused there.

### Majors

- **M1 — Base residual is sharper than "wasted run" (CORRECT §3c/D5).** Mechanism
  mislabel: a wrong Claude-chosen base means the fork parent `S^ != B`, caught by
  `import`'s **`multi-commit`** refusal (§7a:402-403), **not `head-moved`** (which is
  `HEAD != B`, an orchestrator-side condition). All three docs (ADR-011 D5:120-121,
  g2-draft §3c, design §4b:297-299) conflate them — fix the citation. **And a real
  silent-pass gap:** if the worker presents `S^ == B` (parent shown as B) while its tree
  content was developed against an opaque `B'`, then `multi-commit` passes, the belt
  passes, and `git apply --3way` can apply **cleanly but semantically wrongly** (textual
  non-overlap ≠ semantic correctness). So "wasted run, **not** a base-integrity breach"
  is **not guaranteed** — worst case is a clean-applying semantically-wrong import.
  Strengthen the charge-2 confession accordingly; this is a residual, not a closed case.

- **M2 — Matcher gate is doc-supported, NOT empirically confirmed (CORRECT §1 banner).**
  The literal `dispatch-worker` matcher path was **UNTESTED** (notes.md:79-83; PHASE-03-A
  used a `*` matcher + `general-purpose` worker). Only `agent_type` existence + docs-allow-
  matching are confirmed. My §1 "all confirmed" banner overclaims that row. **Worse
  post-pivot:** a matcher↔const drift means the hook **does not fire at all** ⇒ unstamped
  free-writer (= B2). So the **entire pivot rests on a matcher never end-to-end tested.**
  Downgrade to "doc-supported; literal path untested — fresh-session probe owed (IMP)";
  keep τ as an explicit open risk, and the τ drift-test must now pin the matcher string.

- **M3 — ρ rollback is relocated to an un-abortable context, not eliminated (FOLD §4).**
  My draft said "no `git worktree add` ⇒ ρ shrinks." Half-true: **provision now runs
  inside the read-only stamp hook**, so a mid-copy provision failure leaves a
  **half-provisioned worktree AND a running worker** (hook can't abort; Claude owns the
  worktree, no rollback owner). The WorktreeCreate ρ existed precisely to avoid
  half-provisioned forks; the pivot moves that risk somewhere rollback is **impossible**.
  State it: ρ doesn't shrink, it becomes un-rollback-able + un-abortable.

- **M4 — Deferring `create-fork` strands design-body obligations (DESIGN-BODY).** design
  §11:662 / §4b:218-237 / §12 still normatively specify `run_create_fork` with full
  ρ/ψ/τ apparatus, the `write_marker` second caller, the "two callers, one rollback core"
  claim, and the else-branch goldens. Deferring the verb makes the **design body
  internally contradictory.** Lock must edit design §4b/§11/§12, not only the ADRs — my §4
  PHASE-10 prose under-flagged this.

### Minors

- **m1 — σ-moot is actually SOUND (REFINE §3b, downgrade codex's charge).** Opus searched:
  **no skill/verb depends** on a benign isolation:worktree subagent being provisioned —
  `/worktree` provisions its own forks, `/dispatch` forbids `Agent isolation:worktree`.
  So "no functional hidden loss" holds. The only residual is **documentary**: mark
  ADR-011 D7's "net upgrade" framing **superseded/withdrawn** at lock (don't silently
  dissolve it), so a future reader doesn't expect benign-subagent provisioning.

- **m2 — D2a is NOT "unchanged by the pivot" in meaning (FOLD §2b).** The text is the
  same, but the `worker_mode` formula's **safety changed**: pre-pivot `(is_linked &&
  !marker)` could never describe a live worker (no marker ⇒ no worktree); post-pivot it
  can. Re-litigate D2a alongside B2's fix; drop the "independent of which hook stamps"
  framing.

- **m3 — cited memory key (CHECK — resolved).** `mem.pattern.dispatch.subagentstart-
  blocking-but-not-failclosable` is **recorded + verified this session**
  (mem_019ec0a5…); it is absent from the boot snapshot only because the snapshot predates
  the recording. No action.

### LOCK agenda (what D needs from the owner — VH-1)

1. **B1 — the class question (load-bearing):** is claude still inside ADR-006 D9, or an
   explicitly weaker class ("worker may start on an unverified fork; build failure caught
   late at import-verify")? This is the headline decision; everything else folds cleanly.
2. **B2+B3 fix adoption:** approve `is_linked_worktree && !marker_present → fail-closed`
   (refuse Orchestrator/Hook-mint/write) **with `run_stamp_subagent` exempt by verb
   identity**? (Recommended — closes the fail-open *and* the self-clear, with no cost to
   the legit orchestrator.)
3. **M1 — accept the sharpened base residual** (clean-applying-semantically-wrong import
   is possible, not just a wasted run), or require a hook-time/import-time base assertion
   sooner than the deferred IMP-043?
4. On approval, D folds FOLD/CORRECT items into ADR-006 §2 + ADR-011 §3, edits the
   **design body** (§4b/§5/§11/§12) per B3/M4, runs `doctrine adr status` to promote
   ADR-011, and opens IMPs for the fresh-session matcher probe (M2) and the base-assertion
   (M1).
