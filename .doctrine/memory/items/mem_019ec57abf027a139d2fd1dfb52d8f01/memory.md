# Stage-2 replay needs a 3-way CAS; zero-oid creation CAS would refuse the stage-1 refs

The `dispatch sync` verb has two CAS shapes — don't conflate them:

- **stage-1 prepare-review** *creates* `review/<slice>` + `phase/<slice>-NN` using
  **zero-oid CAS** (`update_ref_cas(ref, planned, ZERO_OID)`): git refuses if the ref
  already exists, so a crashed prior run's stale ref is reported, never clobbered.
- **stage-2 integrate** *replays* the journal idempotently using a **3-way CAS**
  (`git::replay_ref`): resolve the target's current oid (absent ↔ `ZERO_OID`) and
  compare against BOTH `planned_new_oid` and `expected_old_oid`:
  - `current == planned` ⇒ **NoOp** (already applied — crash-after-apply recovery),
  - `current == expected_old` ⇒ `update_ref_cas` to planned ⇒ **Applied**,
  - neither ⇒ **Moved** (refuse + report, never force).

**The trap:** replaying the stage-1 rows under zero-oid CAS would wrongly **refuse**
them — the refs already exist (`current == planned`, `expected_old == ZERO`), and
zero-oid CAS reads "ref exists ⇒ Moved". The 3-way compare is what makes replay a
verified no-op on intact refs and the idempotency / crash-recovery spine. This is the
ADR-012 D4 / design §4.1 contract (EX-2).

**Two companions in `src/dispatch.rs::integrate`:**
- **Trunk needs the ref NAME, not the ladder oid.** `git::trunk_commit` resolves the
  trunk *commit* but discards which ref; a CAS update needs the symbolic name. So
  `--trunk <ref>` is taken explicitly on the CLI (symmetric with `--edge <ref>`) —
  framework names roles, project binds refs.
- **Idempotent re-plan dedup.** A re-run must not append a duplicate trunk/edge row;
  integrate skips a target already journaled and replays the recorded intent — the
  crash-after-journal-plan recovery path.

ff-only trunk gate: `git::is_ancestor` (reads `merge-base --is-ancestor` exit code,
0/1/err — a clean false, not a git failure; cousin of the masked `cat-file -e` gotcha
[[mem.pattern.tooling.git-cat-file-e-exit-masked-use-ls-tree]]).

Cousin: stage-2 sources the journal from the branch tip TREE, not the filesystem
([[mem.pattern.dispatch.sync-tree-reads-ledger-not-worktree]]) — it runs after the
coordination worktree is removed, no checkout.
