# SL-064 Design — Coordination-branch isolation: dedicated worktree + integration-sync seam

> Status: **spine + routing policy locked; projection mechanism designed.** The
> structural spine (§1–§3) and the integration-sync routing policy (§4) are settled
> in **ADR-012 (ACCEPTED)**: OQ-A/B/C **resolved**, OQ-D **deferred-with-fence**
> (User ruled (a), now an SL-064 plan-gate). The three post-acceptance projection
> gaps — A journal placement, B `review/<slice>` composition, C harness synthesis —
> are now **mechanized in §4.1–§4.3** (the committed run ledger + the in-process
> tree-filter primitive), within the locked contracts (no ADR change). **Ready for
> `/plan`.**

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

**Four branch/target roles** (ADR-012 Decision 3):
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
  Requirements-aggregation sense.) **On a fork-less arm (claude `Agent`), there is
  no worker fork branch — the deliverable is CUT from `dispatch/<slice>` at sync
  time** (RV-025 M3) so the role is universal across arms.
- **`review/<slice>`** — the **named home for the impl-bundle review unit** (the
  "leave-for-review" target — *not* trunk). Materialised at sync stage-1
  (RV-025 B1).
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
- ⚠️ D7 single-branch recovery **preserved for the funnel ONTO
  `dispatch/<slice>`** — the coordination branch stays the singular SSoT for the
  *inner* loop. The **outward projection step now has a defined recovery
  contract** (RV-023 F-1 closed via OQ-A): a **projection journal committed to
  `dispatch/<slice>`** (target-tip preconditions + completed-step markers, intent
  written before the op) + **idempotent patch-id replay** (`git cherry`), which
  reports — never auto-resolves — on a moved target. See **ADR-012 Decision 4**.
  This restores a D7-analog recovery property *across* projection; the contract is
  locked, its implementation + proof land in this slice's plan/execute.
- ✏️ D7 amended (§5): knowledge trails code *on the coordination branch*;
  projection materialises a reviewable `review/<slice>` ref ahead of code
  integration (it does NOT auto-land intent on trunk — ADR-012 Decision 4).
- ✏️ D8 amended (§5): dedicated coordination worktree + defined projection.
- ✏️ D1 **consciously tightened, not preserved** (RV-023 F-4; ADR-012 Decision 6):
  opinionated default topology, every target a configurable ref — repo-policy
  flexibility is preserved *via config*, but the framework now owns an integration
  topology. (The recanted "D1 preserved via config" framing is retired.)

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

**OQ-3 (visibility) dissolved** by the class split: intent reaches a **visible
review target** (`review/<slice>`) at conclude — never auto-landed on trunk; only
unreviewed *code* lags on `phase/*` branches — which is correct.

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

## §4 — Integration-sync seam (OQs CLOSED — see ADR-012)

> **Status update:** OQ-A/B/C are **resolved** and OQ-D is **deferred-with-fence**
> (ruled (a)); the resolutions are formalised in **ADR-012** (Decisions 2/4/5,
> §Decisions formerly open). The markers below are kept for provenance, each
> annotated with its closure. The architecture was already locked.

**LOCKED architecture (policy-independent):**
- **Two-stage projection, not a funnel change.** The sync verb reads the completed
  `dispatch/<slice>` and projects outward in two stages — **stage-1 prepare-review**
  (materialise `review/<slice>` + `phase/<slice>-NN` + the journal; no trunk write)
  then, after audit, **stage-2 integrate** (optional trunk/`edge`). Coordination
  branch stays the funnel's SSoT. **Crash-recovery is now contracted** (normative
  CAS journal — see OQ-A; closes RV-023 F-1).
- **It's a verb, not skill-prose git.** A `doctrine`-internal verb (in-process
  git) sidesteps the rtk hook *and* is golden-pinnable/testable — the SL-056
  "mechanism out of prose" thesis. Provisional surface `doctrine dispatch sync
  --prepare-review` (name TBD at plan). Skills *call* it; they don't replay git.
- **Targets are configurable refs, opinionated defaults (D1 tightened):**
  `review/<slice>` (intent default — *not* trunk); trunk opt-in ff-only; aggregate
  `edge` (optional); code units as `phase/<slice>-NN`. Framework names roles;
  project binds refs.
- **Never-YOLO-trunk is structural:** intent defaults to `review/<slice>`; trunk is
  opt-in, **ff-only + expected-tip CAS**; no force-push, no auto non-ff;
  conflict/moved-target ⇒ report, never auto-resolve (ADR-006). IMP-043 re-anchor
  lives here.
- **Code units are deliverables:** `phase/<slice>-NN` integrate to trunk
  individually, possibly LLM-assisted 3-way.

**OQ markers (now CLOSED — normative in ADR-012):**
- **OQ-A — RESOLVED (ADR-012 Decision 4).** Two-stage projection; intent →
  `review/<slice>` by default (trunk opt-in ff-only + CAS), **at conclude** (not
  per-batch). Recovery: **journal committed to `dispatch/<slice>` before any ref
  mutation**, per-step `expected_old_oid`/`planned_new_oid` compare-and-swap;
  replay no-ops if target==planned, refuses+reports if diverged (closes RV-023 F-1,
  RV-025 B1/B2).
- **OQ-B — RESOLVED (ADR-012 Decision 2).** Temporal/dependency boundary, **four
  buckets**: prior-governance / impl-bundle / slice-orthogonal / runtime-
  coordination. **Default-hold classifier:** an impl-time write holds with the
  impl bundle unless explicitly marked orthogonal; runtime coordination never
  leaves `dispatch/<slice>` (RV-025 M2). §1's two-class table is *directional*.
- **OQ-C — RESOLVED (ADR-012 Decision 5).** `/audit`'s RV verbs **refuse on a
  worktree fork** ([[mem.pattern.dispatch.rv-verbs-refuse-on-worktree-fork]]) and
  the coordination worktree is removed at conclude ⇒ **conclude → stage-1
  prepare-review → audit from parent/root against the prepared `review/<slice>` +
  `phase/*` refs → reconcile → stage-2 integration.** Audit gates the **actual
  review units before integration**, never from inside the coordination tree;
  failure blocks integration, preserves branches (RV-025 B1).

## §4.1 — Projection mechanism (A/B/C): the run ledger + the tree-filter primitive

> Mechanizes the three gaps left after ADR-012 acceptance (A journal placement, B
> `review/<slice>` composition, C harness synthesis). Mechanism *within* the locked
> ADR-012 contracts — no contract change. All three are projection-side and collapse
> onto **two primitives**: a committed run ledger (A; also the inputs B/C consume)
> and an in-process tree-filter (B/C compose with it).

**The run ledger — committed coordination state on `dispatch/<slice>` (A).**
ADR-012 D4 mandates the CAS journal "committed to `dispatch/<slice>` before any ref
mutation." It lives at a dedicated, **non-gitignored** coordination path that only
ever carries content on `dispatch/<slice>` branches:

```
.doctrine/dispatch/<slice>/
  journal.toml      A — CAS projection rows; written at sync (stage-1/2)
  boundaries.toml   C — per-phase code-commit OIDs; written per phase, funnel-time
  orthogonal.toml   B — entities projected ahead independently; written per projection
```

Three files, not one: they write on different events (boundaries/orthogonal during
the funnel; journal at conclude) and the orthogonal manifest can grow, so separate
files keep each append's whole-file rewrite small and give each a single
responsibility. Co-located under one prefix so B excludes exactly one path.

*Tier carve-out (named honestly).* This is **runtime-coordination tier that is
git-committed** — the storage rule normally makes that tier gitignored/disposable
(`.doctrine/state/`). ADR-012 D4 already requires committing it for crash-durability,
so this is a blessed exception, not a violation of the rule. It is kept *off*
`.doctrine/state/` precisely because that path is gitignored — committing there would
mean a force-add fighting the ignore on every commit. The carve-out is bounded: the
dir exists only on `dispatch/<slice>`, never reaches trunk (B strips it; the branch
never ff's to trunk), and dies with the branch at integration.

*A's recovery primitive needs no invention.* Each `journal.toml` row is
`{ target_ref, expected_old_oid, planned_new_oid, applied_new_oid, status }`
(ADR-012 D4). The compare-and-swap is the native `git update-ref <ref>
<planned_new_oid> <expected_old_oid>` 3-arg form — git refuses the update unless the
ref currently equals `expected_old_oid`. Replay: recompute the planned output from
the recorded source OID; if `target == planned_new_oid` ⇒ verified no-op; if
`target` differs from **both** `expected_old_oid` and `planned_new_oid` ⇒ refuse +
report a moved target (never force, never auto-resolve). The journal commit precedes
the `update-ref`, so a crash between journal-commit and ref-update is recoverable
from the branch. This *implements* the ADR-012 D4 contract (RV-023 F-1); it does not
restate a recovery property the spine had to disclaim.

**The tree-filter primitive — new `src/git.rs` plumbing (B + C).** `src/git.rs`
shells the `git` binary directly via `run_git` + `NORMATIVE_FLAGS` (bypassing the rtk
hook — the §4 "verb, not skill-prose git" thesis), *not* libgit2. The compose step
adds a filtered-tree builder over `read-tree` / `rm --cached` / `write-tree` /
`commit-tree`:

```
filter_tree(source_tree, exclude_globs) -> tree_oid    # read-tree source; rm --cached exclude_globs; write-tree
commit_tree(tree_oid, parent_oid, msg)  -> commit_oid   # commit-tree; no working-tree touch
```

Pure ref/object plumbing — no checkout, no working tree mutation, golden-pinnable.
Both B and C are call sites; the primitive is arm-agnostic. **`filter_tree` reads/
writes a throwaway `GIT_INDEX_FILE`, never the live coordination index** — the
compose step must not disturb the orchestrator's index. This matters because stage-2
runs **after the worktree is removed** (§2 lifecycle; ADR-012 D5 ordering): all of
B/C/CAS are plumbing against refs + objects in the common git dir, reachable from the
parent/root with no checkout — so the primitive being working-tree-free is load-
bearing, not a nicety.

### §4.2 — `review/<slice>` composition (B)

`review/<slice>` is a **filtered projection of the `dispatch/<slice>` tip**. v1 = a
**single squashed commit** (reversible: `review/<slice>` is regenerable from the
SSoT, so a later history-preserving variant is a sync-verb change with no migration):

```
review/<slice> = commit_tree(
  tree   = filter_tree(tip_tree, EXCLUDE),
  parent = trunk_base_B,
  msg    = "review(<slice>): impl bundle")
```

`git diff trunk_base_B...review/<slice>` is then exactly the net impl bundle — one
clean reviewable diff. `EXCLUDE` is **small because most of the runtime tier is never
in the committed tree**: runtime phase sheets are gitignored ⇒ already absent from
`dispatch/<slice>`. The committed tree carries only code + authored `.doctrine/` +
the run ledger, so `EXCLUDE` =
- `.doctrine/dispatch/<slice>/**` — the run-ledger dir (runtime-coordination tier);
- the paths of every entity in `orthogonal.toml` **whose ahead-projection is
  journal-verified** — slice-orthogonal knowledge that already projected ahead
  independently, excluded so the review unit does not re-ship it. **Verified, not
  merely listed:** if an ahead-projection failed/crashed (journal status ≠ verified)
  the entity is *not* excluded and falls back into the review bundle — reviewed once,
  never lost.

Everything else stays = the impl bundle (code + AC evidence + phase notes + design
amendments + impl-caused status), per ADR-012 D2's default-hold classifier. The
classifier *enforcement* (what marks an entity orthogonal) is the OQ-B **plan-gate**;
B's mechanism here only *consumes* its output (`orthogonal.toml`) and excludes those
paths — it does not decide classification.

### §4.3 — Harness synthesis: the per-phase cut (C)

ADR-012 D3: on the fork-less claude arm the worker delta lands directly on
`dispatch/<slice>`, so `phase/<slice>-NN` must be **cut from `dispatch/<slice>` at
sync**. The funnel interleaves `B → [code₁] → [knowledge₁] → [code₂] → …`; the
orchestrator owns steps 7 (code) and 8 (record), so it captures HEAD **between** them
and records the phase's code boundary:

`boundaries.toml` row = `{ phase = "PHASE-NN", code_start_oid, code_end_oid }`
(`code_end_oid` = the worker code commit, *before* the knowledge record commit).

At sync stage-1, in phase order, each phase branch reuses the B primitive:

```
phase/<slice>-NN = commit_tree(
  tree   = filter_tree(tree_of(code_end_oid), keep code paths / strip .doctrine/),
  parent = phase/<slice>-(NN-1),   # trunk_base_B for NN=1
  msg    = "phase(<slice>-NN): <phase code>")
```

`phase-(NN-1)` already holds cumulative code through NN-1, so the diff is precisely
phase NN's code delta. Snapshotting `tree_of(code_end_oid)` (cumulative) rather than
replaying commits makes the cut **agnostic to intra-phase commit count** — multiple
worker batches in one phase collapse correctly; `code_start_oid` is used only for the
empty-phase test. Scope:
- **Claude-arm only.** On codex/pi the worker fork *is* `phase/<slice>-NN` natively
  (ADR-012 D3); the cut is the fork-less synthesis. `boundaries.toml` is populated
  only where the cut consumes it. Stage-2 integrate accepts native and synthesized
  branches uniformly — same ref shape by construction.
- **Empty-code phase** (`code_start_oid == code_end_oid`) ⇒ no branch cut.
- Workers write source-only (R-5), so the code-path filter is a safety belt on the
  claude arm; it keeps C's primitive byte-identical to B's.

**Ordering / composition with §4's two-stage projection.** B and C run at **stage-1
prepare-review** (ADR-012 D4/D5): materialise `review/<slice>` (B) +
`phase/<slice>-NN` (C) and write `journal.toml` rows under CAS — **no trunk write**.
These are ref *creations*, so the CAS uses `old = zero-oid` (`update-ref` refuses if
the ref already exists) — a crashed prior run's stale `review/<slice>` is **detected
and reported, never clobbered** (consistent with the moved-target rule). Audit runs
against those exact refs (D5). **Stage-2 integrate runs after the worktree is
removed**, plumbing-only from parent/root: it replays the journal to push the audited
refs to trunk/`edge` under CAS, and its own journal appends (`applied_new_oid`/
`status`) commit onto `dispatch/<slice>` via `commit_tree` (no checkout). The ledger's
three files are present on `dispatch/<slice>` before stage-1 (boundaries/orthogonal
accumulated through the funnel; journal rows written as each stage mutates refs).

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
- **D2b fence is defence-in-depth, NOT a proof (RV-025 B3).** The R-5 belt /
  IMP-052 post-spawn check / repo-local jail do **not** prove coverage of the full
  Orchestrator verb class (`gc`/sync). So OQ-D's close is NOT "the fence catches
  it"; it is the **plan-gate** (next bullet) + the positive marker (IMP-065).
- D3 minting resolves the trunk ref from the coordination worktree.
- Conclude removes the worktree dir, **keeps** `dispatch/<slice>` +
  `phase/<slice>-NN`.
- Re-anchor at sync onto a moved trunk (IMP-043 relocated).
- **Behaviour-preservation (the gate):** existing dispatch funnel suites stay
  green unchanged — cadence is untouched.
- **Projection mechanism (§4.1–§4.3) — now in-scope evidence (was deferred):**
  - **A — ledger placement / CAS.** `journal.toml` committed to `dispatch/<slice>`
    *before* the `update-ref`; ref updates are CAS (`update-ref <ref> <new> <old>`)
    and refuse a moved target; replay no-ops when `target==planned`, refuses+reports
    when diverged from both (ADR-012 §Verification crash matrix a–e). The ledger dir
    is **absent from `review/<slice>`** and never reaches trunk.
  - **B — composition.** `review/<slice>` diff vs trunk-base = the impl bundle;
    `.doctrine/dispatch/<slice>/**` and every `orthogonal.toml` path are **excluded**;
    impl-bundle authored entities (AC evidence/notes/design amendments/status) are
    **retained**. Golden-pin the filtered tree.
  - **C — per-phase cut.** On the claude arm, `phase/<slice>-NN` cut from
    `boundaries.toml` diffs exactly phase NN's code delta; `.doctrine/` stripped;
    empty-code phase yields no branch; stage-2 integrate treats native (codex/pi
    fork) and synthesized (claude cut) branches identically.
- OQ-D plan-gate tests (Orchestrator-verb restriction + impersonation) remain a
  `/plan` concern (out of this design pass).

## §7 — OQ / risk register

- **OQ-A / OQ-B / OQ-C — CLOSED** (see §4; formalised in ADR-012 Decisions 4/2/5).
  OQ-A delivered the projection journal + idempotent-replay recovery contract
  (RV-023 F-1 closed). **Mechanized post-acceptance in §4.1–§4.3** (run ledger +
  tree-filter); the contracts are unchanged — mechanism only.
- **Projection mechanism (A/B/C) — DESIGNED** (§4.1–§4.3). A: run ledger at
  `.doctrine/dispatch/<slice>/{journal,boundaries,orthogonal}.toml` (committed,
  runtime-coordination carve-out, CAS via `git update-ref`). B: `review/<slice>` =
  squashed filtered tip-tree (v1, reversible). C: per-phase cut from recorded
  boundaries, claude-arm only, reusing B's tree-filter. New code surface: a
  filtered-tree builder in `src/git.rs` + the sync verb's compose step.
- **OQ-D (RV-023 F-2) — RECLASSIFIED to an SL-064 plan-gate** (User ruled (a);
  RV-025 F-3/F-4). **Not an ADR-acceptance blocker.** v1 ships markerless creation
  as a transitional assumption with the D2b fence as defence-in-depth; the **plan
  MUST** (i) restrict Orchestrator-verb invocation (`fork`/`import`/`gc`/sync) to
  the trusted orchestrator path until a positive marker lands, and (ii) carry
  impersonation tests. Positive coordination marker → **IMP-065**.
- Topology-as-own-ADR — **RULED (a): ADR-012 ACCEPTED (RV-025); its acceptance
  gates SL-064 plan** (§5).
- Cold provision build on a quiet-solo-small-slice — accepted cost (DD-1).
- Concurrent-dispatch contention at `edge` conclude — controlled sync point,
  accepted.
- rtk / cargo / `CARGO_TARGET_DIR` — repo-local (ADR-008), layer-walled out of the
  framework decision.
- Dependency: reuses SL-056 `fork` / `provision`; markerless creation extends
  them. ADR-011 spawn seam untouched (references ADR-006, does not amend).

## §8 — Claude-arm base correctness (extension; SL-066-blocker remediation)

> ✅ **CONFIRMED 2026-06-14 (controlled marker-commit test; User-ruled "option Y").**
> An earlier mid-session detour (commit `babd656`) wrongly superseded this section,
> concluding the claude `Agent isolation:worktree` base was "opaque / uncontrollable"
> and routing parallel dispatch exclusively to the subprocess arm ("option X"). **A
> controlled test disproved that detour and vindicated the original §8.0 leg-1 design:**
> with `worktree.baseRef="head"` the worker forks the **spawning orchestrator session's
> local HEAD**. The orchestrator placed the session on a unique marker commit
> (`d58ce62`, on no shared ref); the worker's worktree forked **exactly that marker** —
> not `origin/main`, not an opaque session-root. ([[mem.pattern.dispatch.claude-isolation-worktree-forks-orchestrator-session-head]];
> the two retracted findings were taken with the orchestrator sitting on `main` and/or
> under default `baseRef="fresh"`.)
>
> **Decision (User-ruled, 2026-06-14): option Y — claude isolated workers are base-
> controlled and kept.** The handle is orchestrator **placement**, not a ref-redirect:
> run the orchestrator session inside the `dispatch/<slice>` coordination worktree with
> its tip checked out (§7c stationary-head + §8.0 leg-1). This solves **both** base
> problems on the claude arm, with **no `WorktreeCreate` hook**:
> - **P1 (origin staleness):** `baseRef="head"` forks local HEAD, not `origin/main`.
> - **P2 (dependent-phase base):** orchestrator advances its own HEAD/tree to phase N's
>   integrated tip before spawning N+1 → the next worker forks the dependency. Serial-
>   dependent phases ARE claude-dispatchable (contra the retracted "inline only").
> - **Parallel file-disjoint phases:** orchestrator HEAD = common base B → every worker
>   forks B.
>
> `baseRef="head"` (already set in `settings.local.json`) + the §8.4 post-spawn
> verify-verb ship as before — now the *primary* base mechanism plus its fail-loud belt,
> not a consolation. **`WorktreeCreate` (IMP-072) is demoted to a nicety** for true
> pre-worker fail-closability (§8.1 residual), NOT the base-control mechanism; its σ
> blast-radius is no longer a forcing cost. The subprocess arm (codex/pi → DeepSeek)
> remains the **cheaper** parallel path (~$1/hr), not the *only* one. `claude -p` stays
> OUT (API ~$1000/hr).
>
> §8.0/§8.1/§8.2 below are **vindicated as written**; §8.5's ADR-011 amendment is
> reframed around Y (base controlled without `WorktreeCreate`).

> Added after the spine locked: the claude `/dispatch` arm first shipped **unusable** —
> a worker forked off `origin/HEAD` (32 commits behind local trunk) lacked authored
> state and developed against a stale tree. Root cause: Claude's *default* worktree
> creation forks `origin/HEAD` (`baseRef` default `fresh`). This section makes the
> worker base **== B** via **`baseRef="head"` + orchestrator placement + matcher-scoped
> SubagentStart + a post-spawn verify-verb** (option Y, User-ruled). `WorktreeCreate`
> carried as a deferred fail-closability nicety (IMP-072), not a base-control need.

### §8.0 — Current vs target

**Current.** No `WorktreeCreate` hook; default creation forks `origin/HEAD`. Local
trunk ahead of `origin` ⇒ worker base ≠ B; caught only **late** at `import`
(semantically-wrong clean apply, D5/M1) or outright refused.

**Target — base == B by two legs:**
1. **`worktree.baseRef='head'`** (setting) ⇒ default creation forks **local HEAD**.
   Post-SL-064 the orchestrator runs *inside the isolated `dispatch/<slice>` tree*,
   so its local HEAD **is B**; under §7c stationary-head (already required) worker
   base == B **by construction**. Correctness by default.
2. **Post-spawn base==B verify-verb** (§8.4) — the belt that leg 1 is in force;
   refuse+report a residual wrong base **before `import`**, never a silent wrong-base
   landing. Retires the IMP-043 content-base deferral.

### §8.1 — Honest altitude (the residual, named)

On the claude `Agent` arm the orchestrator regains control **only when the Agent
tool returns** (worker already ran) — there is **no orchestrator moment between
`WorktreeCreate` and the worker's first command**. So the verify-verb is
**post-worker**: it prevents a semantically-wrong *import*, but **cannot prevent the
wasted worker run**. Pre-worker fail-closed needs `WorktreeCreate` (deferred). Net:
leg 1 makes wrong-base *not happen* in the stationary case; leg 2 makes any residual
(setting stripped / HEAD moved mid-spawn) **loud and pre-import** instead of
silent-and-late. Same not-fail-closable class ADR-011 D6 already accepted — strictly
above today's "opaque, caught late."

### §8.2 — D5 falsified

D5's "base opaque and **not** orchestrator-controlled" no longer holds: base is
**local-HEAD-pinned via `baseRef='head'`** and **orchestrator-controlled by
placement** (orchestrator cwd == coordination tree, HEAD==B), verified by leg 2.

### §8.3 — Installer wiring: `baseRef='head'` into `settings.local.json`

A settings-write step on the Claude refresh arm (`install_refresh`), beside the
HookSpec merge, ensuring `worktree.baseRef == "head"`:
- **absent** → write `"head"` (atomic, write-only-on-change, `dry_run`-aware —
  mirrors `install_claude_hook`);
- **present and `"head"`** → no-op;
- **present and other** → **report, do not clobber** (conflict notice; matches the
  `PrintedFallback` posture).

**Home `settings.local.json`** (the installer-owned file) ⇒ the imposition is
**per-operator-machine, not team-committed**. **Framework-level** (every claude
client needs base==B), not a repo-local op — the layer wall holds.

### §8.4 — Post-spawn worker-verify verb (consolidates prose IMP-052 + closes IMP-043)

New `worktree` subcommand the `/dispatch-agent` funnel calls **after the worker
returns, before `import`**, replacing the prose IMP-052 step with mechanism.
Provisional surface: `doctrine worktree verify-worker --base <B> [--dir <worktree>]`.

Pure/impure split (ADR-001; mirrors `classify_stamp`/`classify_import`):

```rust
enum WorkerVerify { Ok }
enum WorkerVerifyRefusal { NoWorkerHead, Unstamped, WrongBase }  // distinct named tokens
fn classify_worker_verify(
    head_resolved: bool,     // worker worktree HEAD resolves
    marker_present: bool,    // IMP-052: the stamp landed
    base_is_ancestor: bool,  // B is an ancestor of the worker HEAD  ⟺  built on B
) -> Result<WorkerVerify, WorkerVerifyRefusal>
```

**base==B test = `git merge-base --is-ancestor B <worker-HEAD>`** — the IMP-043
content-base assertion. Forked at B ⇒ B is-ancestor of S ⇒ pass; forked at stale
`origin/HEAD` ⇒ S descends from origin/HEAD not B ⇒ false ⇒ `WrongBase`. Precond
order: head-resolves → marker → base (unstamped names itself first).

Shell `run_verify_worker`: gather (read marker from the worktree withheld tier;
`merge-base --is-ancestor` via `src/git.rs run_git`, rtk-bypassing) → classify → on
refuse print token + non-zero (funnel **halts, preserves the fork**); on `Ok`
proceed to `import`. Post-worker (§8.1).

### §8.5 — Governance: ADR-011 amendment (in place)

ADR-011 owns the base-pinning cell + altitude table ⇒ **amend in place** (not a new
ADR; correction-with-authority, consistent with SL-064 amending ADR-006 / authoring
ADR-012):
- **D3 table, claude base-pinning cell:** `opaque residual (charge-2, D5)` →
  `local-HEAD via baseRef='head'; orchestrator-controlled by placement (cwd==coord
  tree, HEAD==B); post-spawn base==B verify-verb`.
- **D5:** "opaque and not orchestrator-controlled" → **falsified** (§8.2); the
  clean-applying-semantically-wrong import worst case **closed** by the verb (was
  IMP-043).
- **D6:** correct the premise — the `WorktreeCreate` deferral rested on "payload
  carries no path/base," **wrong about why**: the hook *creates*, so it supplies
  path+base; the real blocker is **no `agent_type`/matcher ⇒ global fire ⇒ σ
  blast-radius**. v1 altitude = `baseRef='head'` + matcher-scoped SubagentStart +
  post-spawn verify-verb. Still not pre-worker fail-closable (same accepted class).
- **D7 (σ blast-radius):** withdrawal **holds for v1** (no `WorktreeCreate`
  installed); the deferred follow-up would **re-open** it (global creation-replace,
  benign pass-through + discriminator race).

**Tracker moves.** IMP-052 promoted **prose → verb** (folded into `verify-worker`).
IMP-043 content-base assertion **closed** by the verb. `WorktreeCreate` pre-worker
fail-closed arm = **IMP-072** (already captured), now gated **only** on a demonstrated
need for true pre-worker fail-closability — arbitrary-B base control is solved by
orchestrator placement under Y, so it is no longer a justification; carries the σ
blast-radius cost explicitly.

### §8.6 — Verification impact

- **Install writer (VT):** golden — `worktree.baseRef="head"` written when absent;
  idempotent no-op when `"head"`; conflict-report (no clobber) when present≠head;
  coexists with boot+stamp entries untouched.
- **Pure `classify_worker_verify` (VT):** unit per verdict (`NoWorkerHead` /
  `Unstamped` / `WrongBase` / `Ok`); precond ordering pinned.
- **`verify-worker` e2e (VT):** stale-base fork ⇒ `WrongBase` + non-zero + fork
  preserved; unstamped ⇒ `Unstamped`; B-based stamped worktree ⇒ `Ok` → proceeds.
- **baseRef harness behaviour (VH):** that Claude forks the spawning orchestrator
  session's local HEAD under `baseRef='head'` is harness behaviour, **not
  doctrine-unit-testable** — **VERIFIED 2026-06-14** by a controlled marker-commit test
  (worker forked the orchestrator's unique marker `d58ce62`). VH satisfied.
- **Matcher drift (VT, regression):** existing `DISPATCH_WORKER_AGENT_TYPE` drift
  test stays green (SubagentStart untouched).
- **Memory corrections (DONE this pass):** retracted
  [[mem.pattern.dispatch.claude-isolation-worktree-base-session-root-opaque]] and
  [[mem.pattern.dispatch.claude-agent-worktree-forks-origin-main-tracking-ref]] (both
  false: base IS controllable by placement); recorded
  [[mem.pattern.dispatch.claude-isolation-worktree-forks-orchestrator-session-head]].
  NB the worker forks the **orchestrator session's local HEAD** (it tracks placement) —
  the babd656-era "it is origin/HEAD" gloss was itself wrong; `origin/HEAD` is only the
  default-`fresh` fallback.

## Decision log (this pass)

- DD-1 always-on dedicated coordination worktree — **locked.**
- DD-2 integration-sync = two-stage downstream projection, a verb, configurable
  targets, never-auto-trunk — **architecture locked; routing policy CLOSED** (OQ-A/
  B/C resolved, ADR-012 Decisions 4/2/5): stage-1 prepare-review → audit → stage-2
  integrate; intent → `review/<slice>` by default (trunk opt-in ff-only + CAS);
  four-bucket temporal/dependency boundary with default-hold classifier;
  audit-from-parent against the prepared review refs; CAS journal recovery.
- Branch/target roles `dispatch/<slice>` / `phase/<slice>-NN` / `review/<slice>` /
  `edge` — **locked.**
- DD-3 markerless coordination-tree creation; regenerate-not-copy provisioning —
  **locked.**
- DD-5 branch-point/re-anchor demote to sync point — **locked.**
- DD-6 worktree-life < branch-life (resolves IMP-041) — **locked.**
- DD-7 run ledger = three co-located committed files at `.doctrine/dispatch/<slice>/`
  (journal/boundaries/orthogonal); split for per-event append efficiency + SRP,
  co-located so B excludes one prefix; runtime-coordination tier git-committed as a
  bounded carve-out — **locked.**
- DD-8 tree-filter primitive (`filter_tree`+`commit_tree` in `src/git.rs`) shared by
  B and C; CAS via native `git update-ref` 3-arg — **locked.**
- DD-9 `review/<slice>` v1 = single squashed filtered tip-tree, parent trunk-base;
  reversible (regenerable from SSoT) — **locked.**
- DD-10 per-phase cut (C) = code-only filtered tree of `code_end_oid` onto prior
  phase branch, from `boundaries.toml`; claude-arm only — **locked.**
- Delta-class temporal boundary, intent-routing, audit ordering — **OQ (closed; see
  §4 / ADR-012).**
- DD-11 claude-arm base correctness = **two legs**: `worktree.baseRef='head'`
  (correct-by-default, base==B in the isolated coord tree under §7c) + a post-spawn
  base==B verify-verb (fail-loud belt, pre-import) — **locked (§8).**
- DD-12 installer writes `worktree.baseRef='head'` to `settings.local.json`
  (per-operator, not team-committed); absent→write, head→no-op, other→report-no-clobber
  — **locked (§8.3).**
- DD-13 `verify-worker` verb consolidates prose IMP-052 (marker) + closes IMP-043
  (base==B via `merge-base --is-ancestor B HEAD`); pure classifier + impure shell
  (ADR-001) — **locked (§8.4).**
- DD-14 ADR-011 amended in place (D3/D5/D6/D7); WorktreeCreate option-3 deferred on
  the σ blast-radius (no agent_type/matcher), NOT on payload fields — **locked (§8.5).**
- DD-15 (2026-06-14) DD-11's "base==B by placement" **empirically confirmed** by a
  controlled marker-commit test: under `baseRef='head'` the worker forks the spawning
  orchestrator session's local HEAD. Reverses the babd656 mid-session detour (option X /
  "base uncontrollable / subprocess-only parallel"), which was drawn from runs taken on
  `main` and/or under default `baseRef='fresh'`. **Option Y locked:** claude isolated
  workers are base-controlled (P1+P2, parallel + serial-dependent) by orchestrator
  placement; `WorktreeCreate`/IMP-072 demoted to a pre-worker fail-closability nicety,
  no longer a base-control need. Two memories retracted, one recorded (§8.6).
