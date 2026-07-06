# dispatch MCP funnel lands object-db commits — coord working tree goes stale (staged reversal)

dispatch_import/dispatch_conclude_phase land commits object-db-only; the coord working tree+index stay at pre-import content, showing as STAGED REVERSALS of the landed delta. A pathless git commit there would silently revert the import. Remedy: git restore --source=HEAD --staged --worktree -- <delta paths> before any suite run or commit in the coord tree.
