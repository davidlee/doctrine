# SL-250 `VH-1` — cold install activates

Reader for `vh-1-cold-install.txt`, which is the raw capture and is verbatim by
`VH-1`'s own requirement. This file derives the count and states what the
capture does and does not evidence. It asserts nothing the raw file does not
show.

Captured 2026-08-08 by the human, in a scratch project, from a binary built off
this slice at `01b1c718b` (PHASE-06 tip).

## Preconditions

`VH-1` states five, so the run is reproducible rather than recounted. The
capture opens with them and the operator attests they held:

1. a scratch project — not this repo
2. no `enabledPlugins` entry for doctrine
3. no marketplace registration
4. the scope key **absent**, exercising the `Project` default rather than a set value
5. a binary built from this slice

Attested, not separately evidenced — the capture shows the *result* of those
conditions, not the conditions themselves. See *What this does not evidence*.

## The count: eleven, from Project settings

`VH-1` insists on eleven rather than seven because seven is the spec count and a
criterion written against it would pass a four-entries-short install. The
per-event drill-downs give eleven:

| event | matcher | hooks | scope |
|---|---|---|---|
| `PreToolUse` | `Agent` | 1 | `[Project]` |
| `PreToolUse` | `Bash` | 2 | `[Project]` |
| `PreToolUse` | `Edit\|Write` | 1 | `[Project]` |
| `PreToolUse` | `Read\|Edit\|Write` | 1 | `[Project]` |
| `PreToolUse` | `Workflow` | 1 | `[Project]` |
| `SessionStart` | `startup\|clear` | 2 | `[Project]` |
| `SubagentStart` | `dispatch-orchestrator` | 1 | `[Project]` |
| `SubagentStop` | `dispatch-orchestrator` | 1 | `[Project]` |
| `WorktreeCreate` | — | 1 | `Project Settings` |

**Eleven, across five events, every one scoped to project settings.** That
matches `plan.toml` `PHASE-04` `EX-4`'s distribution exactly — SessionStart 2,
WorktreeCreate 1, SubagentStart 1, SubagentStop 1, PreToolUse 6.

The scope label is the load-bearing part, not the count. Eleven entries of
unknown provenance would be consistent with the plugin still serving them; the
`[Project]` / `Project Settings` label is what says they came from
`.claude/settings.json`, which is the whole claim.

## The foreign-hook confound, reconciled

The summary menu reports **16 hooks configured** and shows event counts that do
not match doctrine's — notably `SessionStart (3)` where doctrine writes two. The
excess is foreign: the operator's user-scope hooks from `~/.claude/settings.json`
plus an unrelated plugin (`caveman`), all bleeding into the menu's totals, which
do not distinguish scope. The per-event drill-downs do.

It reconciles exactly:

```
doctrine (all [Project])   PreToolUse 6 + SessionStart 2 + SubagentStart 1
                           + SubagentStop 1 + WorktreeCreate 1        = 11
foreign (user + caveman)   SessionStart 1 + Notification 1
                           + UserPromptSubmit 1 + Stop 2              =  5
                                                                       ---
menu total                                                              16
```

No unaccounted entry, and no doctrine entry outside the eleven. The confound is
in the summary menu's presentation, not in the install.

**The arithmetic is also the completeness check on the capture itself.** The
operator transcribed the drill-downs by hand and could not be certain every
doctrine entry was pasted. It closing to the menu's own total settles that: had
a doctrine entry been omitted from the paste, 11 + 5 would not equal 16. The
transcript is complete for the eleven without needing to be trusted as complete.

Incidentally this is `PHASE-04`'s never-clobber property observed live —
foreign hooks on `SessionStart`, an event doctrine also writes, coexisting with
doctrine's own rather than being normalised away.

## Incidental confirmations

- **The portable command form is visible in the live UI.** The `WorktreeCreate`
  row renders `${DOCTRINE_BIN:-doctrin…` (truncated by the menu), so the tracked
  file carries the variable and not a host absolute path. That is SL-195's
  `INV-1` — no absolute host path in a tracked file — observed on the hooks
  surface for the first time (`PHASE-02` `VT-3` asserts it in test; this is it
  live).
- **The `PHASE-05` skills channel works end to end.** The capture's `tree` shows
  35 `.claude/skills/<id>` symlinks resolving to `../../.doctrine/skills/<id>`,
  with the canonical tree materialised under `.doctrine/skills/`. That is
  SPEC-010 responsibilities 3–4 observed on a real cold install rather than in a
  tempdir test.
- Agent defs and the workflow link landed on the same run.

## What this does not evidence

Named here so the audit does not read the capture as covering more than it does.

1. **The three observed effects per event class.** `VH-1` asks for a session
   boot emitting, a `WorktreeCreate` fork, and a `PreToolUse` surfacing — i.e.
   that the entries *fire*, not merely that they are registered. The capture
   shows registration only. `mem.fact.claude.reload-plugins-registers-pretooluse`
   is the reason this distinction is in the criterion at all: it records a case
   where `/hooks` reported hooks live and the `PreToolUse` wall did not fire.
   A count is not a firing.
2. **Preconditions 2 and 3 directly.** No `enabledPlugins` / marketplace state
   was captured. The eleven entries being `[Project]`-scoped is strong indirect
   evidence — a plugin-sourced hook renders differently — but it is inference,
   not the precondition shown.
3. **`.claude/settings.local.json`.** The tree shows the file exists in the
   scratch project. Its contents were not captured, so whether it holds doctrine
   entries (and therefore whether `PHASE-03`'s sweep had anything to evict, or
   correctly left a clean file alone) is unknown from this capture. Not a
   defect signal — at `Project` default with no prior local install there should
   be nothing to sweep — but it is untested here.

Item 1 is the substantive gap and is `VH-1`'s explicit second half. Items 2 and
3 are cheap to close on a re-run if the audit wants them.
