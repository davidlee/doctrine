# Inquisition (Third Tribunal) — SL-056 Design — confirmatory *nihil obstat* re-pass

*Tribunal reconvened 2026-06-13 to pronounce upon the* nihil obstat. *Target:
`design.md` at status `design`, re-locked after the second tribunal's thirteen
charges (commits `db10887`, `afcc186`, `8086d7d`). The Inquisitor came as the
handover commanded — **fresh adversarial eyes, charged to put fire to the welds
the second remediation opened**, not to re-read the prose the accused has learned
to recite. Witnesses re-cross-examined through the CLI, never the raw file:
`adr show 6` (D2/D2b/D7/D8/D9), `memory show mem_019ebed8…` (the landed-oracle
relic), the working tree's own untracked state.*

*The second tribunal sang true: **penance breeds new sin at the welds.** The
accused has learned the verse but not the lesson. The receipt was burned for being
a disposable, crash-surviving flag that gates an irreversible `-D` — and in the
very same breath a **second flag of the identical species** was set beside its
ashes to answer Charge VII. The marker was given a key (`marker --clear`) to
escape its own dungeon — and that key was cut to fit the hand of the one prisoner
it was meant to hold. The accused does **NOT** earn* nihil obstat.

> **HERESIS URITOR; DOCTRINA MANET**

---

## 1. Charges

### CHARGE A — The `superseded` record is the receipt risen from its own ashes: a disposable, crash-surviving runtime flag that licenses an irreversible `git branch -D`. The "fail-safe" defence answers only false-*absence* and is silent on the fatal failure — false-*presence*. **[CRITICAL]**

- **Doctrine violated.** The second tribunal's Charge I sentence — *"A landed-oracle
  keyed on disposable state is no oracle… the false relic to the same flames."*
  ADR-006 **D7**: "the coordination branch **is the durable store**… recovery is
  **rebuild-from-coordination-branch**… knowledge always **trails confirmed code**."
- **Evidence.** `design.md:406-414` (D4, "Superseded forks"): re-dispatch "**records
  the abandoned fork-head as `superseded`** (a small runtime record), and gc reaps
  on **patch-id landed OR a `superseded` record**." `design.md:413` pronounces it
  "**fail-safe**… its *absence* only costs a `--force`, never an erroneous reap
  (Charge I's hazard does not recur)."
- **Revealed under cross-examination.** The fail-safety proof is a sleight of one
  direction. It proves only that a *missing* record costs a `--force` — the benign
  side. It says **nothing** of an *erroneously present* record, and that is the
  whole hazard. Where a `superseded` record exists, gc reaps **without** patch-id
  landing **and without** `--force` (`design.md:413-414`). A `superseded` record
  written for a fork that is in truth **live** is therefore a **silent, un-`--force`d
  `git branch -D` of unmerged work** — the *exact* hazard for which the receipt was
  burned, recommitted one object over. And the record is born wrong on the design's
  own most-common path:
  1. **The crash window the patch-id oracle was praised for closing, reopened.**
     `design.md:398` boasts the patch-id check is "**crash-proof**." Its companion is
     not. Re-dispatch is triggered by an `import` `head-moved` refusal
     (`design.md:300-301`); the orchestrator writes the `superseded` record, then
     spawns the replacement. **Crash between record and respawn** — or a worker that
     dies before producing its replacement delta — leaves the original fork as the
     **only surviving copy** of that work, now bearing a `superseded` mark, awaiting
     a recovery-time gc that reaps it un-`--force`d. The patch-id oracle is
     crash-proof; the OR-clause beside it restores the crash hole verbatim.
  2. **Res derelicta, recommitted (the buried second-tribunal Charge IV).** The
     record has **no removal owner**. D4's gc reaping acts (`design.md:357-367`)
     enumerate worktree, branch, target dir — and **omit the `superseded` record**,
     exactly as the eliminated receipt was omitted. After gc the record **dangles**.
  3. **Name-reuse false-match, recommitted (the buried second-tribunal Charge IX).**
     `fork` refuses an existing branch name but a name is **freed by `gc`'s
     `branch -D`** (`design.md:359`). A dangling `superseded` record keyed by
     fork-branch-name, plus a later fork that **reuses** that freed name, yields a
     **false `superseded` match** → the new, live fork reaped un-`--force`d. The
     patch-id oracle was lauded for being immune to name-reuse (`design.md`/Charge IX
     disposal); the `superseded` OR-clause drags the name-reuse hazard back in
     through the side door.
  The accused eliminated the receipt to abolish Charges I/IV/IX, then built a
  functionally identical runtime flag — *unkeyed lifecycle, name-matchable,
  crash-surviving, gating `-D`* — to answer Charge VII, and re-incurred all three.
- **Risk.** Irreversible reap of unmerged work on the crash-recovery and
  name-reuse paths — the single hazard `gc` exists to prevent — gated on the very
  tier of state the design's own Charge I disclaims.
- **Sentencing.** The `superseded` disposition must be made of the same durable
  stuff as the patch-id oracle or not exist. Options that do not re-light the fire:
  (a) derive "superseded" from **durable git** — a tag or note on the abandoned
  fork-head, written **after** the re-dispatch's replacement has itself landed, so
  no flag outlives the work it dooms; (b) reap superseded forks by **operator-named
  fork-head SHA** at gc, never by an ambient stored record; or (c) accept the
  `--force` for superseded forks and answer Charge VII's reflex with **observability
  alone** (the dry-run verdict already shows "not-landed") rather than a second
  oracle. Whichever is chosen, the record needs a **removal owner in gc** and a
  **collision-proof key** (fork-head SHA, never branch name) — or it is Charge IV and
  Charge IX, exhumed. *The relic you burned is risen in its brother's vestments;
  cast the brother to the same fire, that no flag outlive the commit it pretends to
  certify.*

### CHARGE B — `marker --clear` cuts the prisoner his own key: a **claude** worker, lacking the env-lock that fences codex/pi, can self-unmark and restore to itself **both** worker-write **and** the entire `Orchestrator` verb class — re-opening first-tribunal Charges III *and* IV on the dominant harness. The cited container (bwrap) does not exist for claude. **[CRITICAL]**

- **Doctrine violated.** The slice's foundational thesis — invert **fail-open →
  fail-closed**, "only the prompt can self-arm, **nothing enforces it**" is the sin
  to be slain (`design.md:174-176`). ADR-006 D2 (worker-sole-writer); DC-3 /
  second-tribunal Charge IV (the `Orchestrator` class refused under worker identity).
- **Evidence.** `design.md:218-224` — `marker --clear` "is **refused only by
  `DOCTRINE_WORKER`-env-set**… or if cwd is not the marker's tree root; **not**
  refused by the marker conjunct." `design.md:86-89` (DC-3) — "**A claude
  marker-only worker self-clearing is the accepted D2b residual:** the guard fences
  accidents/misplacement, not malice — **that is D6/bwrap**." `design.md:443-447`
  (D6) — bwrap is "**Subprocess-only (Charge XIII)… codex/pi-only**; for claude, D2b
  stays the deferred residual." `design.md:185-187` — `worker_mode := (is_linked &&
  marker) OR env`; clearing the marker with no env set ⇒ `worker_mode` **false**.
- **Revealed.** Trace the dominant harness. A claude worker carries a **marker and no
  env** (claude has no env seam — Charge XIII, `design.md:159-161`). It invokes two
  CLI verbs in its own worktree:
  1. `doctrine worktree marker --clear` — **not** refused: `DOCTRINE_WORKER` is unset
     (no env seam), cwd **is** the tree root, and the marker conjunct is barred from
     refusing by design (`design.md:223-224`). The marker is gone.
  2. any write or `Orchestrator` verb — `worker_mode` is now **false** → **allowed.**
  The worker has restored to itself not merely authored writes but the **entire
  `Orchestrator` class** — `fork`, `import`, `gc --force` — the precise verbs DC-3
  and second-tribunal Charge IV were minted to deny it. The off-switch the marker
  was supposed to *be* is now a switch **the worker's own hand can throw.** For
  codex/pi this is locked: the worker's env is set ⇒ `marker --clear` refused ⇒ no
  self-unmark. **The protection against self-clearing IS the env leg — the one leg
  claude does not have.** The design's stated container for the residual — "that is
  D6/bwrap" — is **void for claude by the design's own D6** (`design.md:444`). So on
  the named primary harness the worker-sole-writer guarantee rests, after
  `marker --clear`, on **nothing but the worker choosing not to press the button** —
  i.e. on prompt compliance: the *exact* fail-open the slice exists to abolish. The
  R-5 import belt is no rescue here: it guards the *coordination* tree from an
  imported `.doctrine/` delta, but a self-unmarked worker wielding `gc --force` or
  `fork` acts on **git refs directly**, outside any belt.
- **Risk.** The slice's central promise — fail-closed worker-sole-writer — is
  **undelivered for claude**, the harness it most serves; enforcement reduces to
  prompt-trust with a worker-accessible self-destruct and no OS backstop. The design
  scopes its own guard to "accidents, not malice" and points malice at a bwrap that,
  for claude, does not exist — an enforcement floor **strictly below the design's own
  stated minimum** for the named primary harness.
- **Sentencing.** Close the self-unmark for the harness that has no env-lock and no
  bwrap. Candidates: (a) gate `marker --clear` on **`is_linked_worktree == false`**
  (a worker, always in a linked worktree, can never self-clear; the legitimate
  clearer is the coordination root or a deliberately-promoted tree — Charge II's
  cases — which the operator runs from the *parent* tree, not a linked fork); (b)
  require an out-of-band orchestrator token the worker prompt never carries; or (c)
  confess plainly in G3/ADR-011's altitude table that **claude worker-sole-writer is
  prompt-enforced, not mechanism-enforced** — and stop claiming the fail-closed
  inversion for claude. The honest confession (Charge XIII's whole spirit) is
  preferable to a guard with a worker-side key. *The gaoler who cut the key to fit
  the prisoner's hand is no gaoler; brick the wicket or confess the cell stands open.*

### CHARGE C — The claude marker-via-hook keystone is **over-broad and unfallbacked**: the `WorktreeCreate` hook (ADR-006 D9) fires on *every* Agent worktree, not only dispatch workers, and carries no `--worker` discriminator — so it brands non-dispatch isolated worktrees and bricks their legitimate writes; and the entire claude identity story has **no specified contingency** if the O3 spike refutes the hook. **[HIGH]**

- **Doctrine violated.** ADR-006 **D9** — the `WorktreeCreate` hook is the **general**
  creation rung (any worktree), fed by a `.worktreeinclude` allowlist that
  "**excludes the coordination/runtime tier** (`.doctrine/state/`)." DC-1/DC-2 — the
  claude marker is "stamped by the **WorktreeCreate hook**" (`design.md:38-42`,
  `159-161`). First-tribunal Charge III ("spike what you **rely on**, not only what
  you fear"), applied recursively.
- **Evidence.** `design.md:38-42` + `478-481` (G2) — claude identity is the
  hook-stamped marker; the orchestrator "configures the WorktreeCreate hook… to
  provision + stamp the marker." `design.md:550-555` defers proof to the O3 spike.
  ADR-006 D9 (`adr show 6`) — the hook is the harness's **native worktree tool**,
  fired on *any* `WorktreeCreate`, and its allowlist **excludes `.doctrine/state/`**
  — the marker's own tier (`design.md:184`).
- **Revealed.** Three holes at this weld:
  1. **Over-broad branding.** A claude hook is **session/global configuration**, not
     a per-spawn argument. It fires on **every** Agent worktree creation — including
     a parallel, non-dispatch Agent task that merely uses `isolation:worktree`. The
     codex/pi path stamps the marker **deliberately, per worker**, via
     `fork --worker` (`design.md:128-130`); the hook path has **no `--worker`
     discriminator** and stamps **unconditionally**. A non-dispatch isolated Agent
     thereby inherits the marker → `worker_mode` true → its **legitimate writes
     refused** — a false-positive brick the codex/pi path cannot suffer. The design
     never confronts that the hook cannot tell a dispatch worktree from any other.
  2. **Tier incoherence.** D9's provisioning allowlist **excludes** `.doctrine/state/`
     — yet the same hook must **write the marker into** `.doctrine/state/dispatch/
     worker`. The design fuses two opposite tier-relationships ("provisions **+**
     stamps the marker", `design.md:159-161`) under one hook without acknowledging
     that stamping writes into the exact tier provisioning is sworn to exclude.
  3. **No fallback if the spike refutes the keystone.** `design.md:550-555` gates the
     hook on the O3 spike — correctly symmetric with Charge IX. But D6's bwrap spike
     has a stated back-out ("Too costly → back out to D5 + the D2 marker guard",
     `design.md:453`); the **marker-via-hook spike has none.** If the spike finds the
     hook does not fire on `isolation:worktree`, or cannot stamp before the worker's
     first write, claude has **no worker-identity mechanism at all** — no env (Charge
     XIII), no hook — and the slice's primary-harness promise is void, with the design
     silent on Plan C. A keystone deferred to an unrun spike with no contingency is a
     keystone resting on faith, the very posture round-1 Charge III condemned.
- **Risk.** False-positive bricking of legitimate non-dispatch Agent worktrees; a
  single-point-of-failure keystone for the dominant harness with no documented
  branch if its one spike fails.
- **Sentencing.** (a) Give the hook a discriminator — stamp only when the orchestrator
  signals a dispatch spawn (e.g. a sentinel the hook reads, set only for worker
  worktrees), so non-dispatch Agent worktrees are not branded; (b) state explicitly
  that marker-stamping is a **separate act** from allowlist provisioning and is
  *permitted* to write the excluded tier; (c) **name the fallback** the O3 spike
  failure triggers — symmetric with D6's back-out — or escalate the unproven hook to
  a `/consult`-grade blocker before `/plan`, as Charge XIII's keystone was.

### CHARGE D — The Charge XI remediation guards only the **funnel** verbs; the leaked-env-bricks-*authoring* hazard it was raised to answer still presents as a bare "worker refused", not the named "env polluted" error. The cure was applied to the wrong door. **[HIGH]**

- **Doctrine violated.** Second-tribunal Charge XI's own premise — a leaked env
  "**bricks main-side authoring**… while masquerading as a safety refusal"; the
  remediation's promise to surface a leak "as a **named error** rather than a silent
  guard refusal."
- **Evidence.** `design.md:262-267` (and the resolution row, `:645`) — the
  orchestrator asserts its own env clean and fails with the named error **"before
  any `Orchestrator`-classed funnel verb."** `design.md:185-190` — `worker_mode`'s
  env disjunct trips on **any** write-classed verb, including the **authoring** verbs
  (`slice new` / `design` / `plan`), which are **not** `Orchestrator`-classed.
- **Revealed.** Charge XI was raised about a *concurrent agent planning future slices
  on main* whose `slice new`/`design`/`plan` are refused by a leaked
  `DOCTRINE_WORKER` (`inquisition-2.md:280-285`). Those are **write-classed authoring**
  verbs, not `Orchestrator` funnel verbs. The remediation's named-error assertion
  fires **only before `import`/`gc`** — the verbs the *orchestrator* runs — and **not
  at all** before the authoring verbs the *bricked concurrent agent* runs. So the
  exact victim Charge XI named still receives a bare **"worker fork — write
  refused"** on main-side authoring: the masquerade-as-enforcement the charge
  demanded be ended, **unended.** The cure was fitted to the orchestrator's funnel
  door; the wound is at the authoring door, still open. (The D2 observability surface,
  `:233-236`, "signal: env|marker", is a partial salve — a diligent operator *can*
  read "signal: env" and infer a leak — but the charge asked for a named error at the
  point of refusal, and the dominant authoring path delivers none.)
- **Risk.** The precise failure Charge XI indicted — session-scoped env pollution
  silently fail-closing legitimate main-side authoring, read as a correct guard
  refusal — survives the remediation on the authoring path it was raised about.
- **Sentencing.** Extend the negative env assertion (or the named-error
  classification) to the **write-classed authoring** path, not only the
  `Orchestrator` funnel path: when `worker_mode` is tripped **by the env leg on a
  tree that is not a linked worktree** (i.e. on main, where a true worker cannot
  legitimately be), refuse with the **"env polluted, unset it"** named error, not the
  generic worker refusal. A leak on main is provably a leak (no fork marker, not a
  linked worktree); say so by name wherever it bricks a writer.

### CHARGE E — The gc landed-oracle and the import funnel are specified for the **single-commit dispatch fork**, but D4 conscripts gc as the **solo `/execute`** cleanup — and solo execute is multi-commit, cannot pass the single-non-merge `import` funnel, and breaks the oracle's singular "**the** fork commit." Two callers of one verb, two unreconciled commit shapes. **[HIGH]**

- **Doctrine violated.** ADR-006 D6a (solo `/execute` is a full self-orchestrator
  that writes directly); the design's claim (`design.md:416-417`) that "the caller of
  `fork` owns `gc`… solo `/execute` ends with it."
- **Evidence.** `design.md:292-293` — `import` asserts `S^ == B` (**single
  non-merge**) else refuses `multi-commit`. `design.md:388-392` — gc reaps when "**the
  fork commit** marked `-`" by `git cherry` — singular, a v1 single-commit
  assumption. `design.md:416-417` — solo `/execute` is named a caller of `gc`.
  `design.md:225-232` — solo `/execute` writes doctrine state **directly** while
  `is_linked_worktree` is true, i.e. it is a normal TDD phase: **red/green/refactor,
  multiple commits.**
- **Revealed.** The design tunes import and gc for the dispatch fork — **one** distilled
  commit — then reuses gc for solo `/execute`, whose work is the ordinary
  **multi-commit** TDD sequence. Two unanswered fractures:
  1. If solo work reaches coordination through the **same `import` funnel**, its
     multi-commit delta **fails `multi-commit`** (`design.md:293`) — the funnel
     refuses the solo path it is claimed to serve.
  2. If solo work reaches coordination by some **other** route (direct merge —
     unstated), then gc's "**the** fork commit marked `-`" is **underspecified**: a
     multi-commit fork needs **every** commit's patch-id `-` to be provably landed,
     yet the oracle names one. A tip-only check reaps a fork whose earlier commits
     never landed.
  The design never states which route solo takes, nor how the singular oracle ranges
  over a multi-commit fork. "The caller of `fork` owns `gc`" papers over a genuine
  shape mismatch between gc's two conscripted callers.
- **Risk.** Either the solo path cannot use the funnel it is told to use, or gc's
  landed-oracle silently under-checks a multi-commit fork and reaps partially-landed
  work — Charge A's hazard by a second route.
- **Sentencing.** State solo `/execute`'s path to coordination explicitly. If it uses
  `import`, lift the single-commit restriction for it (or squash before import) and
  define the oracle over the **set** of fork commits ("**all** commits `-`"). If it
  merges directly, say so and define gc's landed check for a multi-commit,
  possibly-merged branch. One verb, two callers — specify both shapes or split the
  verb.

### CHARGE F — `import`'s `tree-unclean` guard, as written (`git status --porcelain`-empty), **over-refuses on benign untracked files** — and the repository's *normal* working state carries exactly those. The remediation's own evidence convicts it. **[MEDIUM]**

- **Doctrine violated.** The remediation's claim (`design.md:285-296`, Charge V
  resolution) that the `tree-unclean` check correctly closes the `apply-conflict`
  gap; correctness-first (do not refuse importable trees).
- **Evidence.** `design.md:287-289` — precond step 1 adds "a separate **`git status
  --porcelain`-empty** check." `git status --porcelain` lists **untracked files**
  (`?? path`) in its output; an untracked file makes it **non-empty**. The working
  tree of *this very slice* (the session's `git status`) carries untracked paths —
  `.doctrine/slice/056/inquisition-2.md`, untracked memory items — none of which
  affect a `git apply` of a tracked-file delta.
- **Revealed.** `git apply --3way` cares only about the **tracked** content the patch
  touches; **untracked files cannot cause an apply conflict.** A `tree-unclean` guard
  that treats *any* `--porcelain` output as dirty will **refuse a perfectly
  importable coordination tree** merely because unrelated untracked files exist —
  and untracked files are the repo's **ordinary** state (memory items, gitignored
  scratch, the very inquisition sheets). The guard meant to close the
  `apply-conflict` gap instead manufactures a false `tree-unclean` refusal on the
  common case, re-incurring a softer form of Charge XII's livelock (dispatch
  "doesn't work here").
- **Risk.** Spurious `tree-unclean` refusals on a tree that would import cleanly;
  dispatch unusable wherever benign untracked files live — i.e. normally.
- **Sentencing.** Scope the cleanliness check to what `git apply` actually depends on:
  `git status --porcelain --untracked-files=no`, or a check restricted to **tracked
  modifications and staged changes** (the index and worktree of tracked paths). Pin
  it with a golden: an import that **succeeds** with untracked files present, and
  **refuses** `tree-unclean` on a tracked uncommitted modification.

### CHARGE G — The per-harness router (O8) names a discriminator it never defines: **how** `/dispatch` decides claude-vs-codex/pi is unspecified, and the shared funnel cadence is forked into two sub-skills with no factoring stated. Misdetection lands the wrong altitude. **[MEDIUM]**

- **Doctrine violated.** The slice thesis — harness-agnostic **by construction**;
  CLAUDE.md / project canon — **no parallel implementation** ("find duplication
  before writing").
- **Evidence.** `design.md:512` + D7 — "`/dispatch` becomes a **harness router** →
  `/dispatch-subprocess` (codex/pi) | `/dispatch-agent` (claude)." Handover seam O8
  poses the question; **nowhere** in `design.md` is the detection mechanism named —
  no env probe, no capability flag, no config key.
- **Revealed.** The entire DC-1/DC-2 per-harness split — which backend spawns, which
  identity signal, which altitude — hinges on routing claude to `/dispatch-agent` and
  codex/pi to `/dispatch-subprocess`. Yet the **discriminator is a black box.**
  Misdetection is not benign: claude routed to `/dispatch-subprocess` attempts
  `codex exec` / `claude -p` — the API-billed, harness-specific path Charge XIII
  spent its life excluding. A keystone routing decision with no defined input is the
  same faith-not-works posture as Charge III, one layer up. Compounding it: the
  shared agnostic cadence (the order `import → verify → branch-point → one commit →
  record`, the report-and-halt-on-conflict) lives in **skill prose**, now **split
  across two sub-skills**. The design does not say whether that cadence is factored
  (a shared parent the two specialise) or **duplicated** — and duplicated
  orchestration prose is precisely the parallel implementation canon forbids, and the
  drift surface (two funnel cadences diverging) the slice's own "mechanism in the
  CLI" thesis exists to prevent.
- **Risk.** Silent misdetection drives the wrong harness down the wrong altitude
  (API-billed or fail-open); a duplicated cadence drifts between the sub-skills.
- **Sentencing.** Name the detection mechanism (a capability profile the harness
  declares, an env/config probe, or an explicit `--harness` the operator passes) and
  its **misdetection failure mode** (refuse, do not guess). State that the agnostic
  cadence is **shared** (one parent skill or one CLI-driven sequence the two
  sub-skills call), not copied — and pin the no-duplication claim.

### CHARGE H — A durable memory now **contradicts the re-locked design**: `mem.pattern.dispatch.landed-oracle-needs-import-receipt` still prescribes the **eliminated** receipt as "the sound oracle," scoped to the very file and verb the gc implementer will open. Stale doctrine will mislead the work this design authorises. **[LOW]**

- **Doctrine violated.** CLAUDE.md memory discipline — "**delete memories that turn
  out to be wrong**"; the storage rule (no contradicted derived knowledge left
  standing); ADR-003 (decisions govern proven mechanism — and the superseded
  mechanism must not keep speaking).
- **Evidence.** `memory show mem_019ebed87aca…` — body: "**Sound oracle: an explicit
  import receipt.** `import` stamps a record keyed `{base, fork-head}`… `gc` deletes
  only on a positive receipt." Trust `medium`, **scope.paths `[src/worktree.rs]`**,
  scope.tags `[dispatch, worktree, gc]`, closing line "**Relevant when implementing
  `doctrine worktree gc`.**" The re-locked `design.md:384-400` (Charge I resolution)
  **eliminates the receipt** as unsound and replaces it with the `git cherry`
  patch-id oracle.
- **Revealed.** The memory is the **exact retrieval** a future agent gets when opening
  `src/worktree.rs` to implement `gc` — and it prescribes, with the authority of
  durable doctrine, the receipt the design has **condemned and burned.** A patch-id
  oracle and a receipt cannot both be "the sound oracle"; the memory is now a
  doctrine-vs-design conflict that will steer the implementer toward the ash. It was
  surfaced by the SL-056 inquisition itself (its own provenance line says so) — and
  the inquisition then moved past it without recanting it.
- **Risk.** The gc implementation built on the design's authorisation is misdirected
  by the highest-authority source it will consult for that exact file.
- **Sentencing.** Supersede or rewrite `mem_019ebed8` to record the **patch-id
  (`git cherry`) oracle** as the sound landed-check and the receipt as the *rejected*
  predecessor (it already correctly damns `--merged` and delta-emptiness — keep that;
  invert the conclusion). This is durable-knowledge hygiene owed **before** `/plan`,
  lest the plan inherit a memory that contradicts the design it plans.

---

## Grading of the round-2 resolution claims (the accused's own scorecard, re-tried)

| Round-2 charge | Design's resolution claim | Third-tribunal verdict |
|---|---|---|
| I (receipt timing) | Receipt eliminated; gc gates on durable `git cherry` patch-id — crash-proof | **Patch-id oracle UPHELD** (sound, crash-proof) — **but undone by Charge A**: the `superseded` OR-clause beside it restores the disposable-flag crash/name-reuse hazard. |
| II (`marker --clear`) | Non-`Orchestrator` clear verb breaks the self-brick | **Self-brick fixed; new hole opened (Charge B)** — the verb is self-pressable by a claude worker, reopening Charges III+IV for claude. |
| III (env propagation spike) | O3 spike gains a propagation gate; ADR-011 stays `proposed` | **UPHELD** — soundly reshaped by marker-primary; the propagation gate is the right discipline. |
| IV (receipt removal owner) | Disposed — no receipt exists | **Re-incurred for the `superseded` record (Charge A)** — the new flag has no removal owner. |
| V (`tree-unclean`) | Named `tree-unclean` + `porcelain`-empty check closes the `apply-conflict` gap | **Gap closed in principle; over-refusal introduced (Charge F)** — `porcelain`-empty trips on benign untracked files. |
| VI (solo writer gate) | assert-marker-absent gates every linked→direct-writer transition | **UPHELD as written** — but see Charge E: solo's *commit shape* through gc is still unreconciled. |
| VII (superseded `--force` reflex) | `superseded` record; gc reaps on landed OR superseded | **REJECTED (Charge A)** — the cure is a second unsound runtime flag; the reflex is better answered by observability, not a new oracle. |
| VIII (compensating cleanup) | Renamed; `remove --force`, best-effort, honest non-zero | **UPHELD** — honest and correctly de-claimed. |
| IX (gc lookup key) | Disposed — `git cherry` needs only `--fork` | **UPHELD for patch-id; re-incurred for `superseded` (Charge A)** — the record's key is unspecified and name-matchable. |
| X (observability) | gc dry-run prints per-fork patch-id verdict | **UPHELD** — and is the better answer to Charge VII than the `superseded` oracle. |
| XI (env blast radius) | Child-only env + orchestrator asserts own env clean, named error | **HALF-UPHELD (Charge D)** — named error guards only funnel verbs; the authoring-brick it was raised about still misreports. |
| XII (quiescence) | Named + enforced quiescence constraint; report external mover | **UPHELD** — soundly named and enforced; the right shippable posture. |
| XIII (keystone) | `/consult`-resolved: marker-primary, claude via Agent+hook, per-harness altitude | **Direction UPHELD; keystone unproven (Charge C)** — the hook is over-broad, tier-incoherent, and unfallbacked pending the spike. |

Of the eight round-2 acquittals and resolutions the handover blessed as sound
(patch-id independence, the pure/imperative wall, env-not-self-set, the quiescence
constraint, compensating cleanup), **none is re-opened** — the Inquisition is
fanatical, not unjust. The rot is precisely where the handover sent the fire: the
welds the remediation **opened** (`superseded`, `marker --clear`, the claude hook,
the env-assertion's scope, gc's second caller).

---

## 2. Questions (interrogatories)

1. **(Charge A)** A `superseded` record is written at re-dispatch; the orchestrator
   then crashes before the replacement fork produces its delta. On recovery, does gc
   reap the original fork un-`--force`d? If yes, how does this differ from the receipt
   you burned? What **durable git** state distinguishes "superseded" from "live"?
2. **(Charge A)** Who deletes a `superseded` record, and when? Is it keyed by
   fork-head SHA or by branch name — and what stops a reused branch name from
   false-matching a dangling record?
3. **(Charge B)** What refuses a **claude** worker (marker present, no env) from
   running `marker --clear` in its own worktree and thereby restoring write- and
   `Orchestrator`-class verbs to itself? If nothing does, on what does claude
   worker-sole-writer rest besides the prompt?
4. **(Charge C)** The `WorktreeCreate` hook fires on every Agent worktree. What stops
   it stamping the marker into a **non-dispatch** isolated Agent worktree and bricking
   its writes? And if the O3 spike finds the hook does not stamp in time (or at all),
   what is claude's fallback worker-identity mechanism?
5. **(Charge D)** When a leaked `DOCTRINE_WORKER` refuses a concurrent agent's
   `slice new` on main, does the operator see the named "env polluted" error or the
   generic "worker refused"? If the latter, how is Charge XI's masquerade ended?
6. **(Charge E)** Does solo `/execute`'s multi-commit work reach coordination through
   the single-non-merge `import` funnel (which would refuse `multi-commit`), or by
   another route? Over which commits does gc's patch-id oracle range for a
   multi-commit fork?
7. **(Charge F)** Does the `tree-unclean` check refuse a coordination tree that
   carries only **untracked** files (the repo's normal state)? If so, how does
   dispatch run here at all?
8. **(Charge G)** By what input does `/dispatch` route claude to `/dispatch-agent`
   rather than `/dispatch-subprocess`, and what happens on misdetection? Is the funnel
   cadence shared or duplicated across the two sub-skills?

---

## 3. Pronouncement of Judgement

**The second remediation was true penance for the welds the first inquisition's
fixes opened — and, true to the pattern the tribunal itself named, it opened fresh
seams at the new welds.** The accused has internalised the *form* of the lesson —
the receipt is gone, replaced by a genuinely crash-proof patch-id oracle, and on
that count the work is **sound and commended.** But the *substance* of the lesson —
*do not gate an irreversible `branch -D` on a disposable, crash-surviving, ambiently
stored flag* — was abandoned in the same breath: the **`superseded` record** is the
burned receipt risen in its brother's vestments, and its "fail-safe" defence guards
only the harmless direction (Charge A, **CRITICAL**). Worse, the marker — the
agnostic keystone of the whole edifice — was handed a key to its own dungeon, and
that key was cut to fit the hand of the **claude** worker it was built to hold: with
no env-lock and no bwrap, the dominant harness's worker can self-unmark and restore
to itself both write- and `Orchestrator`-class verbs, re-opening first-tribunal
Charges III **and** IV on the very harness the slice most serves (Charge B,
**CRITICAL**). The claude identity keystone itself rests on an **over-broad,
tier-incoherent, unfallbacked** hook that brands every Agent worktree and is
deferred to an unrun spike with no Plan C (Charge C, **HIGH**). Three further HIGH
charges: the Charge XI cure was fitted to the wrong door and leaves main-side
authoring still masquerading-bricked (D); gc's landed-oracle and the import funnel
are tuned for the dispatch fork yet conscripted for multi-commit solo `/execute`
without reconciliation (E); and a MEDIUM pair — the `tree-unclean` guard over-refuses
on the repo's ordinary untracked state (F), and the per-harness router names a
discriminator it never defines (G). One LOW count of durable-knowledge heresy: a
memory still preaches the receipt the design has burned (H).

**The design does NOT earn** *nihil obstat.* **It must not proceed to `/plan`** until
Charges A and B are remediated and the design re-locked. C–E should land in the same
pass; F–H are tractable. The round-2 acquittals and the soundly-reshaped resolutions
(patch-id oracle, quiescence constraint, compensating cleanup, env-not-self-set, the
pure/imperative wall) **stand** — let the record show the keystone *thesis* is right
even as two of its new welds are smoking.

---

## 4. Sentencing (ordered penance, with verification and just punishment)

1. **Charge A (CRITICAL) — re-anchor or abolish the `superseded` oracle.** Make the
   superseded disposition durable-git (a tag/note on the abandoned fork-head, written
   only after the replacement lands) **or** reap superseded forks by
   operator-named-SHA at gc **or** answer Charge VII's reflex with the dry-run
   verdict alone and keep `--force` for superseded. Give any retained record a
   **removal owner in gc** and a **fork-head-SHA key** (never branch name). *Verify:*
   a golden where the orchestrator records `superseded`, then **crashes before the
   replacement lands** — on recovery gc **refuses** to reap the original
   un-`--force`d; and a reused branch name **cannot** false-match a dangling record.
   *Punishment:* **the relic risen in its brother's robes cast to the same fire** —
   that no flag outlive the work it dooms.

2. **Charge B (CRITICAL) — brick the prisoner's wicket.** Gate `marker --clear` on
   `is_linked_worktree == false` (a worker can never self-clear; the legitimate
   clearer runs from the parent/coordination tree), **or** require an out-of-band
   orchestrator token, **or** confess in G3/ADR-011's altitude table that claude
   worker-sole-writer is **prompt-enforced, not mechanism-enforced**, and withdraw the
   fail-closed claim for claude. *Verify:* a claude worker (marker, no env) in its own
   worktree is **refused** `marker --clear` and cannot restore write/`Orchestrator`
   verbs to itself. *Punishment:* **the gaoler in irons at his own gate** until the
   wicket opens only from outside the cell.

3. **Charge C (HIGH) — discriminate, reconcile the tier, name the fallback.** Stamp
   the hook marker **only** for dispatch-worker worktrees (a sentinel the orchestrator
   sets), not every Agent worktree; state that marker-stamping legitimately writes the
   provisioning-excluded `.doctrine/state/` tier; **name the O3-spike-failure
   fallback** (symmetric with D6's back-out) or escalate the hook to a pre-`/plan`
   `/consult` blocker. *Verify:* a non-dispatch isolated Agent worktree is **not**
   branded; the spike has a defined failure branch. *Punishment:* **the brand struck
   from the smith** who would burn every hide that passes, dispatch or no.

4. **Charge D (HIGH) — fit the cure to the wounded door.** Refuse with the named "env
   polluted" error wherever the **env leg** trips `worker_mode` **on a non-linked
   tree** (provably a leak: no fork, not a worktree) — including the **authoring**
   verbs, not only the funnel verbs. *Verify:* a leaked `DOCTRINE_WORKER` refuses
   main-side `slice new` with the **named** error, not the generic worker refusal.
   *Punishment:* **the smouldering brand quenched at the authoring forge too**, not
   only the funnel.

5. **Charge E (HIGH) — specify gc's second caller.** State solo `/execute`'s path to
   coordination; if `import`, lift single-commit (or squash) and define the oracle
   over **all** fork commits; if direct-merge, define gc's landed check for a
   multi-commit branch. *Verify:* a multi-commit solo fork is reaped only when **every**
   commit is provably landed; an unlanded earlier commit **refuses**. *Punishment:*
   **the strappado for the verb that serves two masters and confesses the shape of
   neither.**

6. **Charge F (MEDIUM) — scope cleanliness to what `apply` depends on.** Use
   `--untracked-files=no` (or check tracked + staged only). *Verify:* import
   **succeeds** with untracked files present; **refuses** `tree-unclean` on a tracked
   uncommitted modification. *Punishment:* **a day in the stocks** for refusing the
   field over the weeds at its edge.

7. **Charge G (MEDIUM) — define the router's eye and forbid the second cadence.**
   Name the harness-detection input and its misdetection-refuses behaviour; state the
   funnel cadence is **shared**, not duplicated across the sub-skills. *Verify:*
   misdetection **refuses** rather than guessing; one cadence, two thin spawn shells.
   *Punishment:* **the stocks** for the router that points without seeing.

8. **Charge H (LOW) — recant the stale relic.** Supersede `mem_019ebed8` to record the
   `git cherry` patch-id oracle as sound and the receipt as the rejected predecessor
   (keep its true damnation of `--merged`/delta-emptiness). *Verify:* the gc
   implementer retrieving `src/worktree.rs` memory reads the **patch-id** oracle, not
   the receipt. *Punishment:* **public recantation in the chapter-house** of the
   doctrine the design has overturned.

Re-lock after 1–2 (3–8 in the same pass). Charges A and B gate the rest; resolve
them first. Then — and only then — a **fourth** tribunal may grant *nihil obstat*
and the slice proceed to `/plan`.

> *Three tribunals now. The first burned the gross heresy; the second found the welds
> still smoking; the third finds the smith has learned to forge a finer flaw. The
> patch-id oracle is true steel — and beside it the accused has set a second blade of
> the old brittle iron, and cut from the keystone a key that fits the captive's hand.
> Re-temper the two new blades at the seams the mending opened — and the doctrine
> endures.*
>
> **HERESIS URITOR; DOCTRINA MANET**
