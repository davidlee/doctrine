# REV REV-043 — Restate runtime-tier defence: absence to single-homed plus guard

Revision (ADR-013) — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

SL-237 single-homes repo-scoped runtime state (phase sheets, the source-delta
registry) in the primary worktree. Authored statements at three altitudes —
ADR-006, SPEC-012, PRD-015 — assert the opposite mechanism. They are **one claim
at three altitudes**, so they are restated together rather than in separate
revisions; splitting them is what let them drift apart in the first place.

**This is a restatement of mechanism, plus ONE named relaxation.** An earlier
draft of this REV claimed pure restatement. That was overreach, caught by RV-322
F-4 and corrected here: the *merge-safety* claim is genuinely preserved, but
single-homing introduces a **new concurrent read-modify-write exposure** that
per-worktree copies did not have. It is stated in § "The named relaxation" below,
not folded into the restatement.

*Scope widened after RV-322 F-1: the original draft asserted "no requirement
changes", inheriting a pre-design sweep that checked REQ-297 and stopped. PRD-015
carries an invariant and two requirements bearing directly on this change.*

### What does NOT change

- **Merge safety (ADR-006 D4's substance) is not weakened — arguably
  strengthened.** The named hazard was a *copied* phase sheet diverging across
  worktrees and merging back. Single-homing creates **no copy**. One file cannot
  diverge from itself.
- **The worker write prohibition (REQ-297, ADR-006 D2) is untouched by SL-237.**
  The CLI guard refuses write-class slice verbs under `worker_mode` on both arms
  — `SliceCommand::Phases` → `Write("slice phases")` and `Phase` →
  `Write("slice phase")` (`src/commands/guard.rs:78,80`). Single-homing changes
  *where a path resolves*, not *who may write*; this REV therefore restates
  nothing about the prohibition and adjudicates nothing about its strength.

  **Scope note (RV-322 round 5, F-1).** An earlier draft argued the prohibition
  additionally has an "OS floor on BOTH arms", citing
  `scripts/pi-spawn-confined.sh` for the codex/pi arm. That argument is **removed
  from this REV**, for two independent reasons: the normative
  `dispatch-subprocess` skill spawns `env -C "$D" DOCTRINE_WORKER=1 …` with no
  bwrap and never references that script, so the citation did not describe the
  shipped path; and citing a doctrine-repo script as evidence for a *platform*
  guarantee is a POL-002 problem regardless. Neither observation is caused by
  SL-237 and neither is load-bearing for it. The gap is **[[IMP-354]]**; whether
  REQ-304's "by construction" survives the cooperative-confinement posture is
  **[[QUE-199]]**. This REV takes no position on either.
- **ADR-006 D4 clause 3 — "the `phases` symlink is relative" — is unchanged.**
  SL-237 (DEC-097) mints the convenience symlink *only in the primary*; where it
  exists it stays relative.
- **No central index or counter is introduced.** The phases directory is
  gitignored (`.gitignore:39`), per-slice, id-derived (nothing is minted or
  counted), and single-writer under D2/D9. Concurrent drives touch disjoint
  `<NNN>` directories. D4's warning that *"any future central index reintroduces
  conflict"* was tested against this design and does not fire **for the
  duplication hazard it names** — see the named relaxation for what it does not
  cover.
- **The provisioning mechanism is untouched.** The allowlist `is_withheld`
  (`src/worktree/allowlist.rs:170`; `WITHHELD` covers `.doctrine/state/slice/**`)
  is not modified by SL-237. **No copy is provisioned into a fork.** The tier
  remains literally absent from the fork's own tree; what changes is that a *path
  resolves outward*. (This is what makes row 3b's revised REQ-296 wording true.
  It is **not** an argument that REQ-296 needed no revision — that was the round-1
  error, corrected in row 3b: the mechanism survives, the *promise* did not.)
- **PRD-015 NF-003 (REQ-304) — "tier exclusion guaranteed by construction rather
  than a trusted check" — is OUT OF SCOPE for this REV.** *Changed after RV-322
  round 5.* Its *provisioning* limb is untouched (no copy reaches a fork). Its
  *write-exclusion* limb raises a question this REV cannot answer without
  adjudicating the dispatch confinement model, which SL-237 does not change:
  see [[QUE-199]]. Earlier drafts adjudicated it as "preserved, construction
  relocated to the OS floor"; that adjudication rested on the removed OS-floor
  argument and is withdrawn rather than re-asserted on weaker evidence.

### What DOES change

The **stated defence mechanism**; PRD-015's non-crossing invariant; and one new
**read** exposure. Plus the named relaxation.

### The named relaxation — concurrent RMW on a now-shared file

Per-worktree copies could **diverge** but could not **race**. One file can race.
`edit_phase_sheet` is an unlocked read-modify-write, and the source-delta
registry is likewise unlocked shared-mutable. `run_phases` carries no
live-drive guard, where `run_reconcile_phases` does
(`src/slice.rs:1236-1244`).

**No locking mechanism is designed, and this REV does not pretend otherwise.**
The exposure is bounded by contract, not by construction:

- ADR-006 D2/D9 make the orchestrator the **sole** phase-state writer during a
  drive, so the ordinary dispatch path has one writer by governance.
- The **single-operator precondition** already documented on
  `run_reconcile_phases` ("doctrine has no cross-machine lock, so two operators
  reconciling one repo concurrently is out of contract") is promoted from a
  verb-local note to a **tier-level assumption** covering the shared file.

This is the honest trade: the previous arrangement avoided the race by keeping
two copies, and paid for it with divergence that required a completion mirror and
generated five patches. Recording the exposure is a precondition of accepting
that trade knowingly.

---

## Row 1 (primary) — ADR-006, `modify`

Two prose edits in the **Decision** block. Both are Decisions, not Context; this
is why the change is routed through a REV rather than noted as drift at reconcile
(SL-237's scope doc originally anticipated the latter and has been corrected).

### 1a — D2, the read parenthetical

**Before**

> Workers may **read** doctrine state freely — its authored and provisioned tiers
> are in the fork (the coordination/runtime tier is withheld, D9) — but **write**
> none of it.

**After (proposed)**

> Workers may **read** doctrine state freely — its authored and provisioned tiers
> are in the fork, and since SL-237 the repo-scoped runtime tier is *reachable*
> (single-homed in the primary, not copied into the fork) — but **write** none of
> it. Withholding was always the *means*; the invariant is the write prohibition,
> which is enforced by the CLI worker guard (D2a) on both arms, under an OS floor
> on each — a read-only mount on the codex/pi arm, PreToolUse confinement on the
> claude arm. (ADR-008 D-B3's "claude cannot be wrapped" prose predates that
> confinement and is stale; it is corrected separately, ISS-278.)

**Why.** The parenthetical asserts the runtime tier is *withheld*. After SL-237 a
fork resolves phase-sheet paths to the primary, so the tier is no longer absent
from a worker's reach. The load-bearing claim of D2 — *workers write none of it* —
is untouched.

**Accepted residual, named not papered.** A worker can now *read* primary phase
sheets, including sibling phases' status in the same drive. This sits against
D6's pre-distillation rationale (the orchestrator distils withheld state into the
worker prompt). It is accepted rather than fenced: D2 already grants free read of
the **authored** tier, which includes the slice design — strictly more sensitive
than a phase sheet's status string — so this adds no new *class* of information.
The residual is prompt discipline (a worker may see, and act on, work belonging to
another phase), not an invariant breach.

### 1b — D4, the per-worktree clause

**Before**

> **D4 — Tier merge-safety invariant.** No shared-mutable authored file across the
> unit of isolation; runtime state stays gitignored/per-worktree; the `phases`
> symlink is relative. This invariant is *defended*, not assumed — any future
> central index reintroduces conflict.

**After (proposed)**

> **D4 — Tier merge-safety invariant.** No shared-mutable authored file across the
> unit of isolation; runtime state stays **gitignored**, and is **repo-scoped
> (single-homed in the primary) or tree-scoped** according to whose fact it is —
> repo-scoped runtime state is never *copied* into a fork, so nothing can diverge;
> the `phases` symlink is relative. This invariant is *defended*, not assumed — any
> future central **authored** index, or any *copied* mutable runtime file,
> reintroduces conflict.
>
> - **D4 amendment (SL-237).** The original clause read "runtime state stays
>   gitignored/**per-worktree**". Per-worktree was an over-broad statement of the
>   real requirement: what defends merge safety is the **absence of a second
>   mutable copy**, not the absence of sharing. Single-homing satisfies the
>   requirement more strongly than per-worktree did — a shared *pointer* to one
>   gitignored file has nothing to merge and nothing to diverge, whereas the
>   per-worktree arrangement it replaces demanded a completion **mirror**
>   (`mirror_completion_into_primary`) precisely because two copies drifted. That
>   mirror is retired by SL-237.

**Why.** The clause is directly falsified. The amendment states what the invariant
was actually protecting and shows the new arrangement protects it better — the
evidence being that the per-worktree arrangement generated five patches
(ISS-212, IMP-272, IDE-028, ISS-269, SL-228 / RV-312 F-6) and required a mirror to
stay coherent.

### 1c — D9, the provisioning invariant's stated ground

*Added after RV-322 round 5 (F-18). D9 states the same claim as D4 from the
provisioning side; the round-4 sweep never reached it.*

**Before**

> **Invariant (guards D2):** the allowlist **excludes the coordination/runtime
> tier** (`.doctrine/state/`, the `phases` symlink, `handover.md`, memory caches)
> by construction — their *absence* in the fork is what makes worker-sole-writer
> free; a copied phase sheet would be invisibly mutable.

**After (proposed)**

> **Invariant (guards D2):** the allowlist **excludes the coordination/runtime
> tier** (`.doctrine/state/`, the `phases` symlink, `handover.md`, memory caches)
> by construction — **no copy** of it is provisioned into a fork, so a worker
> holds nothing that can diverge; a copied phase sheet would be invisibly mutable.
> Where that tier is repo-scoped it is single-homed rather than duplicated
> (SL-237), so a path may resolve outward to it — the write prohibition, not
> unreachability, is what keeps worker-sole-writer free.

**Why.** The allowlist exclusion itself is **unchanged and still by
construction** — `is_withheld` is untouched. What is falsified is only the
clause's stated *ground*: after single-homing, "absence in the fork" no longer
describes the repo-scoped part of the tier, so the invariant needs the write
prohibition named as the load-bearing leg rather than reachability. This is the
same correction as D4 and row 2, from the provisioning side.

---

## Row 2 — SPEC-012, `modify`

Two prose sites, both making the same "by absence" claim: the responsibility
bullet near the top of the spec, and § "Tier merge-safety by construction".

**Before** (§ Tier merge-safety by construction)

> The container defends ADR-006 D4 not by trust but by absence: the
> coordination/runtime tier is never copied into a fork, so a worker has nothing
> shared-mutable to corrupt — a copied phase sheet would be invisibly mutable
> across worktrees. The orchestrator's pre-distilled worker prompt (ADR-006 D6)
> substitutes for the withheld coordination state; provisioning substitutes for the
> absent execution environment. No central index or counter exists to reintroduce a
> conflict.

**After (proposed)**

> The container defends ADR-006 D4 by **non-duplication**, not by trust: the
> repo-scoped runtime tier is never *copied* into a fork — since SL-237 it is
> single-homed in the primary and named by one path from every worktree — so a
> worker has no second mutable copy to diverge. A copied phase sheet would be
> invisibly mutable across worktrees; a shared pointer to one gitignored file is
> not.
>
> The **write** prohibition is enforced, not assumed absent: the CLI worker guard
> refuses every write-class slice verb under `worker_mode` — `slice phase`,
> `slice phases`, `record-delta`, `reconcile-phases` (`src/commands/guard.rs`
> `write_class`, ADR-006 D2a). What each harness can additionally enforce beneath
> that refusal is stated by the arm's own documentation, not here.
>
> The orchestrator's pre-distilled worker prompt (ADR-006 D6) still substitutes for
> coordination state a worker should not need; provisioning still substitutes for
> the absent execution environment. Note that pre-distillation is now a matter of
> *discipline* rather than *reachability* for the repo-scoped runtime tier — see
> ADR-006 D2's accepted read residual. No central authored index or counter exists
> to reintroduce a conflict.

**Three further SPEC-012 surfaces assert the same "absence ⇒ sole-writer-free"
ground and are restated identically** (*added after RV-322 round 5, F-18 — the
round-4 sweep enumerated sites within the documents already under revision and
never reached these*):

| surface | current text | restated to |
|---|---|---|
| `spec-012.toml:21` (structured responsibility) | "the coordination/runtime tier's **absence** in the fork … is what makes worker-sole-writer free" | "…the repo-scoped runtime tier is **single-homed, never copied** into a fork — which is what keeps a worker's tree free of divergent coordination state" |
| `spec-012.md:52` (rendered Responsibilities summary) | "defend tier merge-safety by the tier's **absence** in the fork" | "defend tier merge-safety by **non-duplication** — the tier is never copied into a fork" |
| `spec-012.md:274-276` (§ hypothesis, "Exclude by construction, not by trust") | "the tier's *absence* is what makes worker-sole-writer free" | "the tier is never *copied* into the fork, so a worker holds no divergent second copy; where it is repo-scoped and single-homed, the write prohibition — not unreachability — is what keeps the sole-writer invariant" |

The structured `responsibilities` entry and its rendered summary are **two tiers
of one claim** (`spec-012.toml` + `spec-012.md`); both are listed because editing
only the prose leaves the queried tier stale — the defect this REV had itself
already made in DEC-098 and had raised against.

**Deliberately NOT restated:** none of these edits touches whether the surviving
write-exclusion is "by construction" or cooperative. That is [[QUE-199]].

**Why.** This is spec prose rather than a REQ, but it is a load-bearing
architectural claim, not incidental context.

**No REQ in SPEC-012's OWN roster requires change** — REQ-189..196, REQ-248..252
govern the fork/provision/marker/env contract, and SPEC-021 governs orchestrator
altitude. This is deliberately narrower than the claim the first draft made.
*That* claim — "no REQ requires change" full stop — was inherited from a
pre-design sweep which checked REQ-297 and stopped, and RV-322 F-1 proved it
false: PRD-015 carries an invariant and two requirements bearing directly on this
change, which is why Row 3 exists. The sweep is no longer cited as evidence for
anything; each roster is stated separately, on its own reading.

---

---

## Row 3 — PRD-015, `modify`

*Added after RV-322 F-1.* Three statements were swept; only one needs revising.

### 3a — Invariant 2 (the one that changes)

**Before** (§4 Invariants)

> - The coordination/runtime tier never crosses the isolation boundary into a
>   worker.

**After (proposed)**

> - The coordination/runtime tier is never **copied** across the isolation
>   boundary: an isolated unit holds no second mutable copy, so nothing it can
>   touch can diverge. Where the tier is repo-scoped it is single-homed rather
>   than duplicated, so an isolated unit may *read* it; no unit is permitted to
>   **write** it.

**Why.** A *read* now crosses the boundary. The invariant as written is falsified
by reach, even though nothing is copied and nothing may be written.

**Altitude.** *Corrected after RV-322 round 4 (F-12).* The first draft of this
row wrote "Since SL-237 …" and named the sole-writer guard and the per-arm OS
floor — the same two defects row 3d had already been corrected for, left standing
here. A PRD states evergreen intent: a slice citation dates the document the
moment the slice closes, and the enforcement layer is SPEC-012's to own and
revise.

*Round 5 (F-12) then caught the repair.* Round 4 claimed this row now matched row
3d "verbatim, so the two cannot drift apart again" — untrue on both counts. Only
the middle clause matched (3a said "the boundary guarantees", 3d "the
construction guarantees"), and **duplicating a sentence into two independently
editable surfaces is what causes drift, not what prevents it** — the opposite of
the fix applied to DEC-098's title one round earlier, which worked by *removing*
a restatement. The two rows now say the minimum each surface needs: 3a is an
**invariant** (what is and is not true of the boundary), 3d a **principle** (why
the design is built that way). Neither restates the other.

### 3b — FR-001 (REQ-296): REVISED, wording only

*Changed from "adjudicated, not revised" after RV-322 round 3 (F-1 re-contested).*

**Before**

> Each dispatched unit of work runs in isolation — a private working tree
> provisioned per run — with the coordination/runtime tier **absent by
> construction**, so nothing shared-mutable can be corrupted.

**After (proposed)**

> Each dispatched unit of work runs in isolation — a private working tree
> provisioned per run — with **no copy of the coordination/runtime tier
> provisioned into it**, and **no unit permitted to write that tier**.

**Why the earlier adjudication was wrong.** The first pass preserved this
requirement on the reasoning that its *mechanism* (the provisioning allowlist,
untouched) survives. That conflated mechanism with promise. The requirement's
words are "absent by construction", and a reader takes *absent* to mean
unreachable — which is now false: a path resolves outward and the tier is
readable from an isolated unit. Keeping the sentence because its implementation
still works would leave a product-level promise the product no longer keeps.

**What is preserved, and what is not.** *Sharpened after RV-322 round 4 (F-1).*
An earlier draft of this row claimed the force was untouched "because the
corruption it guards against comes from a second mutable copy, and there is
none". That inference does not hold: **no copy proves non-*divergence*, not
non-*corruption*.** A lost update on a single shared file is corruption and needs
no second copy — and this REV itself admits that exposure (see "The named
relaxation": `edit_phase_sheet` is an unlocked read-modify-write on the
now-shared file).

So the revision **states the two guarantees and drops the consequent**, rather
than repairing it. What remains is exactly what SL-237 leaves standing:

- **no copy is provisioned into the unit** — the allowlist `is_withheld`
  (`src/worktree/allowlist.rs:170`), untouched by SL-237. Construction-backed.
- **no unit is permitted to write the tier** — ADR-006 D2 and the CLI worker
  guard. A *prohibition*, stated as one.

*Second correction, RV-322 round 5 (F-1).* An intermediate draft wrote the second
leg as "**no ability** to write that tier" and called both legs
construction-backed. "No ability" is too strong: the normative subprocess spawn
is unconfined (see the scope note above), and confinement in this system is a
cooperative accident-fence by design
(`mem.fact.dispatch.worker-confinement-is-actor-based`). A prohibition that is
enforced but circumventable is honestly written as a prohibition, not as an
inability — and whether PRD-015 may claim more than that is [[QUE-199]], not this
REV's to answer.

The dropped consequent — "so nothing shared-mutable can be corrupted" — is **not**
replaced. The operator-concurrency hazard is real, is not this requirement's to
promise, and is recorded in "The named relaxation". A requirement that states two
true guarantees is stronger than one that states a conclusion neither leg
supports.

### 3c — NF-003 (REQ-304): OUT OF SCOPE, deliberately unresolved

> Isolation integrity: tier exclusion is guaranteed **by construction rather than
> a trusted check**, and the sole-writer guard fails closed on ambiguity.

*Changed from "adjudicated, not revised" after RV-322 round 5.* Earlier drafts
preserved this on the strength of an OS floor on both arms; that evidence is
withdrawn (see the scope note in the rationale). Rather than re-adjudicate it on
weaker grounds, this REV **takes no position**:

- The requirement's *provisioning* limb is untouched — no copy reaches a fork,
  and `is_withheld` is unmodified.
- Its *write-exclusion* limb, and the "fails closed on ambiguity" clause, turn on
  the dispatch confinement model — which **SL-237 does not change**. Recorded as
  [[QUE-199]] with the specific tension (`describe_mode`'s truth table vs
  `spec-012.md:273-274`'s "loses privilege" framing).

Leaving it visibly unresolved is the point: a future reader must be able to see
that it was swept, that the earlier adjudication was withdrawn, and where the
question went — rather than find it quietly preserved.

### 3d — the structured responsibility, § 2 In-scope, § 3 Principles, § verification basis

*Added after RV-322 F-12. The first pass of Row 3 swept PRD-015's **requirements
and invariants** and stopped there — but the same claim is asserted four more
times in surfaces a reader (or a query) reaches first.* All four say "absent" or
"never leaks", and after SL-237 the repo-scoped part of the tier is
reachable-but-not-copied.

**Before** (structured `responsibilities`, `spec-015.toml:13`)

> Guarantee that no isolated worker writes doctrine-authored state, and that the
> coordination/runtime tier never leaks across the isolation boundary.

**After (proposed)**

> Guarantee that no isolated worker writes doctrine-authored state, and that the
> coordination/runtime tier is never duplicated across the isolation boundary —
> an isolated unit holds no second mutable copy of it.

*Added after RV-322 round 5 (F-12). This is the **queried** tier — the round-4
sweep read `spec-015.md` and stopped, which is precisely the two-tier defect this
REV had raised against DEC-098 one round earlier and then repeated.*

**Before** (§2 In scope)

> non-leakage of the coordination/runtime tier across the isolation boundary

**After (proposed)**

> non-duplication of the coordination/runtime tier across the isolation boundary
> — no second mutable copy in an isolated unit, and no worker write to the tier

**Before** (§3 Principles)

> - **The isolation boundary is enforced by construction, not trust.** The
>   coordination/ runtime tier is *absent* from an isolated unit, so there is
>   nothing shared-mutable to corrupt — not present-but-trusted-not-to-be-touched.

**After (proposed)**

> - **Isolation is achieved by not duplicating shared state, not by trusting
>   units with a copy of it.** No *copy* of the coordination/runtime tier exists
>   in an isolated unit, so nothing it holds can diverge — not
>   present-but-trusted-not-to-be-touched. Where that tier is repo-scoped it is
>   single-homed, which a unit may read; writing it is reserved to the
>   coordinator.

**Before** (§ verification basis)

> Isolation non-leakage is verified by construction — the coordination/runtime
> tier is demonstrably absent from a provisioned unit even under a
> maximally-broad allowlist —

**After (proposed)**

> Isolation non-duplication is verified by construction — the coordination/
> runtime tier is demonstrably *never provisioned into* a unit even under a
> maximally-broad allowlist, so no second copy exists —

**Why.** This is the finding's real substance: revising Invariant 2 alone would
leave three earlier-read surfaces asserting the mechanism the invariant no longer
claims, and a reader who stops at §3 gets the falsified version. Note that the
allowlist sentence in the verification basis stays **true as written** — `WITHHELD`
covers `.doctrine/state/**` and is untouched — so it is reworded for precision
("never provisioned into"), not because it became false.

**Altitude.** *Corrected after RV-322 round 3.* A first draft of this row wrote
"Since SL-237 …" into the principle and named the OS layer as the enforcement
mechanism. Neither belongs in a PRD: a product spec states evergreen intent, and
citing the slice that changed it dates the document the moment the slice closes,
while naming the OS layer fixes an implementation choice that SPEC-012 owns and
may revise. The proposed text says *what is guaranteed* and leaves *how* to the
tech spec. Row 2 carries the mechanism detail, which is where it belongs.

**Net (revised after RV-322 round 5).** The delta is **one claim restated at four
altitudes**, plus one requirement revision:

| authority | surfaces |
|---|---|
| ADR-006 | D2 read parenthetical (1a), D4 per-worktree clause (1b), D9 provisioning ground (1c) |
| SPEC-012 | § tier merge-safety + responsibility bullet (2), structured `responsibilities` + rendered summary + § hypothesis (2) |
| PRD-015 | structured `responsibilities`, §2 in-scope, §3 principle, §4 Invariant 2, verification basis (3a/3d) |
| REQ-296 | wording + scope revision (3b) |

**Deliberately out of scope:** REQ-304 (3c → [[QUE-199]]) and the
platform-confinement gap ([[IMP-354]]). Both were argued inside this REV in
earlier drafts and are removed rather than resolved — they are questions about
the dispatch confinement model, which SL-237 does not change. Recording 3c as
*withdrawn* rather than *preserved* is deliberate: a future reader must be able
to see it was swept, that an earlier adjudication was wrong, and where the
question went.

## Provenance

Raised by **SL-237** during `/design`, 2026-07-29. Design rationale and the three
attacks run against the governing content — the "central index" clause, whether
withholding is load-bearing beyond merge safety, and whether primary resolution
even works from a confined fork — are recorded in
`.doctrine/slice/237/design.md` § 7. None of the three lands.

**Revised 2026-07-29 after external adversarial review RV-322** (raiser:
codex/GPT-5.5), which raised nine findings against the design, all accepted.
Three bear directly on this REV:

- **F-1** — scope was incomplete; PRD-015 was never swept. Row 3 added.
- **F-2** — the claude-arm "no OS floor" claim was **stale governance** that this
  REV would have propagated into SPEC-012. Corrected, with the ADR-008 staleness
  logged as a follow-up rather than silently amended.
- **F-4** — "restatement, not relaxation" was overreach. The named relaxation
  (concurrent RMW) is now stated in its own section.

`SL-237 needs REV-043` is authored (RV-322 F-7): implementation is gated on this
revision reaching `approved`, not merely on it having been raised.
