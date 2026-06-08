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
