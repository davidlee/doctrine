# Primary-tree resolver reuse and the contextual review fork-ban

When a feature needs runtime state shared across worktrees (writer in one tree,
reader in another), two facts save rediscovery:

**Reuse the existing resolver.** `worktree::subagent::primary_worktree(cwd)`
(`src/worktree/subagent.rs:33`) already resolves the repo's PRIMARY working tree
via `git worktree list --porcelain` (first `worktree` entry), correct across
ordinary / separate-git-dir / submodule layouts — unlike `parent(--git-common-dir)`.
Do **not** invent a new "main tree root" helper; lift/share this one.

**The review fork-ban is contextual, not a universal "never co-write parent-tree
gitignored state" law.** `resolve_review_root` (`src/review.rs:1953`) bails when a
review verb resolves to a linked worktree, for two *specific* reasons: (1) a
fork's `WITHHELD` tier hides the parent tree's gitignored `.doctrine/state` from
the fork's filesystem view, and (2) the review baton uses an interactive
per-review CAS lock that cannot span forks. Neither transfers to a writer that is
the **un-jailed orchestrator / solo agent** doing a **single atomic write** under
**orchestrator-sole-writer** (ADR-006) — no fork view, no baton. Don't cite
review.rs:1953 as blanket precedent against cross-worktree state; check whether
the writer is a jailed worker and whether a CAS baton is actually in play.

**Why:** an external reviewer flagged "co-writing primary-tree gitignored state is
shunned" against SL-147's recorded-delta registry; the ban turned out
context-specific (jailed workers + interactive baton), and a sole-writer atomic
write is safe. The same review also caught a reinvented resolver.

**How to apply:** for cross-worktree runtime state, resolve the primary tree with
`primary_worktree`, make the orchestrator/solo session the sole writer, write
atomically. Reserve the fork-ban reasoning for verbs invoked *inside* a jailed
fork or guarded by an interactive lock.

Related: [[mem_019eb741539075c380783b4cff747fec]] (drive RV reviews from the parent
tree / merge-first — the fork-ban from the audit-driving angle). Origin: SL-147
design (RV-148, F-5).
