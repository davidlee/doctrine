# IMP-002: Worker-mode CLI guard and trunk-ref id allocation with reseat

Implements ADR-006 D2a / D3. The doctrine-mediated enforcement half of the
worker-sole-writer invariant, plus fork-safe id allocation.

Scope:
- **Worker-mode guard (D2a):** `DOCTRINE_WORKER=1` makes the CLI hard-refuse every
  doctrine-mediated authored write (`memory record`, `slice/adr/spec new`, status
  transitions, doctrine-driven commits). Covers exactly the writes that mint ids
  and anchor memory. (Raw-tree confinement is out of scope — ADR-006 D2b / IMP-004.)
- **Trunk-ref id allocation (D3):** allocate against the configured trunk ref
  (auto-detect `origin/HEAD` → `main`/`master`, overridable) so ids are minted
  trunk-side before a worktree forks.
- **Reseat (D3 fallback):** `validate` detects duplicate ids (cross-branch offline
  collisions); a reseat verb renumbers a colliding entity.
- **Memory-record worktree warning (ADR-006 amendment):** `memory record` detects
  worktree context (`GIT_DIR != GIT_COMMON`) and warns on squash-orphan risk.

Governing: ADR-006 (D2a, D2b boundary, D3, solo-in-worktree amendment).
