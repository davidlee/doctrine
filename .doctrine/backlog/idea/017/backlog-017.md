# IDE-017: Orchestrator-addressable worker provisioning when worktreeinclude grows divergent untracked state (SL-125 FU-1)

Origin: SL-125 design §2 limitation + Follow-up FU-1 (RV-111 finding F-4).

**Context.** SL-125 fixed ISS-011 Defect C by deriving the stamp provision SOURCE
from the repo's **primary worktree** (`primary_worktree(cwd)`), because the
`SubagentStart` payload carries no orchestrator location — the hook cannot name the
orchestrator tree. This is byte-equivalent to provisioning from the orchestrator
**only for the current `.worktreeinclude`**, which lists one static install
artifact (`.doctrine/doctrine.just`).

**The latent gap.** If `.worktreeinclude` ever grows to include genuinely
per-worktree-divergent *untracked* state that a worker must inherit *from the
orchestrator* (not the primary), the `SubagentStart` hook mechanism cannot supply
it — primary ≠ orchestrator then, and the hook still cannot address the
orchestrator tree. Resolution would need a different design: orchestrator-push
provisioning, or a payload side-channel carrying the orchestrator path.

**Trigger to act:** any addition to `.worktreeinclude` beyond worktree-invariant
static artifacts. Conditional, low-likelihood — captured so it is not rediscovered.
