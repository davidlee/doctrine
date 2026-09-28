# Review writes are refused only in a dispatch worker process

Since SL-269 (DEC-338), `resolve_review_root`'s admission check
(`admit_review(env_worker_set())`) refuses every review write — `new`, the
turn verbs, `status`, `prime`, `unlock` (MCP tools too) — only when
`DOCTRINE_WORKER` is set. Branch shape (`dispatch/<NNN>`) no longer matters at
all: the primary checkout, a dispatch coordination tree, a solo `/worktree`
fork, and an adopted capsule tree are all admitted. Workers still read with
`review show` / `review list`.

The turn baton lands in the resolved tree's own gitignored runtime state — a
cache, rebuilt from the ledger wherever the review is next used. So audit and
close run in whichever linked tree holds the code; there is no merge-first
relocation to a primary or coordination tree before driving the ledger.

**Rule that survives:** one writer per RV at a time, in whichever tree. The
per-review lock and hash checks protect writers inside one tree only; git
merge of the ledger is a best-effort backstop that may conflict, or merge
cleanly into an incoherent ledger. Enforcement lives at the design decision
that replaced the branch-shape test with the worker-process test.

**What tightened, not just loosened:** a worker process whose root resolves
to a non-linked tree (e.g. a bare clone reused as a worker) is now refused too
— the old branch-shape guard would have let it through. The design-run mint
path (`review new` invoked mid design-run) is covered by the same CLI worker
guard, not a separate check.

See [[mem.pattern.worktree.primary-tree-resolver-and-contextual-review-fork-ban]]
for the resolver this admission check builds on.
