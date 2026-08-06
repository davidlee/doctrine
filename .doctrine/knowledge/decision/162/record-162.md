# DEC-162: Claude hooks install on one scope dial

## Decision

All ten hook entries currently in `plugins/doctrine/hooks/hooks.json` are
written directly through the `boot.rs` merge core, at the **single**
flag-selected scope `OQ-1` settled (default: project `.claude/settings.json`).
No per-entry scope routing, and no entry is dropped.

## The inventory: ten entries, six specs

Under [[DEC-161]]'s ordered matcher set the ten entries collapse onto six
`HookSpec`s, one of which already exists:

| entries | spec |
|---|---|
| `SessionStart` → `boot --emit` on `*` | **new** Claude emit spec — canonical args `RESOLVE_EMIT_ARGS`, canonical matcher `SESSION_MATCHER` |
| `WorktreeCreate` → `worktree create-fork` | `HookSpec::create_fork` — **already exists**, held `expect(dead_code)` |
| `SubagentStart` → `worktree nominate` | new, matcher `dispatch-orchestrator` |
| `SubagentStop` → `worktree denominate` | new, matcher `dispatch-orchestrator` |
| 4 × `PreToolUse` → `worktree pretooluse` | **one** new spec, matcher set `Bash`, `Edit\|Write`, `Agent`, `Workflow` |
| 2 × `PreToolUse` → `memory surface` | **one** new spec, matcher set `Read\|Edit\|Write`, `Bash` |

The table sums to ten: 1+1+1+1+4+2. An earlier revision of this record headlined
it as nine while the table already said otherwise; the count was corrected in
place (RV-348 `F-1`), and SL-250's `sec-1` now carries the four-altitude ledger
that keeps entries, specs, emitted entries and printed lines apart.

This confirms SL-250's triage finding `T1` ("~6 new specs, not a constructor").
The existing `HookSpec::sync` (`memory sync`) is already written directly and is
untouched by this slice beyond the scope dial — it never shipped in the plugin,
which is why seven specs cover ten plugin entries.

### `T2` resolves mechanically, not by choice

The plugin's `SessionStart` entry is **not** `HookSpec::boot`'s hook.
`is_doctrine_boot_command` (`src/boot.rs:891`) requires the trailing arg to be
literally `boot`, which `boot --emit` fails. It is `boot_emit`'s hook, and
`is_doctrine_emit_command` (`:932`) recognises `LEGACY_EMIT_ARGS` explicitly so a
stale copy "self-heals to the resolve command on the next install".

So doctrine writes the canonical `prompt resolve --role orchestrator` on
`startup|clear`, and DEC-161's healing property retires the plugin's stale `*`
entry automatically. `HookSpec::boot` stays dead — nothing ships it.

## Why one dial

The six `PreToolUse` entries do spawn up to two doctrine processes per tool call
(`Bash` matches both `worktree pretooluse` and `memory surface`), and
`memory surface` changes what every reader's agent *sees* rather than what
doctrine does. But **none of this is a new cost** — the plugin fires exactly
these entries today. Retirement changes where the entries live, not what runs.

What is genuinely new is that they become *committed*, and that is precisely what
`OQ-1` settled: a scope flag, with `settings.local.json` available for a
collaborator who must not impose hooks on a client team. A second, per-entry dial
on top would be engineering for a user population of "the author and a few people
he knows personally", which SL-250's less-code posture rules out, and it would
have to be re-litigated the moment anyone disagreed about which bucket a spec
belongs in.

## Reversibility (the condition this was accepted under)

- **Code axis — cheap, verified.** `install_hook_to_file(root, rel_path, spec,
  dry_run)` (`src/boot.rs:1595`) already takes the target file per call.
  Splitting scopes later means passing a different `rel_path` at some call sites.
  No core change.
- **Operational axis — not free.** A later flip leaves the previously-written
  entries owned in the *abandoned* file, still firing beside the new ones,
  because the normalize works only within the single file it writes. That is
  `R10`, and the remedy is whatever `inq-5` settles.

## Documented, not engineered

- `PreToolUse` hooks **fail open** — only exit 2 blocks
  (`mem.fact.claude.pretooluse-hook-fail-open`), so a missing or erroring hook
  degrades silently rather than announcing itself.
- On a `Bash` tool call, two doctrine processes fire.

Recorded from design run `dr-019fd692` checkpoint `cp-2` disposing `inq-1`.
