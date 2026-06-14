# SL-064 Design — Coordination-branch isolation: dedicated worktree + integration-sync seam

> Status: **in-progress, multi-pass.** The structural spine (§1–§3) is locked.
> The integration-sync *routing policy* (§4) is deliberately OQ-bearing — to be
> settled in later passes, possibly by a fresh agent. Do not advance to `/plan`
> until OQ-A/B/C are resolved or consciously accepted as plan-time assumptions.

## Design principle: framework vs repo-local (the layer wall)

SL-064 amends **ADR-006 (framework)** only. `rtk`, `cargo`, `CARGO_TARGET_DIR`
are **doctrine-the-repo's** local ops — homed in **ADR-008**, never framework
primitives ([[mem.pattern.design.product-not-compromised-by-project-local-ops]]).
The framework names **roles** (coordination branch, trunk ref, verify command,
regenerate axis); the project binds **instances** (`cargo build`, a per-worktree
`CARGO_TARGET_DIR`). Every section below keeps that wall: repo-local concerns
appear only as named instances of an axis, never as a thing the framework decides.

## §1 — Foundation: branch topology & delta-class routing (LOCKED spine)

**Problem.** Today the coordination branch *is* the session working tree
(`main`): the funnel commits land there directly; "conclude" only flips status +
GCs debris. No integration-sync, because ADR-006 D8 assumes coordination-branch ≡
integration-target. That breaks under (a) foreign writers on `main`, (b)
concurrent dispatches contending with each other, (c) any team/PR posture where
unreviewed code must not touch `main` (the SL-060 retrospective,
[[mem.system.dispatch.orchestrator-on-shared-main-contention-cost]]).

**Two delta classes** — already produced by different actors, not a new
classification to build:

| Class | Producer | Tier | Default destination (directional — see OQ-B) |
|---|---|---|---|
| code / code-coordinated | worker (source-only delta `S`) | `src/`, app trees | `phase/<slice>-NN` → trunk *deliberately* (review/merge) |
| intent / understanding | orchestrator (R-5 sole writer) | `.doctrine/` authored: memory, notes, status, AC evidence, design/plan | trunk *contemporaneously* / for-review (OQ-A) |

The class split rides an **existing seam**: the worker writes source only; the
orchestrator writes all `.doctrine/` knowledge (R-5 forbids workers touching
authored trees). Routing the two to different targets is cheap *because* funnel
step-7 (code import) and step-8 (knowledge record) are already distinct commits.

**Three branch roles:**
- **`dispatch/<slice>`** — coordination branch in its own isolated worktree
  (DD-1, always-on). The funnel's sole write target. **Durable SSoT + crash-
  recovery root.** Slice-scoped and **stable across handover-resume** (a fresh
  orchestrator resumes the same branch — intended, so a per-run discriminator
  would *break* resume). **Concurrent dispatch on *different* slices never shares
  it; concurrent same-slice dispatch is REFUSED at creation** (a creation guard) —
  there is no legitimate two-runs-same-slice case, and the "concurrent runs
  isolated" driver applies across distinct slices only (RV-023 F-3).
- **`phase/<slice>-NN`** — per-worker code unit, **preserved as a deliverable**
  (not import-and-deleted). Individually integrable to trunk, possibly via
  LLM-assisted 3-way merge. (Name avoids "feature" — reserved for the PRD
  Requirements-aggregation sense.)
- **`edge`** — *optional standing aggregate* of all local work. Coordination
  branches integrate here at conclude. Contention only here, only at conclude — a
  controlled sync point off the per-batch hot path. Disabled ⇒ no aggregate, no
  cost.

**Integration-sync = downstream projection from the coordination branch, NOT a
funnel change.** Everything still funnels onto `dispatch/<slice>` in strict D7
order (knowledge-trails-code *on the coordination branch*; recovery rebuilds from
it). Conclude then *projects* outward (§4).

**Invariants:**
- ✅ Untouched: D2 worker-sole-writer, D4 tier-merge-safety, D9 withheld-tier.
- ⚠️ D7 single-branch recovery **preserved only for the funnel ONTO
  `dispatch/<slice>`** — the coordination branch stays the singular SSoT for the
  *inner* loop. The **outward projection step does NOT yet have a recovery
  contract**: a crash mid-sync (intent escaped to trunk, code not yet integrated)
  leaves target state the coordination branch does not record as projected-vs-
  pending. Idempotent replay + a **projection journal** (target-tip preconditions
  + completed steps, recorded on `dispatch/<slice>`) is **required and unproven** —
  it **binds OQ-A**. Until designed, this design makes **no D7-recovery claim
  across projection** (RV-023 F-1).
- ✏️ D7 amended (§5): knowledge trails code *on the coordination branch*;
  integration-sync may project knowledge to trunk ahead of code integration.
- ✏️ D8 amended (§5): dedicated coordination worktree + defined projection.
- ✏️ D1 preserved **via config**: opinionated default topology, every target a
  configurable ref — doctrine does not mandate branch flow.

**DD-1 — always-on (locked).** The coordination worktree is provisioned for
*every* dispatch run, not conditionally. Rationale: (a) the contention surfaces
become unreachable *by construction* rather than by a flag the user must
remember; (b) the opt-out path would **be** the old in-place commit seam — a
parallel implementation of the funnel carrying the exact SL-060 hazards — so a
flag doesn't just add an option, it keeps the hazardous path alive forever; (c)
even a solo user running two concurrent dispatches contends on shared `main`, and
always-on isolates them by construction, killing the pre-emptive "will I run
another?" cognitive load. **Cost:** one cold provision build per run (the verify
build, on a per-worktree target — can't share the session target without
false-green, [[mem.pattern.testing.shared-cargo-target-false-red-in-worktree-audit]]),
fully amortised across the run's batches; only stings a quick quiet solo
small-slice where neither driver applies. Accepted.

**OQ-3 (visibility) dissolved** by the class split: intent reaches trunk
immediately (visible); only unreviewed *code* lags on `phase/*` branches — which
is correct.

## §2 — Coordination worktree: provisioning & lifecycle (LOCKED)

**Provisioning needs no coordination-tier copy.** A worker fork *withholds* the
coordination/runtime tier (a copied phase sheet would be invisibly mutable, D9).
The coordination tree is the **sole writer** of that tier, so it **regenerates**
it instead of copying:
- committed source + authored `.doctrine/` entities → present via checkout.
- runtime phase sheets → regenerated from the **committed `plan.toml`** via
  `slice phases` (the orchestrator writes this tier as it works).
- a build for funnel step-5 verify → the **regenerate axis** (repo-local instance:
  `cargo build`, per-worktree `CARGO_TARGET_DIR` — ADR-008 D-B1).

⇒ The coordination tree runs the **full D9 provision — BOTH axes** (regenerate:
the verify build; **and** the non-doctrine allowlist copy of irreducible
gitignored prerequisites — secrets/local config its verify build may need) **plus
baseline-verify**. The *only* exclusion is the coordination/runtime **tier**,
which is already withheld by the worker allowlist *and* regenerated by the
orchestrator. So existing `doctrine worktree provision` is correct **because the
coordination tree wants exactly the same two axes as a worker, minus a tier it
regenerates** — not because the copy axis is absent (RV-023 F-6).

**The one new mechanism — markerless creation.** `doctrine worktree fork` stamps
the worker disk-marker → the identity guard would *refuse* the orchestrator's
writes (D2a). The coordination tree is the orchestrator: **mode OFF, must write.**
Creation must create + provision but stamp **no worker marker** (marker absent ⇒
D6a "mode, not location, decides" lets it write). Shape: a `--coordination`
creation mode (or distinct verb) — same fork ladder, marker-stamp omitted. This
is the load-bearing new code surface.

**Marker-absence is NOT a positive coordination identity (RV-023 F-2).** ADR-011
D6/M2 confesses an unstamped Claude worker (stamp-hook failure / matcher drift)
runs `marker_present=false` in a linked worktree — *indistinguishable by absence*
from this coordination tree. SL-064 **broadens the blast radius** of that confessed
false-clear: absence no longer merely "looks like solo `/execute`" — it now also
looks like the tree that owns the funnel + the Orchestrator verb class
(`fork`/`import`/`gc`/sync, D2a). This design therefore **does NOT claim to close
that gap by absence**. It explicitly **inherits the ADR-006 D2b residual fence**:
the **R-5 import belt** (rejects any `.doctrine/` path in `B..S`), the **IMP-052
orchestrator post-spawn marker check** (aborts an unstamped fork), the
**env-worker-on-main catch**, and — for the build repo — the **bwrap jail with no
push** (ADR-008). **Verify-item (§6):** that fence covers an unstamped worker
invoking *Orchestrator verbs* from a coordination-shaped tree, not just authored
writes.

**OQ-D (new, deferred) — positive coordination-tree marker vs absence.** The
strictly-safer design stamps a **positive** coordination marker (orchestrator
identity) instead of relying on absence, so the guard distinguishes
legit-coordination-tree from unstamped-worker. But that **touches the
owner-locked D2a positive-signal model** (SL-056 PHASE-05 VH) — not a seam to
redesign inside an inquisition. Carried to a dedicated pass / `/consult`.

**Adversarial catch — D2a's root-assumption goes stale.** Current D2a (positive
signal, SL-056 PHASE-05): `worker_mode = (is_linked_worktree && marker_present) OR
env DOCTRINE_WORKER`, with prose pinning the legit orchestrator at the
coordination **root** (`!is_linked_worktree`). SL-064 **moves the orchestrator
into a linked worktree** (the coordination tree *is* `git worktree add` ⇒ linked).
The positive-signal logic still permits it — `is_linked_worktree=true &&
marker_present=false ⇒ not worker-mode ⇒ writeable` — so permission now rests on
**marker-absence, not on being at the root**. Two consequences:
- **D2a prose amendment (§5):** "orchestrator runs at the coordination root" →
  "orchestrator may run in a linked coordination worktree; its write permission
  rests on marker-absence (positive signal), not on `!is_linked_worktree`."
- **`env DOCTRINE_WORKER` must NOT leak** into the coordination tree's
  environment — the env leg would false-flag worker-mode and refuse the
  orchestrator. Verify-item (§6); a real hazard on codex/pi where the env leg is
  live.

**D3 id-minting holds.** Minting resolves the configured **trunk ref** (a git ref
in the shared common dir, resolvable from any worktree), not coordination HEAD —
works from the coordination tree unchanged. Pin with a verify test (§6).

**Lifecycle / cleanup (resolves IMP-041) — worktree-life < branch-life:**
- create at dispatch start, before batch 1.
- lives the whole run (all batches reuse it).
- conclude: integration-sync projects, then the **worktree directory** is
  removed; the **`dispatch/<slice>` + `phase/<slice>-NN` branches are KEPT** as
  deliverables until integrated. (Today's GC deletes them — the bug the topology
  fixes.)

## §3 — Funnel changes (LOCKED; DD-5)

**The cadence is identical — only *where it runs* changes** (isolated
coordination tree, not session `main`). Contention surfaces collapse as a
consequence, not via new funnel logic.

- **precond (step 1):** coordination tree has no foreign writers ⇒ stays clean ⇒
  **contention #1 (dirty foreign index) unreachable.**
- **delta / R-5 / import / verify (steps 2–5):** unchanged. Worker forks from
  explicit `B` (now a ref on `dispatch/<slice>`); `fork --base B` still pins it.
  The session-HEAD-vs-coordination-HEAD divergence hazard shrinks: the
  orchestrator's cwd *is* the coordination tree, so its HEAD ≡ coordination HEAD ≡
  `B`.
- **branch-point guard (step 6): DEMOTED.** No external movers in the isolated
  tree ⇒ HEAD moves only at the orchestrator's own batch commit ⇒ the guard can't
  trip from a foreign mover within a run. Its real job **relocates to the
  integration-sync point** (trunk may have moved). **IMP-043 re-anchor moves from
  per-batch → sync-time only.**
- **commit / record (steps 7–8):** unchanged, land on `dispatch/<slice>`.

**Inline non-delegable `.doctrine/` writes** now happen in the clean coordination
tree ⇒ **contention #2 (collisions with foreign WIP / swept untracked)
unreachable.**

**Not solved (honest):** contention #3 (rtk masking git exit codes /
stat-proxying diff) is environmental/repo-local — orthogonal to branch placement.
The remaining guards keep the rtk-safe forms (`checkout`-import not `diff|apply`,
`ls-tree` not `cat-file -e`). The structural retirement is §4's *verb* (in-process
git via `src/git.rs` is not rtk-hooked) — not an rtk config change.

**Claude-arm interaction — worker-isolation gap is OUT OF SCOPE, but SL-064
shrinks its blast radius (note, not goal).** The claude `Agent isolation:worktree`
does **not** return a separate fork branch — it **collapses the worker's commit
onto the *parent* branch** ([[mem.pattern.dispatch.claude-agent-worktree-integrates-commit-onto-parent]]).
*Fixing* true worker isolation (a real fork / bwrap confinement, codex/pi model)
is **IMP-004 / IMP-045, not this slice.** But two SL-064 interactions follow and
must be designed-for:
- **Blast radius shrinks.** Pre-SL-064 the parent is shared `main`, so a straying
  claude worker's bad commit lands on shared main (disruptive revert). Post-SL-064
  the parent is the **isolated `dispatch/<slice>`** tree → a straying worker
  corrupts only the **disposable coordination tree, never shared main.** A net
  safety gain even though worker-isolation itself is unfixed.
- **Funnel degrades cleanly on the claude arm:** no fork branch to `import` — the
  worker's delta is *already on* `dispatch/<slice>`, so the R-5 belt + combined
  verify run **post-landing on the coordination branch** (not pre-commit). The
  funnel's correctness *goals* hold; only the import *mechanism* differs by arm.
  The sync verb (§4) and guards must accept a worker delta that arrives
  pre-landed, not only as a fork to import.

## §4 — Integration-sync seam (OQ-BEARING)

**LOCKED architecture (policy-independent):**
- **Projection, not a funnel change.** The sync step reads the completed
  `dispatch/<slice>` and projects outward; coordination branch stays the funnel's
  SSoT. **The projection's own crash-recovery is NOT yet contracted** (journal +
  idempotent replay) — binds OQ-A (RV-023 F-1); do not claim D7 across it.
- **It's a verb, not skill-prose git.** A `doctrine`-internal verb (in-process
  git) sidesteps the rtk hook *and* is golden-pinnable/testable — the SL-056
  "mechanism out of prose" thesis. Provisional surface `doctrine dispatch sync`
  (name TBD at plan). Skills *call* it; they don't replay git.
- **Targets are configurable refs, opinionated defaults (D1):** trunk ref
  (default = configured trunk); aggregate `edge` (optional); code units already
  materialised as `phase/<slice>-NN`. Framework names roles; project binds refs.
- **Never-YOLO-trunk is structural:** no force-push, no auto non-ff merge to
  trunk; conflict/moved-trunk ⇒ report, never auto-resolve (ADR-006). IMP-043
  re-anchor lives here.
- **Code units are deliverables:** `phase/<slice>-NN` integrate to trunk
  individually, possibly LLM-assisted 3-way.

**OQ markers (deferred — the next passes):**
- **OQ-A — intent → trunk: push vs leave-for-review, and *when*** (contemporaneous
  per-batch vs at-conclude). The one autonomous-trunk-write hinge ("to main, *or
  at least* proactively pushed for review").
- **OQ-B — delta-class boundary: what ships together vs projects ahead.** Seed
  principle: the real seam is **temporal, not just code-vs-intent**. Scope +
  design = a **prior, separate review gate** (reviewed and landed *before*
  implementation — "code + a wall of markdown = a shitty PR"). The
  **implementation-time bundle = code + intent-drift that arose *during*
  implementation** (notes, memory, plan adjustments, design amendments) ships
  **together** for the implementation review. §1's clean two-class table is
  therefore *directional*; "intent → trunk contemporaneously" holds only for
  genuinely slice-orthogonal knowledge (e.g. durable cross-cutting memory) — TBD.
- **OQ-C — audit ordering vs sync timing.** `/audit`'s RV verbs **refuse on a
  worktree fork** ([[mem.pattern.dispatch.rv-verbs-refuse-on-worktree-fork]]); the
  coordination tree *is* a worktree and the run's code lives on `dispatch/<slice>`
  there ⇒ audit runs from the parent tree *after* projection (or a parent checkout
  of `dispatch/<slice>`). Couples `/audit` placement to sync timing.

## §5 — Governance split (RV-023 F-5, ruled (a): own ADR)

The topology is framework-significant (ADR-011 precedent) → promoted to **ADR-012
(proposed)**; SL-064 is its implementing slice. **ADR-012 acceptance GATES
SL-064's plan** (cf. SL-056 G1 / ADR-008). The split:

**ADR-006 keeps (placement/identity refinements to the existing worktree posture):**
- **D8 (amend):** coordination branch runs in its own dedicated worktree.
- **D2a (amend prose):** orchestrator may run in a **linked** coordination
  worktree; write-permission rests on **marker-absence** (positive signal), not
  `!is_linked_worktree`; the "orchestrator at the coordination root" assumption is
  retired; `env DOCTRINE_WORKER` must not leak. (Identity *strengthening* — the
  positive coordination marker — is OQ-D, not this amendment.)
- **D9 (addendum):** markerless coordination-tree creation variant.
- **D3 (holds, no amend):** minting resolves the trunk ref from the coordination
  worktree — verify-item.

**ADR-012 owns (the topology + projection + routing):**
- the three branch roles + class-routed projection;
- "globals land on main by convention" → the defined **sync-verb projection
  mechanism** (was a mooted D8 amend; it's topology, so it lives here);
- the **D7 projection-semantics**: knowledge trails code *on the coordination
  branch*; projection may lead — **plus the projection recovery contract** (OQ-A);
- the **D1 tightening — owned honestly**: the framework now opines on integration
  topology (opinionated default, configurable refs). No longer claimed as
  "D1 preserved"; it is a conscious tightening, justified by the team-PR driver.

## §6 — Verification impact

- Black-box golden for the sync verb (once shaped).
- Orchestrator **never writes the session `main` tree mid-run** (contention #1/#2
  unreachable).
- Markerless coordination-tree creation ⇒ orchestrator writes (mode OFF); worker
  fork still marked ⇒ refuses (mode ON). The marker seam carries both.
- **Orchestrator writes from a *linked* coordination worktree** (not the root) —
  the positive-signal guard permits it on marker-absence; and a leaked
  `env DOCTRINE_WORKER` is asserted *not* to false-flag/refuse it.
- **Fence covers Orchestrator-verb impersonation (F-2):** an unstamped worker in a
  coordination-shaped tree invoking `fork`/`import`/`gc`/sync is caught by the
  R-5 belt / IMP-052 post-spawn check / jail — assert it, don't assume it.
- D3 minting resolves the trunk ref from the coordination worktree.
- Conclude removes the worktree dir, **keeps** `dispatch/<slice>` +
  `phase/<slice>-NN`.
- Re-anchor at sync onto a moved trunk (IMP-043 relocated).
- **Behaviour-preservation (the gate):** existing dispatch funnel suites stay
  green unchanged — cadence is untouched.
- OQ-gated tests (A/B/C) deferred to their passes.

## §7 — OQ / risk register

- **OQ-A / OQ-B / OQ-C** — carried (see §4). **OQ-A now also OWNS the projection
  journal + idempotent-replay recovery contract (RV-023 F-1).**
- **OQ-D (new, RV-023 F-2)** — positive coordination-tree marker vs absence
  (touches owner-locked D2a; consult/plan-time, defined §2).
- Topology-as-own-ADR — **RULED (a): ADR-012 created (proposed); its acceptance
  gates SL-064 plan** (§5).
- Cold provision build on a quiet-solo-small-slice — accepted cost (DD-1).
- Concurrent-dispatch contention at `edge` conclude — controlled sync point,
  accepted.
- rtk / cargo / `CARGO_TARGET_DIR` — repo-local (ADR-008), layer-walled out of the
  framework decision.
- Dependency: reuses SL-056 `fork` / `provision`; markerless creation extends
  them. ADR-011 spawn seam untouched (references ADR-006, does not amend).

## Decision log (this pass)

- DD-1 always-on dedicated coordination worktree — **locked.**
- DD-2 integration-sync = downstream projection, a verb, configurable targets,
  never-auto-trunk — **architecture locked; routing policy → OQ-A/B/C.**
- Branch names `dispatch/<slice>` / `phase/<slice>-NN` / `edge` — **locked.**
- DD-3 markerless coordination-tree creation; regenerate-not-copy provisioning —
  **locked.**
- DD-5 branch-point/re-anchor demote to sync point — **locked.**
- DD-6 worktree-life < branch-life (resolves IMP-041) — **locked.**
- Delta-class temporal boundary, intent-routing, audit ordering — **OQ.**
