# Inquisition (Second Tribunal) — SL-056 Design — confirmatory re-pass

*Tribunal reconvened 2026-06-13 for the* nihil obstat *re-pass. Target:
`design.md` at status `design`, re-locked `f406981` after the first tribunal's
eleven charges. Witnesses re-cross-examined via the CLI: `adr show 6` (D2/D2b/D6a/
D7/D8/D9), `adr show 8` (D-B1/D-B3/D-B5), `spec show SPEC-012` (D2/D3, FR-003).
The first tribunal burned the gross heresies — the delta-emptiness oracle, the
`Read`-classed reaper, the marker blind to the worker-on-main. The accused
confessed and did penance. **But penance breeds new sin.** The fixes welded fresh
seams, and the Inquisition was charged to put fire to exactly those welds. It has.
The accused does **not** yet earn* nihil obstat.*

> **HERESIS URITOR; DOCTRINA MANET**

---

## 1. Charges

### CHARGE I — The import receipt certifies the *apply*, not the *commit*; gc's "provably landed" oracle is gated on disposable, non-durable, crash-surviving state outside the coordination branch. **[CRITICAL]**

- **Doctrine violated.** ADR-006 **D7**: the cadence is *strict* — "import delta →
  verify → **commit** → record knowledge," and "the **coordination branch is the
  durable store**… crash/overflow recovery is **rebuild-from-coordination-branch**…
  **knowledge always trails confirmed code**."
- **Evidence.** `design.md:212-216` — `import` does `git apply --3way --index`
  (**non-committing**); step 5 **stamps the receipt "on success"** of the apply;
  the design states "The orchestrator commits **separately**… import ≠ commit." So
  the receipt is born at *apply-time*, before any commit reaches the coordination
  branch. `design.md:271-276` — `gc` "deletes **only** on a positive import
  receipt… Receipt present ⇒ the fork **provably landed** ⇒ safe to reap," with
  `-D` mandatory.
- **Revealed under cross-examination.** "Provably landed" is a lie of one word.
  The receipt proves the patch reached the **index/worktree**, not the **branch**.
  The receipt lives in the **withheld runtime tier** (`design.md:216`,
  `.doctrine/state/**`) — *gitignored, disposable,* `rm -rf`*-able* by the storage
  rule. It is therefore **not part of the D7 durable store** and **not part of the
  rebuild-from-coordination-branch recovery model** — yet it gates an
  **irreversible `git branch -D`**. Stage the window the design's own premise makes
  common: orchestrator imports (apply + receipt stamped), then crashes / overflows
  **before the separate commit**. D7 recovery rebuilds from the coordination branch
  — which **never received the commit**. The receipt, a gitignored file, **survives
  the crash on disk** and now reads "landed." A recovery-time gc sweep (or the next
  `gc --fork`) sees the receipt, runs `-D`, and **destroys the only surviving copy
  of unmerged work.** This is the *exact* hazard Charge I (first tribunal) swore the
  receipt would abolish, reintroduced through a narrower door. The design even
  boasts "D7 cadence preserved" while inverting it: *record* (the receipt) now
  **precedes** *confirmed code*.
- **Risk.** Irreversible reap of unmerged work on the crash-recovery path — the
  single hazard `gc` exists to prevent — gated on state the durability model
  explicitly disclaims.
- **Sentencing.** The receipt must certify the **commit**, not the apply. Stamp it
  **after** the coordination commit lands (anchored to the commit sha), or have
  `gc` verify the delta is reachable from the coordination branch HEAD rather than
  trust a runtime-tier flag. A landed-oracle keyed on disposable state is no oracle.
  *The false relic to the same flames that took its predecessor.*

### CHARGE II — A stray marker on the *coordination tree itself* has no remover, and gc — the only remover — is locked behind the very guard the marker trips. **[CRITICAL]**

- **Doctrine violated.** First-tribunal Charge V's own sentence ("a durable signal
  without a removal owner is unmanaged state — *res derelicta*"); ADR-006 D8 (the
  team-mode orchestrator runs on a **delta branch, itself a linked worktree**);
  DC-3 (`gc` is `Orchestrator`-classed, refused under worker identity).
- **Evidence.** `design.md:248-252` — `gc` removes a marker **only** by
  `git worktree remove`-ing **the spent fork dir**. `design.md:156-158` — the
  coordination tree's marker-absence is guarded by an **assert-marker-absent**
  *check* before promotion. **Nowhere** is there a verb that **clears** a marker
  found on a tree the operator wishes to use as coordination root. `design.md:137`
  + DC-3 — `worker_mode := env OR (is_linked_worktree && marker_present)`, and
  `gc` is refused when `worker_mode` holds.
- **Revealed.** The remediation closed Charge V for **fork** markers (gc removes
  them by removing the fork) and for fork-failure (rollback). It left the
  **coordination tree's own** marker an orphan. Stage team mode (D8): the
  orchestrator runs in a *linked delta-branch worktree* — `is_linked_worktree`
  is **true**. A stale `.doctrine/state/dispatch/worker` lands in it (a reused dir;
  a half-failed gc; a worktree recycled across roles). Now `marker_present` is true
  → `worker_mode` is true → **every authored write on the trusted orchestrator is
  refused** — *and so is `gc`*, because gc is `Orchestrator`-classed and refused
  under `worker_mode`. **The one verb that removes markers cannot run, because a
  marker is present.** The assert-marker-absent gate only *detects* the brick; it
  offers no *remedy*. Recovery collapses to the manual filesystem surgery Charge V
  condemned by name — now with the remover provably unreachable.
- **Risk.** A self-locking bricked coordination tree in exactly the mode (team, D8)
  where the orchestrator is a linked worktree; no in-CLI escape.
- **Sentencing.** Define a marker **clear** path the guard cannot strangle: either a
  non-`Orchestrator`, non-write-classed `worktree marker --clear` (refused only by
  *cwd-is-not-this-tree*, never by `worker_mode`), or make assert-marker-absent a
  **remediating** gate that clears a marker it finds on a tree being promoted to
  coordination root (with a loud receipt of what it cleared). A guard with no key is
  a dungeon, not a gate. *The gaoler who loses the key hangs from his own portcullis.*

### CHARGE III — DC-2's reinstated env leg rests on an *unvalidated* assumption that the spawn seam propagates `DOCTRINE_WORKER` into the worker process; the O3 guard-spike does not gate it, and G3 hardens it into accepted ADR-011 first. **[HIGH]**

- **Doctrine violated.** ADR-003 (decisions govern proven mechanism, not the
  reverse); first-tribunal Charge IX (spike what you fear before governance hardens
  it); the CLI-is-source-of-truth / *ask-don't-infer* canon.
- **Evidence.** DC-2 (`design.md:31-39`) — the env signal is "reliable **because**
  the orchestrator now owns the subprocess spawn… **it sets this env reliably**."
  `design.md:97-98` — "The orchestrator additionally exports `DOCTRINE_WORKER=1`
  into the spawned subprocess." The **entire remediation of first-tribunal Charge
  III** (the worker-on-main fix) rests on this: env is "the *only* signal that sees
  worker-on-main" (`design.md:173`). The O3 guard-spike (`design.md:308-309`,
  G7 sequencing) is scoped to validate "the guard + privilege model" — i.e. *does
  `run()` refuse when the env is set* — **not** *does the env arrive in the worker
  at all.* G3/ADR-011 is authored in step 1 (`design.md:307`), **before** the spike.
- **Revealed.** "Reliable" is asserted, never confessed under test. That
  `env DOCTRINE_WORKER=1 … claude -p` (and `codex exec`, and pi's depth-2
  self-subagent) actually delivers the var to the process where
  `std::env::var_os("DOCTRINE_WORKER")` reads it is an **empirical claim about three
  third-party harnesses** — any of which may scrub, namespace, or drop parent env.
  If it fails for even one, DC-2 collapses to **marker-only**, and first-tribunal
  Charge III (worker-on-main fail-open) **reopens** — the marker is blind to the
  worker on main, by the design's own admission. Worse, the spike as scoped would
  pass (the guard logic is sound) while the real plumbing is broken — a **green
  spike over a holed mechanism**, the precise asymmetry-of-rigor sin of Charge IX.
- **Risk.** The keystone worker-on-main guard is faith, not works; an accepted
  ADR-011 enshrines it before the one experiment that could refute it.
- **Sentencing.** Add to the O3 guard-spike an **explicit propagation gate**:
  spawn a real `claude -p` (and each staged harness) worker and assert the doctrine
  process *sees* the orchestrator-set `DOCTRINE_WORKER`. G3/ADR-011 stays `proposed`
  on the env-reliability claim until that gate is green. Spike what you **rely on**,
  not only what you **fear**.

### CHARGE IV — The import receipt has no removal owner; the store grows unbounded and dangles after gc. **[HIGH]**

- **Doctrine violated.** First-tribunal Charge V (the removal-owner doctrine),
  applied verbatim to the new durable signal the remediation introduced; ADR-006 D7
  ("knowledge trails confirmed code" — and is *reaped* with it).
- **Evidence.** `design.md:216` stamps the receipt; `design.md:248-254` enumerates
  gc's reaping acts — worktree, branch, **target dir** — and **omits the receipt.**
  No verb, anywhere, deletes a receipt.
- **Revealed.** The remediation gave the **marker** a lifecycle (fork writes, gc
  removes, rollback) but gave the **receipt** none. Every dispatch leaves a receipt
  behind forever; after gc reaps the fork, its receipt **dangles**, pointing at a
  deleted branch and a SHA no ref names. The store is *res derelicta* — the same
  unmanaged-state heresy, recommitted one object over. Benign today; a latent
  false-positive surface the instant receipt lookup keys loosely (Charge IX).
- **Sentencing.** Reap the receipt **in gc**, in the same act that reaps the fork it
  certified; or scope receipts to the dispatch run and sweep them at its close.
  Define the lifecycle in full, as the marker's now is.

### CHARGE V — `import`'s "exhaustive" refusal set omits the *tree-not-clean* state; the purge of `apply-conflict` is unsound without it. **[HIGH]**

- **Doctrine violated.** The design's own claim (`design.md:218`) that the refusal
  set is "**exhaustive over permitted states**"; first-tribunal Charge II's
  remediation (purging `apply-conflict` as unreachable).
- **Evidence.** `design.md:200` — precond is "coordination tree **clean**,
  `HEAD == B` (`branch-point-check` reused)." But `branch-point-check` is, by
  SPEC-012 D2/FR-003 and `design.md` C-V, a **ref-equality compare of HEAD vs base
  — it does not inspect the working tree**. `design.md:209-213` purges
  `apply-conflict` *because* "under the `HEAD == B` precond the patch `B..S` applies
  onto the **exact tree it was cut from**, so it cannot conflict." The refusal set
  `{head-moved, multi-commit, doctrine-touch}` (`design.md:218`) names **no**
  reason for an unclean tree.
- **Revealed.** The purge of `apply-conflict` rests on **two** conjuncts —
  `HEAD == B` **and** tree-clean — but only the first has a guard (`head-moved`)
  and a refusal reason. If the coordination tree carries uncommitted changes
  (a half-done orchestrator edit, a prior partial apply), the tree is **not** the
  tree `B..S` was cut from, and `git apply --3way` **can conflict** — resurrecting
  the very refusal reason just buried, now with **no name and no handler**. The set
  is *not* exhaustive over permitted states; it is exhaustive over the states the
  *un-checked* precond merely *assumes*.
- **Sentencing.** Either add a named `tree-unclean` (or `precond-dirty`) refusal and
  a real working-tree-clean check (not `branch-point-check`, which is blind to it),
  **or** retain `apply-conflict` as the catch for the unclean case. The purge and
  the missing check cannot both stand. *Mendacium per exhaustionem.*

### CHARGE VI — The assert-marker-absent gate is scoped to "coordination root"; solo `/execute` in a linked worktree is a second direct-writer class, equally bricked by a stale marker, and ungated. **[MEDIUM]**

- **Doctrine violated.** ADR-006 D6a (the **mode**, not the location, decides — and
  solo-in-worktree writes **directly**); the remediation's promise (`design.md:156`)
  that "a reused/stale fork dir cannot fail-close a **legitimate writer**."
- **Evidence.** `design.md:156-158` gates marker-absence "before a tree may serve as
  a **coordination root**." But D6a/`design.md:165-166` — solo `/execute` is "a full
  agent and its own orchestrator" that **writes doctrine state directly** while
  running in a worktree that is `is_linked_worktree == true`.
- **Revealed.** There are **two** legitimate direct-writer classes on a linked
  worktree: the team-mode coordination root (D8) **and** solo `/execute` (D6a). Both
  satisfy `is_linked_worktree`; both are bricked by a stale
  `.doctrine/state/dispatch/worker` (gitignored, never swept by branch ops). The
  gate names only the first. A solo `/execute` that reuses a dir which once held a
  worker fork (gc skipped/failed) inherits the marker → `worker_mode` true → **solo
  writes refused**, with no gate run on its behalf and (per Charge II) no clearing
  verb.
- **Sentencing.** Wire assert-marker-absent (and the Charge-II clear path) for
  **every** transition of a linked worktree into a direct-writer role — solo
  `/execute` included — not only coordination-root promotion. The "legitimate
  writer" the remediation protects must be defined by *write-mode*, not by the word
  "coordination."

### CHARGE VII — Refused-then-re-dispatched forks carry no receipt and require `--force` to gc; the `--force` reflex the receipt was minted to kill returns through the common path. **[MEDIUM]**

- **Doctrine violated.** First-tribunal Charge I's sentencing intent ("ashes
  scattered so no `--force` reflex takes root").
- **Evidence.** `design.md:201` — on `head-moved`, `import` refuses and "the
  orchestrator **re-dispatches**." A refused import stamps **no** receipt
  (`design.md:216`, "on success"). `design.md:273` — "No receipt ⇒ refuse unless
  `--force`."
- **Revealed.** Moved-HEAD is, by the design's own framing (`design.md:264-265`),
  the **common** case. Each such re-dispatch abandons a fork that genuinely *is*
  spent (superseded) but bears **no receipt** — so reaping it demands `--force`.
  The operator, gc-ing superseded forks every batch, **relearns the `--force`
  reflex by routine**, and on the next fork that is merely *unreceipted-and-actually-
  live* (Charge I's crash window, or a mis-pointed `--fork`) the reflex fires and
  reaps live work. The receipt cleanly disposes the *landed* fork and quietly
  recreates the hazard for the *refused* fork — the more frequent one.
- **Sentencing.** Give superseded forks a **safe, non-`--force` disposition** — e.g.
  re-dispatch records the abandoned fork-head as `superseded`, and gc reaps on
  either a positive receipt **or** a superseded record. Reserve `--force` for the
  genuinely-unknown, so the reflex never becomes muscle.

### CHARGE VIII — "Transactional fork" overclaims: the rollback is itself non-atomic git mutation that can half-fail, and `git worktree remove` on a dirty provisioned dir refuses without `--force`. **[MEDIUM]**

- **Doctrine violated.** First-tribunal Charge VIII's remediation (transactional
  fork "so a partial fork never leaks an orphan"); honest reporting (don't claim
  success on a half-done irreversible act).
- **Evidence.** `design.md:78-81` — "**Transactional** — any failure after step 1
  rolls back (remove worktree, delete branch, reap target dir)." `design.md:91`
  (step 2) — provision **copies files** into the fork, so by rollback time the
  worktree is **dirty**. No `--force` is specified for the rollback `git worktree
  remove`; no rollback-failure semantics are defined.
- **Revealed.** The rollback is three git/disk mutations with no atomicity of their
  own. `git worktree remove` **refuses a dirty worktree without `--force`** — and a
  provisioned fork is always dirty — so the rollback's first act fails, leaving the
  orphan worktree **and** branch behind: precisely the Charge-VIII hazard, one level
  down. Even with `--force`, a remove-succeeds-then-branch-`-D`-fails interleaving
  leaves the "orphan branch with removed worktree" the handover names. The word
  "transactional" promises an atomicity git does not provide here.
- **Sentencing.** Drop the overclaim or earn it: rollback `git worktree remove
  --force`; make rollback **best-effort with explicit leftover diagnostics**; and on
  rollback failure **report the partial state by name and exit non-zero** — never a
  silent or success-coded half-rollback. Name it "compensating cleanup," not a
  "transaction."

### CHARGE IX — gc's receipt lookup key is underspecified: gc has only `--fork <branch>`, but the receipt is keyed `{base, fork-head}` — base is unsuppliable at gc. **[LOW]**

- **Evidence.** `design.md:243` — `gc --fork <branch> [--force]`. `design.md:271` /
  `design.md:216` — receipt keyed `{base, **fork-head**}`. gc takes **no `--base`**.
- **Revealed.** Given only `--fork <branch>`, gc must form the lookup by resolving
  the branch to its head SHA and matching on **fork-head alone** — in which case
  `base` is **stored and never read at the decision point** (the first tribunal's
  Charge XI, recommitted), *or* the lookup is by **branch name**, which a reused
  branch name (post-gc recreation) can **false-match**. Either reading is a defect:
  dead key-state, or an unsound name-based match.
- **Sentencing.** Specify the lookup precisely. If fork-head alone decides, drop
  `base` from the key (or justify it for IMP-043) — do not write state no gate
  reads. If base participates, give gc a way to supply it.

### CHARGE X — The receipt has no observability surface; the operator cannot see what is reapable, and defaults to `--force`. **[LOW]**

- **Doctrine violated.** Symmetry with the remediation's own move — DC-2 grew a
  required CLI worker-mode surface (`design.md:159-162`) precisely so a gitignored
  signal is *discoverable*. The receipt got no such courtesy.
- **Evidence.** `design.md:159-162` adds a `worker_mode` status line; no equivalent
  exists for receipts.
- **Revealed.** An operator facing `gc`'s "no receipt ⇒ refuse" cannot *see* which
  forks bear receipts without reading the gitignored store by hand → reaches for
  `--force` (compounding Charge VII).
- **Sentencing.** Surface receipt status in `doctrine worktree` / gc dry-run output
  ("fork `<b>`: receipted ✓ / unreceipted — `--force` to reap"), as the worker-mode
  surface was added for its signal.

### CHARGE XI — The `worker_mode` **env** leg is location-unqualified; a leaked `DOCTRINE_WORKER` bricks main-side authoring *and* the orchestrator's own funnel, aborting the dispatch it serves. **[HIGH]** *(raised by the User on the re-pass)*

- **Doctrine violated.** ADR-006 D6a (the **mode**, not the location, decides — but
  the env leg makes *process inheritance*, not mode, decide); compartmentalised
  state with a bounded blast radius; the remediation's own asymmetry — the marker
  leg is location-qualified, the env leg is not.
- **Evidence.** `design.md:137` — `worker_mode := env DOCTRINE_WORKER set OR
  (is_linked_worktree && marker_present)`. The env disjunct carries **no location
  predicate**: any process that inherited the var trips it, on main or anywhere,
  with or without a worktree. SPEC-012 FR-004/D3 (the **stale** pre-SL-056 text the
  User read) keys solely on `DOCTRINE_WORKER=1`; G4 rewrites it to the dual signal,
  but the env leg's unqualified nature **survives the rewrite**.
- **Revealed.** The env leg's location-blindness is *deliberate* — it is the only
  signal that catches the worker-on-main hazard (first-tribunal Charge III), so it
  **must** fire regardless of location. But that same blindness is a fail-closed
  landmine: env vars inherit into children and leak across a shell session. Stage
  the leak the design's own prose invites — `env DOCTRINE_WORKER=1 $fork_env claude
  -p …` (`design.md:107`) run in the live orchestrator shell, a stale export, a
  re-sourced session. Now **every** doctrine process in that session reads the var:
  a concurrent agent *planning future slices on main* has its `slice new` / `design`
  / `plan` writes **refused**, and — worse — the **orchestrator itself**, sharing
  the env, can no longer run its `Orchestrator`-classed `import`/`gc` or commit the
  funnel. A leaked env does not abort one worker; it **aborts the entire dispatch by
  bricking the writer at its head**, while masquerading as a safety refusal. The
  remediation added a *qualified* marker leg but never *bounded* the *unqualified*
  env leg, so env hygiene is now both load-bearing (worker-on-main, Charge III) and
  uncontained (this charge) — and unvalidated (Charge III) atop it.
- **Risk.** Session-scoped env pollution silently fail-closes legitimate main-side
  authoring and self-aborts dispatch; the failure presents as a correct guard
  refusal, so the operator mistakes a bricking for enforcement.
- **Sentencing.** Bound the env leg's blast radius without losing the worker-on-main
  catch: (a) set `DOCTRINE_WORKER` **only** in the spawned child's env (never
  `export` into the orchestrator's shell), and pin this in the spawn prose as a hard
  rule, not an example; (b) consider a **negative assertion** on the trusted side —
  the orchestrator asserts its *own* env is clean (`DOCTRINE_WORKER` unset) before
  running any funnel verb, failing loud if polluted, so a leak surfaces as a named
  error rather than a silent dispatch-abort; (c) the existing test discipline
  (`mem.pattern.dispatch.worker-verify-unset-doctrine-worker`) must extend to the
  orchestrator path. *The brand that marks the slave must not be left smouldering
  where the master will grasp it.*

### CHARGE XII — Stationary-only import (A2 struck) collides with doctrine's *expected* concurrent main-side authoring; solo-on-main dispatch livelocks on `head-moved` re-dispatch. **[HIGH]** *(raised by the User on the re-pass)*

- **Doctrine violated.** `mem.system.coordination.concurrent-design-shared-main-worktree`
  — "concurrent design work on the shared main worktree is **expected**"; ADR-006 D8
  (coordination branch placement); the slice's own claim (`design.md:230`) that
  re-dispatch is "truthful and shippable."
- **Evidence.** `design.md:200-201` — import precond `HEAD == B`; `HEAD != B` →
  refuse `head-moved` → "the orchestrator **re-dispatches** from the moved HEAD (no
  in-verb re-anchor in v1)." A2 struck (`slice-056.md:227`) → there is **no**
  re-anchor path in v1; every HEAD move is a full re-dispatch.
- **Revealed.** Workers fork from base `B` = the coordination HEAD at spawn. In solo
  mode the coordination branch **is main**. A concurrent agent planning future
  slices on main — the *expected* condition the memory records — commits
  (`slice new` mints + commits) → main HEAD → `B+1`. Now **every** in-flight
  worker's `import` refuses `head-moved`, and the orchestrator must re-dispatch the
  whole batch from `B+1`. The next planning commit invalidates *that* batch too:
  **livelock under ordinary, expected activity.** The funnel silently assumes a
  *quiescent* coordination branch; doctrine's actual practice on main is not
  quiescent. D8 offers the escape — coordination on a **delta branch** (team mode)
  isolates the funnel from main churn — but the design **never states "dispatch
  requires a quiescent coordination branch (use a delta branch if main is live)"**
  as a constraint, and A2-struck makes every collision maximally expensive
  (re-dispatch, not the cheaper re-anchor IMP-043 defers).
- **Risk.** A dispatch run on a live main starves; the operator either freezes all
  main-side authoring for the dispatch's duration (unscalable, contradicts the
  expected-concurrency memory) or learns dispatch "doesn't work here." Both
  tribunals accepted "re-dispatch is shippable" without testing it against expected
  concurrent main activity.
- **Sentencing.** Make the quiescence requirement **explicit and enforced**: state
  in the design (and SPEC-012 G4) that v1 dispatch requires a coordination branch
  with no concurrent external committers, and that a live main mandates a
  **delta-branch coordination** (D8 team mode). Optionally have the orchestrator
  **detect** a moving coordination HEAD early (the branch-point guard already exists)
  and **report the external mover by name** rather than silently re-dispatching into
  a livelock. The moved-HEAD re-anchor (IMP-043) is the real fix; until it lands, the
  constraint must be named, not assumed. *Do not send the harvesters into a field
  the lord still ploughs.*

### CHARGE XIII — The keystone spawn backend (`claude -p`) is harness-specific AND API-billed; the subprocess seam is unusable for the dominant harness, collapsing DC-2 and re-opening first-tribunal Charge III for claude. **[CRITICAL]** *(raised by the User on the re-pass)*

- **Doctrine violated.** The project's foundational aim — **harness-agnostic**
  (works for claude / codex / pi by construction; `design.md:8-11`, slice-056.md
  thesis); ADR-006 D1 (framework neutrality); first-tribunal Charge III's
  remediation, which rests entirely on this seam.
- **Evidence.** slice-056.md:18-27 — the "keystone insight": *"If the orchestrator
  owns fork creation and spawns workers as subprocesses (`claude -p` / `codex
  exec`), it gains all three"* (env-arm, per-wt `CARGO_TARGET_DIR`, bwrap). DC-2
  (`design.md:33-35`) — the env signal is reliable *"because"* the orchestrator owns
  the subprocess spawn. DC-1 (`design.md:21-28`) and OQ-4 (`design.md:393-395`)
  confine the harness-specific invocation to "thin prose," `claude -p` / `codex
  exec` treated as interchangeable subprocess backends. slice-056.md:38 — *"`claude
  -p` exists in every jail by construction, so the harness-agnostic core has live
  customers."*
- **Revealed under cross-examination (the User's testimony, accepted as fact —
  no verification sought).** `claude -p` is **billed at Anthropic Console API
  rates, not the interactive subscription** — so fanning out N short-lived workers
  via `claude -p` is **economically prohibitive**, and it is moreover a
  **harness-specific command** with no place required in a harness-agnostic skill.
  The design's two backends are **not** interchangeable: a *usable, free,
  env-seamed* subprocess spawn exists for local/subscription-neutral harnesses
  (`codex exec`, pi's depth-2 self-subagent) but **not** for claude. For the claude
  harness the only viable rung is the in-session **`Agent` tool** — the very rung
  `design.md:37-38` dismisses as "degraded," **precisely because it exposes no env
  seam.** Therefore, for claude:
  - the env-arm is **unavailable** → DC-2's env leg cannot be set → `worker_mode`
    degrades to **marker-only** → **worker-on-main (first-tribunal Charge III) is
    fail-open again**, on the dominant harness, undoing the capital remediation;
  - the per-worktree `CARGO_TARGET_DIR` via spawn (D5/O6) and the bwrap wrap (D6/O7)
    — both *also* justified by "the orchestrator owns the spawn" — **likewise
    evaporate**;
  - slice-056.md:38's "live customers" claim is false: `claude -p` *existing* ≠
    *usable*. The harness-agnostic core's flagship customer cannot afford the seam.
- **Risk.** The slice's central promise — *invert fail-open → fail-closed via the
  spawn seam* — holds only on harnesses the slice does not name as primary, and
  **inverts to no improvement (or a regression to marker-only) on claude.** The
  enforcement-altitude table (slice-056.md:54-58) silently assumes the CLI/OS rungs
  are reachable; for claude they are gated behind an API-billed door.
- **Sentencing.** The keystone thesis must be **re-examined, not patched**
  (`/consult`-grade — it touches OQ-1 and ADR-011/G3):
  1. **Strike the "`Agent` is the degraded rung" framing.** For claude, `Agent` is
     the *only economically viable* rung; the design must treat it as a first-class
     backend, not a fallback — and confront that it has **no env seam**.
  2. **Find a subscription-covered, env-bearing claude spawn**, or concede there is
     none. Candidate: the harness-native `WorktreeCreate` hook / `.worktreeinclude`
     path (already in ADR-006 D9) may carry env or marker without `claude -p`; if so,
     the marker leg — *disk, not env* — must be the **primary** worker-identity
     signal for claude, with env demoted to a codex/pi optimisation. This re-opens
     the first tribunal's Charge III adjudication: env-as-worker-on-main-catch is a
     **codex/pi-only** guarantee, not universal.
  3. **No harness-specific command (`claude -p`) may be a *required* element of a
     skill.** ADR-011/G3 records a harness-agnostic *contract* (orchestrator
     provisions + arms + spawns; the binary emits the env block); each harness's
     concrete spawn — including "claude uses the `Agent` tool, env-armed via marker
     not export" — is a per-harness *template*, and at least one harness's template
     must not depend on a billed subprocess.
  4. **SPEC-012 / ADR-011 must state the achievable enforcement altitude *per
     harness*** — claude: marker-only (no env, no spawn-set target, no bwrap) unless
     a free env seam is found; codex/pi: full. The slice cannot claim a uniform
     fail-closed inversion it delivers only off-claude.
  *The cathedral built on a single quarry's stone is no cathedral when that quarry
  bills by the cartload. Re-survey the ground before you raise the keystone.*

---

## Counts on which the accused is ACQUITTED

The Inquisition is fanatical, not unjust. Three counts survive the re-pass:

- **Receipt vs branch-point-check is no contradiction (handover seam 5).** They gate
  **different verbs**: `import` gates on branch-point (`HEAD == B`), `gc` gates on
  the receipt. The receipt was minted *precisely* to decouple gc from
  HEAD-stationarity so a sibling's legitimate HEAD move does not false-refuse a
  spent fork (first-tribunal Charge I). On *this* axis the design is **sound** — the
  decoupling is the point, not a defect. (The receipt's *timing* is the heresy —
  Charge I above — not its independence from branch-point.)
- **The orchestrator is never env-worker-id'd at call time.** The orchestrator is
  the top-level process; it exports `DOCTRINE_WORKER=1` into the **child**
  (`env DOCTRINE_WORKER=1 … claude -p`), never into its own environment. So the
  *env* leg never bricks the orchestrator's `Orchestrator`-classed calls. (The
  *marker* leg can — Charge II — but that is a distinct mechanism, distinctly
  indicted.)
- **The pure/imperative wall still holds.** `target_dir_for_branch`,
  `marker_path`, `classify_import` take their inputs as parameters; the new
  receipt-read/write and rollback impurity sits in the `run_*` shells and `git.rs`.
  No clock, rng, disk, or git crosses a pure signature
  (`mem.pattern.architecture.info-flow-wall-at-signature`). On this count, as on the
  first pass — **not guilty.**

---

## 2. Questions (interrogatories)

1. **Receipt timing (Charge I):** Is the receipt stamped before or after the
   coordination **commit**? If before, by what right does gc call an applied-but-
   uncommitted, gitignored-tier flag "provably landed," against D7's
   rebuild-from-the-branch recovery model?
2. **Coordination-tree marker (Charge II):** Name the verb that clears a stale
   marker from a tree the operator wants as coordination root — given `gc` is
   `Orchestrator`-classed and refused while that marker is present.
3. **Env propagation (Charge III):** Has any experiment confirmed `claude -p` /
   `codex exec` / pi deliver an orchestrator-set `DOCTRINE_WORKER` to the worker's
   doctrine process? If not, on what does the worker-on-main guard rest until the
   spike runs?
4. **Receipt lifecycle (Charge IV):** Who deletes a receipt, and when?
5. **import exhaustiveness (Charge V):** What refuses a *dirty* coordination tree,
   and if nothing does, how is `apply-conflict` unreachable?
6. **Solo writer (Charge VI):** Does solo `/execute` run assert-marker-absent on its
   own fork before writing, or is it bricked by a stale marker like the coordination
   root?
7. **Superseded forks (Charge VII):** How is a refused-then-re-dispatched fork reaped
   without training the `--force` reflex?
8. **gc lookup key (Charge IX):** Does gc match the receipt by fork-head SHA or by
   branch name, and is `base` read at the decision?

---

## 3. Pronouncement of Judgement

**The remediation was true penance for the first eleven sins — and it bred new
ones at the welds.** **Three capital defects stand**, and the third is the gravest:
the keystone spawn backend (`claude -p`) is harness-specific and **API-billed**, so
the subprocess seam on which the *entire slice* rests is **unusable for the dominant
harness** (Charge XIII, the User's) — collapsing DC-2's env leg to marker-only for
claude and thereby **re-opening the first tribunal's capital Charge III** on the very
harness the slice most serves; the slice's "fail-open → fail-closed inversion" is
delivered only off-claude. Beside it: the landed-oracle now certifies the *apply*
and trusts disposable state across a crash (Charge I), reopening the
reap-unmerged-work hazard the receipt was minted to close; and the marker's removal
owner is locked behind the marker's own guard (Charge II), so a stray marker on the
team-mode coordination tree self-bricks with no in-CLI escape — Charge V of the
first tribunal, recommitted with the gaoler's key thrown down the well. Five HIGH
charges follow: the env leg of DC-2 is unproven faith (III), the receipt is itself
*res derelicta* (IV), `import`'s "exhaustive" refusal set is exhaustive only over
the states its unchecked precond assumes (V), the env leg's **unqualified** blast
radius lets a leaked var brick main-side authoring and self-abort the dispatch
(XI, raised by the User), and the stationary-only import **livelocks** against
doctrine's *expected* concurrent main-side authoring (XII, raised by the User). The
lesser charges (VI–X) are the seams the handover named, each confirmed. Thirteen
charges, two tribunals' worth of smoke.

**The design does NOT yet earn** *nihil obstat.* **It must not proceed to `/plan`
until Charges I and II are remediated and the design re-locked.** III–V should land
in the same pass; VI–X are tractable housekeeping. Three counts are righteous and
stand acquitted — let the record show the receipt's *independence* is sound even as
its *timing* burns.

---

## 4. Sentencing (ordered penance, with verification and just punishment)

0. **Charge XIII (CRITICAL) — re-survey the keystone before any other stone.** This
   is `/consult`-grade and precedes all else, for it may rescope the slice: strike
   "`Agent` is the degraded rung"; decide claude's worker-identity signal without a
   billed subprocess (marker-primary, env a codex/pi optimisation); forbid any
   harness-specific command as a *required* skill element; state the achievable
   enforcement altitude **per harness**. *Verify:* a claude dispatch arms
   worker-identity and isolates builds **without** `claude -p`, or the design openly
   confesses claude tops out at marker-only. *Punishment:* **the cathedral's keystone
   struck out and the arch re-surveyed** — no vault raised on a quarry that bills by
   the cartload.

1. **Charge I (CRITICAL) — re-anchor the receipt to the commit.** Stamp on/after the
   coordination commit (key it to the commit sha), or have gc verify branch
   reachability rather than a runtime-tier flag. *Verify:* a golden where the
   orchestrator imports, then **crashes before commit**; on recovery, gc **refuses**
   to reap the fork (no committed delta) — the receipt alone does not license `-D`.
   *Punishment:* **the false relic cast into the same fire as the delta-oracle it
   replaced**, that no flag outlive the commit it pretends to certify.

2. **Charge II (CRITICAL) — give the marker a key the guard cannot strangle.** A
   non-`Orchestrator`, non-write `worktree marker --clear` (gated only by
   cwd-is-this-tree), or a remediating assert-marker-absent that clears + reports.
   *Verify:* a stale marker on a team-mode linked-worktree coordination root is
   cleared **from within the CLI**, restoring writes and gc — no filesystem surgery.
   *Punishment:* **the gaoler's vigil in irons at his own gate**, until a key hangs
   there that opens it from inside.

3. **Charge III (HIGH) — spike the env propagation, not just the guard logic.** Add
   a real-harness propagation gate to the O3 spike; keep ADR-011's env-reliability
   claim `proposed` until green. *Verify:* a spawned `claude -p` worker's doctrine
   process reads the orchestrator-set `DOCTRINE_WORKER`. *Punishment:* **recantation
   in the chapter-house** — reliance proclaimed before it is proven is faith without
   works, the twin of Charge IX.

4. **Charge IV (HIGH) — reap the receipt in gc;** **Charge V (HIGH) — name the
   `tree-unclean` refusal (or keep `apply-conflict`).** *Verify:* gc leaves no
   dangling receipt; a dirty-tree import refuses with a named reason and a golden.
   *Punishment:* **the pillory** for state left derelict; **the strappado** for the
   exhaustiveness lie, until clean-and-`HEAD==B` are *both* guarded or *both*
   confessed as assumptions.

5. **Charges VI–X — the named seams.** Wire the marker-absent/clear gate for solo
   `/execute` (VI); give superseded forks a non-`--force` disposition (VII); rollback
   `--force` + best-effort + honest exit (VIII); pin gc's lookup key (IX); surface
   receipt status (X). *Verify:* solo reuse of a marked dir does not brick; a
   re-dispatch leaves a reapable-without-`--force` fork; a half-failed rollback exits
   non-zero with the leftover named; a reused branch name cannot false-match a
   receipt; `gc` shows receipt status. *Punishment:* **a day in the stocks apiece** —
   public, that each seam be shamed into closure.

6. **Charge XI (HIGH) — bound the env leg's blast radius.** Set `DOCTRINE_WORKER`
   only in the child env (never `export` into the orchestrator shell — pin it as a
   rule, not an example); assert the orchestrator's own env is clean before any
   funnel verb, failing loud on pollution. *Verify:* a leaked `DOCTRINE_WORKER` in
   the orchestrator session surfaces as a **named error**, not a silent
   dispatch-abort; main-side authoring is unaffected by a worker spawn. *Punishment:*
   **the brand quenched** before the master grasps it bare-handed.

7. **Charge XII (HIGH) — name the quiescence constraint.** State in the design and
   SPEC-012 (G4) that v1 dispatch requires a coordination branch with no concurrent
   external committers; a live main mandates delta-branch coordination (D8). Have the
   orchestrator report an external HEAD mover by name rather than livelock into
   blind re-dispatch. *Verify:* a concurrent main-side `slice new` during dispatch
   yields a named "external mover" report, not silent batch invalidation; the
   delta-branch path is documented. *Punishment:* **the harvesters recalled from the
   ploughed field** until the lord's furrow is named and fenced.

Re-lock after 0–2 (3–7 in the same pass). Charge XIII (item 0) gates the rest — it
may rescope the slice, so resolve it first. Then — and only then — a **third**
tribunal may grant *nihil obstat* and the slice proceed to `/plan`.

> *Two tribunals now. The first burned the gross heresy; the second finds the welds
> still smoking. The receipt that certifies an apply is a relic that certifies a
> rumour; the guard that locks away its own key is a gate that imprisons the keeper.
> Mend the mechanism at the seams the mending opened — and the doctrine endures.*
>
> **HERESIS URITOR; DOCTRINA MANET**
