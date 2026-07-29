# REV REV-043 — Restate runtime-tier defence: absence to single-homed plus guard

Revision (ADR-013) — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

SL-237 single-homes repo-scoped runtime state (phase sheets, the source-delta
registry) in the primary worktree. Three authored statements assert the opposite
mechanism. They are **one claim at two altitudes**, so they are restated together
rather than in separate revisions — splitting them is what let them drift apart in
the first place.

**This is a restatement of mechanism, not a relaxation of safety.** That
distinction is the whole point of this REV, and every excerpt below is written to
preserve it.

### What does NOT change

- **Merge safety (ADR-006 D4's substance) is not weakened — arguably
  strengthened.** The named hazard was a *copied* phase sheet diverging across
  worktrees and merging back. Single-homing creates **no copy**. One file cannot
  diverge from itself.
- **The worker write prohibition (REQ-297, ADR-006 D2) stays fully enforced.**
  The CLI worker guard classifies `SliceCommand::Phases` → `Write("slice phases")`
  and `Phase` → `Write("slice phase")` (`src/commands/guard.rs:78,80`), both
  refused under `worker_mode`. On the codex/pi arm the prohibition additionally
  gains an **OS floor**: `scripts/pi-spawn-confined.sh:104-107` ro-binds the whole
  filesystem and rw-binds only the worker's own worktree, so the primary is
  physically read-only inside a worker namespace.
- **ADR-006 D4 clause 3 — "the `phases` symlink is relative" — is unchanged.**
  SL-237 (DEC-097) mints the convenience symlink *only in the primary*; where it
  exists it stays relative.
- **No central index or counter is introduced.** The phases directory is
  gitignored (`.gitignore:39`), per-slice, id-derived (nothing is minted or
  counted), and single-writer under D2/D9. Concurrent drives touch disjoint
  `<NNN>` directories. D4's warning that *"any future central index reintroduces
  conflict"* was tested against this design and does not fire.

### What DOES change

The **stated defence mechanism**, and one new **read** exposure.

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
> which is enforced by the CLI worker guard (D2a) and, on the codex/pi arm, by the
> read-only mount (ADR-008 D-B3).

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
> The **write** prohibition is enforced, not assumed absent: the CLI worker guard
> refuses `slice phase` / `slice phases` under `worker_mode`
> (`src/commands/guard.rs:78,80`, ADR-006 D2a), and on the codex/pi arm the nested
> bwrap profile mounts the primary read-only (ADR-008 D-B3), so the refusal has an
> OS floor as well as a CLI one. The claude arm has no OS floor — as is already
> true of the authored tier in a fork — so there it remains guard- and
> prompt-enforced.
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
the CLI worker guard and, where a subprocess exists to wrap, a read-only mount."*

**Why.** This is spec prose rather than a REQ, but it is a load-bearing
architectural claim, not incidental context. **No REQ requires change**: the
SPEC-012 roster (REQ-189..196, REQ-248..252) governs fork/provision/marker/env
contract, SPEC-021 governs orchestrator altitude, and PRD-015 FR-002 (REQ-297)
prohibits worker *writes* — which stays enforced. Verified in SL-237's pre-design
spec sweep.

---

## Provenance

Raised by **SL-237** during `/design`, 2026-07-29. Design rationale and the three
attacks run against the governing content — the "central index" clause, whether
withholding is load-bearing beyond merge safety, and whether primary resolution
even works from a confined fork — are recorded in
`.doctrine/slice/237/design.md` § 7. None of the three lands; that is why this is
a restatement rather than a contested decision.
