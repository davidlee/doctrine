# Gitignored-tier partition for worktree dispatch (ADR-006 D9)

A worktree fork carries only **committed** files. For parallel dispatch the
gitignored tier is **not monolithic** — partition it by fork-provisioning policy:

- **withhold (coordination/runtime)** — `.doctrine/state/` (phase sheets,
  `boot.md`), the `phases` symlink, `handover.md`, memory `index/embeddings/state`
  caches. Their **absence in the fork is load-bearing**: it is what makes the
  worker-sole-writer invariant (ADR-006 D2) free — a copied phase sheet would be
  invisibly mutable across worktrees, re-opening the hazard. D6's pre-distilled
  prompt substitutes for the withheld state.
- **provision (execution prereq)** — what the worker needs to build/run/verify.
  Two axes:
  - **regenerate** the derivable from committed sources (`cargo build`,
    `npm install`) — superpowers `using-git-worktrees` Step 3.
  - **copy** only the *irreducible* gitignored files (secrets, local config) via a
    **project-owned allowlist** (`.worktreeinclude`, gitignore syntax). Claude Code
    feeds it through the `WorktreeCreate`/`WorktreeRemove` hooks.

**Invariant:** the provisioning allowlist **excludes the coordination tier by
construction** (check it against the known runtime globs). Then **baseline-verify**
the fork builds+tests green *before* dispatch — an unbuildable fork is fixed in
provisioning, never handed to a worker.

**Creation is a preference ladder** (D5, battle-tested by superpowers): detect
existing isolation (`GIT_DIR != GIT_COMMON` + submodule guard) → native worktree
tool → `git worktree add` fallback → work-in-place (solo, no funnel) on sandbox
denial. Doctrine's jail instantiates the regenerate axis with a per-worktree
`CARGO_TARGET_DIR` (ADR-008 D-B1).

Prior art: superpowers `using-git-worktrees`; Claude Code `WorktreeCreate` +
`.worktreeinclude` (mattbrailsford.dev GitHub discussion #54). Governs IMP-003.
Related: [[mem.concept.doctrine.storage-model]], [[mem.concept.doctrine.routing-gate]].
