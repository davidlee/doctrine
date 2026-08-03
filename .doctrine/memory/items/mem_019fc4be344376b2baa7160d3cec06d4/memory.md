# Cite scratch by tier, or a handover loses it

Found in SL-241 PHASE-05 starting T4c/H7, 2026-08-03.

Four consecutive handovers and a phase sheet cited `drivers/falsify-lib.sh`,
`drivers/shake.sh` and four `falsify-t4*.sh` drivers **by path**, as the trail
behind five scored falsification rounds. None had ever been git-tracked, none
was gitignored, and none existed anywhere on the filesystem — confirmed across
every worktree, the stash list, the whole of history, and a `find /` sweep.

The scored `results.tsv` survived, so the evidence was whole. What was lost was
the ability to **re-run** any prior falsification: those claims now rest on
prose in a gitignored runtime sheet.

## Why a path citation is not enough

A path looks identical whether it names an authored artifact, gitignored
runtime state, or untracked scratch in someone's working tree. The reader
cannot tell, and the writer — who can see the file — has no prompt to say. The
failure is silent and arrives a session later, when the tree has been cleaned.

## What to do

- **When a harvest or handover cites an artifact, name its tier**: committed at
  `<hash>` / runtime, regenerable by `<cmd>` / scratch, disposable.
- **If a claim depends on being re-runnable, the thing that reproduces it must
  be committed.** A driver whose output is cited as evidence is not scratch, it
  is part of the evidence.
- **Cheapest check before citing**: `git ls-files <path>` — untracked means say
  so, or commit it.
- **Do not reconstruct a lost driver from prose and re-run it under the old
  claim's name.** The rebuilt contract is inferred; its green is new evidence
  wearing an old label. State the asymmetry instead — which rounds are
  re-runnable and which are attested.

Rides the storage rule ([[mem.fact.doctrine.storage-tiers]]): authored,
runtime, derived. The gap here is that the rule governs where you *write*, and
says nothing about how you *cite*.

## The rule reaches the results file too (SL-241 close-out audit, RV-343 F-11)

The paragraph above says *"the scored `results.tsv` survived, so the evidence
was whole."* That held for one half of the spike and not the other.

The go/no-go's verdict was GO for the ingestion **and confinement** halves. The
ingestion half had committed authority — `results-c3.tsv`, with the corpus
README declaring it wins over any prose. The confinement half had none: P-C2's
seven scored rows lived only at `$SPIKE_CAPSULE_ROOT/probes/c2/results.tsv`,
outside the repository, in a scratch root the rig's own guard exists to keep
disposable. Not gitignored runtime state — **not in the repo at all**. Caught at
audit and copied in with hours to spare, by luck of timing rather than design.

The blind spot is specific and worth naming: attention goes to the artefact the
corpus points at, and the half nobody wrote a summary page for is the half whose
raw table nobody noticed was missing.

- **Check per claim, not per corpus.** If a verdict has *n* independently-scored
  parts, run `git ls-files` on the table behind each one. A corpus that names a
  TSV as "the authority" needs one for every claim it makes, not for the part
  that happened to get a write-up.
- **A scratch root is worse than gitignored runtime state, not equivalent.**
  Runtime state at least sits inside the tree and is named by a tier rule.
  Something under a scratch root has no rule pointing at it and no sweep that
  would find it.
- **Copy verbatim, do not summarise.** A hand-written summary reproduces the gap
  in a new form — the whole point of the TSV-is-authority convention is that the
  prose can be wrong and the table cannot.
