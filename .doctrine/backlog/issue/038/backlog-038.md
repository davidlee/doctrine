# ISS-038: Dispatch stage-2 integrate against a dirty/shared trunk checkout leaves a phantom index a later commit silently reverts

Surfaced during SL-122 close. The slice code was fully audited + green, yet briefly
vanished from `main`. Root cause is a **mechanism defect**, not a one-off.

## The failure chain

1. `doctrine dispatch sync --slice N --integrate --trunk refs/heads/main` advanced
   the `main` **ref** to the admitted close_target merge (which contained the full
   code). Correct so far.
2. The integrate ran against the **shared `main` working tree**, which was **dirty**
   (a concurrent agent's uncommitted work on other slices). Integrate moved the ref
   but did **not** sync the checkout — leaving the **index** holding the slice's code
   files as **staged reverse-deletions** relative to the new HEAD. This is the
   ISS-030 phantom: `git diff --quiet HEAD` reports dirty, the close skill says STOP.
3. A subsequent `.doctrine` commit (made to protect untracked audit artefacts) rode
   that **stale index** — committing the staged reverse-deletions — so `main`
   advanced to a tree with the slice's **docs but not its code**. The integration was
   silently reverted by an unrelated commit.

Recovery (SL-122): `git checkout <admitted-merge> -- <code paths>` to re-materialise
the code, re-commit, verify in a clean worktree. The code was never lost (intact in
`review/N` throughout).

## Why it warrants a fix

- **Integrate must fail-closed on a dirty trunk checkout.** The dangerous step is
  advancing the trunk ref while the shared checkout is dirty. A hard pre-gate
  (`git diff --quiet HEAD && git diff --cached --quiet` on the trunk worktree, refuse
  if dirty) before `--integrate` would prevent the phantom entirely. Alternatively,
  integrate should operate via a **dedicated clean worktree / pure ref CAS** and
  never depend on the shared checkout state.
- **The ISS-030 STOP needs a defined recovery, not just "STOP."** The close skill
  fires the detector but gives no remedy. It must state: the index now holds the
  projection as staged reverse-deletions; **do NOT commit anything** (even unrelated
  files) until the checkout is resynced to the advanced ref — any commit captures the
  phantom and reverts the integration. That exact trap is what bit here.
- **Multi-agent hazard.** Stage-2 integrate moves the shared `main` ref and needs the
  shared checkout clean; doing it while another agent commits to `main` is unsafe.
  The flow should require exclusive control of the trunk checkout or be checkout-
  independent.

Related: RSK-010 (stale dispatch base — the earlier, separate friction on the same
drive). ISS-030 (the phantom-reverse-diff detector this exploits).
