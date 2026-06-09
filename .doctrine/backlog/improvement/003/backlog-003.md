# IMP-003: Dispatch worktree creation: detection and creation paths with guards

Implements ADR-006 D1 / D5 / D6 / D7. The orchestrator funnel and worktree
lifecycle for `/dispatch` (and the optional `/execute` path).

Scope:
- **Detection (D1):** `GIT_DIR != GIT_COMMON` with submodule guard; adapt rather
  than prescribe. Solo trunk-based path stays untouched.
- **Two creation paths (D5):** `/dispatch` (mandatory isolation via harness
  `Agent` isolation) and `/execute` (optional, native tools). Delegate the
  mechanism; never reinvent `git worktree`.
- **Guards (D5):** commit-before-spawn; branch-point check (HEAD pre/post-spawn
  mismatch → re-dispatch).
- **Funnel discipline (D2/D6/D7):** worker returns structured report + source
  delta; orchestrator pre-distills worker context; persists incrementally in
  order **import delta → verify → commit → record knowledge** on the coordination
  branch. Crash/overflow recovery = rebuild from coordination branch + `git
  worktree list`.
- **Worker vs solo (D6a):** worker-mode ON for funnel workers; OFF for solo agents.

Depends on IMP-002 (worker-mode guard, trunk-ref minting). Lands the `/dispatch`
skill (currently a placeholder). Governing: ADR-006.

## Design input — deterministic worker provisioning via `WorktreeCreate` (from SL-029 audit A-6)

SL-029 corrected F1: Claude Code's `WorktreeCreate` hook **ships**
(code.claude.com/docs/en/hooks). It *replaces* git worktree creation — the hook
makes the worktree and returns its path — and fires on `--worktree` /
`isolation: "worktree"` (the **worker dispatch** mechanism), NOT on the raw
`git worktree add` the `/execute` solo path shells (so it's irrelevant to solo;
relevant here).

Candidate funnel design: install a **Claude-only** `WorktreeCreate` hook that
deterministically runs `doctrine worktree provision <fork>` when a worker spawns
with `isolation: "worktree"` — closing the "relies on the worker agent
remembering to provision" gap at the harness seam. **provision stays the sole
copier** (the hook only *guarantees it runs*, it does not become a second copy
path — the copy-seam guarantee is preserved). Sharp edges to decide:
- **Interception scope.** The hook replaces ALL Claude worktree creation in the
  project, not just doctrine's — needs opt-in / scoping, not blanket install.
- **Portability.** Claude-only; a non-Claude funnel agent (codex, pi, …) has no
  hook → must fall back to skill-driven `git worktree add` + provision (rung 3).
  So the hook is an optimisation over the portable path, never a dependency.
- **Force-copy reconciliation.** If a project's hook body copies, the SL-029
  invariant degrades to `check-allowlist` only (design §2 caveat) — a
  doctrine-authored hook must be provision-only.
