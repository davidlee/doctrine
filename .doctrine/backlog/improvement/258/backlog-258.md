# IMP-258: Funnel/instrumentation collision — case-notes append dirties coord tree

## Problem

RFC-011's instrumentation directive instructs the orchestrator to append case
notes to `.doctrine/rfc/011/case-notes.md`. During dispatch, the orchestrator's
Bash cwd is parked in the **coord** tree for the full drive loop. A case-note
appended there dirties the coord working tree, and `worktree import` fails
closed with `import-refused: tree-unclean` — the instrumentation directly breaks
the next phase's funnel precondition.

Witnessed twice:
- SL-196-conclude: case-note append dirtied coord → `worktree import` failed
  → had to relocate the append to the primary tree and `git restore` the coord
- SL-166-close-b: an empty case-note stub header dirtied the tree → `worktree
  land` refused with `land-refused: tree-unclean`

The fix in both cases was to target the **primary** tree's case-notes.md
(`-p /workspace/doctrine` semantics) and restore the coord copy, but the
instrumentation directive doesn't specify this.

## Fix direction

- **Instrumentation directive**: specify that case-notes appends must target
  the PRIMARY/session-root tree, never the coord tree. Append with
  `--path <primary-root>` or `cd` to primary root before appending.
- **Funnel precond**: optionally, `worktree import` could exempt known
  instrumentation paths (`.doctrine/rfc/`) from the clean-tree check, but
  this weakens the belt — prefer the directive fix.
- **Skill note**: `/dispatch` and any skill carrying the instrumentation
  directive should include the primary-tree redirect.

## Related

- RFC-011 instrumentation directive
- SL-196-conclude, SL-166-close-b case-notes
