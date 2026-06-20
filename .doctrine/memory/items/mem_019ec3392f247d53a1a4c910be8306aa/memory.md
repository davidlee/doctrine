# boot.rs HookSpec merge core is generalized over event+matcher

The `.claude/settings.local.json` hook merge core in `src/boot.rs` is the SINGLE
implementation that wires every doctrine-owned Claude hook. Before SL-056 PHASE-11
it was generic over `HookSpec{command, is_ours}` but **hardcoded** the event key
(`SessionStart`) and matcher (`SESSION_MATCHER = "startup|clear"`) inside
`desired_entry` + `session_start_array_mut`. PHASE-11 widened `HookSpec` with
`event: &'static str` + `matcher: &'static str` and threaded them through
`desired_entry(spec)`, `hook_array_mut(value, event)` (the renamed navigator), and
`fallback_for`.

So to add a new doctrine hook of ANY event, **add a `HookSpec` constructor — never
a parallel merge path**:

- `HookSpec::boot` / `HookSpec::sync` → `("SessionStart", SESSION_MATCHER)`.
- `HookSpec::stamp_subagent` → `("SubagentStart", crate::worktree::DISPATCH_WORKER_AGENT_TYPE)`,
  command `<exec> worktree marker --stamp-subagent`, ownership `is_doctrine_stamp_command`
  (suffix-strip — multi-arg, disjoint from the boot/sync predicates).

Callers `plan_hook` / `install_claude_hook` take `&HookSpec`. **SL-124 replaced
the old `find_owned` first-match + `set_command` with a `normalize` in `plan_hook`**:
collect `owned_positions`, no-write iff a single canonical doctrine-sole entry
already exists, else drop every owned hook (`drop_owned_hooks`) and insert one fresh
canonical entry at the first owned hook's execution slot. Ownership is now
**poison-tolerant** via the shared `is_doctrine_program` (strips a trailing
` (deleted)` before the file-name check); the three predicates (`is_doctrine_boot_command`
/ `_sync_command` / `_stamp_command`) defer their program check to it. `boot install`
wires boot+sync; `claude install` (SL-056) wires the stamp hook — all through
`install_claude_hook`, one core, no branching by event. Ownership predicates must
stay pairwise-disjoint so the entries never clobber each other. The matcher for a
`SubagentStart` hook is the agent-type literal and MUST come from the
`DISPATCH_WORKER_AGENT_TYPE` const (a drift test pins it) — never re-spell it.

Related: [[mem.pattern.distribution.skill-refresh-command]] (the `claude install`
rename), [[mem.pattern.dispatch.claude-subagentstart-worker-identity]] (why the
hook is SubagentStart-scoped).
