# Sixth Inquisition — SL-056 design (`design.md`, re-lock target)

> **HERESIS URITOR; DOCTRINA MANET**

Convened for `nihil obstat`. The author remediated four fifth-round charges
(ζ/η/θ HIGH, ι MED; δ acquitted — commit `b18d225`) and re-locked. The Inquisitor
reads not the prose for its absolution. Five rounds the tribunal's law has held,
and it holds a sixth: **penance breeds fresh sin at the welds.** Round 5 forged
three new mechanisms — the single-slot `--arm` lock (θ-remedy), the
`git merge --abort` on conflict (η-remedy), and the `dispatch-fork` marker refusal
(ζ-remedy). The iron was laid to all three. **Each new mechanism leaks at its
seam.** Three fresh heresies are charged; the fifth-round MED (ι) survives the
fire substantially clean and is **near-acquitted**, with one thin residual put to
the User as an interrogatory.

**`Nihil obstat` is DENIED.**

Doctrine consulted: the SL-056 `design.md` (re-lock target), `inquisition-5.md`
(the welds just sealed), `adr show 6` (D2/D2b/D8 — worker-sole-writer,
report-don't-merge), the slice thesis (*mechanism in prose is the design smell;
faith, not works, is the heretic's plea*), and the design's own established law —
**Charge V** (every marker-state mutation has a removal owner) and **Charge VIII**
(a fallible cleanup reports the leftover by name and exits non-zero, never
silent-success). The author's own standards are the rod the author is measured by.

---

## Charges

### Charge κ (kappa) — HIGH — THE SHARPEST WELD: the armed sentinel has **no remover, no timeout, no in-CLI recovery** — a worker (or compliant orchestrator) whose Agent dies before stamping leaves the single-slot lock **armed forever**, and **all future claude dispatch is bricked**. This is Charge V / Charge II recurring **one tier up**, at the sentinel, with none of the lifecycle the marker was granted.

**Doctrine violated.** The design's **own** Charge V law (`design.md:280-289,908`):
*"Lifecycle (owned, not assumed). Written by … removed by `gc` … rolled back if
`fork` fails … cleared by `marker --clear`."* The marker earned a **four-owner
lifecycle** because Charge V damned a state-mutation with no remover. The sentinel
the θ-remedy minted has **one** owner (disarm-on-stamp) and **no failure owner at
all.** And the design's own Charge II remedy (`design.md:284-299,930`): when a
runtime-tier signal can brick the legitimate writer, it **must** carry an in-CLI
cure — that is the whole reason `marker --clear` exists. The sentinel carries
none.

**Evidence.** The θ-remedy (`design.md:198-200,218-225`): the orchestrator
*"arms a dispatch sentinel via `doctrine worktree marker --arm` (a transient signal
**in its own runtime tier** the hook reads; single-slot — refuses `already-armed`
if a sentinel is armed or a stamp is still awaited)"*; the WorktreeCreate hook
*"provisions the fork **and**, only when armed, stamps the marker — then
**disarms**."* Disarm is named at exactly three sites — `design.md:200` (*"then
disarms"*), `:218` (*"await stamp+disarm"*), `:224` (*"await stamp+disarm"*) —
and **every one couples disarm to the stamp event inside the hook.** Confessed
under cross-examination, the design specifies:
- **No `marker --disarm` verb.** Searched end to end — it does not exist.
- **No timeout / lease / TTL** on the armed state.
- **No `--arm --force` / re-arm / clear-stale-arm** path.
- **No arm-rollback** analogous to `fork`'s compensating cleanup (`design.md:152-154`).
- **No sentinel path** even specified — *"its own runtime tier"* (`design.md:198`)
  names no file, where the marker names `.doctrine/state/dispatch/worker`
  (`design.md:256`). The sentinel is both **unowned and unlocated.**
- **`marker --clear` does not reach it.** `--clear` removes the **marker** at the
  cwd tree root (`design.md:290-292`) — a *different* file in a *different* path
  from the *"own runtime tier"* sentinel. There is **no** `marker --clear`
  equivalent for the arm signal.

Now the kill. The arm/stamp/disarm contract assumes the stamp **always fires**.
But the `Agent` tool spawn is the thin **impure** shell (DC-1) — it can die,
error, be interrupted, or never reach WorktreeCreate. On **any** such death:
- The stamp is never consumed ⇒ the hook never disarms ⇒ **the sentinel stays
  armed.**
- Every subsequent `doctrine worktree marker --arm` refuses **`already-armed`**
  (`design.md:198,219-225`) ⇒ **every future claude dispatch cannot arm** ⇒ the
  marker-stamping mechanism is dead ⇒ at best a permanent silent degrade to
  prompt-enforced, at worst a hard refusal — **claude dispatch bricked.**
- The remedy is **filesystem surgery on an unspecified path** — the *exact* toil
  this verb family was convened to abolish, and the *exact* self-brick Charge II
  (`design.md:930`) and Charge V (`design.md:908`) were raised to kill.

The cruelty is symmetric: a **compliant** orchestrator that correctly detects
*"my one Agent died before it stamped"* has **no CLI verb to clean up** — it cannot
`--disarm` (no such verb), cannot `marker --clear` it (wrong path), cannot re-arm
(refused `already-armed`). The mechanism that round 5 crowned for making the
single-slot *"a CLI mechanism, not orchestrator discipline"* (`design.md:218-219`)
has **no CLI exit** from its own locked state. The θ-remedy traded a fail-**open**
race for a fail-**closed brick with no key** — and a brick with no key is the
purer heresy, for it wears the robe of safety.

**Compounding — privilege class of `--arm` is unspecified.** The handover and the
fifth tribunal both asked (`inquisition-5.md:318`): is `--arm` `Orchestrator`-
classed? The design classifies `fork/import/gc/land` (DC-3) and gives `marker
--clear` its bespoke fourth class — but `marker --arm` is classified **nowhere**
(`design.md:728` names it only as *"an orchestrator arming sentinel … single-slot
lock refusing `already-armed`"*, no class). A non-compliant worker — and claude's
worker is confessedly only accident-fenced (Charge B) — may therefore `--arm` to
grief: spam it to keep the slot `already-armed`, or arm so the orchestrator's next
legitimate isolated Agent worktree is mis-branded. The arm verb is **unowned in
its lifecycle AND unclassed in its privilege.**

**Risk.** A blessed, ordinary failure (an Agent dying mid-spawn — the impure shell
doing what impure shells do) **permanently bricks claude marker-dispatch** with no
in-CLI recovery, and forces the operator to `rm` an unspecified runtime file. The
worst outcome is reachable by the harness simply misbehaving once.

**Sentencing.** **Give the sentinel the lifecycle the marker already has — and
the in-CLI cure Charge II already established.** Three teeth, all cheap:
1. **A `doctrine worktree marker --disarm`** (or `--arm --force`) verb — the
   in-CLI remover, mirroring `marker --clear`'s role for the marker. A compliant
   orchestrator whose Agent died clears the slot without filesystem surgery.
2. **A staleness bound on the armed state** — a TTL/lease, or arm carries the
   orchestrator's own liveness token so a dead-orchestrator arm is reclaimable —
   so an *abandoned* arm cannot brick dispatch indefinitely even if `--disarm` is
   never called.
3. **Specify the sentinel path** (as the marker's is specified) and **classify
   `--arm`/`--disarm`** privilege (Orchestrator, or state why a worker arming is
   harmless), closing the grief surface.
Verification: a golden/spike where an **arm followed by a simulated stamp-failure**
leaves the slot recoverable — a subsequent `--disarm` (or TTL expiry) restores
`--arm` to success — never the present permanent `already-armed` brick. *Let the
bolt that admits but one penitent also have a key for the gaoler, lest the gaol
swallow the gaoler whole.*

---

### Charge λ (lambda) — HIGH — `land`'s `git merge --abort` is itself a **fallible git mutation with no failure owner** — on abort-failure `land` falls through to the clean `merge-conflict` refusal and **lies that it left nothing behind**, the *exact* dishonesty Charge VIII condemned and that `fork`'s cleanup already atones for. The η-remedy is honest one mutation deep and faithless the next.

**Doctrine violated.** The design's **own** Charge VIII standard
(`design.md:152-154,936,911`): *"The rollback is itself fallible; on a rollback
failure the verb **reports the leftover state by name and exits non-zero** — never
a silent or success-coded half-rollback."* `fork` wears this belt. `land`'s
round-5 abort does **not**.

**Evidence.** The η-remedy (`design.md:596-606`): *"on a merge conflict →
`git merge --abort` **first** (restore the clean tree `land`'s own step-1 precond
demands), then refuse `merge-conflict`, report + halt."* The whole non-destructive
claim rests on the abort **succeeding**. But the round-5 charge η that birthed this
fix turned on one law — *"`git merge` mutates the index/tree and sets `MERGE_HEAD`
**before** it reports"* (`design.md:603`); *git mutations are not atomic.* That law
does **not stop at the conflict** — it applies with equal force to **`git merge
--abort` itself.** `--abort` can fail: a half-resolved tree it cannot unwind, an
index lock, a concurrent toucher, a filesystem error, a tree dirtied between merge
and abort. On abort-failure the design specifies **nothing**:
- D4b step 3 (`design.md:596-606`) names no abort-failure branch.
- The code-impact row (`design.md:727`) says only *"`git merge --abort` on conflict
  before refusing"* — no honest-non-zero clause, where `fork`'s same row **does**
  carry *"honest non-zero on rollback failure, Charge VIII"* (`design.md:727`).
- Verification (`design.md:829-832`) asserts the tree is **clean** after a conflict
  — it tests the **abort-succeeds** path **only**, and asserts nothing for
  abort-failure.

So when `--abort` fails, `land` proceeds on its planned path: **refuse
`merge-conflict`, report + halt** — a *clean*, recoverable-looking refusal
(`design.md:608` refusal set `{tree-unclean, no-such-fork, dispatch-fork,
merge-conflict}`) — while a **half-merged coordination tree persists** with
`MERGE_HEAD` set and `UU` entries. The verb claims *"leave-nothing-behind, a true
mirror of import"* (`design.md:599`) and has in fact left a wedged tree behind,
reporting the wrong cause. This is Charge η's own wound, displaced one mutation
deeper — and it is **precisely** the silent-success half-cleanup that Charge VIII
made `fork` confess by name.

**Compounding (probe).** Is `git merge --abort` **safe/idempotent if invoked when
not mid-merge**? In land's flow abort fires only on the conflict branch where
`MERGE_HEAD` exists by construction, so this is the lesser hazard — but the design
asserts no guard, and an abort on a non-merging tree errors *"no merge to abort"*.
If any future caller or retry reaches step 3 without an in-progress merge, the
abort errors and the same unhandled-failure fall-through applies.

**Risk.** Every real `land` conflict the design admits as live (*"genuine code
coupling, or coordination moved under the run"*, `design.md:596`) that *also*
resists `--abort` leaves a **wedged coordination tree** while `land` reports a
clean `merge-conflict` and exits as though it left nothing — blinding the operator
to the half-merge against every subsequent `Orchestrator` verb. A fail-closed verb
that, on its cleanup's failure, **fail-dirties and lies** is the contradiction
Charge η was supposed to retire.

**Sentencing.** **Hold `land`'s abort to the standard `fork`'s rollback already
meets.** On `git merge --abort` failure, `land` must **report the wedged tree by
name and exit a DISTINCT non-zero** — not the clean `merge-conflict` code — naming
the leftover (`MERGE_HEAD` set / unmerged paths) and the manual remedy, never a
silent or success-coded half-abort. Add the idempotency guard (abort only when
mid-merge; otherwise report). Verification: a golden where `--abort` is forced to
fail asserts a **non-zero, named "wedged tree" refusal distinct from
`merge-conflict`** and a non-clean tree honestly reported — symmetric with `fork`'s
half-rollback golden (`design.md:752-755`). *The verb that was bound to cauterise
its own cut must not, when the cautery fails, swear the wound is clean.*

---

### Charge μ (mu) — HIGH — the `dispatch-fork` guard reads a marker that lives **inside the worktree**, so a dispatch branch whose **worktree is gone** (gc crash-window, or manual `worktree remove`) bears **no reachable marker** → the guard passes vacuously → **beltless `land` of a dispatch delta is reachable again.** The ζ-remedy did not close the hole; it **relocated** it from "misrouted verb" to "worktree-less fork", and DC-3's "mechanised, not prose" is **false** for that case.

**Doctrine violated.** DC-3's round-5 re-claim (`design.md:114-116`): *"That
condition is **mechanised, not prose**: `land` refuses a fork bearing the worker
marker (`dispatch-fork`) — a misrouted orchestrator running `land` on a dispatch
worker's branch is **named-refused**, not silently belt-bypassed."* And the
findings table (`design.md:999`): *"a misroute is **named-refused**, not silently
belt-bypassed."* Both are **over-claims** — the β/ζ family the design has twice
sworn off (*"governance built on an over-claim is governance that lies"*).

**Evidence.** D4b step 1 (`design.md:589-592`): *"**`<branch>`'s linked worktree
(if any)** does not bear the worker marker else `dispatch-fork` (round-5 Charge ζ —
`land` is solo-only; a marker-bearing fork is a dispatch worker whose delta must
funnel through the belted `import`, never `land`'s beltless merge)."* The marker is
a **file inside the worktree** (`design.md:256`, `.doctrine/state/dispatch/worker`),
in the **gitignored runtime tier** — it is **never committed to the branch**
(`design.md:323-327`: *"`.doctrine/state/**` is already gitignored … already absent
from the import delta"*). Therefore the dispatch identity is recoverable **only**
while the worktree exists. The qualifier *"(if any)"* (`design.md:590`) is the
author's own confession that the worktree may be **absent** — and when it is:
- `<branch>` exists, but has **no linked worktree** → **no marker file anywhere
  reachable from the branch.**
- The guard predicate *"linked worktree (if any) does not bear the marker"*
  evaluates **vacuously true** (no worktree ⇒ no marker borne) → the `else
  dispatch-fork` refusal **does not fire** → `land` proceeds to
  `git merge --no-ff <branch>` (`design.md:593`) and lands the dispatch branch's
  **entire `.doctrine/` delta beltless** onto the coordination branch.

Is the worktree-less dispatch branch **reachable**? Yes — by the design's own
crash-discipline and tooling:
- **gc crash-window.** gc (`design.md:458-461`) does `git worktree remove` (step 1)
  **then** `git branch -D` (step 2) — **two non-atomic git mutations.** A crash
  between them (the *exact* crash-class this design obsesses over — the entire
  patch-id-not-receipt argument, `design.md:510-515`, is crash-driven) leaves the
  **branch alive, worktree gone, marker unreachable.**
- **Manual `git worktree remove`** (no standalone "remove worktree, keep branch"
  verb is sanctioned, but git offers it freely) reaches the same state.
- The design's **own gc test** (`design.md:796`) posits partial/odd fork states as
  live — *"a partial land (conflict half-resolved out-of-band)"* — so worktree-less
  forks are squarely within the states the design models.

The ζ-remedy was supposed to mechanise *"dispatch deltas route through `import`,
never `land`."* It mechanised it with a marker that **evaporates with the
worktree** — so the instant the worktree is gone, containment collapses back to
**verb-choice prose discipline**, the very smell ζ was raised to burn. The hole did
not close; it **moved** from "orchestrator runs the wrong verb on a live fork" to
"orchestrator runs `land` on a worktree-less dispatch branch" — and the marker, by
construction (withheld, uncommitted), **cannot** be recovered from the branch to
catch it.

**Risk.** A reachable **beltless `land`** of a worker's `.doctrine/` delta — the
precise containment failure ζ and β were convened to seal — survives on any
dispatch branch whose worktree was reaped (crash) or removed (hand). And the
load-bearing DC-3 claim *"mechanised, not prose"* propagates **false** into the
G3/ADR-011 altitude table and the G4/SPEC-012 belt-scope statement
(`design.md:716-718`) as canon — the twice-slain over-claim risen a **third** time.

**Sentencing.** The marker **cannot** be made branch-recoverable without breaking
the withheld-tier model (it is uncommitted by construction), so the honest fix is
**two-fold**:
1. **Re-scope the DC-3 claim truthfully** (mirroring the β/ζ re-claims the design
   already made): the `dispatch-fork` marker guard is an **accident-fence against
   the live-worktree misroute only**; a **worktree-less** dispatch branch landed
   via `land` is **not** mechanically caught — it falls to the same D2b/operator
   residual, and the design must **say so plainly** rather than claim a blanket
   *"mechanised, not prose."* Carry the honest scope into DC-3, G3/ADR-011,
   G4/SPEC-012.
2. **Close the cheap window** where the sanctioned tooling produces it: make `gc`'s
   worktree-remove + branch-delete **ordered so a crash cannot strand a
   worktree-less branch** as the *landable* state — delete the branch **first** (or
   guard `land` to **refuse any branch whose worktree is missing**, `worktree-gone`,
   forcing the operator to confirm provenance rather than silently merging a
   marker-less fork). A `land` that refuses *"this branch has no live worktree — I
   cannot verify it is not a dispatch fork; re-create the worktree or `--force`
   knowingly"* converts the silent beltless merge into a named refusal, restoring
   the mechanism ζ claimed.
Verification: a golden where `land --fork <b>` is invoked on a **branch whose
worktree was removed** asserts a **named refusal** (`worktree-gone` /
`dispatch-fork`), **never** a silent beltless `--no-ff` merge; and the altitude
table states the marker guard's scope is the **live-worktree** misroute, not all
misroutes. *Let the brand that was promised on every dispatch fork not vanish with
the cell that bore it — or confess the brand fades, and post a watchman at the
gate instead.*

---

## Near-acquittal (fifth-round MED that substantially survives the iron)

### Charge ι (iota) — NEAR-ACQUITTED — the env-marker spike-gate is honest; `proposed` is works-confessed-unproven, **not** deferred faith. One thin residual is put to the User, not charged.

The fifth tribunal demanded ι receive the symmetric treatment of its siblings C
and D6 — spike-gate, named fallback, `proposed` status, cause-naming. The
round-5 remedy delivers all four (`design.md:731,852-858,1002`): the detection
signal is **spike-gated** (*"the O3 spike must confirm a stable, harness-unique,
launch-mode-robust marker exists per harness"*); a **named fallback** exists
(*"refuses all dispatch with that diagnostic … never a silent revert to
self-belief"*); the refusal **names the cause** (*"env marker for claimed harness
`<h>` not found — mis-seeded, renamed, or launch-mode-stripped"*); and the claim is
held **`proposed` until the marker-existence gate is green.** The handover's probe —
does *"stays `proposed`"* merely defer the same faith one ply (Charge III
recursion)? — is answered **no**: `proposed` is the design's honest *"not yet
proven"* badge, the **opposite** of faith-dressed-as-works. It is works, plainly
labelled unproven. **The core is clean — it passes.**

One **thin residual**, raised as a Question (below), not a charge: the spike
confirms a marker *"per harness"* (`design.md:731`) but the fallback is stated
**globally** (*"refuses **all** dispatch"*). The realistic spike outcome is
**partial** — claude's `CLAUDECODE` stable, a codex/pi marker flaky. The
**per-harness-partial** outcome (gate green for some harnesses, red for others) has
no stated router behaviour. This is granularity, not heresy — within *"resolved
in-skill/at the spike"* — but it deserves an explicit answer before the spike runs.

Also re-affirmed unmolested (re-open only on new evidence): the gc **ancestry-leg
monotonicity** under an advancing HEAD; the **patch-id oracle** crash-proofness;
the **quiescence constraint** (XII); the **compensating cleanup** (VIII, *for
fork* — but see λ, which extends the same honesty test to `land`'s new abort); the
δ `--superseded-head` reframe (round-5 acquittal); **orchestrator-never-env-self-
set**; the **pure/imperative wall**; the round-1→5 resolutions already re-affirmed.

---

## Questions (interrogatories)

1. **(κ)** Who disarms a sentinel whose Agent died before WorktreeCreate fired?
   Name the verb. If there is none, how does a *compliant* orchestrator recover
   in-CLI from a permanent `already-armed`, and what bounds the armed state's
   lifetime against an abandoned arm? Is `--arm` `Orchestrator`-classed — can a
   worker arm to grief?
2. **(λ)** On `git merge --abort` failure, does `land` exit a **distinct non-zero
   naming the wedged tree**, or fall through to the clean `merge-conflict` refusal?
   Where is the abort-failure path written? Is the abort guarded against
   not-mid-merge invocation?
3. **(μ)** Does the `dispatch-fork` guard hold once `<branch>`'s worktree is gone
   (gc crash-window / manual `worktree remove`)? If the marker is unreachable from
   the branch, what stops a beltless `land` of a worktree-less dispatch branch — a
   `worktree-gone` refusal, a gc ordering that deletes the branch first, or an
   honest re-scope of the "mechanised, not prose" claim?
4. **(ι, residual)** If the O3 spike finds a stable marker for **some** harnesses
   but not others, does the router refuse **all** dispatch (as the global fallback
   says) or **only** the harnesses whose marker is unproven?

---

## Pronounce Judgement

**This design is, for a sixth time, tainted by heresy — `NIHIL OBSTAT` IS
DENIED.**

The fifth penance was honest where it confessed — the δ reframe stands acquitted, ι
received its siblings' full spike-gate treatment and **passes**, and the
patch-id oracle, ancestry-leg monotonicity, quiescence constraint, compensating
cleanup (for fork), env-not-self-set, and the pure/imperative wall **stand
acquitted and unmolested.** But the round-5 fixes forged three new mechanisms, and
the iron found the leak in **every one**:

- **κ (HIGH):** the θ-remedy's single-slot `--arm` lock has **no remover, no
  timeout, no in-CLI recovery, no specified path, and no privilege class.** An
  Agent dying before it stamps leaves the sentinel **armed forever** → claude
  dispatch **bricked**, recoverable only by filesystem surgery on an unnamed file.
  Charge V/II self-brick, recurring one tier up, with **none** of the lifecycle the
  marker was granted. A fail-open race traded for a **fail-closed brick with no
  key.**
- **λ (HIGH):** the η-remedy's `git merge --abort` is a **fallible mutation with no
  failure owner.** On abort-failure `land` **fail-dirties and lies** — reports a
  clean `merge-conflict` while a half-merge persists — the exact Charge VIII
  dishonesty `fork`'s cleanup already atones for. Honest one mutation deep,
  faithless the next.
- **μ (HIGH):** the ζ-remedy's `dispatch-fork` marker guard reads a marker **inside
  the worktree**; a **worktree-less** dispatch branch (gc crash-window / manual
  removal) bears **no reachable marker** → the guard passes vacuously →
  **beltless `land` reachable again.** The hole did not close — it **relocated** —
  and DC-3's *"mechanised, not prose"* is the β/ζ over-claim risen a **third** time.

The pattern holds for a **sixth** round: **penance breeds fresh sin at the welds.**
Round 5 added an arm-lock with no key, an abort with no failure owner, and a brand
that vanishes with its cell. The design may **not** proceed to `/plan` until κ, λ,
μ are dispositioned and the design re-locked. **The author marks not their own
homework — a seventh confirmatory inquisition follows the remediation.**

## Sentencing (ordered corrective sequence)

1. **κ first — it is the live self-brick.** Give the sentinel the marker's
   lifecycle: a `marker --disarm` (or `--arm --force`) in-CLI remover, a staleness
   bound on the armed state, a specified path, and a privilege class for
   `--arm`/`--disarm`. Pin with a stamp-failure-then-recover spike. *Cut a key for
   the gaol.*
2. **λ** — hold `land`'s abort to `fork`'s Charge-VIII standard: on abort-failure,
   a **distinct non-zero naming the wedged tree**, never the clean
   `merge-conflict`; guard the not-mid-merge invocation. Pin with a forced-abort-
   failure golden. *Forbid the false oath of cleanliness.*
3. **μ** — re-scope DC-3's claim to the live-worktree case honestly **and** close
   the window: `land` refuses a `worktree-gone` branch (or gc deletes the branch
   before the worktree), propagated to G3/ADR-011, G4/SPEC-012. Pin with a
   worktree-removed-fork-refusal golden. *Post a watchman where the brand fades.*
4. **(ι residual)** — state the router's behaviour on a **per-harness-partial**
   spike result (refuse-all vs refuse-the-unproven). No charge; a Question to
   resolve before the spike.

Each disposition is recorded in a **Sixth-inquisition findings integrated** table
in `design.md` (the durable home; this sheet is the withheld working tier), the
design is **re-locked**, and **only then** is a **seventh** confirmatory
inquisition convened.

> **HERESIS URITOR; DOCTRINA MANET**
