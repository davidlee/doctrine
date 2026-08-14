# IMP-342: Subagent worktree-jail hook blocks read-only CLI reads from delegated research subagents

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Resolution — obsolete (2026-08-14, `SL-254` reconcile)

Same disposition as `IMP-269`, same cause: the hook is deleted. `SL-254` removed
`src/worktree/pretooluse.rs` and `src/worktree/subagent.rs` whole, so nothing
mediates a delegated subagent's `Bash` — read-only `doctrine` CLI reads included.

Named in `DEC-152`'s consequences alongside `IMP-269` as *plausibly discharging,
confirm at reconcile*. This is that confirmation.

Provenance: `SL-254` Follow-Ups `OQ-2` · `DEC-152` · `SL-247` `OQ-3`.
