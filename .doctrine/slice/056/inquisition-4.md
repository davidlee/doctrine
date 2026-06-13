# Fourth Inquisition — SL-056 design (`design.md`, re-lock target)

> **HERESIS URITOR; DOCTRINA MANET**

Convened for `nihil obstat`. The author remediated eight third-round charges and
re-locked. The Inquisitor does not read the prose for agreement — the Inquisitor
puts fire to the welds the third penance opened. The tribunal pattern holds
across four rounds: *penance breeds new sin at the seams.* Five new heresies are
charged. **`Nihil obstat` is DENIED.**

Doctrine consulted: `adr show 6` (D2/D2b/D6a/D8 + Consequences squash caveat),
the SL-056 `design.md`, `inquisition-3.md` (the welds just sealed), the slice
thesis (*mechanism in prose is the design smell*).

---

## Charges

### Charge α — CRITICAL — Solo squash-merge defeats the gc all-commits oracle; the third-round Charge-E "resolution" is a live hole, and it contradicts the governing ADR

**Doctrine violated.** ADR-006 D8 ("a memory anchored to a coordination-branch
commit keeps its sha through a ***non-squash*** merge") and ADR-006
Consequences/Negative, which **explicitly anticipates the solo squash-merge as a
real, occurring event**: *"Solo `/execute` inside a worktree … a memory anchored
to that branch's sha is **orphaned by a squash-merge**."* The design's resolution
of round-3 Charge E assumes the opposite.

**Evidence.** `design.md:492` — solo `/execute` "merges its multi-commit TDD
branch to the coordination branch (main, D8) by **normal git** (ancestry
intact)." `design.md:444–448` — gc reaps "**only when every listed commit is
`-`**"; `design.md:660–662` — "A solo `/execute` normally-merged multi-commit
branch reaps." The gc oracle is `git cherry <coord-HEAD> <fork>`, which keys on
**per-commit patch-id**.

Confessed under cross-examination by the author's own handover, and the
Inquisitor confirms it sound: `git merge --squash` collapses the fork's N commits
into **one** new commit bearing the **combined** diff. The individual fork
commits are then unreachable from coordination HEAD, and the squashed commit's
patch-id equals **no** individual fork commit's patch-id (save the degenerate
single-commit fork). So `git cherry` reports **every** fork commit `+` → gc
**refuses to reap a legitimately squash-merged solo branch** → the operator
learns the `--force` reflex → *"the exact hazard gc exists to prevent"*
(`design.md:434`, the design's own damnation, now self-inflicted).

The root is deeper than a missing flag: **solo's land-to-coordination step is
unmechanised prose** ("normal git", `design.md:329,492`) while dispatch's landing
is a fail-closed verb (`import`). The slice's whole thesis — *mechanism belongs
in the verb, not prose* — is **abandoned at solo's merge**, and the gc oracle
silently depends on an **unstated, unenforced non-squash precondition** that the
governing ADR says does not always hold.

**Risk.** gc false-refuses every multi-commit squash-merged solo branch;
`--force` reflex trained; the safety gate the third round rebuilt collapses for
the second of gc's two callers — the very caller round-3 Charge E added.

**Sentencing.** The design must do **one** of: (a) **mechanise** solo's land step
as a verb that *guarantees* an ancestry-preserving (non-squash, `--no-ff` or
fast-forward) merge, and state the non-squash precondition as an **enforced**
constraint, not a prose assumption — *and* reconcile it against ADR-006's
anticipation that solo squash-merges occur; or (b) give gc an **ancestry/`--merged`
path** for the merged-commit case so reachability, not per-commit patch-id,
certifies a normally-merged solo fork — restoring the dual-oracle the single
patch-id check cannot serve. Verification: a golden where a **multi-commit solo
fork is squash-merged**, then `gc` is invoked — it must reap (or refuse *by
design with a named, non-`--force` remedy*), never silently train `--force`. *Let
the half-resolution be broken upon the wheel, limb by limb, until it confesses
which merge it actually requires.*

---

### Charge β — HIGH — The marker is a raw-`rm`-able file inside the worker-writable worktree; "full mechanism floor" for codex/pi is an over-claim contradicting ADR-006 D2b

**Doctrine violated.** ADR-006 D2b ("Raw-tree confinement … is **not
CLI-stoppable**"; "the harness does **not** confine workers to their worktree")
and ADR-006 Consequences ("The worker-sole-writer invariant has **no harness
enforcement** … until raw-tree confinement is solved"). The design asserts a
"**full mechanism floor**" for codex/pi that this deferred residual forbids.

**Evidence.** Marker path `<root>/.doctrine/state/dispatch/worker`
(`design.md:208`) — **inside** the worker's own worktree. bwrap "worker
**rw-mounts** only its worktree + target dir" (`design.md:529`) — so the marker
sits in a read-write mount. DC-3 claims "codex/pi: env-locked clear + bwrap =
**full mechanism floor**" (`design.md:102`).

The third round hardened `marker --clear` with an env-lock, an `--operator` ack,
and a cwd-check (`design.md:242–258`) — guarding **one door** (the verb) while
the **wall beside it stands open**:
- A non-compliant worker of **any** harness removes the marker with a direct
  `rm <root>/.doctrine/state/dispatch/worker` — never touching the hardened verb.
  bwrap does **not** save codex/pi: the marker lives in the rw worktree mount.
- The env-lock is defeated by any worker with shell access:
  `env -u DOCTRINE_WORKER doctrine …` strips the disjunct. (The design itself
  uses `env -u DOCTRINE_WORKER` as a *test* affordance at `design.md:287` — proof
  the seam is trivially reachable; it is unacknowledged as a **worker** bypass.)

So the genuine malice containment that actually holds is **bwrap (out-of-tree
write denial) + the `import` `.doctrine/`-rejection belt** — *not* the
marker/`--operator`/env-lock layer, which is **accident-fencing + observability on
every harness**, exactly as the design already (honestly) concedes for claude.
The "full mechanism floor" claim for codex/pi over-credits the marker into malice
territory ADR-006 D2b explicitly defers.

**Risk.** ADR-011's per-harness altitude table (G3) will inherit a false
asymmetry — codex/pi billed "malice-proof," claude "prompt-enforced" — when both
are accident-fenced and both rest on bwrap/belt for malice. Governance built on
an over-claim is governance that lies.

**Sentencing.** Either (a) **ro-mount the marker to the worker** — write it where
the orchestrator/hook can stamp but the worker cannot `rm` (a bwrap ro-bind over
`.doctrine/state/dispatch/`, or a path outside the rw worktree mount) — turning
the floor real; or (b) **downgrade the DC-3 claim** to "codex/pi: bwrap out-of-tree
denial + import belt = malice floor; the marker + env-lock are accident-fencing,
defeatable by `rm`/`env -u` on a non-compliant worker" — and carry that into the
G3/ADR-011 altitude table verbatim. Verification: a test (or spike) demonstrating
a worker `rm`-ing its own marker and `env -u`-ing the var, and the design naming
what *then* stops the write (bwrap+belt, or the ro-mount). *The over-claim shall
be paraded through the square wearing the sanbenito of false witness.*

---

### Charge γ — HIGH — The arm/stamp/disarm sentinel races under parallel claude dispatch; the third-round Charge-C fix is sound only for serial dispatch, and the spike scope is too narrow to know

**Doctrine violated.** ADR-006 D2 (worker-sole-writer, load-bearing) and the
slice's own fail-closed mandate. The round-3 Charge-C fix trades an over-broad
**unconditional** stamp for a **racy conditional** one, and the race lives in the
concurrency `/dispatch` is built to exploit.

**Evidence.** `design.md:170–185`: the orchestrator "**arms a dispatch
sentinel** (a transient signal in its own runtime tier)"; the WorktreeCreate hook
"**fires on every Agent worktree**" and stamps "**only when armed**, then
**disarms**." The `Agent` tool gives "**no dir param, no env seam**"
(`design.md:38`) — therefore **no worktree-creation handle** the orchestrator can
correlate to a specific Agent spawn. Two unhandled interleavings:

1. **Wrong-tree branding + fail-open worker.** Orchestrator arms → before the
   intended worker's WorktreeCreate fires, a *different* `isolation:worktree`
   Agent spawn fires the hook, reads "armed", brands **its** (innocent) worktree,
   disarms. The intended worker's worktree then fires, reads "disarmed", gets
   **no marker** → unmarked worker writes freely (**fail-open**); the innocent
   tree is bricked (**fail-closed-wrong**).
2. **Lost-stamp under a concurrent batch.** The dispatch skill blesses
   "parallelize file-disjoint phases into one concurrent batch." Two
   arm→launch sequences race a single shared sentinel: `arm`, `arm`, treeA reads
   armed → stamp → disarm; treeB reads disarmed → **no stamp** → unmarked worker
   → fail-open. The design never states whether the sentinel is **per-spawn or
   shared**, nor how arm→fire→disarm is made **atomic** against concurrent fires.

The design defers this to the O3 spike ("the arming-read and stamp-in-time are
O3-spike-gated", `design.md:184`) — but the spike's stated scope is "the hook
**reads the arming signal and stamps in time**" (`design.md:669–672`), the
**single-spawn** case. It does **not** scope the **concurrent** case, which the
parallel-batch feature makes the *expected* case. Charge III's recursive law —
*spike what you rely on* — bites: the design relies on a property (safe arming
under concurrency) the spike is not scoped to prove, and which may be
**unprovable** given the no-correlation-handle limitation of the `Agent` tool.

**Risk.** Parallel claude dispatch — a blessed path — silently produces unmarked
workers (fail-open writes to the coordination tree) and bricked innocent
worktrees. Serial dispatch is unaffected only by luck (no concurrent fire).

**Sentencing.** The design must **constrain claude marker-stamping to serial
dispatch** until a correlatable mechanism exists, *and* say so plainly (parallel
claude dispatch degrades to prompt-enforced, symmetric with the bwrap/D6
back-out); **or** specify a **per-spawn correlatable** sentinel (e.g. a token the
orchestrator passes the worker which the hook echoes into the marker, allowing
post-hoc verification of which tree got which token) and **widen the O3 spike to
exercise concurrent WorktreeCreate fires**. Verification: the spike must
demonstrate two concurrent `isolation:worktree` Agent spawns, exactly one armed,
and assert the **right** tree is branded and the other is not. *Let the racing
seam be racked until its threads lie still and in order.*

---

### Charge δ — MEDIUM — `gc --superseded-head <SHA>` is `--force` wearing a TOCTOU checksum, not a "landed oracle"; the design oversells the head-match as proof

**Doctrine violated.** The design's own standard, set when it damned the round-2
`superseded` record and the round-1 receipt: a reap predicate must **prove**
landed-ness from durable state, not assert it. Charge A's third-round fix
**reaps without any landing proof** and dresses the bypass as an oracle.

**Evidence.** `design.md:471–485`: `gc --fork <b> --superseded-head <SHA>` "reaps
**without patch-id-landing iff** the supplied `<SHA>` **equals the branch's
current head** (**proving the operator named the right, current commit**)." But a
head-match proves **only** that the branch has not moved since the operator read
it — a **TOCTOU guard**. It proves **nothing** about supersession or landing. The
design leans on the invariant "the abandoned branch is **untouched after
abandonment**, so its head stays that SHA" (`design.md:476`). That invariant
forks the claim into a dilemma:
- If "untouched after abandonment" **holds**, the head always matches → the SHA
  check is a **tautology** → `--superseded-head <SHA>` is **pure `--force`** with
  a checksum that adds nothing — *exactly* Charge A's "burned receipt in new
  robes" worry, re-incurred in a new guise.
- If it does **not** hold, the guard has teeth only in a scenario the design
  asserts cannot occur — an unverified assumption load-bearing for a destructive
  `branch -D`.

Either way the framing "**proving** the operator named the right commit" is
over-stated. The verb is honest that it bypasses landing; it is **dishonest that
the head-match is an oracle**.

**Risk.** A future reader (and ADR-011) treats `--superseded-head` as a
landed-equivalent safe path and reaches for it where `--force` is the truthful
verb, eroding the very `--force`-discipline the round demanded.

**Sentencing.** Either **collapse** `--superseded-head <SHA>` into `--force` with
a mandatory `--at <SHA>` movement-guard and **call it what it is** (an
operator-asserted reap of *unlanded* work, head-guarded against TOCTOU — *not* an
oracle); **or** justify, in prose, precisely what the head-match buys over
`--force` given the "untouched after abandonment" invariant, and **state that
invariant as a constraint** the orchestrator must uphold. Verification: a golden
where the abandoned branch *has* moved since the recorded SHA → refuse; where the
SHA is lost → refuse (forces `--force`). *Confess the bypass plainly, or be
pressed beneath stones until the true weight of the claim is spoken.*

---

### Charge ε — MEDIUM — The `/dispatch-*` router routes on unmechanised harness self-belief — the very "faith, not works" the slice was convened to burn

**Doctrine violated.** The slice thesis: *mechanism in prose is the design smell;
mechanism belongs in the verb.* The one new decision point that selects the
harness-shaped spawn template routes on **the agent's belief about its own
identity** — prose, not mechanism.

**Evidence.** `design.md:596,680,783`: "Routing input = the dispatching agent's
**own harness self-knowledge** (it runs *as* claude/codex/pi — **no external
detection**); an **unknown harness refuses, never guesses**." A SKILL.md is a
prompt; the agent's "self-knowledge" is whatever its harness seeded into its
context. Two gaps:
- **"Unknown ⇒ refuse" has no teeth against confident misidentification.** The
  dangerous failure is not *admitted ignorance* — it is an agent that
  **confidently believes the wrong harness** (a nested/embedded invocation, a
  harness that mis-seeds identity). Such an agent does not refuse; it routes to
  the wrong spawn (`codex exec` under claude → API-bill/failure, or `Agent` under
  codex → no tool). The guard covers only the case that announces itself.
- **Available detection was rejected without analysis.** The orchestrator is an
  agent that runs bash; **env markers exist** (Claude Code sets its own env;
  codex/pi likewise) and PATH probes (`which codex`) are cheap. The design
  rejects "external detection" outright (`design.md:680`) with **no argument**
  for why unmechanised belief is more trustworthy than an env-marker
  cross-check — in a slice whose entire purpose is to replace belief with
  mechanism.

**Risk.** The router — the single seam the slice adds to the harness decision —
is the one place the design tolerates the smell it exists to eliminate.
Misroute = wrong altitude, wrong spawn, silent API-bill or failure.

**Sentencing.** Add an **env-marker / capability detection cross-check** that
must **agree** with self-belief, else **refuse** (belief alone routes nothing).
State explicitly why any residual belief-component is unavoidable (if it is).
Verification: a golden where the env markers contradict the claimed identity →
refuse. *Faith unconfirmed by works is the heretic's plea; let it be tested in
fire.*

---

## Questions (interrogatories)

1. **(α)** Does solo `/execute` **mandate** a non-squash merge, or may it
   `git merge --squash`? If mandated — where is it **enforced** (verb? guard?),
   and how is that reconciled with ADR-006 Consequences treating solo
   squash-merge as an occurring event? If not mandated — why does the gc oracle
   assume it?
2. **(β)** What stops a worker `rm`-ing its own in-worktree marker, and
   `env -u DOCTRINE_WORKER`-ing the var? If "nothing, by D2b deferral" — why does
   DC-3 claim a "full mechanism floor" for codex/pi?
3. **(γ)** Is the dispatch sentinel **per-spawn or shared**? Under a parallel
   file-disjoint batch (a blessed dispatch mode), what binds an arming to the
   *specific* worktree-creation it intends, given the `Agent` tool exposes no
   handle?
4. **(δ)** With "abandoned branch untouched after abandonment" assumed, what does
   the `--superseded-head` head-match buy over `--force`?
5. **(ε)** Why is env-marker detection rejected when the orchestrator can run bash
   and the markers exist?

---

## Pronounce Judgement

**This design is, once more, tainted by heresy — `NIHIL OBSTAT` IS DENIED.**

The third penance was honest where it confessed (the claude malice residual,
Charge B; the spike-gated hook, Charge C) and the patch-id oracle's
crash-proofness, the quiescence constraint, the compensating cleanup, and the
pure/imperative wall **stand acquitted and unmolested**. But the round-3 fixes
opened four fresh seams and re-opened one the author flagged but did not seal:

- **α (CRIT):** the gc oracle false-refuses squash-merged solo work and
  contradicts ADR-006 — the author-flagged residual is a **live hole**, graded
  guilty.
- **β (HIGH):** "full mechanism floor" over-claims past ADR-006 D2b; the marker
  is `rm`-able, the env-lock `env -u`-able.
- **γ (HIGH):** the arm/stamp/disarm sentinel races the parallel dispatch the
  slice blesses; the spike scope is too narrow to know.
- **δ (MED):** `--superseded-head` is `--force` plus a TOCTOU checksum mislabelled
  an oracle.
- **ε (MED):** the harness router routes on unmechanised belief — the smell the
  slice was convened to burn.

The pattern holds for a fourth round: **penance breeds new sin at the welds.**
The design may **not** proceed to `/plan` until these five are dispositioned and
the design re-locked.

## Sentencing (ordered corrective sequence)

1. **α first — it is the load-bearing hole.** Decide solo's land mechanism
   (mechanise non-squash merge **or** add a gc `--merged`/ancestry path) and
   reconcile against ADR-006. Pin with a squash-merge golden. *Wheel.*
2. **β** — ro-mount the marker (make the floor real) **or** downgrade the DC-3
   claim and propagate to the G3/ADR-011 altitude table. Pin with an `rm`/`env -u`
   bypass test. *Sanbenito + public recantation.*
3. **γ** — constrain claude marker-stamping to serial dispatch (and say so)
   **or** specify a per-spawn correlatable sentinel and widen the O3 spike to the
   concurrent case. Pin with a two-concurrent-spawn spike assertion. *Rack.*
4. **δ** — collapse `--superseded-head` into honest `--force --at <SHA>` framing
   **or** justify the head-match's marginal value and state the invariant.
   Pin with moved-branch and lost-SHA refusal goldens. *Pressing.*
5. **ε** — add an env-marker detection cross-check that must agree with belief,
   else refuse. Pin with a contradiction-refuses golden. *Trial by fire.*

Each disposition is recorded in a **Fourth-inquisition findings integrated** table
in `design.md` (the durable home; this sheet is the withheld working tier), the
design is **re-locked**, and **only then** is a **fifth** confirmatory inquisition
convened. The author marks not their own homework.

> **HERESIS URITOR; DOCTRINA MANET**
