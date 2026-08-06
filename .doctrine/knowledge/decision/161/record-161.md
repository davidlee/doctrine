# DEC-161: HookSpec carries an ordered matcher set

## Decision

`HookSpec`'s `matcher: &'static str` becomes an ordered set
(`matchers: &'static [&'static str]`). Ownership stays proven by **command
alone** — `owned_positions` (`src/boot.rs:1039`) is unchanged. `desired_entry`
emits one entry per matcher, and `plan_hook`'s no-write short-circuit
generalises from "exactly one canonical, doctrine-sole entry" to "exactly the
canonical set, in order". Existing specs pass a one-element slice.

Rejected: widening the ownership predicate from `command` to
`(command, matcher)`.

## Why

SL-250's `R5` — the merge core cannot represent the hook sets being moved off
the plugin. Four `PreToolUse` entries share the command `worktree pretooluse`
(matchers `Bash`, `Edit|Write`, `Agent`, `Workflow`) and two share `memory
surface` (`Read|Edit|Write`, `Bash`). Because ownership is command-only and
`plan_hook` (`src/boot.rs:1182`) collapses every owned hook to a single
canonical entry, all four are marked owned and three are silently destroyed.

The two candidate shapes diverge on **what happens when a matcher set later
changes**, and the slice already contains a live instance:

- Under `(command, matcher)` ownership, an entry whose matcher has moved stops
  being owned. It becomes foreign, is never touched again, and both the stale
  and the fresh entry fire — permanently, and with nothing to detect it.
- Under command-only ownership the stale entry is still owned, so the normalize
  drops and rewrites it onto the canonical matcher. The drift heals on the next
  install.

That is not hypothetical. SL-250's design-surface triage finding `T2` records
that `plugins/doctrine/hooks/hooks.json` ships the `SessionStart` hook as
`boot --emit` on matcher `*`, while the code's canonical form is
`prompt resolve --role orchestrator` (`RESOLVE_EMIT_ARGS`, `src/boot.rs:862`) on
`startup|clear` (`SESSION_MATCHER`, `:554`) — and `is_doctrine_emit_command`
(`:934`) deliberately recognises both arg forms. Widening ownership would
manufacture exactly the orphan `T2` describes, for every future matcher edit.

Two further reasons:

- **The behaviour-preservation gate holds by construction.** The matcher-set
  shape is a strict generalisation with N=1 as the existing case, so
  `corpus.rs`'s memory-sync hook and the Codex arm are unaffected. Widening
  ownership *narrows* the predicate instead: a stale doctrine entry sitting on a
  non-canonical matcher is adopted-and-refreshed today and would be orphaned
  after — a behaviour change on four already-shipping specs.
- **The safety-critical part is left alone.** Ownership is what stands between
  the merge core and a user's own hook entries; this shape does not touch it.

The hybrid — widen ownership *and* keep a retired-matcher list to sweep orphans
— buys the same healing at the price of permanent bookkeeping, which SL-250's
less-code posture rules out.

## Consequences

- The cost is modestly more logic inside `plan_hook`'s normalize. The rejected
  option's "less code" is paid for with permanent, undetectable double-fires,
  and SL-250 carves the doctor leg out to IMP-407, so nothing downstream would
  catch them.
- Entry identity becomes the *set*, so a partial hand-edit (deleting one of the
  four entries) is healed on the next install. This stays inside the
  never-clobber contract: only doctrine-owned entries are touched.
- Inserting N entries at the first owned hook's slot preserves position relative
  to foreign entries the same way the current single insert does.
- The new matcher tokens need named constants beside `SETTINGS_REL` (STD-001).
- Does **not** address `R10`'s cross-scope double-fire, which spans two settings
  files and is out of the merge core's reach by design
  (`install_hook_to_file` normalizes within the single file it writes). That is
  the run's `inq-5`.

Settles SL-250's `R5`. Recorded from design run `dr-019fd692` checkpoint `cp-1`
disposing `inq-2`.
