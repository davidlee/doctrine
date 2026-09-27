# Primary-tree resolver reuse (review fork-ban retired)

When a feature needs runtime state shared across worktrees (writer in one tree,
reader in another), two facts save rediscovery:

**Reuse the existing resolver.** `worktree::subagent::primary_worktree(cwd)`
(`src/worktree/subagent.rs:33`) already resolves the repo's PRIMARY working tree
via `git worktree list --porcelain` (first `worktree` entry), correct across
ordinary / separate-git-dir / submodule layouts — unlike `parent(--git-common-dir)`.
Do **not** invent a new "main tree root" helper; lift/share this one.

Review admission is now worker-only, see
[[mem.pattern.review.writes-refused-only-in-worker]].

Related: [[mem_019eb741539075c380783b4cff747fec]] (superseded — the fork-ban
from the audit-driving angle, kept for history). Origin: SL-147 design
(RV-148, F-5).
