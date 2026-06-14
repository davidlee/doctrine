# Dispatch candidates for safe audit interaction

> Slug `wire-dispatch-stage-2-sync-integrate-into-close` is stale (original
> mis-scope); slug is non-authoritative. Title/scope below are current.

> **Repro done 2026-06-15 — see `notes.md` for the verified verdict.** It
> corrected this framing: the projection-base defect (live-tip parenting) is
> **already fixed by RV-030** (post-dated SL-067's dispatch); the live gap is
> *no safe ordinary branch/worktree surface for audit-time review, fixes, and
> experiments after dispatch output is prepared*. The "stale base" language from
> the original title is superseded by notes.md F1-F6.

## Context

The `/dispatch` SL-067 run produced `review/067` + `phase/067-*` deliverables
that **cannot be landed**:

```
diff main..phase/067-02   →  2317 deletions, 0 additions
src/revision.rs:  present on main,  ABSENT on phase/067-02
```

The whole post-fork main (the REV feature, the entire `.doctrine` corpus) reads
as deleted. Cause: the projected phase branches are parented on `91b05c4`
(18 behind main), **not** on the coordination base `26a3125` where the funnel
actually built the units.

Key distinction — the funnel was sound, the **projection** was not:

```
coordination base (dispatch/067):  26a3125   (PHASE-01 code_start; boundaries.toml clean)
funnel phase chain:                26a3125 → 777c76e → 6e89e12   (coherent)
projected phase/067-01 parent:     91b05c4   ← stale, ≠ coordination base
```

`dispatch sync --integrate --trunk main` is **fail-closed**: `is_ancestor`
(`plan_trunk_row`, dispatch.rs:359) trips because the stale-based phase tip does
not fast-forward main → *"trunk moved; re-anchor required, not auto-resolved."*
So trunk was never silently wiped. But there is **no recovery path**: the verb
refuses, or you force-merge the 2317 deletions by hand (the SL-067 salvage
nightmare). The run's deliverables are dead either way.

The **original SL-068 premise was wrong twice**: wiring `--integrate` into
`/close` would not have landed SL-067 (it refuses), and the missing wiring was
not the bug. RV-030 fixed the projection-base defect for fresh runs. The
remaining defect is workflow shape: exact dispatch outputs look like ordinary
branches, but are audit artifacts. Humans and external reviewers naturally treat
them as normal work branches, which turns audit-time review/fix interaction into
a trap.

The instructive failure mode: a reviewer interpreted a dispatch worktree as a
dangerous branch, raised findings into an RV, dispositioned them, then applied
"fix-now" changes in-place on the branch. Doctrine needs to support that kind of
audit interaction deliberately, not by leaving special refs to masquerade as
mergeable branches.

## Scope & Objectives

1. **Preserve exact dispatch evidence.** `dispatch/<slice>`, `review/<slice>`,
   and `phase/<slice>-NN` remain immutable audit/evidence refs. They are not
   redefined into normal feature branches.
2. **Add a safe interaction surface.** Dispatch must materialise an explicit
   candidate branch/worktree on a chosen base for audit-time review, local
   testing, "fix-now" edits, and experiments against other features.
3. **Admit audit-time changes deliberately.** If review fixes are made during
   audit, the accepted OID must be recorded as an admitted review-surface or
   close-target candidate, not left stranded on an ambiguous worktree branch.
4. **Keep trunk protected.** Candidate creation may use an explicit 3-way merge
   onto a chosen base; final trunk integration remains opt-in, post-audit,
   expected-tip guarded, and refuses moved trunk. Close never recreates, updates,
   rebases, or merges a candidate.
5. **Make outputs self-describing.** Commands and status views must distinguish
   evidence refs from candidate branches and point users/reviewers to the safe
   interaction path.

## Non-Goals

- Retrofitting old SL-067 refs. Those refs were cut before RV-030 and remain sunk
  evidence.
- Making `review/<slice>` or `phase/<slice>-NN` themselves ordinary merge
  branches. They stay exact projections for audit.
- Auto-landing to trunk during dispatch conclude. Close remains post-audit.
- Codex/pi worker-fork-base mechanics, unless the repro implicates them.
- Solving every branch-policy style. The design admits multiple workflows through
  candidates; repo-specific policy still decides which candidate, if any, lands.

## Summary

Design direction: add a dispatch candidate/admission layer. Stage-1 sync still
emits exact evidence refs. A new candidate workflow creates normal branches from
those refs on an explicit base, with review-surface and close-target roles kept
distinct. Reviewers and humans interact with candidates, not raw dispatch refs.
Audit fixes are admitted by recording immutable OIDs against the dispatch run;
close integrates only the admitted close-target OID under the existing post-audit
guard.

## Follow-Ups

- `/close` stage-2 integration wiring remains secondary and should target the
  admitted close-target OID, not the raw phase tip or a mutable candidate ref.
- ADR-006/ADR-012 need a narrow amendment: explicit candidate materialisation may
  3-way merge onto a chosen base, while exact refs and trunk integration remain
  fail-closed.
