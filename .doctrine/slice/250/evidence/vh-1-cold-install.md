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

Preconditions 2 and 3 were established by an affirmative act, not merely
observed absent: the operator **removed the doctrine marketplace from their
outside-jail Claude config before the run** (2026-08-08).

That is the strongest available form of the precondition, and it is worth
spelling out why. `known_marketplaces.json` is the registry Claude Code actually
consults at load — `mem.system.claude.plugin-load-model`, verified empirically
on CC 2.1.198. An `enabledPlugins` entry whose marketplace is unregistered is
skipped as an orphan, **silently**, and the plugin is simply absent. So with the
marketplace deregistered, the plugin path was not just unused during this run;
it was unavailable, whether or not a stale `enabledPlugins` entry survived
anywhere. The eleven entries cannot have come from it.

Preconditions 1, 4 and 5 are operator-attested rather than separately captured.

**This is per-user state, outside the repo.** The deregistration happened in the
operator's host config, which no artefact here records and no auditor can
re-derive. That is precisely why `VH-1` demands the preconditions be transcribed
rather than recounted, and why this paragraph exists.

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

## The abandoned scope is clean

The scratch project's `.claude/settings.local.json`, captured in full:

```json
{
  "enabledMcpjsonServers": [
    "doctrine"
  ]
}
```

Three facts fall out, all of them `PHASE-02` / `PHASE-03` claims:

1. **The `Project` default landed everything in `.claude/settings.json`.** No
   `hooks` key here at all — doctrine wrote none of its eleven entries to the
   local file. That is the scope dial working with the key absent.
2. **No `worktree.baseRef` stranded here.** `EX-4`'s "one Claude settings file
   per install, not two" holds: the baseRef followed the hooks into the project
   file rather than being left behind in the sibling.
3. **The sweep correctly did nothing.** With no prior local install there was
   nothing doctrine-owned to evict, and `PHASE-03`'s guard means that is
   `EvictOutcome::Nothing`, not a failure. The one key present is foreign —
   Claude Code wrote `enabledMcpjsonServers` itself when the operator approved
   the MCP server at session start, visible at the head of the raw capture — and
   it survives untouched.

Fact 3 is weak evidence of never-clobber on its own (an empty sweep clobbers
nothing by construction). It is listed because its *absence* would have been
strong evidence against.

## Observed effects — 1 of 3

`VH-1` asks for one observed effect per event class, because registration is not
firing. `mem.fact.claude.reload-plugins-registers-pretooluse` is why the
distinction is in the criterion: it records a case where `/hooks` reported hooks
live and the `PreToolUse` wall did not fire, twice, with a logging shim
confirming zero interception.

### `SessionStart` — CONFIRMED, behaviourally

In a fresh session in the scratch project (2026-08-08), the operator asked for a
React component. The agent declined to build it directly and answered:

> Repo is doctrine-governed. Fart button = code-changing intent → doctrine says
> /route → /slice → design → plan → execute. Heavy for one component. Your call.

then offered a ceremony choice whose "skip doctrine" option was labelled
*"Violates repo CLAUDE.md routing gate — but you're authorising the bypass"*.

That content — the routing gate, the ordered lifecycle chain, the injunction
against code without an approved plan — exists only in the boot snapshot
(`.doctrine/state/boot.md`), which the capture's tree confirms present.

**Why this isolates the hook.** Doctrine has two channels for the boot sector,
and only one of them serves Claude. The `AGENTS.md` @-import is the **codex**
arm's mechanism — `src/install.rs:268` says so in terms: *"`pi` rides the codex
arm (the `AGENTS.md` @-import and the `.pi/` extensions are installed there)"*.
The claude arm's boot surface is the `SessionStart` hook and nothing else. So in
a claude-installed scratch project there is no import path, and the snapshot
reached that session's context through the hook.

**And the operator watched it happen.** An @-import resolving would have
surfaced as a visible read in the session; none did. That closes the one hole
the code argument leaves — a hand-placed `CLAUDE.md` at the scratch project root
would be outside doctrine's knowledge, but not outside the operator's. Direct
observation of the session, not inference from the installer.

This is stronger than a log line. The payload did not merely arrive — the agent
*acted* on it, refusing a bypass it would otherwise have had no reason to
refuse. A hook that registered but did not fire produces an agent that cheerfully
writes the component.

### `WorktreeCreate` — CONFIRMED

The load-bearing one: `isolation: worktree` teardown is *conditional* on this
hook firing (`mem_019f1a5ce1f472219da91d0724bb766b`), so an inert entry changes
dispatch's semantics silently rather than degrading them.

Probe run 2026-08-08 in `/tmp/install_test` (git repo, HEAD `21b9d75`, clean
tree, manual permission mode). One `isolation: worktree` subagent spawned.

| | value |
|---|---|
| subagent `pwd` | `/tmp/install_test/.worktrees/agent-a7f941dff23e28c41` |
| main session `pwd` | `/tmp/install_test` |
| harness metadata | `worktreePath` + `agentId` `a7f941dff23e28c41` |
| Agent-tool error / fallback | none reported |

Main session `git worktree list`, run **after** the subagent returned (the
positive control — the comparison is against a live value, not an assumption):

```
/tmp/install_test                                    21b9d75 [main]
/tmp/install_test/.worktrees/agent-a7f941dff23e28c41 21b9d75 (detached HEAD)
```

**Why this is doctrine's hook and not the harness's own worktree support.**
`.worktrees/<name>` is doctrine's layout, not a generic one:
`WORKTREES_SUBDIR` in `src/worktree/create.rs`, with
`src/worktree/dispatch_record.rs:301` naming `create-fork` *"THE one owner of the
`.worktrees/<name>` layout, in both directions"*. The harness supplies a
suggested slug and consumes the hook's stdout as the path; the path that came
back is doctrine's. A registered-but-inert hook would have produced either an
in-situ subagent or a harness-native path elsewhere.

### `PreToolUse` — outstanding

Six entries across five matchers, the largest block. Trip any matched tool and
confirm the memory-surface or worktree wall reports.

## A trap the probe surfaced, which is not a defect in this slice

The subagent reported all three of its `git` commands failing identically:

```
fatal: not a git repository: (null)   [exit 128]
```

and diagnosed it as *"the parent repository is genuinely gone … confirmed with
the sandbox disabled, so it is not a sandbox filesystem-view artifact"*.

**That diagnosis is wrong**, and the operator refuted it from the main session:
`/tmp/install_test/.git` was present throughout, along with `CLAUDE.md`,
`.doctrine/` and the rest. The real cause is the linked worktree's `.git` *file*
holding a `gitdir:` pointer out to `/tmp/install_test/.git/worktrees/<name>`,
which the subagent's sandboxed filesystem view — scoped to its own worktree —
could not follow. Git reports the unresolvable pointer as `(null)`. The
subagent's "confirmed with the sandbox disabled" claim did not hold.

Recorded because it will recur for every dispatch worker that runs `git` inside
a fork, and because the failure *presents* as catastrophic repo loss. It says
nothing about `WorktreeCreate` firing — the fork demonstrably exists and is
registered in the parent. Operator's note: worktrees behave oddly when the repo
sits outside the usual workspace path, as this scratch project does at `/tmp`.

### `PreToolUse` — outstanding

Six entries across five matchers, the largest block. Trip any matched tool and
confirm the memory-surface or worktree wall reports.
