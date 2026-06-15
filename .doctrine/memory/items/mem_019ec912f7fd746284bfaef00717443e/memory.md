# Close a dispatched slice by landing the admitted close_target with sync --integrate --trunk main

`/close` for a slice whose code was built under `/dispatch` (and never landed on
main — `dispatch sync --prepare-review` at conclude writes only `review/<slice>` +
`phase/<slice>-NN`, never trunk) lands the code via the candidate path the dispatch
machinery provides. The full close-time sequence (run from the parent/root tree,
`DOCTRINE_WORKER` unset, `DOCTRINE_TRUNK_REF=main` because origin/main is stale):

1. `dispatch candidate create --slice N --label close-001 --role close_target
   --payload impl_bundle --base refs/heads/main --source refs/heads/review/N`
   — computes the no-ff 3-way merge of the bundle onto the *current* main
   (merge-base = the old fork base), so all of main's post-fork authored state is
   preserved and only the bundle's delta is layered. Aborts clean on conflict
   (no ref/row/worktree) — a safe, reversible probe.
2. `dispatch candidate admit --slice N --role close_target --candidate
   refs/heads/candidate/N/close-001 --review RV-NNN` — pins `admitted_oid`; writes
   only `candidates.toml`. Reversible.
3. `dispatch sync --slice N --integrate --trunk refs/heads/main` — the only
   committing step. Projects the admitted close_target onto main, ff-only + CAS.

**`--integrate` ALONE LEAVES TRUNK UNTOUCHED — `--trunk refs/heads/main` is
required to move main.** Handover prose that writes bare `--integrate` is wrong;
the CLI help is the source of truth. (`--edge` is the separate aggregate-ref opt-in.)

**The integrate moves the `main` ref but does NOT touch the working tree or index** —
they lag at the old tip, so `git status` then shows the just-landed files as
deleted/modified-away. Resync the *landed paths only* with
`git restore --source=HEAD --staged --worktree <paths>` — disjoint from any
unrelated WIP (e.g. a later slice's untracked docs), which stays untouched.

**Bootstrap:** the candidate verbs ship IN the slice being closed (SL-068), so the
installed/main `doctrine` predates them. Build the candidate-aware binary from
`review/<slice>` first (`git worktree add` + `cargo build` with a dedicated
`CARGO_TARGET_DIR`), then run create/admit/integrate with it. After integrate,
main has the code — a normal `cargo build` rebuilds the candidate-aware binary.

The `impl_bundle` payload legitimately carries the orchestrator's PHASE-07 authored
deliverables (skill pointers, AGENTS.md, memory masters) on top of code — that is how
non-dispatchable authoring reaches main when it was committed on the coordination
branch. See [[mem.pattern.dispatch.candidate-build-seam]] for the create/admit seam
internals and [[mem.pattern.dispatch.gc-squash-indistinguishable-from-unlanded]].
