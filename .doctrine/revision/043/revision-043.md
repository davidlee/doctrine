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
- **The worker write prohibition (REQ-297, ADR-006 D2) stays fully enforced, and
  has an OS floor on BOTH arms.** *Corrected after RV-322 F-2 — an earlier draft
  of this REV asserted the claude arm had no OS floor, on the strength of
  ADR-008 D-B3's text that claude's `Agent` tool "cannot be wrapped". That text
  is **stale** relative to the shipped skill, and this REV would have written the
  staleness into SPEC-012.*
  - CLI guard, both arms: `SliceCommand::Phases` → `Write("slice phases")` and
    `Phase` → `Write("slice phase")` (`src/commands/guard.rs:78,80`), refused
    under `worker_mode`.
  - **codex/pi arm:** `scripts/pi-spawn-confined.sh:110` sets
    `--setenv DOCTRINE_WORKER 1` *unconditionally*, so `worker_mode` holds via
    the env leg regardless of the marker; `:104-107` ro-binds the filesystem and
    rw-binds only the worker's worktree.
  - **claude arm:** two installed PreToolUse hooks —
    `.agents/skills/dispatch-agent/SKILL.md:177-179` — put Bash into a nested
    bwrap (rw worktree, ro everything else) and deny `Edit`/`Write` outside the
    worktree.
  - **Follow-up, not silently fixed here:** ADR-008 D-B3's "claude-only concession"
    prose is stale and needs its own correction. This REV does not amend ADR-008.
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
- **PRD-015 FR-001 (REQ-296) — "the coordination/runtime tier absent by
  construction" — is PRESERVED, not revised.** The provisioning allowlist
  `is_withheld` (`src/worktree/allowlist.rs:170`; `WITHHELD` covers
  `.doctrine/state/slice/**`) is untouched by SL-237. **No copy is provisioned
  into a fork.** The tier remains literally absent from the fork's own tree; what
  changes is that a *path resolves outward*.
- **PRD-015 NF-003 (REQ-304) — "tier exclusion guaranteed by construction rather
  than a trusted check" — is PRESERVED**, with the construction relocated from
  provisioning-absence to the OS floor, on **both** arms (see below).

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
> The **write** prohibition is enforced, not assumed absent. The CLI worker guard
> refuses every write-class slice verb under `worker_mode` — `slice phase`,
> `slice phases`, `record-delta`, `reconcile-phases` (`src/commands/guard.rs`
> `write_class`, ADR-006 D2a) — and `worker_mode` is satisfied on **both** arms,
> by different legs: the codex/pi arm sets `DOCTRINE_WORKER` unconditionally at
> spawn, and the claude arm by the worker **marker** in the fork
> (`is_linked_worktree && marker_present`). Beneath the CLI refusal each arm has
> an **OS floor**: the codex/pi arm ro-binds the filesystem and rw-binds only the
> worker's worktree; the claude arm installs two PreToolUse hooks that put Bash
> in a nested bwrap (rw worktree, ro everything else) and deny `Edit`/`Write`
> outside it. The one deliberate opening is `worker_commit`, which commits on the
> worker's behalf from an *unconfined* server and is bounded by its own gate
> rather than by the wall — it writes no phase-state or registry file.
>
> The orchestrator's pre-distilled worker prompt (ADR-006 D6) still substitutes for
> coordination state a worker should not need; provisioning still substitutes for
> the absent execution environment. Note that pre-distillation is now a matter of
> *discipline* rather than *reachability* for the repo-scoped runtime tier — see
> ADR-006 D2's accepted read residual. No central authored index or counter exists
> to reintroduce a conflict.

The responsibility bullet near the top of the spec is restated to match: *"defend
the tier merge-safety invariant by non-duplication — the repo-scoped runtime tier
is single-homed, never copied into a fork — with the write prohibition enforced by
the CLI worker guard on both arms, under an OS floor on each."*

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
>   boundary: a worker holds no second mutable copy, so nothing it can touch can
>   diverge. Since SL-237 the repo-scoped part of that tier is single-homed and
>   therefore **readable** from a worker; its **write** exclusion is enforced by
>   the sole-writer guard and the per-arm OS floor, not by unreachability.

**Why.** A *read* now crosses the boundary. The invariant as written is falsified
by reach, even though nothing is copied and nothing may be written.

### 3b — FR-001 (REQ-296): REVISED, wording only

*Changed from "adjudicated, not revised" after RV-322 round 3 (F-1 re-contested).*

**Before**

> Each dispatched unit of work runs in isolation — a private working tree
> provisioned per run — with the coordination/runtime tier **absent by
> construction**, so nothing shared-mutable can be corrupted.

**After (proposed)**

> Each dispatched unit of work runs in isolation — a private working tree
> provisioned per run — with **no copy of the coordination/runtime tier
> provisioned into it**, so nothing shared-mutable can be corrupted.

**Why the earlier adjudication was wrong.** The first pass preserved this
requirement on the reasoning that its *mechanism* (the provisioning allowlist,
untouched) survives. That conflated mechanism with promise. The requirement's
words are "absent by construction", and a reader takes *absent* to mean
unreachable — which is now false: a path resolves outward and the tier is
readable from an isolated unit. Keeping the sentence because its implementation
still works would leave a product-level promise the product no longer keeps.

**What is preserved.** The requirement's *force* is untouched, because the
corruption it guards against comes from a second mutable copy, and there is
none. This is a wording revision that makes the requirement say what it has
always actually guaranteed — not a weakening. The allowlist evidence
(`src/worktree/allowlist.rs:170`) is what makes the new wording true, exactly as
it made the old wording's mechanism true.

### 3c — NF-003 (REQ-304): adjudicated, NOT revised

> Isolation integrity: tier exclusion is guaranteed **by construction rather than
> a trusted check**, and the sole-writer guard fails closed on ambiguity.

**Holds unchanged, with the construction relocated.** Write exclusion is OS-level
on both arms (see above), so exclusion remains *by construction* — it did not
degrade into a trusted check. Had only the CLI guard existed, this requirement
would have needed revising; the per-arm evidence is what preserves it.

### 3d — § 2 In-scope bullet, § 3 Principles, § verification basis

*Added after RV-322 F-12. The first pass of Row 3 swept PRD-015's **requirements
and invariants** and stopped there — but the same claim is asserted three more
times in prose that a reader will reach first.* All three say "absent", and after
SL-237 the repo-scoped part of the tier is reachable-but-not-copied.

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

> - **The isolation boundary is enforced by construction, not trust.** No *copy*
>   of the coordination/runtime tier exists in an isolated unit, so there is
>   nothing shared-mutable to corrupt — not present-but-trusted-not-to-be-touched.
>   Where that tier is repo-scoped it is single-homed rather than duplicated, so
>   an isolated unit may *read* it; what the construction guarantees is that the
>   unit cannot **write** it, enforced beneath the boundary rather than by the
>   unit's cooperation.

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
may revise. The proposed text now says *what is guaranteed* ("cannot write it,
enforced beneath the boundary rather than by the unit's cooperation") and leaves
*how* to the tech spec. Row 2 carries the mechanism detail, which is where it
belongs.

**Net:** PRD-015 needs **four** prose edits (Invariant 2, the in-scope bullet, the
principle, the verification basis) and **one** requirement wording revision
(REQ-296, row 3b). REQ-304 remains adjudicated-and-preserved; recording 3c that
way is deliberate — a future reader must be able to see it was swept and why it
held, rather than assume it was missed.

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
