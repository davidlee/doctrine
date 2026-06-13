# Fifth Inquisition — SL-056 design (`design.md`, re-lock target)

> **HERESIS URITOR; DOCTRINA MANET**

Convened for `nihil obstat`. The author remediated five fourth-round charges
(α–ε, commit `2dce335`) and re-locked. The Inquisitor reads not the prose for
its own absolution — the Inquisitor puts the iron to the welds the fourth penance
forged. The tribunal's law holds across five rounds now: *penance breeds fresh
sin at the seams.* The round-4 remedy for Charge α conjured a **new verb** —
`doctrine worktree land` (D4b) — and round-4's remedy for Charge β leaned the
whole malice argument on a belt that **the new verb does not wear**. Two
remediations of one round contradict each other at the doctrine boundary. Three
new heresies are charged; one fourth-round disposition (δ) is **acquitted clean**.

**`Nihil obstat` is DENIED.**

Doctrine consulted: `adr show 6` (D2/D2b/D6a/D8 + Consequences), the SL-056
`design.md` (re-lock target), `inquisition-4.md` (the welds just sealed), the
slice thesis (*mechanism in prose is the design smell; faith, not works, is the
heretic's plea*).

---

## Charges

### Charge ζ (zeta) — HIGH — The import `.doctrine/`-rejection belt is **not** the unconditional containment the β remediation crowns it; `land` (D4b) is a second sanctioned funnel that wears **no belt**, and the two round-4 fixes contradict each other

**Doctrine violated.** The β resolution's own load-bearing sentence, and the
standard β itself set ("governance built on an over-claim is governance that
lies"). DC-3 (`design.md:106-108`) proclaims: *"The **unconditional** malice
containment is the **import `.doctrine/`-rejection belt** (D3) — a worker's
doctrine delta never funnels back **regardless of marker/env**."* The findings
table (`design.md:930`) repeats it: *"the **unconditional** containment is the
import `.doctrine/`-rejection belt."*

**Evidence.** In the **same round**, Charge α's remedy minted `doctrine worktree
land` (D4b). Its mechanical sequence (`design.md:563-575`) and refusal set
(`design.md:574`) are **`{tree-unclean, no-such-fork, merge-conflict}`** — there
is **no `doctrine-touch` belt**. Step 2 is a bare `git merge --no-ff <branch>`
(`design.md:569`) that lands the fork branch's **entire multi-commit delta**,
`.doctrine/` and all, onto the coordination branch. So the corpus now holds
**two** sanctioned landing funnels: `import` (belted, dispatch-only,
`design.md:351-356`) and `land` (**beltless**, solo). The "unconditional"
claim is therefore **false as written**: there exists a sanctioned funnel that
carries a fork's doctrine delta wholesale.

The claim is salvageable *only* by the unstated convention "dispatch deltas route
through `import`, solo deltas through `land`." But that convention is enforced by
**verb choice**, which is **prose discipline** — the exact smell this slice was
convened to burn. Concretely:
- `land` performs **no check** that `<branch>` is a legitimate solo branch rather
  than a dispatch worker's branch (it cannot distinguish them). A trusted-but-
  **misrouted** orchestrator that runs `land --fork <worker-branch>` instead of
  `import` lands that worker's `.doctrine/` payload **with no belt and no malice
  required** — only a wrong verb.
- The β claim's qualifier "**regardless of marker/env**" is precisely inverted
  for `land`: `land` is `Orchestrator`-classed (`design.md:556`), so for the
  `land` path the marker/env identity gate is the **only** containment, and β
  itself confessed that gate is a defeatable accident-fence. The belt that was
  supposed to hold "regardless of marker/env" **is simply absent** on this path.

**The exploit by a *malicious worker* is largely subsumed by the D2b raw-tree
residual** (a worker that can reach the coordination root to run `land` can
already write `main/.doctrine/` directly; under D6/bwrap it can reach neither).
I concede that honestly — this is **not** a new malice capability. But the
**misrouted-orchestrator** path needs **no malice and no raw-tree access**, and
the **governance claim** is false at face value regardless: β's replacement text
trades the slain "full mechanism floor" over-claim for a fresh "belt =
unconditional containment" over-claim, born in the same round that gave doctrine
a beltless funnel.

**Risk.** β's stated risk realised verbatim: the false "unconditional
containment" propagates into the G3/ADR-011 altitude table and the G4/SPEC-012
rewrite (`design.md:677-683`) as load-bearing canon. Governance built on an
over-claim is governance that lies — the author's own words, now self-inflicted a
second time.

**Sentencing.** The belt does **not** get bolted onto `land` (solo is a trusted
self-orchestrator, D6a — it *legitimately* lands doctrine; a belt there is a
category error). Instead **re-scope the claim honestly**, mirroring the β
re-claim of the marker floor: the `.doctrine/` belt is the containment **on the
import/dispatch funnel**, and is unconditional **only conditioned on dispatch
deltas being routed through `import`, never `land`**; `land` is a beltless,
solo-trusted funnel whose only containment is the `Orchestrator`-class accident-
fence, and a worker reaching it is the already-confessed D2b residual. Carry the
re-scope verbatim into DC-3, the G3/ADR-011 altitude table, and G4/SPEC-012.
**And add teeth where they are cheap:** `land` should **refuse a branch carrying
the worker marker** (a dispatch fork) — a fast, in-CLI guard that mechanises the
prose convention and converts "misrouted orchestrator lands worker doctrine" from
a silent success into a named refusal. Verification: a golden where `land` is
invoked on a marker-bearing fork → **refuse**; and the altitude table states the
belt's scope is the import path, not all funnels. *Let the over-claim that rose
from its own twin's ashes be cast back into the fire it was forged in.*

---

### Charge η (eta) — HIGH — `land`'s `merge-conflict` path is **destructive and unowned**: `git merge --no-ff` mutates-then-conflicts, leaving a half-merged coordination tree that the verb's **own** `tree-clean` precond then rejects — it does **not** "mirror import's report-don't-merge"

**Doctrine violated.** The slice's fail-closed mandate and `land`'s own contract.
D4b step 3 (`design.md:570-572`) claims the conflict path is *"refuse
`merge-conflict`, **report + halt** (solo resolves, then re-lands) — never
auto-resolve, **mirroring the funnel's report-don't-merge posture** (ADR-006
D2)."* This claim is **mechanically false** for `git merge`.

**Evidence.** `import`'s report-don't-merge is genuinely non-destructive: under
its `HEAD == B` **and** tree-clean preconds the `git apply --3way` **cannot
conflict** (the design purged `apply-conflict` as a refusal on exactly this
ground — `design.md:378-384`), so import either applies cleanly or never mutates.
`land` is **not** `git apply` — it is `git merge --no-ff` (`design.md:569`),
which **mutates the index and working tree, writes conflict markers, and sets
`MERGE_HEAD` *before* it reports the conflict and exits non-zero.** On the
`merge-conflict` branch:
- The coordination tree is left **mid-merge**: `git status --porcelain
  --untracked-files=no` reports `UU`/`AA`/`DD` unmerged entries ⇒ **non-empty**.
- `land`'s **own** step-1 precond (`design.md:564-566`) is *"coordination tree
  clean (`git status --porcelain --untracked-files=no`-empty) else
  `tree-unclean`."* So the **next** `land` (the promised *"re-lands"*) refuses
  **`tree-unclean`**. So does any `import`. The operator is wedged.
- The design's imperative shell for `land` (`design.md:691`, `classify_land` +
  "drives `git merge --no-ff`") specifies **no `git merge --abort`** anywhere —
  not in D4b, not in code-impact, not in Verification (`design.md:790`).
- The design's own gc test (`design.md:796`) posits *"a **partial land**
  (conflict half-resolved out-of-band)"* as a live state — proof the author knows
  `land` can strand a half-merge — yet **nowhere specifies how the coordination
  tree is restored to clean**. gc refuses the partial fork (correct) but leaves
  the tree mid-merge; the verb family that was meant to abolish filesystem surgery
  now **mandates** a manual `git merge --abort`.

The workflow text is also internally muddled: *"solo resolves, then re-lands."*
If solo resolves the **in-progress** merge and commits, the land has **completed
manually** — there is nothing to "re-land," and the merge commit already makes
fork-tip an ancestor (gc's ancestry leg fires). If instead solo is meant to
`--abort`, fix the source, and re-run `land`, then the verb **must** abort to
restore its own precond. The design commits to neither, and "report+halt mirroring
import" is true of neither.

**Risk.** Every real `land` conflict (the design itself admits *"genuine code
coupling, or coordination moved under the run"*, `design.md:570`) wedges the
coordination tree against all `Orchestrator` verbs until a human performs the
exact git surgery the slice exists to eliminate. A fail-closed verb that
fail-**dirties** is a contradiction in terms.

**Sentencing.** Choose and state **one** posture, with mechanism:
**(a)** the imperative shell runs **`git merge --abort`** on conflict before
returning the refusal — restoring the clean precond, making the verb a *true*
mirror of import's leave-nothing-behind report-don't-merge; solo then fixes the
coupling at source and re-runs `land`; **or (b)** declare the half-merge an
**intentional** hand-off (solo completes the in-progress merge), in which case
delete the false "mirrors import" / "re-lands" language, state that `land` leaves
a resolvable merge in progress, and reconcile that against the `tree-clean`
precond (the re-entry guard must tolerate the operator's own in-progress merge).
If the design instead relies on solo-mode **quiescence** to make conflicts
impossible, it must **state a `land` quiescence constraint** as explicitly as
import states XII (`design.md:408-421`) — it currently does not. Verification: a
golden where `land` hits a real conflict, then **asserts the coordination tree is
clean** (posture a) or **asserts the documented in-progress state and a working
re-entry** (posture b) — never the present silent wedge. *Let the verb that
leaves a wound behind it be bound to cauterise its own cut, or confess plainly
that it bleeds.*

---

### Charge θ (theta) — HIGH — "Serial-only" claude marker-stamping is **prose, not mechanism**: nothing refuses a second armed spawn, and an orchestrator that follows the **blessed** parallel-batch path while armed enters a **fail-open** race — not the clean "degrade to prompt-enforced" the γ remediation claims

**Doctrine violated.** The slice thesis — *mechanism in prose is the design
smell; faith, not works* — and ADR-006 D2 (worker-sole-writer, load-bearing). The
γ remediation traded a racy *unconditional* stamp for a **serial-only constraint
with no enforcement point**.

**Evidence.** `design.md:198-210`: claude marker-stamping *"requires **one armed
spawn in flight at a time**: the orchestrator arms, launches **exactly one**
Agent, awaits its WorktreeCreate stamp+disarm, and does **nothing else
worktree-creating** in that window."* Every clause of that is addressed to **the
orchestrator** — a prompt following `/dispatch-agent` SKILL.md. The sentinel is
*"a transient signal in its own runtime tier"* (`design.md:183`) with **no
specified `arm` verb and no refuse-if-already-armed guard**. Meanwhile the
dispatch skill **blesses** *"parallelize file-disjoint phases into one concurrent
batch."*

So the load-bearing safety property — *at most one armed spawn in flight* — is
guaranteed by **orchestrator self-restraint alone**. Two unhandled facts make
that fail-**open**, not cleanly degraded:
- The design frames the parallel case as *"Parallel file-disjoint claude dispatch
  **degrades to prompt-enforced** (no marker)"* (`design.md:206`) — a clean
  either/or: serial-with-marker **xor** parallel-without-marker. But that clean
  split exists only if the orchestrator **knows to skip arming** when going
  parallel. The natural failure is an orchestrator that **arms** (intending
  serial) and **then** fires a concurrent batch (per the blessing). That is the
  racy middle the round-4 charge named (`inquisition-4.md:135-146`): treeA reads
  armed → stamps → disarms; treeB reads disarmed → **no marker** → **unmarked
  worker writes freely to the coordination tree (fail-open)**, and the wrong tree
  may be branded (innocent worker bricked). The race the γ remediation claimed to
  retire by "serial-only" is **re-entered by the blessed parallel path**, because
  nothing **mechanically** stops it.
- Confirming the failure mode is **not** preventing it. The widened O3 spike
  (`design.md:802-806`) asserts that *"two concurrent `isolation:worktree` spawns
  against one armed sentinel **mis-brand**"* — it **proves the hole**, it does not
  **close** it. "Serial-only is evidence-based, not assumed" (`design.md:209`) is
  evidence that the unmechanised path is dangerous, offered in place of a
  mechanism that would make the dangerous path unreachable.

This is the slice's own indictment turned on the slice: a load-bearing invariant
left to prompt discipline at the one seam round-4 reopened — *faith, not works.*

**Risk.** A blessed path (parallel file-disjoint claude dispatch) silently
produces **unmarked workers writing to the coordination tree** the instant an
orchestrator arms-then-parallelises. The worst outcome (fail-open writes) is
reachable by an orchestrator doing exactly what the dispatch skill invites.

**Sentencing.** **Mechanise the single-slot.** Make `arm` a guarded act that
**refuses if a sentinel is already armed** (or if a stamp is awaited) — a trivial
single-slot lock in the runtime tier. Then an orchestrator that arms-then-fires-
concurrently has its **second arming physically refused**: it either serialises
(arm succeeds, one stamp, disarm) or, going parallel, **cannot arm at all** ⇒ the
honest "no marker, prompt-enforced" degrade the design *claims* but does not
*enforce*. This converts "one in flight at a time" from orchestrator discipline
into CLI mechanism — the slice's whole reason to exist. Verification: a golden /
spike where a second `arm` while armed **refuses**, and where a concurrent batch
under an armed sentinel **cannot produce a second stamp**. *Let the racing seam be
chained to a single bolt that admits but one penitent at a time.*

---

### Charge ι (iota) — MEDIUM — The ε router cross-check mechanised the **logic** but deferred its **load-bearing input** — the env-marker names and their reliability — to "the skill/spike" with **no spike-gate and no named fallback**, unlike its sibling deferrals (C, D6). Charge III's recursion: the design relies on a detection signal it has not proven exists

**Doctrine violated.** Charge III's recursive law, invoked throughout this design
— *spike what you rely on* — and the design's own discipline of marking
spike-contingent claims with a **named fallback** (it does so honestly for the
claude hook, Charge C `design.md:196-197`, and for bwrap, D6 `design.md:626-627`).
ε received the mechanism but **not** the gate.

**Evidence.** `design.md:695`: the router *"probes **env markers** (`CLAUDECODE`
for Claude Code; the codex/pi equivalents — **precise names resolved in-skill/at
the O3 spike, not hardcoded in the binary**) and routes **only when detection
agrees with self-belief**; mismatch or unknown → refuse."* The Verification
(`design.md:807-809`) tests the **agreement logic** (contradict → refuse; agree →
route; unknown → refuse) — but tests **nothing about whether a stable, harness-
unique, launch-mode-robust marker actually exists.** Two unspiked dependencies:
- **The signal may not exist reliably.** Env markers vary by launch mode —
  headless, cron, nested, IDE-embedded. (This very session's harness warns that
  interactively-authenticated services *"may be absent in headless/cron runs."*)
  A marker present in an interactive claude run may be **absent** in a cron-driven
  one, or **renamed** across a Claude Code version. ε mechanised a cross-check
  whose truth depends entirely on a marker that is **TBD** and **version-fragile**
  (cf. `[[mem.pattern.parse.toml-error-classification-fragile]]`).
- **No fallback is named.** C degrades to prompt-enforced; D6 backs out to the
  CLI guard. ε names **neither**. If the spike finds no reliable per-harness
  marker, the design has **not said** what the router does — refuse-all (bricking
  legitimate dispatch on every harness)? fall back to self-belief-only (reopening
  ε)? The "mechanised cross-check" is a shell around a hollow core: the unmechanised
  assertion moved from *"the agent believes it is claude"* to *"some env var, TBD,
  proves it is claude"* — and the second is **not yet known to be true**.

**Compounding (probe to the User, not yet a separate charge):** "unknown ⇒
refuse" is fail-**safe** (a misroute is API-billed/failure; a refusal is
recoverable), and I do **not** fault the refuse-on-uncertain posture. But it
introduces a fragility the pre-ε design lacked: a legitimate harness whose marker
is **absent or renamed** is now **bricked**, and the design does not require the
refusal to **name the cause** (contrast the Charge-D dual-cause message,
`design.md:329-336`). A bare "dispatch refused" leaves the operator blind to a
marker-name drift.

**Risk.** A core remediation claim — "router routes on mechanism, not belief" —
rests on an input the design has not proven obtainable. If the spike returns "no
stable marker," ε silently reverts to the very heresy it claimed to burn, with no
written fallback to catch the fall.

**Sentencing.** **Spike-gate the detection signal, symmetric with C and D6.**
The O3 spike must confirm a **stable, harness-unique, launch-mode-robust** marker
per harness; and the design must name the **fallback** for "no reliable marker
exists" (e.g., refuse-all-dispatch with a named diagnostic, *not* a silent revert
to self-belief). The ε resolution text and the G3 altitude table must carry the
claim as **`proposed` until the marker-existence gate is green** — exactly as the
env-propagation claim is held `proposed` (`design.md:671`). And the router's
refusal must **name the cause** ("env marker for claimed harness `<h>` not found
— harness mis-seeded, renamed, or launch-mode-stripped; dispatch refused"). Pin
with goldens for contradiction-refuses, marker-absent-refuses-**by name**, and a
documented fallback. *Faith dressed as works is the subtler heresy; let the relic
be authenticated before the pilgrims are sent to kneel before it.*

---

## Acquittal (fourth-round disposition that survives the iron)

- **Charge δ — `gc --superseded-head` reframe — ACQUITTED.** The hunt for stray
  "oracle"/"proving-landed" language near `--superseded-head` finds **none**.
  `design.md:507-509` states plainly: *"**This is not an oracle** … it is an
  **operator assertion** … the head-match is a **TOCTOU movement-guard** … **not
  a proof of landing**."* The findings table (`design.md:932`) repeats the honest
  reframe. The earlier "proving the operator named the right commit" framing the
  fourth round condemned is **gone**. The verb is now honest that it reaps
  *unlanded* work on the operator's word, head-guarded against TOCTOU. **Clean —
  let it pass unmolested.**

Also re-affirmed unmolested (re-open only on new evidence): the gc **ancestry
leg** (`--is-ancestor`) is sound — it is monotonic under an advancing
coordination HEAD (once an ancestor, always an ancestor as HEAD moves forward),
so it does **not** false-negate after further coordination commits the way the
rejected delta-emptiness oracle did; the degenerate fork-tip==B and single-commit-
squash cases reap harmlessly or patch-id-correctly (`design.md:479-480`); the
patch-id oracle's crash-proofness; the quiescence constraint (XII); compensating
cleanup (VIII); the pure/imperative wall.

---

## Questions (interrogatories)

1. **(ζ)** Is the `.doctrine/` belt's "unconditional containment" claim intended
   to cover the `land` path? If yes — where is `land`'s belt? If no — why does
   DC-3 say "regardless of marker/env" when `land` is gated by *only* marker/env?
   What stops a misrouted orchestrator running `land` on a dispatch worker's
   branch?
2. **(η)** On a `land` merge conflict, does the imperative shell run `git merge
   --abort`? If not, how does the coordination tree return to the `tree-clean`
   state `land`'s own precond demands before the promised "re-land"?
3. **(θ)** Is the claude arming sentinel a **guarded verb** that refuses a second
   arming while one is in flight, or orchestrator prose? If prose, what *mechanism*
   stops an armed orchestrator from firing the blessed parallel batch into a
   fail-open race?
4. **(ι)** What does the router do if the O3 spike finds **no** stable, launch-
   mode-robust, harness-unique env marker? Is that fallback written anywhere?

---

## Pronounce Judgement

**This design is, for a fifth time, tainted by heresy — `NIHIL OBSTAT` IS
DENIED.**

The fourth penance was honest where it confessed (the `--superseded-head` reframe,
δ — **acquitted**) and the patch-id oracle, the ancestry leg, the quiescence
constraint, the compensating cleanup, and the pure/imperative wall **stand
acquitted and unmolested**. But the round-4 fixes forged new welds and the iron
found them:

- **ζ (HIGH):** the β remediation's "belt = unconditional containment, regardless
  of marker/env" is **falsified by `land`**, the beltless funnel the *same round*
  minted for α. Two fourth-round fixes contradict at the doctrine boundary; the
  over-claim β slew returns in new robes.
- **η (HIGH):** `land`'s `merge-conflict` path is **destructive and unowned** —
  `git merge` mutates-then-conflicts, wedging the coordination tree against
  `land`'s own `tree-clean` precond; the "mirrors import's report-don't-merge"
  claim is mechanically false; no `--abort` is specified.
- **θ (HIGH):** "serial-only" claude stamping is **prose with no enforcement
  point**; the blessed parallel batch re-enters a **fail-open** race the γ remedy
  claimed to retire. *Faith, not works* — at the very seam round-4 reopened.
- **ι (MED):** the ε cross-check mechanised the logic but **deferred its
  load-bearing input** (marker names + reliability) with **no spike-gate and no
  fallback**, unlike its siblings C and D6. Charge III's recursion bites.

The pattern holds for a **fifth** round: **penance breeds fresh sin at the welds.**
The design may **not** proceed to `/plan` until ζ, η, θ are dispositioned, ι is
spike-gated, and the design re-locked. **The author marks not their own homework
— a sixth confirmatory inquisition follows the remediation.**

## Sentencing (ordered corrective sequence)

1. **η first — it is the concrete mechanism bug.** Decide `land`'s conflict
   posture: `git merge --abort` on conflict (true import-mirror) **or** documented
   in-progress hand-off with a reconciled re-entry precond — and add a `land`
   quiescence constraint if conflicts are claimed impossible. Pin with a
   real-conflict golden asserting a clean (or documented) tree. *Cauterise the
   wound.*
2. **ζ** — re-scope the belt claim honestly (containment **on the import path**,
   conditioned on dispatch routing through `import` not `land`; `land` is solo-
   trusted-beltless, worker-reach = D2b residual), propagate to DC-3 / G3-ADR-011 /
   G4-SPEC-012, and add a `land`-refuses-marker-bearing-fork guard. Pin with the
   marker-fork-refusal golden. *Burn the twice-risen over-claim.*
3. **θ** — mechanise the single-slot: `arm` refuses while armed/awaiting-stamp.
   Pin with a second-arm-refuses spike assertion. *One penitent at the bolt.*
4. **ι** — spike-gate the env-marker existence with a named fallback; hold the
   claim `proposed` until green; name the refusal cause. Pin with a marker-absent-
   refuses-by-name golden. *Authenticate the relic.*

Each disposition is recorded in a **Fifth-inquisition findings integrated** table
in `design.md` (the durable home; this sheet is the withheld working tier), the
design is **re-locked**, and **only then** is a **sixth** confirmatory inquisition
convened.

> **HERESIS URITOR; DOCTRINA MANET**
