# ISS-216: doctrine install cannot reseat a changed agent def (skip-if-exists + stale pre-subdir-split symlink)

Surfaced in SL-199 PHASE-05 (EX-6 reseat). Two compounding faults block a clean
re-materialization of a changed agent def:

1. **`doctrine install` is skip-if-exists, no `--force`/reseat.** When a shipped
   def changes (here: `install/agents/claude/dispatch-worker.md` gained
   `isolation: worktree`), install prints `skip … (exists)` for the materialized
   copy and does NOT refresh it. There is no supported path to re-expand an
   already-installed def from the (updated) embedded template.
2. **Stale symlink points at a pre-subdir-split path.** The live-read def the
   harness resolves is `.claude/agents/dispatch-worker.md` → `.doctrine/agents/dispatch-worker.md`
   (no `claude/` segment). Current install targets `.doctrine/agents/**claude/**dispatch-worker.md`.
   So even a forced reinstall would write a file the symlink never resolves, and
   the harness-read def stays stale.

Net: `doctrine install` alone cannot satisfy an EX-6-style "shipped == installed"
reseat. PHASE-05 worked around it by hand-adding the one frontmatter line to the
gitignored/derived live-read file (body was already correctly template-expanded).

Directions to weigh:
- an explicit `install --force`/`reseat` that re-expands existing defs;
- always re-expand agent defs (they are derived from `{{ prompt resolve }}`), only
  user-authored files stay skip-protected;
- repoint / regenerate the `.claude/agents` symlinks to the current subdir layout,
  and drop the pre-split `.doctrine/agents/<name>.md` leftovers.

Relates to the materialization layout; scope a design fork before touching install
semantics (skip-if-exists is deliberately protective of user edits).
