# Split-lineage close: reconciled truth on edge — unite via merge-into-edge before pre-FF+record

**Trigger.** Closing a `/dispatch`ed slice whose reconciled truth is stranded on
`edge`, not on `review/<slice>` or the admitted candidate. Happens when the
dispatch base (main) was never re-promoted from edge after audit, so `/audit` +
`/reconcile` (and any operator-run **corpus migration**) accumulate on edge on top
of the old base, while the code delivery sits on a candidate cut from that old
base. At close: three divergent lineages —
- **candidate** (admitted `review_surface`/`close_target`): the code, based on old main;
- **edge**: the migration + all reconcile authored content (design append-notes,
  the REV, the RV outcome, spec edits, the canonical selector registry);
- **main**: neither.
An authored file both sides touched (e.g. `slice-NNN.toml`) diverges on all three.

**Why the naive path is silently wrong.** The `dispatch status` "next" hint and the
handoff will say `dispatch sync --integrate --trunk main`. That projects the
**code-only candidate** onto main *without* the migration or reconcile content,
strands edge, and leaves a stale authored file — a broken close that still reaches
`done`. The status machine is lineage-blind; it doesn't know reconcile truth lives
on a sibling branch.

**The fix — unite edge⊕candidate first, then the standard pre-FF+record.** This is
[[mem_019f06a18bf97b23bf771740e427b639]] (pre-FF trunk so `close_target` absorbs the
repair) with a merge front-step, because neither `review/<N>` nor the candidate
carries the reconciled truth:

1. **Merge the code candidate INTO edge** (in the primary tree, on edge — a merge
   doesn't switch branches): `git merge --no-ff --no-commit <admitted-candidate-tip>`.
   Code/tests apply clean (edge has no src). Resolve the authored-file conflict
   (`slice-NNN.toml`) to **edge's canonical reconcile version**
   (`git checkout --ours -- <path>`); verify the resolved blob == edge's blob.
   Commit → unified tip `M`. This also makes the candidate tip a **genuine ancestor
   of trunk**, which the `close_target` provenance/record check requires.
2. **Gate `M`** — first time code + migrated corpus coexist; build + `just gate` +
   run the slice's e2e + smoke a real corpus verb (`doctrine survey`) before
   touching main.
3. **FF main → M** (CAS: `git update-ref refs/heads/main M <old-main>`; verify
   old-main is an ancestor). edge is already `M`. main checked out nowhere.
4. **Create + admit a no-op `close_target`**: `dispatch candidate create --role
   close_target --payload code --base refs/heads/main --source refs/heads/review/<N>`
   — `review/<N>` is now an ancestor of main, so the 3-way merge is tree-identical to
   main (verify `cand^{tree}` == `main^{tree}`); `candidate admit --role close_target`.
   (The pre-existing `review_surface` admission is not a trunk payload — the gate
   rejects `review/<N>`; you need a real `close_target`.)
5. **`dispatch sync --integrate --trunk refs/heads/main`** — a ff no-op (+1 empty
   merge commit `f4…`) whose only job is to record the journal trunk row. Verify
   ISS-030 tree-true ([[mem_019ec912f7fd746284bfaef00717443e]]): `git diff --quiet
   HEAD` (excl. unrelated dirty paths) and `--show-journal-trunk-oid` == main.
6. **FF edge → the integrate tip** so `edge == main` again. Then `slice status <N>
   done` (the recorded row satisfies the close gate), commit the authored close
   artefacts, FF main → that.

**Contrast.** [[mem_019f06a18bf97b23bf771740e427b639]] and
`close-deadlock-refresh-base-recovery` assume the reconciled truth is already on
`review/<N>` or reachable by `refresh-base` into the coord branch. Here it is on
**edge**, so the front-step is a plain `git merge` into edge — the coord worktree is
gone by close time and the migration/reconcile were authored there, not on the
bundle. Root cause is the same edge/main-vs-off-edge-code tension that generates
recurring split-lineage close friction (see RFC-011 case-notes, SL-198/SL-220). A
pre-close check diffing the admitted `close_target`'s tree against edge (flagging
migration/authored divergence) would surface the 3-way split in one command.
Related dead-end if you skip the unite step: [[mem_019ee4bac0597bf0809caf56b0e59466]]
(candidate verb can't ingest a hand-resolved merge). First applied: SL-220 close.

## Second application — SL-231 close, 2026-07-28

Recipe applied verbatim and worked. Same signature: code + audit repairs on the
admitted candidate, `/audit` + `/reconcile` on edge, main at neither, and
`slice-231.toml` forked three ways. Two refinements worth pinning.

**Take-both, not always `--ours`.** Step 1 above says resolve the authored-file
conflict to edge's version. That is the *default*, not the rule. Here the
candidate carried a selector edge never had (`src/install.rs`, appended by
PHASE-05 on the dispatch branch and never promoted), so `--ours` would have
silently dropped it. The audit's `## Reconciliation Outcome` had named the
resolution in advance — **read it before resolving**, and verify the count it
predicts (23 selectors) rather than trusting either side wholesale.

**The check that settles `--payload code` vs `impl_bundle`.** This memory says
`code`; [[mem_019f9433d43d7bc186f5de9e2214f3b4]] says `impl_bundle`, "not code",
because `code` re-strips the authored corpus. Both are right for their case, and
the disagreement is not resolvable by reading either. The discriminator is a
one-line check no skill states — run it after `candidate create`, **before**
`candidate admit`:

```bash
[ "$(git rev-parse <cand>^{tree})" = "$(git rev-parse main^{tree})" ]
```

In the split-lineage shape the close_target is a **no-op by construction** (edge
already absorbed the code, then main was FF'd to it), so a tree-identical
candidate proves the projection cannot strip anything — whichever payload label
was passed. If the trees differ, the candidate is NOT a no-op: stop, do not
admit, and re-derive. `candidate create` writes no ref or row on failure and the
admission is reversible, so this check is free. SL-231 used `impl_bundle` and
verified identity; the label turned out not to matter once identity held.

**Structural root cause, restated.** This is the fourth close to hit it
(SL-198, SL-204, SL-220, SL-228, now SL-231). It is not bad luck: main is
promoted from edge only *before* dispatch, so any `/audit` or `/reconcile` work
— which is most of the lifecycle's authored output — is guaranteed to land on
edge after the fork. Split lineage is the **normal** close shape for a
dispatched slice, and the `/close` skill's step 3a happy path
(`--source refs/heads/review/<N>`) describes the exception. Filed as an
observation at the SL-231 close.
